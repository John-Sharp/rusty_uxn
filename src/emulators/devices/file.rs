use crate::emulators::uxn::device::{Device, MainRamInterface};

pub struct File {
    file_name_address: [u8, 2],
    file_name: String,
    success: u16,
}

impl File {
    pub fn new() -> Self {
        File{file_name_address: [0, 0], file_name: "".to_string(), success: 0}
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

        self.file_name = file_name;
        self.success = 0;
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
            _ => {}
        }
    }

    fn read(&mut self, port: u8) -> u8 {

    }

}
