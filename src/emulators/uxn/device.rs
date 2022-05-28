use crate::uxninterface::UxnError;
use std::io;

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
    fn write(&mut self, port: u8, val: u8);
    fn read(&mut self, port: u8) -> u8;
}

pub trait MainRamInterface {}
