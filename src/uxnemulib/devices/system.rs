use crate::uxnemulib::uxn::device::Device;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UxnSystemColor {
    Red1,
    Red2,
    Green1,
    Green2,
    Blue1,
    Blue2,
}

pub trait UxnSystemInterface {
    fn set_working_stack_index(&mut self, index: u8);
    fn get_working_stack_index(&mut self) -> u8;

    fn set_return_stack_index(&mut self, index: u8);
    fn get_return_stack_index(&mut self) -> u8;

    fn set_system_color(&mut self, slot: UxnSystemColor, val: u8);
    fn get_system_color(&self, slot: UxnSystemColor) -> u8;

    fn get_working_stack_iter(&self) -> std::slice::Iter<u8>;
    fn get_return_stack_iter(&self) -> std::slice::Iter<u8>;
}

pub struct System<'a, J>
    where J: UxnSystemInterface,
{
    pub uxn: &'a mut J,
}

impl<'a, J: UxnSystemInterface> Device for System<'a, J> {
    fn write(&mut self, port: u8, val: u8) {
        match port {
            0x0..=0x1 => {
                // not used
            }
            0x2 => {
                // set working stack index to `val`
                self.uxn.set_working_stack_index(val);
            },
            0x3 => {
                // set return stack index to `val`
                self.uxn.set_return_stack_index(val);
            },
            0x4..=0x7 => {
                // not used
            },
            0x8 => {
                self.uxn.set_system_color(UxnSystemColor::Red1, val);
            },
            0x9 => {
                self.uxn.set_system_color(UxnSystemColor::Red2, val);
            },
            0xa => {
                self.uxn.set_system_color(UxnSystemColor::Green1, val);
            },
            0xb => {
                self.uxn.set_system_color(UxnSystemColor::Green2, val);
            },
            0xc => {
                self.uxn.set_system_color(UxnSystemColor::Blue1, val);
            },
            0xd => {
                self.uxn.set_system_color(UxnSystemColor::Blue2, val);
            },
            0xe => {
                // print debug status (no-op for write)
            },
            0xf => {
                // terminate application
            },
            _ => {
                panic!("attempting to write to port out of range");
            }
        }
    }


    fn read(&mut self, port: u8) -> u8 {
        match port {
            0x0..=0x1 => {
                // not used
            }
            0x2 => {
                // get working stack index
                return self.uxn.get_working_stack_index();
            },
            0x3 => {
                // get return stack index
                return self.uxn.get_return_stack_index();
            },
            0x4..=0x7 => {
                // not used
            },
            0x8 => {
                self.uxn.get_system_color(UxnSystemColor::Red1);
            },
            0x9 => {
                self.uxn.get_system_color(UxnSystemColor::Red2);
            },
            0xa => {
                self.uxn.get_system_color(UxnSystemColor::Green1);
            },
            0xb => {
                self.uxn.get_system_color(UxnSystemColor::Green2);
            },
            0xc => {
                self.uxn.get_system_color(UxnSystemColor::Blue1);
            },
            0xd => {
                self.uxn.get_system_color(UxnSystemColor::Blue2);
            },
            0xe => {
                // print debug status
                println!();
            },
            0xf => {
                // terminate application
            },
            _ => {
                panic!("attempting to read from port out of range");
            }
        }

        return 0;
    }
}
