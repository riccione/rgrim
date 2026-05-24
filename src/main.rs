use anyhow::Result;

use rgrim::capture::capture_primary_monitor;
use rgrim::editor::{crop_image, run_editor};
use rgrim::ui::run_sniper_overlay;

fn main() -> Result<()> {
    let captured = capture_primary_monitor()?;
    println!("Captured '{}' ({}x{})", captured.name, captured.width, captured.height);

    let rect = run_sniper_overlay(captured.image.clone());

    match rect {
        Some(region) => {
            println!("Selected region: min={:?} max={:?}", region.min, region.max);
            let cropped = crop_image(&captured.image, &region);
            println!("Cropped region ({}x{}). Launching editor...", cropped.width(), cropped.height());
            run_editor(cropped)?;
        }
        None => println!("Selection cancelled."),
    }

    Ok(())
}
