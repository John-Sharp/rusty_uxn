fn main() {
    if let Err(e) = rusty_uxn::utils::spritemake::run() {
        println!("{}", e);
        std::process::exit(1);
    }
}
