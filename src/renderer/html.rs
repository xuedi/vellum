use super::{ParsedDocument, RenderError, Renderer};
use crate::assets::{embed_image, Assets};
use crate::parser::Section;

#[derive(Debug, Default)]
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn new() -> Self {
        Self
    }

    fn wrap_sections(&self, html: &str, sections: &[Section]) -> String {
        if sections.is_empty() {
            return html.to_string();
        }

        let mut result = String::with_capacity(html.len() * 2);

        if sections[0].start > 0 {
            let pre_content = &html[..sections[0].start];
            if !pre_content.contains("<h1>") {
                result.push_str(pre_content);
            }
        }

        for section in sections {
            let section_content = &html[section.start..section.end];

            let content_start = section_content.find("</h2>").map(|p| p + 5).unwrap_or(0);
            let inner_content = &section_content[content_start..];

            result.push_str(&format!(
                r#"<div class="section" id="section-{}">
    <div class="section-header">
        <span class="toggle-icon">▼</span>
        <h2>{}</h2>
    </div>
    <div class="section-content">
        {}
    </div>
</div>
"#,
                section.id, section.title, inner_content
            ));
        }

        result
    }

    fn generate_nav_buttons(&self, sections: &[Section]) -> String {
        sections
            .iter()
            .map(|s| format!(r#"<button>{}</button>"#, s.title))
            .collect::<Vec<_>>()
            .join("\n        ")
    }

}

impl HtmlRenderer {
    pub fn render_with_options(
        &self,
        document: &ParsedDocument,
        logo_data_uri: &str,
        dropdown_section: Option<&str>,
        assets: &Assets,
    ) -> Result<Vec<u8>, RenderError> {
        let wrapped_content = self.wrap_sections(&document.html_content, &document.sections);

        let nav_buttons = self.generate_nav_buttons(&document.sections);

        let script = if let Some(section_name) = dropdown_section {
            format!(
                "const DROPDOWN_SECTION = \"{}\";\n{}",
                section_name.replace('\"', "\\\""),
                &assets.script
            )
        } else {
            format!("const DROPDOWN_SECTION = null;\n{}", &assets.script)
        };

        let html = assets
            .template
            .replace("{{title}}", &document.title)
            .replace("{{styles}}", &assets.styles)
            .replace("{{logo}}", logo_data_uri)
            .replace("{{nav_buttons}}", &nav_buttons)
            .replace("{{content}}", &wrapped_content)
            .replace("{{script}}", &script);

        Ok(html.into_bytes())
    }
}

impl Renderer for HtmlRenderer {
    fn render(&self, document: &ParsedDocument, logo_path: &str, assets: &Assets) -> Result<Vec<u8>, RenderError> {
        let logo_data_uri = embed_image(logo_path)?;
        self.render_with_options(document, &logo_data_uri, Some("Related Documents"), assets)
    }
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
    fn test_nav_button_generation() {
        let renderer = HtmlRenderer::new();
        let sections = vec![
            Section {
                id: "overview".to_string(),
                title: "Overview".to_string(),
                start: 0,
                end: 100,
            },
            Section {
                id: "details".to_string(),
                title: "Details".to_string(),
                start: 100,
                end: 200,
            },
        ];

        let nav = renderer.generate_nav_buttons(&sections);
        assert!(nav.contains("<button>Overview</button>"));
        assert!(nav.contains("<button>Details</button>"));
    }

    #[test]
    fn test_nav_button_empty_sections() {
        let renderer = HtmlRenderer::new();
        let sections: Vec<Section> = vec![];
        let nav = renderer.generate_nav_buttons(&sections);
        assert!(nav.is_empty());
    }

    #[test]
    fn test_wrap_sections_empty() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Some content</p>";
        let sections: Vec<Section> = vec![];
        let result = renderer.wrap_sections(html, &sections);
        assert_eq!(result, html);
    }

    #[test]
    fn test_wrap_sections_single() {
        let renderer = HtmlRenderer::new();
        let html = "<h2>Overview</h2><p>Content here</p>";
        let sections = vec![Section {
            id: "overview".to_string(),
            title: "Overview".to_string(),
            start: 0,
            end: html.len(),
        }];

        let result = renderer.wrap_sections(html, &sections);
        assert!(result.contains("section-overview"));
        assert!(result.contains("section-header"));
        assert!(result.contains("section-content"));
        assert!(result.contains("toggle-icon"));
    }

    #[test]
    fn test_wrap_sections_multiple() {
        let renderer = HtmlRenderer::new();
        let html = "<h2>First</h2><p>A</p><h2>Second</h2><p>B</p>";
        let sections = vec![
            Section {
                id: "first".to_string(),
                title: "First".to_string(),
                start: 0,
                end: 19,
            },
            Section {
                id: "second".to_string(),
                title: "Second".to_string(),
                start: 19,
                end: html.len(),
            },
        ];

        let result = renderer.wrap_sections(html, &sections);
        assert!(result.contains("section-first"));
        assert!(result.contains("section-second"));
    }

    #[test]
    fn test_wrap_sections_with_h1_prefix() {
        let renderer = HtmlRenderer::new();
        let html = "<h1>Title</h1><h2>Overview</h2><p>Content</p>";
        let sections = vec![Section {
            id: "overview".to_string(),
            title: "Overview".to_string(),
            start: 14,
            end: html.len(),
        }];

        let result = renderer.wrap_sections(html, &sections);
        assert!(!result.contains("<h1>Title</h1>"));
        assert!(result.contains("section-overview"));
    }

    #[test]
    fn test_render_full_document() {
        let renderer = HtmlRenderer::new();

        let dir = std::env::temp_dir();
        let logo_path = dir.join("test_logo.png");
        let png_bytes: [u8; 67] = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        std::fs::write(&logo_path, &png_bytes).unwrap();

        let document = ParsedDocument {
            html_content: "<h2>Test</h2><p>Content</p>".to_string(),
            sections: vec![Section {
                id: "test".to_string(),
                title: "Test".to_string(),
                start: 0,
                end: 27,
            }],
            title: "Test Document".to_string(),
        };

        let assets = test_assets();
        let result = renderer.render(&document, logo_path.to_str().unwrap(), &assets);
        assert!(result.is_ok());

        let html = String::from_utf8(result.unwrap()).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Document</title>"));
        assert!(html.contains("data:image/png;base64,"));
        assert!(html.contains("<button>Test</button>"));
        assert!(html.contains("section-test"));

        std::fs::remove_file(&logo_path).ok();
    }

    #[test]
    fn test_render_missing_logo() {
        let renderer = HtmlRenderer::new();
        let document = ParsedDocument {
            html_content: "<p>Content</p>".to_string(),
            sections: vec![],
            title: "Test".to_string(),
        };

        let assets = test_assets();
        let result = renderer.render(&document, "nonexistent_logo.png", &assets);
        assert!(result.is_err());
    }
}
