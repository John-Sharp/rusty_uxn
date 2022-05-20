use crate::instruction::InstructionFactory;

pub const INIT_VECTOR: u16 = 0x100;

pub mod device; 
use device::{Device, DeviceList, DeviceWriteReturnCode, DeviceReadReturnCode};
use crate::emulators::devices;
use crate::emulators::devices::system::{UxnSystemInterface, UxnSystemColor};
use crate::uxninterface::{Uxn, UxnError, UxnStatus, UxnWithDevices};

struct UxnWithDevicesImpl<'a, J, K>
    where J: Uxn + UxnSystemInterface,
          K: DeviceList,
{
    uxn: &'a mut J,
    device_list: K,
}

impl <'a, J, K> Uxn for UxnWithDevicesImpl<'a, J, K>
    where J: Uxn + UxnSystemInterface,
          K: DeviceList,
{
    fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
        return self.uxn.read_next_byte_from_ram();
    }

    fn read_from_ram(&self, addr: u16) -> u8 {
        return self.uxn.read_from_ram(addr);
    }

    fn write_to_ram(&mut self, addr: u16, val: u8) {
        return self.uxn.write_to_ram(addr, val);
    }

    fn get_program_counter(&self) -> Result<u16, UxnError> {
        return self.uxn.get_program_counter();
    }

    fn set_program_counter(&mut self, addr: u16) {
        return self.uxn.set_program_counter(addr);
    }

    fn push_to_return_stack(&mut self, byte: u8) -> Result<(), UxnError> {
        return self.uxn.push_to_return_stack(byte);
    }

    fn push_to_working_stack(&mut self, byte: u8) -> Result<(), UxnError> {
        return self.uxn.push_to_working_stack(byte);
    }

    fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
        return self.uxn.pop_from_working_stack();
    }

    fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
        return self.uxn.pop_from_return_stack();
    }
}

impl <'a, J, K> UxnWithDevices for UxnWithDevicesImpl<'a, J, K>
    where J: Uxn + UxnSystemInterface,
          K: DeviceList,
{
    fn read_from_device(&mut self, device_address: u8) -> Result<u8, UxnError> {
        match self.device_list.read_from_device(device_address) {
            DeviceReadReturnCode::Success(res) => return res,
            DeviceReadReturnCode::ReadFromSystemDevice(port) => {
                let mut temp_writer = Vec::new();
                let mut system = devices::system::System::new(self.uxn, &mut temp_writer);
                return Ok(system.read(port));
            },
        }
    }

    fn write_to_device(&mut self, device_address: u8, val: u8) {
        match self.device_list.write_to_device(device_address, val) {
            DeviceWriteReturnCode::Success => {},
            DeviceWriteReturnCode::WriteToSystemDevice(port, debug_printer) => {
                let mut system = devices::system::System::new(self.uxn, debug_printer);
                system.write(port, val);
            },
        }
    }
}

pub struct UxnImpl<J> 
   where J: InstructionFactory, 
{
    ram: Vec<u8>,
    program_counter: Result<u16, ()>,
    working_stack: Vec<u8>,
    return_stack: Vec<u8>,
    instruction_factory: J,
    system_colors: [u8;6],
    should_terminate: bool,
}

