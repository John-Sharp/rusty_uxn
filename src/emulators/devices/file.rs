use crate::emulators::uxn::device::{Device, MainRamInterface};
use std::fs::{File, ReadDir};
use std::io::Read;
use std::io;
use std::fs;
use std::path::Path;
use std::iter::Peekable;

enum FsObject {
    None,
    File(File),
    Directory(Peekable<DirEntryProducer>),
}

struct DirEntryProducer {
    inner: ReadDir,
}

impl DirEntryProducer {
    fn new(inner: ReadDir) -> Self {
       DirEntryProducer { inner } 
    }
}
 
impl Iterator for DirEntryProducer {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.inner.next() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Some(Err(err)),
            };

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(err) => return Some(Err(err)),
            };

            let len = if metadata.is_dir() {
                "----".to_string()
            } else if let Ok(len) = u16::try_from(metadata.len()) {
                format!("{:04x}", len)
            } else {
                "????".to_string()
            };

            return Some(Ok(format!("{} {}\n", len, entry.file_name().into_string().expect("error, unsupported filename")).into_bytes()));
        } else {
            return None;
        }
    }
}

pub struct FileDevice {
    file_name_address: [u8; 2],
    file_name: String,
    success: u16,
    fetch_length: [u8; 2],
    target_address: [u8; 2],
    subject: FsObject,
}

fn open_fs_object(fs_name: &str) -> FsObject {
    if Path::new(fs_name).is_dir() {
        if let Ok(dir) = fs::read_dir(fs_name) {
            return FsObject::Directory(DirEntryProducer::new(dir).peekable());
        } else {
            return FsObject::None;
        }
    }

    if let Ok(file) = File::open(fs_name) {
        return FsObject::File(file);
    }
    return FsObject::None;
}

impl FileDevice {
    pub fn new() -> Self {
        FileDevice{file_name_address: [0, 0], file_name: "".to_string(), success: 0,
        fetch_length: [0, 0], target_address: [0, 0], subject: FsObject::None,}
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

        self.subject = FsObject::None;
        self.file_name = file_name;
        self.success = 0;
    }

    fn read_from_dir(&mut self, main_ram: &mut dyn MainRamInterface) {
        let dir = if let FsObject::Directory(dir) = &mut self.subject {
            dir
        } else {
            panic!("in read_from_dir, subject should be FsObject::Directory");
        };

        let bytes_to_write = u16::from_be_bytes(self.fetch_length);
        let bytes_to_write = usize::from(bytes_to_write);

        let mut buffer = Vec::<u8>::new();
        loop {
            let next_entry = dir.peek();
            let next_entry = if let Some(next_entry) = next_entry {
                next_entry
            } else {
                break;
            };

            let next_entry = if let Ok(next_entry) = next_entry {
                next_entry
            } else {
                self.success = 0;
                return;
            };

            if buffer.len() + next_entry.len() > bytes_to_write {
                break;
            }
            let next_entry = dir.next().unwrap().unwrap();
            buffer.extend(next_entry.into_iter());
        }

        main_ram.write(u16::from_be_bytes(self.target_address),
            &mut buffer)
            .expect("had problem reading from file and writing to memory");
        self.success = u16::try_from(buffer.len()).unwrap();
    }

    fn read_from_file(&mut self, main_ram: &mut dyn MainRamInterface) {
        let file = if let FsObject::File(file) = &mut self.subject {
            file
        } else {
            panic!("in read_from_file, subject should be FsObject::File");
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

    fn read_from_fs(&mut self, main_ram: &mut dyn MainRamInterface) {
        match self.subject {
            FsObject::None => {
                self.subject = open_fs_object(&self.file_name);
            },
            _ => {}
        }

        match &mut self.subject {
            FsObject::None => {
                self.success = 0;
                return;
            },
            FsObject::File(_) => {
                self.read_from_file(main_ram);
            },
            FsObject::Directory(_) => {
                self.read_from_dir(main_ram);
            },
        }
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
                self.read_from_fs(main_ram);
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
    use std::collections::HashSet;

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

        let read_values_to_return = tmp_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();

        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        let write_values_to_return = VecDeque::from([
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

        let read_values_to_return = tmp_file_path.bytes()
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

    // try and read a directory and assert that the correct format
    // is produced and the correct paging behaviour can be observed
    #[test]
    fn test_dir_read() {
        let mut mock_ram_interface = MockMainRamInterface::new();

        let tmp_dir_name = format!("test_dir_{}", Uuid::new_v4());
        let mut tmp_dir_path = std::env::temp_dir();
        tmp_dir_path.push(tmp_dir_name);

        fs::create_dir(&tmp_dir_path).expect("could not make test directory");

        let mut test_file_path = tmp_dir_path.clone();
        let test_file_name = format!("test_file_A_{}", Uuid::new_v4());
        test_file_path.push(test_file_name.clone());
        let contents = [0xff; 0x01ab];
        fs::write(&test_file_path, &contents).expect("Failed to write test file");

        let mut test_inner_dir_path = tmp_dir_path.clone();
        let test_inner_dir_name = format!("test_dir_AA_{}", Uuid::new_v4());
        test_inner_dir_path.push(test_inner_dir_name.clone());
        fs::create_dir(&test_inner_dir_path).expect("Failed to create test inner directory");

        let mut large_test_file_path = tmp_dir_path.clone();
        let large_test_file_name = format!("test_file_B_{}", Uuid::new_v4());
        large_test_file_path.push(large_test_file_name.clone());
        let contents = [0xff; 0x10000];
        fs::write(&large_test_file_path, &contents).expect("Failed to write large test file");

        let tmp_dir_path = tmp_dir_path.into_os_string().into_string()
             .expect("could not convert directory path into string");
        let read_values_to_return = tmp_dir_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();

        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        let first_entry = format!("01ab {}\n", test_file_name);
        let entry_len = first_entry.len();
        let expected_contents = HashSet::from([
            first_entry,
            format!("---- {}\n", test_inner_dir_name),
            format!("???? {}\n", large_test_file_name),
        ]);
        let total_len = entry_len * expected_contents.len();

        // TODO add paging
        let write_values_to_return = VecDeque::from([
            Ok(total_len),
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
        let read_arguments_expected = tmp_dir_path.bytes()
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
        let chunk_length = u16::try_from(total_len + 3).unwrap();
        file_device.write(0xa, chunk_length.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, chunk_length.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address the read
        // data should be written to
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        // assert that the contents of the file is written to the correct
        // address
        let write_address_expected = 0xccdd_u16;
        let write_arguments_received = mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap();

        assert_eq!(write_arguments_received.0, write_address_expected);

        let string_received = String::from_utf8(write_arguments_received.1).unwrap();

        // collect individual files and directories into a hash set
        // before comparing since the order cannot be guaranteed
        let received_directory_contents = string_received.split_inclusive('\n')
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();

        assert_eq!(expected_contents, received_directory_contents);

        // clean up
        fs::remove_dir_all(&tmp_dir_path).expect("failed to clean up test directory");
    }
}
