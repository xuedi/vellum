# Vellum Architecture

## Overview

Vellum is a Rust-based static site generator that transforms extended Markdown files into self-contained HTML documents. The output is a single HTML file with all assets (CSS, JavaScript, images) embedded inline, making it fully offline-capable.

## Project Structure

```
vellum/
├── src/
│   ├── main.rs              # CLI entry point, argument parsing
│   ├── lib.rs               # Public API, generation pipeline orchestration
│   ├── parser/
│   │   ├── mod.rs           # Parser module exports
│   │   ├── document.rs      # Document tree parsing, panel extraction
│   │   ├── markdown.rs      # Markdown processing, includes, variables
│   │   ├── sections.rs      # Section extraction, slugification
│   │   └── skill_matrix.rs  # Skill matrix table transformation
│   ├── renderer/
│   │   ├── mod.rs           # Renderer module exports
│   │   └── html.rs          # HTML rendering, template substitution
│   └── assets/
│       ├── mod.rs           # Asset loading from config directory
│       └── embedder.rs      # Image base64 encoding
├── config/
│   └── assets/
│       ├── template.html    # HTML template with placeholders
│       ├── style.css        # Stylesheet
│       └── script.js        # Minimal panel show/hide logic
├── tests/
│   └── integration.rs       # End-to-end tests
└── .claude/
    ├── CLAUDE.md            # Project guidelines
    └── architecture.md      # This file
```

## Processing Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                        INPUT                                    │
│  Markdown file + Logo + Config (title, dropdown section)        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. PROCESS INCLUDES                                            │
│     - Resolve `Include: (path.md)` directives                   │
│     - Recursively embed referenced markdown files               │
│     - Adjust heading levels for included content                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  2. SUBSTITUTE VARIABLES                                        │
│     - Replace {{currentDate}}, {{currentYear}}, etc.            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  3. TRANSFORM MARKERS                                           │
│     - Convert `<! tag` to styled achievement markers            │
│     - Apply colored_tags regex patterns from config             │
│     - Transform skill matrix tables with level coloring         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  4. PARSE DOCUMENT STRUCTURE                                    │
│     - Extract H2 sections → nav buttons                         │
│     - Extract H3 under dropdown section → dropdown items        │
│     - Normalize heading levels per panel (H4→H3 for dropdown)   │
│     - Build ContentPanel list with markdown content             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  5. RENDER HTML                                                 │
│     - Convert each panel's markdown to HTML                     │
│     - Generate navigation (buttons + dropdown)                  │
│     - Embed logo as base64 data URI                             │
│     - Substitute into HTML template                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        OUTPUT                                   │
│  Single self-contained HTML file                                │
└─────────────────────────────────────────────────────────────────┘
```

## Key Data Structures

### DocumentStructure (parser/document.rs)

```rust
struct DocumentStructure {
    nav_buttons: Vec<NavItem>,      // H2 sections → top nav buttons
    dropdown_title: Option<String>, // Configured dropdown section name
    dropdown_items: Vec<NavItem>,   // H3 under dropdown → dropdown options
    panels: Vec<ContentPanel>,      // All pre-rendered content panels
}

struct ContentPanel {
    id: String,              // Unique panel ID (slugified title)
    title: String,           // Panel heading
    markdown_content: String // Normalized markdown (heading levels adjusted)
}
```

### Heading Level Normalization

Content panels are rendered independently of their original document tree position:

- **Regular sections (H2)**: Content starts at H3 level, rendered as-is
- **Dropdown items (H3)**: Content starts at H4 level, shifted up (H4→H3, H5→H4)

This normalization happens in the parser, not the renderer. The renderer receives panels with already-normalized content and doesn't need to know the original document structure.

## Extended Markdown Syntax

### Includes
```markdown
Include: (path/to/file.md)
```

### Variables
```markdown
{{currentDate}}     → 2026-01-13
{{currentYear}}     → 2026
{{currentDateTime}} → 2026-01-13 14:30
```

### Achievement Markers
```markdown
- Completed feature <! milestone
- Fixed bug <! fix
```

### Colored Tags
Config-defined regex patterns are replaced with styled tags:
```toml
[colored_tags]
"KW\\d{2}-OK" = "green"
"KW\\d{2}-FAIL" = "red"
```
Text matching these patterns becomes `<span class="color-tag color-tag-green">KW02-OK</span>`.

### Skill Matrix
Tables under headings containing "skill" or "matrix" are auto-styled with level-based coloring (0-10 scale, red→yellow→green gradient).

## Design Principles

1. **Self-contained output**: No external dependencies, works offline
2. **Pre-render everything**: JavaScript only handles show/hide, no DOM manipulation
3. **Separation of concerns**: Parser normalizes data, renderer is structure-agnostic
4. **Single file deployment**: One HTML file contains everything
