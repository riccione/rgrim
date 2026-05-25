use anyhow::{Result, anyhow};

use rgrim::capture::capture_primary_monitor;
use rgrim::editor::{crop_image, run_editor};
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
    let captured = capture_primary_monitor()?;

    let rect = run_sniper_overlay(captured.image.clone());

    if let Some(region) = rect {
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

        run_editor(cropped, auto_save_msg)?;
    } else {
        println!("Selection cancelled.");
    }

    Ok(())
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
        Box::new(move |_cc| {
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
    #[allow(deprecated)]
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(eframe::egui::Key::Escape) || i.key_pressed(eframe::egui::Key::Q)) {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
            return;
        }

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
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
                    .size(11.0),
            );
        });
    }

    fn ui(&mut self, _ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {}
}
