use anyhow::Result;
use glam::{ivec2, vec2, vec3, IVec2, Vec2, Vec3};
use image::{imageops::flip_vertical_in_place, ImageBuffer, Rgb, RgbImage};
use wavefront::Obj;

type Color = Rgb<u8>;
type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const BLUE: Rgb<u8> = Rgb([0, 0, 255]);

fn main() -> Result<()> {
    let size = 1024;
    let width = size - 1;
    let height = size - 1;
    let mut image = RgbImage::new(width, height);

    let model = Obj::from_file("./obj/african_head.obj")?;

    let half_width = width as f32 / 2f32;
    let half_height = height as f32 / 2f32;

    let light_dir = vec3(0f32, 0f32, -1f32);

    for poly in model.polygons() {
        let mut screen_coords = [Vec2::ZERO; 3];
        let mut world_coords = [Vec3::ZERO; 3];

        for i in 0..3 {
            let v = poly.vertex(i).unwrap().position();
            screen_coords[i] = vec2((v[0] + 1f32) * half_width, (v[1] + 1f32) * half_height);
            world_coords[i] = v.into();
        }

        let n = (world_coords[2] - world_coords[0])
            .cross(world_coords[1] - world_coords[0])
            .normalize();
        let intensity = n.dot(light_dir);

        if intensity > 0f32 {
            let color = Rgb([(intensity * 255f32) as u8; 3]);
            triangle(screen_coords.map(|v| v.as_ivec2()), &mut image, &color);
        }
    }

    flip_vertical_in_place(&mut image);
    image.save_with_format("output.jpg", image::ImageFormat::Jpeg)?;

    Ok(())
}

fn line(mut v0: IVec2, mut v1: IVec2, image: &mut Image, color: &Color) {
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

fn barycentric(points: [Vec2; 3], p: Vec2) -> Vec3 {
    let u = vec3(
        points[2].x - points[0].x,
        points[1].x - points[0].x,
        points[0].x - p.x,
    )
    .cross(vec3(
        points[2].y - points[0].y,
        points[1].y - points[0].y,
        points[0].y - p.y,
    ));
    // `points` and `p` has integer values as coordinates
    // so u.z.abs() < 1 means u.z is 0, which means triangle is degenerate
    // in this case return something with negative coordinates
    if u.z.abs() < 1f32 {
        return vec3(-1f32, 1f32, 1f32);
    }

    vec3(1f32 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
}

fn triangle(points: [IVec2; 3], image: &mut Image, color: &Color) {
    let mut bbox_min = ivec2(image.width() as i32 - 1, image.height() as i32 - 1);
    let mut bbox_max = ivec2(0, 0);
    let clamp = bbox_min.clone();

    for point in points.iter() {
        bbox_min.x = i32::max(0, i32::min(bbox_min.x, point.x));
        bbox_min.y = i32::max(0, i32::min(bbox_min.y, point.y));

        bbox_max.x = i32::min(clamp.x, i32::max(bbox_max.x, point.x));
        bbox_max.y = i32::min(clamp.y, i32::max(bbox_max.y, point.y));
    }

    let points = points.map(|p| p.as_vec2());
    for x in bbox_min.x..=bbox_max.x {
        for y in bbox_min.y..=bbox_max.y {
            let bc_screen = barycentric(points, vec2(x as f32, y as f32));
            if bc_screen.x < 0f32 || bc_screen.y < 0f32 || bc_screen.z < 0f32 {
                continue;
            }

            image.put_pixel(x as u32, y as u32, *color);
        }
    }
}
