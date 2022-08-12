use clap::Parser;

fn main() {
    let args = rusty_uxn::utils::spritemake::Cli::parse();

    if let Err(e) = rusty_uxn::utils::spritemake::run(args) {
        println!("{}", e);
        std::process::exit(1);
    }
}
