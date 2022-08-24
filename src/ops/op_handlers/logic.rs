use super::UxnWrapper;
use super::UxnWithDevices;
use super::UxnError;

// equal handler: pushes 01 to the stack if the two values at the top of the stack are equal, 00 otherwise
pub fn equ_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        if a == b {
            wrapper.push(1)?;
        } else {
            wrapper.push(0)?;
        }
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        if a == b {
            wrapper.push(1)?;
        } else {
            wrapper.push(0)?;
        }
    }

    return Ok(());
}

// not equal handler: pushes 01 to the stack if the two values at the top of the stack are not equal, 00 otherwise
pub fn neq_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        if a == b {
            wrapper.push(0)?;
        } else {
            wrapper.push(1)?;
        }
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        if a == b {
            wrapper.push(0)?;
        } else {
            wrapper.push(1)?;
        }
    }

    return Ok(());
}

// greater than handler: pushes 01 to the stack if the second value at the top of the stack is greater than the value at the top of the stack, 00 otherwise
pub fn gth_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        if b > a {
            wrapper.push(1)?;
        } else {
            wrapper.push(0)?;
        }
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        if b > a {
            wrapper.push(1)?;
        } else {
            wrapper.push(0)?;
        }
    }

    return Ok(());
}

// lesser than handler: pushes 01 to the stack if the second value at the top of the stack is lesser than the value at the top of the stack, 00 otherwise
pub fn lth_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        if b < a {
            wrapper.push(1)?;
        } else {
            wrapper.push(0)?;
        }
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        if b < a {
            wrapper.push(1)?;
        } else {
            wrapper.push(0)?;
        }
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
    fn test_equ_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xaa),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        equ_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x01,),])
        );
    }

    #[test]
    fn test_equ_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xba), Ok(0xaa), Ok(0xba),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()),]));

        equ_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xba,), (0xaa,),
             (0xba,), (0xaa,),
             (0x01,),])
        );
    }

    #[test]
    fn test_neq_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xaa),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        neq_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x00,),])
        );
    }

    #[test]
    fn test_neq_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xba), Ok(0xaa), Ok(0xba),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()),]));

        neq_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xba,), (0xaa,),
             (0xba,), (0xaa,),
             (0x00,),])
        );
    }

    #[test]
    fn test_gth_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x03), Ok(0x05),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        gth_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x01,),])
        );
    }

    #[test]
    fn test_gth_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xba), Ok(0xaa), Ok(0xbb),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()),]));

        gth_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xbb,), (0xaa,),
             (0xba,), (0xaa,),
             (0x01,),])
        );
    }

    #[test]
    fn test_lth_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x03), Ok(0x05),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        lth_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x00,),])
        );
    }

    #[test]
    fn test_lth_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0xaa), Ok(0xba), Ok(0xaa), Ok(0xbb),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
          Ok(()), Ok(()),
          Ok(()), Ok(()),
          Ok(()),]));

        lth_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
             (0xbb,), (0xaa,),
             (0xba,), (0xaa,),
             (0x00,),])
        );
    }
}
