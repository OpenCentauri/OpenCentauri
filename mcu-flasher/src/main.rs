use std::{fs, io::Cursor, path::PathBuf};
mod ymodem;
use md5::{Digest, Md5};
use ymodem::Ymodem;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mcu-flasher", about = "Flash a new firmware over the Elegoo bootloader", version = "0.1")]
struct Args
{
    /// Don't pad with 0x4000 bytes. Not needed if flashing elegoo's stock firmware.
    #[arg(long, default_value_t = true)]
    pub no_pad_firmware: bool,

    // Don't flash firmware and just boot the existing firmware.
    #[arg(long, default_value_t = false)]
    pub skip : bool,

    // Wait until the serial port is available.
    #[arg(long, default_value_t = true)]
    pub wait : bool,

    // Path to the firmware file
    #[arg(long, default_value = "")]
    pub firmware: String,

    #[arg(long, default_value_t = 115200)]
    pub baud : u32,    

    // Path to the device
    #[arg(required = true)]
    pub device: String,
}

fn main() {
    let mut args = Args::parse();
    let mut found = !args.wait;

    while !found
    {
        let ports = serialport::available_ports().expect("No ports found!");
        for p in ports {
            if p.port_name == args.device
            {
                found = true;
            }
        }

        if !found
        {
            println!("Waiting for device at {}...", args.device);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    let mut port = serialport::new(&args.device, args.baud)
        .timeout(std::time::Duration::from_secs(10))
        .dtr_on_open(true)
        .open()
        .expect("Failed to open port");

    if args.skip
    {
        println!("Skipping flash. Booting existing firmware on device: {}", args.device);
        let mut buf = [0u8; 1];
        buf[0] = 'a' as u8;
        port.write(&buf).expect("Failed to write to port");
        port.flush().unwrap();
        return;
    }

    if args.firmware.is_empty() || !PathBuf::from(&args.firmware).exists()
    {
        println!("No firmware file provided or file does not exist. Exiting.");
        return;
    }

    let file_name = PathBuf::from(&args.firmware)
        .file_name()
        .expect("Failed to get file name")
        .to_string_lossy()
        .to_string();

    let mut file_bytes = std::fs::read(&args.firmware)
        .expect("Failed to read firmware file");

    let mut file_size_in_bytes = file_bytes.len() as u64;

    if file_bytes.starts_with(&vec![0x14, 0x18, 0x01, 0x1A])
    {
        println!("Firmware file already has a header. No need to pad.");
        args.no_pad_firmware = true;
    }

    if !args.no_pad_firmware
    {
        let file_size = file_size_in_bytes as u32;

        let mut header = [0u8; 0x10];
        header[0x0..0x4].copy_from_slice(&vec![0x14, 0x18, 0x01, 0x1A]); // Magic
        header[0x4] = 0x01; // Board type
        header[0x5] = 0x02; // Patch version
        header[0x6] = 0x03; // Minor version
        header[0x7] = 0xFF; // Major version
        header[0x8] = 0x01; // Unknown
        header[0xC..0x10].copy_from_slice(&file_size.to_le_bytes());

        let mut hasher = Md5::new();
        hasher.update(&file_bytes);
        let checksum = hasher.finalize();

        println!("MD5 Checksum: {:x}", checksum);

        let padding = [0xFFu8; 0x4000 - 0x20];

        file_bytes = [&header[..], &checksum[..], &padding[..], &file_bytes[..]].concat();
        file_size_in_bytes = file_bytes.len() as u64;
    }

    let mut cursor = Cursor::new(&mut file_bytes);

    Ymodem::new()
        .send(&mut port, &mut cursor, file_name, file_size_in_bytes)
        .unwrap();
}
