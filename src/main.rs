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
enum Asci {
    Dot,
    Star,
    Qoute,
    DoubleQoute,
    Plus,
    Hashtag,
    And,
    At,
}
impl Display for Asci {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Asci::Dot => write!(f, "."),
            Asci::Star => write!(f, "*"),
            Asci::Qoute => write!(f, "\'"),
            Asci::DoubleQoute => write!(f, "\""),
            Asci::Plus => write!(f, "+"),
            Asci::Hashtag => write!(f, "#"),
            Asci::And => write!(f, "&"),
            Asci::At => write!(f, "@"),
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
    let mut buffer: Vec<Vec<Asci>> = vec![vec![Asci::Dot; width as usize]; height as usize];

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
fn pick_asci(pixel_value: u8) -> anyhow::Result<Asci> {
    match pixel_value {
        dot if dot >= 0 && dot <= 31 => {
            return Ok(Asci::Dot);
        }

        dot if dot > 31 && dot <= 62 => {
            return Ok(Asci::Star);
        }
        dot if dot >= 62 && dot <= 93 => {
            return Ok(Asci::Qoute);
        }
        dot if dot >= 93 && dot <= 124 => {
            return Ok(Asci::Plus);
        }
        dot if dot >= 124 && dot <= 155 => {
            return Ok(Asci::DoubleQoute);
        }
        dot if dot >= 155 && dot <= 186 => {
            return Ok(Asci::Hashtag);
        }
        dot if dot >= 186 && dot <= 217 => {
            return Ok(Asci::And);
        }
        dot if dot >= 217 => {
            return Ok(Asci::At);
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
    let base = 150;
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
