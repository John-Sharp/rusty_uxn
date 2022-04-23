pub trait Device {
    fn write(&mut self, port: u8, val: u8);
    fn read(&mut self, port: u8) -> u8;
}
