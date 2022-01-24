use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::fmt;
use std::str::FromStr;
use std::convert::Infallible;

/// A rust implementation of assembler for uxn cpu
#[derive(Parser)]
struct Cli {

    /// The path to the assembly file
    #[clap(parse(from_os_str))]
    src_path: std::path::PathBuf,

    /// The path to the output rom
    #[clap(parse(from_os_str))]
    dst_path: std::path::PathBuf,
}


#[derive(Debug)]
struct CustomError(String);
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
enum OpCode {
    Brk,
}

#[derive(Debug)]
struct OpObject {
    keep: bool,
    ret: bool,
    short: bool,
    op_code: OpCode,
}

#[derive(Debug)]
struct ParseOpObjectError {}

impl FromStr for OpObject {
    type Err = ParseOpObjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 3 {
            return Err(ParseOpObjectError{});
        }

        let ret = match &s[0..3] {
            "BRK" => OpObject{keep: false, ret: false, short: false,
            op_code: OpCode::Brk},
            "LIT" => OpObject{keep: true, ret: false, short: false,
            op_code: OpCode::Brk},
            _ => { return Err(ParseOpObjectError{}) },
        };

        // TODO parse the mode flags
        
        return Ok(ret);
    }
}



#[derive(Debug)]
enum UxnToken {
    Op(OpObject),
    MacroInvocation(String),
    PadAbs(u16),
}

impl FromStr for UxnToken {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(op) = s.parse::<OpObject>() {
            return Ok(UxnToken::Op(op));
        }

        if &s[0..1] == "|" {
            if s.len() < 2 {
                panic!();
            }

            if let Ok(pad_val) = s[1..].parse::<u16>() {
                return Ok(UxnToken::PadAbs(pad_val));
            }
        }

        return Ok(UxnToken::MacroInvocation(s.to_owned()));
    }
}

fn main() {
    let args = Cli::parse();

    let fp = match File::open(args.src_path.as_path()) {
        Ok(fp) => fp,
        Err(err) => {
            println!("Error opening file {}",
                     args.src_path.as_path().display());
            std::process::exit(1);
        },
    };

    let mut in_comment = false;
    let input = BufReader::new(fp).lines().map(|l| {
        let l = l.unwrap();
        let l = l.replace("{", " { ");
        let l = l.replace("}", " } ");

        let l = l.replace("(", " ( ");
        let l = l.replace(")", " ) ");

        l.split_whitespace().map(
            |w| { String::from_str(w).unwrap() }).collect::<Vec::<_>>()
    }).flatten()
    .filter_map(|s| {
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
    })
    .map(|t| t.parse::<UxnToken>().unwrap());

    // go through program, collect macros, expand macros when found in main program, collect labels
    // go through program, write to file, substitute labels



    for i in input {
        println!("**{:?}**", i);
    }
}
