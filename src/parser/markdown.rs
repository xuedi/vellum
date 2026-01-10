use chrono::Local;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::path::Path;

pub fn substitute_variables(markdown: &str) -> String {
    let now = Local::now();

    let mut variables: HashMap<&str, String> = HashMap::new();
    variables.insert("currentDateTime", now.format("%B %Y").to_string());
    variables.insert("currentDate", now.format("%Y-%m-%d").to_string());
    variables.insert("currentYear", now.format("%Y").to_string());

    let mut result = markdown.to_string();
    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, &value);
    }

    result = substitute_last_update_variables(&result);

    result
}

fn substitute_last_update_variables(markdown: &str) -> String {
    use std::fs;

    let mut result = markdown.to_string();
    let pattern_start = "{{lastUpdate:";
    let pattern_end = "}}";

    while let Some(start) = result.find(pattern_start) {
        let after_start = start + pattern_start.len();
        if let Some(end_offset) = result[after_start..].find(pattern_end) {
            let end = after_start + end_offset;
            let file_path = &result[after_start..end];

            let date_str = if let Ok(metadata) = fs::metadata(file_path) {
                if let Ok(modified) = metadata.modified() {
                    let datetime: chrono::DateTime<Local> = modified.into();
                    datetime.format("%Y-%m-%d").to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            let full_pattern = format!("{}{}{}", pattern_start, file_path, pattern_end);
            result = result.replacen(&full_pattern, &date_str, 1);
        } else {
            break;
        }
    }

    result
}

pub fn transform_achievement_markers(markdown: &str) -> String {
    let mut result = String::with_capacity(markdown.len());

    for line in markdown.lines() {
        if let Some(marker_pos) = line.find("<!") {
            let (before, after) = line.split_at(marker_pos);
            let marker_text = after[2..].trim_start();
            if !marker_text.is_empty() {
                result.push_str(before);
                result.push_str("<span class=\"achievement-marker\">");
                result.push_str(marker_text);
                result.push_str("</span>");
            } else {
                result.push_str(line);
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

pub fn process_includes(markdown: &str, base_path: &str) -> String {
    process_includes_recursive(markdown, base_path, 0, false)
}

fn process_includes_recursive(
    markdown: &str,
    base_path: &str,
    parent_level: usize,
    is_included: bool,
) -> String {
    let mut result = String::with_capacity(markdown.len());
    let base = Path::new(base_path);
    let mut current_level = parent_level;
    let mut first_h1_skipped = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Track current heading level
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level > 0 && trimmed.chars().nth(level).map_or(true, |c| c.is_whitespace()) {
                if is_included && level == 1 && !first_h1_skipped {
                    first_h1_skipped = true;
                    continue;
                }

                let effective_level = if is_included {
                    level + parent_level.max(1) - 1
                } else {
                    level
                };

                current_level = effective_level;

                result.push_str(&"#".repeat(effective_level));
                result.push_str(&line[level..]);
                result.push('\n');
                continue;
            }
        }

        if trimmed.starts_with("Include:") {
            if let Some(start) = line.find('(') {
                if let Some(end) = line[start..].find(')') {
                    let path = &line[start + 1..start + end];
                    if path.ends_with(".md") {
                        let full_path = base.join(path);
                        match std::fs::read_to_string(&full_path) {
                            Ok(content) => {
                                if content.starts_with("PRIVATE_NEVER_AS_IS") {
                                    result.push_str(line);
                                    result.push('\n');
                                    continue;
                                }
                                let parent = full_path
                                    .parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|| base_path.to_string());
                                let processed =
                                    process_includes_recursive(&content, &parent, current_level, true);
                                result.push_str(&processed);
                                result.push('\n');
                                continue;
                            }
                            Err(e) => {
                                result.push_str(&format!(
                                    "**Error: Could not include '{}': {}**\n",
                                    path, e
                                ));
                                continue;
                            }
                        }
                    }
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    result
}

pub fn parse_markdown(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

pub fn transform_skill_matrices(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim().to_lowercase();

        if trimmed == "#### skill matrix" {
            let mut table_start = i + 1;

            while table_start < lines.len() && lines[table_start].trim().is_empty() {
                table_start += 1;
            }

            let mut table_lines: Vec<&str> = Vec::new();
            let mut j = table_start;
            while j < lines.len() {
                let table_line = lines[j].trim();
                if table_line.starts_with('|') {
                    table_lines.push(table_line);
                    j += 1;
                } else if !table_line.is_empty() {
                    break;
                } else {
                    j += 1;
                }
            }

            if let Some(html) = parse_skill_matrix_table(&table_lines) {
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

fn parse_skill_matrix_table(table_lines: &[&str]) -> Option<String> {
    if table_lines.len() < 3 {
        return None;
    }

    let header_cols: Vec<&str> = table_lines[0]
        .split('|')
        .map(|s| s.trim())
        .collect();

    let skill_idx = header_cols.iter().position(|&s| s == "Skill")?;
    let level_idx = header_cols.iter().position(|&s| s == "Level")?;
    let notes_idx = header_cols.iter().position(|&s| s == "Notes")?;

    let mut html = String::from("<table class=\"skill-matrix\">\n");
    html.push_str("<thead><tr><th>Skill</th><th>Level</th><th>Notes</th></tr></thead>\n");
    html.push_str("<tbody>\n");

    for line in table_lines.iter().skip(2) {
        let cols: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

        if cols.len() <= skill_idx.max(level_idx).max(notes_idx) {
            continue;
        }

        let skill = cols.get(skill_idx).unwrap_or(&"");
        let level = cols.get(level_idx).unwrap_or(&"");
        let notes = cols.get(notes_idx).unwrap_or(&"");

        if skill.starts_with("**") && skill.ends_with("**") {
            let category = skill.trim_start_matches("**").trim_end_matches("**");
            html.push_str(&format!(
                "<tr class=\"category-row\"><td colspan=\"3\"><strong>{}</strong></td></tr>\n",
                category
            ));
        } else {
            let styled_notes = if notes.to_lowercase().starts_with("wip") {
                let rest = notes[3..].trim_start_matches(" -").trim_start();
                format!("<span class=\"wip-marker\">WIP</span>{}", rest)
            } else {
                notes.to_string()
            };
            let level_class = format!("level-{}", level);
            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"{}\">{}</td><td>{}</td></tr>\n",
                skill, level_class, level, styled_notes
            ));
        }
    }

    html.push_str("</tbody>\n</table>");
    Some(html)
}

pub fn parse_skill_matrix(file_path: &str) -> Option<String> {
    let content = std::fs::read_to_string(file_path).ok()?;

    let mut in_table = false;
    let mut table_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            in_table = true;
            table_lines.push(trimmed);
        } else if in_table && !trimmed.is_empty() {
            break;
        }
    }

    if table_lines.len() < 3 {
        return None;
    }

    let header_cols: Vec<&str> = table_lines[0]
        .split('|')
        .map(|s| s.trim())
        .collect();

    let skill_idx = header_cols.iter().position(|&s| s == "Skill")?;
    let level_idx = header_cols.iter().position(|&s| s == "Level")?;
    let notes_idx = header_cols.iter().position(|&s| s == "Notes")?;

    let mut html = String::from("<table class=\"skill-matrix\">\n");
    html.push_str("<thead><tr><th>Skill</th><th>Level</th><th>Notes</th></tr></thead>\n");
    html.push_str("<tbody>\n");

    for line in table_lines.iter().skip(2) {
        let cols: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

        if cols.len() <= skill_idx.max(level_idx).max(notes_idx) {
            continue;
        }

        let skill = cols.get(skill_idx).unwrap_or(&"");
        let level = cols.get(level_idx).unwrap_or(&"");
        let notes = cols.get(notes_idx).unwrap_or(&"");

        if skill.starts_with("**") && skill.ends_with("**") {
            let category = skill.trim_start_matches("**").trim_end_matches("**");
            html.push_str(&format!(
                "<tr class=\"category-row\"><td colspan=\"3\"><strong>{}</strong></td></tr>\n",
                category
            ));
        } else {
            let styled_notes = if notes.to_lowercase().starts_with("wip") {
                let rest = notes[3..].trim_start_matches(" -").trim_start();
                format!("<span class=\"wip-marker\">WIP</span>{}", rest)
            } else {
                notes.to_string()
            };
            let level_class = format!("level-{}", level);
            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"{}\">{}</td><td>{}</td></tr>\n",
                skill, level_class, level, styled_notes
            ));
        }
    }

    html.push_str("</tbody>\n</table>");
    Some(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_marker_transformation() {
        let input = "- Task done <! first achievement";
        let output = transform_achievement_markers(input);
        assert!(output.contains("<span class=\"achievement-marker\">first achievement</span>"));
    }

    #[test]
    fn test_achievement_marker_no_space() {
        let input = "- Task done <!first achievement";
        let output = transform_achievement_markers(input);
        assert!(output.contains("<span class=\"achievement-marker\">first achievement</span>"));
    }

    #[test]
    fn test_no_marker() {
        let input = "- Regular task";
        let output = transform_achievement_markers(input);
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_empty_marker() {
        let input = "- Task done <!";
        let output = transform_achievement_markers(input);
        assert!(!output.contains("achievement-marker"));
    }

    #[test]
    fn test_basic_markdown_parsing() {
        let input = "# Hello\n\nThis is **bold** text.";
        let output = parse_markdown(input);
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_table_parsing() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = parse_markdown(input);
        assert!(output.contains("<table>"));
        assert!(output.contains("<th>A</th>"));
    }

    #[test]
    fn test_substitute_current_year() {
        let input = "Year: {{currentYear}}";
        let output = substitute_variables(input);
        assert!(output.contains("202"));
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_current_date() {
        let input = "Date: {{currentDate}}";
        let output = substitute_variables(input);
        assert!(output.contains("-"));
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_current_datetime() {
        let input = "Updated: {{currentDateTime}}";
        let output = substitute_variables(input);
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_multiple_variables() {
        let input = "{{currentYear}} and {{currentYear}} again";
        let output = substitute_variables(input);
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_unknown_variable() {
        let input = "Unknown: {{unknownVar}}";
        let output = substitute_variables(input);
        assert!(output.contains("{{unknownVar}}"));
    }

    #[test]
    fn test_process_includes_no_includes() {
        let input = "# Title\n\nSome content";
        let output = process_includes(input, ".");
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_process_includes_with_file() {
        let dir = std::env::temp_dir();
        let include_path = dir.join("include_test.md");
        std::fs::write(&include_path, "Included content here").unwrap();

        let input = format!("Include: [test]({}/include_test.md)", dir.display());
        let output = process_includes(&input, ".");
        assert!(output.contains("Included content here"));
        std::fs::remove_file(&include_path).ok();
    }

    #[test]
    fn test_process_includes_missing_file() {
        let input = "Include: [test](nonexistent_file.md)";
        let output = process_includes(input, ".");
        assert!(output.contains("Error: Could not include"));
    }

    #[test]
    fn test_process_includes_private_file() {
        let dir = std::env::temp_dir();
        let private_path = dir.join("private_test.md");
        std::fs::write(&private_path, "PRIVATE_NEVER_AS_IS\nSecret content").unwrap();

        let input = format!("Include: [test]({}/private_test.md)", dir.display());
        let output = process_includes(&input, ".");
        assert!(!output.contains("Secret content"));
        assert!(output.contains("Include:"));
        std::fs::remove_file(&private_path).ok();
    }

    #[test]
    fn test_process_includes_non_md_file() {
        let input = "Include: [test](file.txt)";
        let output = process_includes(input, ".");
        assert!(output.contains("Include:"));
    }

    #[test]
    fn test_parse_skill_matrix_missing_file() {
        let result = parse_skill_matrix("nonexistent.md");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_skill_matrix_valid() {
        let dir = std::env::temp_dir();
        let path = dir.join("skill_matrix_test.md");
        let content = r#"# Skills
| Skill | Level | Notes |
|-------|-------|-------|
| **Category** | | |
| Rust | 8 | Good |
| Python | 6 | wip - learning |
"#;
        std::fs::write(&path, content).unwrap();

        let result = parse_skill_matrix(path.to_str().unwrap());
        assert!(result.is_some());
        let html = result.unwrap();
        assert!(html.contains("skill-matrix"));
        assert!(html.contains("category-row"));
        assert!(html.contains("Rust"));
        assert!(html.contains("wip-marker"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_parse_skill_matrix_insufficient_rows() {
        let dir = std::env::temp_dir();
        let path = dir.join("skill_short_test.md");
        let content = "| Skill |\n|-------|";
        std::fs::write(&path, content).unwrap();

        let result = parse_skill_matrix(path.to_str().unwrap());
        assert!(result.is_none());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_process_includes_new_logic() {
        let dir = std::env::temp_dir();
        let include_path = dir.join("include_new.md");
        std::fs::write(&include_path, "# Included Title\n## Subtitle\nContent").unwrap();

        let input = format!("## Parent Section\n\nInclude: [test]({}/include_new.md)", dir.display());
        // We need to pass the base path correctly. 
        // Note: process_includes currently doesn't take current level, so we'll need to update its signature or use a wrapper.
        let output = process_includes(&input, ".");
        
        // Expected behavior:
        // # Included Title is ignored.
        // ## Subtitle becomes #### Subtitle (2 + 2 = 4? OR 2 (parent) + (2 (child) - 1 (offset)) = 3?)
        // Wait, "adjust every level to the level of the document where it is included into"
        // If parent is at ## (level 2), maybe it means child's level 2 becomes level 3?
        // Let's assume: new_level = parent_level + child_level - 1 (since level 1 is ignored and effectively acts as level 0)
        // So ## (2) in child becomes 2 + 2 - 1 = 3? OR if it's "to the level", maybe 2 + 1 = 3.
        
        assert!(!output.contains("# Included Title"));
        assert!(output.contains("### Subtitle")); 
        std::fs::remove_file(&include_path).ok();
    }

    #[test]
    fn test_process_includes_no_label() {
        let dir = std::env::temp_dir();
        let include_path = dir.join("include_no_label.md");
        std::fs::write(&include_path, "# Title\nContent").unwrap();

        let input = format!("Include: []({}/include_no_label.md)", dir.display());
        let output = process_includes(&input, ".");
        assert!(output.contains("Content"));
        assert!(!output.contains("# Title"));
        std::fs::remove_file(&include_path).ok();
    }

    #[test]
    fn test_strikethrough() {
        let input = "This is ~~deleted~~ text.";
        let output = parse_markdown(input);
        assert!(output.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_task_list() {
        let input = "- [x] Done\n- [ ] Todo";
        let output = parse_markdown(input);
        assert!(output.contains("checked"));
    }
}
