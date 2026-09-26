use eframe::egui::{self, ColorImage, Pos2, Rect, Sense, TextureHandle, TextureOptions, Vec2};
use image::RgbaImage;
use image::imageops;

use anyhow::Result;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod draw;
mod export;
mod types;

use self::draw::bake_strokes;
use self::export::{copy_to_clipboard, save_to_file};
use self::types::{DrawTool, Stroke};

/// Crops an RgbaImage to the given egui::Rect region.
/// Coordinates are clamped to image bounds. Returns a 0×0 image if the
/// region does not intersect the image.
pub fn crop_image(image: &RgbaImage, region: &Rect) -> RgbaImage {
    let img_w = image.width();
    let img_h = image.height();

    let x = (region.min.x as i32).max(0).min(img_w as i32) as u32;
    let y = (region.min.y as i32).max(0).min(img_h as i32) as u32;

    let max_x = (region.max.x as i32).max(0).min(img_w as i32) as u32;
    let max_y = (region.max.y as i32).max(0).min(img_h as i32) as u32;

    let w = max_x.saturating_sub(x);
    let h = max_y.saturating_sub(y);

    if w == 0 || h == 0 {
        return RgbaImage::new(0, 0);
    }

    imageops::crop_imm(image, x, y, w, h).to_image()
}

/// Outcome of a completed editor session.
#[derive(PartialEq, Eq, Debug)]
pub enum EditorOutcome {
    /// Session ended normally (Escape / Q / window close).
    Closed,
    /// User clicked "New": the app should return to the capture/sniper state.
    NewCapture,
}

/// Runs the main editor window with toolbar and central canvas.
/// `auto_save_msg` is shown in the status bar on launch if set.
pub fn run_editor(image: RgbaImage, auto_save_msg: Option<String>) -> Result<EditorOutcome> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(960.0, 720.0)),
        ..Default::default()
    };

    let mut image_data = Some(image);
    let new_capture = Arc::new(AtomicBool::new(false));
    let new_capture_clone = new_capture.clone();

    eframe::run_native(
        "rgrim-editor",
        native_options,
        Box::new(move |cc| {
            let img = image_data.take().expect("App state consumed twice");
            Ok(Box::new(EditorApp::new(
                &cc.egui_ctx,
                img,
                auto_save_msg,
                new_capture_clone,
            )))
        }),
    )?;

    if new_capture.load(Ordering::Relaxed) {
        Ok(EditorOutcome::NewCapture)
    } else {
        Ok(EditorOutcome::Closed)
    }
}

pub struct EditorApp {
    texture: TextureHandle,
    original_image: RgbaImage,
    active_tool: Option<DrawTool>,
    strokes: Vec<Stroke>,
    current_stroke: Option<Stroke>,
    status_message: Option<String>,
    status_set_at: f64,
    new_capture: Arc<AtomicBool>,
}

impl EditorApp {
    pub fn new(
        ctx: &egui::Context,
        img: RgbaImage,
        status_msg: Option<String>,
        new_capture: Arc<AtomicBool>,
    ) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_corner_radius = egui::CornerRadius::from(8);
        ctx.set_visuals(visuals);
        crate::style::apply_font_scale(ctx, 1.2);

        let size = [img.width() as usize, img.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
        let texture = ctx.load_texture("editor_image", color_image, TextureOptions::default());
        let status_set_at = if status_msg.is_some() { f64::MAX } else { 0.0 };
        let status_message = status_msg;

        Self {
            texture,
            original_image: img,
            active_tool: None,
            strokes: Vec::new(),
            current_stroke: None,
            status_message,
            status_set_at,
            new_capture,
        }
    }

    fn set_status(&mut self, ctx: &egui::Context, msg: String) {
        self.status_message = Some(msg);
        self.status_set_at = ctx.input(|i| i.time);
    }

    fn toggle_tool(&mut self, tool: DrawTool) {
        self.active_tool = if self.active_tool == Some(tool) {
            None
        } else {
            Some(tool)
        };
    }

    fn bake_and_export(&self) -> RgbaImage {
        bake_strokes(&self.original_image, &self.strokes)
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if self.status_message.is_some() {
            let now = ctx.input(|i| i.time);
            if now - self.status_set_at > 3.0 {
                self.status_message = None;
            }
        }

        egui::Panel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .inner_margin(8.0)
                    .fill(ctx.style_of(ctx.theme()).visuals.window_fill()),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);

