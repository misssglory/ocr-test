use std::io::Cursor;

use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};
use xcap::Window;

#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub window_title: String,
    pub app_name: String,
}

pub fn capture_focused_window() -> Result<CapturedFrame> {
    let windows = Window::all().context("failed to enumerate windows")?;

    let window = windows
        .into_iter()
        .find(|window| {
            let focused = window.is_focused().unwrap_or(false);
            let minimized = window.is_minimized().unwrap_or(true);
            focused && !minimized
        })
        .context("no focused, capturable window was found")?;

    let window_title = window.title().unwrap_or_else(|_| "unknown".to_owned());
    let app_name = window.app_name().unwrap_or_else(|_| "unknown".to_owned());

    let image = window
        .capture_image()
        .with_context(|| format!("failed to capture focused window `{window_title}`"))?;

    let width = image.width();
    let height = image.height();

    if width == 0 || height == 0 {
        bail!("focused window produced an empty image");
    }

    let mut cursor = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, ImageFormat::Png)
        .context("failed to encode focused-window capture as PNG")?;

    let bytes = cursor.into_inner();
    let sha256 = hex::encode(Sha256::digest(&bytes));

    Ok(CapturedFrame {
        bytes,
        mime: "image/png",
        sha256,
        width,
        height,
        window_title,
        app_name,
    })
}
