use std::io::Write;
use std::process::{Command, Stdio};

use image::ImageEncoder;
use image::RgbaImage;

use anyhow::{Result, anyhow};

pub(crate) fn copy_to_clipboard(image: &RgbaImage) -> Result<()> {
    let w = image.width() as usize;
    let h = image.height() as usize;
    let bytes = image.as_raw().to_vec();

    let img_data = arboard::ImageData {
        width: w,
        height: h,
        bytes: std::borrow::Cow::Owned(bytes),
    };

    let clipboard = arboard::Clipboard::new();
    if let Ok(mut cb) = clipboard
        && cb.set_image(img_data).is_ok()
    {
        return Ok(());
    }

    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    image::codecs::png::PngEncoder::new(&mut cursor)
        .write_image(
            image.as_raw(),
            w as u32,
            h as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| anyhow!("Failed to encode clipboard buffer to PNG: {}", e))?;

    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| {
            anyhow!(
                "Both arboard and 'wl-copy' failed. If on a pure Wayland compositor, ensure 'wl-clipboard' is installed: {}",
                e
            )
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&png_bytes)?;
    }

    let wl_status = child.wait()?;
    if !wl_status.success() {
        return Err(anyhow!("wl-copy exited with an error status code"));
    }

    Ok(())
}

pub(crate) fn save_to_file(image: &RgbaImage) -> Result<String> {
    let output_dir = crate::export::get_screenshot_directory();
    std::fs::create_dir_all(&output_dir)?;

    let filename = crate::export::generate_screenshot_filename();
    let path = output_dir.join(&filename);

    image.save(&path)?;
    Ok(filename)
}
