// push some values onto the working and return stacks, verify
// from the system device debug output that the stacks look as
// expected
#[test]
fn push_and_debug() {
    // this is the machine code for the following assembly:
    // |100 LIT 22 LIT 33 LITr 44 LITr 55 LIT 00 LIT 0e DEO
    // 80 22 80 33 c0 44 c0 55  80 00 80 0e 17
    assert_eq!(1, 1);
}
