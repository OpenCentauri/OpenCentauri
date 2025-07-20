mod config;
use config::Args;
use clap::Parser;

struct ConnectionInformation {
    ssid: String,
    password: String,
}

fn main() {
    let args = Args::parse();
    let bytes = std::fs::read(&args.config_path).expect("Failed to read configuration file");

    let current_configs_count_1 = u32::from_le_bytes(bytes[0x00..0x04].try_into().unwrap()) as usize;
    let current_configs_count_2 = u32::from_le_bytes(bytes[0x04..0x08].try_into().unwrap()) as usize;
    let max_configs_count = u32::from_le_bytes(bytes[0x08..0x0C].try_into().unwrap());
    let bytes_per_config = u32::from_le_bytes(bytes[0x0C..0x10].try_into().unwrap());

    assert!(current_configs_count_1 == current_configs_count_2);

    // TODO: Verify checksum. What even is this a checksum of?
    let checksum = bytes[0x10..0x20].to_vec();

    let chunks = bytes[0x20..].chunks_exact(bytes_per_config as usize);
    let mut configs = Vec::new();

    chunks.into_iter().take(current_configs_count_1).for_each(|f| {
        configs.push(ConnectionInformation {
            ssid: String::from_utf8_lossy(&f[0x00..0x21]).trim().to_string(),
            password: String::from_utf8_lossy(&f[0x21..]).trim().to_string(),
        });
    });

    configs.reverse();

    match args.action {
        config::Commands::List => {
            println!("Stored configurations: {}/{}", current_configs_count_1, max_configs_count);
            println!("Bytes per configuration: {}", bytes_per_config);
            println!("Checksum: {}", checksum.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(""));
            println!();

            let mut last_used = true;

            configs.iter().for_each(|f| {
                if last_used {
                    println!("[Last used configuration]");
                    last_used = false;
                }

                println!("SSID: {}", f.ssid);
                println!("Password: {}", f.password);
                println!();
            });

            // Implement the logic for listing configurations
            println!("Listing configurations from {}", args.config_path);
        }
        config::Commands::Extract => {
            configs.iter().for_each(|f| {
                println!("{}", f.ssid);
                println!("{}", f.password);
            });
        }
    }
}
