use super::UxnWrapper;
use super::UxnWithDevices;
use super::UxnError;

// and handler: pushes the result of the bitwise operation AND, to the top of the stack
pub fn and_handler(
    u: Box<&mut dyn UxnWithDevices>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        let res = a & b;
        wrapper.push_short(res)?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        let res = a & b;
        wrapper.push(res)?;
    }

    return Ok(());
}

// or handler: pushes the result of the bitwise operation OR, to the top of the stack
pub fn ora_handler(
    u: Box<&mut dyn UxnWithDevices>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        let res = a | b;
        wrapper.push_short(res)?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        let res = a | b;
        wrapper.push(res)?;
    }

    return Ok(());
}

// exclusive or handler: pushes the result of the bitwise operation XOR, to the top of the stack
pub fn eor_handler(
    u: Box<&mut dyn UxnWithDevices>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        let res = a ^ b;
        wrapper.push_short(res)?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        let res = a ^ b;
        wrapper.push(res)?;
    }

    return Ok(());
}

// shift handler: pushes the result of the bitwise operation XOR, to the top of the stack
pub fn sft_handler(
    u: Box<&mut dyn UxnWithDevices>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop()?;
        let b = wrapper.pop_short()?;

        let res = b >> (a & 0xf) << ((a&0xf0) >> 4);
        wrapper.push_short(res)?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        let res = b >> (a & 0xf) << ((a&0xf0) >> 4);
        wrapper.push(res)?;
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
    fn test_and_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x0a), Ok(0xaa),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        and_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x0a,),])
        );
    }

    #[test]
    fn test_and_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x0a), Ok(0x0b), Ok(0xaa), Ok(0xbb),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
        ]));

        and_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
                    (0xbb,),
                    (0xaa,),
                    (0x0b,),
                    (0x0a,),
                    (0x0b,),
                    (0x0a,),
            ])
        );
    }

    #[test]
    fn test_ora_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x0a), Ok(0xa0),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        ora_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,),])
        );
    }

    #[test]
    fn test_ora_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x0a), Ok(0x0b), Ok(0xa0), Ok(0xb0),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
        ]));

        ora_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
                    (0xb0,),
                    (0xa0,),
                    (0x0b,),
                    (0x0a,),
                    (0xbb,),
                    (0xaa,),
            ])
        );
    }

    #[test]
    fn test_eor_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x0a), Ok(0xa0),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        eor_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xaa,),])
        );
    }

    #[test]
    fn test_eor_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x0a), Ok(0x0b), Ok(0xa0), Ok(0xb0),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
        ]));

        eor_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
                    (0xb0,),
                    (0xa0,),
                    (0x0b,),
                    (0x0a,),
                    (0xbb,),
                    (0xaa,),
            ])
        );
    }

    #[test]
    fn test_sft_handler() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x21), Ok(0xff),]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([Ok(()),]));

        sft_handler(Box::new(&mut mock_uxn), false, false, false).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0xfc,),]) // 0xff left shifted by 1, then
                                       // right shifted by 2
        );
    }

    #[test]
    fn test_sft_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();
        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([Ok(0x21), Ok(0xff), Ok(0xff),]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
           Ok(()),
        ]));

        sft_handler(Box::new(&mut mock_uxn), true, true, true).unwrap();

        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
                    (0xff,),
                    (0xff,),
                    (0x21,),
                    (0xff,),
                    (0xfc,),]) // 0xff right shifted by 1, then
                               // left shifted by 2
        );
    }
}
