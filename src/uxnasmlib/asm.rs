use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

mod tokens;
use tokens::UxnToken;

pub struct Asm {
    program: Vec<UxnToken>,
    labels: HashMap<String, u16>,
}

impl Asm {
    pub fn assemble<I>(input: I) -> Result<Self, ()>
    where
        I: Iterator<Item = String>,
    {
        let mut prog_loc = 0;
        let mut labels = HashMap::new();

        let token_strings = split_to_token_strings(input);

        let token_strings = strip_comments(token_strings);

        let input = token_strings.map(|t| {
                let ret = t.parse::<UxnToken>().unwrap();

                match ret {
                    UxnToken::PadAbs(n) => {
                        if n < prog_loc {
                            println!(
                                "Error in program: absolute padding to area of program already written to"
                            );
                            std::process::exit(1);
                        }

                        prog_loc += ret.num_bytes(prog_loc);
                    }
                    UxnToken::PadRel(_) => {
                        prog_loc += ret.num_bytes(prog_loc);
                    }
                    UxnToken::LabelDefine(ref label_name) => {
                        labels.insert(label_name.clone(), prog_loc);
                    }
                    _ => {
                        if prog_loc < 0x100 {
                            println!("Error in program: writing to zero page");
                            std::process::exit(1);
                        }

                        prog_loc += ret.num_bytes(prog_loc);
                    }
                };

                return ret;
            });

        let program = input.collect::<Vec<_>>();

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
}
