use anyhow::Result;

use rgrim::capture::capture_primary_monitor;
use rgrim::editor::{crop_image, run_editor};
use rgrim::export::{generate_screenshot_filename, get_screenshot_directory};
use rgrim::ui::run_sniper_overlay;

fn main() -> Result<()> {
    let captured = capture_primary_monitor()?;

    let rect = run_sniper_overlay(captured.image.clone());

    if let Some(region) = rect {
        // println!("Selected region: min={:?} max={:?}", region.min, region.max);
        let cropped = crop_image(&captured.image, &region);

        let save_dir = get_screenshot_directory();
        let auto_save_msg = match std::fs::create_dir_all(&save_dir) {
            Ok(()) => {
                let filename = generate_screenshot_filename();
                let full_path = save_dir.join(&filename);
                match cropped.save(&full_path) {
                    Ok(()) => {
                        let msg = format!("Screenshot saved to {}", full_path.display());
                        // println!("{}", msg);
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
