use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::fmt;
use std::str::FromStr;
use std::convert::Infallible;
use std::io::Write; 


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
    Deo,
}

#[derive(Debug)]
struct OpObject {
    keep: bool,
    ret: bool,
    short: bool,
    op_code: OpCode,
}

impl OpObject {
    fn get_bytes(&self) -> Vec::<u8> {
        let byte = match self.op_code {
            OpCode::Brk => 0x00,
            OpCode::Deo => 0x17,
        };

        let byte = if self.keep {
            byte | 0b10000000
        } else {
            byte
        };

        let byte = if self.ret {
            byte | 0b01000000
        } else {
            byte
        };

        let byte = if self.short {
            byte | 0b00100000
        } else {
            byte
        };

        return vec!(byte);
    }
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
            "DEO" => OpObject{keep: false, ret: false, short: false,
            op_code: OpCode::Deo},
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
    RawByte(u8),
    RawShort(u16),
    LitByte(u8),
    LitShort(u16),
}

impl UxnToken {
    fn get_bytes(&self) -> Vec::<u8> {
        match self {
            UxnToken::Op(o) => return o.get_bytes(),
            UxnToken::MacroInvocation(_) => return vec!(0xaa, 0xbb),
            UxnToken::PadAbs(n) => return vec!(0x00; (*n).into()),
            UxnToken::RawByte(b) => return vec!(*b),
            UxnToken::RawShort(_) => return vec!(0xdd,),
            UxnToken::LitByte(b) => return vec!(0x80, *b),
            UxnToken::LitShort(s) => {
                let bytes = s.to_be_bytes();
                return vec!(0xA0, bytes[0], bytes[1]);
            }
        }
    }

    fn num_bytes(&self) -> u16 {
        match self {
            UxnToken::Op(_) => return 0x1,
            UxnToken::MacroInvocation(_) => return 0xff,
            UxnToken::PadAbs(n) => return *n,
            UxnToken::RawByte(b) => return 0x1,
            UxnToken::RawShort(n) => return 0x2,
            UxnToken::LitByte(b) => return 0x1,
            UxnToken::LitShort(n) => return 0x2,
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

        if &s[0..1] == "#" {
            let s = &s[1..]; 
            match s.len() {
                2 => {
                    if let Ok(val) = u8::from_str_radix(s, 16) {
                        return Ok(UxnToken::LitByte(val));
                    } else {
                        panic!();
                    }
                },
                4 => {
                    if let Ok(val) = u16::from_str_radix(s, 16) {
                        return Ok(UxnToken::LitShort(val));
                    } else {
                        panic!();
                    }
                },
                _ => {
                    panic!();
                }
            };
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
    let mut prog_loc = 0;
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
    .map(|t| {
        let ret = t.parse::<UxnToken>().unwrap();

        if let UxnToken::PadAbs(n) = ret {
            if n < prog_loc {
                println!("Error in program: absolute padding to area of program already written to");
                std::process::exit(1);
            }

            prog_loc = ret.num_bytes();


        } else {

            if prog_loc < 0x100 {
                println!("Error in program: writing to zero page");
                std::process::exit(1);
            }
            
            prog_loc += ret.num_bytes();
        }

        return ret;
    });


    // go through program, collect macros, expand macros when found in main program, collect labels
    // go through program, write to file, substitute labels

    let mut fp = match File::create(args.dst_path.as_path()) {
        Ok(fp) => fp,
        Err(err) => {
            println!("Error opening destination file {}",
                     args.dst_path.as_path().display());
            std::process::exit(1);
        },

    };

    let mut bytes_encountered = 0;
    for i in input {

        let next_token_bytes = i.get_bytes();

        let bytes_to_write = if bytes_encountered + next_token_bytes.len() < 0x100 {
            0
        } else if bytes_encountered < 0x100 {
            bytes_encountered + next_token_bytes.len() - 0x100
        } else {
            next_token_bytes.len()
        };

        if bytes_to_write > 0 {
            if let Err(err) = fp.write(&next_token_bytes[(next_token_bytes.len()-bytes_to_write)..]) {
                println!("Error writing to file {:?}",
                         err);
                std::process::exit(1);
            }
        }

        bytes_encountered += next_token_bytes.len();
    }
}
