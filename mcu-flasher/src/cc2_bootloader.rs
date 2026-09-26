// Protocol from src/bootloader_com.c in https://github.com/elegooofficial/CentauriCarbon2

use std::{
    fmt, io,
    time::{Duration, Instant},
};

use serialport::SerialPort;

const FRAME_MAGIC: [u8; 2] = [0xa5, 0x5a];
// magic(2) + cmd(1) + len(2) + crc(2)
const FRAME_OVERHEAD: usize = 7;

const CMD_ERASE: u8 = 0x00;
const CMD_PROGRAM: u8 = 0x01;
const CMD_JUMP_TO_APP: u8 = 0x02;
const CMD_PING: u8 = 0x03;

const PING_VALUE: u32 = 0x1234_5678;
// BOOTLOADER_COM_MAX_PAYLOAD
const MAX_PAYLOAD_LENGTH: usize = 4096;

// Firmware bytes per PROGRAM command. The MCU's receive ringbuffer is 1024
// bytes and we wait for each ack before sending more, so this stays well
// inside it even while a flash write stalls the parse task.
const CHUNK_SIZE: usize = 512;

// A ping only defers the MCU's pending auto-jump by 50ms, so we have to ping
// faster than that to hold it in the bootloader.
const PING_INTERVAL: Duration = Duration::from_millis(20);
const READ_POLL: Duration = Duration::from_millis(5);
const ERASE_TIMEOUT: Duration = Duration::from_secs(5);
const PROGRAM_TIMEOUT: Duration = Duration::from_secs(2);

// The MCU arms its independent watchdog with a ~410ms timeout and the stock
// ERASE handler erases every page in one loop without feeding it, so erasing
// a whole programmed app region resets the MCU before it can ack.
//
// ERASE always starts at the application base and takes only a length, so
// walk the length up one page at a time: each request re-erases the pages
// already blanked, which is fast, plus at most one programmed page.
const ERASE_STEP: u32 = 2048;

#[derive(Debug)]
pub enum Error {
    Serial(serialport::Error),
    Io(io::Error),
    NotInBootloader,
    NoResponse { command: u8 },
    EraseRejected { requested: u32, erased: u32 },
    ProgramRejected { offset: u32, written: u32 },
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Serial(err) => write!(f, "serial error: {}", err),
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::NotInBootloader => write!(f, "no response from the bootloader"),
            Error::NoResponse { command } => {
                write!(f, "no response to command {:#04x}", command)
            }
            Error::EraseRejected { requested, erased } => write!(
                f,
                "erase rejected: asked for {} bytes, MCU erased {}",
                requested, erased
            ),
            Error::ProgramRejected { offset, written } => write!(
                f,
                "program rejected at offset {:#x}: MCU wrote {} bytes",
                offset, written
            ),
        }
    }
}

