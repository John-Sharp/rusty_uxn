use crate::emulators::uxn::device::{Device, MainRamInterface};

pub struct MouseDevice {
    vector: [u8; 2],
    cursor_pos: [[u8; 2]; 2],
    scroll: [[u8; 2]; 2],
    click_state: u8,
}

pub enum Button {
    Left,
    Middle,
    Right,
}

impl MouseDevice {
    pub fn new() -> Self {
        MouseDevice {
            vector: [0; 2],
            cursor_pos: [[0; 2]; 2],
            scroll: [[0; 2]; 2],
            click_state: 0,
        }
    }

    pub fn read_vector(&self) -> u16 {
        return u16::from_be_bytes(self.vector);
    }

    pub fn notify_cursor_position(&mut self, cursor_pos: &[u16; 2]) {
        self.cursor_pos[0] = cursor_pos[0].to_be_bytes();
        self.cursor_pos[1] = cursor_pos[1].to_be_bytes();
    }

    pub fn notify_button_down(&mut self, button: Button) {
        let mask = match button {
            Button::Left => 1,
            Button::Middle => 1 << 1,
            Button::Right => 1 << 2,
        };

        self.click_state |= mask;
    }

    pub fn notify_button_up(&mut self, button: Button) {
        let mask = match button {
            Button::Left => 1,
            Button::Middle => 1 << 1,
            Button::Right => 1 << 2,
        };

        self.click_state &= !mask;
    }

    pub fn notify_scroll(&mut self, distance: &[i16; 2]) {
        self.scroll[0] = distance[0].to_be_bytes();
        self.scroll[1] = distance[1].to_be_bytes();
    }
}

impl Device for MouseDevice {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulators::uxn::device::MainRamInterfaceError;

    struct MockMainRamInterface {}
    impl MainRamInterface for MockMainRamInterface {
        fn read(&self, _address: u16, _num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError> {
            panic!("should not be called");
        }

        fn write(&mut self, _address: u16, _bytes: &[u8]) -> Result<usize, MainRamInterfaceError> {
            panic!("should not be called");
        }
    }

    #[test]
    fn test_set_get_vector() {
        let mut mouse_device = MouseDevice::new();

        let initial_vector = mouse_device.read_vector();
        assert_eq!(initial_vector, 0);

        mouse_device.write(0x0, 0xab, &mut MockMainRamInterface{});
        mouse_device.write(0x1, 0xcd, &mut MockMainRamInterface{});

        let vector = mouse_device.read_vector();
        assert_eq!(vector, 0xabcd);

        assert_eq!(mouse_device.read(0x0), 0xab);
        assert_eq!(mouse_device.read(0x1), 0xcd);
    }

    #[test]
    fn test_set_get_cursor_position() {
        let mut mouse_device = MouseDevice::new();

        mouse_device.notify_cursor_position(&[123, 65535]);

        assert_eq!(mouse_device.read(0x2), 0x00);
        assert_eq!(mouse_device.read(0x3), 0x7b);

        assert_eq!(mouse_device.read(0x4), 0xff);
        assert_eq!(mouse_device.read(0x5), 0xff);
    }

    #[test]
    fn test_set_get_click_state() {
        let mut mouse_device = MouseDevice::new();

        mouse_device.notify_button_down(Button::Left);
        mouse_device.notify_button_down(Button::Right);

        assert_eq!(mouse_device.read(0x6), 1 | (1<<2));

        mouse_device.notify_button_up(Button::Right);
        assert_eq!(mouse_device.read(0x6), 1);
    }

    #[test]
    fn test_set_get_scroll() {
        let mut mouse_device = MouseDevice::new();

        mouse_device.notify_scroll(&[2, -1]);

        assert_eq!(mouse_device.read(0xa), 0x00);
        assert_eq!(mouse_device.read(0xb), 0x02);

        assert_eq!(mouse_device.read(0xc), 0xff);
        assert_eq!(mouse_device.read(0xd), 0xff);
    }
}
