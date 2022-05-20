use crate::emulators::uxn::device::Device;
use std::io;
use std::io::Write;

pub struct Console {
    vector: [u8; 2],
    received_input: u8,
}

impl Console {
    pub fn new() -> Self {
        let vector = [0u8; 2];
        let received_input = 0;
        Console{vector, received_input}
    }

    pub fn read_vector(&self) -> u16 {
        return u16::from_be_bytes(self.vector);
    }

    pub fn provide_input(&mut self, input: u8) {
        self.received_input = input;
    }
}

impl Device for Console {
    fn write(&mut self, port: u8, val: u8) {
        if port > 0xf {
            panic!("attempting to write to port out of range");
        }

        match port {
            0x0 => {
                self.vector[0] = val;
            },
            0x1 => {
                self.vector[1] = val;
            },
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

    fn read(&mut self, port: u8) -> u8 {
        match port {
            0x0 => {
                return self.vector[0];
            },
            0x1 => {
                return self.vector[1];
            },
            0x2 => {
                return self.received_input;
            },
            _ => {},
        }

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_vector() {
        let mut console = Console::new();

        let initial_vector = console.read_vector();
        assert_eq!(initial_vector, 0);

        console.write(0x0, 0xab);
        console.write(0x1, 0xcd);

        let vector = console.read_vector();
        assert_eq!(vector, 0xabcd);

        assert_eq!(console.read(0x0), 0xab);
        assert_eq!(console.read(0x1), 0xcd);
    }

    #[test]
    fn test_read() {
        let mut console = Console::new();

        // initial read should return 0x00
        assert_eq!(console.read(0x2), 0x00);

        // provide some inputted text
        console.provide_input(0x8a);

        // read should return what was inputted
        assert_eq!(console.read(0x2), 0x8a);

        // and again
        assert_eq!(console.read(0x2), 0x8a);

        // provide some different inputted text
        console.provide_input(0x7b);
        assert_eq!(console.read(0x2), 0x7b);
    }
}
