pub mod uxnasm {
    use clap::Parser;

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



}
