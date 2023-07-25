use std::{fs::read_to_string, path::Path};

use anyhow::{anyhow, Result};
use image::{imageops::flip_vertical_in_place, ImageBuffer, Rgb, RgbImage};

type Color = Rgb<u8>;
type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn main() -> Result<()> {
    let white = Rgb([255, 255, 255]);
    let _red = Rgb([255, 0, 0]);

    let width = 1023;
    let height = 1023;
    let mut image = RgbImage::new(width + 1, height + 1);

    let model = load_obj("./obj/african_head.obj")?;

    for face in model.faces.iter() {
        for i in 0..3 {
            let v0 = model.vertices[face[i]];
            let v1 = model.vertices[face[(i + 1) % 3]];
            let x0 = (v0[0] + 1f32) * width as f32 / 2f32;
            let y0 = (v0[1] + 1f32) * height as f32 / 2f32;
            let x1 = (v1[0] + 1f32) * width as f32 / 2f32;
            let y1 = (v1[1] + 1f32) * height as f32 / 2f32;
            line(
                x0 as i32, y0 as i32, x1 as i32, y1 as i32, &mut image, &white,
            );
        }
    }

    flip_vertical_in_place(&mut image);
    image.save_with_format("output.jpg", image::ImageFormat::Jpeg)?;

    Ok(())
}

fn line(x0: i32, y0: i32, x1: i32, y1: i32, image: &mut Image, color: &Color) {
    let (mut x0, mut x1, mut y0, mut y1) = (x0, x1, y0, y1);
    // if the line is steep, we transpose the image
    let steep = i32::abs(x0 - x1) < i32::abs(y0 - y1);
    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }

    // make it left-to-right
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let derror2 = i32::abs(dy) * 2;
    let mut error2 = 0;
    let mut y = y0;

    for x in x0..=x1 {
        if steep {
            image.put_pixel(y as u32, x as u32, *color);
        } else {
            image.put_pixel(x as u32, y as u32, *color);
        }

        error2 += derror2;
        if error2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            error2 -= dx * 2;
        }
    }
}

#[derive(Debug)]
struct Wavefront {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
}

fn load_obj(path: impl AsRef<Path>) -> Result<Wavefront> {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    let parse_face_index = |t: &str| -> Result<usize> {
        let index: usize = t.split_once('/').ok_or(anyhow!("o no"))?.0.parse()?;
        Ok(index - 1)
    };

    for line in read_to_string(path)?.lines().into_iter() {
        let tokens = line.split(' ').collect::<Vec<_>>();

        if tokens[0] == "v" {
            vertices.push([tokens[1].parse()?, tokens[2].parse()?, tokens[3].parse()?]);
        } else if tokens[0] == "f" {
            faces.push([
                parse_face_index(tokens[1])?,
                parse_face_index(tokens[2])?,
                parse_face_index(tokens[3])?,
            ])
        }
    }

    Ok(Wavefront { vertices, faces })
}
