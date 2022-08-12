use crate::emulators::uxn::device::{Device, MainRamInterface};
use std::fs::{File, ReadDir};
use std::io::{Read, Write};
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
            let file_name = entry.file_name().into_string().expect("unsupported file");
            return Some(Ok(produce_dir_entry_string(&file_name, metadata).into_bytes()));
        } else {
            return None;
        }
    }
}

fn produce_dir_entry_string(file_name: &str, metadata: fs::Metadata) -> String {
    let len = if metadata.is_dir() {
        "----".to_string()
    } else if let Ok(len) = u16::try_from(metadata.len()) {
        format!("{:04x}", len)
    } else {
        "????".to_string()
    };

    return format!("{} {}\n", len, file_name);
}

pub struct FileDevice {
    file_name_address: [u8; 2],
    file_name: String,
    success: u16,
    fetch_length: [u8; 2],
    target_address: [u8; 2],
    stat_target_address: [u8; 2],
    write_target_address: [u8; 2],
    subject: FsObject,
    append: u8,
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
        fetch_length: [0, 0], target_address: [0, 0], stat_target_address: [0, 0],
        write_target_address: [0, 0], subject: FsObject::None, append: 0,}
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

    fn stat_from_fs(&mut self, main_ram: &mut dyn MainRamInterface) {
        let metadata = fs::metadata(&self.file_name);
        let metadata = if let Ok(metadata) = metadata {
            metadata
        } else {
            self.success = 0;
            return;
        };

        let output = produce_dir_entry_string(
            Path::new(&self.file_name).file_name().unwrap().to_str().unwrap(),
            metadata)
            .into_bytes();

        if output.len() > usize::from(u16::from_be_bytes(self.fetch_length)) {
            self.success = 0;
            return;
        }

        main_ram.write(u16::from_be_bytes(self.stat_target_address),
            &output)
            .expect("had problem reading from file and writing to memory");
        self.success = u16::try_from(output.len()).unwrap();
    }

    fn delete_from_fs(&mut self) {
        let res = fs::remove_file(&self.file_name);
        if let Ok(_) = res {
            self.success = 1;
        } else {
            self.success = 0;
        }
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

    fn write_to_fs(&mut self, main_ram: &mut dyn MainRamInterface) {
        let mut f = File::options();
        f.write(true);
        if self.append == 0x1 {
            f.append(true);
        } else {
            f.truncate(true);
        }

        f.create(true);
        let f = f.open(&self.file_name);

        let mut f = if let Ok(f) = f {
            f
        } else {
            self.success = 0;
            return;
        };

        let data_to_write = main_ram.read(
            u16::from_be_bytes(self.write_target_address),
            u16::from_be_bytes(self.fetch_length));

        let data_to_write = if let Ok(d) = data_to_write {
            d
        } else {
            self.success = 0;
            return;
        };

        f.write(&data_to_write).expect("Failed to write to file system");
    }
}

