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

fn default_trigger_bind() -> String {
    "127.0.0.1:4490".to_owned()
}

fn default_timeout_secs() -> u64 {
    30
}
