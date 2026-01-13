# Getting Started

This guide covers Vellum v0.1.0 installation and setup.

## Installation

Clone the repository and build with Cargo:

```bash
git clone git@github.com:xuedi/vallum.git
cd vallum
cargo build --release
```

Run `just install` to set up the global configuration directory at `~/.config/vellum/`.

## Quick Start

1. Create your main Markdown file with level 2 headings for each section
2. Configure `config.toml` with your input and output paths
3. Run `vellum` to generate your HTML

## Project Structure

A typical Vellum project looks like this:

```
project/
  config/
    config.toml
    logo.png
  data/
    index.md
    section1.md
    section2.md
  index.html (generated)
```

## Configuration Location

Vellum searches for configuration in this order:

1. `~/.config/vellum/config.toml` (global)
2. `./config/config.toml` (current directory)
3. Parent directories up to 4 levels

This allows both global setups and project-specific configurations.
