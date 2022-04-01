use cratpe::uxninterface::Uxn;
use crate::uxninterface::UxnError;

// get function pointers that push/pop/peek from the correct Uxn stacks depending on which mode
// flags are present
fn get_helper_fns<T: Uxn + ?Sized>(keep: bool, ret: bool) -> (fn(&mut T, u8), fn(&mut T) -> Result<u8, UxnError>,) {
    let push = if ret == false {
        Uxn::push_to_working_stack
    } else {
        Uxn::push_to_return_stack
    };

    let pop = if ret == false {
        if keep {
            Uxn::peek_at_working_stack
        } else {
            Uxn::pop_from_working_stack
        }
    } else {
        if keep {
            Uxn::peek_at_return_stack
        } else {
            Uxn::pop_from_return_stack
        }
    };

    return (push, pop);
}

pub fn lit_handler(u: Box<&mut dyn Uxn>, keep: bool, short: bool, ret: bool) -> Result<(), UxnError> {
    let (push, _) = get_helper_fns(keep, ret); 

    // read byte/short from ram
    let a = u.read_next_byte_from_ram()?;

    push(*u, a);

    if short == false {
        return Ok(());
    }
        
    let a = u.read_next_byte_from_ram()?;
    push(*u, a);

    return Ok(());
}

