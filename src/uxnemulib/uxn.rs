use std::fmt;
use std::error::Error;
use crate::ops::OpObject;

pub const INIT_VECTOR: u16 = 0x100;

pub struct UxnImpl {
    ram: Vec<u8>,
    program_counter: u16,
    working_stack: Vec<u8>,
    return_stack: Vec<u8>,
}

#[derive(Debug)]
pub enum UxnError {
    InvalidMemoryAccess{address: u16},
}

impl fmt::Display for UxnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UxnError::InvalidMemoryAccess{address} => {
                write!(f, "attempt to access invalid memory address: {:x}", address)
            }
        }
    }
}

impl Error for UxnError {}

use crate::uxninterface::Uxn;

impl Uxn for UxnImpl {
    fn read_from_ram(&self, addr: u16) -> u8 {
        return self.ram[usize::from(addr)];
    }

    fn get_program_counter(&self) -> u16 {
        return self.program_counter;
    }

    fn set_program_counter(&mut self, addr: u16) {
        self.program_counter = addr;
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

        return Ok(UxnImpl{ram, program_counter:0, working_stack: Vec::new(),
        return_stack: Vec::new()});
    }

    pub fn run(&mut self, vector: u16) -> Result<(), UxnError>
    {
        self.set_program_counter(vector);
        loop {
            let instr = self.read_from_ram(self.get_program_counter());

            println!("executing {:x}", instr);
            println!("rst: {:?}", self.return_stack);
            println!("wst: {:?}", self.working_stack);

            if instr == 0x0 {
                return Ok(());
            }

            self.set_program_counter(self.get_program_counter() + 1);

            // parse instr into OpObject
            let op = OpObject::from_byte(instr);
 

            // call its handler
            op.execute(Box::new(self));
        }
    }
}
