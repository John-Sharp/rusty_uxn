use super::UxnWrapper;
use super::UxnWithDevices;
use super::UxnError;

fn do_signed_jump(wrapper: &mut UxnWrapper, dst: i8) -> Result<u16, UxnError> {
    let current_pc = wrapper.uxn.get_program_counter().unwrap();

    let dst = i32::from(current_pc) + i32::from(dst);

    if let Ok(dst) = u16::try_from(dst) {
        wrapper.uxn.set_program_counter(dst);
    } else {
        return Err(UxnError::OutOfRangeMemoryAddress);
    };

    return Ok(current_pc);
}

// jump handler: moves the program counter by a signed value equal to the byte on the top of the stack, or an absolute address in short mode
pub fn jmp_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let dst = wrapper.pop_short()?;

        wrapper.uxn.set_program_counter(dst);
    } else {
        let dst = wrapper.pop()?;
        do_signed_jump(&mut wrapper, i8::from_be_bytes([dst]))?;
    }

    return Ok(());
}

// jump conditional handler: if the byte preceeding the address is not 00, moves the program counter by a signed value equal to the byte on the top of the stack, or an absolute address in short mode
pub fn jcn_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let dst = wrapper.pop_short()?;

        let b = wrapper.pop()?;

        if b != 0 {
            wrapper.uxn.set_program_counter(dst);
        }
    } else {
        let dst = wrapper.pop()?;

        let b = wrapper.pop()?;
        if b != 0 {
            do_signed_jump(&mut wrapper, i8::from_be_bytes([dst]))?;
        }
    }

    return Ok(());
}

// jump stash return handler: pushes the value of the program counter to the return-stack and moves the program counter by a signed value equal to the byte on the top of the stack, or an absolute address in short mode
pub fn jsr_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let dst = wrapper.pop_short()?;

        let pc = wrapper.uxn.get_program_counter()?;
        let [b2, b1] = pc.to_be_bytes();

        wrapper.push_to_return_stack(b2)?;
        wrapper.push_to_return_stack(b1)?;

        wrapper.uxn.set_program_counter(dst);
    } else {
        let dst = wrapper.pop()?;
        let pc = do_signed_jump(&mut wrapper, i8::from_be_bytes([dst]))?;
        let [b2, b1] = pc.to_be_bytes();
        wrapper.push_to_return_stack(b2)?;
        wrapper.push_to_return_stack(b1)?;
    }

    return Ok(());
}

// stash handler: moves the value at the top of the stack, to the return stack
pub fn sth_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let b1 = wrapper.pop()?;
        let b2 = wrapper.pop()?;

        wrapper.push_to_return_stack(b2)?;
        wrapper.push_to_return_stack(b1)?;
    } else {
        let b = wrapper.pop()?;
        wrapper.push_to_return_stack(b)?;
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
    fn test_jmp_handler() {
        let mut mock_uxn = MockUxn::new();

        let jmp_val = -0x11i8;
        let jmp_val = jmp_val.to_be_bytes()[0];
        
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(jmp_val)]));
        mock_uxn.get_program_counter_values_to_return = RefCell::new(VecDeque::from([Ok(0x23)]));

        jmp_handler(&mut mock_uxn, false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([(0x12,),]) // program counter is mocked to be 0x23, jump value is -0x11,
                                       // so program counter should be set to 0x23-0x11=0x12
        );
    }

    #[test]
    fn test_jmp_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xbb)]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(())]));

        jmp_handler(&mut mock_uxn, true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([(0xbbaa,),])
        );

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xbb,), (0xaa,),])
        );
    }

    #[test]
    fn test_jmp_handler_out_of_range() {
        let mut mock_uxn = MockUxn::new();
        let jmp_val = 0x1;
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(jmp_val)]));
        mock_uxn.get_program_counter_values_to_return = RefCell::new(VecDeque::from([Ok(0xffff)]));

        let ret = jmp_handler(&mut mock_uxn, false, false, false);

        assert_eq!(
            ret,
            Err(UxnError::OutOfRangeMemoryAddress));
    }

    #[test]
    fn test_jcn_handler() {
        let mut mock_uxn = MockUxn::new();

        let jmp_val = -0x11i8;
        let jmp_val = jmp_val.to_be_bytes()[0];
        
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(jmp_val), Ok(0x01)]));
        mock_uxn.get_program_counter_values_to_return = RefCell::new(VecDeque::from([Ok(0x23)]));

        jcn_handler(&mut mock_uxn, false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([(0x12,),]) // program counter is mocked to be 0x23, jump value is -0x11,
                                       // so program counter should be set to 0x23-0x11=0x12
        );
    }
    
    #[test]
    fn test_jcn_handler_condition_false() {
        let mut mock_uxn = MockUxn::new();

        let jmp_val = -0x11i8;
        let jmp_val = jmp_val.to_be_bytes()[0];

        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(jmp_val), Ok(0x00)]));

        jcn_handler(&mut mock_uxn, false, false, false).unwrap();
        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([]) // since condition was 0x00 no program counter should be set
        );
    }

    #[test]
    fn test_jcn_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xbb), Ok(0x01)]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()), Ok(()),]));

        jcn_handler(&mut mock_uxn, true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([(0xbbaa,),])
        );

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0x01,), (0xbb,), (0xaa,),])
        );
    }

    #[test]
    fn test_jsr_handler() {
        let mut mock_uxn = MockUxn::new();

        let jmp_val = -0x11i8;
        let jmp_val = jmp_val.to_be_bytes()[0];
        
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(jmp_val)]));
        mock_uxn.get_program_counter_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa23)]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()),]));

        jsr_handler(&mut mock_uxn, false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa12,),]) // program counter is mocked to be 0x23, jump value is -0x11,
                                       // so program counter should be set to 0xaa23-0x11=0xaa12
        );

        // the old program counter should also have been pushed to the return stack (broken into bytes)
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,), (0x23,)])
        );
    }

    #[test]
    fn test_jsr_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xbb)]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(())]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()),]));
        mock_uxn.get_program_counter_values_to_return = RefCell::new(VecDeque::from([Ok(0x0102),]));

        jsr_handler(&mut mock_uxn, true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .set_program_counter_arguments_received
                .into_inner(),
            VecDeque::from([(0xbbaa,),])
        );

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xbb,), (0xaa,),])
        );

        // old program counter value should also be pushed to the 'return' stack,
        // but since we're in return mode the 'return' stack is actually the working
        // stack
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0x01,), (0x02,),])
        );
    }

    #[test]
    fn test_jsr_handler_out_of_range() {
        let mut mock_uxn = MockUxn::new();
        let jmp_val = 0x1;
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(jmp_val)]));
        mock_uxn.get_program_counter_values_to_return = RefCell::new(VecDeque::from([Ok(0xffff)]));

        let ret = jsr_handler(&mut mock_uxn, false, false, false);

        assert_eq!(
            ret,
            Err(UxnError::OutOfRangeMemoryAddress));
    }

    #[test]
    fn test_sth_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xcd)]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(())]));

        sth_handler(&mut mock_uxn, false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xcd,),])
        );
    }

    #[test]
    fn test_sth_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xcd), Ok(0xab)]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()), Ok(()),]));

        sth_handler(&mut mock_uxn, true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xab,),(0xcd,),])
        );

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xab,),(0xcd,),])
        );
    }
}
