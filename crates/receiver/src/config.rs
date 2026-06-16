use std::path::Path;

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ReceiverConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
    pub token: String,
    #[serde(default = "default_max_text_kb")]
    pub max_text_kb: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
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

fn default_bind() -> String { "0.0.0.0:4489".to_owned() }
fn default_max_text_kb() -> usize { 1024 }
fn default_storage_dir() -> String { "./received".to_owned() }
