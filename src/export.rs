use std::path::PathBuf;
use std::sync::OnceLock;

use chrono::Local;

static SCREENSHOT_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Determines where to save screenshots.
/// Priority: `RGRIM_DIR` env var => `~/Pictures/Screenshots` => `XDG_PICTURES_DIR/Screenshots` => `./screenshots`
pub fn get_screenshot_directory() -> PathBuf {
    SCREENSHOT_DIR
        .get_or_init(|| {
            if let Ok(env_path) = std::env::var("RGRIM_DIR") {
                return PathBuf::from(env_path);
            }

            let home_screenshots =
                dirs::home_dir().map(|p| p.join("Pictures").join("Screenshots"));
            let pictures_screenshots = dirs::picture_dir().map(|p| p.join("Screenshots"));
            let local_fallback = PathBuf::from("screenshots");

            if let Some(path) = home_screenshots.as_ref().filter(|p| p.exists()) {
                return path.clone();
            }
            if let Some(path) = pictures_screenshots.as_ref().filter(|p| p.exists()) {
                return path.clone();
            }

            home_screenshots
                .or(pictures_screenshots)
                .unwrap_or(local_fallback)
        })
        .clone()
}

/// Generates a filename: `screenshot_YYYY-MM-DD_HH-MM-SS.png`
pub fn generate_screenshot_filename() -> String {
    let now = Local::now();
    now.format("screenshot_%Y-%m-%d_%H-%M-%S.png").to_string()
}
