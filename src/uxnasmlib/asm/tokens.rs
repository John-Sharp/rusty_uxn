use crate::uxnasmlib::asm::prog_state::ProgState;
use std::fmt;
use std::str::FromStr;

use crate::ops::OpObject;

#[derive(Debug, PartialEq, Clone)]
pub enum LabelRef {
    Label {
        label_name: String,
    },
    FullSubLabel {
        label_name: String,
        sub_label_name: String,
    },
    SubLabel {
        sub_label_name: String,
    },
}

impl FromStr for LabelRef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c1 = s.get(0..1);

        // attempt to parse as partial sub-label
        if let Some("&") = c1 {
            return Ok(LabelRef::SubLabel {
                sub_label_name: s[1..].to_owned(),
            });
        }

        // attempt to parse as full sub-label
        if let Some((label_name, sub_label_name)) = s.split_once('/') {
            let label_name = label_name.to_owned();
            let sub_label_name = sub_label_name.to_owned();

            return Ok(LabelRef::FullSubLabel {
                label_name,
                sub_label_name,
            });
        }

        return Ok(LabelRef::Label {
            label_name: s.to_owned(),
        });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UxnToken {
    Op(OpObject),
    MacroDefine(String),
    MacroStartDelimiter,
    MacroEndDelimiter,
    PadAbs(u16),
    PadRel(u16),
    LabelDefine(String),
    SubLabelDefine(String),
    // Include,
    LitByte(u8),
    LitShort(u16),
    LitAddressZeroPage(LabelRef),
    LitAddressRel(LabelRef),
    LitAddressAbs(LabelRef),
    RawAbsAddr(LabelRef),
    MacroInvocation(String),
    RawByte(u8),
    RawShort(u16),
    RawWord(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub enum GetBytesError {
    UndefinedLabel {
        label_name: String,
    },
    UndefinedSubLabel {
        label_name: String,
        sub_label_name: String,
    },
    LabelNotInZeroPage {
        label_name: String,
    },
    SubLabelNotInZeroPage {
        label_name: String,
        sub_label_name: String,
    },
    RelLabelNotInRange {
        label_name: String,
    },
    RelSubLabelNotInRange {
        label_name: String,
        sub_label_name: String,
    },
    NotWritableToken,
}

fn get_address_of_label(
    label_ref: &LabelRef,
    prog_state: &ProgState,
) -> Result<u16, GetBytesError> {
    match label_ref {
        LabelRef::Label { label_name } => {
            if let Some(label) = prog_state.labels.get(label_name) {
                return Ok(label.address);
            } else {
                return Err(GetBytesError::UndefinedLabel {
                    label_name: label_name.clone(),
                });
            }
        }
        LabelRef::SubLabel { sub_label_name } => {
            let label_name = &prog_state.current_label;

            if let Some(label) = prog_state.labels.get(label_name) {
                if let Some(sub_label) = label.sub_labels.get(sub_label_name) {
                    return Ok(*sub_label);
                }
            }

            return Err(GetBytesError::UndefinedSubLabel {
                label_name: label_name.clone(),
                sub_label_name: sub_label_name.clone(),
            });
        }
        LabelRef::FullSubLabel {
            label_name,
            sub_label_name,
        } => {
            if let Some(label) = prog_state.labels.get(label_name) {
                if let Some(sub_label) = label.sub_labels.get(sub_label_name) {
                    return Ok(*sub_label);
                }
            }

            return Err(GetBytesError::UndefinedSubLabel {
                label_name: label_name.clone(),
                sub_label_name: sub_label_name.clone(),
            });
        }
    }
}

impl UxnToken {
    pub fn get_bytes(&self, prog_state: &ProgState) -> Result<Vec<u8>, GetBytesError> {
        match self {
            UxnToken::Op(o) => return Ok(o.get_bytes()),
            UxnToken::MacroDefine(_) => return Err(GetBytesError::NotWritableToken),
            UxnToken::MacroStartDelimiter => return Err(GetBytesError::NotWritableToken),
            UxnToken::MacroEndDelimiter => return Err(GetBytesError::NotWritableToken),
            UxnToken::MacroInvocation(_) => return Err(GetBytesError::NotWritableToken),
            UxnToken::PadAbs(_) => return Err(GetBytesError::NotWritableToken),
            UxnToken::PadRel(_) => return Err(GetBytesError::NotWritableToken),
            UxnToken::RawByte(b) => return Ok(vec![*b]),
            UxnToken::RawShort(s) => {
                let bytes = s.to_be_bytes();
                return Ok(vec![bytes[0], bytes[1]]);
            }
            UxnToken::RawWord(w) => {
                return Ok(w.clone());
            }
            UxnToken::LitByte(b) => return Ok(vec![0x80, *b]),
            UxnToken::LitShort(s) => {
                let bytes = s.to_be_bytes();
                return Ok(vec![0xA0, bytes[0], bytes[1]]);
            }
            UxnToken::LitAddressZeroPage(label_ref) => {
                let address = get_address_of_label(label_ref, prog_state)?;
                if address > 0xff {
                    // not in zero-page
                    match label_ref {
                        LabelRef::Label { label_name } => {
                            return Err(GetBytesError::LabelNotInZeroPage {
                                label_name: label_name.clone(),
                            });
                        }
                        LabelRef::SubLabel { sub_label_name } => {
                            let label_name = &prog_state.current_label;
                            return Err(GetBytesError::SubLabelNotInZeroPage {
                                label_name: label_name.clone(),
                                sub_label_name: sub_label_name.clone(),
                            });
                        }
                        LabelRef::FullSubLabel {
                            label_name,
                            sub_label_name,
                        } => {
                            return Err(GetBytesError::SubLabelNotInZeroPage {
                                label_name: label_name.clone(),
                                sub_label_name: sub_label_name.clone(),
                            });
                        }
                    }
                }

                let bytes = address.to_be_bytes();
                return Ok(vec![0x80, bytes[1]]);
            }
            UxnToken::LitAddressRel(label_ref) => {
                let address = get_address_of_label(label_ref, prog_state)?;
                let address: i32 = i32::from(address) - i32::from(prog_state.counter) - 3;
                if address > i8::MAX.into() || address < i8::MIN.into() {
                    // more than one byte needed for rel address
                    match label_ref {
                        LabelRef::Label { label_name } => {
                            return Err(GetBytesError::RelLabelNotInRange {
                                label_name: label_name.clone(),
                            });
                        }
                        LabelRef::SubLabel { sub_label_name } => {
                            let label_name = &prog_state.current_label;
                            return Err(GetBytesError::RelSubLabelNotInRange {
                                label_name: label_name.clone(),
                                sub_label_name: sub_label_name.clone(),
                            });
                        }
                        LabelRef::FullSubLabel {
                            label_name,
                            sub_label_name,
                        } => {
                            return Err(GetBytesError::RelSubLabelNotInRange {
                                label_name: label_name.clone(),
                                sub_label_name: sub_label_name.clone(),
                            });
                        }
                    }
                }

                let address: i8 = address.try_into().unwrap();
                let bytes = address.to_be_bytes();

                return Ok(vec![0x80, bytes[0]]);
            }
            UxnToken::LitAddressAbs(label_ref) => {
                let address = get_address_of_label(label_ref, prog_state)?;
                let bytes = address.to_be_bytes();

                return Ok(vec![0xa0, bytes[0], bytes[1]]);
            }
            UxnToken::LabelDefine(_) => return Err(GetBytesError::NotWritableToken),
            UxnToken::SubLabelDefine(_) => return Err(GetBytesError::NotWritableToken),
            UxnToken::RawAbsAddr(label_ref) => {
                let address = get_address_of_label(label_ref, prog_state)?;
                let bytes = address.to_be_bytes();
                return Ok(vec![bytes[0], bytes[1]]);
            }
        }
    }

    fn num_bytes(&self) -> u16 {
        match self {
            UxnToken::Op(_) => return 0x1,
            UxnToken::MacroDefine(_) => return 0x0,
            UxnToken::MacroStartDelimiter => return 0x0,
            UxnToken::MacroEndDelimiter => return 0x0,
            UxnToken::MacroInvocation(_) => return 0x0,
            UxnToken::RawByte(_) => return 0x1,
            UxnToken::RawShort(_) => return 0x2,
            UxnToken::RawWord(w) => return w.len().try_into().unwrap(),
            UxnToken::LitByte(_) => return 0x2,
            UxnToken::LitShort(_) => return 0x3,
            UxnToken::LitAddressZeroPage(_) => return 0x2,
            UxnToken::LitAddressRel(_) => return 0x2,
            UxnToken::LitAddressAbs(_) => return 0x3,
            UxnToken::LabelDefine(_) => return 0x0,
            UxnToken::SubLabelDefine(_) => return 0x0,
            UxnToken::RawAbsAddr(_) => return 0x2,
            UxnToken::PadAbs(_) => panic!(),
            UxnToken::PadRel(_) => panic!(),
        }
    }

    pub fn update_prog_counter(&self, prog_counter: u16) -> u16 {
        match self {
            UxnToken::PadAbs(n) => return *n,
            UxnToken::PadRel(n) => return prog_counter + *n,
            _ => return prog_counter + self.num_bytes(),
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
            ParseError::RuneAbsentArg { rune } => {
                write!(f, "'{}' missing argument", rune)
            }
            ParseError::RuneInvalidArg { rune, supplied_arg } => {
                write!(f, "'{}' has invalid argument '{}'", rune, supplied_arg)
            }
        }
    }
}

#[cfg(feature = "asm")]
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

        if &s[0..1] == "%" {
            if s.len() < 2 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "%".to_owned(),
                });
            }

            return Ok(UxnToken::MacroDefine((&s[1..]).to_owned()));
        }

        if s == "{" {
            return Ok(UxnToken::MacroStartDelimiter);
        }

        if s == "}" {
            return Ok(UxnToken::MacroEndDelimiter);
        }

        if &s[0..1] == "\"" {
            let sb = (&s[1..]).as_bytes();

            if s.len() < 2 {
                return Err(ParseError::RuneAbsentArg {
                    rune: "\"".to_owned(),
                });
            }

            if sb.iter().filter(|x| **x > 0x7f).count() > 0 {
                // not ascii
                return Err(ParseError::RuneInvalidArg {
                    rune: "\"".to_owned(),
                    supplied_arg: s[1..].to_owned(),
                });
            }

            return Ok(UxnToken::RawWord(sb.iter().map(|x| *x).collect()));
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
                }
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

        if &s[0..1] == "." {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: ".".to_owned(),
                });
            }

            let label_ref = s[1..].parse::<LabelRef>().unwrap();

            return Ok(UxnToken::LitAddressZeroPage(label_ref));
        }

        if &s[0..1] == "," {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: ",".to_owned(),
                });
            }

            let label_ref = s[1..].parse::<LabelRef>().unwrap();

            return Ok(UxnToken::LitAddressRel(label_ref));
        }

        if &s[0..1] == ";" {
            if s.len() == 1 {
                return Err(ParseError::RuneAbsentArg {
                    rune: ";".to_owned(),
                });
            }

            let label_ref = s[1..].parse::<LabelRef>().unwrap();

            return Ok(UxnToken::LitAddressAbs(label_ref));
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
    use crate::uxnasmlib::asm::prog_state::Label;
    use std::collections::HashMap;
    use crate::ops;

    // test `from_str()` for `LabelRef`; test that each type of label
    // can be correctly parsed
    #[test]
    fn test_label_ref_from_str() {
        let inputs = [
            (
                "test_label",
                LabelRef::Label {
                    label_name: "test_label".to_owned(),
                },
            ),
            (
                "&sub_label",
                LabelRef::SubLabel {
                    sub_label_name: "sub_label".to_owned(),
                },
            ),
            (
                "test_label/sub_label",
                LabelRef::FullSubLabel {
                    label_name: "test_label".to_owned(),
                    sub_label_name: "sub_label".to_owned(),
                },
            ),
        ];

        for (input, expected) in inputs.into_iter() {
            let returned = input.parse::<LabelRef>();
            assert_eq!(returned, Ok(expected));
        }
    }

    #[test]
    fn test_get_bytes_not_writable() {
        let inputs = [
            UxnToken::MacroDefine("blah".to_owned()),
            UxnToken::MacroStartDelimiter,
            UxnToken::MacroEndDelimiter,
            UxnToken::MacroInvocation("blah".to_owned()),
            UxnToken::PadAbs(0xff),
            UxnToken::PadRel(0xff),
            UxnToken::LabelDefine("blah".to_owned()),
            UxnToken::SubLabelDefine("blah".to_owned()),
        ];

        let labels = HashMap::new();
        let prog_state = ProgState {
            counter: 0x0,
            labels: &labels,
            current_label: "".to_owned(),
        };

        for input in inputs.into_iter() {
            let returned = input.get_bytes(&prog_state);
            assert_eq!(returned, Err(GetBytesError::NotWritableToken));
        }
    }

    // test `get_bytes` function; for each possible input token,
    // verify that the correct sequence of bytes is produced for it
    #[test]
    fn test_get_bytes_happy() {
        let mut labels = HashMap::new();
        labels.insert("test_label_zp".to_owned(), Label::new(0x0012));
        labels
            .get_mut("test_label_zp")
            .unwrap()
            .sub_labels
            .insert("sub_label".to_owned(), 0x0015);
        labels
            .get_mut("test_label_zp")
            .unwrap()
            .sub_labels
            .insert("sub_label2".to_owned(), 0x0076);

        labels.insert("test_label".to_owned(), Label::new(0x1234));
        labels.insert("test_label2".to_owned(), Label::new(0x1235));
        labels
            .get_mut("test_label2")
            .unwrap()
            .sub_labels
            .insert("sub_label".to_owned(), 0x4576);

        let prog_state = ProgState {
            counter: 0x0,
            labels: &labels,
            current_label: "test_label_zp".to_owned(),
        };

        let inputs = [
            (
                UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()),
                vec![0x17],
            ),
            (UxnToken::RawByte(0xab), vec![0xab]),
            (UxnToken::RawShort(0xabcd), vec![0xab, 0xcd]),
            (UxnToken::LitByte(0xab), vec![0x80, 0xab]),
            (UxnToken::LitShort(0xabcd), vec![0xA0, 0xab, 0xcd]),
            (
                UxnToken::LitAddressZeroPage("test_label_zp".parse::<LabelRef>().unwrap()),
                vec![0x80, 0x12],
            ),
            (
                UxnToken::LitAddressZeroPage(
                    "test_label_zp/sub_label".parse::<LabelRef>().unwrap(),
                ),
                vec![0x80, 0x15],
            ),
            (
                UxnToken::LitAddressZeroPage("&sub_label2".parse::<LabelRef>().unwrap()),
                vec![0x80, 0x76],
            ),
            (
                UxnToken::LitAddressRel("test_label_zp".parse::<LabelRef>().unwrap()),
                vec![0x80, 0x0f],
            ),
            (
                UxnToken::LitAddressRel("test_label_zp/sub_label".parse::<LabelRef>().unwrap()),
                vec![0x80, 0x12],
            ),
            (
                UxnToken::LitAddressRel("&sub_label2".parse::<LabelRef>().unwrap()),
                vec![0x80, 0x73],
            ),
            (
                UxnToken::LitAddressAbs("test_label_zp".parse::<LabelRef>().unwrap()),
                vec![0xa0, 0x00, 0x12],
            ),
            (
                UxnToken::LitAddressAbs("test_label_zp/sub_label".parse::<LabelRef>().unwrap()),
                vec![0xa0, 0x00, 0x15],
            ),
            (
                UxnToken::LitAddressAbs("&sub_label2".parse::<LabelRef>().unwrap()),
                vec![0xa0, 0x00, 0x76],
            ),
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
                vec![0x00, 0x76],
            ),
            (
                UxnToken::RawWord("hello world".as_bytes().iter().copied().collect()),
                vec![
                    0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
                ],
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

        let inputs = [
            UxnToken::RawAbsAddr("test_label_xyz".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressZeroPage("test_label_xyz".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressRel("test_label_xyz".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressAbs("test_label_xyz".parse::<LabelRef>().unwrap()),
        ];

        for input in inputs.into_iter() {
            let output = input.get_bytes(&ProgState {
                counter: 0,
                labels: &labels,
                current_label: "".to_owned(),
            });

            assert_eq!(
                output,
                Err(GetBytesError::UndefinedLabel {
                    label_name: "test_label_xyz".to_owned(),
                })
            );
        }
    }

    // test `get_bytes` function with a full sub-label that hasn't been defined
    #[test]
    fn test_get_bytes_unrecognised_sub_label_full() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x1234));
        labels
            .get_mut("test_label")
            .unwrap()
            .sub_labels
            .insert("sub_label".to_owned(), 0x4576);

        let inputs = [
            UxnToken::RawAbsAddr("test_label/sub_label_xyz".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressZeroPage("test_label/sub_label_xyz".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressRel("test_label/sub_label_xyz".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressAbs("test_label/sub_label_xyz".parse::<LabelRef>().unwrap()),
        ];

        for input in inputs.into_iter() {
            let output = input.get_bytes(&ProgState {
                counter: 0,
                labels: &labels,
                current_label: "".to_owned(),
            });

            assert_eq!(
                output,
                Err(GetBytesError::UndefinedSubLabel {
                    label_name: "test_label".to_owned(),
                    sub_label_name: "sub_label_xyz".to_owned(),
                })
            );
        }
    }

    // test `get_bytes` function with a partial sub-label that hasn't been defined
    #[test]
    fn test_get_bytes_unrecognised_sub_label_partial() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x1234));
        labels
            .get_mut("test_label")
            .unwrap()
            .sub_labels
            .insert("sub_label".to_owned(), 0x4576);
        labels.insert("test_label2".to_owned(), Label::new(0x1234));

        let inputs = [
            UxnToken::RawAbsAddr("&sub_label".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressZeroPage("&sub_label".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressRel("&sub_label".parse::<LabelRef>().unwrap()),
            UxnToken::LitAddressAbs("&sub_label".parse::<LabelRef>().unwrap()),
        ];

        for input in inputs.into_iter() {
            let output = input.get_bytes(&ProgState {
                counter: 0,
                labels: &labels,
                current_label: "test_label2".to_owned(),
            });

            assert_eq!(
                output,
                Err(GetBytesError::UndefinedSubLabel {
                    label_name: "test_label2".to_owned(),
                    sub_label_name: "sub_label".to_owned(),
                })
            );
        }
    }

    #[test]
    fn test_update_prog_counter() {
        let prog_counter = 0;

        let inputs = [
            (UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()), 0x1),
            (UxnToken::MacroDefine("blah".to_owned()), 0x00),
            (UxnToken::MacroStartDelimiter, 0x00),
            (UxnToken::MacroEndDelimiter, 0x00),
            (UxnToken::MacroInvocation("blah".to_owned()), 0x00),
            (UxnToken::PadAbs(0x1ff), 0x1ff),
            (UxnToken::PadRel(0x1fe), 0x1fe),
            (UxnToken::RawByte(0xfe), 0x1),
            (UxnToken::RawShort(0xabcd), 0x2),
            (
                UxnToken::RawWord("hello world".as_bytes().iter().copied().collect()),
                0xb,
            ),
            (UxnToken::LitByte(0xab), 0x2),
            (UxnToken::LitShort(0xabcd), 0x3),
            (
                UxnToken::LitAddressZeroPage("test_label".parse::<LabelRef>().unwrap()),
                0x2,
            ),
            (
                UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap()),
                0x2,
            ),
            (
                UxnToken::LitAddressAbs("test_label".parse::<LabelRef>().unwrap()),
                0x3,
            ),
            (UxnToken::LabelDefine("test_label".to_owned()), 0x0),
            (UxnToken::SubLabelDefine("test_sub_label".to_owned()), 0x0),
            (
                UxnToken::RawAbsAddr("test_label".parse::<LabelRef>().unwrap()),
                0x2,
            ),
        ];

        for (token, expected) in inputs.into_iter() {
            let returned = token.update_prog_counter(prog_counter);
            assert_eq!(returned, expected);
        }
    }

    // test update_prog_counter function when the program counter is not 0
    #[test]
    fn test_update_prog_counter_non_zero() {
        let prog_counter = 0x5;

        let inputs = [
            (
                UxnToken::Op("DEO".parse::<ops::OpObject>().unwrap()),
                0x1 + 0x5,
            ),
            (UxnToken::MacroDefine("blah".to_owned()), 0x05),
            (UxnToken::MacroStartDelimiter, 0x05),
            (UxnToken::MacroEndDelimiter, 0x05),
            (UxnToken::MacroInvocation("blah".to_owned()), 0x05),
            (UxnToken::PadAbs(0x1ff), 0x01ff),
            (UxnToken::PadRel(0x1fe), 0x203),
            (UxnToken::RawByte(0xfe), 0x6),
            (UxnToken::RawShort(0xabcd), 0x7),
            (
                UxnToken::RawWord("hello world".as_bytes().iter().copied().collect()),
                0xb + 0x5,
            ),
            (UxnToken::LitByte(0xab), 0x2 + 0x5),
            (UxnToken::LitShort(0xabcd), 0x3 + 0x5),
            (
                UxnToken::LitAddressZeroPage("test_label".parse::<LabelRef>().unwrap()),
                0x2 + 0x5,
            ),
            (
                UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap()),
                0x2 + 0x5,
            ),
            (
                UxnToken::LitAddressAbs("test_label".parse::<LabelRef>().unwrap()),
                0x3 + 0x5,
            ),
            (UxnToken::LabelDefine("test_label".to_owned()), 0x0 + 0x5),
            (
                UxnToken::SubLabelDefine("test_sub_label".to_owned()),
                0x0 + 0x5,
            ),
            (
                UxnToken::RawAbsAddr("test_label".parse::<LabelRef>().unwrap()),
                0x2 + 0x5,
            ),
        ];

        for (token, expected) in inputs.into_iter() {
            let returned = token.update_prog_counter(prog_counter);
            assert_eq!(returned, expected);
        }
    }

    #[test]
    fn test_get_bytes_lit_address_rel() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x70));
        let mut prog_state = ProgState {
            counter: 0x70,
            labels: &labels,
            current_label: "".to_owned(),
        };

        let token = UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap());
        let returned = token.get_bytes(&prog_state);
        assert_eq!(returned, Ok(vec![0x80, (-3i8).to_be_bytes()[0]]));

        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x7f + 3));
        prog_state.counter = 0x0;
        prog_state.labels = &labels;

        let token = UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap());
        let returned = token.get_bytes(&prog_state);
        assert_eq!(returned, Ok(vec![0x80, 0x7f]));

        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x00));
        prog_state.counter = 0x7f - 2;
        prog_state.labels = &labels;

        let token = UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap());
        let returned = token.get_bytes(&prog_state);
        assert_eq!(returned, Ok(vec![0x80, 0x80]));
    }

    #[test]
    fn test_get_bytes_lit_address_rel_failed() {
        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x00));
        let mut prog_state = ProgState {
            counter: 0x7f + 3,
            labels: &labels,
            current_label: "".to_owned(),
        };

        let token = UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap());
        let returned = token.get_bytes(&prog_state);
        assert_eq!(
            returned,
            Err(GetBytesError::RelLabelNotInRange {
                label_name: "test_label".to_owned()
            })
        );

        let mut labels = HashMap::new();
        labels.insert("test_label".to_owned(), Label::new(0x00));
        prog_state.counter = 0x7f - 1;
        prog_state.labels = &labels;

        let token = UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap());
        let returned = token.get_bytes(&prog_state);

        assert_eq!(
            returned,
            Err(GetBytesError::RelLabelNotInRange {
                label_name: "test_label".to_owned()
            })
        );
    }

    // test from_str for UxnToken with an input that should be parsed as a raw byte
    #[test]
    fn test_from_str_raw_byte() {
        let input = "ab";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::RawByte(0xab);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a macro define
    #[test]
    fn test_from_str_macro_define() {
        let input = "%test_macro";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::MacroDefine("test_macro".to_owned());
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a macro start delimiter
    #[test]
    fn test_from_str_macro_start_delimiter() {
        let input = "{";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::MacroStartDelimiter;
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a macro end delimiter
    #[test]
    fn test_from_str_macro_end_delimiter() {
        let input = "}";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::MacroEndDelimiter;
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a macro invocation
    #[test]
    fn test_from_str_macro_invocation() {
        let input = "test_macro";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::MacroInvocation("test_macro".to_string());
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

    // test from_str for UxnToken with an input that should be parsed as a raw word
    #[test]
    fn test_from_str_raw_word() {
        let input = "\"helloWorld";
        let output = input.parse::<UxnToken>();

        let expected = UxnToken::RawWord(vec![
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x57, 0x6f, 0x72, 0x6c, 0x64,
        ]);
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a literal address in the
    // zero page
    #[test]
    fn test_from_str_lit_address_zero_page() {
        let input = ".test_label";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::LitAddressZeroPage("test_label".parse::<LabelRef>().unwrap());
        assert_eq!(output, Ok(expected));
    }

    // test from_str for UxnToken with an input that should be parsed as a literal relative address
    #[test]
    fn test_from_str_lit_address_rel() {
        let input = ",test_label";
        let output = input.parse::<UxnToken>();
        let expected = UxnToken::LitAddressRel("test_label".parse::<LabelRef>().unwrap());
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
        let inputs = ["|", "$", "#", "'", "@", ":", ".", ",", "\""];

        for input in inputs {
            let output = input.parse::<UxnToken>();
            let expected = Err(ParseError::RuneAbsentArg {
                rune: input.to_owned(),
            });

            assert_eq!(output, expected);
        }
    }

    // test from_str for UxnToken with an input that should trigger an error because a rune has
    // invalid argument attached
    #[test]
    fn test_from_str_rune_invalid_arg() {
        let inputs = [
            "|zz",
            "$uu",
            "#abcdd",
            "#ax",
            "#abxd",
            "'aa",
            "'???",
            "\"h???lloWorld",
        ];

        for input in inputs {
            let output = input.parse::<UxnToken>();
            let expected = Err(ParseError::RuneInvalidArg {
                rune: input[0..1].to_owned(),
                supplied_arg: input[1..].to_owned(),
            });

            assert_eq!(output, expected);
        }
    }
}
