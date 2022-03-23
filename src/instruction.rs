use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

pub trait InstructionFactory {
    fn from_byte(&self, byte: u8) -> Box<dyn Instruction>;
}

pub trait Instruction {
    fn execute(&self, uxn: Box::<&mut dyn Uxn>) -> Result<(), UxnError>;
}
