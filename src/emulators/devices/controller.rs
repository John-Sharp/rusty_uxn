use crate::emulators::uxn::device::{Device, MainRamInterface};

pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    fn to_code(&self) -> u8 {
        match self {
            Button::A => 1,
            Button::B => 1<<1,
            Button::Select => 1<<2,
            Button::Start => 1<<3,
            Button::Up => 1<<4,
            Button::Down => 1<<5,
            Button::Left => 1<<6,
            Button::Right => 1<<7,
        }
    }
}

pub struct ControllerDevice {
    vector: [u8; 2],
    button_state: u8,
    key: u8,
}

impl ControllerDevice {
    pub fn new() -> Self {
        ControllerDevice {
            vector: [0; 2],
            button_state: 0,
            key: 0,
        }
    }

    pub fn read_vector(&self) -> u16 {
        return u16::from_be_bytes(self.vector);
    }

    pub fn notify_key_press(&mut self, key: u8) {
        self.key = key;
    }

    pub fn notify_button_down(&mut self, button: Button) -> bool {
        let button_old = self.button_state;
        self.button_state |= button.to_code();
        if button_old != self.button_state {
            return true;
        }
        return false;
    }

    pub fn notify_button_up(&mut self, button: Button) {
        self.button_state &= !button.to_code();
    }
}

impl Device for ControllerDevice {
    fn write(&mut self, port: u8, val: u8, _main_ram: &mut dyn MainRamInterface) {
        if port > 0xf {
            panic!("attempting to write to port out of range");
        }

        match port {
            0x0 => {
                self.vector[0] = val;
            },
            0x1 => {
                self.vector[1] = val;
            },
            _ => {}
        }
    }

    fn read(&mut self, port: u8) -> u8 {
        if port > 0xf {
            panic!("attempting to read from port out of range");
        }

        match port {
            0x0 => return self.vector[0],
            0x1 => return self.vector[1],
            0x2 => return self.button_state,
            0x3 => return self.key,
            _ => {
                return 0x0;
            }
        }
    }
}
