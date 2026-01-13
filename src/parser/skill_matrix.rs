/// Skill matrix detection and transformation module.
///
/// Provides flexible pattern recognition for skill/competency tables in Markdown.

/// Keywords that indicate a skill matrix heading (case insensitive).
const HEADING_KEYWORDS: &[&str] = &["skill", "matrix", "competenc", "proficienc"];

/// Column names that indicate a skill/name column (case insensitive).
const SKILL_COLUMNS: &[&str] = &[
    "skill",
    "skills",
    "name",
    "competency",
    "technology",
    "tool",
    "area",
    "topic",
    "category",
    "item",
];

/// Column names that indicate a value/level column (case insensitive).
const VALUE_COLUMNS: &[&str] = &[
    "level",
    "rating",
    "score",
    "value",
    "proficiency",
    "experience",
    "expertise",
    "grade",
    "rank",
];

/// Column names that indicate a notes/description column (case insensitive).
const NOTES_COLUMNS: &[&str] = &["note", "notes", "description", "comment", "details", "info"];

/// Detected column indices for a skill matrix table.
#[derive(Debug)]
pub struct TableColumns {
    pub skill_idx: usize,
    pub value_idx: usize,
    pub notes_idx: Option<usize>,
    pub skill_header: String,
    pub value_header: String,
    pub notes_header: Option<String>,
}

/// Check if a heading line indicates a skill matrix section.
pub fn is_skill_matrix_heading(line: &str) -> bool {
    let trimmed = line.trim();

    if !trimmed.starts_with('#') {
        return false;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count == 0 {
        return false;
    }

    // Check if character after hashes is whitespace or end of string
    if let Some(c) = trimmed.chars().nth(hash_count) {
        if !c.is_whitespace() {
            return false;
        }
    }

    let heading_text = trimmed[hash_count..].trim().to_lowercase();

    // Check if any keyword is present in the heading
    HEADING_KEYWORDS
        .iter()
        .any(|keyword| heading_text.contains(keyword))
}

/// Detect column indices from a table header row.
pub fn detect_columns(header_row: &str) -> Option<TableColumns> {
    let cols: Vec<&str> = header_row.split('|').map(|s| s.trim()).collect();

    let mut skill_idx = None;
    let mut value_idx = None;
    let mut notes_idx = None;
    let mut skill_header = String::new();
    let mut value_header = String::new();
    let mut notes_header = None;

    for (i, col) in cols.iter().enumerate() {
        let lower = col.to_lowercase();

        // Check for skill column
        if skill_idx.is_none()
            && SKILL_COLUMNS
                .iter()
                .any(|&s| lower == s || lower.starts_with(s))
        {
            skill_idx = Some(i);
            skill_header = col.to_string();
            continue;
        }

        // Check for value column
        if value_idx.is_none()
            && VALUE_COLUMNS
                .iter()
                .any(|&v| lower == v || lower.starts_with(v))
        {
            value_idx = Some(i);
            value_header = col.to_string();
            continue;
        }

        // Check for notes column
        if notes_idx.is_none()
            && NOTES_COLUMNS
                .iter()
                .any(|&n| lower == n || lower.starts_with(n))
        {
            notes_idx = Some(i);
            notes_header = Some(col.to_string());
        }
    }

    // We need at least a skill and value column
    match (skill_idx, value_idx) {
        (Some(s), Some(v)) => Some(TableColumns {
            skill_idx: s,
            value_idx: v,
            notes_idx,
            skill_header,
            value_header,
            notes_header,
        }),
        _ => None,
    }
}

/// Transform skill matrix content into HTML.
pub fn transform_skill_matrices(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if is_skill_matrix_heading(line) {
            // Output the heading first (so section navigation works)
            result.push_str(line);
            result.push('\n');

            // Collect any content between heading and table
            let mut table_start = i + 1;
            while table_start < lines.len() && !lines[table_start].trim().starts_with('|') {
                result.push_str(lines[table_start]);
                result.push('\n');
                table_start += 1;
            }

            let mut table_lines: Vec<&str> = Vec::new();
            let mut j = table_start;
            while j < lines.len() {
                let table_line = lines[j].trim();
                if table_line.starts_with('|') {
                    table_lines.push(table_line);
                    j += 1;
                } else if table_line.is_empty() {
                    j += 1;
                } else {
                    break;
                }
            }

            if let Some(html) = render_skill_matrix_table(&table_lines) {
                result.push_str("<div class=\"skill-matrix-container\">\n");
                result.push_str(&html);
                result.push_str("\n</div>\n\n");
                i = j;
                continue;
            }
        }

        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    result
}

/// Render a skill matrix table as HTML.
fn render_skill_matrix_table(table_lines: &[&str]) -> Option<String> {
    if table_lines.len() < 3 {
        return None;
    }

    let columns = detect_columns(table_lines[0])?;
    let has_notes = columns.notes_idx.is_some();
    let colspan = if has_notes { 3 } else { 2 };

    let mut html = String::from("<table class=\"skill-matrix\">\n<thead><tr>");
    html.push_str(&format!("<th>{}</th>", columns.skill_header));
    html.push_str(&format!("<th>{}</th>", columns.value_header));
    if let Some(ref notes_header) = columns.notes_header {
        html.push_str(&format!("<th>{}</th>", notes_header));
    }
    html.push_str("</tr></thead>\n<tbody>\n");

    for line in table_lines.iter().skip(2) {
        let cols: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

        let skill = cols.get(columns.skill_idx).unwrap_or(&"");
        let value = cols.get(columns.value_idx).unwrap_or(&"");
        let notes = columns
            .notes_idx
            .and_then(|idx| cols.get(idx).copied())
            .unwrap_or("");

        // Category row detection (bold skill name, empty value)
        if skill.starts_with("**") && skill.ends_with("**") && value.is_empty() {
            let category = skill.trim_start_matches("**").trim_end_matches("**");
            html.push_str(&format!(
                "<tr class=\"category-row\"><td colspan=\"{}\"><strong>{}</strong></td></tr>\n",
                colspan, category
            ));
        } else if !skill.is_empty() {
            let styled_notes = style_notes(notes);
            let level_class = format!("level-{}", value);

            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"{}\">{}</td>",
                skill, level_class, value
            ));
            if has_notes {
                html.push_str(&format!("<td>{}</td>", styled_notes));
            }
            html.push_str("</tr>\n");
        }
    }

    html.push_str("</tbody>\n</table>");
    Some(html)
}

