use std::collections::HashMap;
use std::error;
use std::fmt;
use std::io;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::str::FromStr;

pub mod prog_state {
    use std::collections::HashMap;

    #[derive(Debug, PartialEq)]
    pub struct Label {
        pub address: u16,
        pub sub_labels: HashMap<String, u16>,
    }

    impl Label {
        pub fn new(address: u16) -> Self {
            Label {
                address,
                sub_labels: HashMap::new(),
            }
        }
    }

    pub struct ProgState<'a> {
        pub counter: u16,
        pub labels: &'a HashMap<String, Label>,
        pub current_label: String,
    }
}

use prog_state::Label;
use prog_state::ProgState;

mod tokens;
use tokens::UxnToken;

mod macros;

pub struct Asm {
    program: Vec<UxnToken>,
    labels: HashMap<String, Label>,
}

#[derive(Debug, PartialEq)]
pub enum AsmError {
    AbsPaddingRegression,
    ZeroPageWrite,
    TokenParseError {
        parse_error: tokens::ParseError,
    },
    Output {
        error: io::ErrorKind,
        msg: String,
    },
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
    SubLabelWithNoLabel {
        sub_label_name: String,
    },
    RelLabelNotInRange {
        label_name: String,
    },
    RelSubLabelNotInRange {
        label_name: String,
        sub_label_name: String,
    },
    MacroDefineWithinMacro {
        outer_macro_name: String,
        inner_macro_name: String,
    },
    DoubleMacroDefine {
        macro_name: String,
    },
    MalformedMacroDefine {
        macro_name: String,
    },
    MacroStartDelimiterMisplaced,
    MacroEndDelimiterMisplaced,
    UnrecognisedMacro {
        macro_name: String,
    },
    MaxMacroCallDepthExceeded {
        macro_name: String,
    },
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsmError::AbsPaddingRegression => {
                write!(
                    f,
                    "attempt to pad to absolute program location behind current counter"
                )
            }
            AsmError::ZeroPageWrite => {
                write!(f, "zero page write")
            }
            AsmError::TokenParseError { parse_error } => {
                write!(f, "failed to parse token: {}", parse_error)
            }
            AsmError::Output { error: _error, msg } => {
                write!(f, "output error: {}", msg)
            }
            AsmError::UndefinedLabel { label_name } => {
                write!(f, "undefined label: {}", label_name)
            }
            AsmError::UndefinedSubLabel {
                label_name,
                sub_label_name,
            } => {
                write!(f, "undefined sub-label: {}/{}", label_name, sub_label_name)
            }
            AsmError::LabelNotInZeroPage { label_name } => {
                write!(f, "label not in zero page: {}", label_name)
            }
            AsmError::SubLabelNotInZeroPage {
                label_name,
                sub_label_name,
            } => {
                write!(
                    f,
                    "sub-label not in zero page: {}/{}",
                    label_name, sub_label_name
                )
            }
            AsmError::SubLabelWithNoLabel { sub_label_name } => {
                write!(f, "sub-label defined before label: {}", sub_label_name)
            }
            AsmError::RelLabelNotInRange { label_name } => {
                write!(f, "relative label not in range: {}", label_name)
            }
            AsmError::RelSubLabelNotInRange {
                label_name,
                sub_label_name,
            } => {
                write!(
                    f,
                    "relative sub-label not in range: {}/{}",
                    label_name, sub_label_name
                )
            }
            AsmError::MacroDefineWithinMacro {
                inner_macro_name,
                outer_macro_name,
            } => {
                write!(
                    f,
                    "macro ('{}') defined within macro '{}'",
                    inner_macro_name, outer_macro_name
                )
            }
            AsmError::DoubleMacroDefine { macro_name } => {
                write!(f, "macro '{}' defined twice", macro_name)
            }
            AsmError::MalformedMacroDefine { macro_name } => {
                write!(f, "macro '{}' incorrectly defined", macro_name)
            }
            AsmError::MacroStartDelimiterMisplaced => {
                write!(f, "misplaced '{{'")
            }
            AsmError::MacroEndDelimiterMisplaced => {
                write!(f, "misplaced '}}'")
            }
            AsmError::UnrecognisedMacro { macro_name } => {
                write!(f, "undefined macro '{}'", macro_name)
            }
            AsmError::MaxMacroCallDepthExceeded { macro_name } => {
                write!(f, "too many nested macro calls for call to '{}'",
                       macro_name)
            }
        }
    }
}

