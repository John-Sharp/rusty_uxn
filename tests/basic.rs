use uuid::Uuid;
use std::fs;
use rusty_uxn::emulators::uxnclilib;

// push some values onto the working and return stacks, verify
// from the system device debug output that the stacks look as
// expected
#[test]
fn push_and_debug() {
    // this is the machine code for the following assembly:
    // |100 LIT 22 LIT 33 LITr 44 LITr 55 LIT 00 LIT 0e DEO LIT 01 LIT 0f DEO
    let prog = vec![0x80, 0x22, 0x80, 0x33, 0xc0, 0x44, 0xc0, 0x55,
        0x80, 0x00, 0x80, 0x0e, 0x17, 0x80, 0x01, 0x80, 0x0f, 0x17];

    let tmp_file_name = format!("push_and_debug{}", Uuid::new_v4());
    let mut tmp_file_path = std::env::temp_dir();
    tmp_file_path.push(tmp_file_name);

    fs::write(&tmp_file_path, &prog).expect("Failed to write test program");

    let cli_options = uxnclilib::Cli{rom: tmp_file_path, input: "".to_string()};
    let mut stdout_output = Vec::new();
    let mut stderr_output = Vec::new();
    let mut debug_output = Vec::new();
    let config = uxnclilib::Config{
        stdout_writer: &mut stdout_output,
        stderr_writer: &mut stderr_output,
        debug_writer: &mut debug_output};

    uxnclilib::run(cli_options, config).expect("Failed to execute test program");

    // the debug output should be printed to the debug_writer and should give the contents of the working
    // stack followed by the contents of the return stack
    assert_eq!(String::from_utf8(debug_output).unwrap(), "<wst> 22 33\n<rst> 44 55\n");
}
