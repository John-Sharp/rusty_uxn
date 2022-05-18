use crate::emulators::uxnemulib::uxn::device::Device;
use std::io;
use std::io::Write;

pub struct Console {
    device_mem: Vec<u8>,
}

impl Console {
    pub fn new() -> Self {
        let device_mem = vec!(0u8;16);
        Console{device_mem}
    }
}

impl Device for Console {
    fn write(&mut self, port: u8, val: u8) {
        if port > 0xf {
            panic!("attempting to write to port out of range");
        }

        self.device_mem[port as usize] = val;

        match port {
            0x8 => {
                print!("{}", val as char);
                io::stdout().flush().expect("error writing to stdout");
            },
            0x9 => {
                eprint!("{}", val as char);
                io::stderr().flush().expect("error writing to stderr");
            },
            _ => {}
        }
    }

    fn read(&mut self, _port: u8) -> u8 {
        panic!("not yet implemented");
    }
}
