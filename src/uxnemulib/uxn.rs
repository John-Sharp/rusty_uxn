use std::fmt;
use std::error::Error;

pub const INIT_VECTOR: u16 = 0x100;

pub struct Uxn {
    ram: Vec<u8>,
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

impl Uxn {
    pub fn new<I>(rom: I) -> Result<Self, UxnError>
    where
        I: Iterator<Item = u8>,
    {
        let mut ram = vec![0x0; 0x10000];

        let init_vector: usize = INIT_VECTOR.into();
        for (ram_loc, val) in (&mut ram[init_vector..]).iter_mut().zip(rom).take(0x10000 - init_vector) {
            *ram_loc = val;
        }

        return Ok(Uxn{ram});
    }

    pub fn run(&self, vector: u16) -> Result<(), UxnError>
    {
        let mut program_counter: usize = vector.into();
        loop {
            let instr = match self.ram.get(program_counter) {
                Some(instr) => *instr,
                None => { return Err(UxnError::InvalidMemoryAccess{address: vector}); }
            };

            if instr == 0x0 {
                return Ok(());
            }

            println!("need to execute {:x}", instr);
            program_counter += 1;
        }
    }
}
