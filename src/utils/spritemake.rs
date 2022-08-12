use std::error::Error;
use std::fmt;
use image::io::Reader as ImageReader;
use image::Rgb;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SpriteImgDimError {
    w: u32,
    h: u32,
}

impl fmt::Display for SpriteImgDimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sprite image of incorrect dimensions, width and height should be multiples of 8, given dimensions: ({}, {})", self.w, self.h)
    }
}
impl Error for SpriteImgDimError {}

#[derive(Debug)]
pub struct ColorNotRecognisedError {
    r: u8,
    g: u8,
    b: u8,
}
impl fmt::Display for ColorNotRecognisedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color not recognised as one of 4 indexed colors: Rgb({}, {}, {})",
        self.r, self.g, self.b)
    }
}
impl Error for ColorNotRecognisedError {}

struct ColorMap {
    map: HashMap<Rgb<u8>, (bool, bool)>,
}
impl ColorMap {
    fn new(colors: &[Rgb<u8>; 4]) -> Self {
        ColorMap{
            map: HashMap::from([
                                (colors[0], (false, false)),
                                (colors[1], (false, true)),
                                (colors[2], (true, false)),
                                (colors[3], (true, true)),
            ])
        }
    }

    fn get_color_index(&self, p: &Rgb<u8>) -> Result<(bool, bool), ColorNotRecognisedError> {
        match self.map.get(p) {
            Some(v) => Ok(*v),
            None => Err(ColorNotRecognisedError{r: p.0[0], g: p.0[1], b: p.0[2]}),
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {

    let sprite_img = ImageReader::open("rabbit.png")?.decode()?.to_rgb8();

    if sprite_img.width() % 8 != 0 || sprite_img.height() %8 != 0 {
        return Err(Box::new(SpriteImgDimError{w: sprite_img.width(), h: sprite_img.height()}));
    }

    let color_map = ColorMap::new(&[
        Rgb([0x46, 0xa0, 0x13]),
        Rgb([0xc0, 0xbe, 0xa7]),
        Rgb([0x09, 0x00, 0x53]),
        Rgb([0xf4, 0xf6, 0x31]),
    ]);

    let row_size : usize = (sprite_img.width()/8).try_into().unwrap();

    let mut lower_bytes_row = vec![[0u8; 8]; row_size];
    let mut upper_bytes_row = vec![[0u8; 8]; row_size];

    for (x, y, p) in sprite_img.enumerate_pixels() {
        let (upper_bit, lower_bit) = color_map.get_color_index(&p)?;

        let x_i = (x/8) as usize;
        let y_i = (y%8) as usize;

        // println!("({}, {})", x_i, y_i);
        // println!("{} {}", sprite_img.width()/8, x_i);

        lower_bytes_row[x_i][y_i] = lower_bytes_row[x_i][y_i] << 1;
        upper_bytes_row[x_i][y_i] = upper_bytes_row[x_i][y_i] << 1;

        if lower_bit {
            lower_bytes_row[x_i][y_i] |= 1;
        }
        if upper_bit {
            upper_bytes_row[x_i][y_i] |= 1;
        }

        if y % 8 == 7 && x == sprite_img.width()-1 {
            for (lower_bytes, upper_bytes) in lower_bytes_row.iter().zip(upper_bytes_row) {
                let mut first = true;
                for lower_byte in lower_bytes {
                    if !first {
                        print!(" ");
                    }
                    first = false;
                    print!("{:02x}", lower_byte);
                }
                print!("    ");

                let mut first = true;
                for upper_byte in upper_bytes {
                    if !first {
                        print!(" ");
                    }
                    first = false;
                    print!("{:02x}", upper_byte);
                }
                println!("");
            }
            println!("");
            lower_bytes_row = vec![[0u8; 8]; row_size];
            upper_bytes_row = vec![[0u8; 8]; row_size];
        }

    }


    return Ok(());
}
