use std::fmt;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum UxnError {
    OutOfRangeMemoryAddress,
    StackUnderflow,
}

impl fmt::Display for UxnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UxnError::OutOfRangeMemoryAddress => {
                write!(f, "attempt to access out of range memory address")
            },
            UxnError::StackUnderflow => {
                write!(f, "stack underflow encountered")
            },
        }
    }
}

impl Error for UxnError {}


pub trait Uxn {
    fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError>;
    fn read_from_ram(&self, addr: u16) -> u8;
    fn get_program_counter(&self) -> Result<u16, UxnError>;
    fn set_program_counter(&mut self, addr: u16);
    fn push_to_return_stack(&mut self, byte: u8);
    fn push_to_working_stack(&mut self, byte: u8);
    fn peek_at_working_stack(&mut self) -> Result<u8, UxnError>;
    fn pop_from_working_stack(&mut self) -> Result<u8, UxnError>;
    fn peek_at_return_stack(&mut self) -> Result<u8, UxnError>;
    fn pop_from_return_stack(&mut self) -> Result<u8, UxnError>;
    fn write_to_device(&mut self, device_address: u8, val: u8);
}
