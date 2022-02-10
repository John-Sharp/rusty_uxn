use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;

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

    #[derive(Debug)]
    pub struct ParseOpObjectError {}

    impl FromStr for OpObject {
        type Err = ParseOpObjectError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() < 3 {
                return Err(ParseOpObjectError {});
            }

            let ret = match &s[0..3] {
                "BRK" => OpObject {
                    keep: false,
                    ret: false,
                    short: false,
                    op_code: OpCode::Brk,
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

            // TODO parse the mode flags

            return Ok(ret);
        }
    }
}

use ops::OpObject;

#[derive(Debug, PartialEq)]
pub enum UxnToken {
    Op(OpObject),
    MacroInvocation(String),
    PadAbs(u16),
    PadRel(u16),
    RawByte(u8),
    RawShort(u16),
    LitByte(u8),
    LitShort(u16),
    LabelDefine(String),
    RawAbsAddr(String),
}

impl UxnToken {
    pub fn get_bytes(&self, prog_counter: u16, labels: &HashMap<String, u16>) -> Vec<u8> {
        match self {
            UxnToken::Op(o) => return o.get_bytes(),
            UxnToken::MacroInvocation(_) => return vec![0xaa, 0xbb],
            UxnToken::PadAbs(n) => {
                let bytes_to_write = *n - prog_counter;

                return vec![0x00; bytes_to_write.into()];
            }
            UxnToken::PadRel(n) => return vec![0x00; (*n).into()],
            UxnToken::RawByte(b) => return vec![*b],
            UxnToken::RawShort(_) => return vec![0xdd],
            UxnToken::LitByte(b) => return vec![0x80, *b],
            UxnToken::LitShort(s) => {
                let bytes = s.to_be_bytes();
                return vec![0xA0, bytes[0], bytes[1]];
            }
            UxnToken::LabelDefine(_) => return vec![],
            UxnToken::RawAbsAddr(label) => {
                println!("label is {}", label);
                if let Some(addr) = labels.get(label) {
                    let bytes = addr.to_be_bytes();
                    return vec![bytes[0], bytes[1]];
                } else {
                    panic!();
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
            UxnToken::RawAbsAddr(_) => return 0x2,
        }
    }
}

impl FromStr for UxnToken {
    type Err = Infallible;

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
                // TODO replace these with parse errors
                panic!();
            }

            if let Ok(pad_val) = u16::from_str_radix(&s[1..], 16) {
                return Ok(UxnToken::PadAbs(pad_val));
            }
        }

        if &s[0..1] == "$" {
            if s.len() < 2 {
                // TODO replace these with parse errors
                panic!();
            }

            if let Ok(pad_val) = u16::from_str_radix(&s[1..], 16) {
                return Ok(UxnToken::PadRel(pad_val));
            }
        }

        if &s[0..1] == "#" {
            let s = &s[1..];
            match s.len() {
                2 => {
                    if let Ok(val) = u8::from_str_radix(s, 16) {
                        return Ok(UxnToken::LitByte(val));
                    } else {
                        panic!();
                    }
                }
                4 => {
                    if let Ok(val) = u16::from_str_radix(s, 16) {
                        return Ok(UxnToken::LitShort(val));
                    } else {
                        panic!();
                    }
                }
                _ => {
                    panic!();
                }
            };
        }

        if &s[0..1] == "'" {
            if s.len() > 2 {
                panic!();
            }

            let s = (&s[1..]).as_bytes();

            if s[0] > 0x7f {
                // not ascii
                panic!();
            }

            return Ok(UxnToken::RawByte(s[0]));
        }

        if &s[0..1] == "@" {
            if s.len() == 1 {
                // label with no name
                panic!();
            }

            return Ok(UxnToken::LabelDefine((&s[1..]).to_owned()));
        }

        if &s[0..1] == ":" {
            if s.len() == 1 {
                // label with no name
                panic!();
            }

            return Ok(UxnToken::RawAbsAddr((&s[1..]).to_owned()));
        }

        return Ok(UxnToken::MacroInvocation(s.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test `get_bytes` function; for each possible input token,
    // verify that the correct sequence of bytes is produced for it
    #[test]
    fn test_get_bytes_happy() {
        let prog_counter = 0x0;
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), 0x1234);

        let inputs = [
            (UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()),
            vec![0x17,]),

            // TODO
            (UxnToken::MacroInvocation("test_macro".to_owned()),
            vec![0xaa, 0xbb]),

            (UxnToken::PadAbs(0x100),
             vec![0x00; 0x100]),

            (UxnToken::PadRel(0x80),
             vec![0x00; 0x80]),

            (UxnToken::RawByte(0xab),
             vec![0xab]),

            // TODO
            (UxnToken::RawShort(0xabab),
             vec![0xdd]),

            (UxnToken::LitByte(0xab),
             vec![0x80, 0xab]),

            (UxnToken::LitShort(0xabcd),
             vec![0xA0, 0xab, 0xcd]),

            (UxnToken::LabelDefine("test_label".to_owned()),
             vec![]),

            (UxnToken::RawAbsAddr("test_label".to_owned()),
             vec![0x12, 0x34]),
        ];

        for (token, expected) in inputs.into_iter() {
            let returned = token.get_bytes(prog_counter, &labels);
            assert_eq!(returned, expected);
        }
    }

    // TODO should have better error
    // test `get_bytes` function with a label that hasn't been defined
    #[test]
    #[should_panic]
    fn test_get_bytes_unrecognised_label() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), 0x1234);

        let input = UxnToken::RawAbsAddr("test_label_xyz".to_owned());
        input.get_bytes(0, &labels);
    }

    #[test]
    fn test_num_bytes_happy() {
        let prog_counter = 0;

        let inputs = [
            (UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()),
             0x1),

            // TODO
            (UxnToken::MacroInvocation("blah".to_owned()),
             0xff),

            (UxnToken::PadAbs(0x1ff),
             0x1ff),

            (UxnToken::PadRel(0x1fe),
             0x1fe),

            (UxnToken::RawByte(0xfe),
             0x1),

            (UxnToken::RawShort(0xabcd),
             0x2),

            (UxnToken::LitByte(0xab),
             0x2),

            (UxnToken::LitShort(0xabcd),
             0x3),

            (UxnToken::LabelDefine("test_label".to_owned()),
             0x0),

            (UxnToken::RawAbsAddr("test_label".to_owned()),
             0x2),
        ];

        for (token, expected) in inputs.into_iter() {
            let returned = token.num_bytes(prog_counter);
            assert_eq!(returned, expected);
        }
    }
}
