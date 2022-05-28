use crate::emulators::uxn::device::{Device, MainRamInterface, MainRamInterfaceError};
use std::io;

pub struct Console<J, K>
    where J: io::Write, 
          K: io::Write,
{
    vector: [u8; 2],
    received_input: u8,
    stdout_writer: J,
    stderr_writer: K,
}

impl<J, K> Console<J, K>
    where J: io::Write,
          K: io::Write,
{
    pub fn new(stdout_writer: J, stderr_writer: K) -> Self {
        let vector = [0u8; 2];
        let received_input = 0;
        Console{vector, received_input, stdout_writer, stderr_writer,}
    }

    pub fn read_vector(&self) -> u16 {
        return u16::from_be_bytes(self.vector);
    }

    pub fn provide_input(&mut self, input: u8) {
        self.received_input = input;
    }
}

impl<J, K> Device for Console<J, K>
    where J: io::Write,
          K: io::Write,
{
    fn write(&mut self, port: u8, val: u8, _main_ram: &mut dyn MainRamInterface) {
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
                write!(self.stdout_writer, "{}", val as char)
                    .expect("error writing to stdout");
                self.stdout_writer.flush()
                    .expect("error flushing stdout");
            },
            0x9 => {
                write!(self.stderr_writer, "{}", val as char)
                    .expect("error writing to stderr");
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

    struct MockMainRamInterface {}
    impl MainRamInterface for MockMainRamInterface {
        fn read(&self, address: u16, num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError> {
            panic!("should not be called");
        }
    }

    #[test]
    fn test_set_get_vector() {
        let mut console = Console::new(Vec::new(), Vec::new());

        let initial_vector = console.read_vector();
        assert_eq!(initial_vector, 0);

        console.write(0x0, 0xab, &mut MockMainRamInterface{});
        console.write(0x1, 0xcd, &mut MockMainRamInterface{});

        let vector = console.read_vector();
        assert_eq!(vector, 0xabcd);

        assert_eq!(console.read(0x0), 0xab);
        assert_eq!(console.read(0x1), 0xcd);
    }

    #[test]
    fn test_read() {
        let mut console = Console::new(Vec::new(), Vec::new());

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

    #[test]
    fn test_write_stdout_stderr() {
        let mut stdout_writer = Vec::new();
        let mut stderr_writer = Vec::new();

        let mut console = Console::new(&mut stdout_writer, &mut stderr_writer);

        console.write(0x8, 0x01, &mut MockMainRamInterface{});
        console.write(0x8, 0x02, &mut MockMainRamInterface{});
        console.write(0x9, 0x04, &mut MockMainRamInterface{});
        console.write(0x8, 0x03, &mut MockMainRamInterface{});
        console.write(0x9, 0x05, &mut MockMainRamInterface{});
        console.write(0x9, 0x06, &mut MockMainRamInterface{});

        assert_eq!(stdout_writer, vec![0x01, 0x02, 0x03]);
        assert_eq!(stderr_writer, vec![0x04, 0x05, 0x06]);
    }
}
