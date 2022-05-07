use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;

use speedy2d::Window;
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::Graphics2D;
use speedy2d::color::Color;

use crate::ops::OpObjectFactory;
mod devices;
use devices::console::Console;

mod device_list_impl;
use device_list_impl::{DeviceListImpl, DeviceEntry};

/// A rust implementation of the uxn virtual machine
#[derive(Parser)]
pub struct Cli {
    /// Rom to run
    #[clap(parse(from_os_str))]
    pub rom: std::path::PathBuf,
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

struct MyWindowHandler {}

impl WindowHandler for MyWindowHandler
{
    fn on_draw(&mut self, _helper: &mut WindowHelper, graphics: &mut Graphics2D)
    {
        // Draw things here using `graphics`
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));
        graphics.draw_circle((100.0, 100.0), 75.0, Color::BLUE);
        println!("redrawing");
//        helper.request_redraw();
    }
}

pub mod uxn;

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let rom = match File::open(config.rom.as_path()) {
        Ok(fp) => fp,
        Err(_err) => {
            return Err(Box::new(RomReadError {
                fname: config.rom.as_path().display().to_string().clone(),
            }));
        }
    };
    let rom = BufReader::new(rom).bytes();
    let rom = rom.map(|b| b.unwrap());
    let instruction_factory_impl = OpObjectFactory{};

    let mut uxn = uxn::UxnImpl::new(rom, instruction_factory_impl)?;

    let mut console_device = Console::new();

    let mut device_list: HashMap::<u8, DeviceEntry<io::Stderr>> = HashMap::new();
    device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(io::stderr()));
    device_list.insert(0x1, DeviceEntry::Device(&mut console_device));
    let device_list = DeviceListImpl::new(device_list);

    uxn.run(uxn::INIT_VECTOR, device_list)?;

    let window = Window::new_centered("Title", (512, 320)).unwrap();
    window.run_loop(MyWindowHandler{});
}
