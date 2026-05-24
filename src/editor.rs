use std::io::Write;
use std::process::{Command, Stdio};

use eframe::egui::{self, ColorImage, Pos2, Rect, Sense, TextureHandle, TextureOptions, Vec2};
use image::ImageEncoder;
use image::RgbaImage;
use image::imageops;

/// Represents a selected screen region captured by the sniper overlay.
#[derive(Clone, Debug)]
pub struct SelectionRect {
    pub region: Rect,
}

/// Crops an RgbaImage to the given egui::Rect region.
/// Assumes 1:1 pixel mapping (pixels_per_point == 1.0).
pub fn crop_image(image: &RgbaImage, region: &Rect) -> RgbaImage {
    let x = region.min.x as u32;
    let y = region.min.y as u32;
    let w = (region.max.x - region.min.x) as u32;
    let h = (region.max.y - region.min.y) as u32;
    let mut img = image.clone();
    imageops::crop(&mut img, x, y, w, h).to_image()
}

/// Runs the main editor window with toolbar and central canvas.
/// `auto_save_msg` is shown in the status bar on launch if set.
pub fn run_editor(image: RgbaImage, auto_save_msg: Option<String>) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(960.0, 720.0)),
        ..Default::default()
    };

    let mut image_data = Some(image);

    eframe::run_native(
        "rgrim-editor",
        native_options,
        Box::new(move |cc| {
            let img = image_data.take().expect("App state consumed twice");
            Ok(Box::new(EditorApp::new(&cc.egui_ctx, img, auto_save_msg)))
        }),
    )?;

    Ok(())
}

#[derive(PartialEq)]
enum Tool {
    None,
    Pen,
    Highlighter,
}

struct Stroke {
    points: Vec<Pos2>,
    color: egui::Color32,
    thickness: f32,
}

pub struct EditorApp {
    texture: TextureHandle,
    original_image: RgbaImage,
    active_tool: Tool,
    strokes: Vec<Stroke>,
    current_stroke: Option<Stroke>,
    status_message: Option<String>,
    status_set_at: f64,
}

impl EditorApp {
    pub fn new(ctx: &egui::Context, img: RgbaImage, status_msg: Option<String>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_corner_radius = egui::CornerRadius::from(8);
        ctx.set_visuals(visuals);

        let size = [img.width() as usize, img.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
        let texture = ctx.load_texture("editor_image", color_image, TextureOptions::default());
        let original = img.clone();
        let status_message = status_msg.clone();
        let status_set_at = if status_msg.is_some() { f64::MAX } else { 0.0 };

        Self {
            texture,
            original_image: original,
            active_tool: Tool::None,
            strokes: Vec::new(),
            current_stroke: None,
            status_message,
            status_set_at,
        }
    }

    fn set_status(&mut self, ctx: &egui::Context, msg: String) {
        self.status_message = Some(msg);
        self.status_set_at = ctx.input(|i| i.time);
    }

    fn bake_and_export(&self) -> RgbaImage {
        bake_strokes(&self.original_image, &self.strokes)
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}

    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if self.status_message.is_some() {
            let now = ctx.input(|i| i.time);
            if now - self.status_set_at > 3.0 {
                self.status_message = None;
            }
        }

        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::none()
                    .inner_margin(8.0)
                    .fill(ctx.style().visuals.window_fill()),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);

                    let pen_btn = ui.add_sized(
                        [60.0, 28.0],
                        egui::SelectableLabel::new(self.active_tool == Tool::Pen, "Pen"),
                    );
                    if pen_btn.clicked() {
                        self.active_tool = if self.active_tool == Tool::Pen {
                            Tool::None
                        } else {
                            Tool::Pen
                        };
                    }

                    let hl_btn = ui.add_sized(
                        [86.0, 28.0],
                        egui::SelectableLabel::new(
                            self.active_tool == Tool::Highlighter,
                            "Highlighter",
                        ),
                    );
                    if hl_btn.clicked() {
                        self.active_tool = if self.active_tool == Tool::Highlighter {
                            Tool::None
                        } else {
                            Tool::Highlighter
                        };
                    }

                    let clear_btn = ui.add_sized([60.0, 28.0], egui::Button::new("Clear"));
                    if clear_btn.clicked() {
                        self.strokes.clear();
                        self.current_stroke = None;
                    }

                    ui.separator();

                    let copy_btn = ui.add_sized([65.0, 28.0], egui::Button::new("Copy"));
                    if copy_btn.clicked() {
                        let baked = self.bake_and_export();
                        match copy_to_clipboard(&baked) {
                            Ok(()) => self.set_status(ctx, "Copied to clipboard!".into()),
                            Err(e) => self.set_status(ctx, format!("Copy failed: {}", e)),
                        }
                    }

