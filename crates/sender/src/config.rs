use std::{path::Path, time::Duration};

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SenderConfig {
    pub trigger: TriggerConfig,
    pub server: ServerConfig,
    pub capture: CaptureConfig,
    pub ocr: OcrConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerConfig {
    #[serde(default = "default_trigger_bind")]
    pub bind: String,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct OcrConfig {
    #[serde(default = "default_tesseract")]
    pub command: String,
    #[serde(default = "default_languages")]
    pub languages: String,
    #[serde(default = "default_psm")]
    pub psm: u8,
    #[serde(default = "default_oem")]
    pub oem: u8,
    #[serde(default = "default_ocr_timeout_secs")]
    pub timeout_secs: u64,
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

    pub fn text_url(&self) -> String {
        format!("{}/v1/text", self.url.trim_end_matches('/'))
    }
}

impl OcrConfig {
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

fn default_trigger_bind() -> String { "127.0.0.1:4490".to_owned() }
fn default_timeout_secs() -> u64 { 30 }
fn default_tesseract() -> String { "tesseract".to_owned() }
fn default_languages() -> String { "eng".to_owned() }
fn default_psm() -> u8 { 6 }
fn default_oem() -> u8 { 1 }
fn default_ocr_timeout_secs() -> u64 { 30 }
