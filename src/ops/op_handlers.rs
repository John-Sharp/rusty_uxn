use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

struct UxnWrapper<'a> {
    uxn: Box<&'a mut dyn Uxn>,
    push_fn: fn(&mut (dyn Uxn + 'a), u8) -> Result<(), UxnError>,
    pop_fn: fn(&mut (dyn Uxn + 'a)) -> Result<u8, UxnError>,
    keep: bool,
    popped_values: Vec<u8>,
}

impl<'a> UxnWrapper<'a> {
    fn new(uxn: Box<&'a mut dyn Uxn>, keep: bool, ret: bool) -> Self {
        let push_fn = if ret == false {
            Uxn::push_to_working_stack
        } else {
            Uxn::push_to_return_stack
        };

        let pop_fn = if ret == false {
            Uxn::pop_from_working_stack
        } else {
            Uxn::pop_from_return_stack
        };

        UxnWrapper {
            uxn,
            push_fn,
            pop_fn,
            keep,
            popped_values: Vec::new(),
        }
    }

    fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
        self.uxn.read_next_byte_from_ram()
    }

    fn read_from_ram(&self, addr: u16) -> u8 {
        self.uxn.read_from_ram(addr)
    }

    fn push(&mut self, byte: u8) -> Result<(), UxnError> {
        // If in keep mode, popped_values will be populated with
        // what has been popped in the course of this operation.
        // Push these back on the stack to restore it to its
        // state prior to any pops before pushing the desired
        // value
        while let Some(val) = self.popped_values.pop() {
            (self.push_fn)(*self.uxn, val).expect("Couldn't push");
        }

        (self.push_fn)(*self.uxn, byte)
    }

    fn pop(&mut self) -> Result<u8, UxnError> {
        let popped = (self.pop_fn)(*self.uxn)?;

        if self.keep {
            self.popped_values.push(popped);
        }

        return Ok(popped);
    }

    fn write_to_device(&mut self, device_address: u8, val: u8) {
        self.uxn.write_to_device(device_address, val)
    }
}

impl<'a> Drop for UxnWrapper<'a> {
    fn drop(&mut self) {
        // if in keep mode, popped values will be populated with
        // what has been popped in the course of this operation.
        // Push these back onto the stack to ensure they are kept
        while let Some(val) = self.popped_values.pop() {
            (self.push_fn)(*self.uxn, val).expect("Couldn't push");
        }
    }
}

// literal handler: pushes the next value seen in the program onto the stack
pub fn lit_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    // read byte/short from ram
    let a = wrapper.read_next_byte_from_ram()?;

    wrapper.push(a)?;

    if short == false {
        return Ok(());
    }

    let a = wrapper.read_next_byte_from_ram()?;
    wrapper.push(a)?;

    return Ok(());
}

// increment handler: adds 1 to value at top of stack
pub fn inc_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == false {
        let val = wrapper.pop()?;
        let val = val + 1;
        wrapper.push(val)?;
    } else {
        let val_b1 = wrapper.pop()?;
        let val_b2 = wrapper.pop()?;
        let val = u16::from_be_bytes([val_b2, val_b1]) + 1;
        let [val_b2, val_b1] = val.to_be_bytes();
        wrapper.push(val_b2)?;
        wrapper.push(val_b1)?;
    }

    return Ok(());
}

// pop handler: removes the value at the top of the stack
pub fn pop_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    // pop with the keep flag is a no-op
    if keep == true {
        return Ok(());
    }

    let mut wrapper = UxnWrapper::new(u, keep, ret);
    wrapper.pop()?;

    if short == true {
        wrapper.pop()?;
    }

    return Ok(());
}

// duplicate handler: duplicates the value at the top of the stack
pub fn dup_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let byte_low = wrapper.pop()?;
        let byte_high = wrapper.pop()?;

        wrapper.push(byte_high)?;
        wrapper.push(byte_low)?;
        wrapper.push(byte_high)?;
        wrapper.push(byte_low)?;

    } else {
        let byte = wrapper.pop()?;
        wrapper.push(byte)?;
        wrapper.push(byte)?;
    }

    return Ok(());
}

