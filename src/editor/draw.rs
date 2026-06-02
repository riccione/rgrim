use eframe::egui::Color32;
use image::RgbaImage;

use super::types::Stroke;

pub(crate) fn bake_strokes(image: &RgbaImage, strokes: &[Stroke]) -> RgbaImage {
    let (w, h) = image.dimensions();
    let mut output = image.clone();

    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }

        let mapped: Vec<(i32, i32)> = stroke
            .points
            .iter()
            .map(|p| {
                (
                    (p.x.clamp(0.0, 1.0) * w as f32) as i32,
                    (p.y.clamp(0.0, 1.0) * h as f32) as i32,
                )
            })
            .collect();

        let radius = (stroke.thickness / 2.0).ceil() as i32;

        for i in 1..mapped.len() {
            draw_thick_segment(&mut output, mapped[i - 1], mapped[i], radius, stroke.color);
        }
    }

    output
}

fn draw_thick_segment(
    img: &mut RgbaImage,
    p1: (i32, i32),
    p2: (i32, i32),
    radius: i32,
    color: Color32,
) {
    let dx = (p2.0 - p1.0).abs();
    let dy = -(p2.1 - p1.1).abs();
    let sx = if p1.0 < p2.0 { 1 } else { -1 };
    let sy = if p1.1 < p2.1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = p1.0;
    let mut y = p1.1;

    loop {
        draw_filled_circle(img, x, y, radius, color);

        if x == p2.0 && y == p2.1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_filled_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Color32) {
    let (w, h) = img.dimensions();
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    let pixel = img.get_pixel_mut(px as u32, py as u32);
                    blend_pixel(pixel, color);
                }
            }
        }
    }
}

fn blend_pixel(pixel: &mut image::Rgba<u8>, color: Color32) {
    let src = [color.r(), color.g(), color.b(), color.a()];
    if src[3] == 255 {
        *pixel = image::Rgba([src[0], src[1], src[2], 255]);
    } else if src[3] > 0 {
        let a = src[3] as f32 / 255.0;
        let inv = 1.0 - a;
        let d = pixel.0;
        pixel.0 = [
            (src[0] as f32 * a + d[0] as f32 * inv) as u8,
            (src[1] as f32 * a + d[1] as f32 * inv) as u8,
            (src[2] as f32 * a + d[2] as f32 * inv) as u8,
            255,
        ];
    }
}
