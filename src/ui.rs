use eframe::egui::{
    self, ColorImage, CornerRadius, Pos2, Rect, Sense, StrokeKind, TextureHandle, TextureOptions,
};
use image::RgbaImage;
use std::sync::{Arc, Mutex};

/// Runs the full-screen sniper overlay.
/// Returns the selected region, or `None` if cancelled.
pub fn run_sniper_overlay(background: RgbaImage) -> Option<Rect> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    // Pass the background image data but lazily build the texture inside the creation context 'cc'
    let mut background_data = Some(background);

    eframe::run_native(
        "rgrim-sniper",
        native_options,
        Box::new(move |cc| {
            // Force 1:1 scaling BEFORE eframe performs window sizing and bounding loops
            cc.egui_ctx.set_pixels_per_point(1.0);

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
    selection_start: Option<Pos2>,
    selection_end: Option<Pos2>,
    result: Arc<Mutex<Option<Rect>>>,
}

impl SniperOverlay {
    fn new(ctx: &egui::Context, img: RgbaImage, result: Arc<Mutex<Option<Rect>>>) -> Self {
        let w = img.width() as usize;
        let h = img.height() as usize;
        let pixels = img.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied([w, h], &pixels);

        // Cache the texture allocation on initialize to stop high GPU thrashing loops
        let texture = ctx.load_texture("screenshot", color_image, TextureOptions::default());

        Self {
            texture,
            selection_start: None,
            selection_end: None,
            result,
        }
    }
}

impl eframe::App for SniperOverlay {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}

    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.0);

        let viewport_rect = ctx.viewport_rect();

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            *self.result.lock().unwrap() = Some(viewport_rect);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                // Use cached texture handle cleanly
                ui.painter().image(
                    self.texture.id(),
                    viewport_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                let response =
                    ui.interact(viewport_rect, ui.next_auto_id(), Sense::click_and_drag());

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
                            rect = viewport_rect;
                        }

                        *self.result.lock().unwrap() = Some(rect);
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
