// Heavily based on https://github.com/TGMM/xymodem.rs

use std::io::{self, Read, Write};
use log::{info, warn, log};

pub fn calc_crc(data: &[u8]) -> u16 {
    crc16::State::<crc16::XMODEM>::calculate(data)
}

pub fn get_byte<R: Read>(reader: &mut R) -> std::io::Result<u8> {
    let mut buff = [0];
    (reader.read_exact(&mut buff))?;
    Ok(buff[0])
}

/// Turns timeout errors into `Ok(None)`
pub fn get_byte_timeout<R: Read>(reader: &mut R) -> std::io::Result<Option<u8>> {
    match get_byte(reader) {
        Ok(c) => Ok(Some(c)),
        Err(err) => {
            if err.kind() == io::ErrorKind::TimedOut {
                Ok(None)
            } else {
                Err(err)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),

    /// The number of communications errors exceeded `max_errors` in a single
    /// transmission.
    ExhaustedRetries,

    /// The transmission was canceled by the other end of the channel.
    Canceled,
}

// TODO: Send CAN byte after too many errors
// TODO: Handle CAN bytes while sending
// TODO: Implement Error for Error

const SOH: u8 = 0x01;
const STX: u8 = 0x02;
const EOT: u8 = 0x04;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const CAN: u8 = 0x18;
const CRC: u8 = 0x43;

pub type Result<T> = std::result::Result<T, Error>;

/// Configuration for the YMODEM transfer.
#[derive(Copy, Clone, Debug)]
pub struct Ymodem {
    /// The number of errors that can occur before the communication is
    /// considered a failure. Errors include unexpected bytes and timeouts waiting for bytes.
    pub max_errors: u32,

    /// The number of errors that can occur before the communication is
    /// considered a failure. Errors include unexpected bytes and timeouts waiting for bytes.
    ///
    /// This only applies to the initial packet
    pub max_initial_errors: u32,

    /// The byte used to pad the last block. YMODEM can only send blocks of a certain size,
    /// so if the message is not a multiple of that size the last block needs to be padded.
    pub pad_byte: u8,

    /// Ignores all non-digit characters on the file_size string
    /// in the start frame (Ex. 12345V becomes 12345)
    pub ignore_non_digits_on_file_size: bool,

    errors: u32,
    initial_errors: u32,
}

impl Ymodem {
    /// Creates the YMODEM config with default parameters.
    pub fn new() -> Self {
        // Ymodem doesn't support 128 byte packages
        // or regular checksum
        Ymodem {
            max_errors: 16,
            max_initial_errors: 16,
            pad_byte: 0x1a,
            errors: 0,
            initial_errors: 0,
            ignore_non_digits_on_file_size: false,
        }
    }

    /// Starts the YMODEM transmission.
    ///
    /// `dev` should be the serial communication channel (e.g. the serial device).
    /// `stream` should be the message to send (e.g. a file).
    ///
    /// # Timeouts
    /// This method has no way of setting the timeout of `dev`, so it's up to the caller
    /// to set the timeout of the device before calling this method. Timeouts on receiving
    /// bytes will be counted against `max_errors`, but timeouts on transmitting bytes
    /// will be considered a fatal error.
    pub fn send<D: Read + Write, R: Read>(
        &mut self,
        dev: &mut D,
        stream: &mut R,
        file_name: String,
        file_size_in_bytes: u64,
    ) -> Result<()> {
        self.errors = 0;
        let packets_to_send = f64::ceil(file_size_in_bytes as f64 / 1024.0) as u32;
        let last_packet_size = file_size_in_bytes % 1024;

        dbg!("Starting YMODEM transfer");
        (self.start_send(dev))?;
        dbg!("First byte received. Sending start frame.");
        (self.send_start_frame(dev, file_name, file_size_in_bytes, packets_to_send))?;
        dbg!("Start frame acknowledged. Sending stream.");
        (self.send_stream(dev, stream, packets_to_send, last_packet_size))?;
        dbg!("Sending EOT");
        (self.finish_send(dev))?;

        Ok(())
    }

    fn start_send<D: Read + Write>(&mut self, dev: &mut D) -> Result<()> {
        let mut cancels = 0u32;
        loop {
            match (get_byte_timeout(dev))? {
                Some(c) => match c {
                    CRC => {
                        dbg!("16-bit CRC requested");
                        return Ok(());
                    }
                    CAN => {
                        warn!("Cancel (CAN) byte received");
                        cancels += 1;
                    }
                    c => warn!("Unknown byte received at start of YMODEM transfer: {}", c),
                },
                None => warn!("Timed out waiting for start of YMODEM transfer."),
            }

            self.errors += 1;

            if cancels >= 2 {
                eprint!(
                    "Transmission canceled: received two cancel (CAN) bytes \
                        at start of YMODEM transfer"
                );
                return Err(Error::Canceled);
            }

            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) at start of YMODEM transfer.",
                    self.max_errors
                );
                if let Err(err) = dev.write_all(&[CAN]) {
                    warn!("Error sending CAN byte: {}", err);
                }
                return Err(Error::ExhaustedRetries);
            }
        }
    }

    fn send_start_frame<D: Read + Write>(
        &mut self,
        dev: &mut D,
        file_name: String,
        file_size_in_bytes: u64,
        package_count : u32,
    ) -> Result<()> {
        let mut buff = vec![0x00; 1024 as usize + 3];
        buff[0] = STX;
        buff[1] = 0x00;
        buff[2] = 0xFF;

        let mut curr_buff_idx = 3;
        for byte in file_name.as_bytes() {
            buff[curr_buff_idx] = *byte;
            curr_buff_idx += 1;
        }

        // We leave one 0 to indicate the name ends here
        curr_buff_idx += 1;

        println!("{}", file_size_in_bytes);

        for byte in format!("{}", file_size_in_bytes).as_bytes() {
            buff[curr_buff_idx] = *byte;
            curr_buff_idx += 1;
        }

        buff[curr_buff_idx] = ' ' as u8;
        curr_buff_idx += 1;

        for byte in "15016031235".as_bytes() {
            buff[curr_buff_idx] = *byte;
            curr_buff_idx += 1;
        }

        buff[curr_buff_idx] = ' ' as u8;
        curr_buff_idx += 1;

        for byte in format!("{:o}", package_count).as_bytes() {
            buff[curr_buff_idx] = *byte;
            curr_buff_idx += 1;
        }

        buff[curr_buff_idx] = ' ' as u8;

        let crc = calc_crc(&buff[3..]);
        buff.push(((crc >> 8) & 0xFF) as u8);
        buff.push((crc & 0xFF) as u8);

        println!("{}", buff.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(""));

        (dev.write_all(&buff))?;
        (dev.flush())?;

        loop {
            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == ACK {
                        dbg!("Received ACK for start frame");
                        break;
                    } else {
                        warn!("Expected ACK, got {}", c);
                    }
                    // TODO handle CAN bytes
                }
                None => warn!("Timeout waiting for ACK for start frame"),
            }

            self.errors += 1;
            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while sending start frame in YMODEM transfer",
                    self.max_errors
                );
                return Err(Error::ExhaustedRetries);
            }
        }

        loop {
            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == CRC {
                        dbg!("Received C for start frame");
                        break;
                    } else {
                        warn!("Expected C, got {}", c);
                    }
                    // TODO handle CAN bytes
                }
                None => warn!("Timeout waiting for C for start frame"),
            }

            self.errors += 1;
            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while sending start frame in YMODEM transfer",
                    self.max_errors
                );
                return Err(Error::ExhaustedRetries);
            }
        }

        return Ok(());
    }

    fn send_stream<D: Read + Write, R: Read>(
        &mut self,
        dev: &mut D,
        stream: &mut R,
        packets_to_send: u32,
        last_packet_size: u64,
    ) -> Result<()> {
        let mut block_num = 0u32;
        loop {
            let packet_size = if block_num + 1 == packets_to_send && last_packet_size <= 128 {
                128
            } else {
                1024
            };
            let mut buff = vec![self.pad_byte; packet_size as usize + 3];
            let n = (stream.read(&mut buff[3..]))?;
            if n == 0 {
                dbg!("Reached EOF");
                return Ok(());
            }

            block_num += 1;
            if packet_size == 128 {
                buff[0] = SOH;
            } else {
                buff[0] = STX;
            }
            buff[1] = (block_num & 0xFF) as u8;
            buff[2] = 0xFF - buff[1];

            let crc = calc_crc(&buff[3..]);
            buff.push(((crc >> 8) & 0xFF) as u8);
            buff.push((crc & 0xFF) as u8);

            println!("Sending block {}", block_num);
            (dev.write_all(&buff))?;
            (dev.flush())?;

            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == ACK {
                        dbg!("Received ACK for block {}", block_num);
                        continue;
                    } else {
                        warn!("Expected ACK, got {}", c);
                    }
                    // TODO handle CAN bytes
                }
                None => warn!("Timeout waiting for ACK for block {}", block_num),
            }

            self.errors += 1;

            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while sending block {} in YMODEM transfer",
                    self.max_errors, block_num
                );
                return Err(Error::ExhaustedRetries);
            }
        }
    }

    fn finish_send<D: Read + Write>(&mut self, dev: &mut D) -> Result<()> {
        loop {
            (dev.write_all(&[EOT]))?;
            (dev.flush())?;

            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == NAK {
                        break;
                    } else {
                        log::warn!("Expected ACK, got {}", c);
                    }
                }
                None => warn!("Timeout waiting for ACK for EOT"),
            }

            self.errors += 1;

            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while waiting for ACK for EOT",
                    self.max_errors
                );
                return Err(Error::ExhaustedRetries);
            }
        }

        loop {
            (dev.write_all(&[EOT]))?;
            (dev.flush())?;

            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == ACK {
                        info!("YMODEM transmission successful");
                        break;
                    } else {
                        log::warn!("Expected ACK, got {}", c);
                    }
                }
                None => warn!("Timeout waiting for ACK for EOT"),
            }

            self.errors += 1;

            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while waiting for ACK for EOT",
                    self.max_errors
                );
                return Err(Error::ExhaustedRetries);
            }
        }

        loop {
            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == CRC {
                        info!("YMODEM transmission successful");
                        break;
                    } else {
                        log::warn!("Expected ACK, got {}", c);
                    }
                }
                None => warn!("Timeout waiting for ACK for EOT"),
            }

            self.errors += 1;

            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while waiting for ACK for EOT",
                    self.max_errors
                );
                return Err(Error::ExhaustedRetries);
            }
        }

        self.send_end_frame(dev)?;

        Ok(())
    }

    fn send_end_frame<D: Read + Write>(&mut self, dev: &mut D) -> Result<()> {
        let mut buff = vec![0x00; 128 as usize + 3];
        buff[0] = SOH;
        buff[1] = 0x00;
        buff[2] = 0xFF;

        let crc = calc_crc(&buff[3..]);
        buff.push(((crc >> 8) & 0xFF) as u8);
        buff.push((crc & 0xFF) as u8);

        (dev.write_all(&buff))?;
        (dev.flush())?;

        loop {
            match (get_byte_timeout(dev))? {
                Some(c) => {
                    if c == ACK {
                        dbg!("Received ACK for start frame");
                        break;
                    } else {
                        warn!("Expected ACK, got {}", c);
                    }
                    // TODO handle CAN bytes
                }
                None => warn!("Timeout waiting for ACK for start frame"),
            }

            self.errors += 1;
            if self.errors >= self.max_errors {
                eprint!(
                    "Exhausted max retries ({}) while sending start frame in YMODEM transfer",
                    self.max_errors
                );
                return Err(Error::ExhaustedRetries);
            }
        }

        return Ok(());
    }
}