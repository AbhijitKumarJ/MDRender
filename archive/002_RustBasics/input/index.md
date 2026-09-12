---
title: "MDRender - Modern Markdown Documentation"
description: "A fast Markdown to HTML converter with Light/Dark modes and theme support"
date: "2026-09-09"
author: "MDRender Team"
tags: [markdown, html, themes, rust]
---

# MDRender - Modern Markdown Documentation

Welcome to **MDRender**! This documentation generator converts Markdown files into beautiful, responsive HTML sites with built-in dark mode and theme switching.

> [!NOTE]
> All common styling and JavaScript live in the `assets/` subfolder, ensuring high performance, zero redundancy, and clean updates.

> [!TIP]
> Use the theme selector and mode toggle in the top-right corner to switch between **GitHub**, **Dracula**, **Nord**, **Solarized**, and **Sepia** themes!

---

## Features Overview

Here is a summary of what MDRender provides out-of-the-box:

- 🌓 **Light & Dark Modes**: Seamless system auto-detection and manual toggle.
- 🎨 **Multiple Themes**: GitHub, Dracula, Nord, Solarized, and Sepia.
- 📱 **Fully Responsive Layout**: Mobile drawer navigation, sticky header, and desktop table of contents.
- 📋 **Code Copying**: Instant one-click copy button with visual confirmation.
- 🔍 **Instant Search**: Real-time document filtering in the sidebar.
- 🗂️ **Directory Hierarchy**: Preserves folder structure and computes correct relative paths automatically.

---

## Code Example

Here is a snippet showcasing Rust code block rendering:

```rust
use std::path::Path;

fn main() {
    println!("Converting Markdown with MDRender...");
    let output_dir = Path::new("./output");
    assert!(output_dir.join("assets/style.css").exists());
}
```

And a JavaScript snippet:

```javascript
// Switch theme smoothly
function switchTheme(newTheme) {
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('mdrender-theme', newTheme);
}
```

---

## GitHub Flavored Markdown (GFM)

### Tables

| Feature | Supported | Description |
| :--- | :---: | :--- |
| Tables | ✅ | Clean bordered responsive tables with hover states |
| Callouts / Alerts | ✅ | `[!NOTE]`, `[!TIP]`, `[!WARNING]`, `[!CAUTION]` |
| Task Lists | ✅ | Checkboxes for todo lists |
| Heading Anchors | ✅ | Hoverable `#` links for direct section linking |

### Task List

- [x] Recursive folder traversal
- [x] Light / Dark mode toggle
- [x] Shared `assets/` subfolder
- [x] Live watch mode
- [ ] Custom plugins support

---

## Next Steps

Check out our [Getting Started Guide](guide/getting-started.html) to learn how to configure and run the tool.
