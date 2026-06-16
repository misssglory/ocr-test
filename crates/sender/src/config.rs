use std::{path::Path, time::Duration};

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SenderConfig {
    pub server: ServerConfig,
    pub capture: CaptureConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub token: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaptureConfig {
    pub device_id: String,
    pub monitor_index: Option<usize>,
    #[serde(default = "default_true")]
    pub prefer_primary: bool,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_jpeg_quality")]
    pub jpeg_quality: u8,
    pub region: Option<CaptureRegion>,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    #[serde(default)]
    pub send_unchanged: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl SenderConfig {
    pub fn load(path: &Path) -> Result<Self> {
        Figment::new()
            .merge(Toml::file(path))
            .extract()
            .with_context(|| format!("failed to load sender config from {}", path.display()))
    }
}

impl ServerConfig {
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    pub fn ocr_url(&self) -> String {
        format!("{}/v1/ocr", self.url.trim_end_matches('/'))
    }
}

fn default_true() -> bool {
    true
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_format() -> String {
    "png".to_owned()
}

fn default_jpeg_quality() -> u8 {
    90
}

fn default_interval_ms() -> u64 {
    1_000
}
