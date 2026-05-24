use anyhow::{Result, anyhow};
use image::{DynamicImage, RgbaImage};
use xcap::Monitor;

pub struct CapturedScreen {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub image: RgbaImage,
}

pub fn capture_primary_monitor() -> Result<CapturedScreen> {
    let monitors = Monitor::all().map_err(|e| anyhow!("Failed to list monitors: {}", e))?;

    let primary = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or_else(|| anyhow!("No monitors detected on the system"))?;

    let monitor_name = primary.name()?;
    let width = primary.width()?;
    let height = primary.height()?;

    let xcap_image = primary.capture_image().map_err(|e| {
        anyhow!(
            "Hardware capture failed for monitor '{}': {}",
            monitor_name,
            e
        )
    })?;

    // Convert xcap's image wrapper into a standard image::DynamicImage
    // This automatically corrects pixel formats, padding, and row strides.
    let dynamic_img: DynamicImage = xcap_image.into();
    
    // Safely extract a perfectly aligned RgbaImage buffer
    let rgba_buffer = dynamic_img.to_rgba8();

    Ok(CapturedScreen {
        name: monitor_name,
        width,
        height,
        image: rgba_buffer,
    })
}
