use super::UxnWrapper;
use super::Uxn;
use super::UxnError;

// load zero-page handler: pushes the value at an address within the first 256 bytes of memory, to the top of the stack
pub fn ldz_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {

    return Ok(());
}

// store zero-page handler: writes a value to an address within the first 256 bytes of memory
pub fn stz_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {

    return Ok(());
}

// load relative handler: pushes the value at a relative address, to the top of the stack. The possible relative range is -128 to +127 bytes
pub fn ldr_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {

    return Ok(());
}

// store relative handler: writes a value to a relative address. The possible relative range is -128 to +127 bytes
pub fn str_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {

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

// store absolute handler: writes a value to an absolute address
pub fn sta_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {

    return Ok(());
}

// device input handler: pushes a value from the device page, to the top of the stack. The target device might capture the reading to trigger an I/O event
pub fn dei_handler(
    u: Box<&mut dyn Uxn>,
    keep: bool,
    short: bool,
    ret: bool,
) -> Result<(), UxnError> {

    return Ok(());
}

// device output handler: writes a value to the device page. The target device might capture the writing to trigger an I/O event
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::MockUxn;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    #[test]
    fn test_ldz_handler() {
        let mut mock_uxn = MockUxn::new();
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
}
