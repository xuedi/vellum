use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub start: usize,
    pub end: usize,
}

pub fn extract_sections(html: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut id_counts: HashMap<String, usize> = HashMap::new();

    let mut search_start = 0;
    while let Some(h2_start) = html[search_start..].find("<h2>") {
        let absolute_start = search_start + h2_start;

        if let Some(h2_end) = html[absolute_start..].find("</h2>") {
            let tag_content_start = absolute_start + 4;
            let tag_content_end = absolute_start + h2_end;

            let title = html[tag_content_start..tag_content_end].to_string();

            let base_id = slugify(&title);
            let count = id_counts.entry(base_id.clone()).or_insert(0);
            let id = if *count == 0 {
                base_id.clone()
            } else {
                format!("{}-{}", base_id, count)
            };
            *count += 1;

            sections.push(Section {
                id,
                title,
                start: absolute_start,
                end: 0,
            });

            search_start = absolute_start + h2_end + 5;
        } else {
            break;
        }
    }

    for i in 0..sections.len() {
        sections[i].end = if i + 1 < sections.len() {
            sections[i + 1].start
        } else {
            html.len()
        };
    }

    sections
}

pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_section() {
        let html = "<h1>Title</h1><h2>Overview</h2><p>Content</p>";
        let sections = extract_sections(html);

        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Overview");
        assert_eq!(sections[0].id, "overview");
    }

    #[test]
    fn test_extract_multiple_sections() {
        let html = "<h2>First</h2><p>A</p><h2>Second</h2><p>B</p>";
        let sections = extract_sections(html);

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "First");
        assert_eq!(sections[1].title, "Second");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Self-Assessment (EP)"), "self-assessment-ep");
    }

    #[test]
    fn test_duplicate_ids() {
        let html = "<h2>Test</h2><h2>Test</h2>";
        let sections = extract_sections(html);

        assert_eq!(sections[0].id, "test");
        assert_eq!(sections[1].id, "test-1");
    }
}
