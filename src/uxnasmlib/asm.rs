use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

mod tokens;
use std::convert::Infallible;
use tokens::UxnToken;

pub struct Asm {
    program: Vec<UxnToken>,
    labels: HashMap<String, u16>,
}

#[derive(Debug, PartialEq)]
pub enum AsmError {
    General,
    AbsPaddingRegression,
    ZeroPageWrite,
    TokenParseError, // TODO improve
}

impl Asm {
    pub fn assemble<I>(input: I) -> Result<Self, AsmError>
    where
        I: Iterator<Item = String>,
    {
        let token_strings = split_to_token_strings(input);

        let token_strings = strip_comments(token_strings);

        let tokens = token_strings.map(|t| t.parse::<UxnToken>());

        let mut labels = HashMap::new();

        let validated_tokens = validate_tokens(tokens, &mut labels);

        let program = validated_tokens.collect::<Result<Vec<_>, AsmError>>()?;

        return Ok(Asm { labels, program });
    }

    pub fn output<W>(&mut self, mut target: W)
    where
        W: Write,
    {
        let mut bytes_encountered = 0usize;
        for i in &self.program {
            let next_token_bytes = i.get_bytes(bytes_encountered.try_into().unwrap(), &self.labels);

            let bytes_to_write = if bytes_encountered + next_token_bytes.len() < 0x100 {
                0
            } else if bytes_encountered < 0x100 {
                bytes_encountered + next_token_bytes.len() - 0x100
            } else {
                next_token_bytes.len()
            };

            if bytes_to_write > 0 {
                if let Err(err) =
                    target.write(&next_token_bytes[(next_token_bytes.len() - bytes_to_write)..])
                {
                    println!("Error writing to file {:?}", err);
                    std::process::exit(1);
                }
            }

            bytes_encountered += next_token_bytes.len();
        }
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

        l.split_whitespace()
            .map(|w| String::from_str(w).unwrap())
            .collect::<Vec<_>>()
    });

    StringIter { inner_iter: x }
}

fn validate_tokens<'a, I: 'a>(
    input: I,
    labels: &'a mut HashMap<String, u16>,
) -> impl Iterator<Item = Result<UxnToken, AsmError>> + 'a
where
    I: Iterator<Item = Result<UxnToken, Infallible>>,
{
    let mut prog_loc = 0u16;

    input.map(move |t| match t {
        Ok(t) => {
            match t {
                UxnToken::PadAbs(n) => {
                    if n < prog_loc {
                        return Err(AsmError::AbsPaddingRegression);
                    }

                    prog_loc += t.num_bytes(prog_loc);
                }
                UxnToken::PadRel(_) => {
                    prog_loc += t.num_bytes(prog_loc);
                }
                UxnToken::LabelDefine(ref label_name) => {
                    labels.insert(label_name.clone(), prog_loc);
                }
                _ => {
                    if prog_loc < 0x100 {
                        return Err(AsmError::ZeroPageWrite);
                    }

                    prog_loc += t.num_bytes(prog_loc);
                }
            };

            return Ok(t);
        }
        Err(e) => {
            return Err(AsmError::TokenParseError);
        }
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
        return Some(s);
    });

    StringIter { inner_iter: x }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test `split_to_token_strings` function; create input with
    // bracket separators and assert that it is split as expected
    // into token strings
    #[test]
    fn test_split_to_token_strings() {
        let input = vec![
            "tokenA tokenB tokenC{tokenD".to_owned(),
            "}tokenE               (tokenF            tokenG".to_owned(),
        ];

        assert_eq!(
            split_to_token_strings(input.into_iter()).collect::<Vec<_>>(),
            vec!(
                "tokenA", "tokenB", "tokenC", "{", "tokenD", "}", "tokenE", "(", "tokenF", "tokenG"
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
        let mut labels = HashMap::new();
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
        ];

        let output = validate_tokens(input.into_iter(), &mut labels).collect::<Vec<_>>();

        let expected_output: Vec<Result<UxnToken, AsmError>>;
        expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
        ];

        assert_eq!(output, expected_output);

        let expected_labels = HashMap::new();
        assert_eq!(labels, expected_labels);
    }

    // test `validate_tokens` function, with multiple paddings;
    // test multiple paddings
    // (both relative and absolue) and
    // assert that function successfully runs
    #[test]
    fn test_validate_tokens_happy_multi_padding() {
        let mut labels = HashMap::new();
        let input = vec![
            Ok(UxnToken::PadRel(0x80)),
            Ok(UxnToken::PadAbs(0xc0)),
            Ok(UxnToken::PadRel(0x40)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
        ];

        let output = validate_tokens(input.into_iter(), &mut labels).collect::<Vec<_>>();

        let expected_output: Vec<Result<UxnToken, AsmError>>;
        expected_output = vec![
            Ok(UxnToken::PadRel(0x80)),
            Ok(UxnToken::PadAbs(0xc0)),
            Ok(UxnToken::PadRel(0x40)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
        ];

        assert_eq!(output, expected_output);

        let expected_labels = HashMap::new();
        assert_eq!(labels, expected_labels);
    }

    // test `validate_tokens` function with labels; test having
    // labels in the token stream and check they're location
    // is stored as expected in a hash map
    #[test]
    fn test_validate_tokens_happy_label() {
        let mut labels = HashMap::new();
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::LabelDefine("test_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawAbsAddr("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::LabelDefine("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::RawAbsAddr("test_label".to_owned())),
        ];

        let output = validate_tokens(input.into_iter(), &mut labels).collect::<Vec<_>>();

        let expected_output: Vec<Result<UxnToken, AsmError>>;
        expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::LabelDefine("test_label".to_owned())),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawAbsAddr("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::LabelDefine("test_label2".to_owned())),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::RawAbsAddr("test_label".to_owned())),
        ];

        assert_eq!(output, expected_output);

        let mut expected_labels = HashMap::new();
        expected_labels.insert("test_label".to_owned(), 0x100);
        expected_labels.insert("test_label2".to_owned(), 0x105);
        assert_eq!(labels, expected_labels);
    }

    // test `validate_tokens` function padding regression error;
    // test having two absolute paddings, one padding to before
    // current program location. Assert the correct error is
    // received
    #[test]
    fn test_validate_tokens_padding_regression() {
        let mut labels = HashMap::new();
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawShort(0xbbcc)),
            Ok(UxnToken::PadAbs(0x101)),
        ];

        let output =
            validate_tokens(input.into_iter(), &mut labels).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::AbsPaddingRegression));
    }

    // test `validate_tokens` function zero page write error;
    // test that attempting to write to the zero page results
    // in the correct error
    #[test]
    fn test_validate_tokens_zero_page_write() {
        let mut labels = HashMap::new();
        let input = vec![Ok(UxnToken::PadAbs(0xff)), Ok(UxnToken::RawByte(0xaa))];

        let output =
            validate_tokens(input.into_iter(), &mut labels).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::ZeroPageWrite));
    }

    // TODO
    // test `validate_tokens` function token parse error;
    // test that a parse error in the input stream is correctly
    // propagated as a AsmError::TokenParseError
    #[test]
    fn test_validate_tokens_token_parse_error() {}
}
