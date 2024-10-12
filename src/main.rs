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

    ///choose how to save the output image , as a png , to a text file or just print it to the
    ///terminal.
    #[arg(value_enum , default_value_t = OutputType::Ter ) ]
    output: OutputType,
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputType {
    ///print in terminal (DEFAULT)
    Ter,
    ///png with a transparent background.
    Img,
    ///text file.
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

impl AsciLevel {
    fn ratio(&self) -> f32 {
        match self {
            AsciLevel::Empty => 0.0,
            AsciLevel::One => 0.31,
            AsciLevel::Two => 0.62,
            AsciLevel::Three => 0.93,
            AsciLevel::Four => 0.124,
            AsciLevel::Five => 0.155,
            AsciLevel::Six => 0.188,
            AsciLevel::Seven => 0.217,
            AsciLevel::Eight => 0.1,
        }
    }
}
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let img = image::open(&cli.img_path).context(format!(
        "failed to open image at {}.",
        cli.img_path.display()
    ))?;

    let f = File::open(cli.img_path)?;

    match cli.output {
        OutputType::Ter => convert_to_asci_terminal(img)?,
        OutputType::Img => convert_to_asci_img(img)?,
        OutputType::Txt => convert_to_asci_file(img)?,
    };

    Ok(())
}
fn pixel_to_asci(pixel_value: u8) -> anyhow::Result<AsciLevel> {
    match pixel_value {
        0 => Ok(AsciLevel::Empty),
        1..=31 => Ok(AsciLevel::One),
        32..=62 => Ok(AsciLevel::Two),
        63..=93 => Ok(AsciLevel::Three),
        94..=124 => Ok(AsciLevel::Four),
        125..=155 => Ok(AsciLevel::Five),
        156..=186 => Ok(AsciLevel::Six),
        187..=217 => Ok(AsciLevel::Seven),
        218..=255 => Ok(AsciLevel::Eight),
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
    let height = height * line_height;
    let mut asci_img = RgbaImage::new(width, height);

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
                    let x = (x as i32 + bb.min.x).max(0) as u32;
                    let y = (y as i32 + bb.min.y).max(0) as u32;

                    if x < width && y < height {
                        asci_img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
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
    return image::DynamicImage::from(image::imageops::resize(&img, 100, 50, imageops::FilterType::Nearest));
    }
    return image::DynamicImage::from(image::imageops::resize(&img, width, height, imageops::FilterType::Lanczos3));

}
