use std::str::FromStr;
use std::fmt;
use crate::uxnasmlib::asm::prog_state::ProgState;
use crate::uxnasmlib::asm::prog_state::Label;

pub mod ops {
    use std::str::FromStr;

    #[derive(Debug, PartialEq)]
    pub enum OpCode {
        Brk,
        Deo,
    }

    #[derive(Debug, PartialEq)]
    pub struct OpObject {
        keep: bool,
        ret: bool,
        short: bool,
        op_code: OpCode,
    }

    impl OpObject {
        pub fn get_bytes(&self) -> Vec<u8> {
            let byte = match self.op_code {
                OpCode::Brk => 0x00,
                OpCode::Deo => 0x17,
            };

            let byte = if self.keep { byte | 0b10000000 } else { byte };

            let byte = if self.ret { byte | 0b01000000 } else { byte };

            let byte = if self.short { byte | 0b00100000 } else { byte };

            return vec![byte];
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct ParseOpObjectError {}

    impl FromStr for OpObject {
        type Err = ParseOpObjectError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() < 3 {
                return Err(ParseOpObjectError {});
            }

            let opcode = s.get(0..3);
            if opcode.is_none() {
                return Err(ParseOpObjectError {});
            }

            let opcode = opcode.unwrap();

            let mut ret = match opcode {
                "BRK" => {
                    if s.len() > 3 {
                        return Err(ParseOpObjectError {});
                    }

                    OpObject {
                    keep: false,
                    ret: false,
                    short: false,
                    op_code: OpCode::Brk,
                    }
                },
                "LIT" => OpObject {
                    keep: true,
                    ret: false,
                    short: false,
                    op_code: OpCode::Brk,
                },
                "DEO" => OpObject {
                    keep: false,
                    ret: false,
                    short: false,
                    op_code: OpCode::Deo,
                },
                _ => return Err(ParseOpObjectError {}),
            };

            for mode_flag in s.chars().skip(3) {
                match mode_flag {
                    '2' => {
                        ret.short = true;
                    },
                    'k' => {
                        if ret.op_code == OpCode::Brk {
                            return Err(ParseOpObjectError {});
                        }

                        ret.keep = true;
                    },
                    'r' => {
                        ret.ret = true;
                    },
                    _ => return Err(ParseOpObjectError {}),
                };
            }

            return Ok(ret);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    
        // test `get_bytes` function; for each possible op code,
        // verify that the correct sequence of bytes is produced for it
        #[test]
        fn test_get_bytes_happy() {
            let inputs = [
                (OpCode::Brk, vec![0x00,]),
                (OpCode::Deo, vec![0x17,]),
            ];

            for (input, expected_output) in inputs.into_iter() {
                let input = OpObject{
                    keep: false,
                    ret: false,
                    short: false,
                    op_code: input
                };

                let output = input.get_bytes();
                assert_eq!(output, expected_output);
            }
        }

        // test `get_bytes` function; verify that a selection
        // of modifier flags leads to the correct output
        #[test]
        fn test_get_bytes_happy_with_modifiers() {
            let input = OpObject{
                keep: true,
                ret: false,
                short: true,
                op_code: OpCode::Deo,
            };
            let expected_output = vec![0xb7,];
            let output = input.get_bytes();

            assert_eq!(output, expected_output);

            let input = OpObject{
                keep: true,
                ret: true,
                short: false,
                op_code: OpCode::Deo,
            };
            let expected_output = vec![0xd7,];
            let output = input.get_bytes();

            assert_eq!(output, expected_output);
        }

        // test `from_str` function for operation
        // strings with no modifier flags
        #[test]
        fn test_from_str_happy() {
            let inputs = [
                ("BRK", OpCode::Brk),
                ("DEO", OpCode::Deo),
            ];

            for (input, expected_output) in inputs {
                let output = input.parse::<OpObject>();
                let expected_output = Ok(OpObject{
                    keep: false,
                    ret: false,
                    short: false,
                    op_code: expected_output,
                });

                assert_eq!(output, expected_output);
            }
        }

        // test `from_str` function for LIT operation
        // string with no modifier flags
        #[test]
        fn test_from_str_happy_lit() {
            let output = "LIT".parse::<OpObject>();
            let expected_output = Ok(OpObject{
                keep: true,
                ret: false,
                short: false,
                op_code: OpCode::Brk,
            });

            assert_eq!(output, expected_output);
        }

        #[test]
        fn test_from_str_happy_mode_flags() {
            let input = "DEO2rk";
            let expected_output = Ok(OpObject{
                keep: true,
                ret: true,
                short: true,
                op_code: OpCode::Deo,
            });

            let output = input.parse::<OpObject>();
            assert_eq!(output, expected_output);

            let input = "DEOkr2";
            let expected_output = Ok(OpObject{
                keep: true,
                ret: true,
                short: true,
                op_code: OpCode::Deo,
            });

            let output = input.parse::<OpObject>();
            assert_eq!(output, expected_output);

            let input = "DEOr2";
            let expected_output = Ok(OpObject{
                keep: false,
                ret: true,
                short: true,
                op_code: OpCode::Deo,
            });

            let output = input.parse::<OpObject>();
            assert_eq!(output, expected_output);
        }

        #[test]
        fn test_from_str_forbidden_mode_flags() {
            let inputs = ["BRKr", "BRKk", "BRK2", "LITk"];

            for input in inputs {
                let output = input.parse::<OpObject>();
                assert_eq!(output, Err(ParseOpObjectError {}));
            }
        }

        #[test]
        fn test_from_str_unrecognised_op_string() {
            let inputs = [
                "BRKK",
                "BOK",
                "BK",
            ];

            for input in inputs {
                let output = input.parse::<OpObject>();
                assert_eq!(output, Err(ParseOpObjectError{}));
            }
        }
    }
}

use ops::OpObject;

#[derive(Debug, PartialEq)]
pub enum LabelRef {
    Label { label_name: String },
    FullSubLabel { label_name: String, sub_label_name: String },
    SubLabel { sub_label_name: String },
}

impl FromStr for LabelRef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c1 = s.get(0..1);