impl Device for FileDevice {
    fn write(&mut self, port: u8, val: u8, main_ram: &mut dyn MainRamInterface) {
        if port > 0xf {
            panic!("attempting to write to port out of range");
        }

        match port {
            0x4 => {
                self.stat_target_address[0] = val;
            },
            0x5 => {
                self.stat_target_address[1] = val;
                self.stat_from_fs(main_ram);
            },
            0x6 => {
                self.delete_from_fs();
            },
            0x7 => {
                self.append = val;
            }
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
            0xe => {
                self.write_target_address[0] = val;
            },
            0xf => {
                self.write_target_address[1] = val;
                self.write_to_fs(main_ram);
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
    use std::io;
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
        // set chunk length to be slightly more than needed to fetch
        // two entries but will require two fetches to fetch the entire
        // directory
        let chunk_len = 2 * entry_len + 4;

        let write_values_to_return = VecDeque::from([
            Ok(chunk_len),
            Ok(entry_len),
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
        let chunk_len = u16::try_from(chunk_len).unwrap();
        file_device.write(0xa, chunk_len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, chunk_len.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address the read
        // data should be written to
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        // assert that the contents of the directory is written to the correct
        // address
        let write_address_expected = 0xccdd_u16;
        let write_arguments_received = mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap();
        assert_eq!(write_arguments_received.0, write_address_expected);

        // assert that the 'success' field has been written to with the 
        // expected number of bytes, 2*entry_len
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, u16::try_from(2*entry_len).unwrap());

        let string_received = String::from_utf8(write_arguments_received.1).unwrap();
        let mut received_directory_contents = string_received.split_inclusive('\n')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // do a second write to finish off the directory
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);

        let write_arguments_received = mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap();
        assert_eq!(write_arguments_received.0, write_address_expected);

        // assert that the 'success' field has been written to with the 
        // expected number of bytes, entry_len
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, u16::try_from(entry_len).unwrap());

        let string_received = String::from_utf8(write_arguments_received.1).unwrap();
        received_directory_contents.extend(string_received.split_inclusive('\n')
            .map(|s| s.to_string())
            .collect::<Vec<String>>());

        // assert that the number of entries retrieved is correct
        assert_eq!(received_directory_contents.len(), 3);

        // collect individual files and directories into a hash set
        // before comparing since the order cannot be guaranteed
        let received_directory_contents = received_directory_contents
            .into_iter()
            .collect::<HashSet<String>>();

        assert_eq!(expected_contents, received_directory_contents);

        // assert that one more attempt to write sets success to 0
        file_device.write(0xc, 0xcc, &mut mock_ram_interface);
        file_device.write(0xd, 0xdd, &mut mock_ram_interface);
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, 0);

        // clean up
        fs::remove_dir_all(&tmp_dir_path).expect("failed to clean up test directory");
    }

    // try and stat a file, assert that the correct format is produced
    #[test]
    fn test_stat_file() {
        let mut mock_ram_interface = MockMainRamInterface::new();
        let mut file_device = FileDevice::new();

        // make a test file (of length 0x0a0b)
        let mut test_file_path = std::env::temp_dir();
        let test_file_name = format!("test_file_{}", Uuid::new_v4());
        test_file_path.push(test_file_name.clone());
        fs::write(&test_file_path, &[0xff; 0x0a0b]).expect("Failed to write test file");
        let test_file_path = test_file_path.into_os_string().into_string()
             .expect("could not convert file path into string");


        let read_values_to_return = test_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        // set length of memory area the file stat should be written to to be just big enough,
        // the length of the file name plus 4 characters for the file size, plus a space, plus the
        // new line character
        let len = test_file_name.len() + 6;
        let len = u16::try_from(len).unwrap();
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xcc, &mut mock_ram_interface);
        file_device.write(0x9, 0xcd, &mut mock_ram_interface);

        let mut expected_start_address = 0xcccd;
        let read_arguments_expected = test_file_path.bytes()
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


        let write_values_to_return = VecDeque::from([
            Ok(usize::from(len)),
        ]);
        mock_ram_interface.write_values_to_return = RefCell::new(
            write_values_to_return);

        // write to the address(stat) port, and assert that the correct
        // expected string ('0a0b test_file_<guid>\n') has been written
        // to the correct location of the mock ram interface
        file_device.write(0x4, 0x12, &mut mock_ram_interface);
        file_device.write(0x5, 0x34, &mut mock_ram_interface);

        let expected_output = (0x1234, format!("0a0b {}\n", test_file_name));
        let write_arguments_received = mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap();
        let write_arguments_received = (write_arguments_received.0, String::from_utf8(write_arguments_received.1).unwrap());
        assert_eq!(write_arguments_received, expected_output);

        // assert that the 'success' field has been written to with the 
        // expected number of bytes
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, u16::try_from(len).unwrap());

