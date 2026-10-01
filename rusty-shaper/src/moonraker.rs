//! Tiny plain-HTTP Moonraker client for sending g-code commands to Klipper/Kalico.
//!
//! Moonraker on the printer is local/plain HTTP. Avoid pulling in a full HTTP
//! client/TLS stack here: the ARMv7 musl binary previously segfaulted at startup
//! when linked with `minreq`, before `main()` or clap parsing ran.

use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use thiserror::Error;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RESPONSE_BYTES: u64 = 1 << 20; // 1 MiB

#[derive(Error, Debug)]
pub enum MoonrakerError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Klippy not ready: {0}")]
    KlippyNotReady(String),
    #[error("Command rejected: {0}")]
    CommandRejected(String),
    #[error("Moonraker error: {0}")]
    ApiError(String),
}

/// Send a g-code script to Moonraker's /printer/gcode/script endpoint.
pub fn send_gcode_script(ip: &str, port: u16, script: &str) -> Result<(), MoonrakerError> {
    let body = serde_json::json!({"script": script});
    let status = post_json(ip, port, "/printer/gcode/script", &body)?;
    check_moonraker_error(&status)?;

    let result = status.get("result").and_then(|r| r.as_str()).unwrap_or("");
    if result != "ok" {
        return Err(MoonrakerError::ApiError(format!(
            "Unexpected response: {}",
            result
        )));
    }

    Ok(())
}

/// Query the input_shaper object state from Klippy.
pub fn query_input_shaper(ip: &str, port: u16) -> Result<serde_json::Value, MoonrakerError> {
    let body = serde_json::json!({"objects": {"input_shaper": null}});
    let status = post_json(ip, port, "/printer/objects/query", &body)?;
    check_moonraker_error(&status)?;
    Ok(status)
}

fn post_json(
    host: &str,
    port: u16,
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, MoonrakerError> {
    let body_text = body.to_string();
    let request = build_http_post_request(host, port, path, &body_text);

    let mut stream = TcpStream::connect((host, port))
        .map_err(|e| MoonrakerError::Connection(format!("{host}:{port}: {e}")))?;
    stream
        .set_read_timeout(Some(HTTP_TIMEOUT))
        .map_err(|e| MoonrakerError::Connection(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(HTTP_TIMEOUT))
        .map_err(|e| MoonrakerError::Connection(format!("set write timeout: {e}")))?;

    stream
        .write_all(request.as_bytes())
        .map_err(|e| MoonrakerError::Connection(format!("write request: {e}")))?;

    let mut response = Vec::new();
    stream
        .take(MAX_RESPONSE_BYTES)
        .read_to_end(&mut response)
        .map_err(|e| MoonrakerError::Connection(format!("read response: {e}")))?;

    let response_text = String::from_utf8(response)
        .map_err(|e| MoonrakerError::ApiError(format!("Non-UTF8 HTTP response: {e}")))?;

    parse_http_json_response(&response_text)
}

fn build_http_post_request(host: &str, port: u16, path: &str, body: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         User-Agent: rusty-shaper/0.1\r\n\
         Content-Type: application/json\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {body}",
        body.len()
    )
}

fn parse_http_json_response(response: &str) -> Result<serde_json::Value, MoonrakerError> {
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .or_else(|| response.split_once("\n\n"))
        .ok_or_else(|| MoonrakerError::ApiError("Malformed HTTP response".to_string()))?;

    let status_line = headers
        .lines()
        .next()
        .ok_or_else(|| MoonrakerError::ApiError("Missing HTTP status line".to_string()))?;
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| MoonrakerError::ApiError(format!("Malformed status line: {status_line}")))?
        .parse()
        .map_err(|e| MoonrakerError::ApiError(format!("Invalid HTTP status code: {e}")))?;

    let body_text = if has_chunked_transfer_encoding(headers) {
        decode_chunked_body(body)?
    } else {
        body.to_string()
    };

    let json: serde_json::Value = serde_json::from_str(body_text.trim())
        .map_err(|e| MoonrakerError::ApiError(format!("Invalid JSON response: {e}")))?;

    if !(200..300).contains(&status_code) && json.get("error").is_none() {
        return Err(MoonrakerError::ApiError(format!(
            "HTTP {status_code}: {}",
            body_text.trim()
        )));
    }

    Ok(json)
}

fn has_chunked_transfer_encoding(headers: &str) -> bool {
    headers.lines().any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        name.trim().eq_ignore_ascii_case("transfer-encoding")
            && value
                .split(',')
                .any(|encoding| encoding.trim().eq_ignore_ascii_case("chunked"))
    })
}