impl From<serialport::Error> for Error {
    fn from(err: serialport::Error) -> Error {
        Error::Serial(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

/// Catch the board in its bootloader and write `image` to the application
/// partition, then start it.
///
/// The bootloader only waits 200ms after reset before jumping to the
/// application, so the board has to be reset while `window` is running. Each
/// ping holds it for another 50ms, until the first ERASE cancels the jump.
pub fn deploy(port: &mut dyn SerialPort, image: &[u8], window: Duration) -> Result<()> {
    port.set_timeout(READ_POLL)?;
    if !enter_bootloader(port, window)? {
        return Err(Error::NotInBootloader);
    }
    // Must reach ERASE within 50ms of the last pong or the MCU jumps anyway.
    flash(port, image)?;
    jump_to_app(port)
}

/// Ping until the bootloader answers, or `window` elapses.
fn enter_bootloader(port: &mut dyn SerialPort, window: Duration) -> Result<bool> {
    let deadline = Instant::now() + window;

    while Instant::now() < deadline {
        let payload = transact(port, CMD_PING, &PING_VALUE.to_le_bytes(), PING_INTERVAL)?;
        if let Some(payload) = payload {
            if read_u32(&payload, 0) == Some(PING_VALUE) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Erase and program the application partition.
fn flash(port: &mut dyn SerialPort, image: &[u8]) -> Result<()> {
    // The MCU programs whole 32-bit words and mishandles a shorter tail, so
    // pad to a word with the erased value.
    let mut image = image.to_vec();
    while image.len() % 4 != 0 {
        image.push(0xff);
    }
    let size = image.len() as u32;

    let mut erased = 0;
    let mut target = 0;
    while target < size {
        target = (target + ERASE_STEP).min(size);

        let ack = require(
            transact(port, CMD_ERASE, &target.to_le_bytes(), ERASE_TIMEOUT)?,
            CMD_ERASE,
        )?;
        erased = read_u32(&ack, 0).unwrap_or(0);
        if erased < target {
            return Err(Error::EraseRejected {
                requested: target,
                erased,
            });
        }
    }
    println!("Erased {} bytes", erased);

    let total = (image.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    for (index, chunk) in image.chunks(CHUNK_SIZE).enumerate() {
        let offset = (index * CHUNK_SIZE) as u32;
        let length = chunk.len() as u32;

        println!("Sending block {}/{}", index + 1, total);

        // bootloader_cmd_program_req_t { offset, length, size } + data
        let mut payload = Vec::with_capacity(12 + chunk.len());
        payload.extend_from_slice(&offset.to_le_bytes());
        payload.extend_from_slice(&length.to_le_bytes());
        payload.extend_from_slice(&size.to_le_bytes());
        payload.extend_from_slice(chunk);

        let ack = require(
            transact(port, CMD_PROGRAM, &payload, PROGRAM_TIMEOUT)?,
            CMD_PROGRAM,
        )?;
        // bootloader_cmd_program_ack_t { length, offset }
        let written = read_u32(&ack, 0).unwrap_or(0);
        let next = read_u32(&ack, 4).unwrap_or(0);
        if written != length || next != offset + length {
            return Err(Error::ProgramRejected { offset, written });
        }
    }

    Ok(())
}

/// JUMP is not acknowledged - the MCU jumps ~1ms later.
fn jump_to_app(port: &mut dyn SerialPort) -> Result<()> {
    write_packet(port, CMD_JUMP_TO_APP, &[])
}

fn require(payload: Option<Vec<u8>>, command: u8) -> Result<Vec<u8>> {
    payload.ok_or(Error::NoResponse { command })
}

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().unwrap()))
}

/// Send a command and wait for the matching response. The MCU echoes the
/// request's command id in its ack; there is no NACK, so a rejected command
/// is indistinguishable from silence.
fn transact(
    port: &mut dyn SerialPort,
    command: u8,
    payload: &[u8],
    timeout: Duration,
) -> Result<Option<Vec<u8>>> {
    // Anything already buffered predates this request, so it cannot be the
    // response.
    let _ = port.clear(serialport::ClearBuffer::Input);
    write_packet(port, command, payload)?;

    let deadline = Instant::now() + timeout;
    let mut received = Vec::new();
    let mut chunk = [0u8; 256];

    loop {
        while let Some((cmd, payload, end)) = find_frame(&received) {
            received.drain(..end);
            if cmd == command {
                return Ok(Some(payload));
            }
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        match port.read(&mut chunk) {
            Ok(count) => received.extend_from_slice(&chunk[..count]),
            Err(err) if err.kind() == io::ErrorKind::TimedOut => {}
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
            Err(err) => return Err(err.into()),
        }
    }
}

fn write_packet(port: &mut dyn SerialPort, command: u8, payload: &[u8]) -> Result<()> {
    port.write_all(&packet(command, payload))?;
    port.flush()?;
    Ok(())
}

fn packet(command: u8, payload: &[u8]) -> Vec<u8> {
    let length = u16::try_from(payload.len()).expect("CC2 payload is too large");
    let mut packet = Vec::with_capacity(FRAME_OVERHEAD + payload.len());
    packet.extend_from_slice(&FRAME_MAGIC);
    packet.push(command);
    packet.extend_from_slice(&length.to_be_bytes());
    packet.extend_from_slice(payload);
    packet.extend_from_slice(&calc_crc(payload).to_be_bytes());
    packet
}

/// Find the first complete, CRC-valid frame in `data`.
///
/// Returns the command, its payload, and how many bytes to drain. Skips past
/// anything that looks like a header but fails to check out - the magic can
/// legitimately appear inside a payload.
fn find_frame(data: &[u8]) -> Option<(u8, Vec<u8>, usize)> {
    let mut start = 0;

    while start + FRAME_OVERHEAD <= data.len() {
        if data[start..start + 2] != FRAME_MAGIC {
            start += 1;
            continue;
        }

        let command = data[start + 2];
        let length = u16::from_be_bytes([data[start + 3], data[start + 4]]) as usize;
        if length > MAX_PAYLOAD_LENGTH {
            start += 1;
            continue;
        }

        let end = start + 5 + length + 2;
        if end > data.len() {
            // Either a real frame still arriving or a false header. Keep
            // scanning; a real one is found on a later call.
            start += 1;
            continue;
        }

        let payload = &data[start + 5..start + 5 + length];
        let crc = u16::from_be_bytes([data[end - 2], data[end - 1]]);
        if calc_crc(payload) != crc {
            start += 1;
            continue;
        }

        return Some((command, payload.to_vec(), end));
    }

    None
}

fn calc_crc(data: &[u8]) -> u16 {
    crc16::State::<crc16::MCRF4XX>::calculate(data)
}
