#!/usr/bin/env python3
"""
Remove file list entries with invalid UTF-8 names from a Centauri Carbon (CC1)
running stock firmware, over the network. No root access needed.

Such an entry makes the SDCP file list response (Cmd 258) invalid UTF-8, so
browsers drop the web UI's websocket: no files, no history, no camera.

For every broken entry the script uploads a tiny placeholder file with exactly
the same (broken) name and then deletes it with Cmd 259, which also makes the
printer drop the entry from its file list.

Usage: python3 fix_file_list_utf8.py <printer-ip> [--dry-run]
Only the Python 3 standard library is needed.
"""
import base64
import hashlib
import http.client
import os
import re
import socket
import struct
import sys
import time
import uuid

WS_PORT = 3030
HTTP_PORT = 80


def ws_connect(ip):
    sock = socket.create_connection((ip, WS_PORT), timeout=10)
    key = base64.b64encode(os.urandom(16)).decode()
    sock.sendall(
        (
            f"GET /websocket HTTP/1.1\r\nHost: {ip}:{WS_PORT}\r\n"
            "Upgrade: websocket\r\nConnection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        ).encode()
    )
    buf = b""
    while b"\r\n\r\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk:
            raise ConnectionError("websocket handshake failed")
        buf += chunk
    head, rest = buf.split(b"\r\n\r\n", 1)
    if b" 101 " not in head.split(b"\r\n", 1)[0]:
        raise ConnectionError(head.decode(errors="replace"))
    return sock, rest


def ws_send(sock, payload):
    # Text frame with raw bytes: the payload may contain non-UTF-8 bytes on
    # purpose, so it is built by hand instead of with a websocket library.
    mask = os.urandom(4)
    n = len(payload)
    if n < 126:
        header = bytes([0x81, 0x80 | n])
    elif n < 65536:
        header = bytes([0x81, 0x80 | 126]) + struct.pack(">H", n)
    else:
        header = bytes([0x81, 0x80 | 127]) + struct.pack(">Q", n)
    sock.sendall(header + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(payload)))


def ws_frames(sock, buf, timeout):
    """Collect frame payloads for `timeout` seconds (no UTF-8 validation)."""
    frames, end = [], time.time() + timeout
    sock.settimeout(0.5)
    while time.time() < end:
        try:
            chunk = sock.recv(65536)
            if not chunk:
                break
            buf += chunk
        except socket.timeout:
            pass
        while len(buf) >= 2:
            length, off = buf[1] & 0x7F, 2
            if length == 126:
                if len(buf) < 4:
                    break
                length, off = struct.unpack(">H", buf[2:4])[0], 4
            elif length == 127:
                if len(buf) < 10:
                    break
                length, off = struct.unpack(">Q", buf[2:10])[0], 10
            if len(buf) < off + length:
                break
            frames.append(buf[off:off + length])
            buf = buf[off + length:]
    return frames


def sdcp(ip, cmd, data, timeout=4.0, match=None):
    """Send one SDCP command; `data` is raw JSON bytes.

    Returns the first frame containing `match` (default: the command's response).
    """
    match = match or b'"Cmd":%d,' % cmd
    sock, buf = ws_connect(ip)
    try:
        ws_send(
            sock,
            b'{"Id":"","Data":{"Cmd":%d,"Data":%s,"RequestID":"%s","MainboardID":"",'
            b'"TimeStamp":%d,"From":0},"Topic":"sdcp/request/"}'
            % (cmd, data, uuid.uuid4().hex.encode(), int(time.time())),
        )
        for frame in ws_frames(sock, buf, timeout):
            if match in frame:
                return frame
    finally:
        sock.close()
    raise TimeoutError(f"no response to Cmd {cmd}")


def printer_idle(ip):
    # Uploads are refused while printing (error -4), and nothing here should
    # touch storage mid-print anyway.
    status = sdcp(ip, 0, b"{}", match=b'"CurrentStatus"')
    m = re.search(rb'"CurrentStatus":\s*\[([^\]]*)\]', status)
    return m is not None and m.group(1).strip() == b"0"


