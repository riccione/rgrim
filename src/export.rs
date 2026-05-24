use std::path::PathBuf;

use chrono::Local;

/// Determines where to save screenshots.
/// Priority: `RGRIM_DIR` env var => `~/Pictures/Screenshots` => `./screenshots`
pub fn get_screenshot_directory() -> PathBuf {
    if let Ok(env_path) = std::env::var("RGRIM_DIR") {
        return PathBuf::from(env_path);
    }

    let candidates: [PathBuf; 3] = [
        // Standard: ~/Pictures/Screenshots
        dirs::home_dir()
            .map(|p| p.join("Pictures").join("Screenshots"))
            .unwrap_or_default(),
        // XDG-reported picture dir + Screenshots
        dirs::picture_dir()
            .map(|p| p.join("Screenshots"))
            .unwrap_or_default(),
        // Local workspace fallback
        PathBuf::from("screenshots"),
    ];

    for path in &candidates {
        if path.exists() {
            return path.clone();
        }
    }

    // None exist yet — use the preferred location
    candidates[0].clone()
}

/// Generates a filename: `screenshot_YYYY-MM-DD_HH-MM-SS.png`
pub fn generate_screenshot_filename() -> String {
    let now = Local::now();
    now.format("screenshot_%Y-%m-%d_%H-%M-%S.png").to_string()
}
