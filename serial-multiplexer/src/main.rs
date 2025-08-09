use clap::Parser;
use serialport::{Error, SerialPort, TTYPort};
use std::{
    collections::HashMap,
    fs::{self, create_dir, remove_file},
    io::{Read, Write},
    os::unix::fs::symlink,
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex, mpsc::Receiver},
    time::Duration,
};

use crate::config::{Args, SerialEntryRaw};
use crate::serial_connection::*;
mod config;
mod serial_connection;

fn main() {
    println!("Hello, world!");
    let args = Args::parse();
    if (!args.with_virtual_ports && !args.with_real_ports)
        || (args.with_virtual_ports && args.with_real_ports)
    {
        eprintln!("You must specify either --with_virtual_ports or --with_real_ports");
        exit(1);
    }

    let config_path = PathBuf::from(&args.config);
    if !config_path.exists() {
        eprintln!("Config file does not exist: {}", config_path.display());
        exit(2);
    }

    let config = fs::read_to_string(&config_path).unwrap();
    let serial_ports_raw: HashMap<String, SerialEntryRaw> = toml::from_str(&config).unwrap();
    if serial_ports_raw.is_empty() {
        eprintln!("No serial ports found in the config file.");
        exit(3);
    }

    let multiplexed_port_manager = SerialPortManager::with_settings(SerialConnectionSettings {
        baud_rate: args.baud,
        device_path: args.device,
    });

    let mut unused = vec![];
    let (main_bus_sender, main_bus_receiver) = std::sync::mpsc::channel::<DataBlock>();

    let mut sender_processors = vec![];
    let mut receiver_processors = vec![];
    let mut senders = vec![];

    if args.with_real_ports {
        serial_ports_raw.iter().for_each(|f| {
            let config = SerialConnectionSettings {
                baud_rate: f.1.baud_rate,
                device_path: f.1.device_path.clone(),
            };

            let (port_sender, port_receiver) = std::sync::mpsc::channel::<DataBlock>();

            let serial_port_manager = SerialPortManager::with_settings(config);
            let serial_port_manager_ref = Arc::new(Mutex::new(serial_port_manager));

            sender_processors.push(SerialConnectionSenderProcessor {
                id: f.1.id,
                port_manager: serial_port_manager_ref.clone(),
                port_receiver: port_receiver,
            });

            receiver_processors.push(SerialConnectionReceiverProcessor {
                id: f.1.id,
                port_manager: serial_port_manager_ref,
                write_to_main_bus: main_bus_sender.clone(),
            });

            senders.push(SerialConnectionSender {
                id: f.1.id,
                port_sender: port_sender,
            });
        });
    } else {
        serial_ports_raw.iter().for_each(|f| {
            let entry = f.1;

            let (port_sender, port_receiver) = std::sync::mpsc::channel::<DataBlock>();

            let (mut master, slave) = TTYPort::pair().expect("Unable to create ptty pair");
            master.set_timeout(Duration::MAX).unwrap();

            let name = slave.name().unwrap();
            unused.push(slave);

            let mut link_path = std::env::temp_dir();
            link_path.push("vtty");
            if !link_path.exists() {
                create_dir(&link_path).unwrap();
            }

            link_path.push(f.0);
            let _ = remove_file(&link_path);

            symlink(name, link_path).unwrap();

            let serial_port_manager = SerialPortManager::with_port(master);
            let serial_port_manager_ref = Arc::new(Mutex::new(serial_port_manager));

            sender_processors.push(SerialConnectionSenderProcessor {
                id: entry.id,
                port_manager: serial_port_manager_ref.clone(),
                port_receiver: port_receiver,
            });

            receiver_processors.push(SerialConnectionReceiverProcessor {
                id: entry.id,
                port_manager: serial_port_manager_ref,
                write_to_main_bus: main_bus_sender.clone(),
            });

            senders.push(SerialConnectionSender {
                id: entry.id,
                port_sender,
            });
        });
    }

    println!("Starting communication loop...");
    communicate(
        sender_processors,
        receiver_processors,
        senders,
        main_bus_receiver,
        multiplexed_port_manager,
    );
}

