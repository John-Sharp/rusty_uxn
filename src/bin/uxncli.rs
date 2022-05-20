use clap::Parser;
use std::io;

fn main() {
    let args = rusty_uxn::emulators::uxnclilib::Cli::parse();
    let other_config = rusty_uxn::emulators::uxnclilib::Config{stderr_writer: io::stderr()};

    if let Err(e) = rusty_uxn::emulators::uxnclilib::run(args, other_config) {
        println!("{}", e);
        std::process::exit(1);
    }
}