        fs::remove_file(test_file_path).expect("Failed to clean up test file");
    }

    // try and stat a directory, assert that the correct format is produced
    #[test]
    fn test_stat_directory() {
        let mut mock_ram_interface = MockMainRamInterface::new();
        let mut file_device = FileDevice::new();

        // make a test directory
        let mut test_dir_path = std::env::temp_dir();
        let test_dir_name = format!("test_dir_{}", Uuid::new_v4());
        test_dir_path.push(test_dir_name.clone());
        fs::create_dir(&test_dir_path).expect("could not make test directory");
        let test_dir_path = test_dir_path.into_os_string().into_string()
             .expect("could not convert directory path into string");

        let read_values_to_return = test_dir_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        // set length of memory area the file stat should be written to to be just big enough, the
        // length of the file name plus 4 characters for the directory 'size' (just the string
        // '----'), plus a space, plus the new line character
        let len = test_dir_name.len() + 6;
        let len = u16::try_from(len).unwrap();
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xcc, &mut mock_ram_interface);
        file_device.write(0x9, 0xcd, &mut mock_ram_interface);

        let mut expected_start_address = 0xcccd;
        let read_arguments_expected = test_dir_path.bytes()
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

        let write_values_to_return = VecDeque::from([
            Ok(usize::from(len)),
        ]);
        mock_ram_interface.write_values_to_return = RefCell::new(
            write_values_to_return);

        // write to the address(stat) port, and assert that the correct
        // expected string ('---- test_dir_<guid>\n') has been written
        // to the correct location of the mock ram interface
        file_device.write(0x4, 0x12, &mut mock_ram_interface);
        file_device.write(0x5, 0x34, &mut mock_ram_interface);

        let expected_output = (0x1234, format!("---- {}\n", test_dir_name));
        let write_arguments_received = mock_ram_interface.write_arguments_received.borrow_mut().pop_front().unwrap();
        let write_arguments_received = (write_arguments_received.0, String::from_utf8(write_arguments_received.1).unwrap());
        assert_eq!(write_arguments_received, expected_output);

        // assert that the 'success' field has been written to with the 
        // expected number of bytes
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, u16::try_from(len).unwrap());

        fs::remove_dir_all(&test_dir_path).expect("failed to clean up test directory");
    }

    // try and stat a non-existent file, assert that the correct format is produced
    #[test]
    fn test_stat_non_existent() {
        let mut mock_ram_interface = MockMainRamInterface::new();
        let mut file_device = FileDevice::new();

        // the file path we will attempt (and fail) to stat
        let mut test_path = std::env::temp_dir();
        let non_existent_file_name = format!("test_file_{}", Uuid::new_v4());
        test_path.push(non_existent_file_name.clone());
        let non_existent_file_path = test_path.into_os_string().into_string()
             .expect("could not convert path into string");

        let read_values_to_return = non_existent_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        // set length of memory area the file stat should be written to
        let len = 99_u16;
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xcc, &mut mock_ram_interface);
        file_device.write(0x9, 0xcd, &mut mock_ram_interface);

        let mut expected_start_address = 0xcccd;
        let read_arguments_expected = non_existent_file_path.bytes()
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

        // write to the address(stat) port
        file_device.write(0x4, 0x12, &mut mock_ram_interface);
        file_device.write(0x5, 0x34, &mut mock_ram_interface);

        // assert that the 'success' field has been written to with 0
        // since the file does not exist
        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, 0_u16);
    }

    // try and stat a file, but with the area the stat is to be written to set to be too small for
    // the output. Assert that success is set to 0
    #[test]
    fn test_stat_file_failure() {
        let mut mock_ram_interface = MockMainRamInterface::new();
        let mut file_device = FileDevice::new();

        // make a test file (of length 0x0001)
        let mut test_file_path = std::env::temp_dir();
        let test_file_name = format!("test_file_{}", Uuid::new_v4());
        test_file_path.push(test_file_name.clone());
        fs::write(&test_file_path, &[0xff; 0x0001]).expect("Failed to write test file");
        let test_file_path = test_file_path.into_os_string().into_string()
             .expect("could not convert file path into string");

        let read_values_to_return = test_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        // set length of memory area the file stat should be written to to be just too small
        let len = test_file_name.len() + 5;
        let len = u16::try_from(len).unwrap();
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xcc, &mut mock_ram_interface);
        file_device.write(0x9, 0xcd, &mut mock_ram_interface);

        let mut expected_start_address = 0xcccd;
        let read_arguments_expected = test_file_path.bytes()
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

        // write to the address(stat) port, and assert that success is set 
        // to 0 (since there isn't enough space to write the entry)
        file_device.write(0x4, 0x12, &mut mock_ram_interface);
        file_device.write(0x5, 0x34, &mut mock_ram_interface);

        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_eq!(success, 0_u16);
        fs::remove_file(test_file_path).expect("Failed to clean up test file");
    }

    // test using the delete file functionality of the file device, verifying that it deletes the
    // file and that it sets the success flag correctly
    #[test]
    fn test_delete() {
        let mut mock_ram_interface = MockMainRamInterface::new();
        let mut file_device = FileDevice::new();

        // make a test file (of length 0x0001)
        let mut test_file_path = std::env::temp_dir();
        let test_file_name = format!("test_file_{}", Uuid::new_v4());
        test_file_path.push(test_file_name.clone());
        fs::write(&test_file_path, &[0xff; 0x0001]).expect("Failed to write test file");
        let test_file_path = test_file_path.into_os_string().into_string()
             .expect("could not convert file path into string");

        let read_values_to_return = test_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xcc, &mut mock_ram_interface);
        file_device.write(0x9, 0xcd, &mut mock_ram_interface);

        let mut expected_start_address = 0xcccd;
        let read_arguments_expected = test_file_path.bytes()
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

        // write to the delete port, and assert that success is not set 
        // to 0
        file_device.write(0x6, 0x01, &mut mock_ram_interface);

        let success = u16::from_be_bytes([
            file_device.read(0x2),
            file_device.read(0x3),
        ]);
        assert_ne!(success, 0_u16);

        // assert that the test file no longer exists
        let res = fs::metadata(&test_file_path);
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::NotFound); 
    }

    // test the write functionality of the file device, writing a file, overwriting it,
    // and then appending it. Check that each of these stages works as expected
    #[test]
    fn test_write() {
        let mut mock_ram_interface = MockMainRamInterface::new();
        let mut file_device = FileDevice::new();

        let mut test_file_path = std::env::temp_dir();
        let test_file_name = format!("test_file_{}", Uuid::new_v4());
        test_file_path.push(test_file_name);
        let test_file_path = test_file_path.into_os_string().into_string()
             .expect("could not convert file path into string");

        let read_values_to_return = test_file_path.bytes()
            .chain([0x0_u8,])
            .map(|b| Ok(vec!(b)))
            .collect::<VecDeque<_>>();
        mock_ram_interface.read_values_to_return = RefCell::new(
            read_values_to_return);

        // write to the file device, setting the address that the
        // file name should be read from
        file_device.write(0x8, 0xcc, &mut mock_ram_interface);
        file_device.write(0x9, 0xcd, &mut mock_ram_interface);

        let mut expected_start_address = 0xcccd;
        let read_arguments_expected = test_file_path.bytes()
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

        let test_file_contents = "first test file contents".to_string();

        // set length of memory area that the file contents should be read from
        let len = test_file_contents.len();
        let len = u16::try_from(len).unwrap();
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // prepare the ram interface to provide the contents of the file that is to
        // be written
        mock_ram_interface.read_values_to_return.get_mut().push_front(
            Ok(test_file_contents.clone().bytes().collect::<Vec<_>>()));

        // write to the addr(write) port
        file_device.write(0xe, 0x12, &mut mock_ram_interface);
        file_device.write(0xf, 0x34, &mut mock_ram_interface);

        // assert that the mock ram interface had its read method called with
        // the expected address and number of bytes to read
        assert_eq!(
            mock_ram_interface.read_arguments_received.get_mut().pop_back().unwrap(),
            (0x1234, len));

        // verify that the file actually exists and contains what is expected
        let contents = String::from_utf8(fs::read(&test_file_path).unwrap()).unwrap();
        assert_eq!(contents, test_file_contents);

        let test_file_contents = "second contents".to_string();

        // set length of memory area that the file contents should be read from
        let len = test_file_contents.len();
        let len = u16::try_from(len).unwrap();
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // prepare the ram interface to provide the contents of the file that is to
        // be written
        mock_ram_interface.read_values_to_return.get_mut().push_front(
            Ok(test_file_contents.clone().bytes().collect::<Vec<_>>()));

        // write to the addr(write) port
        file_device.write(0xe, 0x12, &mut mock_ram_interface);
        file_device.write(0xf, 0x34, &mut mock_ram_interface);

        // assert that the mock ram interface had its read method called with
        // the expected address and number of bytes to read
        assert_eq!(
            mock_ram_interface.read_arguments_received.get_mut().pop_back().unwrap(),
            (0x1234, len));

        // verify that the file actually exists and contains what is expected
        // (and the original file contents have been deleted)
        let contents = String::from_utf8(fs::read(&test_file_path).unwrap()).unwrap();
        assert_eq!(contents, test_file_contents);

        let appended_contents = " with something appended".to_string();
        // set length of memory area that the file contents should be read from
        let len = appended_contents.len();
        let len = u16::try_from(len).unwrap();
        file_device.write(0xa, len.to_be_bytes()[0], &mut mock_ram_interface);
        file_device.write(0xb, len.to_be_bytes()[1], &mut mock_ram_interface);

        // prepare the ram interface to provide the contents of the file that is to
        // be written
        mock_ram_interface.read_values_to_return.get_mut().push_front(
            Ok(appended_contents.clone().bytes().collect::<Vec<_>>()));

        // set the 'append' byte
        file_device.write(0x7, 0x1, &mut mock_ram_interface);

        // write to the addr(write) port
        file_device.write(0xe, 0x12, &mut mock_ram_interface);
        file_device.write(0xf, 0x34, &mut mock_ram_interface);

        // verify that the file actually exists and contains what is expected
        // (the original file contents with the new section appended)
        let contents = String::from_utf8(fs::read(&test_file_path).unwrap()).unwrap();
        assert_eq!(contents, "second contents with something appended");

        fs::remove_file(test_file_path).expect("Failed to clean up test file");
    }
}
