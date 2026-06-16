use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::fs;
use uuid::Uuid;

use crate::config::StorageConfig;

pub async fn store_text(config: &StorageConfig, request_id: Uuid, text: &str) -> Result<Option<String>> {
    if !config.save_text {
        return Ok(None);
    }

    let directory = PathBuf::from(&config.directory);
    fs::create_dir_all(&directory)
        .await
        .with_context(|| format!("failed to create {}", directory.display()))?;

    let path = directory.join(format!("{request_id}.txt"));
    fs::write(&path, text)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(Some(path.to_string_lossy().into_owned()))
}
