use crate::emulators::uxn::device::{Device, MainRamInterface};
use std::fs::File;
use std::io::Read;

pub struct FileDevice {
    file_name_address: [u8; 2],
    file_name: String,
    success: u16,
    fetch_length: [u8; 2],
    target_address: [u8; 2],
    file_handle: Option<File>,
}

impl FileDevice {
    pub fn new() -> Self {
        FileDevice{file_name_address: [0, 0], file_name: "".to_string(), success: 0,
        fetch_length: [0, 0], target_address: [0, 0], file_handle: None}
    }

    fn refresh_file_name(&mut self, main_ram: &mut dyn MainRamInterface) {
        let mut file_name = Vec::new();
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
            file_name_address += 1;
        }

        let file_name = String::from_utf8(file_name).expect(
            "Non utf8 string encountered when reading file name");

        self.file_handle = None;
        self.file_name = file_name;
        self.success = 0;
    }

    fn read_from_file(&mut self, main_ram: &mut dyn MainRamInterface) {
        let mut file = if let Some(file) = &self.file_handle {
            file
        } else {
            if let Ok(file) = File::open(&self.file_name) {
                self.file_handle = Some(file);
                if let Some(file) = &self.file_handle {
                    file
                } else {
                    panic!();
                }
            } else {
                self.success = 0;
                return;
            }
        };

        let mut buf = vec!(0; usize::from(u16::from_be_bytes(self.fetch_length)));
        let num_bytes_read = if let Ok(num_butes_read) = file.read(&mut buf) {
            num_butes_read
        } else {
            self.success = 0;
            return;
        };

        main_ram.write(u16::from_be_bytes(self.target_address),
            &mut buf[..num_bytes_read])
            .expect("had problem reading from file and writing to memory");
        self.success = u16::try_from(num_bytes_read).unwrap();
    }
}

impl Device for FileDevice {
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
        if port > 0xf {
            panic!("attempting to read from port out of range");
        }

        match port {
            0x2 => {
                return self.success.to_be_bytes()[0];
            },
            0x3 => {
                return self.success.to_be_bytes()[1];
            },
            _ => {
                return 0x0;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::cell::RefCell;
    use crate::emulators::uxn::device::MainRamInterfaceError;
    use uuid::Uuid;
    use std::fs;

    struct MockMainRamInterface {
        read_arguments_received: RefCell<VecDeque<(u16, u16,)>>,
        read_values_to_return: RefCell<VecDeque<Result<Vec<u8>, MainRamInterfaceError>>>,

        write_arguments_received: RefCell<VecDeque<(u16, Vec<u8>)>>,
        write_values_to_return: RefCell<VecDeque<Result<usize, MainRamInterfaceError>>>,
    }
    impl MockMainRamInterface {
        fn new() -> Self {
            MockMainRamInterface{
                read_arguments_received: RefCell::new(VecDeque::new()),
                read_values_to_return: RefCell::new(VecDeque::new()),
                write_arguments_received: RefCell::new(VecDeque::new()),
                write_values_to_return: RefCell::new(VecDeque::new()),
            }
        }
    }
    impl MainRamInterface for MockMainRamInterface {
        fn read(&self, address: u16, num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError> {
            self.read_arguments_received.borrow_mut()
                .push_back((address, num_bytes));
            return self.read_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn write(&mut self, address: u16, bytes: &[u8]) -> Result<usize, MainRamInterfaceError> {
            self.write_arguments_received.borrow_mut()
                .push_back((address, bytes.to_vec()));
            return self.write_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }
    }

    #[test]
    fn test_file_read() {
        let mut mock_ram_interface = MockMainRamInterface::new();

        let tmp_file_name = format!("test_file_read_{}", Uuid::new_v4());
        let mut tmp_file_path = std::env::temp_dir();
        tmp_file_path.push(tmp_file_name);
        let contents = "file contents 1234";
        fs::write(&tmp_file_path, &contents).expect("Failed to write test program");
        let tmp_file_path = tmp_file_path.into_os_string().into_string()
             .expect("could not convert file path into string");

        let mut read_values_to_return = tmp_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();

        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        let mut write_values_to_return = VecDeque::from([
            Ok(contents.len() - 3),
            Ok(3),
            Ok(0),
        ]);
        mock_ram_interface.write_values_to_return = RefCell::new(
            write_values_to_return);

        let mut file_device = FileDevice::new();

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xaa, &mut mock_ram_interface);
        file_device.write(0x9, 0xbb, &mut mock_ram_interface);

        let mut expected_start_address = 0xaabb;
        let read_arguments_expected = tmp_file_path.bytes()
            .chain([0x0_u8,])
            .map(|_b| {
                expected_start_address += 1;
                return (expected_start_address-1, 1);
            })
            .collect::<VecDeque<_>>();

        // assert that the file device has queried the ram and
        // read the file name from it
        assert_eq!(
            *mock_ram_interface.read_arguments_received.borrow(),
            read_arguments_expected);

        // write to the file device, setting the length to 
        // be read
        let chunk_length = u16::try_from(contents.len() - 3).unwrap();
        file_device.write(0xa, chunk_length.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, chunk_length.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address the read
        // data should be written to
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        // assert that the contents of the file is written to the correct
        // address
        let write_arguments_expected = (0xccdd_u16, "file contents 1".bytes()
            .collect::<Vec<_>>());
        assert_eq!(
            mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap(),
            write_arguments_expected);

        // assert that the success field is set to correct value
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, chunk_length);

        // read the data again
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        // assert that the remaining contents of the file is written
        // to the correct address
        let write_arguments_expected = (0xccdd_u16,
            "234".bytes().collect::<Vec<_>>());
        assert_eq!(
            mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap(),
            write_arguments_expected);

        // assert that the success field is set to correct value
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, 3);

        // read the data again, now that the whole file has been read
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        // assert that the success field is set to correct value
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, 0);
    }

    // try to read from a file where the file does not exist
    #[test]
    fn test_file_read_non_existent() {
        let mut mock_ram_interface = MockMainRamInterface::new();

        let tmp_file_name = format!("non_existent_file_{}", Uuid::new_v4());
        let mut tmp_file_path = std::env::temp_dir();
        tmp_file_path.push(tmp_file_name);
        let tmp_file_path = tmp_file_path.into_os_string().into_string()
             .expect("could not convert file path into string");

        let mut read_values_to_return = tmp_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        let mut file_device = FileDevice::new();

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xaa, &mut mock_ram_interface);
        file_device.write(0x9, 0xbb, &mut mock_ram_interface);

        let mut expected_start_address = 0xaabb;
        let read_arguments_expected = tmp_file_path.bytes()
            .chain([0x0_u8,])
            .map(|_b| {
                expected_start_address += 1;
                return (expected_start_address-1, 1);
            })
            .collect::<VecDeque<_>>();

        // assert that the file device has queried the ram and
        // read the file name from it
        assert_eq!(
            *mock_ram_interface.read_arguments_received.borrow(),
            read_arguments_expected);

        // write to the file device, setting the length to 
        // be read (this is an arbitrary value of 5 bytes in this case)
        let chunk_length = 5_u16;
        file_device.write(0xa, chunk_length.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, chunk_length.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address the read
        // data should be written to (not that it will be written,
        // since the file doesn't exist)
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        // assert that the success field is set to correct value of 0
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, 0);

        // assert that nothing has been written to ram
        assert_eq!(
            mock_ram_interface.write_arguments_received.into_inner(),
            VecDeque::new());
    }
}
