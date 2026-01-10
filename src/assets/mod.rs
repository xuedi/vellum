mod embedder;

pub use embedder::{embed_image, EmbedError};

use std::path::Path;

#[derive(Debug, Clone)]
pub struct Assets {
    pub styles: String,
    pub script: String,
    pub template: String,
}

impl Assets {
    pub fn load(config_dir: &Path) -> Result<Self, EmbedError> {
        let assets_dir = config_dir.join("assets");

        let styles = std::fs::read_to_string(assets_dir.join("style.css")).map_err(|e| {
            EmbedError::ReadError {
                path: assets_dir.join("style.css").display().to_string(),
                source: e,
            }
        })?;

        let script = std::fs::read_to_string(assets_dir.join("script.js")).map_err(|e| {
            EmbedError::ReadError {
                path: assets_dir.join("script.js").display().to_string(),
                source: e,
            }
        })?;

        let template = std::fs::read_to_string(assets_dir.join("template.html")).map_err(|e| {
            EmbedError::ReadError {
                path: assets_dir.join("template.html").display().to_string(),
                source: e,
            }
        })?;

        Ok(Self {
            styles,
            script,
            template,
        })
    }
}
