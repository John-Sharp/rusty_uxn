use crate::uxnemulib::uxn::device::Device;

pub trait UxnSystemInterface {
    fn set_working_stack_index(&mut self, index: u8);
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
            },
            0x4..=0x7 => {
                // not used
            },
            0x8..=0x9 => {
                // set red component of the four system colours
            },
            0xa..=0xb => {
                // set blue component of the four system colours
            },
            0xc..=0xd => {
                // set green component of the four system colours
            },
            0xe => {
                // print debug status
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
        if port > 0xf {
            panic!("attempting to read port out of range");
        }
        // TODO
        return 0;
    }
}
