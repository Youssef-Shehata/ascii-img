use clap::ValueEnum;
use rusttype::{point, Font, Scale};
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::Write,
    path::PathBuf,
};

use anyhow::Context;
use image::{
    imageops::{self},
    DynamicImage, GenericImageView, Rgba, RgbaImage,
};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    ///path to the img
    #[arg(short, long, value_name = "FILE", required = true)]
    img_path: PathBuf,

    #[arg(value_enum , default_value_t = OutputType::Ter ) ]
    output: OutputType,
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputType {
    ///print in terminal (DEFAULT)
    Ter,
    ///save as a png with a transparent background.
    Img,
    ///save as a text file.
    Txt,
}

#[derive(Clone)]
enum AsciLevel {
    Empty,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}
impl Display for AsciLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AsciLevel::Empty => write!(f, " "),
            AsciLevel::One => write!(f, "."),
            AsciLevel::Two => write!(f, ":"),
            AsciLevel::Three => write!(f, "-"),
            AsciLevel::Four => write!(f, "="),
            AsciLevel::Five => write!(f, "*"),
            AsciLevel::Six => write!(f, "#"),
            AsciLevel::Seven => write!(f, "%"),
            AsciLevel::Eight => write!(f, "@"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let img = image::open(&cli.img_path).context(format!(
        "failed to open image at {}.",
        cli.img_path.display()
    ))?;

    match cli.output {
        OutputType::Ter => convert_to_asci_terminal(img)?,
        OutputType::Img => convert_to_asci_img(img)?,
        OutputType::Txt => convert_to_asci_file(img)?,
    };

    Ok(())
}

fn pixel_to_asci(pixel_value: u8) -> anyhow::Result<AsciLevel> {
    match pixel_value {
        dot if dot == 0 => {
            return Ok(AsciLevel::Empty);
        }
        dot if dot <= 31 => {
            return Ok(AsciLevel::One);
        }
        dot if dot <= 62 => {
            return Ok(AsciLevel::Two);
        }
        dot if dot <= 93 => {
            return Ok(AsciLevel::Three);
        }
        dot if dot <= 124 => {
            return Ok(AsciLevel::Four);
        }
        dot if dot <= 155 => {
            return Ok(AsciLevel::Five);
        }
        dot if dot <= 186 => {
            return Ok(AsciLevel::Six);
        }
        dot if dot <= 217 => {
            return Ok(AsciLevel::Seven);
        }
        dot if dot >= 217 => {
            return Ok(AsciLevel::Eight);
        }
        _ => {
            anyhow::bail!("corrupted image");
        }
    }
}

fn convert_to_asci_file(img: DynamicImage) -> anyhow::Result<()> {
    let img = img.grayscale();
    let (width, height) = img.dimensions();
    let mut buffer: Vec<Vec<AsciLevel>> =
        vec![vec![AsciLevel::One; width as usize]; height as usize];

    for (width, height, pixels) in img.pixels() {
        let height = height as usize;
        let width = width as usize;
        buffer[height][width] = pixel_to_asci(pixels.0[0])?;
    }
    let mut f = File::create(format!("asci.txt")).context("creating file")?;

    for line in buffer.iter() {
        for char in line.iter() {
            f.write_all(format!("{}", char).as_bytes())?;
        }
        f.write(b"\n")?;
    }

    Ok(())
}
fn convert_to_asci_img(img: DynamicImage) -> anyhow::Result<()> {
    let line_height = 5;
    let img = img.grayscale();
    let (width, height) = img.dimensions();
    let mut buffer: Vec<Vec<AsciLevel>> =
        vec![vec![AsciLevel::One; width as usize]; height as usize];

    for (width, height, pixels) in img.pixels() {
        let height = height as usize;
        let width = width as usize;
        buffer[height][width] = pixel_to_asci(pixels.0[0])?;
    }
    let width = width * line_height;
    let height= height* line_height;
    let mut asci_img = RgbaImage::new(width , height );

    // Set the background to transparent
    for pixel in asci_img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    let font_data = include_bytes!("VT323/VT323-Regular.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("Error loading font");
    let scale = Scale { x: 10.0, y: 10.0 };

    let start = point(0.0, 0.0);

    for (i, line) in buffer.iter().enumerate() {
        let mut s = String::new();
        for i in line {
            s.push_str(&i.to_string());
        }
        let line_start = point(start.x, start.y + i as f32 * line_height as f32);
        for glyph in font.layout(&s, scale, line_start) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, _| {
                    let x = x + bb.min.x as u32;
                    let y = y + bb.min.y as u32;

                    if x < width && y < height {
                        asci_img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                    }
                });
            }
        }
    }
    asci_img.save("asci.png").expect("Error saving image");

    Ok(())
}
fn convert_to_asci_terminal(img: DynamicImage) -> anyhow::Result<()> {
    let img = resize(img);
    let (width, height) = img.dimensions();
    let mut buffer: Vec<Vec<AsciLevel>> =
        vec![vec![AsciLevel::One; width as usize]; height as usize];

    for (width, height, pixels) in img.pixels() {
        let height = height as usize;
        let width = width as usize;
        buffer[height][width] = pixel_to_asci(pixels.0[0])?;
    }

    for line in buffer.iter() {
        for i in line {
            print!("{i}");
        }
        println!("\n");
    }
    Ok(())
}
fn resize(img: DynamicImage) -> DynamicImage {
    let img = img.grayscale();
    let (mut width, mut height) = img.dimensions();

    let ratio = (width / height) as f32;

    let base = width / 7;
    if ratio > 1.0 {
        width = base;
        height = base * ratio as u32;
    } else if ratio < 1.0 && ratio > 0.0 {
        height = base;
        width = base * ratio as u32;
    } else {
        return img.resize(300, 320, imageops::Lanczos3);
    }

    return img.resize(width, height, imageops::Lanczos3);
}