/// Style notes content (e.g., WIP markers).
fn style_notes(notes: &str) -> String {
    if notes.to_lowercase().starts_with("wip") {
        let rest = notes
            .get(3..)
            .unwrap_or("")
            .trim_start_matches(" -")
            .trim_start();
        format!("<span class=\"wip-marker\">WIP</span>{}", rest)
    } else {
        notes.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_detection_skill_matrix() {
        assert!(is_skill_matrix_heading("## skill matrix"));
        assert!(is_skill_matrix_heading("### Skill Matrix"));
        assert!(is_skill_matrix_heading("#### SKILL MATRIX"));
    }

    #[test]
    fn test_heading_detection_skill_only() {
        assert!(is_skill_matrix_heading("## Skills"));
        assert!(is_skill_matrix_heading("## My Skills"));
        assert!(is_skill_matrix_heading("### Technical Skills"));
    }

    #[test]
    fn test_heading_detection_matrix_only() {
        assert!(is_skill_matrix_heading("## Matrix"));
        assert!(is_skill_matrix_heading("## Competency Matrix"));
    }

    #[test]
    fn test_heading_detection_competency() {
        assert!(is_skill_matrix_heading("## Competencies"));
        assert!(is_skill_matrix_heading("### Core Competencies"));
    }

    #[test]
    fn test_heading_detection_proficiency() {
        assert!(is_skill_matrix_heading("## Proficiency Levels"));
        assert!(is_skill_matrix_heading("### Language Proficiency"));
    }

    #[test]
    fn test_heading_detection_negative() {
        assert!(!is_skill_matrix_heading("## Overview"));
        assert!(!is_skill_matrix_heading("## Projects"));
        assert!(!is_skill_matrix_heading("Not a heading"));
        assert!(!is_skill_matrix_heading("#NoSpace"));
    }

    #[test]
    fn test_column_detection_standard() {
        let header = "| Skill | Level | Notes |";
        let cols = detect_columns(header).unwrap();
        assert_eq!(cols.skill_idx, 1);
        assert_eq!(cols.value_idx, 2);
        assert_eq!(cols.notes_idx, Some(3));
    }

    #[test]
    fn test_column_detection_alternative_names() {
        let header = "| Technology | Rating | Description |";
        let cols = detect_columns(header).unwrap();
        assert_eq!(cols.skill_header, "Technology");
        assert_eq!(cols.value_header, "Rating");
        assert_eq!(cols.notes_header, Some("Description".to_string()));
    }

    #[test]
    fn test_column_detection_no_notes() {
        let header = "| Name | Score |";
        let cols = detect_columns(header).unwrap();
        assert_eq!(cols.skill_idx, 1);
        assert_eq!(cols.value_idx, 2);
        assert!(cols.notes_idx.is_none());
    }

    #[test]
    fn test_column_detection_missing_value() {
        let header = "| Skill | Notes |";
        assert!(detect_columns(header).is_none());
    }

    #[test]
    fn test_transform_basic() {
        let input = r#"## Skills

| Skill | Level | Notes |
|-------|-------|-------|
| Rust  | 8     | Good  |
"#;
        let output = transform_skill_matrices(input);
        assert!(output.contains("skill-matrix"));
        assert!(output.contains("level-8"));
        assert!(output.contains("Rust"));
    }

    #[test]
    fn test_transform_with_categories() {
        let input = r#"## Skills

| Skill | Level | Notes |
|-------|-------|-------|
| **Languages** | | |
| Rust  | 8     | Good  |
"#;
        let output = transform_skill_matrices(input);
        assert!(output.contains("category-row"));
        assert!(output.contains("Languages"));
    }

    #[test]
    fn test_transform_without_notes() {
        let input = r#"## Proficiency

| Technology | Rating |
|------------|--------|
| Docker     | 7      |
"#;
        let output = transform_skill_matrices(input);
        assert!(output.contains("skill-matrix"));
        assert!(output.contains("Docker"));
        assert!(output.contains("level-7"));
    }

    #[test]
    fn test_wip_marker() {
        let styled = style_notes("WIP - learning");
        assert!(styled.contains("wip-marker"));
        assert!(styled.contains("learning"));
    }
}