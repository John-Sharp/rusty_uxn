use uuid::Uuid;
use std::fs;
use rusty_uxn::emulators::uxnclilib;
use std::io::Cursor;
use chrono::{Local, Datelike};

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
    let stdin_input = Cursor::new("");
    let mut stderr_output = Vec::new();
    let mut debug_output = Vec::new();
    let config = uxnclilib::Config{
        stdout_writer: &mut stdout_output,
        stdin_reader: stdin_input,
        stderr_writer: &mut stderr_output,
        debug_writer: &mut debug_output};

    uxnclilib::run(cli_options, config).expect("Failed to execute test program");

    // the debug output should be printed to the debug_writer and should give the contents of the working
    // stack followed by the contents of the return stack
    assert_eq!(String::from_utf8(debug_output).unwrap(), "<wst> 22 33\n<rst> 44 55\n");
}


// test console, reading in and printing out
#[test]
fn console_test() {

// This program is the compiled result of the following:
//
// %HALT { #010f DEO }
// 
// ( devices )
// |10 @Console [ &vector $2 &read $1 &pad $5 &write $1 &error $1 ]
// 
// |0
// @uname-len $1
// @uname
// 
// |100
// ;on-stdin .Console/vector DEO2
// #00 .uname-len STZ (initialise uname-len)
// BRK
// 
// @on-stdin ( -> )
//     .Console/read DEI DUP
// 
//     LIT 'q EQU ;read-complete JCN2
// 
//     .uname .uname-len LDZ ADD STZ
//       (store character just read into memory pointed to by uname + uname-len)
// 
//     .uname-len LDZ INC .uname-len STZ
// BRK
// 
// @read-complete
//     POP
//     #00 ,&i STR
// 
//     ( print the greeting preamble )
//     ;greeting-preamble
//     &while 
//         ( send ) LDAk .Console/write DEO
//         ( loop ) INC2 LDAk ,&while JCN
//         POP2
// 
//     ( print the name previously entered )
//     &while2
//         .uname ,&i LDR ADD LDZ .Console/write DEO
// 
//         ,&i LDR INC ,&i STR
// 
//         ,&i LDR .uname-len LDZ NEQ  ,&while2 JCN
// 
//     ( print an error message )
//     ;error-msg
//     &while3
//         ( send ) LDAk .Console/error DEO
//         ( loop ) INC2 LDAk ,&while3 JCN
//         POP2
// 
//     HALT
//     BRK
// 
//     &i $1
//
// @greeting-preamble
// "Hello, 20 00
// 
// @error-msg
// "Error 20 "test
//
// It should read input until encountering a 'q' where it will print to stdout the message
// 'Hello, <input received until 'q'>' and then on stderr the message 'Error test'
    let prog = vec![
        0xa0, 0x01, 0x0c, 0x80, 0x10, 0x37, 0x80, 0x00, 0x80,
        0x00, 0x11, 0x00, 0x80, 0x12, 0x16, 0x06, 0x80, 0x71,
        0x08, 0xa0, 0x01, 0x26, 0x2d, 0x80, 0x01, 0x80, 0x00,
        0x10, 0x18, 0x11, 0x80, 0x00, 0x10, 0x01, 0x80, 0x00,
        0x11, 0x00, 0x02, 0x80, 0x00, 0x80, 0x3a, 0x13, 0xa0,
        0x01, 0x67, 0x94, 0x80, 0x18, 0x17, 0x21, 0x94, 0x80,
        0xf7, 0x0d, 0x22, 0x80, 0x01, 0x80, 0x28, 0x12, 0x18,
        0x10, 0x80, 0x18, 0x17, 0x80, 0x20, 0x12, 0x01, 0x80,
        0x1c, 0x13, 0x80, 0x19, 0x12, 0x80, 0x00, 0x10, 0x09,
        0x80, 0xe5, 0x0d, 0xa0, 0x01, 0x6f, 0x94, 0x80, 0x19,
        0x17, 0x21, 0x94, 0x80, 0xf7, 0x0d, 0x22, 0xa0, 0x01,
        0x0f, 0x17, 0x00, 0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
        0x2c, 0x20, 0x00, 0x45, 0x72, 0x72, 0x6f, 0x72, 0x20,
        0x74, 0x65, 0x73, 0x74,];

    let tmp_file_name = format!("console_test{}", Uuid::new_v4());
    let mut tmp_file_path = std::env::temp_dir();
    tmp_file_path.push(tmp_file_name);

    fs::write(&tmp_file_path, &prog).expect("Failed to write test program");

    let cli_options = uxnclilib::Cli{rom: tmp_file_path, input: "first".to_string()};
    let mut stdout_output = Vec::new();
    let stdin_input = Cursor::new(" secondq");
    let mut stderr_output = Vec::new();
    let mut debug_output = Vec::new();
    let config = uxnclilib::Config{
        stdout_writer: &mut stdout_output,
        stdin_reader: stdin_input,
        stderr_writer: &mut stderr_output,
        debug_writer: &mut debug_output};

    uxnclilib::run(cli_options, config).expect("Failed to execute test program");
    
    // the debug output should be printed to the debug_writer and should give the contents of the working
    // stack followed by the contents of the return stack
    assert_eq!(String::from_utf8(stdout_output).unwrap(), "Hello, first second");
    assert_eq!(String::from_utf8(stderr_output).unwrap(), "Error test");
}

