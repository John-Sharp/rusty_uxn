use super::UxnWrapper;
use super::UxnWithDevices;
use super::UxnError;

// add handler: pushes the sum of the two values at the top of the stack
pub fn add_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()? as i32;
        let b = wrapper.pop_short()? as i32;

        wrapper.push_short((a+b) as u16)?;
    } else {
        let a = wrapper.pop()? as i16;
        let b = wrapper.pop()? as i16;

        wrapper.push((a+b) as u8)?;
    }

    return Ok(());
}

// subtract handler: pushes the difference of the first value minus the second, to the top of the stack
pub fn sub_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()? as i32;
        let b = wrapper.pop_short()? as i32;

        wrapper.push_short((b-a) as u16)?;
    } else {
        let a = wrapper.pop()? as i16;
        let b = wrapper.pop()? as i16;

        wrapper.push((b-a) as u8)?;
    }

    return Ok(());
}

// multiply handler: pushes the product of the first and second values at the top of the stack
pub fn mul_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        wrapper.push_short(a.wrapping_mul(b))?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        wrapper.push(a.wrapping_mul(b))?;
    }

    return Ok(());

}

// divide handler: pushes the quotient of the first value over the second, to the top of the stack
pub fn div_handler(
    u: &mut dyn UxnWithDevices,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {
    let mut wrapper = UxnWrapper::new(u, keep, ret);

    if short == true {
        let a = wrapper.pop_short()?;
        let b = wrapper.pop_short()?;

        wrapper.push_short(b/a)?;
    } else {
        let a = wrapper.pop()?;
        let b = wrapper.pop()?;

        wrapper.push(b/a)?;
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
    fn test_add_handler() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x11),
            Ok(0x12),
        ]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
        ]));

        add_handler(&mut mock_uxn, false, false, false).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x23,),])
        );
    }

    #[test]
    fn test_add_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x11),
            Ok(0x12),
            Ok(0x13),
            Ok(0x14),
        ]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
        ]));

        add_handler(&mut mock_uxn, true, true, true).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
              (0x14,),
              (0x13,),
              (0x12,),
              (0x11,),
              (0x26,),         // 0x1413 + 0x1211 = 0x2624
              (0x24,),
            ])
        );
    }

    #[test]
    fn test_sub_handler() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x11),
            Ok(0x14),
        ]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
        ]));

        sub_handler(&mut mock_uxn, false, false, false).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x03,),])
        );
    }

    #[test]
    fn test_sub_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x12),
            Ok(0x12),
            Ok(0x13),
            Ok(0x14),
        ]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
        ]));

        sub_handler(&mut mock_uxn, true, true, true).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
              (0x14,),
              (0x13,),
              (0x12,),
              (0x12,),
              (0x02,),         // 0x1413 - 0x1212 = 0x0201
              (0x01,),
            ])
        );
    }

    #[test]
    fn test_mul_handler() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x02),
            Ok(0x02),
        ]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
        ]));

        mul_handler(&mut mock_uxn, false, false, false).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x04,),])
        );
    }

    #[test]
    fn test_mul_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x11),
            Ok(0x00),
            Ok(0x14),
            Ok(0x00),
        ]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
        ]));

        mul_handler(&mut mock_uxn, true, true, true).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
              (0x00,),
              (0x14,),
              (0x00,),
              (0x11,),
              (0x01,),         // 0x0011 * 0x0014 = 0x0154
              (0x54,),
            ])
        );
    }

    #[test]
    fn test_div_handler() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x02),
            Ok(0x08),
        ]));
        mock_uxn.push_to_working_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
        ]));

        div_handler(&mut mock_uxn, false, false, false).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_working_stack_arguments_received
                .into_inner(),
            VecDeque::from([(0x04,),])
        );
    }

    #[test]
    fn test_div_handler_keep_short_return_mode() {
        let mut mock_uxn = MockUxn::new();

        mock_uxn.pop_from_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(0x03),
            Ok(0x00),
            Ok(0x51),
            Ok(0x03),
        ]));
        mock_uxn.push_to_return_stack_values_to_return = RefCell::new(VecDeque::from([
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
        ]));

        div_handler(&mut mock_uxn, true, true, true).unwrap();
        assert_eq!(
            mock_uxn
                .push_to_return_stack_arguments_received
                .into_inner(),
            VecDeque::from([
              (0x03,),
              (0x51,),
              (0x00,),
              (0x03,),
              (0x01,),         // 0x0351 / 0x0003 = 0x011b
              (0x1b,),
            ])
        );
    }
}
