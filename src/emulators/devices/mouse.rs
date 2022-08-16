use crate::emulators::uxn::device::{Device, MainRamInterface};

pub struct MouseDevice {
    vector: [u8; 2],
    cursor_pos: [[u8; 2]; 2],
    scroll: [[u8; 2]; 2],
    click_state: u8,
}

impl MouseDevice {
    pub fn new() -> Self {
        MouseDevice{
            vector: [0; 2],
            cursor_pos: [[0; 2]; 2],
            scroll: [[0; 2]; 2],
            click_state: 0,
        }
    }

    pub fn notify_cursor_position(&mut self, cursor_pos: &[u16; 2]) {
        println!("mouse device was given position ({}, {})", cursor_pos[0], cursor_pos[1]);
    }
}

impl Device for MouseDevice {
    fn write(&mut self, _port: u8, _val: u8, _main_ram: &mut dyn MainRamInterface) {
        // writing to mouse device is a no-op
    }

    fn read(&mut self, port: u8) -> u8 {
        if port > 0xf {
            panic!("attempting to read from port out of range");
        }

        match port {
            0x0 => return self.vector[0],
            0x1 => return self.vector[1],
            0x2 => return self.cursor_pos[0][0],
            0x3 => return self.cursor_pos[0][1],
            0x4 => return self.cursor_pos[1][0],
            0x5 => return self.cursor_pos[1][1],
            0x6 => return self.click_state,
            0xa => return self.scroll[0][0],
            0xb => return self.scroll[0][1],
            0xc => return self.scroll[1][0],
            0xd => return self.scroll[1][1],
            _ => {
                return 0x0;
            },
        }
    }
}
