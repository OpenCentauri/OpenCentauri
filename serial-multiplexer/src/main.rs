use clap::Parser;
use serialport::{SerialPort, TTYPort};
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

    let mut multiplexed_port = match serialport::new(&args.device, args.baud)
        .timeout(Duration::MAX)
        .open_native()
    {
        Ok(port) => port,
        Err(e) => {
            eprintln!(
                "Failed to open multiplexed serial port {}: {}",
                args.device, e
            );
            exit(5);
        }
    };

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
            master.set_timeout(Duration::from_millis(100u64)).unwrap();

            let name = slave.name().unwrap();
            unused.push(slave);

            let mut link_path = std::env::home_dir().unwrap_or(PathBuf::from("/dev"));

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
        &mut multiplexed_port,
    );
}

fn communicate(
    sender_processors: Vec<SerialConnectionSenderProcessor>,
    receiver_processors: Vec<SerialConnectionReceiverProcessor>,
    senders: Vec<SerialConnectionSender>,
    main_bus_receiver: Receiver<DataBlock>,
    multiplexed_port: &mut TTYPort,
) {
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

    let mut multiplexed_port_clone = multiplexed_port.try_clone_native().unwrap();

    std::thread::spawn(move || {
        loop {
            let data = main_bus_receiver.recv().unwrap();

            let mut mini_buff = [0u8; 2];
            mini_buff[0] = data.id as u8;
            mini_buff[1] = data.data.len() as u8;

            multiplexed_port_clone.write_all(&mini_buff).unwrap();
            multiplexed_port_clone.write_all(&data.data).unwrap();

            #[cfg(debug_assertions)]
            println!("Sent {} bytes for device {}", data.data.len(), data.id);
        }
    });

    let mut serial_ports = senders
        .into_iter()
        .map(|f| (f.id as u32, f))
        .collect::<HashMap<u32, SerialConnectionSender>>();

    loop {
        let mut mini_buff = [0u8; 2];
        if multiplexed_port.read_exact(&mut mini_buff).is_ok() {
            let id = mini_buff[0];
            let length = mini_buff[1] as usize;

            let mut buff = vec![0u8; length];
            if multiplexed_port.read_exact(&mut buff).is_ok() {
                #[cfg(debug_assertions)]
                println!("Received {} bytes for device {}", length, id);

                if let Some(port) = serial_ports.get_mut(&(id as u32)) {
                    port.port_sender
                        .send(DataBlock { id, data: buff })
                        .expect("Failed to send data block to port sender");
                } else {
                    eprintln!(
                        "Device with id {} does not exist. Assuming we're not in sync! Waiting 1s and trying again...",
                        id
                    );
                    multiplexed_port
                        .clear(serialport::ClearBuffer::Input)
                        .unwrap();
                    std::thread::sleep(Duration::from_secs(1u64));
                    multiplexed_port
                        .clear(serialport::ClearBuffer::Input)
                        .unwrap();
                }
            }
        }
    }
}
