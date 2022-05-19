use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;

use speedy2d::Window;
use speedy2d::window::{WindowHandler, WindowHelper, WindowStartupInfo};
use speedy2d::Graphics2D;
use speedy2d::color::Color;

use crate::ops::OpObjectFactory;
use crate::emulators::devices::console::Console;

use crate::emulators::devices::device_list_impl::{DeviceListImpl, DeviceEntry};
use std::io::Write;

use crate::instruction;

/// A rust implementation of the uxn virtual machine
#[derive(Parser)]
pub struct Cli {
    /// Rom to run
    #[clap(parse(from_os_str))]
    pub rom: std::path::PathBuf,
}

pub struct Config<J: Write> {
    pub stderr_writer: J,
}

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

use crate::emulators::uxn;

struct MyWindowHandler<J: instruction::InstructionFactory, K: Write> {
    uxn: uxn::UxnImpl<J>,
    console_device: Console,
    stderr_writer: K,
}

impl<J: instruction::InstructionFactory, K: Write>  WindowHandler for MyWindowHandler<J, K>
{
    fn on_draw(&mut self, _helper: &mut WindowHelper, graphics: &mut Graphics2D)
    {
        // Draw things here using `graphics`
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));
        graphics.draw_circle((100.0, 100.0), 75.0, Color::BLUE);
        println!("redrawing");
//        helper.request_redraw();
    }

    fn on_start(&mut self, _helper: &mut WindowHelper, _info: WindowStartupInfo) {
        // TODO run uxn from init vector
        let mut device_list: HashMap::<u8, DeviceEntry<&mut K>> = HashMap::new();
        device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(&mut self.stderr_writer));
        device_list.insert(0x1, DeviceEntry::Device(&mut self.console_device));
        let device_list = DeviceListImpl::new(device_list);

        self.uxn.run(uxn::INIT_VECTOR, device_list);
        // start thread that sleeps for 1/60 second and then triggers an event to trigger
        // the screen draw vector
    }
}

pub fn run<J: Write + 'static>(cli_config: Cli, other_config: Config<J>) -> Result<(), Box<dyn Error>> {
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



    let window = Window::new_centered("Title", (512, 320)).unwrap();
    window.run_loop(MyWindowHandler{
        uxn, console_device, stderr_writer: other_config.stderr_writer,
    });
    return Ok(());
}
