use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("Failed to read image file '{path}': {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },

    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
}

pub fn embed_image<P: AsRef<Path>>(path: P) -> Result<String, EmbedError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    let bytes = std::fs::read(path).map_err(|e| EmbedError::ReadError {
        path: path_str.clone(),
        source: e,
    })?;

    let mime = match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some(ext) => return Err(EmbedError::UnsupportedFormat(ext.to_string())),
        None => return Err(EmbedError::UnsupportedFormat("unknown".to_string())),
    };

    let encoded = STANDARD.encode(&bytes);

    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_png() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_image.png");
        let png_bytes: [u8; 67] = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        std::fs::write(&path, &png_bytes).unwrap();

        let result = embed_image(&path).unwrap();
        assert!(result.starts_with("data:image/png;base64,"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_embed_jpeg() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_image.jpg");
        let jpeg_bytes: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0];
        std::fs::write(&path, &jpeg_bytes).unwrap();

        let result = embed_image(&path).unwrap();
        assert!(result.starts_with("data:image/jpeg;base64,"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_embed_gif() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_image.gif");
        let gif_bytes = b"GIF89a";
        std::fs::write(&path, gif_bytes).unwrap();

        let result = embed_image(&path).unwrap();
        assert!(result.starts_with("data:image/gif;base64,"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_embed_svg() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_image.svg");
        let svg_content = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        std::fs::write(&path, svg_content).unwrap();

        let result = embed_image(&path).unwrap();
        assert!(result.starts_with("data:image/svg+xml;base64,"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_embed_webp() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_image.webp");
        let webp_bytes = b"RIFF\x00\x00\x00\x00WEBP";
        std::fs::write(&path, webp_bytes).unwrap();

        let result = embed_image(&path).unwrap();
        assert!(result.starts_with("data:image/webp;base64,"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_unsupported_format() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_file.txt");
        std::fs::write(&path, "test").unwrap();

        let result = embed_image(&path);
        assert!(matches!(result, Err(EmbedError::UnsupportedFormat(ext)) if ext == "txt"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_no_extension() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_file_no_ext");
        std::fs::write(&path, "test").unwrap();

        let result = embed_image(&path);
        assert!(matches!(result, Err(EmbedError::UnsupportedFormat(ext)) if ext == "unknown"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_missing_file() {
        let result = embed_image("nonexistent.png");
        assert!(matches!(result, Err(EmbedError::ReadError { .. })));
    }
}
