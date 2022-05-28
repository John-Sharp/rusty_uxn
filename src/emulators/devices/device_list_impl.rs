use super::super::uxn::device::{Device, DeviceList, DeviceWriteReturnCode, DeviceReadReturnCode, MainRamInterface, MainRamInterfaceError};
use crate::uxninterface::UxnError;
use std::collections::HashMap;
use std::io::Write;

pub enum DeviceEntry<'a, J> 
    where J: Write
{
    Device(&'a mut dyn Device),
    SystemPlaceHolder(J),
}

pub struct DeviceListImpl<'a, J> 
    where J: Write
{
    list: HashMap<u8, DeviceEntry<'a, J>>,
}

impl<'a, J> DeviceListImpl<'a, J> 
    where J: Write
{
    pub fn new(list: HashMap<u8, DeviceEntry<'a, J>>) -> Self {
        DeviceListImpl{list}
    }
}

impl<'a, J> DeviceList for DeviceListImpl<'a, J> 
    where J: Write
{
    type DebugWriter = J;

    fn write_to_device(&mut self, device_address: u8, val: u8, main_ram: &mut dyn MainRamInterface) -> DeviceWriteReturnCode<J> {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // look up correct device using index
        let device = match self.list.get_mut(&device_index) {
            // normal device
            Some(DeviceEntry::Device(device)) => device,

            // device is 'system' device so needs special handling by the calling context
            Some(DeviceEntry::SystemPlaceHolder(debug_printer)) => {
                return DeviceWriteReturnCode::WriteToSystemDevice(device_port, debug_printer);
            },

            // device not found under this index
            None => return DeviceWriteReturnCode::Success, // TODO return unrecognised device error?
        };

        // pass port and value through to device
        device.write(device_port, val, main_ram);

        return DeviceWriteReturnCode::Success;
    }

    fn read_from_device(&mut self, device_address: u8) -> DeviceReadReturnCode {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // look up correct device using index
        let device = match self.list.get_mut(&device_index) {
            // normal device
            Some(DeviceEntry::Device(device)) => device,

            // device is 'system' device so needs special handling by the calling context
            Some(DeviceEntry::SystemPlaceHolder(_)) => {
                return DeviceReadReturnCode::ReadFromSystemDevice(device_port);
            },

            // device not found under this index
            None => return DeviceReadReturnCode::Success(Err(UxnError::UnrecognisedDevice)),
        };

        return DeviceReadReturnCode::Success(Ok(device.read(device_port)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockDeviceA {
        pub write_arguments_received: RefCell<VecDeque<(u8, u8)>>,

        pub read_arguments_received: RefCell<VecDeque<(u8,)>>,
        pub read_values_to_return: RefCell<VecDeque<u8>>,
    }

    impl MockDeviceA {
        fn new() -> Self {
            MockDeviceA {
                write_arguments_received: RefCell::new(VecDeque::new()),
                read_arguments_received: RefCell::new(VecDeque::new()),
                read_values_to_return: RefCell::new(VecDeque::new()),
            }
        }

    }

    impl Device for MockDeviceA {
        fn write(&mut self, port: u8, val: u8, _main_ram: &mut dyn MainRamInterface) {
            self.write_arguments_received
                .borrow_mut()
                .push_back((port, val));
        }
        fn read(&mut self, port: u8) -> u8 {
            self.read_arguments_received
                .borrow_mut()
                .push_back((port,));

            return self
                .read_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }
    }

    struct MockDeviceB {
        pub write_arguments_received: RefCell<VecDeque<(u8, u8)>>,

        pub read_arguments_received: RefCell<VecDeque<(u8,)>>,
        pub read_values_to_return: RefCell<VecDeque<u8>>,
    }

    impl MockDeviceB {
        fn new() -> Self {
            MockDeviceB {
                write_arguments_received: RefCell::new(VecDeque::new()),
                read_arguments_received: RefCell::new(VecDeque::new()),
                read_values_to_return: RefCell::new(VecDeque::new()),
            }
        }

    }

    impl Device for MockDeviceB {
        fn write(&mut self, port: u8, val: u8, _main_ram: &mut dyn MainRamInterface) {
            self.write_arguments_received
                .borrow_mut()
                .push_back((port, val));
        }
        fn read(&mut self, port: u8) -> u8 {
            self.read_arguments_received
                .borrow_mut()
                .push_back((port,));

            return self
                .read_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }
    }

    struct MockMainRamInterface {}
    impl MainRamInterface for MockMainRamInterface {
        fn read(&self, address: u16, num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError> {
            panic!("should not be called");
        }
    }

    #[test]
    fn test_write() {
        let mut mock_device_a = MockDeviceA::new();
        let mut mock_device_b = MockDeviceB::new();

        let mut device_list = HashMap::new();
        device_list.insert(0x0, DeviceEntry::Device(&mut mock_device_a));
        device_list.insert(0x2, DeviceEntry::Device(&mut mock_device_b));
        device_list.insert(0x3, DeviceEntry::SystemPlaceHolder(Vec::new()));

        let mut device_list = DeviceListImpl::new(device_list);

        // write 23 to device 0x0, port 0xb
        let ret = device_list.write_to_device(0x0b, 23, &mut MockMainRamInterface{});
        assert_eq!(ret, DeviceWriteReturnCode::Success);


        // write 60 to device 0x2, port 0x4
        let ret = device_list.write_to_device(0x24, 60, &mut MockMainRamInterface{});
        assert_eq!(ret, DeviceWriteReturnCode::Success);

        // write 77 to device 0x3, port 0x9
        let ret = device_list.write_to_device(0x39, 77, &mut MockMainRamInterface{});

        match ret {
            DeviceWriteReturnCode::WriteToSystemDevice(0x9, debug_printer) => {
                // write a test string to the debug printer provided by
                // WriteToSystemDevice, will check this later
                writeln!(debug_printer, "this is a test").unwrap();
            },
            _ => panic!("write_to_device did not return what was expected"),
        }

        // check that the debug printer that was passed with the 'write to system device' command
        // was the correct one
        if let DeviceEntry::SystemPlaceHolder(debug_printer) = device_list.list.get_mut(&0x3).unwrap() {
            assert_eq!(debug_printer, "this is a test\n".as_bytes());
        } else {
            panic!("did not find expected device in device list slot 0x3");
        }

        // assert that the mock devices received the expected arguments
        assert_eq!(
            mock_device_a
                .write_arguments_received
                .into_inner(),
            VecDeque::from([(0xb, 23,),])
        );

        assert_eq!(
            mock_device_b
                .write_arguments_received
                .into_inner(),
            VecDeque::from([(0x4, 60,),])
        );
    }

    #[test]
    fn test_read() {
        let mut mock_device_a = MockDeviceA::new();
        mock_device_a.read_values_to_return = RefCell::new(VecDeque::from([
            0x12,
        ]));
        let mut mock_device_b = MockDeviceB::new();
        mock_device_b.read_values_to_return = RefCell::new(VecDeque::from([
            0x34,
        ]));


        let mut device_list = HashMap::new();
        device_list.insert(0x0, DeviceEntry::Device(&mut mock_device_a));
        device_list.insert(0x2, DeviceEntry::Device(&mut mock_device_b));
        device_list.insert(0x3, DeviceEntry::SystemPlaceHolder(Vec::new()));

        let mut device_list = DeviceListImpl::new(device_list);

        // read from device 0x0, port 0xb
        let ret = device_list.read_from_device(0x0b);
        assert_eq!(ret, DeviceReadReturnCode::Success(Ok(0x12)));

        // read from device 0x2, port 0x4
        let ret = device_list.read_from_device(0x24);
        assert_eq!(ret, DeviceReadReturnCode::Success(Ok(0x34)));

        // read from device 0x3, port 0x9
        let ret = device_list.read_from_device(0x39);
        assert_eq!(ret, DeviceReadReturnCode::ReadFromSystemDevice(0x9));

        // read from unknown device
        let ret = device_list.read_from_device(0x59);
        assert_eq!(ret, DeviceReadReturnCode::Success(Err(UxnError::UnrecognisedDevice)));

        // assert that the mock devices received the expected arguments
        assert_eq!(
            mock_device_a
                .read_arguments_received
                .into_inner(),
            VecDeque::from([(0xb,),])
        );

        assert_eq!(
            mock_device_b
                .read_arguments_received
                .into_inner(),
            VecDeque::from([(0x4,),])
        );
    }
}
