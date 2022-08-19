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
use crate::uxninterface::UxnStatus;

use speedy2d::Window;
use speedy2d::window::{WindowHandler, WindowHelper, WindowStartupInfo, WindowCreationOptions, WindowSize,
    MouseButton, MouseScrollDistance};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::dimen::Vector2;
use speedy2d::image::{ImageDataType, ImageSmoothingMode};

use crate::ops::OpObjectFactory;
use crate::emulators::devices::{console::Console, file::FileDevice, datetime::DateTimeDevice, screen::ScreenDevice,
    mouse::MouseDevice, controller::ControllerDevice};
use crate::emulators::devices::mouse;

use crate::emulators::devices::device_list_impl::{DeviceListImpl, DeviceEntry};
use std::io::Write;

use crate::instruction;
use crate::emulators::uxn;


const INITIAL_DIMENSIONS: [u16; 2] = [64*8, 40*8];

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

struct EmuDevices<J: Write, K: Write, M: Write> {
    console_device: Console<J, K>,
    file_device: FileDevice,
    datetime_device: DateTimeDevice,
    debug_writer: M,
    screen_device: ScreenDevice,
    mouse_device: MouseDevice,
    controller_device: ControllerDevice,
}

fn construct_device_list<J: Write, K: Write, M: Write>(devices: &mut EmuDevices<J, K, M>) -> DeviceListImpl<'_, &mut M> {
    let mut device_list: HashMap::<u8, DeviceEntry<&mut M>> = HashMap::new();
    device_list.insert(0x0, DeviceEntry::SystemPlaceHolder(&mut devices.debug_writer));
    device_list.insert(0x1, DeviceEntry::Device(&mut devices.console_device));
    device_list.insert(0x2, DeviceEntry::Device(&mut devices.screen_device));
    device_list.insert(0x8, DeviceEntry::Device(&mut devices.controller_device));
    device_list.insert(0x9, DeviceEntry::Device(&mut devices.mouse_device));
    device_list.insert(0xa, DeviceEntry::Device(&mut devices.file_device));
    device_list.insert(0xc, DeviceEntry::Device(&mut devices.datetime_device));
    let device_list = DeviceListImpl::new(device_list);
    return device_list;
}

enum UxnEvent {
    ScreenRefresh,
}

struct MyWindowHandler<J: instruction::InstructionFactory, K: Write, L: Write, M: Write> {
    uxn: uxn::UxnImpl<J>,
    devices: EmuDevices<K, L, M>,
}


impl<J: instruction::InstructionFactory, K: Write, L: Write, M: Write> MyWindowHandler<J, K, L, M> {
    fn execute_vector(&mut self, vector: u16, helper: &mut WindowHelper<UxnEvent>) {
        let res = self.uxn.run(vector, construct_device_list(&mut self.devices));

        match res {
            Ok(UxnStatus::Terminate) => {
                // gracefully close
                helper.terminate_loop();
            },
            Ok(UxnStatus::Halt) => {
                // continue rendering the screen
            },
            Err(e) => {
                println!("{}", e);
                helper.terminate_loop();
            },
        }
    }
}


impl<J: instruction::InstructionFactory, K: Write, L: Write, M: Write>  WindowHandler<UxnEvent> for MyWindowHandler<J, K, L, M>
{
    fn on_draw(&mut self, _helper: &mut WindowHelper<UxnEvent>, graphics: &mut Graphics2D)
    {
        // Draw things here using `graphics`
        let mut draw_fn = |size: &[u16; 2], pixels: &[u8]| {
            let size = Vector2::new(size[0] as u32, size[1] as u32);
            
            let image_handle = graphics.create_image_from_raw_pixels(
                ImageDataType::RGB,
                ImageSmoothingMode::NearestNeighbor,
                size,
                pixels
                ).unwrap();
            graphics.clear_screen(Color::from_rgb(0.0, 0.0, 0.0));
            graphics.draw_image((0.0, 0.0), &image_handle);
        };

        self.devices.screen_device.draw(&mut draw_fn);
    }

