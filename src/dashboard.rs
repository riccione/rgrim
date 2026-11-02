//! Dashboard window: the GUI entry point for starting captures.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};

/// Runs the dashboard window and returns the delay the user picked for the
/// next capture, or `None` if the window was dismissed.
pub fn run() -> Result<Option<Duration>> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };

    let capture_request = Arc::new(Mutex::new(None));
    let capture_request_clone = capture_request.clone();

    eframe::run_native(
        "rgrim Dashboard",
        native_options,
        Box::new(move |cc| {
            crate::style::apply_font_scale(&cc.egui_ctx, 1.2);
            Ok(Box::new(DashboardApp {
                capture_request: capture_request_clone,
            }))
        }),
    )
    .context("Dashboard failure")?;

    Ok(*capture_request.lock().unwrap())
}

struct DashboardApp {
    /// `None` until a button asks for a capture; the value is the delay
    /// before that capture starts.
    capture_request: Arc<Mutex<Option<Duration>>>,
}

impl DashboardApp {
    fn request_capture(&self, ctx: &eframe::egui::Context, delay: Duration) {
        *self.capture_request.lock().unwrap() = Some(delay);
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
    }
}

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if ctx.input(|i| {
            i.key_pressed(eframe::egui::Key::Escape) || i.key_pressed(eframe::egui::Key::Q)
        }) {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
            return;
        }

        eframe::egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("📸 rgrim Screen Utility");
                ui.label("A lightweight, cross-platform sniper annotation engine.");
                ui.label(
                    eframe::egui::RichText::new(format!("v{}", crate::VERSION))
                        .color(eframe::egui::Color32::GRAY)
                        .size(13.0),
                );
            });
            ui.separator();

            ui.label("Quick Keyboard Guide:");
            ui.label(" • Enter (Sniper Mode) : Select entire screen canvas");
            ui.label(" • Escape / Q : Instantly abort open overlays");
            ui.add_space(15.0);

            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(
                        [240.0, 30.0],
                        eframe::egui::Button::new("⚡ Take Screenshot Now"),
                    )
                    .clicked()
                {
                    self.request_capture(&ctx, Duration::ZERO);
                }
                if ui
                    .add_sized([240.0, 30.0], eframe::egui::Button::new("⏱ Capture in 3 s"))
                    .clicked()
                {
                    self.request_capture(&ctx, Duration::from_secs(3));
                }
                if ui
                    .add_sized([240.0, 30.0], eframe::egui::Button::new("⏱ Capture in 5 s"))
                    .clicked()
                {
                    self.request_capture(&ctx, Duration::from_secs(5));
                }
            });

            ui.add_space(4.0);
            ui.label(
                eframe::egui::RichText::new("Esc / Q — Close")
                    .color(eframe::egui::Color32::GRAY)
                    .size(13.0),
            );
        });
    }
}
