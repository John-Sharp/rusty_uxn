use clap::Parser;
use std::io;

fn main() {
    let args = rusty_uxn::uxnemulib::Cli::parse();
    let other_config = rusty_uxn::uxnemulib::Config{stderr_writer: io::stderr()};

    if let Err(e) = rusty_uxn::uxnemulib::run(args, other_config) {
        println!("{}", e);
        std::process::exit(1);
    }
}