        // attempt to parse as partial sub-label
        if let Some("&") = c1 {
            return Ok(LabelRef::SubLabel{
                sub_label_name: s[1..].to_owned()});
        }

        // attempt to parse as full sub-label
        if let Some((label_name, sub_label_name)) = s.split_once('/') {
            let label_name = label_name.to_owned();
            let sub_label_name = sub_label_name.to_owned();

            return Ok(LabelRef::FullSubLabel{
                label_name,
                sub_label_name
            });
        }

        return Ok(LabelRef::Label{label_name: s.to_owned()});
    }
}

#[derive(Debug, PartialEq)]
pub enum UxnToken {
    Op(OpObject),
    // MacroDefine,
    PadAbs(u16),
    PadRel(u16),
    LabelDefine(String),
    SubLabelDefine(String),
    // Include,
    LitByte(u8),
    LitShort(u16),
    // LitAddressZeroPage(String),
    // LitAddressRel(String),
    // LitAddressAbs(String),
    RawAbsAddr(LabelRef),
    MacroInvocation(String),
    RawByte(u8),
    RawShort(u16),
    // RawWord(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub enum GetBytesError {
    UndefinedLabel { label_name: String },
    UndefinedSubLabel { label_name: String, sub_label_name: String },
}

impl UxnToken {
    pub fn get_bytes(&self, prog_state: &ProgState) -> Result<Vec<u8>, GetBytesError> {
        match self {
            UxnToken::Op(o) => return Ok(o.get_bytes()),
            UxnToken::MacroInvocation(_) => return Ok(vec![0xaa, 0xbb]),
            UxnToken::PadAbs(n) => {
                let bytes_to_write = *n - prog_state.counter;

                return Ok(vec![0x00; bytes_to_write.into()]);
            }
            UxnToken::PadRel(n) => return Ok(vec![0x00; (*n).into()]),
            UxnToken::RawByte(b) => return Ok(vec![*b]),
            UxnToken::RawShort(s) => {
                let bytes = s.to_be_bytes();
                return Ok(vec![bytes[0], bytes[1]]);
            },
            UxnToken::LitByte(b) => return Ok(vec![0x80, *b]),
            UxnToken::LitShort(s) => {
                let bytes = s.to_be_bytes();
                return Ok(vec![0xA0, bytes[0], bytes[1]]);
            }
            UxnToken::LabelDefine(_) => return Ok(vec![]),
            UxnToken::SubLabelDefine(_) => return Ok(vec![]),
            UxnToken::RawAbsAddr(label_ref) => {

                match label_ref {
                    LabelRef::Label{label_name} => {
                        if let Some(label) = prog_state.labels.get(label_name) {
                            let bytes = label.address.to_be_bytes();
                            return Ok(vec![bytes[0], bytes[1]]);
                        } else {
                            return Err(GetBytesError::UndefinedLabel {
                                label_name: label_name.clone(),
                            });
                        }
                    },
                    LabelRef::SubLabel{sub_label_name} => {
                        let label_name = &prog_state.current_label;

                        if let Some(label) = prog_state.labels.get(label_name) {
                            if let Some(sub_label) = label.sub_labels.get(sub_label_name) {
                                let bytes = sub_label.to_be_bytes();
                                return Ok(vec![bytes[0], bytes[1]]);
                            }   
                        }

                        return Err(GetBytesError::UndefinedSubLabel {
                            label_name: label_name.clone(),
                            sub_label_name: sub_label_name.clone(),
                        });
                    },
                    LabelRef::FullSubLabel{label_name: label_name, sub_label_name} => {
                        if let Some(label) = prog_state.labels.get(label_name) {
                            if let Some(sub_label) = label.sub_labels.get(sub_label_name) {
                                let bytes = sub_label.to_be_bytes();
                                return Ok(vec![bytes[0], bytes[1]]);
                            }   
                        }

                        return Err(GetBytesError::UndefinedSubLabel {
                            label_name: label_name.clone(),
                            sub_label_name: sub_label_name.clone(),
                        });
                    },
                }

            }
        }
    }

