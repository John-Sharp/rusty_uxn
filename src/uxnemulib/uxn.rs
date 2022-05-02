use crate::instruction::InstructionFactory;

use std::collections::HashMap;

pub const INIT_VECTOR: u16 = 0x100;

pub mod device; 
use device::Device;

struct UxnWithDevices<'a, J>
    where J: Uxn,
{
    uxn: &'a mut J,
    device_list: HashMap<u8, &'a mut dyn Device>,
}

impl <'a, J> Uxn for UxnWithDevices<'a, J>
    where J: Uxn,
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

    fn read_from_device(&mut self, device_address: u8) -> Result<u8, UxnError> {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // look up correct device using index
        let device = match self.device_list.get_mut(&device_index) {
            Some(device) => device,
            None => return Err(UxnError::UnrecognisedDevice),
        };

        return Ok(device.read(device_port));
    }

    fn write_to_device(&mut self, device_address: u8, val: u8) {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // look up correct device using index
        let device = match self.device_list.get_mut(&device_index) {
            Some(device) => device,
            None => return, // TODO return unrecognised device error?
        };

        // pass port and value through to device
        device.write(device_port, val);
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
    devices: HashMap<u8, Box<dyn Device>>,
}

use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

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

    fn read_from_device(&mut self, device_address: u8) -> Result<u8, UxnError> {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // look up correct device using index
        let device = match self.devices.get_mut(&device_index) {
            Some(device) => device,
            None => return Err(UxnError::UnrecognisedDevice),
        };

        return Ok(device.read(device_port));
    }

    fn write_to_device(&mut self, device_address: u8, val: u8) {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // look up correct device using index
        let device = match self.devices.get_mut(&device_index) {
            Some(device) => device,
            None => return, // TODO return unrecognised device error?
        };

        // pass port and value through to device
        device.write(device_port, val);
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

        let devices = HashMap::new();

        return Ok(UxnImpl{ram, program_counter:Ok(0), working_stack: Vec::new(),
        return_stack: Vec::new(), instruction_factory, devices});
    }

    // TODO pass in device list object to this function
    // execute with an object that implements Uxn but 
    // has owns mutable reference to device list. write_to_device
    // uses this list, all other functions are the same
    // at end of run the object goes out of scope
    pub fn run<'a>(&'a mut self, vector: u16, devices: HashMap<u8, &'a mut dyn Device>) -> Result<(), UxnError>
    {
        self.set_program_counter(vector);

        let mut uxn_with_devices = UxnWithDevices {
            uxn: self,
            device_list: devices,
        };

        loop {
            let instr = uxn_with_devices.read_next_byte_from_ram();
            if instr == Err(UxnError::OutOfRangeMemoryAddress) {
                return Ok(());
            }
            let instr = instr.unwrap();

            if instr == 0x0 {
                return Ok(());
            }

            // get the operation that the instruction represents
            let op = uxn_with_devices.uxn.instruction_factory.from_byte(instr);

            // call its handler
            // TODO I don't think I need a box around this
            op.execute(Box::new(&mut uxn_with_devices))?;
        }
    }

    pub fn add_device(&mut self, device_index: u8, device: Box<dyn Device>) {
        self.devices.insert(
            device_index,
            device);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::instruction::Instruction;
    use std::collections::VecDeque;

    struct MockInstruction {
        byte: u8,
        ret_vec: Rc<RefCell<Vec<u8>>>,
    }
    impl Instruction for MockInstruction {
        fn execute(&self, _uxn: Box::<&mut dyn Uxn>) -> Result<(), UxnError> {
            self.ret_vec.borrow_mut().push(self.byte);
            Ok(())
        }
    }

    struct MockInstructionFactory {
        ret_vec : Rc<RefCell<Vec<u8>>>,
    }
    impl MockInstructionFactory {
        fn new() -> Self {
            MockInstructionFactory{ret_vec: Rc::new(RefCell::new(Vec::<u8>::new())),}
        }
    }
    impl InstructionFactory for MockInstructionFactory {
        fn from_byte(&self, byte: u8) -> Box<dyn Instruction> {
            return Box::new(MockInstruction{byte: byte, ret_vec: Rc::clone(&self.ret_vec)});
        }
    }
    struct MockDevice {
        write_to_device_arguments_received: Rc<RefCell<Vec<(u8, u8)>>>,

        read_from_device_arguments_received: Rc<RefCell<Vec<(u8,)>>>,
        read_from_device_values_to_return: Rc<RefCell<VecDeque<u8>>>,
    }

    impl MockDevice {
        fn new(write_to_device_arguments_received: Rc<RefCell<Vec<(u8, u8)>>>) -> Self {
           MockDevice{
               write_to_device_arguments_received,
               read_from_device_arguments_received: Rc::new(RefCell::new(Vec::new())),
               read_from_device_values_to_return: Rc::new(RefCell::new(VecDeque::new())),
           } 
        }
    }

    impl Device for MockDevice {
        fn write(&mut self, port: u8, val: u8) {
            self.write_to_device_arguments_received.borrow_mut().push((port, val));
        }

        fn read(&mut self, port: u8) -> u8 {
            self.read_from_device_arguments_received.borrow_mut().push((port,));
            return self.read_from_device_values_to_return.borrow_mut().pop_front().unwrap();
        }
    }


    // test creating a UxnImpl and calling its run method with a typical
    // starting vector. Verify works as expected
    #[test]
    fn test_run_basic() -> Result<(), UxnError> {
        let rom : Vec<u8> = vec!(0xaa, 0xbb, 0xcc, 0xdd);

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new())?;
        uxn.run(0x102, HashMap::new())?;

        assert_eq!(vec!(0xcc, 0xdd), *uxn.instruction_factory.ret_vec.borrow());
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
            MockInstructionFactory::new())?;
        uxn.run(0xfffd, HashMap::new())?;

        // the instructions at addresses 0xfffd, 0xfffe, 0xffff should have been
        // executed
        assert_eq!(vec!(0xaa, 0xaa, 0xaa), *uxn.instruction_factory.ret_vec.borrow());
        Ok(())
    }

    // test that adding a device and then attempting to write to that device leads
    // to the correct port and value being passed through to the device
    #[test]
    fn test_write_to_device() -> Result<(), UxnError> {
        let rom = Vec::new();

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new())?;

        // vector for holding arguments passed to device's write method
        let write_to_device_arguments_received = Rc::new(RefCell::new(Vec::new()));

        // add the device at index 0x0
        uxn.add_device(0x0, Box::new(MockDevice::new(write_to_device_arguments_received.clone())));

        // pass value 0x22 to port 0xa of device at index 0x0
        uxn.write_to_device(0x0a, 0x22);

        // check that write method of device was called with correct port and 
        // value
        assert_eq!(vec!((0x0a, 0x22)), *write_to_device_arguments_received.borrow());

        Ok(())
    }

    // identical to MockDevice, but a different type
    struct MockDeviceB {
        write_to_device_arguments_received: Rc<RefCell<Vec<(u8, u8)>>>,

        read_from_device_arguments_received: Rc<RefCell<Vec<(u8,)>>>,
        read_from_device_values_to_return: Rc<RefCell<VecDeque<u8>>>,
    }

    impl MockDeviceB {
        fn new(write_to_device_arguments_received: Rc<RefCell<Vec<(u8, u8)>>>) -> Self {
           MockDeviceB{
               write_to_device_arguments_received,
               read_from_device_arguments_received: Rc::new(RefCell::new(Vec::new())),
               read_from_device_values_to_return: Rc::new(RefCell::new(VecDeque::new())),
           } 
        }
    }

    impl Device for MockDeviceB {
        fn write(&mut self, port: u8, val: u8) {
            self.write_to_device_arguments_received.borrow_mut().push((port, val));
        }

        fn read(&mut self, port: u8) -> u8 {
            self.read_from_device_arguments_received.borrow_mut().push((port,));
            return self.read_from_device_values_to_return.borrow_mut().pop_front().unwrap();
        }
    }

    // test having two devices (of different types) and check writing to them
    // leads to correct write methods being called
    #[test]
    fn test_write_to_devices() -> Result<(), UxnError> {
        let rom = Vec::new();

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new())?;

        // vector for arguments to write method of first device
        let write_to_device_arguments_received = Rc::new(RefCell::new(Vec::new()));

        // vector for arguments to write method of second device
        let write_to_device_arguments_received_b = Rc::new(RefCell::new(Vec::new()));

        // add first device (of type MockDevice) at index 0x0
        uxn.add_device(0x0, Box::new(MockDevice::new(write_to_device_arguments_received.clone())));

        // add second device (of type MockDeviceB) at index 0xb
        uxn.add_device(0xb, Box::new(MockDeviceB::new(write_to_device_arguments_received_b.clone())));

        // write 0x22 to port 0xa of device with index 0x0, write 0x24 to port 0x1 of device with
        // index 0xb
        uxn.write_to_device(0x0a, 0x22);
        uxn.write_to_device(0xb1, 0x24);

        assert_eq!(vec!((0x0a, 0x22)), *write_to_device_arguments_received.borrow());
        assert_eq!(vec!((0x01, 0x24)), *write_to_device_arguments_received_b.borrow());

        Ok(())
    }
}