def json_unescape(raw):
    """Turn a JSON-escaped name (as the printer sends it) into raw filename bytes."""
    def repl(m):
        esc = m.group(1)
        if esc.startswith(b"u"):
            return chr(int(esc[1:], 16)).encode()
        return {b'"': b'"', b"\\": b"\\", b"/": b"/", b"b": b"\b",
                b"f": b"\f", b"n": b"\n", b"r": b"\r", b"t": b"\t"}[esc]
    return re.sub(rb'\\(u[0-9a-fA-F]{4}|["\\/bfnrt])', repl, raw)


def wait_until(check, timeout=15.0, interval=1.0):
    end = time.time() + timeout
    while True:
        if check():
            return True
        if time.time() >= end:
            return False
        time.sleep(interval)


def file_known(ip, name):
    # Cmd 260 (file details) answers Ack 0 only once the printer has the file.
    return b'"Ack":0' in sdcp(ip, 260, b'{"Url":"' + name + b'"}')


def entry_present(ip, name):
    return b'"name":"' + name + b'"' in sdcp(ip, 258, b'{"Url":"/local"}')


def broken_entries(ip):
    resp = sdcp(ip, 258, b'{"Url":"/local"}')
    names = re.findall(rb'"name":"((?:[^"\\]|\\.)*)"', resp)
    broken = []
    for name in names:
        try:
            name.decode("utf-8")
        except UnicodeDecodeError:
            broken.append(name)
    return len(names), broken


def upload_placeholder(ip, filename):
    data = b"; placeholder, deleted right away by fix_file_list_utf8.py\n"
    boundary = b"----" + uuid.uuid4().hex.encode()

    def field(name, value):
        return (b"--" + boundary + b'\r\nContent-Disposition: form-data; name="'
                + name + b'"\r\n\r\n' + value + b"\r\n")

    body = (
        field(b"TotalSize", str(len(data)).encode())
        + field(b"Uuid", uuid.uuid4().hex.encode())
        + field(b"Offset", b"0")
        + field(b"Check", b"1")
        + field(b"S-File-MD5", hashlib.md5(data).hexdigest().encode())
        + b"--" + boundary + b'\r\nContent-Disposition: form-data; name="File"; filename="'
        + filename + b'"\r\nContent-Type: application/octet-stream\r\n\r\n'
        + data + b"\r\n--" + boundary + b"--\r\n"
    )
    conn = http.client.HTTPConnection(ip, HTTP_PORT, timeout=15)
    conn.putrequest("POST", "/uploadFile/upload")
    conn.putheader("Content-Type", "multipart/form-data; boundary=" + boundary.decode())
    conn.putheader("Content-Length", str(len(body)))
    conn.endheaders()
    conn.send(body)
    resp = conn.getresponse()
    text = resp.read()
    if resp.status != 200 or b'"success":true' not in text:
        raise RuntimeError(f"upload failed: {resp.status} {text[:200]!r}")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    ip, dry_run = sys.argv[1], "--dry-run" in sys.argv[2:]

    total, broken = broken_entries(ip)
    print(f"{total} entries in /local, {len(broken)} with an invalid UTF-8 name")
    for name in broken:
        print("  ", name)
    if not broken or dry_run:
        return
    if not printer_idle(ip):
        sys.exit("The printer is busy (printing?). Run this again when it is idle.")

    for name in broken:
        raw = json_unescape(name)
        if not raw.startswith(b"/local/") or b"/" in raw[len(b"/local/"):]:
            print("skipping (not a plain file in /local):", name)
            continue
        print("fixing", name)
        upload_placeholder(ip, raw[len(b"/local/"):])
        # The printer processes an upload for a moment; a delete sent right
        # away is acknowledged but ignored.
        if not wait_until(lambda: file_known(ip, name)):
            print("   the printer did not pick up the placeholder, skipping")
            continue
        for attempt in range(1, 4):
            sdcp(ip, 259, b'{"FileList":["' + name + b'"],"FolderList":[]}')
            if wait_until(lambda: not entry_present(ip, name), timeout=10):
                print(f"   removed (delete attempt {attempt})")
                break
        else:
            print("   still listed after 3 attempts")

    total, broken = broken_entries(ip)
    print(f"after: {total} entries, {len(broken)} with an invalid UTF-8 name")
    sys.exit(1 if broken else 0)


if __name__ == "__main__":
    main()
