use crate::emulators::uxn::device::{Device, MainRamInterface};
use std::fs::File;

pub struct File {
    file_name_address: [u8, 2],
    file_name: String,
    success: u16,
    fetch_length: [u8, 2],
    target_address: [u8, 2],
    file_handle = Option<File>,
}

impl File {
    pub fn new() -> Self {
        File{file_name_address: [0, 0], file_name: "".to_string(), success: 0,
        fetch_length: [0, 0], target_address: [0, 0], file_handle: None}
    }

    fn refresh_file_name(&mut self, main_ram: &mut dyn MainRamInterface) {
        let file_name = Vec::new();
        let mut file_name_address = u16::from_be_bytes(self.file_name_address);
        loop {
            let byte = match main_ram.read(file_name_address, 1) {
                Ok(bytes) => bytes[0],
                Err(err) => panic!("Failed to read file name: {}", err),
            };
            if byte == 0 {
                break;
            }
            file_name.push(byte);
            file_name_address++;
        }

        let file_name = String::from_utf8(file_name).expect(
            "Non utf8 string encountered when reading file name");

        self.file_handle = None;
        self.file_name = file_name;
        self.success = 0;
    }

    fn read_from_file(&mut self, main_ram: &mut dyn MainRamInterface) {
        let file = if let Some(file) = self.file_handle {
            file
        } else {
            if let Ok(file) = open(self.file_name) {
                file
            } else {
                self.success = 0;
                return;
            }
        };

        let mut buf = vec!(0; usize::from(u16::from_be_bytes(self.fetch_length)));
        if let num_bytes_read = if let Ok(num_butes_read) = file.read(&mut buf) {
            num_butes_read
        } else {
            self.success = 0;
            return;
        };

        // TODO write 

    }
}

impl Device for File {
    fn write(&mut self, port: u8, val: u8, main_ram: &mut dyn MainRamInterface) {
        if port > 0xf {
            panic!("attempting to write to port out of range");
        }

        match port {
            0x8 => {
                self.file_name_address[0] = val;
            },
            0x9 => {
                self.file_name_address[1] = val;
                self.refresh_file_name(main_ram);
            },
            0xa => {
                self.fetch_length[0] = val;
            },
            0xb => {
                self.fetch_length[1] = val;
            },
            0xc => {
                self.target_address[0] = val;
            },
            0xd => {
                self.target_address[1] = val;
                self.read_from_file(main_ram);
            },
            _ => {}
        }
    }

    fn read(&mut self, port: u8) -> u8 {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
