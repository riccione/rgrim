use eframe::egui::Color32;
use image::RgbaImage;

use super::types::Stroke;

pub(crate) fn bake_strokes(image: &RgbaImage, strokes: &[Stroke]) -> RgbaImage {
    let (w, h) = image.dimensions();
    let mut output = image.clone();

    for stroke in strokes {
        if stroke.points.is_empty() {
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

        // Click-only stroke (a drag that recorded one point): stamp a single dot.
        if mapped.len() == 1 {
            let (x, y) = mapped[0];
            draw_filled_circle(&mut output, x, y, radius, stroke.color);
            continue;
        }

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
    // Color32 components are premultiplied (ecolor's invariant), so
    // source-over onto an opaque destination is out = src + dst * (1 - a).
    // For valid premultiplied colors (src <= a) this can never exceed 255,
    // and a == 0 / a == 255 fall out as no-op / overwrite without specials.
    let inv = 255 - color.a() as u32;
    let d = &mut pixel.0;
    d[0] = (color.r() as u32 + (d[0] as u32 * inv + 127) / 255) as u8;
    d[1] = (color.g() as u32 + (d[1] as u32 * inv + 127) / 255) as u8;
    d[2] = (color.b() as u32 + (d[2] as u32 * inv + 127) / 255) as u8;
}

#[cfg(test)]
mod tests {
    use super::{Stroke, bake_strokes, blend_pixel};
    use eframe::egui::{Color32, Pos2};
    use image::{Rgba, RgbaImage};

    fn blend(bg: [u8; 4], color: Color32) -> [u8; 4] {
        let mut px = Rgba(bg);
        blend_pixel(&mut px, color);
        px.0
    }

    #[test]
    fn transparent_leaves_pixel_untouched() {
        let c = Color32::from_rgba_unmultiplied(255, 0, 0, 0);
        assert_eq!(blend([10, 20, 30, 255], c), [10, 20, 30, 255]);
    }

    #[test]
    fn opaque_overwrites() {
        let c = Color32::from_rgb(9, 8, 7);
        assert_eq!(blend([10, 20, 30, 255], c), [9, 8, 7, 255]);
    }

    #[test]
    fn highlighter_blends_as_31_percent_yellow() {
        let hl = Color32::from_rgba_unmultiplied(255, 255, 0, 80);
        assert_eq!(blend([255, 255, 255, 255], hl), [255, 255, 175, 255]);
        assert_eq!(blend([0, 0, 0, 255], hl), [80, 80, 0, 255]);
    }

    #[test]
    fn premultiplied_source_over_never_overflows() {
        let bright = Color32::from_rgba_unmultiplied(255, 255, 255, 254);
        // A wrapping implementation would land below 255 here.
        assert_eq!(blend([255, 255, 255, 255], bright), [255, 255, 255, 255]);
    }

    #[test]
    fn single_point_stroke_bakes_a_dot() {
        let base = RgbaImage::from_pixel(32, 32, Rgba([0, 0, 0, 255]));
        let stroke = Stroke {
            points: vec![Pos2::new(0.5, 0.5)],
            color: Color32::RED,
            thickness: 5.0,
        };

        let out = bake_strokes(&base, std::slice::from_ref(&stroke));

        // radius = ceil(5 / 2) = 3, so center and the dx == 3 rim are stamped...
        assert_eq!(out.get_pixel(16, 16).0, [255, 0, 0, 255]);
        assert_eq!(out.get_pixel(19, 16).0, [255, 0, 0, 255]);
        // ...and beyond it the background is untouched.
        assert_eq!(out.get_pixel(26, 16).0, [0, 0, 0, 255]);
    }
}
