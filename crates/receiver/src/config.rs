use std::{path::Path, time::Duration};

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ReceiverConfig {
    pub server: ServerConfig,
    pub ocr: OcrConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
    pub token: String,
    #[serde(default = "default_max_upload_mb")]
    pub max_upload_mb: usize,
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
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub save_images: bool,
    #[serde(default)]
    pub save_text: bool,
    #[serde(default = "default_storage_dir")]
    pub directory: String,
}

impl ReceiverConfig {
    pub fn load(path: &Path) -> Result<Self> {
        Figment::new()
            .merge(Toml::file(path))
            .extract()
            .with_context(|| format!("failed to load receiver config from {}", path.display()))
    }
}

impl OcrConfig {
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

fn default_bind() -> String {
    "0.0.0.0:4489".to_owned()
}

fn default_max_upload_mb() -> usize {
    20
}

fn default_tesseract() -> String {
    "tesseract".to_owned()
}

fn default_languages() -> String {
    "eng".to_owned()
}

fn default_psm() -> u8 {
    6
}

fn default_oem() -> u8 {
    1
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_storage_dir() -> String {
    "./received".to_owned()
}