//nip handler: removes the second value from the stack
pub fn nip_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let byte_low = wrapper.pop()?;
        let byte_high = wrapper.pop()?;
    
        let _discarded_low = wrapper.pop()?;
        let _discarded_high = wrapper.pop()?;

        wrapper.push(byte_high)?;
        wrapper.push(byte_low)?;
    } else {
        let byte = wrapper.pop()?;
        let _discarded = wrapper.pop()?;

        wrapper.push(byte)?;
    }

    return Ok(());
}

// swap handler: exchanges the first and second values at the top of the stack
pub fn swp_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let short_low_byte = wrapper.pop()?;
        let short_high_byte = wrapper.pop()?;
    
        let short2_low_byte = wrapper.pop()?;
        let short2_high_byte = wrapper.pop()?;

        wrapper.push(short_high_byte)?;
        wrapper.push(short_low_byte)?;

        wrapper.push(short2_high_byte)?;
        wrapper.push(short2_low_byte)?;
    } else {
        let byte = wrapper.pop()?;
        let byte2 = wrapper.pop()?;

        wrapper.push(byte)?;
        wrapper.push(byte2)?;
    }

    return Ok(());
}

// over handler: duplicates the second value at the top of the stack
pub fn ovr_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let short_low_byte = wrapper.pop()?;
        let short_high_byte = wrapper.pop()?;
    
        let short2_low_byte = wrapper.pop()?;
        let short2_high_byte = wrapper.pop()?;

        wrapper.push(short2_high_byte)?;
        wrapper.push(short2_low_byte)?;

        wrapper.push(short_high_byte)?;
        wrapper.push(short_low_byte)?;
        
        wrapper.push(short2_high_byte)?;
        wrapper.push(short2_low_byte)?;
    } else {
        let byte = wrapper.pop()?;
        let byte2 = wrapper.pop()?;

        wrapper.push(byte2)?;
        wrapper.push(byte)?;
        wrapper.push(byte2)?;
    }

    return Ok(());
}

// rotate handler: rotates three values at the top of the stack, to the left, wrapping
pub fn rot_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a_low = wrapper.pop()?;
        let a_high = wrapper.pop()?;

        let b_low = wrapper.pop()?;
        let b_high = wrapper.pop()?;

        let c_low = wrapper.pop()?;
        let c_high = wrapper.pop()?;

        wrapper.push(b_high)?;
        wrapper.push(b_low)?;

        wrapper.push(a_high)?;
        wrapper.push(a_low)?;

        wrapper.push(c_high)?;
        wrapper.push(c_low)?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;
        let c = wrapper.pop()?;

        wrapper.push(b)?;
        wrapper.push(a)?;
        wrapper.push(c)?;
    }

    return Ok(());
}

pub fn deo_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    // get device address to write to from working/return stack
    let device_address = wrapper.pop()?;

    // pop byte from working/return stack
    let value = wrapper.pop()?;

    // write byte to device responsible for device address
    wrapper.write_to_device(device_address, value);

    // if in short mode get another byte from working/return stack
    // and write to device
    if short == true {
        let value = wrapper.pop()?;
        wrapper.write_to_device(device_address, value);
    }
    return Ok(());
}