fn decode_chunked_body(body: &str) -> Result<String, MoonrakerError> {
    let mut rest = body.as_bytes();
    let mut decoded = Vec::new();

    loop {
        let Some(line_end) = find_crlf(rest) else {
            return Err(MoonrakerError::ApiError(
                "Malformed chunked response: missing chunk size".to_string(),
            ));
        };
        let size_line = std::str::from_utf8(&rest[..line_end])
            .map_err(|e| MoonrakerError::ApiError(format!("Invalid chunk header: {e}")))?;
        let size_hex = size_line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|e| MoonrakerError::ApiError(format!("Invalid chunk size: {e}")))?;
        rest = &rest[line_end + 2..];

        if size == 0 {
            break;
        }
        if rest.len() < size + 2 {
            return Err(MoonrakerError::ApiError(
                "Malformed chunked response: truncated chunk".to_string(),
            ));
        }
        if &rest[size..size + 2] != b"\r\n" {
            return Err(MoonrakerError::ApiError(
                "Malformed chunked response: missing chunk terminator".to_string(),
            ));
        }

        decoded.extend_from_slice(&rest[..size]);
        rest = &rest[size + 2..];
    }

    String::from_utf8(decoded)
        .map_err(|e| MoonrakerError::ApiError(format!("Non-UTF8 chunked body: {e}")))
}

fn find_crlf(bytes: &[u8]) -> Option<usize> {
    bytes.windows(2).position(|w| w == b"\r\n")
}

fn check_moonraker_error(json: &serde_json::Value) -> Result<(), MoonrakerError> {
    if let Some(error) = json.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        let code = error.get("code").and_then(|c| c.as_u64()).unwrap_or(0) as u16;

        return Err(match code {
            503 => MoonrakerError::KlippyNotReady(msg.to_string()),
            400 => MoonrakerError::CommandRejected(msg.to_string()),
            _ => MoonrakerError::ApiError(msg.to_string()),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn test_build_http_post_request() {
        let request = build_http_post_request(
            "127.0.0.1",
            7125,
            "/printer/gcode/script",
            "{\"script\":\"M112\"}",
        );

        assert!(request.starts_with("POST /printer/gcode/script HTTP/1.1\r\n"));
        assert!(request.contains("Host: 127.0.0.1:7125\r\n"));
        assert!(request.contains("Content-Type: application/json\r\n"));
        assert!(request.contains("Connection: close\r\n"));
        assert!(request.contains("Content-Length: 17\r\n"));
        assert!(request.ends_with("{\"script\":\"M112\"}"));
    }

    #[test]
    fn test_parse_http_json_response_ok() {
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 15\r\n\r\n{\"result\":\"ok\"}";
        let json = parse_http_json_response(response).unwrap();
        assert_eq!(json.get("result").and_then(|r| r.as_str()), Some("ok"));
    }

    #[test]
    fn test_parse_http_json_response_chunked() {
        let response = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nF\r\n{\"result\":\"ok\"}\r\n0\r\n\r\n";
        let json = parse_http_json_response(response).unwrap();
        assert_eq!(json.get("result").and_then(|r| r.as_str()), Some("ok"));
    }

    #[test]
    fn test_chunked_transfer_encoding_header_variants() {
        assert!(has_chunked_transfer_encoding(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip, chunked\r\n"
        ));
        assert!(has_chunked_transfer_encoding(
            "HTTP/1.1 200 OK\r\ntransfer-encoding: Chunked\r\n"
        ));
        assert!(!has_chunked_transfer_encoding(
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n"
        ));
    }

    #[test]
    fn test_decode_chunked_body_rejects_missing_chunk_terminator() {
        let err = decode_chunked_body("F\r\n{\"result\":\"ok\"}XX0\r\n\r\n").unwrap_err();
        assert!(err.to_string().contains("chunk terminator"));
    }

    #[test]
    fn test_send_gcode_script_posts_expected_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);

            assert!(request.starts_with("POST /printer/gcode/script HTTP/1.1"));
            assert!(request.contains("\"script\":\"SET_INPUT_SHAPER SHAPER_TYPE_X=MZV\""));

            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"result\":\"ok\"}")
                .unwrap();
        });

        send_gcode_script("127.0.0.1", port, "SET_INPUT_SHAPER SHAPER_TYPE_X=MZV").unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_send_gcode_script_maps_command_rejection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf).unwrap();
            let body = b"{\"error\":{\"code\":400,\"message\":\"bad gcode\"}}";
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                std::str::from_utf8(body).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let err = send_gcode_script("127.0.0.1", port, "BAD").unwrap_err();
        assert!(matches!(err, MoonrakerError::CommandRejected(_)));
        handle.join().unwrap();
    }

    #[test]
    fn test_query_input_shaper_returns_json() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);

            assert!(request.starts_with("POST /printer/objects/query HTTP/1.1"));
            assert!(request.contains("input_shaper"));

            let body = b"{\"result\":{\"status\":{\"input_shaper\":{\"shaper_type_x\":\"mzv\"}}}}";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                std::str::from_utf8(body).unwrap()
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let json = query_input_shaper("127.0.0.1", port).unwrap();
        assert_eq!(
            json.pointer("/result/status/input_shaper/shaper_type_x")
                .and_then(|v| v.as_str()),
            Some("mzv")
        );
        handle.join().unwrap();
    }
}
