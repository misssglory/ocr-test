use std::io::Cursor;

use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};
use xcap::Monitor;

pub struct CapturedFrame {
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
}

pub fn capture_full_screen() -> Result<CapturedFrame> {
    let monitors = Monitor::all().context("failed to enumerate monitors")?;
    if monitors.is_empty() {
        bail!("xcap found no monitors");
    }

    let monitor = monitors
        .iter()
        .find(|monitor| monitor.is_primary().unwrap_or(false))
        .unwrap_or(&monitors[0]);

    let monitor_name = monitor
        .name()
        .unwrap_or_else(|_| "unknown-monitor".to_owned());

    let image = monitor
        .capture_image()
        .with_context(|| format!("failed to capture monitor {monitor_name}"))?;

    let width = image.width();
    let height = image.height();
    let mut bytes = Vec::new();

    DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .context("failed to encode screenshot as PNG")?;

    let sha256 = hex::encode(Sha256::digest(&bytes));

    Ok(CapturedFrame {
        bytes,
        sha256,
        width,
        height,
        monitor_name,
    })
}
