use std::io::Cursor;

use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageFormat, codecs::jpeg::JpegEncoder};
use sha2::{Digest, Sha256};
use xcap::Monitor;

use crate::config::CaptureConfig;

#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
}

pub fn list_monitors() -> Result<()> {
    let monitors = Monitor::all().context("failed to enumerate monitors")?;

    if monitors.is_empty() {
        bail!("no monitors were found");
    }

    for (index, monitor) in monitors.iter().enumerate() {
        let name = monitor
            .friendly_name()
            .unwrap_or_else(|_| "unknown".to_owned());
        let width = monitor.width().unwrap_or_default();
        let height = monitor.height().unwrap_or_default();
        let primary = monitor.is_primary().unwrap_or(false);

        println!("{index}: {name} — {width}x{height}, primary={primary}");
    }

    Ok(())
}

pub fn capture(config: &CaptureConfig) -> Result<CapturedFrame> {
    let monitors = Monitor::all().context("failed to enumerate monitors")?;
    let monitor = select_monitor(monitors, config)?;

    let image = match config.region {
        Some(region) => monitor
            .capture_region(region.x, region.y, region.width, region.height)
            .context("failed to capture configured screen region")?,
        None => monitor
            .capture_image()
            .context("failed to capture monitor")?,
    };

    let width = image.width();
    let height = image.height();
    let dynamic = DynamicImage::ImageRgba8(image);
    let (bytes, mime) = encode(dynamic, &config.format, config.jpeg_quality)?;
    let sha256 = hex::encode(Sha256::digest(&bytes));

    Ok(CapturedFrame {
        bytes,
        mime,
        sha256,
        width,
        height,
    })
}

fn select_monitor(monitors: Vec<Monitor>, config: &CaptureConfig) -> Result<Monitor> {
    if monitors.is_empty() {
        bail!("no monitors were found");
    }

    if let Some(index) = config.monitor_index {
        return monitors.into_iter().nth(index).with_context(|| {
            format!("monitor_index {index} is out of range; run `screen-ocr-sender list-monitors`")
        });
    }

    if config.prefer_primary {
        if let Some(primary) = monitors
            .iter()
            .position(|monitor| monitor.is_primary().unwrap_or(false))
        {
            return monitors.into_iter().nth(primary).context("primary monitor vanished");
        }
    }

    monitors.into_iter().next().context("no monitor available")
}

fn encode(image: DynamicImage, format: &str, jpeg_quality: u8) -> Result<(Vec<u8>, &'static str)> {
    let mut cursor = Cursor::new(Vec::new());

    match format.to_ascii_lowercase().as_str() {
        "png" => {
            image
                .write_to(&mut cursor, ImageFormat::Png)
                .context("failed to encode screenshot as PNG")?;
            Ok((cursor.into_inner(), "image/png"))
        }
        "jpg" | "jpeg" => {
            if !(1..=100).contains(&jpeg_quality) {
                bail!("jpeg_quality must be between 1 and 100");
            }

            JpegEncoder::new_with_quality(&mut cursor, jpeg_quality)
                .encode_image(&image)
                .context("failed to encode screenshot as JPEG")?;
            Ok((cursor.into_inner(), "image/jpeg"))
        }
        other => bail!("unsupported capture format `{other}`; use `png` or `jpeg`"),
    }
}
