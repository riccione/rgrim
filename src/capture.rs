use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use image::{DynamicImage, RgbaImage};
use xcap::Monitor;

/// Poll interval while waiting for the compositor to settle (~2.5 frames
/// at 60 Hz). A fade-out produces different pixels every refresh, so any
/// two consecutive identical captures mean presentation has caught up.
const SETTLE_POLL: Duration = Duration::from_millis(40);

/// Hard cap on settle-waiting. On perpetually animating desktops (video,
/// blinking cursors, live wallpapers) stability may never be observed, so
/// we hand back the newest frame rather than blocking indefinitely.
const SETTLE_MAX_WAIT: Duration = Duration::from_millis(1000);

pub struct CapturedScreen {
    pub name: String,
    pub image: RgbaImage,
}

pub fn capture_primary_monitor() -> Result<CapturedScreen> {
    let monitors = Monitor::all().context("Failed to list monitors")?;

    let primary = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or_else(|| anyhow!("No monitors detected on the system"))?;

    let monitor_name = primary.name()?;

    let xcap_image = primary
        .capture_image()
        .with_context(|| format!("Hardware capture failed for monitor '{}'", monitor_name))?;

    // Convert xcap's image wrapper into a standard image::DynamicImage
    // This automatically corrects pixel formats, padding, and row strides.
    let dynamic_img: DynamicImage = xcap_image.into();

    // Safely extract a perfectly aligned RgbaImage buffer
    let rgba_buffer = dynamic_img.to_rgba8();

    Ok(CapturedScreen {
        name: monitor_name,
        image: rgba_buffer,
    })
}

/// Captures the primary monitor once its output is stable across two
/// consecutive frames. Use after closing a window of ours: compositors
/// keep rendering the dying window (fade-out animations), so grabbing
/// immediately can bleed it into the next capture. A blind sleep is
/// either too short on slow compositors or wasted on fast ones; instead
/// poll until the screen stops changing, with [`SETTLE_MAX_WAIT`] as the
/// animated-desktop bail-out (newest frame returned on timeout).
pub fn capture_settled_monitor() -> Result<CapturedScreen> {
    let start = Instant::now();
    let mut previous = capture_primary_monitor()?;

    loop {
        std::thread::sleep(SETTLE_POLL);
        let current = capture_primary_monitor()?;
        if current.image == previous.image || start.elapsed() >= SETTLE_MAX_WAIT {
            return Ok(current);
        }
        previous = current;
    }
}
