# Features

Vellum v0.1.0 includes the following features:

## Self-Contained Output

Everything gets embedded into a single HTML file. CSS styles, JavaScript for navigation, your logo as base64, and all content live together. No broken links, no missing assets, no dependency on external servers.

## Automatic Navigation

Level 2 headings automatically become clickable navigation buttons. Readers can jump to any section instantly. No manual anchor management required.

## File Includes

Split large documents into smaller, manageable pieces. Use the include syntax to pull content from other Markdown files. Vellum merges everything during generation.

## Offline Ready

Generated documents work without internet access. Share via email, USB drives, or local file systems. Perfect for environments with restricted connectivity.

## Template Variables

Use built-in variables like `{{CURRENT_YEAR}}` or `{{CURRENT_DATE}}` in your content. Vellum substitutes them during generation, keeping your documents fresh.

## Colored Tags

Define regex patterns in your config to automatically highlight text as colored tags. Perfect for status indicators, version numbers, or any recurring patterns you want to stand out.

Available colors: green, grey, red, blue, yellow, orange, purple.

## Dropdown Navigation

Group related sections under a dropdown menu to keep the navigation bar clean. Ideal for archiving older content or organizing technical details.
