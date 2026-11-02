//! Monitor screen capture (backed by xcap).

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

/// One frame grabbed from a monitor.
pub struct CapturedScreen {
    /// Display name as reported by the OS (e.g. `eDP-1`).
    pub name: String,
    /// Raw RGBA pixels at the monitor's native resolution.
    pub image: RgbaImage,
}

/// Enumerates monitors and returns the primary one, falling back to the
/// first enumerated monitor when none is flagged primary.
fn primary_monitor() -> Result<Monitor> {
    let monitors = Monitor::all().context("Failed to list monitors")?;

    let index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);

    monitors
        .into_iter()
        .nth(index)
        .ok_or_else(|| anyhow!("No monitors detected on the system"))
}

/// Grabs one frame from `monitor`. `name` is only used to enrich errors.
fn capture_monitor(monitor: &Monitor, name: &str) -> Result<RgbaImage> {
    let xcap_image = monitor
        .capture_image()
        .with_context(|| format!("Hardware capture failed for monitor '{name}'"))?;

    // Convert xcap's image wrapper into a standard image::DynamicImage
    // This automatically corrects pixel formats, padding, and row strides.
    let dynamic_img: DynamicImage = xcap_image.into();

    // Safely extract a perfectly aligned RgbaImage buffer
    Ok(dynamic_img.to_rgba8())
}

/// Captures the primary monitor, falling back to the first enumerated
/// monitor when none is flagged primary.
pub fn capture_primary_monitor() -> Result<CapturedScreen> {
    let monitor = primary_monitor()?;
    let name = monitor.name()?;
    let image = capture_monitor(&monitor, &name)?;

    Ok(CapturedScreen { name, image })
}

/// Captures the primary monitor once its output is stable across two
/// consecutive frames. Use after closing a window of ours: compositors
/// keep rendering the dying window (fade-out animations), so grabbing
/// immediately can bleed it into the next capture. A blind sleep is
/// either too short on slow compositors or wasted on fast ones; instead
/// poll until the screen stops changing, with `SETTLE_MAX_WAIT` as the
/// animated-desktop bail-out (newest frame returned on timeout).
pub fn capture_settled_monitor() -> Result<CapturedScreen> {
    let monitor = primary_monitor()?;
    let name = monitor.name()?;
    let start = Instant::now();
    let mut previous = capture_monitor(&monitor, &name)?;

    loop {
        std::thread::sleep(SETTLE_POLL);
        let current = capture_monitor(&monitor, &name)?;
        if current == previous || start.elapsed() >= SETTLE_MAX_WAIT {
            return Ok(CapturedScreen {
                name,
                image: current,
            });
        }
        previous = current;
    }
}
