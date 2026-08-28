use std::process::Stdio;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use tokio::process::Command;

use crate::config::CaptureConfig;

#[derive(Debug)]
pub struct CapturedRegion {
    pub png: Vec<u8>,
    pub sha256: String,
    pub geometry: String,
    pub width: u32,
    pub height: u32,
}

pub async fn capture_selected_region(config: &CaptureConfig) -> Result<CapturedRegion> {
    let slurp = Command::new(&config.slurp_command)
        .args(["-f", "%x,%y %wx%h"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| format!("failed to start {}", config.slurp_command))?;

    if !slurp.status.success() {
        let stderr = String::from_utf8_lossy(&slurp.stderr).trim().to_owned();
        bail!("region selection cancelled or slurp failed: {stderr}");
    }

    let geometry = String::from_utf8(slurp.stdout)
        .context("slurp returned non-UTF-8 geometry")?
        .trim()
        .to_owned();

    let (width, height) = parse_geometry(&geometry)?;

    let grim = Command::new(&config.grim_command)
        .args(["-g", &geometry, "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .with_context(|| format!("failed to start {}", config.grim_command))?;

    if !grim.status.success() {
        let stderr = String::from_utf8_lossy(&grim.stderr).trim().to_owned();
        bail!("grim failed to capture selected region: {stderr}");
    }

    if grim.stdout.is_empty() {
        bail!("grim returned an empty PNG");
    }

    let sha256 = hex::encode(Sha256::digest(&grim.stdout));

    Ok(CapturedRegion {
        png: grim.stdout,
        sha256,
        geometry,
        width,
        height,
    })
}

fn parse_geometry(geometry: &str) -> Result<(u32, u32)> {
    let (_, size) = geometry
        .split_once(' ')
        .with_context(|| format!("invalid slurp geometry {geometry:?}"))?;
    let (width, height) = size
        .split_once('x')
        .with_context(|| format!("invalid slurp size {size:?}"))?;

    let width: u32 = width.parse().context("invalid selected width")?;
    let height: u32 = height.parse().context("invalid selected height")?;

    if width == 0 || height == 0 {
        bail!("selected region has zero size");
    }

    Ok((width, height))
}
