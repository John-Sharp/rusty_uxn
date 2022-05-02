use crate::instruction::InstructionFactory;

pub const INIT_VECTOR: u16 = 0x100;

pub mod device; 
use device::{Device, DeviceList, DeviceWriteReturnCode, DeviceReadReturnCode};
use crate::uxnemulib::devices;
use crate::uxnemulib::devices::system::UxnSystemInterface;

struct UxnWithDevices<'a, J, K>
    where J: Uxn + UxnSystemInterface,
          K: DeviceList,
{
    uxn: &'a mut J,
    device_list: K,
}

impl <'a, J, K> Uxn for UxnWithDevices<'a, J, K>
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

    fn read_from_device(&mut self, device_address: u8) -> Result<u8, UxnError> {
        match self.device_list.read_from_device(device_address) {
            DeviceReadReturnCode::Success(res) => return res,
            DeviceReadReturnCode::ReadFromSystemDevice(port) => {
                let mut system = devices::system::System{uxn: self.uxn};
                return Ok(system.read(port));
            },
        }
    }

    fn write_to_device(&mut self, device_address: u8, val: u8) {
        match self.device_list.write_to_device(device_address, val) {
            DeviceWriteReturnCode::Success => {},
            DeviceWriteReturnCode::WriteToSystemDevice(port) => {
                let mut system = devices::system::System{uxn: self.uxn};
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

    // TODO remove
    fn read_from_device(&mut self, _device_address: u8) -> Result<u8, UxnError> {
        panic!("should not be called");
    }

    // TODO remove
    fn write_to_device(&mut self, _device_address: u8, _val: u8) {
        panic!("should not be called");
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

        return Ok(UxnImpl{ram, program_counter:Ok(0), working_stack: Vec::new(),
        return_stack: Vec::new(), instruction_factory});
    }

    // TODO pass in device list object to this function
    // execute with an object that implements Uxn but 
    // has owns mutable reference to device list. write_to_device
    // uses this list, all other functions are the same
    // at end of run the object goes out of scope
    pub fn run<K: DeviceList>(&mut self, vector: u16, devices: K) -> Result<(), UxnError>
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
}

impl<J> UxnSystemInterface for UxnImpl<J>
where
J: InstructionFactory,
{
    // TODO
    fn set_working_stack_index(&mut self, _index: u8) {
    }
}

impl<J> Device for UxnImpl<J>
where
J: InstructionFactory,
{
    fn write(&mut self, port: u8, val: u8) {
        let mut system = devices::system::System{uxn: self};

        return system.write(port, val);
    }

    fn read(&mut self, port: u8) -> u8 {
        let mut system = devices::system::System{uxn: self};

        return system.read(port);
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

    struct MockDeviceList {}
    impl DeviceList for MockDeviceList {
        fn write_to_device(&mut self, _device_address: u8, _val: u8) -> DeviceWriteReturnCode {
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
            MockInstructionFactory::new())?;
        uxn.run(0x102, MockDeviceList{})?;

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
        uxn.run(0xfffd, MockDeviceList{})?;

        // the instructions at addresses 0xfffd, 0xfffe, 0xffff should have been
        // executed
        assert_eq!(vec!(0xaa, 0xaa, 0xaa), *uxn.instruction_factory.ret_vec.borrow());
        Ok(())
    }
}
