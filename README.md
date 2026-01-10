# Vellum

A minimalist Rust static site generator that transforms a collection of Markdown files into self-contained HTML
documents that can be shared more easily with their embedded assets.

Initially used for a professional portfolio with skill matrix and extension for achievements, but also useful to make
any larger Markdown document collection transportable and readable for people who prefer not to read Markdown files.

## Features

- **Self-contained output** - Generated HTML includes all CSS, JS, and images inline
- **Offline-capable** - No external dependencies in the output
- **Achievement markers** - Special `<!` syntax for highlighting milestones
- **File includes** - Reference other Markdown files with `Content in: [name](path.md)`
- **Skill matrix** - Separate skill matrix rendering with color-coded levels
- **Configurable dropdown** - Convert any section into a dropdown menu

## Requirements

- Rust 1.70+
- [just](https://github.com/casey/just) command runner

## Installation

```bash
git clone git@github.com:xuedi/vellum.git
cd vellum
just install
```

This installs the binary to `~/.local/bin/vellum` and sets up the config directory at `~/.config/vellum/` with default assets.

## Usage

```bash
# Generate HTML output (requires config at ~/.config/vellum/)
vellum

# Or during development:
just run
```

## Configuration

Configuration and assets are stored in `~/.config/vellum/`:

```
~/.config/vellum/
├── config.toml      # Configuration file
├── style.css        # CSS styles (customizable)
├── script.js        # JavaScript (customizable)
└── template.html    # HTML template (customizable)
```

Edit `config.toml` to configure your document:

```toml
[document]
title = "My Portfolio"
dropdown = "Related Documents"  # Section to render as dropdown (empty to disable)

[paths]
markdown = "content/index.md"
logo = "logo.png"
output = "output/index.html"
skill_matrix = "content/matrix.md"  # Optional skill matrix (empty to disable)
```

You can customize the CSS, JavaScript, and HTML template by editing the files in the config directory.

## Content Conventions

### Daily Log Format
```markdown
- 2025-01-08: Description of work done
- 2025-01-07: after 4 months, finally <! finished project x 
```

Lines ending with `<!` are highlighted as achievements. (finished project x)

### File Includes
```markdown
Content in: [section-name](path/to/file.md)
```

This automatically inlines the referenced file's content.

### Template Variables
- `{{currentDate}}` - Current date (YYYY-MM-DD)
- `{{currentYear}}` - Current year
- `{{currentDateTime}}` - Current date and time
- `{{lastUpdate:file.md}}` - Last modification date of a file

## License

MIT