impl error::Error for AsmError {}

impl Asm {
    pub fn assemble<I>(input: I) -> Result<Self, AsmError>
    where
        I: Iterator<Item = String>,
    {
        let token_strings = split_to_token_strings(input);

        let token_strings = strip_comments(token_strings);

        let tokens = token_strings.map(|t| t.parse::<UxnToken>());

        // convert token stream error tokens::ParseError type into AsmError
        let tokens = validate_tokens(tokens);

        // strip macro definitions, expand macro invocations
        let tokens = macros::process_macros(tokens);

        // populate labels map
        let mut labels = HashMap::new();
        let tokens = get_labels(tokens, &mut labels);

        // collect, returning first error encountered
        let program = tokens.collect::<Result<Vec<_>, AsmError>>()?;

        return Ok(Asm { labels, program });
    }

    pub fn output<W>(&mut self, mut target: W) -> Result<(), AsmError>
    where
        W: Write + Seek,
    {
        let mut prog_state = ProgState {
            counter: 0,
            labels: &self.labels,
            current_label: "".to_owned(),
        };
        let mut high_water_mark = 0u16;

        for token in &self.program {

            if let UxnToken::LabelDefine(label_name) = token {
                prog_state.current_label = label_name.clone();
                continue;
            }

            match token.get_bytes(&prog_state) {
                Ok(bytes) => {

                    // check for zero page write
                    if prog_state.counter < 0x100 {
                        return Err(AsmError::ZeroPageWrite);
                    }

                    if prog_state.counter > high_water_mark {
                        // we are writing to a location in the program beyond
                        // which we have previously written, fill the bytes that
                        // have not been written yet with zeros

                        if let Err(err) = target.seek(SeekFrom::End(0)) {
                            return Err(AsmError::Output {
                                error: err.kind(),
                                msg: err.to_string(),
                            });
                        }

                        if let Err(err) = 
                        target.write(
                            &vec![0x00; (prog_state.counter - high_water_mark).into()])
                        {
                            return Err(AsmError::Output {
                                error: err.kind(),
                                msg: err.to_string(),
                            });
                        }
                    } else {
                        // the program counter is pointing to a part of the 
                        // 'target' that has already been written to, seek to
                        // that location
                        if let Err(err) = target.seek(SeekFrom::Start((prog_state.counter - 0x100).into())) {
                            return Err(AsmError::Output {
                                error: err.kind(),
                                msg: err.to_string(),
                            });
                        }
                    }

                    // at this point the 'target' file/buffer is guaranteed
                    // to be at the location we want to write the bytes corresponding
                    // to the current token, so write
                    if let Err(err) =
                        target.write(&bytes)
                    {
                            return Err(AsmError::Output {
                                error: err.kind(),
                                msg: err.to_string(),
                            });
                    }
                }
                Err(tokens::GetBytesError::NotWritableToken) => {
                    // not really an error, just a token (such as PadRel/PadAbs)
                    // that isn't designed to write any bytes, but may
                    // change the program counter
                }
                // the following are all real errors 
                Err(tokens::GetBytesError::UndefinedLabel {
                    label_name,
                }) => {
                    return Err(AsmError::UndefinedLabel {
                        label_name,
                    });
                }
                Err(tokens::GetBytesError::UndefinedSubLabel {
                    label_name,
                    sub_label_name,
                }) => {
                    return Err(AsmError::UndefinedSubLabel {
                        label_name,
                        sub_label_name,
                    });
                }
                Err(tokens::GetBytesError::LabelNotInZeroPage { label_name }) => {
                    return Err(AsmError::LabelNotInZeroPage { label_name });
                }
                Err(tokens::GetBytesError::SubLabelNotInZeroPage {
                    label_name,
                    sub_label_name,
                }) => {
                    return Err(AsmError::SubLabelNotInZeroPage {
                        label_name,
                        sub_label_name,
                    });
                }
                Err(tokens::GetBytesError::RelLabelNotInRange { label_name }) => {
                    return Err(AsmError::RelLabelNotInRange { label_name });
                }
                Err(tokens::GetBytesError::RelSubLabelNotInRange {
                    label_name,
                    sub_label_name,
                }) => {
                    return Err(AsmError::RelSubLabelNotInRange {
                        label_name,
                        sub_label_name,
                    });
                }
            }
            
            prog_state.counter = token.update_prog_counter(prog_state.counter);
            if prog_state.counter > high_water_mark {
                high_water_mark = prog_state.counter;
            }
        }
        return Ok(());
    }
}

