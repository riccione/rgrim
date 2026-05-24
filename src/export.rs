use std::path::PathBuf;

use chrono::Local;

/// Determines where to save screenshots.
/// Priority: `RGRIM_DIR` env var => `~/Pictures/Screenshots` => `./screenshots`
pub fn get_screenshot_directory() -> PathBuf {
    if let Ok(env_path) = std::env::var("RGRIM_DIR") {
        return PathBuf::from(env_path);
    }

    if let Some(mut home_dir) = dirs::picture_dir() {
        home_dir.push("Screenshots");
        return home_dir;
    }

    PathBuf::from("screenshots")
}

/// Generates a filename: `screenshot_YYYY-MM-DD_HH-MM-SS.png`
pub fn generate_screenshot_filename() -> String {
    let now = Local::now();
    now.format("screenshot_%Y-%m-%d_%H-%M-%S.png").to_string()
}
