use anyhow::Result;
use image::{Rgb, RgbImage};

fn main() -> Result<()> {
    let white = Rgb([255, 255, 255]);
    let red = Rgb([255, 0, 0]);

    let mut image = RgbImage::new(100, 100);
    image.put_pixel(52, 41, red);
    image.save_with_format("output.jpg", image::ImageFormat::Jpeg)?;
    
    Ok(())
}
