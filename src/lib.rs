pub mod uxninterface;
pub mod instruction;
pub mod ops;

#[cfg(feature = "asm")]
pub mod uxnasmlib;


#[cfg(feature = "emu")]
pub mod uxnemulib;
