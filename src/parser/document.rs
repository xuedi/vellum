/// Document structure parsing module.
///
/// Parses processed markdown into a tree structure that separates
/// regular sections from dropdown items for pre-rendering.

use super::sections::slugify;

/// A navigation item (either button or dropdown option)
#[derive(Debug, Clone)]
pub struct NavItem {
    pub id: String,
    pub title: String,
}

/// A content panel with pre-rendered HTML
#[derive(Debug, Clone)]
pub struct ContentPanel {
    pub id: String,
    pub title: String,
    pub markdown_content: String,
    pub is_dropdown_item: bool,
}

/// Parsed document structure
#[derive(Debug)]
pub struct DocumentStructure {
    pub nav_buttons: Vec<NavItem>,
    pub dropdown_title: Option<String>,
    pub dropdown_items: Vec<NavItem>,
    pub panels: Vec<ContentPanel>,
}

/// Parse processed markdown into a document structure.
///
/// Identifies H2 sections as nav buttons, and if a dropdown section is configured,
/// extracts H3 subsections within that section as dropdown items.
pub fn parse_document_structure(markdown: &str, dropdown_section: Option<&str>) -> DocumentStructure {
    let mut nav_buttons = Vec::new();
    let mut dropdown_title = None;
    let mut dropdown_items = Vec::new();
    let mut panels = Vec::new();
    let mut used_ids: Vec<String> = Vec::new();

    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check for H2 heading (section)
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            let title = trimmed[3..].trim().to_string();
            let id = generate_unique_id(&title, &mut used_ids);

            // Check if this is the dropdown section
            if dropdown_section.is_some() && dropdown_section.unwrap() == title {
                dropdown_title = Some(title.clone());

                // Extract H3 subsections within this section
                i += 1;
                while i < lines.len() {
                    let sub_line = lines[i];
                    let sub_trimmed = sub_line.trim();

                    // Stop at next H2
                    if sub_trimmed.starts_with("## ") && !sub_trimmed.starts_with("### ") {
                        break;
                    }

                    // Found an H3 subsection
                    if sub_trimmed.starts_with("### ") && !sub_trimmed.starts_with("#### ") {
                        let sub_title = sub_trimmed[4..].trim().to_string();
                        let sub_id = generate_unique_id(&sub_title, &mut used_ids);

                        // Collect content until next H3 or H2
                        let mut content_lines = Vec::new();
                        i += 1;
                        while i < lines.len() {
                            let content_line = lines[i];
                            let content_trimmed = content_line.trim();

                            if content_trimmed.starts_with("## ")
                                || (content_trimmed.starts_with("### ")
                                    && !content_trimmed.starts_with("#### "))
                            {
                                break;
                            }
                            content_lines.push(content_line);
                            i += 1;
                        }

                        dropdown_items.push(NavItem {
                            id: sub_id.clone(),
                            title: sub_title.clone(),
                        });

                        panels.push(ContentPanel {
                            id: sub_id,
                            title: sub_title,
                            markdown_content: content_lines.join("\n"),
                            is_dropdown_item: true,
                        });

                        continue;
                    }
                    i += 1;
                }
                continue;
            }

            // Regular section - collect content until next H2
            let mut content_lines = Vec::new();
            i += 1;
            while i < lines.len() {
                let content_line = lines[i];
                let content_trimmed = content_line.trim();

                if content_trimmed.starts_with("## ") && !content_trimmed.starts_with("### ") {
                    break;
                }
                content_lines.push(content_line);
                i += 1;
            }

            nav_buttons.push(NavItem {
                id: id.clone(),
                title: title.clone(),
            });

            panels.push(ContentPanel {
                id,
                title,
                markdown_content: content_lines.join("\n"),
                is_dropdown_item: false,
            });

            continue;
        }

        i += 1;
    }

    DocumentStructure {
        nav_buttons,
        dropdown_title,
        dropdown_items,
        panels,
    }
}

/// Generate a unique ID from a title, avoiding duplicates
fn generate_unique_id(title: &str, used_ids: &mut Vec<String>) -> String {
    let base_id = slugify(title);
    let mut id = base_id.clone();
    let mut counter = 1;

    while used_ids.contains(&id) {
        id = format!("{}-{}", base_id, counter);
        counter += 1;
    }

    used_ids.push(id.clone());
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_sections() {
        let markdown = "## Overview\nContent 1\n\n## Projects\nContent 2\n";
        let doc = parse_document_structure(markdown, None);

        assert_eq!(doc.nav_buttons.len(), 2);
        assert_eq!(doc.nav_buttons[0].title, "Overview");
        assert_eq!(doc.nav_buttons[1].title, "Projects");
        assert_eq!(doc.panels.len(), 2);
        assert!(doc.dropdown_title.is_none());
    }

    #[test]
    fn test_parse_with_dropdown() {
        let markdown = r#"## Overview
Content 1

## More
### Projects
Project content

### Worklog
Worklog content
"#;
        let doc = parse_document_structure(markdown, Some("More"));

        assert_eq!(doc.nav_buttons.len(), 1);
        assert_eq!(doc.nav_buttons[0].title, "Overview");
        assert_eq!(doc.dropdown_title, Some("More".to_string()));
        assert_eq!(doc.dropdown_items.len(), 2);
        assert_eq!(doc.dropdown_items[0].title, "Projects");
        assert_eq!(doc.dropdown_items[1].title, "Worklog");
        assert_eq!(doc.panels.len(), 3);
    }

    #[test]
    fn test_unique_ids() {
        let markdown = "## Test\nContent 1\n\n## Test\nContent 2\n";
        let doc = parse_document_structure(markdown, None);

        assert_eq!(doc.nav_buttons[0].id, "test");
        assert_eq!(doc.nav_buttons[1].id, "test-1");
    }

    #[test]
    fn test_dropdown_panel_content() {
        let markdown = r#"## More
### Projects
Line 1
Line 2
"#;
        let doc = parse_document_structure(markdown, Some("More"));

        assert_eq!(doc.dropdown_items.len(), 1);
        let panel = &doc.panels[0];
        assert!(panel.is_dropdown_item);
        assert!(panel.markdown_content.contains("Line 1"));
        assert!(panel.markdown_content.contains("Line 2"));
    }
}
