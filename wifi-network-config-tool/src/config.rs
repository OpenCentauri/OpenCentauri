use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "wifi-network-config-tool",
    about = "Utility to interact with elegoo's wireless configuration",
    version = "0.1"
)]
pub struct Args {
    #[arg(required = true)]
    pub config_path: String,

    #[command(subcommand)]
    pub action: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List,
    Extract,
}
