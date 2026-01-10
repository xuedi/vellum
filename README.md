# 📜 Vellum

[![Gitea Release](https://img.shields.io/badge/Version-v0.1.0-31c754.svg)](https://github.com/xuedi/vellum/releases)
[![EUPL Licence](https://img.shields.io/badge/Licence-EUPL_v1.2-31c754.svg)](https://eupl.eu/1.2/en)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Built with Just](https://img.shields.io/badge/built%20with-just-blue.svg)](https://github.com/casey/just)

Vellum is a minimalist Rust-powered static site generator designed to transform collections of Markdown files into beautiful, self-contained HTML documents. It's perfect for professional portfolios, technical documentation, or any project where you want a single, portable file that includes all its assets (CSS, JS, and images).

Initially built for professional portfolios with built-in support for skill matrices and achievement tracking, Vellum makes your Markdown collections transportable and readable for everyone.

## ✨ Features

- 📦 **Zero-Dependency Output**: Generates a single HTML file with all CSS, JS, and images inlined (Base64).
- 🌐 **Offline First**: Works perfectly without an internet connection.
- 🏆 **Achievement Tracking**: Highlight milestones with a simple custom syntax.
- 🔗 **Smart Includes**: Compose large documents from multiple Markdown files effortlessly.
- 📊 **Skill Matrix**: Render beautiful, color-coded skill tables automatically.
- 🏗️ **Template Customization**: Fully customizable HTML templates, CSS, and JavaScript.
- 📅 **Dynamic Variables**: Inject dates, years, and file metadata directly into your content.

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/xuedi/vellum.git
cd vellum

# Install (installs binary to ~/.local/bin and config to ~/.config/vellum)
just install

# Run with default config
vellum
```

## 🛠️ Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Just](https://github.com/casey/just) command runner

### Install from Source
```bash
just install
```
This command builds the release version and:
1. Moves the `vellum` binary to `~/.local/bin/`.
2. Sets up `~/.config/vellum/` with default `config.toml`, `style.css`, `script.js`, and `template.html`.

## 📖 Usage

### Basic Command
```bash
vellum
```
By default, Vellum looks for a configuration at `~/.config/vellum/config.toml`.

### Custom Configuration
```bash
vellum --config path/to/your/config/
```

### Development
If you are working on the Vellum source code, you can use `just`:
```bash
just run                # Run the generator
just demo               # Generate all demo files (portfolio, rockband, etc.)
just test               # Run the test suite
just check              # Run all quality checks (fmt, lint, test)
```

## ⚙️ Configuration

Vellum is configured via a `config.toml` file.

```toml
[document]
title = "My Professional Portfolio"
dropdown = "Archives"  # Optional: Header section to turn into a dropdown

[paths]
markdown = "data/index.md"     # Entry point Markdown file
logo = "assets/logo.png"       # Path to your logo
output = "dist/index.html"     # Where the generated HTML will be saved
skill_matrix = "data/skills.md" # Optional: Separate skill matrix source
```

## 📝 Custom Syntax & Conventions

### 🎯 Achievement Markers
Highlight key achievements in lists using the `<!` marker:
```markdown
- 2024-05-20: Lead the migration to Microservices <! Achievement unlocked: Zero downtime migration
```
*The text after `<!` will be styled prominently in the output.*

### 🧩 File Includes
Keep your project organized by splitting content into multiple files:
```markdown
## Projects
Include: [Project List](projects.md)
```
Vellum will automatically inline the content of `projects.md` at that location.

### 📊 Skill Matrix
Create professional, color-coded skill tables by using the `#### skill matrix` header followed by a Markdown table:

```markdown
#### skill matrix

| Skill          | Level | Notes                 |
|----------------|-------|-----------------------|
| **Languages**  |       |                       |
| Rust           | 9     | Expert level          |
| Python         | 7     | Automation & Scripting|
| **Tools**      |       |                       |
| Docker         | 8     | WIP - Optimizing CI   |
```
*Levels (0-10) are automatically color-coded in the generated HTML.*

### 🕒 Template Variables
Use dynamic variables that update every time you build:
- `{{CURRENT_DATE}}`: Current date (YYYY-MM-DD)
- `{{CURRENT_YEAR}}`: Current year (YYYY)
- `{{CURRENT_DATETIME}}`: Full timestamp
- `{{LAST_UPDATE:file.md}}`: Modification date of a specific file

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for more information.