// test datetime device, printing out datetime and ensuring it is correct
#[test]
fn datetime_test() {

// This program is the compiled result of the following:
//
// %RTN { JMP2r }
// %HALT { #010f DEO }
// %EMIT { .Console/write DEO }
// 
// ( devices )
// |10 @Console [ &vector $2 &read $1 &pad $5 &write $1 &error $1 ]
// |c0 @Datetime   [ &year   $2 &month    $1 &day    $1 &hour  $1 &minute $1 &second $1 &dotw $1 &doty $2 &isdst $1 ]
// 
// |100
// ( read day )
// .Datetime/day DEI
// ;byte-to-console JSR2
// 
// LIT '/ EMIT
// 
// ( read month )
// .Datetime/month DEI #01 ADD
// ;byte-to-console JSR2
// 
// LIT '/ .Console/write DEO
// 
// ( read year )
// .Datetime/year DEI2
// ;short-to-console JSR2
// 
// HALT
// BRK
// 
// @byte-to-console ( byte -- )
// 
//   #0a SWP DUP 
// 
//   &start-loop
//   #0a DIV #00 EQU ,&exit-loop JCN 
// 
//     DUP DUP #0a DIV #0a MUL SUB
//     SWP #0a DIV DUP
//     ,&start-loop JMP
//   &exit-loop
// 
//   &start-loop2
//   DUP #0a EQU ,&exit-loop2 JCN 
//     #30 ADD EMIT
//     ,&start-loop2 JMP
//   &exit-loop2
//   POP
// 
// RTN
//
// 
// @short-to-console ( short -- )
// 
//   #000a SWP2 DUP2 
// 
//   &start-loop
//   #000a DIV2 #0000 EQU2 ,&exit-loop JCN 
// 
//     DUP2 DUP2 #000a DIV2 #000a MUL2 SUB2
//     SWP2 #000a DIV2 DUP2
//     ,&start-loop JMP
//   &exit-loop
// 
//   &start-loop2
//   DUP2 #000a EQU2 ,&exit-loop2 JCN 
//     #30 ADD EMIT POP
//     ,&start-loop2 JMP
//   &exit-loop2
//   POP2
// 
// RTN
// 
// It should print today's date to console as d/m/yyyy
    let prog = vec![
        0x80, 0xc3, 0x16, 0xa0, 0x01, 0x27, 0x2e, 0x80, 0x2f,
        0x80, 0x18, 0x17, 0x80, 0xc2, 0x16, 0x80, 0x01, 0x18,
        0xa0, 0x01, 0x27, 0x2e, 0x80, 0x2f, 0x80, 0x18, 0x17,
        0x80, 0xc0, 0x36, 0xa0, 0x01, 0x57, 0x2e, 0xa0, 0x01,
        0x0f, 0x17, 0x00, 0x80, 0x0a, 0x04, 0x06, 0x80, 0x0a,
        0x1b, 0x80, 0x00, 0x08, 0x80, 0x11, 0x0d, 0x06, 0x06,
        0x80, 0x0a, 0x1b, 0x80, 0x0a, 0x1a, 0x19, 0x04, 0x80,
        0x0a, 0x1b, 0x06, 0x80, 0xe6, 0x0c, 0x06, 0x80, 0x0a,
        0x08, 0x80, 0x09, 0x0d, 0x80, 0x30, 0x18, 0x80, 0x18,
        0x17, 0x80, 0xf0, 0x0c, 0x02, 0x6c, 0xa0, 0x00, 0x0a,
        0x24, 0x26, 0xa0, 0x00, 0x0a, 0x3b, 0xa0, 0x00, 0x00,
        0x28, 0x80, 0x14, 0x0d, 0x26, 0x26, 0xa0, 0x00, 0x0a,
        0x3b, 0xa0, 0x00, 0x0a, 0x3a, 0x39, 0x24, 0xa0, 0x00,
        0x0a, 0x3b, 0x26, 0x80, 0xe1, 0x0c, 0x26, 0xa0, 0x00,
        0x0a, 0x28, 0x80, 0x0a, 0x0d, 0x80, 0x30, 0x18, 0x80,
        0x18, 0x17, 0x02, 0x80, 0xee, 0x0c, 0x22, 0x6c,];

    let tmp_file_name = format!("datetime_test{}", Uuid::new_v4());
    let mut tmp_file_path = std::env::temp_dir();
    tmp_file_path.push(tmp_file_name);

    fs::write(&tmp_file_path, &prog).expect("Failed to write test program");

    let cli_options = uxnclilib::Cli{rom: tmp_file_path, input: "".to_string()};
    let mut stdout_output = Vec::new();
    let stdin_input = Cursor::new("");
    let mut stderr_output = Vec::new();
    let mut debug_output = Vec::new();
    let config = uxnclilib::Config{
        stdout_writer: &mut stdout_output,
        stdin_reader: stdin_input,
        stderr_writer: &mut stderr_output,
        debug_writer: &mut debug_output};

    uxnclilib::run(cli_options, config).expect("Failed to execute test program");
    let today = Local::today();
    let expected = format!("{}/{}/{}", today.day(), today.month(), today.year());
    
    assert_eq!(String::from_utf8(stdout_output).unwrap(), expected);
}
