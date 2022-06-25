use clap::Parser;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use speedy2d::Window;
use speedy2d::window::{WindowHandler, WindowHelper, WindowStartupInfo, WindowCreationOptions, WindowSize};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::dimen::Vector2;

use crate::ops::OpObjectFactory;
use crate::emulators::devices::console::Console;

use crate::emulators::devices::device_list_impl::{DeviceListImpl, DeviceEntry};
use std::io::Write;

use crate::instruction;
use crate::emulators::uxn;

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

enum UxnEvent {
    ScreenRefresh,
}

struct MyWindowHandler<J: instruction::InstructionFactory, K: Write, L: Write, M: Write> {
    uxn: uxn::UxnImpl<J>,
    console_device: Console<L, M>,
    debug_writer: K,
}

impl<J: instruction::InstructionFactory, K: Write, L: Write, M: Write>  WindowHandler<UxnEvent> for MyWindowHandler<J, K, L, M>
{
    fn on_draw(&mut self, _helper: &mut WindowHelper<UxnEvent>, graphics: &mut Graphics2D)
    {
        // Draw things here using `graphics`
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));
        graphics.draw_circle((100.0, 100.0), 75.0, Color::BLUE);
        println!("redrawing");
//        helper.request_redraw();
    }

    fn on_start(&mut self, _helper: &mut WindowHelper<UxnEvent>, _info: WindowStartupInfo) {
        // TODO run uxn from init vector
        let mut device_list: HashMap::<u8, DeviceEntry<&mut K>> = HashMap::new();
        device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(&mut self.debug_writer));
        device_list.insert(0x1, DeviceEntry::Device(&mut self.console_device));
        let device_list = DeviceListImpl::new(device_list);

        self.uxn.run(uxn::INIT_VECTOR, device_list);
        // start thread that sleeps for 1/60 second and then triggers an event to trigger
        // the screen draw vector
    }

    fn on_user_event(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        user_event: UxnEvent
    ) {
        match user_event {
            UxnEvent::ScreenRefresh => {
                println!("should be refreshing screen here");
            },
        }
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

    let uxn = uxn::UxnImpl::new(rom, instruction_factory_impl)?;

    let console_device = Console::new(io::stdout(), io::stderr());


    let window = Window::<UxnEvent>::new_with_user_events("Title", WindowCreationOptions::new_windowed(WindowSize::PhysicalPixels(Vector2::new(512, 320)), None)).unwrap();

    let window_refresh_event_sender = window.create_user_event_sender();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(17));
            window_refresh_event_sender.send_event(UxnEvent::ScreenRefresh).unwrap();
        }
    });

    window.run_loop(MyWindowHandler{
        uxn, console_device, debug_writer: other_config.stderr_writer,
    });
}
