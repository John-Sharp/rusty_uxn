use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::fmt;

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

    let input = BufReader::new(fp).split(b' ').map(|x| {
        String::from_utf8(x.unwrap()).unwrap()
    });


    for i in input {
        println!("{}", i);
    }
}
