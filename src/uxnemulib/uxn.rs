use std::fmt;
use std::error::Error;

pub struct Uxn {
    ram: Vec<u8>,
}

#[derive(Debug)]
pub struct UxnError {
}

impl fmt::Display for UxnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error running Uxn")
    }
}

impl Error for UxnError {}

impl Uxn {
    pub fn new<I>(rom: I) -> Result<Self, UxnError>
    where
        I: Iterator<Item = u8>,
    {
        let mut ram = vec![0x0; 0x10000];

        for (ram_loc, val) in (&mut ram[0x100..]).iter_mut().zip(rom).take(0x100000 - 0x100) {
            *ram_loc = val;
        }

        return Ok(Uxn{ram});
    }
}
