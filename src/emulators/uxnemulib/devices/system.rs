use crate::emulators::uxn::device::Device;
use std::io::Write;

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
    fn get_working_stack_index(&self) -> u8;

    fn set_return_stack_index(&mut self, index: u8);
    fn get_return_stack_index(&self) -> u8;

    fn set_system_color(&mut self, slot: UxnSystemColor, val: u8);
    fn get_system_color(&self, slot: UxnSystemColor) -> u8;

    fn get_working_stack_iter(&self) -> std::slice::Iter<u8>;
    fn get_return_stack_iter(&self) -> std::slice::Iter<u8>;
}

pub struct System<'a, J, K>
    where J: UxnSystemInterface,
          K: Write,
{
    pub uxn: &'a mut J,
    debug_writer: K
}

impl<'a, J, K> System<'a, J, K>
    where J: UxnSystemInterface,
          K: Write,
{
    pub fn new(uxn: &'a mut J, debug_writer: K) -> Self {
        System {
            uxn,
            debug_writer,
        }
    }
}

impl<'a, J: UxnSystemInterface, K: Write> Device for System<'a, J, K> {
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
                // print debug status
                let working_stack_status_string = 
                    self.uxn.get_working_stack_iter()
                    .map(|x| { return format!("{:02x}", x); })
                    .collect::<Vec<String>>();
                let working_stack_status_string = 
                    working_stack_status_string.join(" ");

                writeln!(self.debug_writer, "<wst> {}", working_stack_status_string)
                    .expect("could not write debug output");

                let return_stack_status_string = 
                    self.uxn.get_return_stack_iter()
                    .map(|x| { return format!("{:02x}", x); })
                    .collect::<Vec<String>>();
                let return_stack_status_string = 
                    return_stack_status_string.join(" ");