    pub fn num_bytes(&self, prog_counter: u16) -> u16 {
        match self {
            UxnToken::Op(_) => return 0x1,
            UxnToken::MacroInvocation(_) => return 0xff,
            UxnToken::PadAbs(n) => return *n - prog_counter,
            UxnToken::PadRel(n) => return *n,
            UxnToken::RawByte(_) => return 0x1,
            UxnToken::RawShort(_) => return 0x2,
            UxnToken::LitByte(_) => return 0x2,
            UxnToken::LitShort(_) => return 0x3,
            UxnToken::LabelDefine(_) => return 0x0,
            UxnToken::SubLabelDefine(_) => return 0x0,
            UxnToken::RawAbsAddr(_) => return 0x2,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    RuneAbsentArg { rune: String },
    RuneInvalidArg { rune: String, supplied_arg: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::RuneAbsentArg{rune} => {
                write!(f, "'{}' missing argument", rune)
            },
            ParseError::RuneInvalidArg{rune, supplied_arg} => {
                write!(f, "'{}' has invalid argument '{}'", rune, supplied_arg)
            },
        }
    }
}

impl FromStr for UxnToken {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(op) = s.parse::<OpObject>() {
            return Ok(UxnToken::Op(op));
        }

        if s.len() == 2 {
            if let Ok(raw) = u8::from_str_radix(s, 16) {
                return Ok(UxnToken::RawByte(raw));
            }
        }

        if s.len() == 4 {
            if let Ok(raw) = u16::from_str_radix(s, 16) {
                return Ok(UxnToken::RawShort(raw));
            }
        }

