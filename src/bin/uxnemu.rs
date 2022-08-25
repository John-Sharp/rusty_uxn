//! emu

use clap::Parser;
use std::io;

fn main() {
    let args = rusty_uxn::emulators::uxnemulib::Cli::parse();
    let other_config = rusty_uxn::emulators::uxnemulib::Config{stderr_writer: io::stderr()};

    if let Err(e) = rusty_uxn::emulators::uxnemulib::run(args, other_config) {
        println!("{}", e);
        std::process::exit(1);
    }
}