                writeln!(self.debug_writer, "<rst> {}", return_stack_status_string)
                    .expect("could not write debug output");
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
                return self.uxn.get_system_color(UxnSystemColor::Red1);
            },
            0x9 => {
                return self.uxn.get_system_color(UxnSystemColor::Red2);
            },
            0xa => {
                return self.uxn.get_system_color(UxnSystemColor::Green1);
            },
            0xb => {
                return self.uxn.get_system_color(UxnSystemColor::Green2);
            },
            0xc => {
                return self.uxn.get_system_color(UxnSystemColor::Blue1);
            },
            0xd => {
                return self.uxn.get_system_color(UxnSystemColor::Blue2);
            },
            0xe => {
                // print debug status (no-op for read)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::cell::RefCell;

    struct MockUxn {
        set_working_stack_index_arguments_received: RefCell<VecDeque<(u8,)>>,

        get_working_stack_index_arguments_received: RefCell<VecDeque<()>>,
        get_working_stack_index_values_to_return: RefCell<VecDeque<u8>>,

        set_return_stack_index_arguments_received: RefCell<VecDeque<(u8,)>>,

        get_return_stack_index_arguments_received: RefCell<VecDeque<()>>,
        get_return_stack_index_values_to_return: RefCell<VecDeque<u8>>,

        set_system_color_arguments_received: RefCell<VecDeque<(UxnSystemColor, u8)>>,

        get_system_color_arguments_received: RefCell<VecDeque<(UxnSystemColor,)>>,
        get_system_color_values_to_return: RefCell<VecDeque<u8>>,

        get_working_stack_iter_values_to_return: Vec<u8>,
        get_return_stack_iter_values_to_return: Vec<u8>,
    }

    impl MockUxn {
        pub fn new() -> Self {
            MockUxn {
                set_working_stack_index_arguments_received: RefCell::new(VecDeque::new()),
                get_working_stack_index_arguments_received: RefCell::new(VecDeque::new()),
                get_working_stack_index_values_to_return: RefCell::new(VecDeque::new()),

                set_return_stack_index_arguments_received: RefCell::new(VecDeque::new()),
                get_return_stack_index_arguments_received: RefCell::new(VecDeque::new()),
                get_return_stack_index_values_to_return: RefCell::new(VecDeque::new()),

                set_system_color_arguments_received: RefCell::new(VecDeque::new()),

                get_system_color_arguments_received: RefCell::new(VecDeque::new()),
                get_system_color_values_to_return: RefCell::new(VecDeque::new()),

                get_working_stack_iter_values_to_return: Vec::new(),
                get_return_stack_iter_values_to_return: Vec::new(),
            }
        }
    }

    impl UxnSystemInterface for MockUxn {
        fn set_working_stack_index(&mut self, index: u8) {
            self.set_working_stack_index_arguments_received
                .borrow_mut()
                .push_back((index,));
        }

        fn get_working_stack_index(&self) -> u8 {
            self.get_working_stack_index_arguments_received
                .borrow_mut()
                .push_back(());

            return self
                .get_working_stack_index_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn set_return_stack_index(&mut self, index: u8) {
            self.set_return_stack_index_arguments_received
                .borrow_mut()
                .push_back((index,));
        }

        fn get_return_stack_index(&self) -> u8 {
            self.get_return_stack_index_arguments_received
                .borrow_mut()
                .push_back(());

            return self
                .get_return_stack_index_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn set_system_color(&mut self, slot: UxnSystemColor, val: u8) {
            self.set_system_color_arguments_received
                .borrow_mut()
                .push_back((slot, val));
        }

        fn get_system_color(&self, slot: UxnSystemColor) -> u8 {
            self.get_system_color_arguments_received
                .borrow_mut()
                .push_back((slot,));

            return self
                .get_system_color_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn get_working_stack_iter(&self) -> std::slice::Iter<u8> {
            return self
                .get_working_stack_iter_values_to_return
                .iter();
        }
        fn get_return_stack_iter(&self) -> std::slice::Iter<u8> {
            return self
                .get_return_stack_iter_values_to_return
                .iter();
        }
    }

    #[test]
    fn test_set_working_stack_index() {
        let mut mock_uxn = MockUxn::new();

        let mut system = System {
            uxn: &mut mock_uxn,
            debug_writer: Vec::new(),
        };

        // 0x2 is the port for setting the working stack index,
        // 0x76 is the value to set it to
        system.write(0x2, 0x76);

        assert_eq!(mock_uxn.set_working_stack_index_arguments_received.into_inner(),
          VecDeque::from([(0x76,)]));
    }

    #[test]
    fn test_get_working_stack_index() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.get_working_stack_index_values_to_return
            .borrow_mut()
            .push_back(0x76);

        let mut system = System {
            uxn: &mut mock_uxn,
            debug_writer: Vec::new(),
        };

        // 0x2 is the port for getting the working stack index,
        let res = system.read(0x2);

        assert_eq!(res, 0x76);
        assert_eq!(mock_uxn.get_working_stack_index_arguments_received.into_inner(),
          VecDeque::from([()]));
    }

    #[test]
    fn test_set_return_stack_index() {
        let mut mock_uxn = MockUxn::new();

        let mut system = System {
            uxn: &mut mock_uxn,
            debug_writer: Vec::new(),
        };

        // 0x3 is the port for setting the return stack index,
        // 0x76 is the value to set it to
        system.write(0x3, 0x76);

        assert_eq!(mock_uxn.set_return_stack_index_arguments_received.into_inner(),
          VecDeque::from([(0x76,)]));
    }

    #[test]
    fn test_get_return_stack_index() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.get_return_stack_index_values_to_return
            .borrow_mut()
            .push_back(0x76);

        let mut system = System {
            uxn: &mut mock_uxn,
            debug_writer: Vec::new(),
        };

        // 0x3 is the port for getting the return stack index,
        let res = system.read(0x3);

        assert_eq!(res, 0x76);
        assert_eq!(mock_uxn.get_return_stack_index_arguments_received.into_inner(),
          VecDeque::from([()]));
    }

    #[test]
    fn test_set_system_color() {
        let mut mock_uxn = MockUxn::new();

        let test_cases = vec![
            (0x8, UxnSystemColor::Red1, 0x76),
            (0x9, UxnSystemColor::Red2, 0x96),
            (0xa, UxnSystemColor::Green1, 0x61),
            (0xb, UxnSystemColor::Green2, 0x01),
            (0xc, UxnSystemColor::Blue1, 0x41),
            (0xd, UxnSystemColor::Blue2, 0x29),
        ];

        for (port, expected_color, expected_val) in test_cases {
            let mut system = System {
                uxn: &mut mock_uxn,
                debug_writer: Vec::new(),
            };
            system.write(port, expected_val);

            assert_eq!(mock_uxn.set_system_color_arguments_received.borrow_mut()
                       .pop_back().unwrap(),
              (expected_color, expected_val));
        }
    }

    #[test]
    fn test_get_system_color() {
        let mut mock_uxn = MockUxn::new();

        let test_cases = vec![
            (0x8, UxnSystemColor::Red1, 0x76),
            (0x9, UxnSystemColor::Red2, 0x96),
            (0xa, UxnSystemColor::Green1, 0x61),
            (0xb, UxnSystemColor::Green2, 0x01),
            (0xc, UxnSystemColor::Blue1, 0x41),
            (0xd, UxnSystemColor::Blue2, 0x29),
        ];

        for (port, expected_color, expected_val) in test_cases {
            mock_uxn.get_system_color_values_to_return
                .borrow_mut()
                .push_back(expected_val);

            let mut system = System {
                uxn: &mut mock_uxn,
                debug_writer: Vec::new(),
            };

            let res = system.read(port);

            assert_eq!(res, expected_val);
            assert_eq!(mock_uxn.get_system_color_arguments_received
                       .borrow_mut().pop_back().unwrap(),
                    (expected_color,));
        }
    }

    #[test]
    fn test_print_debug() {
        let mut mock_uxn = MockUxn::new();
        let mut output_received = Vec::new();

        let working_stack = vec![4,5,6];
        let return_stack = vec![1,2,3];

        mock_uxn.get_working_stack_iter_values_to_return = working_stack;
        mock_uxn.get_return_stack_iter_values_to_return = return_stack;


        let mut system = System {
            uxn: &mut mock_uxn,
            debug_writer: &mut output_received,
        };

        // 0xe is the port for printing debug information, it doesn't matter what byte is written
        // there so the value 0x22 is entirely arbritrary
        system.write(0xe, 0x22);

        assert_eq!(&(String::from_utf8(output_received).unwrap()),
            "<wst> 04 05 06\n<rst> 01 02 03\n");
    }
}
