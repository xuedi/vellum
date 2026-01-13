use chrono::Local;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::path::Path;

pub fn substitute_variables(markdown: &str, base_path: &str) -> String {
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

    result = substitute_last_update_variables(&result, base_path);

    result
}

fn substitute_last_update_variables(markdown: &str, base_path: &str) -> String {
    use std::fs;

    let mut result = markdown.to_string();
    let pattern_start = "{{lastUpdate:";
    let pattern_end = "}}";
    let base = Path::new(base_path);

    while let Some(start) = result.find(pattern_start) {
        let after_start = start + pattern_start.len();
        if let Some(end_offset) = result[after_start..].find(pattern_end) {
            let end = after_start + end_offset;
            let file_path = &result[after_start..end];

            // Resolve path relative to base_path
            let full_path = base.join(file_path);

            let date_str = if let Ok(metadata) = fs::metadata(&full_path) {
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

pub fn process_includes(markdown: &str, base_path: &str, _dropdown_section: Option<&str>) -> String {
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
                // Skip first h1 in included files (treated as file title)
                if is_included && level == 1 && !first_h1_skipped {
                    first_h1_skipped = true;
                    continue;
                }

                // For included content, adjust heading level based on parent context
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
                                let processed = process_includes_recursive(
                                    &content,
                                    &parent,
                                    current_level,
                                    true,
                                );
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
        let output = substitute_variables(input, ".");
        assert!(output.contains("202"));
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_current_date() {
        let input = "Date: {{currentDate}}";
        let output = substitute_variables(input, ".");
        assert!(output.contains("-"));
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_current_datetime() {
        let input = "Updated: {{currentDateTime}}";
        let output = substitute_variables(input, ".");
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_multiple_variables() {
        let input = "{{currentYear}} and {{currentYear}} again";
        let output = substitute_variables(input, ".");
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_substitute_unknown_variable() {
        let input = "Unknown: {{unknownVar}}";
        let output = substitute_variables(input, ".");
        assert!(output.contains("{{unknownVar}}"));
    }

    #[test]
    fn test_substitute_last_update_with_path() {
        use std::fs;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file_path = subdir.join("test.md");
        fs::write(&file_path, "content").unwrap();

        let input = "Updated: {{lastUpdate:subdir/test.md}}";
        let output = substitute_variables(input, dir.path().to_str().unwrap());

        // Should contain a date, not "unknown"
        assert!(!output.contains("{{"));
        assert!(!output.contains("unknown"));
        assert!(output.contains("-")); // Date format YYYY-MM-DD
    }

    #[test]
    fn test_process_includes_no_includes() {
        let input = "# Title\n\nSome content";
        let output = process_includes(input, ".", None);
        assert_eq!(output.trim(), input);
    }

    #[test]
    fn test_process_includes_with_file() {
        let dir = std::env::temp_dir();
        let include_path = dir.join("include_test.md");
        std::fs::write(&include_path, "Included content here").unwrap();

        let input = format!("Include: [test]({}/include_test.md)", dir.display());
        let output = process_includes(&input, ".", None);
        assert!(output.contains("Included content here"));
        std::fs::remove_file(&include_path).ok();
    }

    #[test]
    fn test_process_includes_missing_file() {
        let input = "Include: [test](nonexistent_file.md)";
        let output = process_includes(input, ".", None);
        assert!(output.contains("Error: Could not include"));
    }

    #[test]
    fn test_process_includes_private_file() {
        let dir = std::env::temp_dir();
        let private_path = dir.join("private_test.md");
        std::fs::write(&private_path, "PRIVATE_NEVER_AS_IS\nSecret content").unwrap();

        let input = format!("Include: [test]({}/private_test.md)", dir.display());
        let output = process_includes(&input, ".", None);
        assert!(!output.contains("Secret content"));
        assert!(output.contains("Include:"));
        std::fs::remove_file(&private_path).ok();
    }

    #[test]
    fn test_process_includes_non_md_file() {
        let input = "Include: [test](file.txt)";
        let output = process_includes(input, ".", None);
        assert!(output.contains("Include:"));
    }

    #[test]
    fn test_process_includes_new_logic() {
        let dir = std::env::temp_dir();
        let include_path = dir.join("include_new.md");
        std::fs::write(&include_path, "# Included Title\n## Subtitle\nContent").unwrap();

        let input = format!("## Parent Section\n\nInclude: [test]({}/include_new.md)", dir.display());
        // We need to pass the base path correctly. 
        // Note: process_includes currently doesn't take current level, so we'll need to update its signature or use a wrapper.
        let output = process_includes(&input, ".", None);
        
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
        let output = process_includes(&input, ".", None);
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
