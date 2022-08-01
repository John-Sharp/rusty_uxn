use crate::emulators::uxn::device::{Device, MainRamInterface};
use std::collections::HashMap;

pub trait UxnSystemScreenInterface {
    fn get_system_colors(&self, colors: &mut [u8; 6]) -> bool;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UxnColorIndex {
    Zero,
    One,
    Two,
    Three,
}

impl TryFrom<u8> for UxnColorIndex {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UxnColorIndex::Zero),
            1 => Ok(UxnColorIndex::One),
            2 => Ok(UxnColorIndex::Two),
            3 => Ok(UxnColorIndex::Three),
            _ => Err("color indicies only exist for values 0, 1, 2, 3")
        }
    }
}

struct Layer {
    pixels: Vec<Vec<UxnColorIndex>>,
}

impl Layer {
    fn new(dimensions: &[u16; 2]) -> Self {
        Layer {
            pixels: vec![vec![UxnColorIndex::Zero; usize::from(dimensions[0])]; usize::from(dimensions[1])],
        }
    }

    fn set_pixel(&mut self, coordinate: &[u16; 2], color: UxnColorIndex) -> bool {
        if self.pixels[usize::from(coordinate[1])][usize::from(coordinate[0])] != color {
            self.pixels[usize::from(coordinate[1])][usize::from(coordinate[0])] = color;
            return true;
        }

        return false;
    }
}

pub struct ScreenDevice {
    layers: [Layer; 2],
    pixels: Vec<u8>,
    dim: [[u8; 2]; 2],
    auto_byte: u8,
    changed: bool,
    vector: [u8; 2],
    target_location: [[u8; 2]; 2],
    last_pixel_value: u8,
    system_colors_raw: [u8; 6],
    system_colors: HashMap<UxnColorIndex, [u8; 3]>,
    sprite_repeat: u8,
    auto_inc_address: bool,
    auto_inc_x: bool,
    auto_inc_y: bool,
}

const FG: usize = 0;
const BG: usize = 1;

impl ScreenDevice {
    pub fn new(dimensions: &[u16; 2]) -> Self {
        ScreenDevice {
            layers: [Layer::new(dimensions), Layer::new(dimensions)],
            pixels: vec![0; usize::from(dimensions[0]) * usize::from(dimensions[1]) * 3],
            dim: [dimensions[0].to_be_bytes(), dimensions[1].to_be_bytes()],
            auto_byte: 0,
            changed: true,
            vector: [0; 2],
            target_location: [[0; 2], [0; 2]],
            last_pixel_value: 0,
            system_colors_raw: [0; 6],
            system_colors: HashMap::from([
                (UxnColorIndex::Zero, [0,0,0]),
                (UxnColorIndex::One, [0,0,0]),
                (UxnColorIndex::Two, [0,0,0]),
                (UxnColorIndex::Three, [0,0,0]),
            ]),
            sprite_repeat: 0,
            auto_inc_address: false,
            auto_inc_x: false,
            auto_inc_y: false,
        }
    }

    fn pixel_write(&mut self, val: u8) {
        let layer = if val & 0x40 > 0 { FG } else { BG };

        let color_index = val & 0x3;
        let color_index = UxnColorIndex::try_from(color_index).unwrap();

        let target_x = u16::from_be_bytes(
            [self.target_location[0][0], self.target_location[0][1]]);
        let target_y = u16::from_be_bytes(
            [self.target_location[1][0], self.target_location[1][1]]);

        if self.layers[layer].set_pixel(&[target_x, target_y], color_index) {
            self.changed = true;
        }

        if self.auto_inc_x {
            [self.target_location[0][0], self.target_location[0][1]] = (target_x + 1).to_be_bytes();
        }
        if self.auto_inc_y {
            [self.target_location[1][0], self.target_location[1][1]] = (target_y + 1).to_be_bytes();
        }
    }

    fn update_system_colors(&mut self) {
        *self.system_colors.get_mut(&UxnColorIndex::Zero).unwrap() = [
            (self.system_colors_raw[0] >> 4) & 0xf,
            (self.system_colors_raw[2] >> 4) & 0xf,
            (self.system_colors_raw[4] >> 4) & 0xf,
        ];

        *self.system_colors.get_mut(&UxnColorIndex::One).unwrap() = [
            (self.system_colors_raw[0]) & 0xf,
            (self.system_colors_raw[2]) & 0xf,
            (self.system_colors_raw[4]) & 0xf,
        ];

        *self.system_colors.get_mut(&UxnColorIndex::Two).unwrap() = [
            (self.system_colors_raw[1] >> 4) & 0xf,
            (self.system_colors_raw[3] >> 4) & 0xf,
            (self.system_colors_raw[5] >> 4) & 0xf,
        ];

        *self.system_colors.get_mut(&UxnColorIndex::Three).unwrap() = [
            (self.system_colors_raw[1]) & 0xf,
            (self.system_colors_raw[3]) & 0xf,
            (self.system_colors_raw[5]) & 0xf,
        ];

        for (_, val) in self.system_colors.iter_mut() {
            for component in val.iter_mut() {
                *component |= (*component)<<4;
            }
        }
    }