struct StringIter<I>
where
    I: Iterator<Item = String>,
{
    inner_iter: I,
}

impl<I> Iterator for StringIter<I>
where
    I: Iterator<Item = String>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next()
    }
}

fn split_to_token_strings<I>(input: I) -> impl Iterator<Item = String>
where
    I: Iterator<Item = String>,
{
    let x = input.flat_map(|l| {
        let l = l.replace("{", " { ");
        let l = l.replace("}", " } ");

        let l = l.replace("(", " ( ");
        let l = l.replace(")", " ) ");

        let l = l.replace("[", " [ ");
        let l = l.replace("]", " ] ");

        l.split_whitespace()
            .map(|w| String::from_str(w).unwrap())
            .collect::<Vec<_>>()
    });

    StringIter { inner_iter: x }
}

fn validate_tokens<I>(input: I) -> impl Iterator<Item = Result<UxnToken, AsmError>>
where
    I: Iterator<Item = Result<UxnToken, tokens::ParseError>>,
{
    input.map(|t| match t {
        Ok(t) => Ok(t),
        Err(e) => {
            return Err(AsmError::TokenParseError { parse_error: e });
        }
    })
}

fn get_labels<'a, I: 'a>(
    input: I,
    labels: &'a mut HashMap<String, Label>,
) -> impl Iterator<Item = Result<UxnToken, AsmError>> + 'a
where
    I: Iterator<Item = Result<UxnToken, AsmError>>,
{
    let mut current_label = None;
    let mut prog_loc = 0u16;

    input.map(move |t| match t {
        Ok(UxnToken::LabelDefine(ref label_name)) => {
            current_label = Some(label_name.clone());
            let label = Label::new(prog_loc);
            labels.insert(label_name.clone(), label);
            t
        }
        Ok(UxnToken::SubLabelDefine(ref sub_label_name)) => {
            if let Some(current_label) = &current_label {
                labels
                    .get_mut(current_label)
                    .unwrap()
                    .sub_labels
                    .insert(sub_label_name.clone(), prog_loc);
            } else {
                return Err(AsmError::SubLabelWithNoLabel {
                    sub_label_name: sub_label_name.clone(),
                });
            }
            t
        }
        Ok(t) => {
            prog_loc = t.update_prog_counter(prog_loc);
            Ok(t)
        }
        Err(e) => Err(e),
    })
}

