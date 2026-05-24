use eframe::egui::{
    self, ColorImage, CornerRadius, Pos2, Rect, Sense, StrokeKind, TextureHandle, TextureOptions,
};
use image::RgbaImage;
use std::sync::{Arc, Mutex};

/// Runs the full-screen sniper overlay.
/// Returns the selected region in physical pixels, or `None` if cancelled.
pub fn run_sniper_overlay(background: RgbaImage) -> Option<Rect> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };

    let mut background_data = Some(background);

    eframe::run_native(
        "rgrim-sniper",
        native_options,
        Box::new(move |cc| {
            let raw_img = background_data.take().expect("App state consumed twice");
            let app = SniperOverlay::new(&cc.egui_ctx, raw_img, result_clone);
            Ok(Box::new(app))
        }),
    )
    .ok()?;

    result.lock().ok().and_then(|r| *r)
}

struct SniperOverlay {
    texture: TextureHandle,
    result: Arc<Mutex<Option<Rect>>>,
    selection_start: Option<Pos2>,
    selection_end: Option<Pos2>,
    image_width: f32,
    image_height: f32,
}

impl SniperOverlay {
    pub fn new(ctx: &egui::Context, img: RgbaImage, result: Arc<Mutex<Option<Rect>>>) -> Self {
        let w = img.width() as f32;
        let h = img.height() as f32;

        let w_usize = img.width() as usize;
        let h_usize = img.height() as usize;
        let pixels = img.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied([w_usize, h_usize], &pixels);
        let texture = ctx.load_texture("background_image", color_image, TextureOptions::default());

        Self {
            texture,
            result,
            selection_start: None,
            selection_end: None,
            image_width: w,
            image_height: h,
        }
    }
}

impl eframe::App for SniperOverlay {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}

    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Q)) {
            *self.result.lock().unwrap() = None;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let content_rect = ui.available_rect_before_wrap();

                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    *self.result.lock().unwrap() = Some(Rect::from_min_max(
                        Pos2::new(0.0, 0.0),
                        Pos2::new(self.image_width, self.image_height),
                    ));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }

                ui.painter().image(
                    self.texture.id(),
                    content_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                let response = ui.interact(content_rect, ui.next_auto_id(), Sense::click_and_drag());

                if response.drag_started() {
                    self.selection_start = response.interact_pointer_pos();
                    self.selection_end = self.selection_start;
                }

                if response.dragged() {
                    self.selection_end = response.interact_pointer_pos();
                }

                let was_dragging = self.selection_start.is_some();
                if was_dragging && !response.dragged() && !response.is_pointer_button_down_on() {
                    if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                        let mut rect = Rect::from_two_pos(start, end);

                        if rect.area() <= 1.0 {
                            rect = content_rect;
                        }

                        let scale_x = self.image_width / content_rect.width();
                        let scale_y = self.image_height / content_rect.height();

                        let physical_rect = Rect::from_min_max(
                            Pos2::new(rect.min.x * scale_x, rect.min.y * scale_y),
                            Pos2::new(rect.max.x * scale_x, rect.max.y * scale_y),
                        );

                        *self.result.lock().unwrap() = Some(physical_rect);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let rect = Rect::from_two_pos(start, end);
                    ui.painter().rect(
                        rect,
                        CornerRadius::ZERO,
                        egui::Color32::from_rgba_premultiplied(0, 120, 255, 80),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 120, 255)),
                        StrokeKind::Inside,
                    );
                }
            });

        ctx.request_repaint();
    }
}
