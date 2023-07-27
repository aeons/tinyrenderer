use anyhow::Result;
use glam::{vec2, vec3, IVec2, Vec2, Vec3};
use image::{imageops::flip_vertical_in_place, ImageBuffer, Rgb, RgbImage};
use wavefront::Obj;

type Color = Rgb<u8>;
type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

const SIZE: u32 = 1024;
const WIDTH: u32 = SIZE - 1;
const HEIGHT: u32 = SIZE - 1;

const HALF_WIDTH: f32 = WIDTH as f32 / 2f32;
const HALF_HEIGHT: f32 = HEIGHT as f32 / 2f32;

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const BLUE: Rgb<u8> = Rgb([0, 0, 255]);

fn main() -> Result<()> {
    let mut image = RgbImage::new(WIDTH, HEIGHT);

    let model = Obj::from_file("./obj/african_head.obj")?;

    let light_dir = vec3(0f32, 0f32, -1f32);
    let mut zbuffer = vec![f32::MIN; (WIDTH * HEIGHT) as usize];

    let mut screen_coords = [Vec3::ZERO; 3];
    let mut world_coords = [Vec3::ZERO; 3];

    for poly in model.polygons() {
        for i in 0..3 {
            let v = poly.vertex(i).unwrap().position().into();
            world_coords[i] = v;
            screen_coords[i] = world_to_screen(&v);
        }

        let n = (world_coords[2] - world_coords[0])
            .cross(world_coords[1] - world_coords[0])
            .normalize();
        let intensity = n.dot(light_dir);

        if intensity > 0f32 {
            let color = Rgb([(intensity * 255f32) as u8; 3]);
            triangle(screen_coords, &mut zbuffer, &mut image, &color);
        }
    }

    flip_vertical_in_place(&mut image);
    image.save_with_format("output.jpg", image::ImageFormat::Jpeg)?;

    Ok(())
}

fn world_to_screen(v: &Vec3) -> Vec3 {
    vec3(
        (v.x + 1f32) * HALF_WIDTH + 0.5f32,
        (v.y + 1f32) * HALF_HEIGHT + 0.5f32,
        v.z,
    )
}

fn barycentric(points: [Vec3; 3], p: Vec2) -> Vec3 {
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
    // `u.z` is integer, if it is zero then the triangle is degenerate
    if u.z.abs() > 1e-2f32 {
        vec3(1f32 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
    } else {
        vec3(-1f32, 1f32, 1f32)
    }
}

fn bounding_box(points: &[Vec3]) -> (Vec2, Vec2) {
    let mut bbox_min = vec2(f32::MAX, f32::MAX);
    let mut bbox_max = vec2(f32::MIN, f32::MIN);
    let clamp = vec2(WIDTH as f32 - 1f32, HEIGHT as f32 - 1f32);

    for point in points.iter() {
        bbox_min.x = f32::max(0f32, f32::min(bbox_min.x, point.x));
        bbox_min.y = f32::max(0f32, f32::min(bbox_min.y, point.y));

        bbox_max.x = f32::min(clamp.x, f32::max(bbox_max.x, point.x));
        bbox_max.y = f32::min(clamp.y, f32::max(bbox_max.y, point.y));
    }

    (bbox_min, bbox_max)
}

fn triangle(points: [Vec3; 3], zbuffer: &mut [f32], image: &mut Image, color: &Color) {
    let (bbox_min, bbox_max) = bounding_box(&points);

    for x in bbox_min.x as i32..=bbox_max.x as i32 {
        for y in bbox_min.y as i32..=bbox_max.y as i32 {
            let bc_screen = barycentric(points, vec2(x as f32, y as f32));
            if bc_screen.x < 0f32 || bc_screen.y < 0f32 || bc_screen.z < 0f32 {
                continue;
            }

            let mut z = 0f32;
            for i in 0..3 {
                z += points[i].z * bc_screen[i];
            }

            let z_idx = (x + y * WIDTH as i32) as usize;
            if z > zbuffer[z_idx] {
                zbuffer[z_idx] = z;
                image.put_pixel(x as u32, y as u32, *color);
            }
        }
    }
}