fn strip_comments<I>(input: I) -> impl Iterator<Item = String>
where
    I: Iterator<Item = String>,
{
    let mut in_comment = false;
    let x = input.filter_map(move |s| {
        if s == "(" {
            in_comment = true;
            return None;
        }
        let was_in_comment = in_comment;
        if s == ")" {
            in_comment = false;
        }
        if was_in_comment {
            return None;
        }
        if s == "[" {
            return None;
        }
        if s == "]" {
            return None;
        }
        return Some(s);
    });

    StringIter { inner_iter: x }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokens::LabelRef;
    use std::io::Cursor;

    // test `split_to_token_strings` function; create input with
    // bracket separators and assert that it is split as expected
    // into token strings
    #[test]
    fn test_split_to_token_strings() {
        let input = vec![
            "tokenA tokenB tokenC{tokenD".to_owned(),
            "}tokenE               (tokenF            tokenG".to_owned(),
            "tokenH[tokenI]tokenJ [tokenK".to_owned(),
        ];

        assert_eq!(
            split_to_token_strings(input.into_iter()).collect::<Vec<_>>(),
            vec!(
                "tokenA", "tokenB", "tokenC", "{", "tokenD", "}", "tokenE", "(", "tokenF", "tokenG",
                "tokenH", "[", "tokenI", "]", "tokenJ", "[", "tokenK",
            )
            .into_iter()
            .map(|t| t.to_owned())
            .collect::<Vec<_>>()
        );
    }

    // test `strip_comments` function; create token string input
    // and assert that token strings laying between '(' ')' tokens
    // are removed from the input
    #[test]
    fn test_strip_comments() {
        let input = vec![
            "tokenA", "tokenB", "tokenC", "(", "here", "is", "a", "comment", ")", "tokenG",
        ]
        .into_iter()
        .map(|t| t.to_owned());

        assert_eq!(
            strip_comments(input.into_iter()).collect::<Vec<_>>(),
            vec!("tokenA", "tokenB", "tokenC", "tokenG")
                .into_iter()
                .map(|t| t.to_owned())
                .collect::<Vec<_>>()
        );
    }

    // test `validate_tokens` function; test simplest happy path,
    // create token input and assert that function successfully
    // runs
    #[test]
    fn test_validate_tokens_happy() {
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
        ];

        let output = validate_tokens(input.into_iter()).collect::<Vec<_>>();

        let expected_output: Vec<Result<UxnToken, AsmError>>;
        expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
        ];

        assert_eq!(output, expected_output);
    }

    // test `validate_tokens` function token parse error;
    // test that a parse error in the input stream is correctly
    // propagated as a AsmError::TokenParseError
    #[test]
    fn test_validate_tokens_token_parse_error() {
        let input = vec![
            Ok(UxnToken::PadAbs(0xff)),
            Err(tokens::ParseError::RuneAbsentArg {
                rune: "|".to_owned(),
            }),
        ];

        let output = validate_tokens(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(
            output,
            Err(AsmError::TokenParseError {
                parse_error: tokens::ParseError::RuneAbsentArg {
                    rune: "|".to_owned()
                },
            })
        );
    }

    // test `get_labels` function; test having
    // labels in the token stream and check their location
    // is stored as expected in a hash map
    #[test]
    fn test_get_labels_happy() {
        let mut labels = HashMap::new();
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::LabelDefine("test_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawAbsAddr(
                "test_label2".parse::<LabelRef>().unwrap(),
            )),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::LabelDefine("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::RawAbsAddr(
                "test_label".parse::<LabelRef>().unwrap(),
            )),
        ];

        let output = get_labels(input.into_iter(), &mut labels).collect::<Vec<_>>();

        let expected_output: Vec<Result<UxnToken, AsmError>>;
        expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::LabelDefine("test_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawAbsAddr(
                "test_label2".parse::<LabelRef>().unwrap(),
            )),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::LabelDefine("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::RawAbsAddr(
                "test_label".parse::<LabelRef>().unwrap(),
            )),
        ];

        assert_eq!(output, expected_output);

        let mut expected_labels = HashMap::new();
        expected_labels.insert("test_label".to_owned(), Label::new(0x100));
        expected_labels.insert("test_label2".to_owned(), Label::new(0x105));
        assert_eq!(labels, expected_labels);
    }

    // test `get_labels` function with sub-labels; test having
    // sub-labels in the token stream and check their location
    // is stored as expected in a hash map
    #[test]
    fn test_get_labels_happy_sub_label() {
        let mut labels = HashMap::new();
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::LabelDefine("test_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawAbsAddr(
                "test_label2".parse::<LabelRef>().unwrap(),
            )),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::SubLabelDefine("test_sub_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::SubLabelDefine("test_sub_label2".to_owned())),
            Ok(UxnToken::LabelDefine("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::RawAbsAddr(
                "test_label".parse::<LabelRef>().unwrap(),
            )),
            Ok(UxnToken::SubLabelDefine("test_sub_label".to_owned())),
        ];

        let output = get_labels(input.into_iter(), &mut labels).collect::<Vec<_>>();

        let expected_output: Vec<Result<UxnToken, AsmError>>;
        expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::LabelDefine("test_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawAbsAddr(
                "test_label2".parse::<LabelRef>().unwrap(),
            )),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::SubLabelDefine("test_sub_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::SubLabelDefine("test_sub_label2".to_owned())),
            Ok(UxnToken::LabelDefine("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::RawAbsAddr(
                "test_label".parse::<LabelRef>().unwrap(),
            )),
            Ok(UxnToken::SubLabelDefine("test_sub_label".to_owned())),
        ];

        assert_eq!(output, expected_output);

        let mut expected_labels = HashMap::new();
        expected_labels.insert("test_label".to_owned(), Label::new(0x100));
        expected_labels.insert("test_label2".to_owned(), Label::new(0x106));
        expected_labels
            .get_mut("test_label")
            .unwrap()
            .sub_labels
            .insert("test_sub_label".to_owned(), 0x105);
        expected_labels
            .get_mut("test_label")
            .unwrap()
            .sub_labels
            .insert("test_sub_label2".to_owned(), 0x106);

        expected_labels
            .get_mut("test_label2")
            .unwrap()
            .sub_labels
            .insert("test_sub_label".to_owned(), 0x10a);

        assert_eq!(labels, expected_labels);
    }

    // test `get_labels` function sub-labels without labels error;
    // test having a sub-label in the token stream before any label
    // is defined and check the correct error is returned
    #[test]
    fn test_get_labels_sub_label_without_label() {
        let mut labels = HashMap::new();

        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::SubLabelDefine("test_sub_label".to_owned())),
        ];

        let output =
            get_labels(input.into_iter(), &mut labels).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(
            output,
            Err(AsmError::SubLabelWithNoLabel {
                sub_label_name: "test_sub_label".to_owned()
            })
        );
    }

    #[test]
    fn test_output_happy() {
        let mut input = Asm {
            program: vec![
                UxnToken::PadAbs(0x102),
                UxnToken::RawByte(0x1),
                UxnToken::LitShort(0xaabb),
                UxnToken::PadAbs(0x109),
                UxnToken::LitByte(0x22),
                UxnToken::PadRel(0x5),
                UxnToken::LitByte(0x33),
            ],
            labels: HashMap::new(),
        };

        let expected_output = vec![
            0x00, 0x00, 0x1, 0xa0, 0xaa, 0xbb, 0x00, 0x00, 0x00, 0x80, 0x22, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x33,
        ];

        let mut output = Vec::new();
        let mut output = Cursor::new(output);
        let res = input.output(&mut output);
        let output = output.into_inner();

        assert_eq!(res, Ok(()));
        assert_eq!(output, expected_output);
    }

    // test writing to the zero page generates the correct error
    #[test]
    fn test_output_zero_page_write() {
        let mut input = Asm {
            program: vec![
                UxnToken::PadAbs(0xfe),
                UxnToken::PadRel(0x01),
                UxnToken::RawByte(0xaa),
            ],
            labels: HashMap::new()
        };

        let mut output = Cursor::new(Vec::new());
        let res = input.output(&mut output);

        let expected_output = Err(AsmError::ZeroPageWrite);
        assert_eq!(res, expected_output);
    }

    // test writing to the zero page generates the correct error, 
    // when you jump back to the zero page
    #[test]
    fn test_output_zero_page_write_jump_back() {
        let mut input = Asm {
            program: vec![
                UxnToken::PadAbs(0xff),
                UxnToken::PadRel(0x01),
                UxnToken::RawByte(0xaa),
                UxnToken::PadAbs(0xff),
                UxnToken::RawByte(0xbb),
            ],
            labels: HashMap::new()
        };

        let mut output = Cursor::new(Vec::new());
        let res = input.output(&mut output);

        let expected_output = Err(AsmError::ZeroPageWrite);
        assert_eq!(res, expected_output);
    }

    #[test]
    fn test_output_jump_back() {
        let mut input = Asm {
            program: vec![
                UxnToken::PadAbs(0x100),
                UxnToken::RawByte(0xff),
                UxnToken::RawByte(0xff),
                UxnToken::RawByte(0xff),
                UxnToken::RawByte(0xff),
                UxnToken::PadAbs(0x101),
                UxnToken::RawByte(0xbb),
            ],
            labels: HashMap::new(),
        };

        let expected_output = vec![
            0xff, 0xbb, 0xff, 0xff
        ];

        let mut output = Vec::new();
        let mut output = Cursor::new(output);
        let res = input.output(&mut output);
        let output = output.into_inner();

        assert_eq!(res, Ok(()));
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_output_unrecognised_label() {
        let mut input = Asm {
            program: vec![UxnToken::RawAbsAddr(
                "unrecognised".parse::<LabelRef>().unwrap(),
            )],
            labels: HashMap::new(),
        };

        let mut writer = Vec::new();
        let mut writer = Cursor::new(writer);
        let output = input.output(&mut writer);

        assert_eq!(
            output,
            Err(AsmError::UndefinedLabel {
                label_name: "unrecognised".to_owned()
            })
        );
    }

    #[test]
    fn test_output_unrecognised_sub_label() {
        let mut input = Asm {
            program: vec![UxnToken::RawAbsAddr(
                "label/unrecognised".parse::<LabelRef>().unwrap(),
            )],
            labels: HashMap::new(),
        };

        let mut writer = Vec::new();
        let mut writer = Cursor::new(writer);
        let output = input.output(&mut writer);

        assert_eq!(
            output,
            Err(AsmError::UndefinedSubLabel {
                label_name: "label".to_owned(),
                sub_label_name: "unrecognised".to_owned()
            })
        );
    }

    #[test]
    fn test_output_label_not_in_zero_page() {
        let mut labels = HashMap::new();
        labels.insert("label".to_owned(), Label::new(0x100));
        let mut input = Asm {
            program: vec![UxnToken::LitAddressZeroPage(
                "label".parse::<LabelRef>().unwrap(),
            )],
            labels,
        };

        let mut writer = Vec::new();
        let mut writer = Cursor::new(writer);
        let output = input.output(&mut writer);

        assert_eq!(
            output,
            Err(AsmError::LabelNotInZeroPage {
                label_name: "label".to_owned(),
            })
        );
    }

    #[test]
    fn test_output_sub_label_not_in_zero_page() {
        let mut labels = HashMap::new();
        labels.insert("label".to_owned(), Label::new(0xfe));
        labels
            .get_mut("label")
            .unwrap()
            .sub_labels
            .insert("sub_label".to_owned(), 0x101);
        let mut input = Asm {
            program: vec![UxnToken::LitAddressZeroPage(
                "label/sub_label".parse::<LabelRef>().unwrap(),
            )],
            labels,
        };

        let mut writer = Vec::new();
        let mut writer = Cursor::new(writer);
        let output = input.output(&mut writer);

        assert_eq!(
            output,
            Err(AsmError::SubLabelNotInZeroPage {
                label_name: "label".to_owned(),
                sub_label_name: "sub_label".to_owned(),
            })
        );
    }

    #[test]
    fn test_output_label_not_in_range() {
        let mut labels = HashMap::new();
        labels.insert("label".to_owned(), Label::new(0xffff));

        let mut input = Asm {
            program: vec![UxnToken::LitAddressRel(
                "label".parse::<LabelRef>().unwrap(),
            )],
            labels,
        };

        let mut writer = Vec::new();
        let mut writer = Cursor::new(writer);
        let output = input.output(&mut writer);

        assert_eq!(
            output,
            Err(AsmError::RelLabelNotInRange {
                label_name: "label".to_owned(),
            })
        );
    }

    #[test]
    fn test_output_sub_label_not_in_range() {
        let mut labels = HashMap::new();
        labels.insert("label".to_owned(), Label::new(0xfffc));
        labels
            .get_mut("label")
            .unwrap()
            .sub_labels
            .insert("sub_label".to_owned(), 0xfffd);

        let mut input = Asm {
            program: vec![UxnToken::LitAddressRel(
                "label/sub_label".parse::<LabelRef>().unwrap(),
            )],
            labels,
        };

        let mut writer = Vec::new();
        let mut writer = Cursor::new(writer);
        let output = input.output(&mut writer);

        assert_eq!(
            output,
            Err(AsmError::RelSubLabelNotInRange {
                label_name: "label".to_owned(),
                sub_label_name: "sub_label".to_owned()
            })
        );
    }
}
