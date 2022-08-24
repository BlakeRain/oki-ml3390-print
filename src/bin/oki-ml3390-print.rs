use std::{io::BufRead, time::Duration};

fn main() {
    let found = rusb::devices()
        .expect("Unable to retrieve devices")
        .iter()
        .find(|device| {
            let desc = device
                .device_descriptor()
                .expect("Unable to access device descriptor");
            return desc.vendor_id() == 0x06bc && desc.product_id() == 0x0031;
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

            let stdin = std::io::stdin();
            let mut lines = stdin.lock().lines();

            // Read lines from 'stdin' and write them to the printer
            while let Some(buffer) = lines.next() {
                match buffer {
                    Err(err) => panic!("Unable to read from stdin: {:?}", err),
                    Ok(mut buffer) => {
                        buffer.push('\n');

                        let slice = buffer.as_bytes();
                        let mut written = 0;

                        while written < slice.len() {
                            match handle.write_bulk(1, slice, Duration::from_secs(10)) {
                                Err(err) => panic!("Unable to write to printer: {:?}", err),
                                Ok(n) => written = n,
                            }
                        }
                    }
                }
            }

            // Write a form-feed at the end of the document
            if let Err(err) = handle.write_bulk(1, "\x0c".as_bytes(), Duration::from_secs(10)) {
                panic!("Unable to write to printer: {:?}", err);
            }
        }
    }
}
