//! Integration tests for the HTML generator.
//!
//! These tests use real temporary files to test the full pipeline.

use vellum::{generate_html, generate_html_from_content, validate_inputs, GeneratorConfig, assets::Assets};
use std::fs;
use tempfile::TempDir;

/// Creates a minimal valid PNG file (1x1 transparent pixel).
fn create_test_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

/// Creates test assets for integration tests.
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
fn test_full_pipeline_with_temp_files() {
    let dir = TempDir::new().unwrap();

    // Create test markdown file
    let markdown_path = dir.path().join("test.md");
    let markdown_content = r#"# Test Document

## Overview

This is a test document.

## Features

- Feature one
- Feature two <! milestone
"#;
    fs::write(&markdown_path, markdown_content).unwrap();

    // Create test logo file
    let logo_path = dir.path().join("logo.png");
    fs::write(&logo_path, create_test_png()).unwrap();

    // Create config
    let config = GeneratorConfig {
        markdown_path: markdown_path.to_str().unwrap().to_string(),
        logo_path: logo_path.to_str().unwrap().to_string(),
        title: "Test Document".to_string(),
        output_path: dir.path().join("output.html").to_str().unwrap().to_string(),
        dropdown_section: None,
    };

    // Validate inputs
    assert!(validate_inputs(&config).is_ok());

    // Generate HTML
    let assets = test_assets();
    let result = generate_html(&config, &assets);
    assert!(result.is_ok());

    let (html, stats) = result.unwrap();

    // Verify output
    assert!(!html.is_empty());
    let html_str = String::from_utf8(html).unwrap();
    assert!(html_str.contains("<!DOCTYPE html>"));
    assert!(html_str.contains("<title>Test Document</title>"));
    assert!(html_str.contains("data:image/png;base64,"));

    // Verify stats
    assert_eq!(stats.section_count, 2); // Overview and Features
    assert_eq!(stats.achievement_markers, 1);
}

#[test]
fn test_validate_inputs_missing_markdown() {
    let dir = TempDir::new().unwrap();

    // Create only logo, not markdown
    let logo_path = dir.path().join("logo.png");
    fs::write(&logo_path, create_test_png()).unwrap();

    let config = GeneratorConfig {
        markdown_path: dir.path().join("missing.md").to_str().unwrap().to_string(),
        logo_path: logo_path.to_str().unwrap().to_string(),
        title: "Test".to_string(),
        output_path: "output.html".to_string(),
        dropdown_section: None,
    };

    let result = validate_inputs(&config);
    assert!(result.is_err());
}

#[test]
fn test_validate_inputs_missing_logo() {
    let dir = TempDir::new().unwrap();

    // Create only markdown, not logo
    let markdown_path = dir.path().join("test.md");
    fs::write(&markdown_path, "# Test").unwrap();

    let config = GeneratorConfig {
        markdown_path: markdown_path.to_str().unwrap().to_string(),
        logo_path: dir.path().join("missing.png").to_str().unwrap().to_string(),
        title: "Test".to_string(),
        output_path: "output.html".to_string(),
        dropdown_section: None,
    };

    let result = validate_inputs(&config);
    assert!(result.is_err());
}

#[test]
fn test_validate_inputs_both_exist() {
    let dir = TempDir::new().unwrap();

    let markdown_path = dir.path().join("test.md");
    fs::write(&markdown_path, "# Test").unwrap();

    let logo_path = dir.path().join("logo.png");
    fs::write(&logo_path, create_test_png()).unwrap();

    let config = GeneratorConfig {
        markdown_path: markdown_path.to_str().unwrap().to_string(),
        logo_path: logo_path.to_str().unwrap().to_string(),
        title: "Test".to_string(),
        output_path: "output.html".to_string(),
        dropdown_section: None,
    };

    assert!(validate_inputs(&config).is_ok());
}

#[test]
fn test_generate_html_from_content_pure() {
    let markdown = r#"# Hello World

## Introduction

Welcome to the test.

## Details

Some details here.
"#;
    let logo_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let assets = test_assets();

    let result = generate_html_from_content(markdown, ".", "Test Title", logo_uri, None, &assets);
    assert!(result.is_ok());

    let (html, stats) = result.unwrap();

    // Verify HTML structure
    let html_str = String::from_utf8(html).unwrap();
    assert!(html_str.contains("<!DOCTYPE html>"));
    assert!(html_str.contains("<title>Test Title</title>"));
    assert!(html_str.contains("section-introduction"));
    assert!(html_str.contains("section-details"));

    // Verify stats
    assert_eq!(stats.section_count, 2);
    assert!(stats.source_lines > 0);
}

#[test]
fn test_generate_html_from_content_with_includes() {
    let dir = TempDir::new().unwrap();

    // Create an include file
    let include_path = dir.path().join("include.md");
    fs::write(&include_path, "This is included content.").unwrap();

    let markdown = format!(
        r#"# Main Document

## Section

Content in: [include]({})
"#,
        include_path.to_str().unwrap()
    );

    let logo_uri = "data:image/png;base64,AAAA";
    let assets = test_assets();

    let result = generate_html_from_content(&markdown, ".", "Test", logo_uri, None, &assets);
    assert!(result.is_ok());

    let (html, stats) = result.unwrap();
    let html_str = String::from_utf8(html).unwrap();

    // The included content should be in the output
    assert!(html_str.contains("included content"));
    assert!(stats.expanded_lines > stats.source_lines);
}

#[test]
fn test_generate_html_from_content_empty_document() {
    let markdown = "";
    let logo_uri = "data:image/png;base64,AAAA";
    let assets = test_assets();

    let result = generate_html_from_content(markdown, ".", "Empty", logo_uri, None, &assets);
    assert!(result.is_ok());

    let (html, stats) = result.unwrap();
    assert!(!html.is_empty()); // Should still produce valid HTML
    assert_eq!(stats.source_lines, 0);
    assert_eq!(stats.section_count, 0);
}

#[test]
fn test_generate_html_missing_file() {
    let config = GeneratorConfig {
        markdown_path: "/nonexistent/path/file.md".to_string(),
        logo_path: "/nonexistent/path/logo.png".to_string(),
        title: "Test".to_string(),
        output_path: "output.html".to_string(),
        dropdown_section: None,
    };
    let assets = test_assets();

    let result = generate_html(&config, &assets);
    assert!(result.is_err());
}

#[test]
fn test_config_from_file() {
    let dir = TempDir::new().unwrap();

    // Create a config file
    let config_path = dir.path().join("config.toml");
    let config_content = r#"
[document]
title = "Test Title"
dropdown = "Custom Dropdown"

[paths]
markdown = "test.md"
logo = "logo.png"
output = "out/index.html"
"#;
    fs::write(&config_path, config_content).unwrap();

    let config = GeneratorConfig::from_file(&config_path).unwrap();
    assert_eq!(config.title, "Test Title");
    assert_eq!(config.markdown_path, "test.md");
    assert_eq!(config.logo_path, "logo.png");
    assert_eq!(config.output_path, "out/index.html");
    assert_eq!(config.dropdown_section, Some("Custom Dropdown".to_string()));
}
