# Vellum - Command Runner
# Usage: just <command>

# Overview over all possible commands and it parameters
default:
    just --list

# Build the project
build:
    cargo build

# Build release version
release:
    cargo build --release

# Run the generator (creates index.html)
run:
    cargo run

# Run tests only
test:
    cargo test

# Run tests with full HTML coverage report (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html --output-dir .
    @echo "Coverage report: coverage-report.html"

# Run clippy lints
lint:
    cargo clippy

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt --check

# Run all quality checks (format check, lint, test)
check: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Full clean rebuild
rebuild: clean build

# Install binary to ~/.local/bin and config/ to ~/.config/vellum/
install: release
    mkdir -p ~/.local/bin ~/.config/vellum/assets
    cp target/release/vellum ~/.local/bin/
    @test -f ~/.config/vellum/config.toml || cp config.toml ~/.config/vellum/config.toml
    @test -f ~/.config/vellum/assets/style.css || cp config/assets/style.css ~/.config/vellum/assets/
    @test -f ~/.config/vellum/assets/script.js || cp config/assets/script.js ~/.config/vellum/assets/
    @test -f ~/.config/vellum/assets/template.html || cp config/assets/template.html ~/.config/vellum/assets/
    @echo "Installed binary to ~/.local/bin/vellum"
    @echo "Installed config/assets to ~/.config/vellum/"
    @echo "Ensure ~/.local/bin is in your PATH"

# Uninstall binary and config
uninstall:
    rm -f ~/.local/bin/vellum
    rm -rf ~/.config/vellum
    @echo "Removed ~/.local/bin/vellum"
    @echo "Removed ~/.config/vellum/"

# Generate demo HTML files in project root
demo: build
    @echo "Generating demo-portfolio.html..."
    cargo run -- --config demo/portfolio/config
    @echo ""
    @echo "Generating demo-vellum.html..."
    cargo run -- --config demo/vellum/config
    @echo ""
    @echo "Generating demo-rockband.html..."
    cargo run -- --config demo/rockband/config
    @echo ""
    @echo "Demo files generated:"
    @ls -la demo-*.html 2>/dev/null || echo "  (no demo files found)"
