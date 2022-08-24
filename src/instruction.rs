use crate::uxninterface::UxnWithDevices;
use crate::uxninterface::UxnError;

pub trait InstructionFactory {
    fn from_byte(&self, byte: u8) -> Box<dyn Instruction>;
}

pub trait Instruction {
    fn execute(&self, uxn: &mut dyn UxnWithDevices) -> Result<(), UxnError>;
}
