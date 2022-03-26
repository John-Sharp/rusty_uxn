use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

// get function pointers that push/pop/peek from the correct Uxn stacks depending on which mode
// flags are present
fn get_helper_fns<T: Uxn + ?Sized>(keep: bool, ret: bool) -> (fn(&mut T, u8),) {
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
    }

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

    let device_address = pop(*u);

    // pop byte/short from working/return stack

    // write byte/short to device responsible for device address
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockUxn {
        read_next_byte_from_ram_ret_values: RefCell<VecDeque<Result<u8, UxnError>>>,
        ram_values: RefCell<VecDeque<u8>>,
        ram_addresses_recvd: RefCell<VecDeque<u16>>,
        get_prog_counter_ret_values: RefCell<VecDeque<Result<u16, UxnError>>>,
        prog_counter_recv_values: RefCell<VecDeque<u16>>,
        working_stack: RefCell<VecDeque<u8>>,
        return_stack: RefCell<VecDeque<u8>>,
    }

    impl Uxn for MockUxn {
        fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
            return self.read_next_byte_from_ram_ret_values.borrow_mut().pop_front().unwrap();
        }

        fn read_from_ram(&self, addr: u16) -> u8 {
            self.ram_addresses_recvd.borrow_mut().push_back(addr);
            return self.ram_values.borrow_mut().pop_front().unwrap();
        }
    
        fn get_program_counter(&self) -> Result<u16, UxnError> {
            return self.get_prog_counter_ret_values.borrow_mut()
                .pop_front().unwrap();
        }
    
        fn set_program_counter(&mut self, addr: u16) {
            self.prog_counter_recv_values.borrow_mut().push_back(addr);
        }
    
        fn push_to_return_stack(&mut self, byte: u8) {
            self.return_stack.borrow_mut().push_back(byte);
        }
    
        fn push_to_working_stack(&mut self, byte: u8) {
            self.working_stack.borrow_mut().push_back(byte);
        }
        
        fn peek_at_working_stack(&mut self) -> Result<u8, UxnError> {
            let last = self.working_stack.borrow().last();
            if last.is_none() {
                return Err(UxnError::StackUnderflow);
            }
            return Ok(last.unwrap());
        }

        fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
            let last = self.working_stack.borrow().pop();
            if last.is_none() {
                return Err(UxnError::StackUnderflow);
            }
            return Ok(last.unwrap());
        }

        fn peek_at_return_stack(&mut self) -> Result<u8, UxnError> {
            let last = self.return_stack.borrow().last();
            if last.is_none() {
                return Err(UxnError::StackUnderflow);
            }
            return Ok(last.unwrap());
        }

        fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
            let last = self.return_stack.borrow().pop();
            if last.is_none() {
                return Err(UxnError::StackUnderflow);
            }
            return Ok(last.unwrap());
        }
    }

    #[test]
    fn test_lit_handler() {
        let mut mock_uxn = MockUxn{
            read_next_byte_from_ram_ret_values: RefCell::new(VecDeque::from([Ok(0xaa),])),
            ram_values: RefCell::new(VecDeque::new()),
            ram_addresses_recvd: RefCell::new(VecDeque::new()),
            get_prog_counter_ret_values: RefCell::new(VecDeque::new()),
            prog_counter_recv_values: RefCell::new(VecDeque::new()),
            working_stack: RefCell::new(VecDeque::new()),
            return_stack: RefCell::new(VecDeque::new()),
        };

        lit_handler(Box::new(&mut mock_uxn),
            false, false, false).unwrap();

        assert_eq!(mock_uxn.working_stack.into_inner(),
        VecDeque::from([0xaa,]));
        assert_eq!(mock_uxn.return_stack.into_inner(), VecDeque::new());
    }

    #[test]
    fn test_lit_handler_short_mode() {
        let mut mock_uxn = MockUxn{
            read_next_byte_from_ram_ret_values: RefCell::new(VecDeque::from([Ok(0xaa),Ok(0xab)])),
            ram_values: RefCell::new(VecDeque::new()),
            ram_addresses_recvd: RefCell::new(VecDeque::new()),
            get_prog_counter_ret_values: RefCell::new(VecDeque::new()),
            prog_counter_recv_values: RefCell::new(VecDeque::new()),
            working_stack: RefCell::new(VecDeque::new()),
            return_stack: RefCell::new(VecDeque::new()),
        };

        lit_handler(Box::new(&mut mock_uxn),
            false, true, false).unwrap();

        assert_eq!(mock_uxn.working_stack.into_inner(),
        VecDeque::from([0xaa, 0xab]));
        assert_eq!(mock_uxn.return_stack.into_inner(),
        VecDeque::new());
    }

    #[test]
    fn test_lit_handler_ret_mode() {
        let mut mock_uxn = MockUxn{
            read_next_byte_from_ram_ret_values: RefCell::new(VecDeque::from([Ok(0xaa),])),
            ram_values: RefCell::new(VecDeque::new()),
            ram_addresses_recvd: RefCell::new(VecDeque::new()),
            get_prog_counter_ret_values: RefCell::new(VecDeque::new()),
            prog_counter_recv_values: RefCell::new(VecDeque::new()),
            working_stack: RefCell::new(VecDeque::new()),
            return_stack: RefCell::new(VecDeque::new()),
        };

        lit_handler(Box::new(&mut mock_uxn),
            false, false, true).unwrap();

        assert_eq!(mock_uxn.return_stack.into_inner(),
        VecDeque::from([0xaa,]));
        assert_eq!(mock_uxn.working_stack.into_inner(),
        VecDeque::new());
    }
}
