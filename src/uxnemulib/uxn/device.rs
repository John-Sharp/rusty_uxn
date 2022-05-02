use crate::uxninterface::UxnError;

#[derive(PartialEq, Debug)]
pub enum DeviceWriteReturnCode {
    Success,
    WriteToSystemDevice(u8),
}

#[derive(PartialEq, Debug)]
pub enum DeviceReadReturnCode {
    Success(Result<u8, UxnError>),
    ReadFromSystemDevice(u8),
}

pub trait DeviceList {
    fn write_to_device(&mut self, device_address: u8, val: u8) -> DeviceWriteReturnCode;
    fn read_from_device(&mut self, device_address: u8) -> DeviceReadReturnCode;
}

pub trait Device {
    fn write(&mut self, port: u8, val: u8);
    fn read(&mut self, port: u8) -> u8;
}