    fn on_start(&mut self, helper: &mut WindowHelper<UxnEvent>, _info: WindowStartupInfo) {
        helper.set_cursor_visible(false);

        self.execute_vector(uxn::INIT_VECTOR, helper);
    }

    fn on_user_event(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        user_event: UxnEvent
    ) {
        match user_event {
            UxnEvent::ScreenRefresh => {
                if self.devices.screen_device.get_draw_required(&self.uxn) {
                    helper.request_redraw();
                }
            },
        }
    }

    fn on_mouse_move(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        position: Vector2<f32>
    ) {
        let x = position.x as u16;
        let y = position.y as u16;
        self.devices.mouse_device.notify_cursor_position(&[x, y]);

        let mouse_vector = self.devices.mouse_device.read_vector();
        self.execute_vector(mouse_vector, helper);
    }

    fn on_mouse_button_down(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        button: MouseButton
    ) {
        let button = if let Some(button) = convert_button_to_device_button(button) {
            button
        } else {
            return;
        };

        self.devices.mouse_device.notify_button_down(button);

        let mouse_vector = self.devices.mouse_device.read_vector();
        self.execute_vector(mouse_vector, helper);
    }

    fn on_mouse_button_up(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        button: MouseButton
    ) {
        let button = if let Some(button) = convert_button_to_device_button(button) {
            button
        } else {
            return;
        };

        self.devices.mouse_device.notify_button_up(button);

        let mouse_vector = self.devices.mouse_device.read_vector();
        self.execute_vector(mouse_vector, helper);
    }

    fn on_mouse_wheel_scroll(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        distance: MouseScrollDistance
    ) {
        let (x, y) = match distance {
            MouseScrollDistance::Lines{x, y, ..} => (x, y),
            _ => { return; }
        };

        // casting down from f64 to i16 could lead to overflow, but in practise
        // the numbers for mouse scroll distance are small
        self.devices.mouse_device.notify_scroll(&[x as i16, y as i16]);

        let mouse_vector = self.devices.mouse_device.read_vector();
        self.execute_vector(mouse_vector, helper);
    }

    fn on_keyboard_char(
        &mut self,
        helper: &mut WindowHelper<UxnEvent>,
        unicode_codepoint: char
    ) {
        if (unicode_codepoint.is_ascii()) {
            self.devices.controller_device.notify_key_press(unicode_codepoint as u8);
        }

        let controller_vector = self.devices.controller_device.read_vector();
        self.execute_vector(controller_vector, helper);
    }
}

fn convert_button_to_device_button(button: MouseButton) -> Option<mouse::Button> {
    match button {
        MouseButton::Left => Some(mouse::Button::Left),
        MouseButton::Right => Some(mouse::Button::Right),
        MouseButton::Middle => Some(mouse::Button::Middle),
        _ => None,
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

    let file_device = FileDevice::new();
    let datetime_device = DateTimeDevice::new();
    let screen_device = ScreenDevice::new(&INITIAL_DIMENSIONS);
    let mouse_device = MouseDevice::new();
    let controller_device = ControllerDevice::new();
    let emu_devices = EmuDevices{
        console_device, file_device, datetime_device, debug_writer: other_config.stderr_writer,
        screen_device, mouse_device, controller_device};

    let window_creation_options = WindowCreationOptions::new_windowed(WindowSize::PhysicalPixels(Vector2::new(INITIAL_DIMENSIONS[0].into(), INITIAL_DIMENSIONS[1].into())), None);
    let window_creation_options = window_creation_options.with_resizable(false);

    let window = Window::<UxnEvent>::new_with_user_events(
        "Title",
        window_creation_options).unwrap();

    let window_refresh_event_sender = window.create_user_event_sender();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(17));
            window_refresh_event_sender.send_event(UxnEvent::ScreenRefresh).unwrap();
        }
    });

    window.run_loop(MyWindowHandler{
        uxn, devices: emu_devices
    });
}
