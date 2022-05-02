use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;

use speedy2d::Window;
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::Graphics2D;
use speedy2d::color::Color;


use crate::ops::OpObjectFactory;
use crate::uxnemulib::uxn::device::{Device, DeviceList, DeviceWriteReturnCode, DeviceReadReturnCode};
mod devices;
use devices::console::Console;
use crate::uxninterface::UxnError;

struct DeviceListImpl<'a> {
    list: HashMap<u8, &'a mut dyn Device>,
}

impl<'a> DeviceList for DeviceListImpl<'a> {
    fn write_to_device(&mut self, device_address: u8, val: u8) -> DeviceWriteReturnCode {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // TODO this magic number represents the system device,
        // have a better way of setting it
        if device_index == 0x0 {
            return DeviceWriteReturnCode::WriteToSystemDevice(device_port);
        }

        // look up correct device using index
        let device = match self.list.get_mut(&device_index) {
            Some(device) => device,
            None => return DeviceWriteReturnCode::Success, // TODO return unrecognised device error?
        };

        // pass port and value through to device
        device.write(device_port, val);

        return DeviceWriteReturnCode::Success;
    }

    fn read_from_device(&mut self, device_address: u8) -> DeviceReadReturnCode {
        // index of device is first nibble of device address
        let device_index = device_address >> 4;

        // port is second nibble of device address
        let device_port = device_address & 0xf;

        // TODO this magic number represents the system device,
        // have a better way of setting it
        if device_index == 0x0 {
            return DeviceReadReturnCode::ReadFromSystemDevice(device_port);
        }

        // look up correct device using index
        let device = match self.list.get_mut(&device_index) {
            Some(device) => device,
            None => return DeviceReadReturnCode::Success(Err(UxnError::UnrecognisedDevice)),
        };

        return DeviceReadReturnCode::Success(Ok(device.read(device_port)));
    }
}

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
    uxn.add_device(0x1, Box::new(Console::new()));

    let mut console_device = Console::new();

    let mut device_list: HashMap::<u8, &mut dyn Device> = HashMap::new();
    device_list.insert(0x1, &mut console_device);
    let device_list = DeviceListImpl{list: device_list};

    uxn.run(uxn::INIT_VECTOR, device_list)?;

    let window = Window::new_centered("Title", (512, 320)).unwrap();
    window.run_loop(MyWindowHandler{});
}
