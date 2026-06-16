use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::fs;
use uuid::Uuid;

use crate::config::StorageConfig;

#[derive(Debug, Default)]
pub struct StoredPaths {
    pub image: Option<String>,
    pub text: Option<String>,
}

pub async fn store(
    config: &StorageConfig,
    request_id: Uuid,
    mime: &str,
    image: &[u8],
    text: &str,
) -> Result<StoredPaths> {
    if !config.save_images && !config.save_text {
        return Ok(StoredPaths::default());
    }

    let directory = Path::new(&config.directory);
    fs::create_dir_all(directory)
        .await
        .with_context(|| format!("failed to create storage directory {}", directory.display()))?;

    let mut paths = StoredPaths::default();

    if config.save_images {
        let extension = match mime {
            "image/jpeg" => "jpg",
            _ => "png",
        };
        let path = directory.join(format!("{request_id}.{extension}"));
        fs::write(&path, image)
            .await
            .with_context(|| format!("failed to save image to {}", path.display()))?;
        paths.image = Some(display_path(path));
    }

    if config.save_text {
        let path = directory.join(format!("{request_id}.txt"));
        fs::write(&path, text)
            .await
            .with_context(|| format!("failed to save OCR text to {}", path.display()))?;
        paths.text = Some(display_path(path));
    }

    Ok(paths)
}

fn display_path(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
