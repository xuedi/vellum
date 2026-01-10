
### One Markdown File with Subfiles

Vellum works with a single main Markdown file as your entry point. This file can include content from other Markdown files using the include syntax. This lets you organize large documents into manageable pieces while producing a single output.

To include another file, use: `Content in: [label](filename.md)`

The included content gets merged into your main document during generation.

### Level 2 Headings Become Navigation Buttons

Every level 2 heading (`## Section Name`) in your document automatically becomes a navigation button. Vellum extracts these headings and creates a button bar at the top of the generated HTML. Clicking a button scrolls to that section.

This gives your readers quick access to any part of the document without manual table of contents maintenance.

### Dropdown Menus for Archive Sections

If you want certain sections grouped under a dropdown menu instead of individual buttons, use a special heading name in your config. Setting a level 2 heading name like "Archive" or "More" in the dropdown configuration causes Vellum to collect those sections into a dropdown menu.

This keeps your navigation clean when you have many sections or want to de-emphasize older content while still keeping it accessible.
