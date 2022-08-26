use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;

use crate::uxninterface::UxnStatus;
use crate::ops::OpObjectFactory;
use crate::emulators::devices::console::Console;
use crate::emulators::devices::file::FileDevice;
use crate::emulators::devices::datetime::DateTimeDevice;
use crate::emulators::RomReadError;

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
    pub input: Vec<String>,
}

pub struct Config<J: Write, K: Read, L: Write, M: Write> {
    pub stdout_writer: J, // used by console device for stdout
    pub stdin_reader: K,  // used for reading console input and passing on to console device
    pub stderr_writer: L, // used by console device for stderr
    pub debug_writer: M,  // used by system device for debug output
}

struct CliDevices<J: Write, K: Write, M: Write> {
    console_device: Console<J, K>,
    file_device: FileDevice,
    datetime_device: DateTimeDevice,
    debug_writer: M,
}

fn construct_device_list<J: Write, K: Write, M: Write>(devices: &mut CliDevices<J, K, M>) -> DeviceListImpl<'_, &mut M> {
    let mut device_list: HashMap::<u8, DeviceEntry<&mut M>> = HashMap::new();
    device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(&mut devices.debug_writer));
    device_list.insert(0x1, DeviceEntry::Device(&mut devices.console_device));
    device_list.insert(0xa, DeviceEntry::Device(&mut devices.file_device));
    device_list.insert(0xc, DeviceEntry::Device(&mut devices.datetime_device));
    let device_list = DeviceListImpl::new(device_list);
    return device_list;
}

pub fn run<J: Write, K: Read, L: Write, M: Write>(cli_config: Cli, other_config: Config<J, K, L, M>) -> Result<(), Box<dyn Error>> {
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

    let console_device = Console::new(
        other_config.stdout_writer,
        other_config.stderr_writer);

    let file_device = FileDevice::new();

    let datetime_device = DateTimeDevice::new();

    let mut cli_devices = CliDevices{
        console_device, file_device, datetime_device, debug_writer: other_config.debug_writer};

    // initial run of program
    let res = uxn.run(uxn::INIT_VECTOR, construct_device_list(&mut cli_devices))?;

    match res {
        UxnStatus::Terminate => { return Ok(()); },
        UxnStatus::Halt => {},
    }

    // for the input given on the command line, make each byte of it, in turn, available through
    // the console device and trigger the console input vector
    for input in cli_config.input {
        for c in input.bytes().chain("\n".bytes()) { 
            cli_devices.console_device.provide_input(c);

            let console_vector = cli_devices.console_device.read_vector();
            let res = uxn.run(console_vector, construct_device_list(&mut cli_devices))?;

            match res {
                UxnStatus::Terminate => { return Ok(()); },
                UxnStatus::Halt => {},
            }
        }
    }

    // for input provided via stdin, make each byte available through the console device and
    // trigger the console input vector
    for c in other_config.stdin_reader.bytes() {
        match c {
            Ok(c) => {
                cli_devices.console_device.provide_input(c);
                let console_vector = cli_devices.console_device.read_vector();
                let res = uxn.run(console_vector, construct_device_list(&mut cli_devices))?;

                match res {
                    UxnStatus::Terminate => { return Ok(()); },
                    UxnStatus::Halt => {},
                }
            },
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    return Ok(());
}
