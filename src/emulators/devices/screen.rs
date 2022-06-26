use crate::emulators::uxn::device::{Device, MainRamInterface};

#[derive(Clone, PartialEq)]
enum UxnColorIndex {
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
}

pub struct ScreenDevice {
    layers: [Layer; 2],
    pixels: Vec<u8>,
    dim: [[u8; 2]; 2],
    changed: bool,
    vector: [u8; 2],
    target_location: [[u8; 2]; 2],
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
        }
    }

    fn pixel_write(&mut self, val: u8) {
        let layer = if val & 0x40 > 0 { FG } else { BG };

        let color_index = val & 0x3;
        let color_index = UxnColorIndex::try_from(color_index).unwrap();

        let target_x = u16::from_be_bytes(
            [self.target_location[0][0], self.target_location[0][1]]);
        let target_x = usize::from(target_x);
        let target_y = u16::from_be_bytes(
            [self.target_location[1][0], self.target_location[1][1]]);
        let target_y = usize::from(target_y);

        let target_pixel = &mut self.layers[layer].pixels[target_y][target_x];
        if *target_pixel != color_index {
            *target_pixel = color_index; 
            self.changed = true;
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

    #[test]
    fn test_create() {
        let screen = ScreenDevice::new(&[64*8, 40*8]);

        assert_eq!(screen.pixels.len(), 64*8*40*8*3);
    }

    // TODO write test for writing pixel
}
