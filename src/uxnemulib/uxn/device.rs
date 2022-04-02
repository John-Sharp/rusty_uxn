pub trait Device {
    fn write(&mut self, port: u8, val: u8);
}
