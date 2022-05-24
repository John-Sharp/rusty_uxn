use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;

use crate::uxninterface::UxnStatus;
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

    /// Initial console input for uxn virtual machine
    pub input: String,
}

pub struct Config<J: Write, K: Write, L: Write> {
    pub stdout_writer: J, // used by console device for stdout
    pub stderr_writer: K, // used by console device for stderr
    pub debug_writer: L,  // used by system device for debug output
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

pub fn run<J: Write, K: Write, L: Write>(cli_config: Cli, mut other_config: Config<J, K, L>) -> Result<(), Box<dyn Error>> {
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

    let mut console_device = Console::new(
        other_config.stdout_writer,
        other_config.stderr_writer);

    // initial run of program
    {
        let mut device_list: HashMap::<u8, DeviceEntry<&mut L>> = HashMap::new();
        device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(&mut other_config.debug_writer));
        device_list.insert(0x1, DeviceEntry::Device(&mut console_device));
        let device_list = DeviceListImpl::new(device_list);

        let res = uxn.run(uxn::INIT_VECTOR, device_list)?;

        match res {
            UxnStatus::Terminate => { return Ok(()); },
            UxnStatus::Halt => {},
        }
    }

    // for the input given on the command line, make each byte of it, in turn, available through
    // the console device and trigger the console input vector
    for c in cli_config.input.bytes() {
        console_device.provide_input(c);
        let console_vector = console_device.read_vector();
        let mut device_list: HashMap::<u8, DeviceEntry<&mut L>> = HashMap::new();
        device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(&mut other_config.debug_writer));
        device_list.insert(0x1, DeviceEntry::Device(&mut console_device));
        let device_list = DeviceListImpl::new(device_list);

        let res = uxn.run(console_vector, device_list)?;

        match res {
            UxnStatus::Terminate => { return Ok(()); },
            UxnStatus::Halt => {},
        }
    }

    return Ok(());
}
