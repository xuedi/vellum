# Technical

## Architecture

Include: [Architecture](../../../.claude/architecture.md)

## Configuration Reference

The `config.toml` file supports these sections (as of v0.1.0):

**[document]**
- `title` - Browser tab title and header text
- `dropdown` - Heading name to group under dropdown menu (empty to disable)

**[paths]**
- `markdown` - Path to main input file
- `logo` - Path to logo image (PNG, JPG, SVG, WebP, GIF supported)
- `output` - Path for generated HTML file
- `skill_matrix` - Optional path to skill matrix Markdown file

**[colored_tags]**
- Key-value pairs where keys are regex patterns and values are color names
- Available colors: green, grey, red, blue, yellow, orange, purple
- Example: `"v\\d+\\.\\d+\\.\\d+" = "blue"` highlights version numbers

## Supported Image Formats

Vellum embeds images as base64 data URIs. Supported formats:

| Format | MIME Type |
|--------|-----------|
| PNG | image/png |
| JPEG | image/jpeg |
| GIF | image/gif |
| WebP | image/webp |
| SVG | image/svg+xml |

## Include Syntax

To include content from another file:

```
Include: [label](filename.md)
```

The label is for documentation purposes. The file path is relative to the main Markdown file. Only `.md` files can be included. Files starting with underscore are treated as private and skipped.

## Template Variables

Available variables for substitution:

| Variable | Output |
|----------|--------|
| `{{CURRENT_YEAR}}` | Four-digit year (e.g., 2025) |
| `{{CURRENT_DATE}}` | ISO date (e.g., 2025-01-10) |
| `{{CURRENT_DATETIME}}` | ISO datetime (e.g., 2025-01-10T14:30:00) |

## Error Handling

Vellum provides specific error types:

- **ConfigReadError** - Cannot read configuration file
- **ConfigParseError** - Invalid TOML syntax or missing fields
- **FileReadError** - Cannot read input files
- **LogoEmbedError** - Problem embedding logo image
- **OutputWriteError** - Cannot write output file
