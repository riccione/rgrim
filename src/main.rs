use anyhow::{Result, anyhow};

use rgrim::capture::{capture_primary_monitor, capture_settled_monitor};
use rgrim::editor::{EditorOutcome, crop_image, run_editor};
use rgrim::export::{generate_screenshot_filename, get_screenshot_directory};
use rgrim::ui::run_sniper_overlay;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let show_gui = args.iter().any(|arg| arg == "--gui" || arg == "-g");

    if show_gui {
        run_dashboard_interface()?;
    } else {
        trigger_instant_capture_flow()?;
    }

    Ok(())
}

fn trigger_instant_capture_flow() -> Result<()> {
    let mut captured = capture_primary_monitor()?;

    loop {
        let rect = run_sniper_overlay(&captured.image)?;

        let Some(region) = rect else {
            println!("Selection cancelled.");
            return Ok(());
        };

        let cropped = crop_image(&captured.image, &region);

        let save_dir = get_screenshot_directory();
        let auto_save_msg = match std::fs::create_dir_all(&save_dir) {
            Ok(()) => {
                let filename = generate_screenshot_filename();
                let full_path = save_dir.join(&filename);
                match cropped.save(&full_path) {
                    Ok(()) => {
                        let msg = format!("Screenshot saved to {}", full_path.display());
                        Some(msg)
                    }
                    Err(e) => {
                        eprintln!("Auto-save failed: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create screenshot directory: {}", e);
                None
            }
        };

        match run_editor(cropped, auto_save_msg)? {
            EditorOutcome::NewCapture => {
                // The compositor may still be rendering the closed editor
                // window; capture_settled_monitor() waits for stable output
                // instead of relying on a guessed delay.
                captured = capture_settled_monitor()?;
            }
            EditorOutcome::Closed => return Ok(()),
        }
    }
}

fn run_dashboard_interface() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([400.0, 250.0])
            .with_resizable(false),
        ..Default::default()
    };

    let capture_triggered = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let capture_triggered_clone = capture_triggered.clone();

    eframe::run_native(
        "rgrim Dashboard",
        native_options,
        Box::new(move |cc| {
            rgrim::style::apply_font_scale(&cc.egui_ctx, 1.2);
            Ok(Box::new(DashboardApp {
                trigger_capture: capture_triggered_clone,
            }))
        }),
    )
    .map_err(|e| anyhow!("Dashboard failure: {}", e))?;

    if capture_triggered.load(std::sync::atomic::Ordering::Relaxed) {
        trigger_instant_capture_flow()?;
    }

    Ok(())
}

struct DashboardApp {
    trigger_capture: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
            });
            ui.separator();

            ui.label("Quick Keyboard Guide:");
            ui.label(" • Enter (Sniper Mode) : Select entire screen canvas");
            ui.label(" • Escape / Q : Instantly abort open overlays");
            ui.add_space(15.0);

            if ui.button("⚡ Take Screenshot Now").clicked() {
                self.trigger_capture
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
            }

            ui.add_space(4.0);
            ui.label(
                eframe::egui::RichText::new("Esc / Q — Close")
                    .color(eframe::egui::Color32::GRAY)
                    .size(13.0),
            );
        });
    }
}
