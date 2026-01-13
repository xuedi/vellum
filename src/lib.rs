pub mod assets;
pub mod parser;
pub mod renderer;

use assets::{embed_image, Assets};
use parser::{
    parse_document_structure, process_includes, substitute_variables,
    transform_achievement_markers, transform_skill_matrices,
};
use renderer::{HtmlRenderer, RenderError};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub markdown_path: String,
    pub logo_path: String,
    pub title: String,
    pub output_path: String,
    pub dropdown_section: Option<String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            markdown_path: "example/index.md".to_string(),
            logo_path: "example/logo.png".to_string(),
            title: "My Portfolio".to_string(),
            output_path: "output/index.html".to_string(),
            dropdown_section: Some("Projects".to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    document: DocumentConfig,
    paths: PathsConfig,
}

#[derive(Debug, Deserialize)]
struct DocumentConfig {
    title: String,
    #[serde(default)]
    dropdown: String,
}

#[derive(Debug, Deserialize)]
struct PathsConfig {
    markdown: String,
    logo: String,
    output: String,
}

impl GeneratorConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, GeneratorError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            GeneratorError::ConfigReadError {
                path: path.as_ref().display().to_string(),
                source: e,
            }
        })?;

        let config_file: ConfigFile = toml::from_str(&content).map_err(|e| {
            GeneratorError::ConfigParseError {
                path: path.as_ref().display().to_string(),
                message: e.to_string(),
            }
        })?;

        let dropdown_section = if config_file.document.dropdown.is_empty() {
            None
        } else {
            Some(config_file.document.dropdown)
        };

        Ok(Self {
            markdown_path: config_file.paths.markdown,
            logo_path: config_file.paths.logo,
            title: config_file.document.title,
            output_path: config_file.paths.output,
            dropdown_section,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    pub source_lines: usize,
    pub expanded_lines: usize,
    pub achievement_markers: usize,
    pub html_content_size: usize,
    pub section_count: usize,
}

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Failed to read markdown file '{path}': {source}")]
    MarkdownReadError {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to read config file '{path}': {source}")]
    ConfigReadError {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to parse config file '{path}': {message}")]
    ConfigParseError { path: String, message: String },

    #[error("Input file not found: {0}")]
    InputNotFound(String),

    #[error("Asset embedding failed: {0}")]
    AssetError(#[from] assets::EmbedError),

    #[error("Rendering failed: {0}")]
    RenderError(#[from] RenderError),
}

pub fn generate_html_from_content(
    markdown: &str,
    base_path: &str,
    title: &str,
    logo_data_uri: &str,
    dropdown_section: Option<&str>,
    assets: &Assets,
) -> Result<(Vec<u8>, GenerationStats), GeneratorError> {
    let mut stats = GenerationStats {
        source_lines: markdown.lines().count(),
        ..Default::default()
    };

    // Step 1: Process includes
    let with_includes = process_includes(markdown, base_path, dropdown_section);
    stats.expanded_lines = with_includes.lines().count();

    // Step 2: Substitute variables
    let with_variables = substitute_variables(&with_includes);

    // Step 3: Transform achievement markers
    let transformed = transform_achievement_markers(&with_variables);
    stats.achievement_markers = transformed.matches("achievement-marker").count();

    // Step 4: Transform skill matrices
    let with_skill_matrices = transform_skill_matrices(&transformed);

    // Step 5: Parse document structure (extracts sections and dropdown items)
    let doc_structure = parse_document_structure(&with_skill_matrices, dropdown_section);
    stats.section_count = doc_structure.nav_buttons.len() + doc_structure.dropdown_items.len();

    // Step 6: Render using the new panel-based approach
    let renderer = HtmlRenderer::new();
    let output = renderer.render_from_structure(&doc_structure, title, logo_data_uri, assets)?;

    // Calculate HTML content size from output
    stats.html_content_size = output.len();

    Ok((output, stats))
}

pub fn generate_html(config: &GeneratorConfig, assets: &Assets) -> Result<(Vec<u8>, GenerationStats), GeneratorError> {
    let markdown = std::fs::read_to_string(&config.markdown_path).map_err(|e| {
        GeneratorError::MarkdownReadError {
            path: config.markdown_path.clone(),
            source: e,
        }
    })?;

    let base_path = Path::new(&config.markdown_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let logo_data_uri = embed_image(&config.logo_path)?;

    let dropdown_section = config.dropdown_section.as_deref();
    generate_html_from_content(&markdown, &base_path, &config.title, &logo_data_uri, dropdown_section, assets)
}

pub fn validate_inputs(config: &GeneratorConfig) -> Result<(), GeneratorError> {
    if !Path::new(&config.markdown_path).exists() {
        return Err(GeneratorError::InputNotFound(config.markdown_path.clone()));
    }
    if !Path::new(&config.logo_path).exists() {
        return Err(GeneratorError::InputNotFound(config.logo_path.clone()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_assets() -> Assets {
        Assets {
            styles: "body { color: black; }".to_string(),
            script: "console.log('test');".to_string(),
            template: r#"<!DOCTYPE html>
<html>
<head><title>{{title}}</title><style>{{styles}}</style></head>
<body>
<img src="{{logo}}" />
<nav>{{nav_buttons}}</nav>
<main>{{content}}</main>
<script>{{script}}</script>
</body>
</html>"#
                .to_string(),
        }
    }

    #[test]
    fn test_generate_html_from_content_basic() {
        let markdown = "# Title\n\n## Section One\n\nSome content here.";
        let logo_uri = "data:image/png;base64,AAAA";
        let assets = test_assets();

        let result = generate_html_from_content(markdown, ".", "Test Doc", logo_uri, None, &assets);
        assert!(result.is_ok());

        let (html, stats) = result.unwrap();
        assert!(!html.is_empty());
        assert_eq!(stats.source_lines, 5);
        assert_eq!(stats.section_count, 1);
    }

    #[test]
    fn test_generate_html_from_content_with_markers() {
        let markdown = "## Log\n\n- 2024-01-01: Did something <! milestone";
        let logo_uri = "data:image/png;base64,AAAA";
        let assets = test_assets();

        let (_, stats) = generate_html_from_content(markdown, ".", "Test", logo_uri, None, &assets).unwrap();
        assert_eq!(stats.achievement_markers, 1);
    }

    #[test]
    fn test_generate_html_from_content_multiple_sections() {
        let markdown = "## One\n\nA\n\n## Two\n\nB\n\n## Three\n\nC";
        let logo_uri = "data:image/png;base64,AAAA";
        let assets = test_assets();

        let (_, stats) = generate_html_from_content(markdown, ".", "Test", logo_uri, None, &assets).unwrap();
        assert_eq!(stats.section_count, 3);
    }

    #[test]
    fn test_generation_stats_default() {
        let stats = GenerationStats::default();
        assert_eq!(stats.source_lines, 0);
        assert_eq!(stats.expanded_lines, 0);
        assert_eq!(stats.achievement_markers, 0);
    }

    #[test]
    fn test_validate_inputs_missing_markdown() {
        let config = GeneratorConfig {
            markdown_path: "nonexistent.md".to_string(),
            logo_path: "logo.png".to_string(),
            title: "Test".to_string(),
            output_path: "output/test.html".to_string(),
            dropdown_section: None,
        };
        let result = validate_inputs(&config);
        assert!(matches!(result, Err(GeneratorError::InputNotFound(_))));
    }

    #[test]
    fn test_generator_config_default() {
        let config = GeneratorConfig::default();
        assert_eq!(config.markdown_path, "example/index.md");
        assert_eq!(config.logo_path, "example/logo.png");
        assert_eq!(config.title, "My Portfolio");
        assert_eq!(config.output_path, "output/index.html");
        assert_eq!(config.dropdown_section, Some("Projects".to_string()));
    }
}
