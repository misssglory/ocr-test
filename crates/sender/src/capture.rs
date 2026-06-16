use std::io::Cursor;

use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};
use xcap::Monitor;

#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
}

/// Capture the complete primary monitor.
///
/// If xcap cannot identify a primary monitor, the first enumerated monitor is
/// used as a fallback. This avoids all focused-window and region-coordinate
/// handling, which is considerably less reliable under wlroots compositors.
pub fn capture_full_screen() -> Result<CapturedFrame> {
    let monitors = Monitor::all().context("failed to enumerate monitors")?;

    if monitors.is_empty() {
        bail!("xcap did not report any monitors");
    }

    let primary_index = monitors
        .iter()
        .position(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(0);

    let monitor = &monitors[primary_index];
    let monitor_name = monitor
        .name()
        .unwrap_or_else(|_| format!("monitor-{primary_index}"));

    let image = monitor
        .capture_image()
        .with_context(|| format!("failed to capture full monitor `{monitor_name}`"))?;

    let width = image.width();
    let height = image.height();

    if width == 0 || height == 0 {
        bail!("monitor capture produced an empty image");
    }

    let mut cursor = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, ImageFormat::Png)
        .context("failed to encode full-screen capture as PNG")?;

    let bytes = cursor.into_inner();
    let sha256 = hex::encode(Sha256::digest(&bytes));

    Ok(CapturedFrame {
        bytes,
        mime: "image/png",
        sha256,
        width,
        height,
        monitor_name,
    })
}
