use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;

use crate::ops::OpObjectFactory;
use crate::emulators::devices::console::Console;

use crate::emulators::devices::device_list_impl::{DeviceListImpl, DeviceEntry};
use std::io::Write;

use crate::emulators::uxn;

/// A rust implementation of the uxn virtual machine (without graphical display)
#[derive(Parser)]
pub struct Cli {
    /// Rom to run
    #[clap(parse(from_os_str))]
    pub rom: std::path::PathBuf,
}

pub struct Config<J: Write> {
    pub stderr_writer: J,
}

// TODO share this with uxnemu
#[derive(Debug)]
pub struct RomReadError {
    fname: String,
}

impl fmt::Display for RomReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error opening ROM: {}", self.fname)
    }
}

impl Error for RomReadError {}

pub fn run<J: Write>(cli_config: Cli, other_config: Config<J>) -> Result<(), Box<dyn Error>> {
    let rom = match File::open(cli_config.rom.as_path()) {
        Ok(fp) => fp,
        Err(_err) => {
            return Err(Box::new(RomReadError {
                fname: cli_config.rom.as_path().display().to_string().clone(),
            }));
        }
    };
    let rom = BufReader::new(rom).bytes();
    let rom = rom.map(|b| b.unwrap());
    let instruction_factory_impl = OpObjectFactory{};

    let mut uxn = uxn::UxnImpl::new(rom, instruction_factory_impl)?;

    let mut console_device = Console::new();

    let mut device_list: HashMap::<u8, DeviceEntry<J>> = HashMap::new();
    device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(other_config.stderr_writer));
    device_list.insert(0x1, DeviceEntry::Device(&mut console_device));
    let device_list = DeviceListImpl::new(device_list);

    uxn.run(uxn::INIT_VECTOR, device_list)?;

    return Ok(());
}
