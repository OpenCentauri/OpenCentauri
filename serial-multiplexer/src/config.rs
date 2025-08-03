use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SerialEntryRaw {
    pub device_path: String,
    pub baud_rate: u32,
    pub id: u8,
}

#[derive(Parser, Debug)]
#[command(
    name = "serial-multiplexer",
    about = "Send multiple serial devices over a single serial",
    version = "0.1"
)]
pub struct Args {
    #[arg(long, default_value_t = false)]
    pub with_virtual_ports: bool,

    #[arg(long, default_value_t = false)]
    pub with_real_ports: bool,

    #[arg(required = true)]
    pub device: String,

    #[arg(required = true)]
    pub config: String,

    #[arg(long, default_value_t = 115200)]
    pub baud: u32,
}