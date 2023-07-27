use anyhow::Result;
use glam::IVec2;
use image::{imageops::flip_vertical_in_place, ImageBuffer, Rgb, RgbImage};
use wavefront::Obj;

type Color = Rgb<u8>;
type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn main() -> Result<()> {
    let white = Rgb([255, 255, 255]);
    let _red = Rgb([255, 0, 0]);

    let width = 1023;
    let height = 1023;
    let mut image = RgbImage::new(width + 1, height + 1);

    let model = Obj::from_file("./obj/african_head.obj")?;

    let half_width = width as f32 / 2f32;
    let half_height = height as f32 / 2f32;
    for poly in model.polygons() {
        for i in 0..3 {
            let v0 = poly.vertex(i).unwrap().position();
            let v1 = poly.vertex((i + 1) % 3).unwrap().position();

            let x0 = (v0[0] + 1f32) * half_width;
            let y0 = (v0[1] + 1f32) * half_height;
            let v0 = IVec2::new(x0 as i32, y0 as i32);

            let x1 = (v1[0] + 1f32) * half_width;
            let y1 = (v1[1] + 1f32) * half_height;
            let v1 = IVec2::new(x1 as i32, y1 as i32);

            line(v0, v1, &mut image, &white);
        }
    }

    flip_vertical_in_place(&mut image);
    image.save_with_format("output.jpg", image::ImageFormat::Jpeg)?;

    Ok(())
}

fn line(v0: IVec2, v1: IVec2, image: &mut Image, color: &Color) {
    let (mut v0, mut v1) = (v0, v1);

    let steep = v0.x.abs_diff(v1.x) < v0.y.abs_diff(v1.y);
    // if the line is steep, we transpose the image
    if steep {
        std::mem::swap(&mut v0.x, &mut v0.y);
        std::mem::swap(&mut v1.x, &mut v1.y);
    }

    // make it left-to-right
    if v0.x > v1.x {
        std::mem::swap(&mut v0, &mut v1);
    }

    let dx = v1.x - v0.x;
    let dy = v1.y - v0.y;
    let derror2 = dy.abs() * 2;
    let slope = if v1.y > v0.y { 1 } else { -1 };
    let mut error2 = 0;
    let mut y = v0.y;

    for x in v0.x..=v1.x {
        if steep {
            image.put_pixel(y as u32, x as u32, *color);
        } else {
            image.put_pixel(x as u32, y as u32, *color);
        }

        error2 += derror2;
        if error2 > dx {
            y += slope;
            error2 -= dx * 2;
        }
    }
}