impl<J> Uxn for UxnImpl<J>
where
J: InstructionFactory,
{
    fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
        if self.program_counter.is_err() {
            return Err(UxnError::OutOfRangeMemoryAddress);
        }
        let program_counter = self.program_counter.unwrap();
        let ret = self.ram[usize::from(program_counter)];

        if program_counter == u16::MAX {
            self.program_counter = Err(());
        } else {
            self.program_counter = Ok(program_counter+1);
        }

        return Ok(ret);
    }

    fn read_from_ram(&self, addr: u16) -> u8 {
        return self.ram[usize::from(addr)];
    }

    fn write_to_ram(&mut self, addr: u16, val: u8) {
        self.ram[usize::from(addr)] = val;
    }

    fn get_program_counter(&self) -> Result<u16, UxnError> {
        if self.program_counter.is_err() {
            return Err(UxnError::OutOfRangeMemoryAddress);    
        }
        return Ok(self.program_counter.unwrap());
    }

    fn set_program_counter(&mut self, addr: u16) {
        self.program_counter = Ok(addr);
    }

    // TODO check for stack overflow
    fn push_to_return_stack(&mut self, byte: u8) -> Result<(), UxnError> {
        self.return_stack.push(byte);
        Ok(())
    }

    fn push_to_working_stack(&mut self, byte: u8) -> Result<(), UxnError> {
        self.working_stack.push(byte);
        Ok(())
    }

    // TODO check for stack underflow
    fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
        Ok(self.working_stack.pop().unwrap())
    }

    fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
        Ok(self.return_stack.pop().unwrap())
    }
}

impl<J> UxnImpl<J> 
where
J: InstructionFactory,
{
    pub fn new<I>(rom: I, instruction_factory: J) -> Result<Self, UxnError>
    where
        I: Iterator<Item = u8>,
        J: InstructionFactory,
    {
        let mut ram = vec![0x0; 0x10000];

        let init_vector: usize = INIT_VECTOR.into();
        for (ram_loc, val) in (&mut ram[init_vector..]).iter_mut().zip(rom).take(0x10000 - init_vector) {
            *ram_loc = val;
        }

        // TODO figure out the default colors
        let system_colors = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

        let should_terminate = false;

        return Ok(UxnImpl{ram, program_counter:Ok(0), working_stack: Vec::new(),
        return_stack: Vec::new(), instruction_factory, system_colors, should_terminate});
    }

    // TODO pass in device list object to this function
    // execute with an object that implements Uxn but 
    // has owns mutable reference to device list. write_to_device
    // uses this list, all other functions are the same
    // at end of run the object goes out of scope
    pub fn run<K: DeviceList>(&mut self, vector: u16, devices: K) -> Result<UxnStatus, UxnError>
    {
        self.set_program_counter(vector);

        let mut uxn_with_devices = UxnWithDevicesImpl {
            uxn: self,
            device_list: devices,
        };

        loop {
            let instr = uxn_with_devices.read_next_byte_from_ram();
            if instr == Err(UxnError::OutOfRangeMemoryAddress) {
                return Ok(UxnStatus::Halt);
            }
            let instr = instr.unwrap();

            if instr == 0x0 {
                return Ok(UxnStatus::Halt);
            }

            // get the operation that the instruction represents
            let op = uxn_with_devices.uxn.instruction_factory.from_byte(instr);

            // call its handler
            // TODO I don't think I need a box around this
            op.execute(Box::new(&mut uxn_with_devices))?;

            if uxn_with_devices.uxn.should_terminate {
                return Ok(UxnStatus::Terminate);
            }
        }
    }
}

fn system_color_to_index(system_color: UxnSystemColor) -> usize {
    match system_color {
        UxnSystemColor::Red1 => 0,
        UxnSystemColor::Red2 => 1,
        UxnSystemColor::Green1 => 2,
        UxnSystemColor::Green2 => 3,
        UxnSystemColor::Blue1 => 4,
        UxnSystemColor::Blue2 => 5,
    }
}

