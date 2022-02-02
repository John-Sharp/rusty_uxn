use std::error::Error;
use std::fmt;
use std::fs::File;
use clap::Parser;
use std::io::BufReader;
use std::io::BufRead;

/// A rust implementation of assembler for uxn cpu
#[derive(Parser)]
pub struct Cli {

    /// The path to the assembly file
    #[clap(parse(from_os_str))]
    pub src_path: std::path::PathBuf,

    /// The path to the output rom
    #[clap(parse(from_os_str))]
    pub dst_path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct FileOpenError {
    fname: String,
    fdesc: String,
}

impl fmt::Display for FileOpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error opening {} file: {}",
               self.fdesc,
               self.fname)
    }
}

impl Error for FileOpenError {
}



mod asm {
    use std::io::Write; 
    use std::collections::HashMap;
    use std::str::FromStr;

    mod tokens {
        use std::collections::HashMap;
        use std::str::FromStr;
        use std::convert::Infallible;

        pub mod ops {
            use std::str::FromStr; 

            #[derive(Debug)]
            pub enum OpCode {
                Brk,
                Deo,
            }

            #[derive(Debug)]
            pub struct OpObject {
                keep: bool,
                ret: bool,
                short: bool,
                op_code: OpCode,
            }

            impl OpObject {
                pub fn get_bytes(&self) -> Vec::<u8> {
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
            pub struct ParseOpObjectError {}
            
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
        }

        use ops::OpObject;
        
        #[derive(Debug)]
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
            pub fn get_bytes(&self, prog_counter: u16, labels: &HashMap<String, u16>) -> Vec::<u8> {
                match self {
                    UxnToken::Op(o) => return o.get_bytes(),
                    UxnToken::MacroInvocation(_) => return vec!(0xaa, 0xbb),
                    UxnToken::PadAbs(n) => {
                        let bytes_to_write = *n - prog_counter;

                        return vec!(0x00; bytes_to_write.into());
                    },
                    UxnToken::PadRel(n) => return vec!(0x00; (*n).into()),
                    UxnToken::RawByte(b) => return vec!(*b),
                    UxnToken::RawShort(_) => return vec!(0xdd,),
                    UxnToken::LitByte(b) => return vec!(0x80, *b),
                    UxnToken::LitShort(s) => {
                        let bytes = s.to_be_bytes();
                        return vec!(0xA0, bytes[0], bytes[1]);
                    },
                    UxnToken::LabelDefine(_) => return vec!(),
                    UxnToken::RawAbsAddr(label) => {
                        println!("label is {}", label);
                        if let Some(addr) = labels.get(label) {
                            let bytes = addr.to_be_bytes();
                            return vec!(bytes[0], bytes[1]);
                        } else {
                            panic!();
                        }
                    },
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
                    UxnToken::LitByte(_) => return 0x1,
                    UxnToken::LitShort(_) => return 0x2,
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
    }

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
                let mut in_comment = false;
                let mut prog_loc = 0;
                let mut labels = HashMap::new();

                let input = input.map(|l| {
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

                    match ret {
                        UxnToken::PadAbs(n) => {
                            if n < prog_loc {
                                println!("Error in program: absolute padding to area of program already written to");
                                std::process::exit(1);
                            }

                            prog_loc += ret.num_bytes(prog_loc);
                        },
                        UxnToken::PadRel(_) => {
                            prog_loc += ret.num_bytes(prog_loc);
                        }
                        UxnToken::LabelDefine(ref label_name) => {
                            labels.insert(
                                label_name.clone(),
                                prog_loc);
                        },
                        _ => {
                            if prog_loc < 0x100 {
                                println!("Error in program: writing to zero page");
                                std::process::exit(1);
                            }

                            prog_loc += ret.num_bytes(prog_loc);
                        },
                    };

                    return ret;
                });

                let program = input.collect::<Vec::<_>>();

                return Ok(Asm{labels, program});
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
                        if let Err(err) = target.write(&next_token_bytes[(next_token_bytes.len()-bytes_to_write)..]) {
                            println!("Error writing to file {:?}",
                                     err);
                            std::process::exit(1);
                        }
                    }

                    bytes_encountered += next_token_bytes.len();
                }
            }
    }
}



pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    println!("in the run function src path: {}",
        config.src_path.as_path().display());

    let fp = match File::open(config.src_path.as_path()) {
        Ok(fp) => fp,
        Err(_err) => {
            return Err(Box::new(FileOpenError{
                fname: config.src_path.as_path().display().to_string().clone(),
                fdesc: "input".to_string(),
            }));
        },
    };

    let input = BufReader::new(fp).lines().map(|l| l.unwrap());

    let mut program = asm::Asm::assemble(input).unwrap();

    let fp = match File::create(config.dst_path.as_path()) {
        Ok(fp) => fp,
        Err(_err) => {
            return Err(Box::new(FileOpenError{
                fname: config.dst_path.as_path().display().to_string().clone(),
                fdesc: "output".to_string(),
            }));
        },

    };

    program.output(fp);

    return Ok(());
}
