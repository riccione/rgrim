use eframe::egui::{self, ColorImage, Pos2, Rect, Sense, TextureHandle, TextureOptions, Vec2};
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
pub fn run_editor(image: RgbaImage) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(960.0, 720.0)),
        ..Default::default()
    };

    let mut image_data = Some(image);

    eframe::run_native(
        "rgrim-editor",
        native_options,
        Box::new(move |cc| {
            let img = image_data.take().expect("App state consumed twice");
            Ok(Box::new(EditorApp::new(&cc.egui_ctx, img)))
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
    active_tool: Tool,
    strokes: Vec<Stroke>,
    current_stroke: Option<Stroke>,
}

impl EditorApp {
    pub fn new(ctx: &egui::Context, img: RgbaImage) -> Self {
        let w = img.width() as usize;
        let h = img.height() as usize;
        let pixels = img.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied([w, h], &pixels);
        let texture = ctx.load_texture("editor_image", color_image, TextureOptions::default());
        Self {
            texture,
            active_tool: Tool::None,
            strokes: Vec::new(),
            current_stroke: None,
        }
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    }

    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tool == Tool::Pen, "Pen").clicked() {
                    self.active_tool = if self.active_tool == Tool::Pen { Tool::None } else { Tool::Pen };
                }
                if ui.selectable_label(self.active_tool == Tool::Highlighter, "Highlighter").clicked() {
                    self.active_tool = if self.active_tool == Tool::Highlighter { Tool::None } else { Tool::Highlighter };
                }
                if ui.button("Clear").clicked() {
                    self.strokes.clear();
                    self.current_stroke = None;
                }
                let _ = ui.button("Copy");
                let _ = ui.button("Save");
            });
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let img_size = self.texture.size_vec2();
                let scale = (available.x / img_size.x).min(available.y / img_size.y).min(1.0);
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
                    let response = ui.interact(image_rect, ui.next_auto_id(), Sense::click_and_drag());

                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let normalized = screen_to_image(pos, image_rect);
                            self.current_stroke = Some(Stroke {
                                points: vec![normalized],
                                color: match self.active_tool {
                                    Tool::Highlighter => egui::Color32::from_rgba_premultiplied(255, 255, 0, 80),
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
                    if was_dragging && !response.dragged() && !response.is_pointer_button_down_on() {
                        if let Some(stroke) = self.current_stroke.take() {
                            if !stroke.points.is_empty() {
                                self.strokes.push(stroke);
                            }
                        }
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
