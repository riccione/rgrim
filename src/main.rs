use anyhow::Result;

use rgrim::capture::capture_primary_monitor;
use rgrim::ui::run_sniper_overlay;

fn main() -> Result<()> {
    let captured = capture_primary_monitor()?;
    println!(
        "Captured '{}' ({}x{})",
        captured.name, captured.width, captured.height
    );

    let rect = run_sniper_overlay(captured.image);
    match rect {
        Some(r) => println!("Selected region: min={:?} max={:?}", r.min, r.max),
        None => println!("Selection cancelled."),
    }

    Ok(())
}