pub fn deo_handler(u: Box<&mut dyn Uxn>, keep: bool, short: bool, ret: bool) -> Result<(), UxnError> {
    let (_, pop) = get_helper_fns(keep, ret); 

    // get device address to write to from working/return stack
    let device_address = pop(*u)?;

    // pop byte from working/return stack
    let value = pop(*u)?;

    // write byte to device responsible for device address
    u.write_to_device(device_address, value);

    // if in short mode get another byte from working/return stack
    // and write to device
    if short == true {
        let value = pop(*u)?;
        u.write_to_device(device_address, value);
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockUxn {
        read_next_byte_from_ram_arguments_received: RefCell<VecDeque<()>>,
        read_next_byte_from_ram_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        read_from_ram_arguments_received: RefCell<VecDeque<(u16,)>>,
        read_from_ram_values_to_return: RefCell<VecDeque<u8>>,

        get_program_counter_arguments_received: RefCell<VecDeque<()>>,
        get_program_counter_values_to_return: RefCell<VecDeque<Result<u16, UxnError>>>,

        set_program_counter_arguments_received: RefCell<VecDeque<(u16,)>>,

        push_to_return_stack_arguments_received: RefCell<VecDeque<(u8,)>>,

        push_to_working_stack_arguments_received: RefCell<VecDeque<(u8,)>>,

        peek_at_working_stack_arguments_received: RefCell<VecDeque<()>>,
        peek_at_working_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        pop_from_working_stack_arguments_received: RefCell<VecDeque<()>>,
        pop_from_working_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        peek_at_return_stack_arguments_received: RefCell<VecDeque<()>>,
        peek_at_return_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        pop_from_return_stack_arguments_received: RefCell<VecDeque<()>>,
        pop_from_return_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        write_to_device_arguments_received: RefCell<VecDeque<(u8, u8)>>,
    }

    impl MockUxn {
        fn new() -> Self {
            MockUxn{
                read_next_byte_from_ram_arguments_received: RefCell::new(VecDeque::new()),
                read_next_byte_from_ram_values_to_return: RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab),])),

                read_from_ram_arguments_received: RefCell::new(VecDeque::new()),
                read_from_ram_values_to_return: RefCell::new(VecDeque::new()),

                get_program_counter_arguments_received: RefCell::new(VecDeque::new()),
                get_program_counter_values_to_return: RefCell::new(VecDeque::new()),

                set_program_counter_arguments_received: RefCell::new(VecDeque::new()),

                push_to_return_stack_arguments_received: RefCell::new(VecDeque::new()),

                push_to_working_stack_arguments_received: RefCell::new(VecDeque::new()), 

                peek_at_working_stack_arguments_received: RefCell::new(VecDeque::new()),
                peek_at_working_stack_values_to_return: RefCell::new(VecDeque::new()),

                pop_from_working_stack_arguments_received: RefCell::new(VecDeque::new()),
                pop_from_working_stack_values_to_return: RefCell::new(VecDeque::new()),

                peek_at_return_stack_arguments_received: RefCell::new(VecDeque::new()),
                peek_at_return_stack_values_to_return: RefCell::new(VecDeque::new()),

                pop_from_return_stack_arguments_received: RefCell::new(VecDeque::new()),
                pop_from_return_stack_values_to_return: RefCell::new(VecDeque::new()),

                write_to_device_arguments_received: RefCell::new(VecDeque::new()),
            }
        }

    }

    impl Uxn for MockUxn {
        fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
            self.read_next_byte_from_ram_arguments_received
                .borrow_mut().push_back(());
            return self.read_next_byte_from_ram_values_to_return
                .borrow_mut().pop_front().unwrap();
        }

        fn read_from_ram(&self, addr: u16) -> u8 {
            self.read_from_ram_arguments_received
                .borrow_mut().push_back((addr,));
            return self.read_from_ram_values_to_return.borrow_mut().pop_front().unwrap();
        }
    
        fn get_program_counter(&self) -> Result<u16, UxnError> {
            self.get_program_counter_arguments_received
                .borrow_mut().push_back(());
            return self.get_program_counter_values_to_return.borrow_mut()
                .pop_front().unwrap();
        }
    
        fn set_program_counter(&mut self, addr: u16) {
            self.set_program_counter_arguments_received
                .borrow_mut().push_back((addr,));
        }
    
        fn push_to_return_stack(&mut self, byte: u8) {
            self.push_to_return_stack_arguments_received
                .borrow_mut().push_back((byte,));
        }
    
        fn push_to_working_stack(&mut self, byte: u8) {
            self.push_to_working_stack_arguments_received
                .borrow_mut().push_back((byte,));
        }
        
        fn peek_at_working_stack(&mut self) -> Result<u8, UxnError> {
            self.peek_at_working_stack_arguments_received
                .borrow_mut().push_back(());
            return self.peek_at_working_stack_values_to_return.borrow_mut()
                .pop_front().unwrap();
        }

        fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
            self.pop_from_working_stack_arguments_received
                .borrow_mut().push_back(());
            return self.pop_from_working_stack_values_to_return.borrow_mut()
                .pop_front().unwrap();
        }

        fn peek_at_return_stack(&mut self) -> Result<u8, UxnError> {
            self.peek_at_return_stack_arguments_received
                .borrow_mut().push_back(());
            return self.peek_at_return_stack_values_to_return.borrow_mut()
                .pop_front().unwrap();
        }

        fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
            self.pop_from_return_stack_arguments_received
                .borrow_mut().push_back(());
            return self.pop_from_return_stack_values_to_return.borrow_mut()
                .pop_front().unwrap();
        }

        fn write_to_device(&mut self, device_address: u8, val: u8) {
            self.write_to_device_arguments_received.borrow_mut().push_back((device_address, val));
        }
    }

    #[test]
    fn test_lit_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.read_next_byte_from_ram_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa),]));

        lit_handler(Box::new(&mut mock_uxn),
            false, false, false).unwrap();

        assert_eq!(mock_uxn.read_next_byte_from_ram_arguments_received.into_inner(),
            VecDeque::from([(),]));
        assert_eq!(mock_uxn.push_to_working_stack_arguments_received.into_inner(),
        VecDeque::from([(0xaa,),]));
        assert_eq!(mock_uxn.push_to_return_stack_arguments_received.into_inner(),
        VecDeque::new());
    }

    #[test]
    fn test_lit_handler_short_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.read_next_byte_from_ram_values_to_return = RefCell::new(
            VecDeque::from([Ok(0xaa), Ok(0xab),]));

        lit_handler(Box::new(&mut mock_uxn),
            false, true, false).unwrap();

        assert_eq!(mock_uxn.read_next_byte_from_ram_arguments_received.into_inner(),
            VecDeque::from([(),()]));
        assert_eq!(mock_uxn.push_to_working_stack_arguments_received.into_inner(),
        VecDeque::from([(0xaa,), (0xab,),]));
        assert_eq!(mock_uxn.push_to_return_stack_arguments_received.into_inner(),
        VecDeque::new());
    }

    #[test]
    fn test_lit_handler_ret_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.read_next_byte_from_ram_values_to_return = RefCell::new(
            VecDeque::from([Ok(0xaa),]));

        lit_handler(Box::new(&mut mock_uxn),
            false, false, true).unwrap();

        assert_eq!(mock_uxn.read_next_byte_from_ram_arguments_received.into_inner(),
            VecDeque::from([(),]));
        assert_eq!(mock_uxn.push_to_working_stack_arguments_received.into_inner(),
        VecDeque::new());
        assert_eq!(mock_uxn.push_to_return_stack_arguments_received.into_inner(),
        VecDeque::from([(0xaa,),]));
    }

    #[test]
    fn test_deo_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(
            VecDeque::from([
                           Ok(0xaa), // should be used as device address
                           Ok(0xba), // should be used as value to write
            ]));

        deo_handler(Box::new(&mut mock_uxn),
            false, false, false).unwrap();

        assert_eq!(mock_uxn.pop_from_working_stack_arguments_received.into_inner(),
            VecDeque::from([(), (),]));
        assert_eq!(mock_uxn.write_to_device_arguments_received.into_inner(),
            VecDeque::from([(0xaa, 0xba),]));
    }

    // TODO peek functions need to take position to peek at
    // #[test]
    // fn test_deo_handler_keep_mode() {
    //     let mut mock_uxn = MockUxn::new();
    //     mock_uxn.peek_at_working_stack_values_to_return = RefCell::new(
    //         VecDeque::from([
    //                        Ok(0xaa), // should be used as device address
    //                        Ok(0xba), // should be used as value to write
    //         ]));

    //     deo_handler(Box::new(&mut mock_uxn),
    //         true, false, false).unwrap();

    //     assert_eq!(mock_uxn.peek_at_working_stack_arguments_received.into_inner(),
    //         VecDeque::from([(), (),]));
    //     assert_eq!(mock_uxn.write_to_device_arguments_received.into_inner(),
    //         VecDeque::from([(0xaa, 0xba),]));
    // }
}