fn communicate(
    sender_processors: Vec<SerialConnectionSenderProcessor>,
    receiver_processors: Vec<SerialConnectionReceiverProcessor>,
    senders: Vec<SerialConnectionSender>,
    main_bus_receiver: Receiver<DataBlock>,
    multiplexed_port_manager: SerialPortManager,
) {
    let multiplexed_port_manager_ref = Arc::new(Mutex::new(multiplexed_port_manager));
    let multiplexed_port_manager_ref_clone = multiplexed_port_manager_ref.clone();

    sender_processors.into_iter().for_each(|f| {
        std::thread::spawn(move || {
            f.process_loop();
        });
    });

    receiver_processors.into_iter().for_each(|f| {
        std::thread::spawn(move || {
            f.process_loop();
        });
    });

    std::thread::spawn(move || {
        multiplexed_port_sender(multiplexed_port_manager_ref_clone, main_bus_receiver);
    });

    let serial_ports = senders
        .into_iter()
        .map(|f| (f.id as u32, f))
        .collect::<HashMap<u32, SerialConnectionSender>>();

    multiplexed_port_receiver(serial_ports, multiplexed_port_manager_ref);
}

fn multiplexed_port_sender(
    multiplexed_port_manager: Arc<Mutex<SerialPortManager>>,
    main_bus_receiver: Receiver<DataBlock>,
) {
    let mut multiplexed_port = give_port(&multiplexed_port_manager);

    loop {
        let data = main_bus_receiver.recv().unwrap();

        let len = data.data.len();
        let mut buff = [0u8; 2 + 255];
        buff[0] = data.id as u8;
        buff[1] = len as u8;
        buff[2..(2 + len)].copy_from_slice(&data.data);

        if let Err(e) = multiplexed_port.write_all(&buff) {
            // Something horrible happened, the multiplexed port is likely dead. Dropping packets until port is alive again...
            eprintln!("Failed to write to multiplexed port: {}", e);
            multiplexed_port = give_port(&multiplexed_port_manager);

            while main_bus_receiver.try_recv().is_ok() {
                // Clear the main bus receiver
            }

            continue;
        }

        #[cfg(debug_assertions)]
        println!("Sent {} bytes for device {}", data.data.len(), data.id);
    }
}

fn multiplexed_port_receiver(
    serial_connection_senders: HashMap<u32, SerialConnectionSender>,
    multiplexed_port_manager: Arc<Mutex<SerialPortManager>>,
) {
    let mut multiplexed_port = give_port(&multiplexed_port_manager);
    let mut senders = serial_connection_senders;

    loop {
        let mut mini_buff = [0u8; 2];

        if let Err(e) = multiplexed_port.read_exact(&mut mini_buff) {
            eprintln!(
                "Failed to read from multiplexed port (reading header): {}",
                e
            );
            multiplexed_port = give_port(&multiplexed_port_manager);
            continue;
        }

        let id = mini_buff[0];
        let length = mini_buff[1] as usize;

        if length == 0 {
            let reason = format!("Received zero-length data for device {}", id);
            clear_buff_with_error_handling(
                &mut multiplexed_port,
                &reason,
                &multiplexed_port_manager,
            );
            continue;
        }

        let mut buff = vec![0u8; length];

        if let Err(e) = multiplexed_port.read_exact(&mut buff) {
            eprintln!("Failed to read from multiplexed port (reading data): {}", e);
            multiplexed_port = give_port(&multiplexed_port_manager);
            continue;
        }

        #[cfg(debug_assertions)]
        println!("Received {} bytes for device {}", length, id);

        if let Some(port) = senders.get_mut(&(id as u32)) {
            port.port_sender
                .send(DataBlock { id, data: buff })
                .expect("Failed to send data block to port sender");
        } else {
            let reason = format!("Device with id {} does not exist", id);
            clear_buff_with_error_handling(
                &mut multiplexed_port,
                &reason,
                &multiplexed_port_manager,
            );
        }
    }
}

fn clear_buffer(port: &TTYPort, reason: &str) -> Result<(), Error> {
    eprintln!(
        "{}. Assuming we're not in sync! Waiting 1s and trying again...",
        reason
    );
    port.clear(serialport::ClearBuffer::Input)?;
    std::thread::sleep(Duration::from_secs(1u64));
    port.clear(serialport::ClearBuffer::Input)
}

fn clear_buff_with_error_handling(
    port: &mut TTYPort,
    reason: &str,
    port_manager: &Arc<Mutex<SerialPortManager>>,
) {
    if let Err(e) = clear_buffer(port, &reason) {
        eprintln!("Failed to clear buffer: {}", e);
        *port = give_port(port_manager);
    }
}
