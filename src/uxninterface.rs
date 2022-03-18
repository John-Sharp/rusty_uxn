pub trait Uxn {
    fn read_from_ram(&self, addr: u16) -> u8;
}