                    let new_btn = ui.add_sized([72.0, 34.0], egui::Button::new("New"));
                    if new_btn.clicked() {
                        self.new_capture.store(true, Ordering::Relaxed);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }

                    let pen_btn = ui.add_sized(
                        [72.0, 34.0],
                        egui::Button::selectable(self.active_tool == Some(DrawTool::Pen), "Pen"),
                    );
                    if pen_btn.clicked() {
                        self.toggle_tool(DrawTool::Pen);
                    }

                    let hl_btn = ui.add_sized(
                        [104.0, 34.0],
                        egui::Button::selectable(
                            self.active_tool == Some(DrawTool::Highlighter),
                            "Highlighter",
                        ),
                    );
                    if hl_btn.clicked() {
                        self.toggle_tool(DrawTool::Highlighter);
                    }

                    let clear_btn = ui.add_sized([72.0, 34.0], egui::Button::new("Clear"));
                    if clear_btn.clicked() {
                        self.strokes.clear();
                        self.current_stroke = None;
                    }

                    ui.separator();

                    let copy_btn = ui.add_sized([78.0, 34.0], egui::Button::new("Copy"));
                    if copy_btn.clicked() {
                        let baked = self.bake_and_export();
                        match copy_to_clipboard(&baked) {
                            Ok(()) => self.set_status(&ctx, "Copied to clipboard!".into()),
                            Err(e) => self.set_status(&ctx, format!("Copy failed: {}", e)),
                        }
                    }

                    let save_btn = ui.add_sized([78.0, 34.0], egui::Button::new("Save"));
                    if save_btn.clicked() {
                        let baked = self.bake_and_export();
                        match save_to_file(&baked) {
                            Ok(path) => self.set_status(&ctx, format!("Saved to {}", path)),
                            Err(e) => self.set_status(&ctx, format!("Save failed: {}", e)),
                        }
                    }
                });
            });

        if let Some(msg) = &self.status_message {
            egui::Panel::bottom("status_bar")
                .frame(
                    egui::Frame::new()
                        .inner_margin(6.0)
                        .fill(egui::Color32::from_rgb(0, 120, 255)),
                )
                .show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(msg)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 20, 22)))
            .show(ui, |ui| {
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

                if let Some(tool) = self.active_tool {
                    let response =
                        ui.interact(image_rect, ui.next_auto_id(), Sense::click_and_drag());

                    if response.drag_started()
                        && let Some(pos) = response.interact_pointer_pos()
                    {
                        let normalized = screen_to_image(pos, image_rect);
                        let (color, thickness) = tool.drawing_properties();
                        self.current_stroke = Some(Stroke {
                            points: vec![normalized],
                            color,
                            thickness,
                        });
                    }

                    if response.dragged()
                        && let Some(stroke) = &mut self.current_stroke
                        && let Some(pos) = response.interact_pointer_pos()
                    {
                        let normalized = screen_to_image(pos, image_rect);
                        stroke.points.push(normalized);
                    }

                    let was_dragging = self.current_stroke.is_some();
                    if was_dragging
                        && !response.dragged()
                        && !response.is_pointer_button_down_on()
                        && let Some(stroke) = self.current_stroke.take()
                        && !stroke.points.is_empty()
                    {
                        self.strokes.push(stroke);
                    }
                }
            });
    }
}

fn screen_to_image(screen_pos: Pos2, image_rect: Rect) -> Pos2 {
    Pos2::new(
        ((screen_pos.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0),
        ((screen_pos.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0),
    )
}

fn paint_stroke(ui: &mut egui::Ui, stroke: &Stroke, image_rect: Rect) {
    let mut points = stroke.points.iter().map(|p| {
        Pos2::new(
            image_rect.min.x + p.x * image_rect.width(),
            image_rect.min.y + p.y * image_rect.height(),
        )
    });

    let Some(mut p1) = points.next() else {
        return;
    };
    let egui_stroke = egui::Stroke::new(stroke.thickness, stroke.color);

    for p2 in points {
        ui.painter().line_segment([p1, p2], egui_stroke);
        p1 = p2;
    }
}
