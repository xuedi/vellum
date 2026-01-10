mod html;

pub use html::HtmlRenderer;

use crate::assets::Assets;
use crate::parser::Section;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Asset embedding failed: {0}")]
    AssetError(#[from] crate::assets::EmbedError),

    #[error("Invalid document structure: {0}")]
    StructureError(String),
}

#[derive(Debug)]
pub struct ParsedDocument {
    pub html_content: String,
    pub sections: Vec<Section>,
    pub title: String,
}

pub trait Renderer {
    fn render(&self, document: &ParsedDocument, logo_path: &str, assets: &Assets) -> Result<Vec<u8>, RenderError>;
}