    // test whether the screen has changed since the last time `draw()` was called
    // and, therefore, if a call to `draw()` is necessary to render the screen. As a side effect,
    // this function looks up what the system colours are and, if they have changed, caches them.
    // Intended use for this function is to be called periodically and only to schedule a full
    // redraw (whereupon `draw()` is called) when this function returns true
    pub fn get_draw_required(&mut self,
        system: &dyn UxnSystemScreenInterface) -> bool {
        if system.get_system_colors(&mut self.system_colors_raw) {
            self.changed = true;
            self.update_system_colors();
        }

        return self.changed;
    }

    // update the internal buffer containing the pixels to be rendered. Pass a reference to this
    // buffer, and the dimensions of the screen, to `draw_fn`, which can be used to render to the
    // screen
    pub fn draw(&mut self, draw_fn: &mut dyn FnMut(&[u16; 2], &[u8])) {
        let mut fg_pixels = self.layers[FG].pixels.iter().flatten();
        let mut bg_pixels = self.layers[BG].pixels.iter().flatten();

        let mut pixel_iter = self.pixels.iter_mut();
        for (fg_pixel, bg_pixel) in fg_pixels.zip(bg_pixels) {
            let color = match fg_pixel {
                UxnColorIndex::Zero => {
                    bg_pixel
                },
                _ => fg_pixel,
            };
            let color = &self.system_colors[color];

            let screen_pixel_r = pixel_iter.next().unwrap();
            let screen_pixel_g = pixel_iter.next().unwrap();
            let screen_pixel_b = pixel_iter.next().unwrap();

            *screen_pixel_r = color[0];
            *screen_pixel_g = color[1];
            *screen_pixel_b = color[2];
        }

        let dim = [
            u16::from_be_bytes([self.dim[0][0], self.dim[0][1]]),
            u16::from_be_bytes([self.dim[1][0], self.dim[1][1]]),
        ];

        draw_fn(&dim, &self.pixels);
        self.changed = false;
    }

    fn resize(&mut self) {
        let dimensions = [
            u16::from_be_bytes([self.dim[0][0], self.dim[0][1]]),
            u16::from_be_bytes([self.dim[1][0], self.dim[1][1]]),
        ];

        self.layers = [Layer::new(&dimensions), Layer::new(&dimensions)];
        self.pixels = vec![0; usize::from(dimensions[0]) * usize::from(dimensions[1]) * 3];
        self.changed = true;
    }

    fn set_auto(&mut self, val: u8) {
        self.sprite_repeat = val >> 4;
        self.auto_inc_address = if (val & 0x04) == 1 { true } else { false };
        self.auto_inc_x = if (val & 0x01) != 0 { true } else { false };
        self.auto_inc_y = if (val & 0x02) != 0 { true } else { false };
    }
}

