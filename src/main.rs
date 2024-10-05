use std::{
    env,
    fmt::{self, Display, Formatter},
};

use anyhow::Context;
use image::{
    imageops::{self},
    DynamicImage, GenericImageView,
};
#[derive(Clone)]
enum AsciLevel {
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
            AsciLevel::One => write!(f, "."),
            AsciLevel::Two => write!(f, "'"),
            AsciLevel::Three => write!(f, "*"),
            AsciLevel::Four => write!(f, "o"),
            AsciLevel::Five => write!(f, "+"),
            AsciLevel::Six => write!(f, "a"),
            AsciLevel::Seven => write!(f, "&"),
            AsciLevel::Eight => write!(f, "@"),
        }
    }
}
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let Some(img_path) = args.get(1) else {
        anyhow::bail!("please provide a path to an image")
    };

    let img = image::open(img_path).context(format!("failed to open image at {img_path}."))?;

    let img = resize(img);
    let (width, height) = img.dimensions();
    let mut buffer: Vec<Vec<AsciLevel>> =
        vec![vec![AsciLevel::One; width as usize]; height as usize];

    for (width, height, pixels) in img.pixels() {
        let height = height as usize;
        let width = width as usize;
        buffer[height][width] = pick_asci(pixels.0[0])?;
    }

    for line in buffer.iter() {
        for char in line.iter() {
            print!("{}", char);
        }
        print!("\n");
    }
    Ok(())
}
fn pick_asci(pixel_value: u8) -> anyhow::Result<AsciLevel> {
    match pixel_value {
        dot if dot >=0 && dot <= 31 => {
            return Ok(AsciLevel::One);
        }
        dot if dot > 31 && dot <= 62 => {
            return Ok(AsciLevel::Two);
        }
        dot if dot >= 62 && dot <= 93 => {
            return Ok(AsciLevel::Three);
        }
        dot if dot >= 93 && dot <= 124 => {
            return Ok(AsciLevel::Four);
        }
        dot if dot >= 124 && dot <= 155 => {
            return Ok(AsciLevel::Five);
        }
        dot if dot >= 155 && dot <= 186 => {
            return Ok(AsciLevel::Six);
        }
        dot if dot >= 186 && dot <= 217 => {
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
fn resize(img: DynamicImage) -> DynamicImage {
    let img = img.grayscale();
    let (mut width, mut height) = img.dimensions();

    let ratio = (width / height) as f32;
    let base = width / 7;
    if ratio > 1.0 {
        width = base;
        height = base * ratio as u32;
    } else {
        height = base;
        width = base * ratio as u32;
    }

    let img = img.resize(width, height, imageops::Gaussian);
    return img;
}
