use crate::ops::OpObject;

pub const INIT_VECTOR: u16 = 0x100;

pub struct UxnImpl {
    ram: Vec<u8>,
    program_counter: Result<u16, ()>,
    working_stack: Vec<u8>,
    return_stack: Vec<u8>,
}

use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

impl Uxn for UxnImpl {
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

impl UxnImpl {
    pub fn new<I>(rom: I) -> Result<Self, UxnError>
    where
        I: Iterator<Item = u8>,
    {
        let mut ram = vec![0x0; 0x10000];

        let init_vector: usize = INIT_VECTOR.into();
        for (ram_loc, val) in (&mut ram[init_vector..]).iter_mut().zip(rom).take(0x10000 - init_vector) {
            *ram_loc = val;
        }

        return Ok(UxnImpl{ram, program_counter:Ok(0), working_stack: Vec::new(),
        return_stack: Vec::new()});
    }

    pub fn run(&mut self, vector: u16) -> Result<(), UxnError>
    {
        self.set_program_counter(vector);
        loop {
            let instr = self.read_next_byte_from_ram()?;

            println!("executing {:x}", instr);

            if instr == 0x0 {
                return Ok(());
            }

            // parse instr into OpObject
            let op = OpObject::from_byte(instr);
 

            // call its handler
            op.execute(Box::new(self))?;

            println!("rst: {:?}", self.return_stack);
            println!("wst: {:?}", self.working_stack);
        }
    }
}
