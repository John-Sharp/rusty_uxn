use crate::ops::OpObject;

use crate::instruction::Instruction;
use crate::instruction::InstructionFactory;

pub const INIT_VECTOR: u16 = 0x100;

pub struct UxnImpl<J> 
   where J: InstructionFactory, 
{
    ram: Vec<u8>,
    program_counter: Result<u16, ()>,
    working_stack: Vec<u8>,
    return_stack: Vec<u8>,
    instruction_factory: J
}

use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

impl<J> Uxn for UxnImpl<J>
where
J: InstructionFactory
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
    fn push_to_return_stack(&mut self, byte: u8) {
        self.return_stack.push(byte);
    }

    fn push_to_working_stack(&mut self, byte: u8) {
        self.working_stack.push(byte);
    }
}

impl<J> UxnImpl<J> 
where
J: InstructionFactory
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

    pub fn run(&mut self, vector: u16) -> Result<(), UxnError>
    {
        self.set_program_counter(vector);
        loop {
            let instr = self.read_next_byte_from_ram();
            if instr == Err(UxnError::OutOfRangeMemoryAddress) {
                return Ok(());
            }
            let instr = instr.unwrap();

            println!("executing {:x}", instr);

            if instr == 0x0 {
                return Ok(());
            }

            // get the operation that the instruction represents
            let op = self.instruction_factory.from_byte(instr);

            // call its handler
            op.execute(Box::new(self))?;

            println!("rst: {:?}", self.return_stack);
            println!("wst: {:?}", self.working_stack);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    struct MockInstruction {
        byte: u8,
        ret_vec: Rc<RefCell<Vec<u8>>>,
    }
    impl Instruction for MockInstruction {
        fn execute(&self, uxn: Box::<&mut dyn Uxn>) -> Result<(), UxnError> {
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

    // test creating a UxnImpl and calling its run method with a typical
    // starting vector. Verify works as expected
    #[test]
    fn test_run_basic() -> Result<(), UxnError> {
        let rom : Vec<u8> = vec!(0xaa, 0xbb, 0xcc, 0xdd);

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new())?;
        uxn.run(0x102)?;

        assert_eq!(vec!(0xcc, 0xdd), *uxn.instruction_factory.ret_vec.borrow());
        Ok(())
    }

    // test calling UxnImpl::run with a ram configuration that reads right
    // to the end of the address space, verify that Ok is returned
    #[test]
    fn test_run_ram_full() -> Result<(), UxnError> {
        // note that this rom is larger than the portion of ram it is copied to,
        // it will just be truncated
        let rom : Vec<u8> = vec!(0xaa; (0x10000));

        let mut uxn = UxnImpl::new(
            rom.into_iter(),
            MockInstructionFactory::new())?;
        uxn.run(0xfffd)?;

        // the instructions at addresses 0xfffd, 0xfffe, 0xffff should have been
        // executed
        assert_eq!(vec!(0xaa, 0xaa, 0xaa), *uxn.instruction_factory.ret_vec.borrow());
        Ok(())
    }
}