                    let save_btn = ui.add_sized([65.0, 28.0], egui::Button::new("Save"));
                    if save_btn.clicked() {
                        let baked = self.bake_and_export();
                        match save_to_file(&baked) {
                            Ok(path) => self.set_status(ctx, format!("Saved to {}", path)),
                            Err(e) => self.set_status(ctx, format!("Save failed: {}", e)),
                        }
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 22)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let img_size = self.texture.size_vec2();
                let scale = (available.x / img_size.x)
                    .min(available.y / img_size.y)
                    .min(1.0);
                let scaled = img_size * scale;
                let image_rect = Rect::from_center_size(ui.clip_rect().center(), scaled);

                ui.painter().image(
                    self.texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                for stroke in &self.strokes {
                    paint_stroke(ui, stroke, image_rect);
                }
                if let Some(stroke) = &self.current_stroke {
                    paint_stroke(ui, stroke, image_rect);
                }

                if self.active_tool != Tool::None {
                    let response =
                        ui.interact(image_rect, ui.next_auto_id(), Sense::click_and_drag());

                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let normalized = screen_to_image(pos, image_rect);
                            self.current_stroke = Some(Stroke {
                                points: vec![normalized],
                                color: match self.active_tool {
                                    Tool::Highlighter => {
                                        egui::Color32::from_rgba_premultiplied(255, 255, 0, 80)
                                    }
                                    Tool::Pen => egui::Color32::RED,
                                    _ => unreachable!(),
                                },
                                thickness: match self.active_tool {
                                    Tool::Highlighter => 24.0,
                                    Tool::Pen => 3.0,
                                    _ => unreachable!(),
                                },
                            });
                        }
                    }

                    if response.dragged() {
                        if let Some(stroke) = &mut self.current_stroke {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let normalized = screen_to_image(pos, image_rect);
                                stroke.points.push(normalized);
                            }
                        }
                    }

                    let was_dragging = self.current_stroke.is_some();
                    if was_dragging && !response.dragged() && !response.is_pointer_button_down_on()
                    {
                        if let Some(stroke) = self.current_stroke.take() {
                            if !stroke.points.is_empty() {
                                self.strokes.push(stroke);
                            }
                        }
                    }
                }
            });

        if let Some(msg) = &self.status_message {
            egui::TopBottomPanel::bottom("status_bar")
                .frame(
                    egui::Frame::none()
                        .inner_margin(6.0)
                        .fill(egui::Color32::from_rgb(0, 120, 255)),
                )
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(msg)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                    });
                });
        }
    }
}

fn screen_to_image(screen_pos: Pos2, image_rect: Rect) -> Pos2 {
    Pos2::new(
        ((screen_pos.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0),
        ((screen_pos.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0),
    )
}

fn paint_stroke(ui: &mut egui::Ui, stroke: &Stroke, image_rect: Rect) {
    if stroke.points.is_empty() {
        return;
    }
    let mapped: Vec<Pos2> = stroke
        .points
        .iter()
        .map(|p| {
            Pos2::new(
                image_rect.min.x + p.x * image_rect.width(),
                image_rect.min.y + p.y * image_rect.height(),
            )
        })
        .collect();

    for i in 1..mapped.len() {
        ui.painter().line_segment(
            [mapped[i - 1], mapped[i]],
            egui::Stroke::new(stroke.thickness, stroke.color),
        );
    }
}

fn bake_strokes(image: &RgbaImage, strokes: &[Stroke]) -> RgbaImage {
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
    color: egui::Color32,
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

fn draw_filled_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: egui::Color32) {
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

fn blend_pixel(pixel: &mut image::Rgba<u8>, color: egui::Color32) {
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

fn copy_to_clipboard(image: &RgbaImage) -> anyhow::Result<()> {
    let w = image.width() as usize;
    let h = image.height() as usize;
    let bytes = image.as_raw().to_vec();

    let img_data = arboard::ImageData {
        width: w,
        height: h,
        bytes: std::borrow::Cow::Owned(bytes),
    };

    // Attempt 1: Try native arboard handler
    let clipboard = arboard::Clipboard::new();
    if let Ok(mut cb) = clipboard {
        if cb.set_image(img_data).is_ok() {
            return Ok(());
        }
    }

    // Attempt 2: Fallback explicitly tailored for Sway/Wayland via wl-copy
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    image::codecs::png::PngEncoder::new(&mut cursor)
        .write_image(
            image.as_raw(),
            w as u32,
            h as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| anyhow::anyhow!("Failed to encode clipboard buffer to PNG: {}", e))?;

    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| {
            anyhow::anyhow!(
                "Both arboard and 'wl-copy' failed. If on a pure Wayland compositor, ensure 'wl-clipboard' is installed: {}",
                e
            )
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&png_bytes)?;
    }

    let wl_status = child.wait()?;
    if !wl_status.success() {
        return Err(anyhow::anyhow!("wl-copy exited with an error status code"));
    }

    Ok(())
}

fn save_to_file(image: &RgbaImage) -> anyhow::Result<String> {
    let output_dir = crate::export::get_screenshot_directory();
    std::fs::create_dir_all(&output_dir)?;

    let filename = crate::export::generate_screenshot_filename();
    let path = output_dir.join(&filename);

    image.save(&path)?;
    Ok(filename)
}
