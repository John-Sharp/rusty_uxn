use clap::Parser;

fn main() {
    let args = rusty_uxn::uxnasmlib::Cli::parse();

    if let Err(e) = rusty_uxn::uxnasmlib::run(args) {
        println!("{}", e);
        std::process::exit(1);
    }
}
