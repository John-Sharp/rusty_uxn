use crate::emulators::uxn::device::{Device, MainRamInterface};
use std::collections::HashMap;

pub trait UxnSystemScreenInterface {
    fn get_system_colors(&self, colors: &mut [u8; 6]) -> bool;
}

#[derive(Clone, PartialEq, Eq, Hash)]
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
    changed: bool,
    vector: [u8; 2],
    target_location: [[u8; 2]; 2],
    system_colors_raw: [u8; 6],
    system_colors: HashMap<UxnColorIndex, [u8; 3]>,
}

const FG: usize = 0;
const BG: usize = 1;

impl ScreenDevice {
    fn new(dimensions: &[u16; 2]) -> Self {
        ScreenDevice {
            layers: [Layer::new(dimensions), Layer::new(dimensions)],
            pixels: vec![0; usize::from(dimensions[0]) * usize::from(dimensions[1]) * 3],
            dim: [dimensions[0].to_be_bytes(), dimensions[1].to_be_bytes()],
            changed: true,
            vector: [0; 2],
            target_location: [[0; 2], [0; 2]],
            system_colors_raw: [0; 6],
            system_colors: HashMap::from([
                (UxnColorIndex::Zero, [0,0,0]),
                (UxnColorIndex::One, [0,0,0]),
                (UxnColorIndex::Two, [0,0,0]),
                (UxnColorIndex::Three, [0,0,0]),
            ]),
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

    fn draw_if_changed(&mut self,
                       system: &dyn UxnSystemScreenInterface,
                       draw_fn: &dyn Fn(&[u16; 2], &[u8])) {
        if system.get_system_colors(&mut self.system_colors_raw) {
            self.changed = true;
            self.update_system_colors();
        }

        if self.changed {
            let mut fg_pixels = self.layers[FG].pixels.iter().flatten();
            let mut bg_pixels = self.layers[BG].pixels.iter().flatten();

            let mut i = 0;

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
                i+=1;
            }

            let dim = [
                u16::from_be_bytes([self.dim[0][0], self.dim[0][1]]),
                u16::from_be_bytes([self.dim[1][0], self.dim[1][1]]),
            ];

            draw_fn(&dim, &self.pixels);
        }
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
                // TODO resize screen
            },
            0x4 => {
                self.dim[1][0] = val;
            },
            0x5 => {
                self.dim[1][1] = val;
                // TODO resize screen
            },
            0x6 => {
                // TODO save as auto value
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
                self.pixel_write(val);
            },
            _ => {}
        }
    }

    fn read(&mut self, port: u8) -> u8 {
        match port {
            _ => {},
        }

        return 0;
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

    struct MockUxnSystemScreenInterface<F: Fn(&mut [u8; 6]) -> bool> {
        get_system_colors_inner: F,
    }
    impl<F: Fn(&mut [u8; 6]) -> bool> UxnSystemScreenInterface for MockUxnSystemScreenInterface<F> {
        fn get_system_colors(&self, colors: &mut [u8; 6]) -> bool {
            (self.get_system_colors_inner)(colors)
        }
    }

    #[test]
    fn test_create() {
        let screen = ScreenDevice::new(&[64*8, 40*8]);

        assert_eq!(screen.pixels.len(), 64*8*40*8*3);
    }

    // TODO write test for writing pixel
    #[test]
    fn test_pixel_write() {
        let mut screen = ScreenDevice::new(&[0x1f, 0x2f]);
        let mut mock_ram_interface = MockMainRamInterface{};
        let mock_system_screen_interface = MockUxnSystemScreenInterface{
            get_system_colors_inner: |colors| {
                let system_colors = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab];
                if colors == &system_colors {
                    return false;
                }

                *colors = system_colors;
                return true;
            }
        };

        // set location to (0x18, 0x2d)
        let target_x = u16::to_be_bytes(0x18);
        screen.write(0x8, target_x[0], &mut mock_ram_interface);
        screen.write(0x9, target_x[1], &mut mock_ram_interface);
        let target_y = u16::to_be_bytes(0x2d);
        screen.write(0xa, target_y[0], &mut mock_ram_interface);
        screen.write(0xb, target_y[1], &mut mock_ram_interface);

        // set the background to colour index 2 and paint the pixel
        let color = 0x02; 
        screen.write(0xe,color, &mut mock_ram_interface);

        let mut expected_pixels = vec![[0x00_u8, 0x44_u8, 0x88_u8]; 0x1f*0x2f];
        expected_pixels[0x1f*0x2d + 0x18] = [0x22, 0x66, 0xaa];
        let expected_pixels = expected_pixels
            .into_iter().flatten().collect::<Vec<_>>();

        let draw_fn = |dim: &[u16; 2], pixels: &[u8]| {
            assert_eq!(pixels, &expected_pixels);
            assert_eq!(&[0x1f, 0x2f], dim);
        };
        screen.draw_if_changed(&mock_system_screen_interface, &draw_fn);

        // TODO try drawing again and assert draw_fn not called
        // TODO make another change and assert draw_fn called
    }
}
