use super::UxnWrapper;
use super::UxnWithDevices;
use super::UxnError;

// literal handler: pushes the next value seen in the program onto the stack
pub fn lit_handler(
    u: &mut dyn UxnWithDevices,
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
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == false {
        let val = wrapper.pop()?;
        let val = (val as u16 + 1) as u8;
        wrapper.push(val)?;
    } else {
        let val_b1 = wrapper.pop()?;
        let val_b2 = wrapper.pop()?;
        let val = (u16::from_be_bytes([val_b2, val_b1]) as u32 + 1) as u16;
        let [val_b2, val_b1] = val.to_be_bytes();
        wrapper.push(val_b2)?;
        wrapper.push(val_b1)?;
    }

    return Ok(());
}

// pop handler: removes the value at the top of the stack
pub fn pop_handler(
    u: &mut dyn UxnWithDevices,
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
    u: &mut dyn UxnWithDevices,
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
    u: &mut dyn UxnWithDevices,
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
    u: &mut dyn UxnWithDevices,
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
    u: &mut dyn UxnWithDevices,
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
    u: &mut dyn UxnWithDevices,
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


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::MockUxn;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    #[test]
    fn test_lit_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.read_next_byte_from_ram_values_to_return =
            RefCell::new(VecDeque::from([Ok(0xaa)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        lit_handler(&mut mock_uxn, false, false, false).unwrap();

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

        lit_handler(&mut mock_uxn, false, true, false).unwrap();

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

        lit_handler(&mut mock_uxn, false, false, true).unwrap();

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

        inc_handler(&mut mock_uxn, false, false, false).unwrap();

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

        inc_handler(&mut mock_uxn, true, true, true).unwrap();

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

        pop_handler(&mut mock_uxn, false, false, false).unwrap();

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

        pop_handler(&mut mock_uxn, false, true, true).unwrap();

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

        dup_handler(&mut mock_uxn, false, false, false).unwrap();

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

        dup_handler(&mut mock_uxn, true, true, true).unwrap();

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

        nip_handler(&mut mock_uxn, false, false, false).unwrap();

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

        nip_handler(&mut mock_uxn, true, true, true).unwrap();

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

        swp_handler(&mut mock_uxn, false, false, false).unwrap();

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

        swp_handler(&mut mock_uxn, true, true, true).unwrap();

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

        ovr_handler(&mut mock_uxn, false, false, false).unwrap();

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

        ovr_handler(&mut mock_uxn, true, true, true).unwrap();

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

        rot_handler(&mut mock_uxn, false, false, false).unwrap();

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

        rot_handler(&mut mock_uxn, true, true, true).unwrap();

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
}
