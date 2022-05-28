use crate::uxninterface::UxnError;
use std::io;
use std::fmt;
use std::error::Error;

#[derive(PartialEq, Debug)]
pub enum DeviceWriteReturnCode<'a, K>
    where K: io::Write,
{
    Success,
    WriteToSystemDevice(u8, &'a mut K),
}

#[derive(PartialEq, Debug)]
pub enum DeviceReadReturnCode {
    Success(Result<u8, UxnError>),
    ReadFromSystemDevice(u8),
}

pub trait DeviceList 
{
    type DebugWriter: io::Write;

    fn write_to_device(&mut self, device_address: u8, val: u8, main_ram: &mut dyn MainRamInterface) -> DeviceWriteReturnCode<Self::DebugWriter>;
    fn read_from_device(&mut self, device_address: u8) -> DeviceReadReturnCode;
}

pub trait Device {
    fn write(&mut self, port: u8, val: u8, main_ram: &mut dyn MainRamInterface);
    fn read(&mut self, port: u8) -> u8;
}

#[derive(Debug, PartialEq)]
pub enum MainRamInterfaceError {
    AddressOutOfBounds,
}

impl fmt::Display for MainRamInterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainRamInterfaceError::AddressOutOfBounds => {
                write!(f, "attempt to access out of range memory address")
            }
        }
    }
}

impl Error for MainRamInterfaceError {}

pub trait MainRamInterface {
    fn read(&self, address: u16, num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError>;
}
