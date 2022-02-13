use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

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
        write!(f, "Error opening {} file: {}", self.fdesc, self.fname)
    }
}

impl Error for FileOpenError {}

mod asm;

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let fp = match File::open(config.src_path.as_path()) {
        Ok(fp) => fp,
        Err(_err) => {
            return Err(Box::new(FileOpenError {
                fname: config.src_path.as_path().display().to_string().clone(),
                fdesc: "input".to_string(),
            }));
        }
    };

    let input = BufReader::new(fp).lines().map(|l| l.unwrap());

    let mut program = asm::Asm::assemble(input)?;

    let fp = match File::create(config.dst_path.as_path()) {
        Ok(fp) => fp,
        Err(_err) => {
            return Err(Box::new(FileOpenError {
                fname: config.dst_path.as_path().display().to_string().clone(),
                fdesc: "output".to_string(),
            }));
        }
    };

    program.output(fp)?;

    return Ok(());
}
