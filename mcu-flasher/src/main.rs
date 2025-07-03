use std::{io::Cursor, path::PathBuf};
mod ymodem;
use ymodem::Ymodem;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mcu-flasher", about = "Flash a new firmware over the Elegoo bootloader", version = "0.1")]
struct Args
{
    /// Pad with 0x4000 bytes. Not needed if flashing elegoo's stock firmware.
    #[arg(long, default_value_t = true)]
    pub pad_firmware: bool,

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
    let args = Args::parse();
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

    let file_size_in_bytes = file_bytes.len() as u64;

    let mut cursor = Cursor::new(&mut file_bytes);

    println!("Sup");

    Ymodem::new()
        .send(&mut port, &mut cursor, file_name, file_size_in_bytes)
        .unwrap();

    println!("Hello, world!");
}