impl<J> UxnSystemInterface for UxnImpl<J>
where
J: InstructionFactory,
{
    fn set_working_stack_index(&mut self, index: u8) {
        self.working_stack.resize(index.into(), 0);
    }

    fn get_working_stack_index(&self) -> u8 {
        u8::try_from(self.working_stack.len()).unwrap()
    }

    fn set_return_stack_index(&mut self, index: u8) {
        self.return_stack.resize(index.into(), 0);
    }

    fn get_return_stack_index(&self) -> u8 {
        u8::try_from(self.return_stack.len()).unwrap()
    }

    fn set_system_color(&mut self, slot: UxnSystemColor, val: u8) {
        self.system_colors[system_color_to_index(slot)] = val;
    }
    
    fn get_system_color(&self, slot: UxnSystemColor) -> u8 {
        self.system_colors[system_color_to_index(slot)]
    }

    fn start_termination(&mut self) {
        self.should_terminate = true;
    }

    fn get_working_stack_iter(&self) -> std::slice::Iter<u8> {
        self.working_stack.iter()
    }

    fn get_return_stack_iter(&self) -> std::slice::Iter<u8> {
        self.return_stack.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::instruction::Instruction;

    struct MockInstruction {
        byte: u8,
        ret_vec: Rc<RefCell<Vec<u8>>>,
        is_terminate_instruction: bool,
    }
    impl Instruction for MockInstruction {
        fn execute(&self, uxn: Box::<&mut dyn UxnWithDevices>) -> Result<(), UxnError> {
            if self.is_terminate_instruction {
                uxn.write_to_device(0x99, 0x99);
            }


            self.ret_vec.borrow_mut().push(self.byte);
            Ok(())
        }
    }

    struct MockInstructionFactory {
        ret_vec : Rc<RefCell<Vec<u8>>>,
        terminate_instruction: u8, // special byte, that when encountered will lead to 
                                   // 'start_termination' being called on the uxn
    }
    impl MockInstructionFactory {
        fn new(terminate_instruction: u8) -> Self {
            MockInstructionFactory{ret_vec: Rc::new(RefCell::new(Vec::<u8>::new())),
            terminate_instruction}
        }
    }
    impl InstructionFactory for MockInstructionFactory {
        fn from_byte(&self, byte: u8) -> Box<dyn Instruction> {
            let is_terminate_instruction = if byte == self.terminate_instruction {
                true
            } else {
                false
            };
            return Box::new(MockInstruction{byte: byte, ret_vec: Rc::clone(&self.ret_vec), is_terminate_instruction});
        }
    }

    struct MockDeviceList {
        err_writer: Vec<u8>,
    }

    impl MockDeviceList {
        fn new() -> Self {
            MockDeviceList{err_writer: Vec::new()}
        }
    }

    impl DeviceList for MockDeviceList {
        type DebugWriter = Vec<u8>;

        fn write_to_device(&mut self, device_address: u8, val: u8) -> DeviceWriteReturnCode<Self::DebugWriter> {

            // special code in integration tests to trigger termination
            if device_address == 0x99 && val == 0x99 {
                return DeviceWriteReturnCode::WriteToSystemDevice(0xf, &mut self.err_writer);
            }

            return DeviceWriteReturnCode::Success;
        }

        fn read_from_device(&mut self, _device_address: u8) -> DeviceReadReturnCode {
            return DeviceReadReturnCode::Success(Ok(0));
        }
    }


    // test creating a UxnImpl and calling its run method with a typical
    // starting vector. Verify works as expected
    #[test]
    fn test_run_basic() -> Result<(), UxnError> {
        let rom : Vec<u8> = vec!(0xaa, 0xbb, 0xcc, 0xdd);

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new(0xff))?;
        let res = uxn.run(0x102, MockDeviceList::new())?;

        assert_eq!(vec!(0xcc, 0xdd), *uxn.instruction_factory.ret_vec.borrow());

        assert_eq!(UxnStatus::Halt, res);

        Ok(())
    }

    // test calling UxnImpl::run with a ram configuration that reads right
    // to the end of the address space, verify that Ok is returned
    #[test]
    fn test_run_ram_full() -> Result<(), UxnError> {
        // note that this rom is larger than the portion of ram it is copied to,
        // it will just be truncated
        let rom : Vec<u8> = vec!(0xaa; 0x10000);

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new(0xff))?;
        let res = uxn.run(0xfffd, MockDeviceList::new())?;

        // the instructions at addresses 0xfffd, 0xfffe, 0xffff should have been
        // executed
        assert_eq!(vec!(0xaa, 0xaa, 0xaa), *uxn.instruction_factory.ret_vec.borrow());

        assert_eq!(UxnStatus::Halt, res);

        Ok(())
    }
    
    #[test]
    fn test_run_terminate() -> Result<(), UxnError> {
        // 4th byte is terminate byte, so program should stop there
        let rom : Vec<u8> = vec!(0xaa, 0xbb, 0xcc, 0xff, 0xdd);

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new(0xff))?;

        let res = uxn.run(0x100, MockDeviceList::new())?;

        assert_eq!(vec!(0xaa, 0xbb, 0xcc, 0xff), *uxn.instruction_factory.ret_vec.borrow());

        assert_eq!(UxnStatus::Terminate, res);

        Ok(())
    }

    #[test]
    fn test_read_write_normal_device() {
        struct MockUxn {}
        impl Uxn for MockUxn {
            fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
                panic!("should not be called");
            }

            fn read_from_ram(&self, _addr: u16) -> u8 {
                panic!("should not be called");
            }

            fn write_to_ram(&mut self, _addr: u16, _val: u8) {
                panic!("should not be called");
            }

            fn get_program_counter(&self) -> Result<u16, UxnError> {
                panic!("should not be called");
            }

            fn set_program_counter(&mut self, _addr: u16) {
                panic!("should not be called");
            }

            fn push_to_return_stack(&mut self, _byte: u8) -> Result<(), UxnError> {
                panic!("should not be called");
            }

            fn push_to_working_stack(&mut self, _byte: u8) -> Result<(), UxnError> {
                panic!("should not be called");
            }

            fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
                panic!("should not be called");
            }

            fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
                panic!("should not be called");
            }
        }
        impl UxnSystemInterface for MockUxn {
            fn set_working_stack_index(&mut self, _index: u8) {
                panic!("should not be called");
            }

            fn get_working_stack_index(&self) -> u8 {
                panic!("should not be called");
            }

            fn set_return_stack_index(&mut self, _index: u8) {
                panic!("should not be called");
            }

            fn get_return_stack_index(&self) -> u8 {
                panic!("should not be called");
            }

            fn set_system_color(&mut self, _slot: UxnSystemColor, _val: u8) {
                panic!("should not be called");
            }

            fn get_system_color(&self, _slot: UxnSystemColor) -> u8 {
                panic!("should not be called");
            }

            fn start_termination(&mut self) {
                panic!("should not be called");
            }

            fn get_working_stack_iter(&self) -> std::slice::Iter<u8> {
                panic!("should not be called");
            }

            fn get_return_stack_iter(&self) -> std::slice::Iter<u8> {
                panic!("should not be called");
            }
        }

        struct MockDeviceList {}

        impl DeviceList for MockDeviceList {
            type DebugWriter = Vec<u8>;

            fn write_to_device(&mut self, device_address: u8, val: u8) -> DeviceWriteReturnCode<Self::DebugWriter> {
                assert_eq!(device_address, 0x35);
                assert_eq!(val, 0x22);

                return DeviceWriteReturnCode::Success;
            }

            fn read_from_device(&mut self, device_address: u8) -> DeviceReadReturnCode {
                assert_eq!(device_address, 0x58);
                return DeviceReadReturnCode::Success(Ok(0x14));
            }
        }

        let mut uxn_with_devices = UxnWithDevicesImpl{uxn: &mut MockUxn{}, device_list: MockDeviceList{}};

        // test read_from_device, MockDeviceList::read_from_device should be passed the
        // correct arguments
        let res = uxn_with_devices.read_from_device(0x58);

        assert_eq!(res, Ok(0x14));

        // test write_to_device, MockDeviceList::write_to_device should be passed the
        // correct arguments
        uxn_with_devices.write_to_device(0x35, 0x22);
    }

    #[test]
    fn test_read_write_system_device() {
        struct MockUxn {
            set_working_stack_index_called: bool,
            mock_working_stack: Vec<u8>,
            mock_return_stack: Vec<u8>,
        }
        impl Uxn for MockUxn {
            fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
                panic!("should not be called");
            }

            fn read_from_ram(&self, _addr: u16) -> u8 {
                panic!("should not be called");
            }

            fn write_to_ram(&mut self, _addr: u16, _val: u8) {
                panic!("should not be called");
            }

            fn get_program_counter(&self) -> Result<u16, UxnError> {
                panic!("should not be called");
            }

            fn set_program_counter(&mut self, _addr: u16) {
                panic!("should not be called");
            }

            fn push_to_return_stack(&mut self, _byte: u8) -> Result<(), UxnError> {
                panic!("should not be called");
            }

            fn push_to_working_stack(&mut self, _byte: u8) -> Result<(), UxnError> {
                panic!("should not be called");
            }

            fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
                panic!("should not be called");
            }

            fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
                panic!("should not be called");
            }
        }
        impl UxnSystemInterface for MockUxn {
            fn set_working_stack_index(&mut self, index: u8) {
                self.set_working_stack_index_called = true;
                assert_eq!(index, 0x96);
            }

            fn get_working_stack_index(&self) -> u8 {
                return 0x91;
            }

            fn set_return_stack_index(&mut self, _index: u8) {
                panic!("should not be called");
            }

            fn get_return_stack_index(&self) -> u8 {
                panic!("should not be called");
            }

            fn set_system_color(&mut self, _slot: UxnSystemColor, _val: u8) {
                panic!("should not be called");
            }

            fn get_system_color(&self, _slot: UxnSystemColor) -> u8 {
                panic!("should not be called");
            }

            fn start_termination(&mut self) {
                panic!("should not be called");
            }

            fn get_working_stack_iter(&self) -> std::slice::Iter<u8> {
                return self.mock_working_stack.iter();
            }

            fn get_return_stack_iter(&self) -> std::slice::Iter<u8> {
                return self.mock_return_stack.iter();
            }
        }

        struct MockDeviceList {
            debug_printer: Vec<u8>,
            expected_device_address: u8,
            expected_val: u8,
        }

        impl MockDeviceList {
            fn new() -> Self {
                MockDeviceList {
                    debug_printer: Vec::new(),
                    expected_device_address: 0,
                    expected_val: 0,
                }
            }
        }

        impl DeviceList for MockDeviceList {
            type DebugWriter = Vec<u8>;


            fn write_to_device(&mut self, device_address: u8, val: u8) -> DeviceWriteReturnCode<Self::DebugWriter> {
                assert_eq!(device_address, self.expected_device_address);
                assert_eq!(val, self.expected_val);

                return DeviceWriteReturnCode::WriteToSystemDevice(device_address & 0xf, &mut self.debug_printer);
            }

            fn read_from_device(&mut self, device_address: u8) -> DeviceReadReturnCode {
                assert_eq!(device_address, 0x42);
                return DeviceReadReturnCode::ReadFromSystemDevice(0x2);
            }
        }

        let mut uxn_with_devices = UxnWithDevicesImpl{
            uxn: &mut MockUxn{set_working_stack_index_called: false, mock_working_stack: vec![1,2,3], mock_return_stack: vec![4,5,6]},
            device_list: MockDeviceList::new()};

        // test write_to_device, MockDeviceList::write_to_device should be passed the correct
        // arguments and since it returns WriteToSystemDevice and the device address ends in the
        // nibble 0x2 then the System device should result in
        // UxnSystemInterface::set_working_stack_index being called with the value 0x96
        uxn_with_devices.device_list.expected_device_address = 0x32;
        uxn_with_devices.device_list.expected_val = 0x96;
        uxn_with_devices.write_to_device(0x32, 0x96);
        assert_eq!(uxn_with_devices.uxn.set_working_stack_index_called, true);


        // test read_from_device, MockDeviceList::read_from_device should be passed the correct
        // arguments and since it returns ReadFromSystemDevice and the device address ends in the
        // nibble 0x2 then the System device should result in
        // UxnSystemInterface::get_working_stack_index being called with the value 0x91
        let ret = uxn_with_devices.read_from_device(0x42);
        assert_eq!(ret, Ok(0x91));


        // test write_to device with the System device asking to print the debug string.
        // Verify that the debug printer returned by the MockDeviceList is actually 
        // used for the debug string printing
        uxn_with_devices.device_list.expected_device_address = 0x3e;
        uxn_with_devices.device_list.expected_val = 0x11;
        uxn_with_devices.write_to_device(0x3e, 0x11);
        assert_eq!(uxn_with_devices.device_list.debug_printer,
                   "<wst> 01 02 03\n<rst> 04 05 06\n".as_bytes());
    }

    #[test]
    fn test_set_get_stack_index() -> Result<(), UxnError> {
        let mut uxn = UxnImpl::new(
            vec!().into_iter(),
            MockInstructionFactory::new(0xff))?;

        uxn.push_to_working_stack(0x2)?;
        uxn.push_to_working_stack(0x3)?;

        assert_eq!(uxn.get_working_stack_index(), 2);

        uxn.set_working_stack_index(6);

        assert_eq!(uxn.working_stack, vec!(0x2, 0x3, 0x0, 0x0, 0x0, 0x0));

        uxn.set_working_stack_index(1);

        assert_eq!(uxn.working_stack, vec!(0x2,));

        uxn.push_to_return_stack(0x4)?;
        uxn.push_to_return_stack(0x6)?;
        uxn.push_to_return_stack(0x3)?;

        assert_eq!(uxn.get_return_stack_index(), 3);

        uxn.set_return_stack_index(5);

        assert_eq!(uxn.return_stack, vec!(0x4, 0x6, 0x3, 0x0, 0x0,));

        uxn.set_return_stack_index(1);

        assert_eq!(uxn.return_stack, vec!(0x4,));

        Ok(())
    }

    #[test]
    fn test_set_get_system_color() -> Result<(), UxnError> {
        let mut uxn = UxnImpl::new(
            vec!().into_iter(),
            MockInstructionFactory::new(0xff))?;

        let test_cases = [
            (UxnSystemColor::Red1, 12),
            (UxnSystemColor::Red2, 34),
            (UxnSystemColor::Green1, 56),
            (UxnSystemColor::Green2, 78),
            (UxnSystemColor::Blue1, 90),
            (UxnSystemColor::Blue2, 123),
        ];

        for &(system_color, val) in &test_cases {
            assert_eq!(uxn.get_system_color(system_color), 0);
            uxn.set_system_color(system_color, val);
            assert_eq!(uxn.get_system_color(system_color), val);
        }

        Ok(())
    }

    #[test]
    fn test_get_stack_iter() -> Result<(), UxnError> {
        let mut uxn = UxnImpl::new(
            vec!().into_iter(),
            MockInstructionFactory::new(0xff))?;

        uxn.push_to_working_stack(0x2)?;
        uxn.push_to_working_stack(0x3)?;
        uxn.push_to_working_stack(0x4)?;

        for (&v, expected) in uxn.get_working_stack_iter().zip([0x2, 0x3, 0x4,]) {
            assert_eq!(v, expected);
        }

        uxn.push_to_return_stack(0x5)?;
        uxn.push_to_return_stack(0x6)?;
        uxn.push_to_return_stack(0x7)?;

        for (&v, expected) in uxn.get_return_stack_iter().zip([0x5, 0x6, 0x7,]) {
            assert_eq!(v, expected);
        }

        Ok(())
    }
}