// load absolute handler: push value at absolute address to the top of the stack
pub fn lda_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    // get 16 bit absolute address (first byte on stack provides least significant byte of address,
    // second byte on stack the most significant)
    let address = wrapper.pop()? as u16;
    let address = address + ((wrapper.pop()? as u16) << 8);

    let value = wrapper.read_from_ram(address);
    wrapper.push(value)?;

    if short == false {
        return Ok(());
    }

    if address == u16::MAX {
        return Err(UxnError::OutOfRangeMemoryAddress);
    }

    let value = wrapper.read_from_ram(address + 1);
    wrapper.push(value)?;

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
        push_to_return_stack_values_to_return: RefCell<VecDeque<Result<(), UxnError>>>,

        push_to_working_stack_arguments_received: RefCell<VecDeque<(u8,)>>,
        push_to_working_stack_values_to_return: RefCell<VecDeque<Result<(), UxnError>>>,

        pop_from_working_stack_arguments_received: RefCell<VecDeque<()>>,
        pop_from_working_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        pop_from_return_stack_arguments_received: RefCell<VecDeque<()>>,
        pop_from_return_stack_values_to_return: RefCell<VecDeque<Result<u8, UxnError>>>,

        write_to_device_arguments_received: RefCell<VecDeque<(u8, u8)>>,
    }

    impl MockUxn {
        fn new() -> Self {
            MockUxn {
                read_next_byte_from_ram_arguments_received: RefCell::new(VecDeque::new()),
                read_next_byte_from_ram_values_to_return: RefCell::new(VecDeque::from([
                    Ok(0xaa),
                    Ok(0xab),
                ])),

                read_from_ram_arguments_received: RefCell::new(VecDeque::new()),
                read_from_ram_values_to_return: RefCell::new(VecDeque::new()),

                get_program_counter_arguments_received: RefCell::new(VecDeque::new()),
                get_program_counter_values_to_return: RefCell::new(VecDeque::new()),

                set_program_counter_arguments_received: RefCell::new(VecDeque::new()),

                push_to_return_stack_arguments_received: RefCell::new(VecDeque::new()),
                push_to_return_stack_values_to_return: RefCell::new(VecDeque::new()),

                push_to_working_stack_arguments_received: RefCell::new(VecDeque::new()),
                push_to_working_stack_values_to_return: RefCell::new(VecDeque::new()),

                pop_from_working_stack_arguments_received: RefCell::new(VecDeque::new()),
                pop_from_working_stack_values_to_return: RefCell::new(VecDeque::new()),

                pop_from_return_stack_arguments_received: RefCell::new(VecDeque::new()),
                pop_from_return_stack_values_to_return: RefCell::new(VecDeque::new()),

                write_to_device_arguments_received: RefCell::new(VecDeque::new()),
            }
        }
    }

    impl Uxn for MockUxn {
        fn read_next_byte_from_ram(&mut self) -> Result<u8, UxnError> {
            self.read_next_byte_from_ram_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .read_next_byte_from_ram_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn read_from_ram(&self, addr: u16) -> u8 {
            self.read_from_ram_arguments_received
                .borrow_mut()
                .push_back((addr,));
            return self
                .read_from_ram_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn get_program_counter(&self) -> Result<u16, UxnError> {
            self.get_program_counter_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .get_program_counter_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn set_program_counter(&mut self, addr: u16) {
            self.set_program_counter_arguments_received
                .borrow_mut()
                .push_back((addr,));
        }

        fn push_to_return_stack(&mut self, byte: u8) -> Result<(), UxnError> {
            self.push_to_return_stack_arguments_received
                .borrow_mut()
                .push_back((byte,));

            return self
                .push_to_return_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn push_to_working_stack(&mut self, byte: u8) -> Result<(), UxnError> {
            self.push_to_working_stack_arguments_received
                .borrow_mut()
                .push_back((byte,));

            return self
                .push_to_working_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn pop_from_working_stack(&mut self) -> Result<u8, UxnError> {
            self.pop_from_working_stack_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .pop_from_working_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn pop_from_return_stack(&mut self) -> Result<u8, UxnError> {
            self.pop_from_return_stack_arguments_received
                .borrow_mut()
                .push_back(());
            return self
                .pop_from_return_stack_values_to_return
                .borrow_mut()
                .pop_front()
                .unwrap();
        }

        fn write_to_device(&mut self, device_address: u8, val: u8) {
            self.write_to_device_arguments_received
                .borrow_mut()
                .push_back((device_address, val));
        }
    }

    #[test]
    fn test_lit_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.read_next_byte_from_ram_values_to_return =
            RefCell::new(VecDeque::from([Ok(0xaa)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        lit_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .read_next_byte_from_ram_arguments_received
                .into_inner(),
            VecDeque::from([(),])
        );
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,),])
        );
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::new()
        );
    }

    #[test]
    fn test_lit_handler_short_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.read_next_byte_from_ram_values_to_return =
            RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab)]));
        mock_uxn.push_to_working_stack_values_to_return =
            RefCell::new(VecDeque::from([Ok(()), Ok(())]));

        lit_handler(Box::new(&mut mock_uxn), false, true, false).unwrap();

        assert_eq!(
            mock_uxn
                .read_next_byte_from_ram_arguments_received
                .into_inner(),
            VecDeque::from([(), ()])
        );
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,), (0xab,),])
        );
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::new()
        );
    }

    #[test]
    fn test_lit_handler_ret_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.read_next_byte_from_ram_values_to_return =
            RefCell::new(VecDeque::from([Ok(0xaa)]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        lit_handler(Box::new(&mut mock_uxn), false, false, true).unwrap();

        assert_eq!(
            mock_uxn
                .read_next_byte_from_ram_arguments_received
                .into_inner(),
            VecDeque::from([(),])
        );
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::new()
        );
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,),])
        );
    }

    #[test]
    fn test_inc_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        inc_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xab,),])
        );
    }

    #[test]
    fn test_inc_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return =
            RefCell::new(VecDeque::from([Ok(0xab), Ok(0xaa)])); // will be
                                                                // treated as
                                                                // the short
                                                                // 0xaaab
        mock_uxn.push_to_return_stack_values_to_return =
            RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(())]));

        inc_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,), (0xab,), (0xaa,), (0xac,),])
        );
    }

    #[test]
    fn test_pop_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa)]));

        pop_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .pop_from_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(),])
        );
    }

    #[test]
    fn test_pop_handler_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab)]));

        pop_handler(Box::new(&mut mock_uxn), false, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .pop_from_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(), (),])
        );
    }

    #[test]
    fn test_dup_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()),]));

        dup_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,), (0xaa,)])
        );
    }

    #[test]
    fn test_dup_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xbb)]));
        // will push one short because in 'keep mode', and two shorts
        // to duplicate the short popped
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()),]));

        dup_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xbb,), (0xaa,), (0xbb,), (0xaa,), (0xbb,), (0xaa,),])
        );
    }

    #[test]
    fn test_nip_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        nip_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,),])
        );
    }

    #[test]
    fn test_nip_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab), Ok(0xac), Ok(0xad),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()),]));

        nip_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xad,), (0xac,), (0xab,), (0xaa,), (0xab,), (0xaa,),])
        );
    }

    #[test]
    fn test_swp_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()),]));

        swp_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,), (0xab,),])
        );
    }

    #[test]
    fn test_swp_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab), Ok(0xac), Ok(0xad),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()),]));

        swp_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xad,), (0xac,), (0xab,), (0xaa,), (0xab,), (0xaa,), (0xad,), (0xac,)])
        );
    }

    #[test]
    fn test_ovr_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()),]));

        ovr_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xab,), (0xaa,), (0xab,)])
        );
    }

    #[test]
    fn test_ovr_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab), Ok(0xac), Ok(0xad),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()), Ok(()),]));

        ovr_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xad,), (0xac,), (0xab,), (0xaa,), (0xad,), (0xac,), (0xab,), (0xaa,), (0xad,), (0xac,)])
        );
    }

    #[test]
    fn test_rot_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xab),  Ok(0xac),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()),]));

        rot_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xab,), (0xaa,), (0xac,)])
        );
    }

    #[test]
    fn test_rot_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xba), Ok(0xab), Ok(0xbb), Ok(0xac), Ok(0xbc),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()), Ok(()),]));

        rot_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xbc,), (0xac,),
             (0xbb,), (0xab,),
             (0xba,), (0xaa,),

             (0xbb,), (0xab,),
             (0xba,), (0xaa,),
             (0xbc,), (0xac,),])
        );
    }

    #[test]
    fn test_deo_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0xaa), // should be used as device address
            Ok(0xba), // should be used as value to write
        ]));

        deo_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .pop_from_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(), (),])
        );
        assert_eq!(
            mock_uxn.write_to_device_arguments_received.into_inner(),
            VecDeque::from([(0xaa, 0xba),])
        );
    }

    #[test]
    fn test_deo_handler_keep_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0xaa), // should be used as device address
            Ok(0xba), // should be used as value to write
        ]));

        mock_uxn.push_to_working_stack_values_to_return =
            RefCell::new(VecDeque::from([Ok(()), Ok(())]));

        deo_handler(Box::new(&mut mock_uxn), true, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .pop_from_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(), (),])
        );
        assert_eq!(
            mock_uxn.write_to_device_arguments_received.into_inner(),
            VecDeque::from([(0xaa, 0xba),])
        );

        // since in keep mode what was popped from the stack should be pushed
        // back onto the stack at the end of handling the operation, check this
        // is the case
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xba,), (0xaa,),])
        );
    }

    #[test]
    fn test_deo_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0xaa), // should be used as device address
            Ok(0xba), // should be used as value to write
            Ok(0xc1), // should be used as second value to write
        ]));

        mock_uxn.push_to_return_stack_values_to_return =
            RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(())]));

        deo_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .pop_from_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(), (), (),])
        );

        // two write_to_device function calls should be made, writing to the
        // same device address, but with the first and second byte of the
        // short
        assert_eq!(
            mock_uxn.write_to_device_arguments_received.into_inner(),
            VecDeque::from([(0xaa, 0xba), (0xaa, 0xc1)])
        );

        // since in keep mode what was popped from the stack should be pushed
        // back onto the stack at the end of handling the operation, check this
        // is the case
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xc1,), (0xba,), (0xaa,),])
        );
    }

    #[test]
    fn test_lda_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0xa1), // least significant byte of address to load
            Ok(0xb2), // most significant byte
        ]));
        mock_uxn.read_from_ram_values_to_return = RefCell::new(VecDeque::from([0xc3])); // value 'read from ram'
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        lda_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        // handler should attempt to read from the address constructed out of
        // bytes it popped from the stack
        assert_eq!(
            mock_uxn.read_from_ram_arguments_received.into_inner(),
            VecDeque::from([(0xb2a1,)])
        );

        // handler should then write the value read from ram to the stack
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xc3,)])
        );
    }

    #[test]
    fn test_lda_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0xaa), // least significant byte of address to load
            Ok(0xba), // most significant byte
        ]));

        mock_uxn.push_to_return_stack_values_to_return =
            RefCell::new(VecDeque::from([Ok(()), Ok(()), Ok(()), Ok(())]));

        mock_uxn.read_from_ram_values_to_return = RefCell::new(VecDeque::from([0xd3, 0xd4])); // short that is 'read from ram'

        lda_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        // handler should attempt to read from the address constructed out of
        // bytes it popped from the stack, and then from (address+1)
        assert_eq!(
            mock_uxn.read_from_ram_arguments_received.into_inner(),
            VecDeque::from([(0xbaaa,), (0xbaab,)])
        );

        // handler should then write the short read from ram to the stack, preceeded by what was
        // previously on the stack, since the handler was called in keep mode
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xba,), (0xaa,), (0xd3,), (0xd4,)])
        );
    }

    // in short mode, the LDA operation can error if passed an address at
    // the end of ram (since it tries to fetch two bytes from ram). Test
    // the error is correctly triggered
    #[test]
    fn test_lda_handler_out_of_range_error() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0xff), // least significant byte of address to load
            Ok(0xff), // most significant byte, together they make
                      // an address right at the end of addressable
                      // ram
        ]));

        mock_uxn.read_from_ram_values_to_return = RefCell::new(VecDeque::from([0xaa, 0xaa])); // short that would be 'read from ram'
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        let result = lda_handler(Box::new(&mut mock_uxn), false, true, false);

        assert_eq!(result, Err(UxnError::OutOfRangeMemoryAddress));
    }
}
