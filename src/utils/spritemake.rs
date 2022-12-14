use std::error::Error;
use std::fmt;
use image::io::Reader as ImageReader;
use image::Rgb;
use std::collections::HashMap;
use clap::Parser;

/// Helper utility for creating sprite drawing routines for uxn based virtual machines
#[derive(Parser)]
pub struct Cli {
    /// Path to image file to be used as basis of sprite
    #[clap(parse(from_os_str))]
    pub img_path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct SpriteImgDimError {
    w: u32,
    h: u32,
}

impl fmt::Display for SpriteImgDimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sprite image of incorrect dimensions, width should be multiple of 8, height should be one more than multiple of 8  given dimensions: ({}, {})", self.w, self.h)
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

fn print_preamble(name: &str, w: u32, h: u32) {
    println!("@paint-{} (y x -- )", name);
    println!("");
    println!(".Screen/x DEO2 .Screen/y DEO2");
    println!("#{}6 .Screen/auto DEO", w-1);
    println!(";{}sprite .Screen/addr DEO2", name);

    let mut deo_vec = Vec::<String>::new();
    for _ in 0..(h-1) {
        deo_vec.push("DEOk".to_owned());
    }
    deo_vec.push("DEO".to_owned());
    println!("#c5 .Screen/sprite {}", deo_vec.join(" "));
    println!("");
    println!("JMP2r");
    println!("");
    println!("@{}sprite", name);
}

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {

    let sprite_img = ImageReader::open(config.img_path.as_path())?.decode()?.to_rgb8();

    let sprite_name = config.img_path.as_path().file_stem().unwrap().to_str().unwrap();

    if sprite_img.width() % 8 != 0 || (sprite_img.height()-1) %8 != 0 {
        return Err(Box::new(SpriteImgDimError{w: sprite_img.width(), h: sprite_img.height()}));
    }

    print_preamble(sprite_name, sprite_img.width()/8, (sprite_img.height()-1)/8);

    let color_map = ColorMap::new(&[
        *sprite_img.get_pixel(0, 0),        
        *sprite_img.get_pixel(1, 0),        
        *sprite_img.get_pixel(2, 0),        
        *sprite_img.get_pixel(3, 0),        
    ]);

    let row_size : usize = (sprite_img.width()/8).try_into().unwrap();

    let mut lower_bytes_row = vec![[0u8; 8]; row_size];
    let mut upper_bytes_row = vec![[0u8; 8]; row_size];

    let mut row_iter = sprite_img.enumerate_rows();
    // just discard the first row, since that contains info about the index colors that has already
    // been read
    row_iter.next();

    for (y, r) in row_iter {
        let y = y-1; // adjust y coordinate to forget about the first row

        for (x, _, p) in r {
            let (upper_bit, lower_bit) = color_map.get_color_index(&p)?;

            let x_i = (x/8) as usize;
            let y_i = (y%8) as usize;

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
    }


    return Ok(());
}
