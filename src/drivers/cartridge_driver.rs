use std::fs::File;
use std::path::Path;
use std::io::Read;

pub struct CartridgeDriver {
    pub rom: [u8; 3584],
    pub size: usize,
}

impl CartridgeDriver {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {

        let mut f = File::open(path).expect("file not found");
        let mut buffer = [0u8; 3584];

        let bytes_read = if let Ok(bytes_read) = f.read(&mut buffer) {
            bytes_read
        } else {
            0
        };

        CartridgeDriver {
            rom: buffer,
            size: bytes_read,
        }
    }
}