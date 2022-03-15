use clap::Parser;

fn main() {
    let args = rusty_uxn::uxnemulib::Cli::parse();

    if let Err(e) = rusty_uxn::uxnemulib::run(args) {
        println!("{}", e);
        std::process::exit(1);
    }
}