impl Device for ScreenDevice {
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
            0x2 => {
                self.dim[0][0] = val;
            },
            0x3 => {
                self.dim[0][1] = val;
                self.resize();
            },
            0x4 => {
                self.dim[1][0] = val;
            },
            0x5 => {
                self.dim[1][1] = val;
                self.resize();
            },
            0x6 => {
                self.auto_byte = val;
                self.set_auto(val);
            },
            0x8 => {
                self.target_location[0][0] = val;
            },
            0x9 => {
                self.target_location[0][1] = val;
            },
            0xa => {
                self.target_location[1][0] = val;
            },
            0xb => {
                self.target_location[1][1] = val;
            },
            0xe => {
                self.last_pixel_value = val;
                self.pixel_write(val);
            },
            _ => {}
        }
    }

    fn read(&mut self, port: u8) -> u8 {
        match port {
            0x0 => return self.vector[0],
            0x1 => return self.vector[1],
            0x2 => return self.dim[0][0],
            0x3 => return self.dim[0][1],
            0x4 => return self.dim[1][0],
            0x5 => return self.dim[1][1],
            0x6 => return self.auto_byte,
            0x8 => return self.target_location[0][0],
            0x9 => return self.target_location[0][1],
            0xa => return self.target_location[1][0],
            0xb => return self.target_location[1][1],
            0xe => return self.last_pixel_value,
            _ => {},
        }

        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulators::uxn::device::MainRamInterfaceError;
    use std::cell::RefCell;

    struct MockMainRamInterface {}
    impl MainRamInterface for MockMainRamInterface {
        fn read(&self, _address: u16, _num_bytes: u16) -> Result<Vec<u8>, MainRamInterfaceError> {
            panic!("should not be called");
        }

        fn write(&mut self, _address: u16, _bytes: &[u8]) -> Result<usize, MainRamInterfaceError> {
            panic!("should not be called");
        }
    }

    struct MockUxnSystemScreenInterface {
        system_colors_raw: [u8; 6],
    }
    impl UxnSystemScreenInterface for MockUxnSystemScreenInterface {
        fn get_system_colors(&self, colors: &mut [u8; 6]) -> bool {
            if colors == &self.system_colors_raw {
                return false;
            }

            *colors = self.system_colors_raw;
            return true;
        }
    }

    // on a screen large enough to need two shorts to describe its dimensions, draw a
    // pixel and assert that when the draw function is called the correct bitmap
    // is provided to be drawn
    #[test]
    fn test_pixel_draw() {
        let mut screen = ScreenDevice::new(&[0x1f, 0x2f]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]};

        // set location to (0x18, 0x2d)
        let target_x = u16::to_be_bytes(0x18);
        screen.write(0x8, target_x[0], &mut mock_ram_interface);
        screen.write(0x9, target_x[1], &mut mock_ram_interface);
        let target_y = u16::to_be_bytes(0x2d);
        screen.write(0xa, target_y[0], &mut mock_ram_interface);
        screen.write(0xb, target_y[1], &mut mock_ram_interface);

        // set the background to colour index 2 and paint the pixel
        let color = 0x02; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 0x1f*0x2f];
        expected_pixels[0x1f*0x2d + 0x18] = [0x22, 0x66, 0xaa];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[0x1f, 0x2f], dim);
        };

        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);
    }

    // drawing a pixel to screen, assert that calling get_draw_required only returns true if
    // something has changed
    #[test]
    fn test_pixel_write_repeated() {
        let mut screen = ScreenDevice::new(&[16, 9]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]};

        // set location to (2, 3)
        screen.write(0x9, 2, &mut mock_ram_interface);
        screen.write(0xb, 3, &mut mock_ram_interface);

        // set the background to colour index 3 and paint the pixel
        let color = 0x03; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        // on first draw, assert we get what is expected
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);

        // calling draw_if_changed with no change should mean that get_draw_required returns false
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), false);

        // set location to (0, 0) and draw a pixel colour index 1 (on foreground)
        screen.write(0x9, 0, &mut mock_ram_interface);
        screen.write(0xb, 0, &mut mock_ram_interface);
        let color = 0x41; 
        screen.write(0xe, color, &mut mock_ram_interface);

        // now that something has changed, draw_fn should be called with new bitmap
        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        expected_pixels[16*0 + 0] = [0x11, 0x55, 0x99];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);
    }

    // test that changing system colors counts as a change in get_draw_required
    #[test]
    fn test_system_color_change() {
        let mut screen = ScreenDevice::new(&[16, 9]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]};

        // set location to (2, 3)
        screen.write(0x9, 2, &mut mock_ram_interface);
        screen.write(0xb, 3, &mut mock_ram_interface);

        // set the background to colour index 3 and paint the pixel
        let color = 0x03; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        // on first draw, assert we get what is expected
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true); 
        screen.draw(&mut draw_fn);

        // change the system colors
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54]};

        let mut expected_pixels = vec![[0xff_u8, 0xbb_u8, 0x77_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0xcc, 0x88, 0x44];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();
        let called = RefCell::new(false);
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
            *called.borrow_mut() = true;
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true); 
        screen.draw(&mut draw_fn);
        assert_eq!(*called.borrow(), true);
    }

    // test that foreground pixels are drawn over background if foreground pixel is anything other
    // than index 0
    #[test]
    fn test_foreground_background() {
        let mut screen = ScreenDevice::new(&[16, 9]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]};

        // set location to (2, 3)
        screen.write(0x9, 2, &mut mock_ram_interface);
        screen.write(0xb, 3, &mut mock_ram_interface);

        // set the background to colour index 3 and paint the pixel
        let color = 0x03; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        // on first draw, assert we get what is expected
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);

        // set the foreground to color index 1 and paint the pixel
        let color = 0x41; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x11, 0x55, 0x99];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        // assert that now foreground is drawn over the background
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);

        // set foreground to color index 0 so that background should show
        // through again
        let color = 0x40; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);
    }

    // test that after a screen dimension change, the screen is cleared
    #[test]
    fn test_dimension_change() {
        let mut screen = ScreenDevice::new(&[16, 9]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]};

        // set location to (2, 3)
        screen.write(0x9, 2, &mut mock_ram_interface);
        screen.write(0xb, 3, &mut mock_ram_interface);

        // set the background to colour index 3 and paint the pixel
        let color = 0x03; 
        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        // on first draw, assert we get what is expected
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);

        // change the width dimension
        let new_width = 12_u16;
        let new_width_bytes = new_width.to_be_bytes();
        screen.write(0x2, new_width_bytes[0], &mut mock_ram_interface);
        screen.write(0x3, new_width_bytes[1], &mut mock_ram_interface);

        // screen should now be of new dimension, and blank
        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 12*9];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();
        let called = RefCell::new(false);
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[12, 9], dim);
            *called.borrow_mut() = true;
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);
        assert_eq!(*called.borrow(), true);

        // change the height dimension
        let new_height = 4_u16;
        let new_height_bytes = new_height.to_be_bytes();
        screen.write(0x4, new_height_bytes[0], &mut mock_ram_interface);
        screen.write(0x5, new_height_bytes[1], &mut mock_ram_interface);

        // screen should now be of new dimension, and blank
        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 12*4];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();
        let called = RefCell::new(false);
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[12, 4], dim);
            *called.borrow_mut() = true;
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);
        assert_eq!(*called.borrow(), true);
    }

    // test the auto flag for incrementing x and y coordinates when pixel painting
    #[test]
    fn test_auto_inc_pixel() {
        let mut screen = ScreenDevice::new(&[16, 9]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            system_colors_raw: [0x01, 0x23, 0x45, 0x67, 0x89, 0xab]};

        // set location to (2, 3)
        screen.write(0x9, 2, &mut mock_ram_interface);
        screen.write(0xb, 3, &mut mock_ram_interface);

        // set the auto byte to increment x
        screen.write(0x6, 0x1, &mut mock_ram_interface);

        // set the background to colour index 3 and paint the pixel
        let color = 0x03; 
        screen.write(0xe, color, &mut mock_ram_interface);

        // x coordinate should be incremented by 1
        let new_x = u16::from_be_bytes([screen.read(0x8), screen.read(0x9)]);
        let new_y = u16::from_be_bytes([screen.read(0xa), screen.read(0xb)]);
        assert_eq!([new_x, new_y], [3, 3]);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        // on first draw, assert we get what is expected
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);

        // set the foreground to colour index 1 and paint the pixel
        let color = 0x11; 
        screen.write(0xe, color, &mut mock_ram_interface);

        // pixel that is painted should have x coordinate incremented by 1
        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        expected_pixels[16*3 + 3] = [0x11, 0x55, 0x99];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);

        // set the auto byte to increment x and y simulataneously
        screen.write(0x6, 0x3, &mut mock_ram_interface);
        // set the foreground to colour index 1 and paint the pixel (location will be 4,3)
        let color = 0x11; 

        let new_x = u16::from_be_bytes([screen.read(0x8), screen.read(0x9)]);
        let new_y = u16::from_be_bytes([screen.read(0xa), screen.read(0xb)]);
        assert_eq!([new_x, new_y], [4, 3]);

        screen.write(0xe, color, &mut mock_ram_interface);

        // x and y coordinate should be both incremented by 1
        let new_x = u16::from_be_bytes([screen.read(0x8), screen.read(0x9)]);
        let new_y = u16::from_be_bytes([screen.read(0xa), screen.read(0xb)]);
        assert_eq!([new_x, new_y], [5, 4]);

        screen.write(0xe, color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 16*9];
        expected_pixels[16*3 + 2] = [0x33, 0x77, 0xbb];
        expected_pixels[16*3 + 3] = [0x11, 0x55, 0x99];
        expected_pixels[16*3 + 4] = [0x11, 0x55, 0x99];
        expected_pixels[16*4 + 5] = [0x11, 0x55, 0x99];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();
        let mut draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[16, 9], dim);
        };
        assert_eq!(screen.get_draw_required(&mock_system_screen_interface), true);
        screen.draw(&mut draw_fn);
    }
}