        if &s[0..1] == "|" {
            if s.len() < 2 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "|".to_owned(),
                });
            }

            if let Ok(pad_val) = u16::from_str_radix(&s[1..], 16) {
                return Ok(UxnToken::PadAbs(pad_val));
            }

            return Err(ParseError::RuneInvalidArg {
                rune: "|".to_owned(),
                supplied_arg: s[1..].to_owned(),
            });
        }

        if &s[0..1] == "$" {
            if s.len() < 2 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "$".to_owned(),
                });
            }

            if let Ok(pad_val) = u16::from_str_radix(&s[1..], 16) {
                return Ok(UxnToken::PadRel(pad_val));
            }

            return Err(ParseError::RuneInvalidArg {
                rune: "$".to_owned(),
                supplied_arg: s[1..].to_owned(),
            });
        }

        if &s[0..1] == "#" {
            let s = &s[1..];
            match s.len() {
                0 => {
                    return Err(ParseError::RuneAbsentArg {
                        rune: "#".to_owned(),
                    });
                },
                2 => {
                    if let Ok(val) = u8::from_str_radix(s, 16) {
                        return Ok(UxnToken::LitByte(val));
                    } else {
                        return Err(ParseError::RuneInvalidArg {
                            rune: "#".to_owned(),
                            supplied_arg: s.to_owned(),
                        });
                    }
                }
                4 => {
                    if let Ok(val) = u16::from_str_radix(s, 16) {
                        return Ok(UxnToken::LitShort(val));
                    } else {
                        return Err(ParseError::RuneInvalidArg {
                            rune: "#".to_owned(),
                            supplied_arg: s.to_owned(),
                        });
                    }
                }
                _ => {
                    return Err(ParseError::RuneInvalidArg {
                        rune: "#".to_owned(),
                        supplied_arg: s.to_owned(),
                    });
                }
            };
        }

        if &s[0..1] == "'" {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "'".to_owned(),
                });
            }

            if s.len() > 2 {
                return Err(ParseError::RuneInvalidArg {
                    rune: "'".to_owned(),
                    supplied_arg: s[1..].to_owned(),
                });
            }

            let sb = (&s[1..]).as_bytes();

            if sb[0] > 0x7f {
                // not ascii
                return Err(ParseError::RuneInvalidArg {
                    rune: "'".to_owned(),
                    supplied_arg: s[1..].to_owned(),
                });
            }

            return Ok(UxnToken::RawByte(sb[0]));
        }

        if &s[0..1] == "@" {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "@".to_owned(),
                });
            }

            return Ok(UxnToken::LabelDefine((&s[1..]).to_owned()));
        }

        if &s[0..1] == "&" {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "&".to_owned(),
                });
            }

            return Ok(UxnToken::SubLabelDefine((&s[1..]).to_owned()));
        }

        if &s[0..1] == ":" {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: ":".to_owned(),
                });
            }

            let label_ref = s[1..].parse::<LabelRef>().unwrap();

            return Ok(UxnToken::RawAbsAddr(label_ref));
        }

        return Ok(UxnToken::MacroInvocation(s.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;


    // test `from_str()` for `LabelRef`; test that each type of label
    // can be correctly parsed
    #[test]
    fn test_label_ref_from_str() {
        let inputs = [
            ("test_label", LabelRef::Label{label_name: "test_label".to_owned()}),
            ("&sub_label", LabelRef::SubLabel{sub_label_name: "sub_label".to_owned()}),
            ("test_label/sub_label", LabelRef::FullSubLabel{
                label_name: "test_label".to_owned(),
                sub_label_name: "sub_label".to_owned()}),
        ];

        for (input, expected) in inputs.into_iter() {
            let returned = input.parse::<LabelRef>();
            assert_eq!(returned, Ok(expected));
        }
    }

    // test `get_bytes` function; for each possible input token,
    // verify that the correct sequence of bytes is produced for it
    #[test]
    fn test_get_bytes_happy() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x1234));
        labels.insert("test_label2".to_owned(), Label::new(0x1235));
        labels.get_mut("test_label2").unwrap()
            .sub_labels.insert("sub_label".to_owned(), 0x4576);
        labels.get_mut("test_label2").unwrap()
            .sub_labels.insert("sub_label2".to_owned(), 0x9876);

        let prog_state = ProgState{
            counter: 0x0,
            labels: &labels,
            current_label: "test_label2".to_owned()};

        let inputs = [
            (
                UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()),
                vec![0x17],
            ),
            // TODO
            (
                UxnToken::MacroInvocation("test_macro".to_owned()),
                vec![0xaa, 0xbb],
            ),
            (UxnToken::PadAbs(0x100), vec![0x00; 0x100]),
            (UxnToken::PadRel(0x80), vec![0x00; 0x80]),
            (UxnToken::RawByte(0xab), vec![0xab]),
            (UxnToken::RawShort(0xabcd), vec![0xab, 0xcd]),
            (UxnToken::LitByte(0xab), vec![0x80, 0xab]),
            (UxnToken::LitShort(0xabcd), vec![0xA0, 0xab, 0xcd]),
            (UxnToken::LabelDefine("test_label".to_owned()), vec![]),
            (UxnToken::SubLabelDefine("test_sub_label".to_owned()), vec![]),
            (
                UxnToken::RawAbsAddr("test_label".parse::<LabelRef>().unwrap()),
                vec![0x12, 0x34],
            ),
            (
                UxnToken::RawAbsAddr("test_label2/sub_label".parse::<LabelRef>().unwrap()),
                vec![0x45, 0x76],
            ),
            (
                UxnToken::RawAbsAddr("&sub_label2".parse::<LabelRef>().unwrap()),
                vec![0x98, 0x76],
            ),
        ];

        for (token, expected) in inputs.into_iter() {
            let returned = token.get_bytes(&prog_state);
            assert_eq!(returned, Ok(expected));
        }
    }

    // test `get_bytes` function with a label that hasn't been defined
    #[test]
    fn test_get_bytes_unrecognised_label() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x1234));

        let input = UxnToken::RawAbsAddr("test_label_xyz".parse::<LabelRef>().unwrap());
        let output = input.get_bytes(&ProgState{
            counter: 0,
            labels: &labels,
            current_label: "".to_owned()});

        assert_eq!(output, Err(
                GetBytesError::UndefinedLabel {
                    label_name: "test_label_xyz".to_owned(),
                }));
    }

    // test `get_bytes` function with a full sub-label that hasn't been defined
    #[test]
    fn test_get_bytes_unrecognised_sub_label_full() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x1234));
        labels.get_mut("test_label").unwrap().sub_labels
            .insert("sub_label".to_owned(), 0x4576);

        let input = UxnToken::RawAbsAddr(
            "test_label/sub_label_xyz".parse::<LabelRef>().unwrap());
        let output = input.get_bytes(&ProgState{
            counter: 0,
            labels: &labels,
            current_label: "".to_owned()});

        assert_eq!(output, Err(
                GetBytesError::UndefinedSubLabel {
                    label_name: "test_label".to_owned(),
                    sub_label_name: "sub_label_xyz".to_owned(),
                }));
    }

    // test `get_bytes` function with a partial sub-label that hasn't been defined
    #[test]
    fn test_get_bytes_unrecognised_sub_label_partial() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x1234));
        labels.get_mut("test_label").unwrap().sub_labels
            .insert("sub_label".to_owned(), 0x4576);
        labels.insert("test_label2".to_owned(), Label::new(0x1234));

        let input = UxnToken::RawAbsAddr(
            "&sub_label".parse::<LabelRef>().unwrap());
        let output = input.get_bytes(&ProgState{
            counter: 0,
            labels: &labels,
            current_label: "test_label2".to_owned()});

        assert_eq!(output, Err(
                GetBytesError::UndefinedSubLabel {
                    label_name: "test_label2".to_owned(),
                    sub_label_name: "sub_label".to_owned(),
                }));
    }

    #[test]
    fn test_num_bytes_happy() {
        let prog_counter = 0;

        let inputs = [
            (UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()), 0x1),
            // TODO
            (UxnToken::MacroInvocation("blah".to_owned()), 0xff),
            (UxnToken::PadAbs(0x1ff), 0x1ff),
            (UxnToken::PadRel(0x1fe), 0x1fe),
            (UxnToken::RawByte(0xfe), 0x1),
            (UxnToken::RawShort(0xabcd), 0x2),
            (UxnToken::LitByte(0xab), 0x2),
            (UxnToken::LitShort(0xabcd), 0x3),
            (UxnToken::LabelDefine("test_label".to_owned()), 0x0),
            (UxnToken::SubLabelDefine("test_sub_label".to_owned()), 0x0),
            (UxnToken::RawAbsAddr("test_label".parse::<LabelRef>().unwrap()), 0x2),
        ];

        for (token, expected) in inputs.into_iter() {
            let returned = token.num_bytes(prog_counter);
            assert_eq!(returned, expected);
        }
    }

    // test get_bytes function when the program counter is not 0
    #[test]
    fn test_get_bytes_prog_counter() {
        let prog_state = ProgState{
            counter: 0x70,
            labels: &HashMap::new(),
            current_label: "".to_owned()};
        let token = UxnToken::PadAbs(0x100);
        let returned = token.get_bytes(&prog_state);
        assert_eq!(returned, Ok(vec![0x0; 0x90]));
    }

    // test get_bytes function when the program counter is not 0
    // but the absolute padding is behind the program counter
    #[test]
    #[should_panic]
    fn test_get_bytes_prog_counter_fail() {
        let prog_state = ProgState{
            counter: 0x170,
            labels: &HashMap::new(),
            current_label: "".to_owned()};
        let token = UxnToken::PadAbs(0x100);
        let _ = token.get_bytes(&prog_state);
    }

    // test num_bytes function when the program counter is not 0
    #[test]
    fn test_num_bytes_prog_counter() {
        let prog_counter = 0x70;
        let token = UxnToken::PadAbs(0x100);
        let returned = token.num_bytes(prog_counter);
        assert_eq!(returned, 0x90);
    }

    // TODO need to return error test num_bytes function when the program counter is not 0 but the absolute padding is behind the program counter
    #[test]
    #[should_panic]
    fn test_num_bytes_prog_counter_fail() {
        let prog_counter = 0x170;
        let token = UxnToken::PadAbs(0x100);
        token.num_bytes(prog_counter);
    }

    // test from_str for UxnToken with an input that should be parsed as a raw byte
    #[test]
    fn test_from_str_raw_byte() {
        let input = "ab";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::RawByte(0xab);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as absolute padding 
    #[test]
    fn test_from_str_pad_abs() {
        let input = "|abcd";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::PadAbs(0xabcd);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as relative padding 
    #[test]
    fn test_from_str_pad_rel() {
        let input = "$abcd";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::PadRel(0xabcd);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as literal bytes/shorts 
    #[test]
    fn test_from_str_lit_byte_short() {
        let input = "#cd";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::LitByte(0xcd);
        assert_eq!(output, Ok(expected));

        let input = "#abcd";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::LitShort(0xabcd);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed from a character 
    #[test]
    fn test_from_str_char() {
        let input = "'X";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::RawByte(0x58);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a label define
    #[test]
    fn test_from_str_label_define() {
        let input = "@test_label";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::LabelDefine("test_label".to_owned());
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a sub-label define
    #[test]
    fn test_from_str_sub_label_define() {
        let input = "&test_sub_label";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::SubLabelDefine("test_sub_label".to_owned());
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a raw absolute address
    #[test]
    fn test_from_str_raw_abs_address() {
        let input = ":test_label";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::RawAbsAddr("test_label".parse::<LabelRef>().unwrap());
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should trigger an error because a rune has no
    // argument attached
    #[test]
    fn test_from_str_rune_absent() {
        let inputs = ["|", "$", "#", "'", "@", ":"];

        for input in inputs {
            let output = input.parse::<UxnToken>();
            let expected = Err(ParseError::RuneAbsentArg{
                rune: input.to_owned()});

            assert_eq!(output, expected);
        }
    }

    // test from_str for UxnToken with an input that should trigger an error because a rune has
    // invalid argument attached
    #[test]
    fn test_from_str_rune_invalid_arg() {
        let inputs = ["|zz", "$uu", "#abcdd",
        "#ax", "#abxd", "'aa", "'€", ];

        for input in inputs {
            let output = input.parse::<UxnToken>();
            let expected = Err(ParseError::RuneInvalidArg{
                rune: input[0..1].to_owned(),
                supplied_arg: input[1..].to_owned()});

            assert_eq!(output, expected);
        }
    }
}
