pub trait Uxn {
    fn read_from_ram(&self, addr: u16) -> u8;
    fn get_program_counter(&self) -> u16;
    fn set_program_counter(&mut self, addr: u16);
    fn push_to_return_stack(&mut self, byte: u8);
    fn push_to_working_stack(&mut self, byte: u8);
}
