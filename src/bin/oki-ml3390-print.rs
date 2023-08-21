use clap::Parser;
use rusb::{DeviceHandle, GlobalContext};
use std::{
    io::{BufRead, Read},
    time::Duration,
};

#[derive(Debug, Parser)]
struct Options {
    /// Read binary from stdin (useful for Epson escape data from GhostScript)
    #[clap(short, long)]
    binary: bool,
    /// Add a form-feed to the end of the output (unavailable with `--binary`)
    #[clap(short, long)]
    form_feed: bool,
}

fn feed_lines(handle: &DeviceHandle<GlobalContext>) {
    for buffer in std::io::stdin().lock().lines() {
        match buffer {
            Err(err) => panic!("Unable to read from stdin: {:?}", err),
            Ok(mut buffer) => {
                buffer.push('\n');

                let slice = buffer.as_bytes();
                let mut written = 0;

                while written < slice.len() {
                    match handle.write_bulk(1, &slice[written..], Duration::from_secs(10)) {
                        Err(err) => panic!("Unable to write to printer: {:?}", err),
                        Ok(n) => written = n,
                    }
                }
            }
        }
    }
}

fn feed_binary(handle: &DeviceHandle<GlobalContext>) {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = [0; 1024];

    loop {
        match stdin.read(&mut buffer[..]) {
            Err(err) => panic!("Unable to read from stdin: {:?}", err),
            Ok(0) => break,
            Ok(length) => {
                println!("Read {length} bytes");
                let mut slice = &buffer[..length];
                let mut written = 0;
                while written < length {
                    match handle.write_bulk(1, slice, Duration::from_secs(10)) {
                        Err(err) => panic!("Unable to write to printer: {:?}", err),
                        Ok(n) => {
                            written = n;
                            println!("Written {n} bytes; {written}/{length}");
                            if n < length {
                                slice = &buffer[written..];
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let options = Options::parse();

    let found = rusb::devices()
        .expect("Unable to retrieve devices")
        .iter()
        .find(|device| {
            let desc = device
                .device_descriptor()
                .expect("Unable to access device descriptor");
            desc.vendor_id() == 0x06bc && desc.product_id() == 0x0031
        });

    match found {
        None => {
            panic!("Unable to find Oki ML-3390");
        }

        Some(device) => {
            let mut handle = device.open().expect("Tried to open device");

            handle
                .claim_interface(0)
                .expect("Attempted to claim interface");

            if options.binary {
                if options.form_feed {
                    eprintln!("Ignoring use of '--form-feed' in binary mode");
                }

                match handle.write_bulk(1, &[0x1b, 0x40], Duration::from_secs(10)) {
                    Ok(n) => assert_eq!(n, 2, "Only transmitted {n} initialization bytes"),
                    Err(err) => panic!("Unable to send initialization bytes to printer: {err:?}"),
                }

                feed_binary(&handle);
            } else {
                feed_lines(&handle);

                if options.form_feed {
                    // Write a form-feed at the end of the document
                    if let Err(err) =
                        handle.write_bulk(1, "\x0c".as_bytes(), Duration::from_secs(10))
                    {
                        panic!("Unable to write to printer: {:?}", err);
                    }
                }
            }
        }
    }
}
