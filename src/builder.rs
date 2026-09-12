use crate::assets;
use crate::config::Config;
use crate::html::{self, NavItem, PageContext};
use crate::markdown::{self, FrontMatter, RenderedMarkdown};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct DiscoveredMarkdown {
    rel_md_path: PathBuf,
    rel_html_path: String,
    raw_content: String,
    title: String,
    section: String,
    frontmatter: FrontMatter,
}

pub fn build(config: &Config) -> Result<()> {
    println!("\n🚀 Building documentation with MDRender...");
    println!("  Input:  {}", config.input.display());
    println!("  Output: {}", config.output.display());

    // 1. Clean output if requested
    if config.clean && config.output.exists() {
        println!("  🧹 Cleaning output directory...");
        fs::remove_dir_all(&config.output)
            .with_context(|| format!("Failed to clean {}", config.output.display()))?;
    }

    fs::create_dir_all(&config.output)
        .with_context(|| format!("Failed to create output dir {}", config.output.display()))?;

    // 2. Ensure assets (assets/style.css, assets/theme.js)
    println!("  🎨 Checking common assets...");
    assets::ensure_assets(&config.output)?;

    // 3. Check input directory
    if !config.input.exists() {
        if config.init {
            println!("  📦 Initializing sample markdown content in {}...", config.input.display());
            scaffold_sample_content(&config.input)?;
        } else {
            println!("  ⚠️ Input directory does not exist. Creating sample content...");
            scaffold_sample_content(&config.input)?;
        }
    }

    // 4. Scan input directory
    let mut markdown_files: Vec<DiscoveredMarkdown> = Vec::new();
    let mut static_assets: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut has_root_index = false;

    for entry in WalkDir::new(&config.input).follow_links(true) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let rel_path = path.strip_prefix(&config.input)?;

            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "md" || ext_lower == "markdown" {
                    let raw_content = fs::read_to_string(path)
                        .with_context(|| format!("Failed to read {}", path.display()))?;

                    let rel_html_path_buf = rel_path.with_extension("html");
                    let rel_html_path = rel_html_path_buf.to_string_lossy().replace('\\', "/");

                    if rel_html_path == "index.html" {
                        has_root_index = true;
                    }

                    // Determine section name from parent directory
                    let section = if let Some(parent) = rel_path.parent() {
                        let p = parent.to_string_lossy().to_string();
                        if p.is_empty() {
                            "General".to_string()
                        } else {
                            format_section_name(&p)
                        }
                    } else {
                        "General".to_string()
                    };

                    // Extract frontmatter & quick title
                    let (fm, _) = markdown::parse_frontmatter(&raw_content);
                    let fallback_title = rel_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(format_title_from_stem)
                        .unwrap_or_else(|| "Untitled".to_string());

                    let title = fm.title.clone().unwrap_or(fallback_title);

                    markdown_files.push(DiscoveredMarkdown {
                        rel_md_path: rel_path.to_path_buf(),
                        rel_html_path,
                        raw_content,
                        title,
                        section,
                        frontmatter: fm,
                    });
                    continue;
                }
            }

            // Non-markdown static files (e.g. images)
            let out_dest = config.output.join(rel_path);
            static_assets.push((path.to_path_buf(), out_dest));
        }
    }

    // 5. Copy static assets (e.g., images)
    let mut copied_assets = 0;
    for (src, dest) in &static_assets {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dest)?;
        copied_assets += 1;
    }
    if copied_assets > 0 {
        println!("  📎 Copied {} static resource(s)", copied_assets);
    }

    // 6. If no root index.md exists, generate a root index.html listing all pages
    if !has_root_index && !markdown_files.is_empty() {
        println!("  ℹ️ No index.md found in root; auto-generating index portal...");
        let index_entry = generate_portal_index(&markdown_files, &config.title);
        markdown_files.insert(0, index_entry);
    }

    // Sort navigation items: root index first, then alphabetical by section and title
    markdown_files.sort_by(|a, b| {
        let a_is_index = a.rel_html_path == "index.html";
        let b_is_index = b.rel_html_path == "index.html";
        if a_is_index && !b_is_index {
            std::cmp::Ordering::Less
        } else if !a_is_index && b_is_index {
            std::cmp::Ordering::Greater
        } else if a.section != b.section {
            a.section.cmp(&b.section)
        } else {
            a.title.cmp(&b.title)
        }
    });

    // Build global nav items list
    let nav_items: Vec<NavItem> = markdown_files
        .iter()
        .map(|doc| NavItem {
            title: doc.title.clone(),
            rel_html_path: doc.rel_html_path.clone(),
            section: doc.section.clone(),
        })
        .collect();

    // 7. Render all HTML pages
    let total_pages = markdown_files.len();
    for doc in &markdown_files {
        let rendered: RenderedMarkdown =
            markdown::render_markdown(&doc.raw_content, &doc.title);

        let page_ctx = PageContext {
            site_title: &config.title,
            page_title: &rendered.title,
            frontmatter: &doc.frontmatter,
            content_html: &rendered.html_content,
            toc: &rendered.toc,
            current_rel_path: &doc.rel_html_path,
            nav_items: &nav_items,
        };

        let full_html = html::render_html_page(&page_ctx);
        let out_path = config.output.join(&doc.rel_html_path);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&out_path, full_html)
            .with_context(|| format!("Failed to write HTML to {}", out_path.display()))?;

        println!("  ✓ Rendered: {} -> {}", doc.rel_md_path.display(), doc.rel_html_path);
    }

    println!("\n✨ Done! Rendered {} page(s) successfully into `{}`.\n", total_pages, config.output.display());
    Ok(())
}

fn format_section_name(s: &str) -> String {
    s.split(|c| c == '/' || c == '\\' || c == '-' || c == '_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn format_title_from_stem(stem: &str) -> String {
    stem.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn generate_portal_index(docs: &[DiscoveredMarkdown], site_title: &str) -> DiscoveredMarkdown {
    let mut content = format!("# {}\n\nWelcome to the documentation portal. Below are the available documents:\n\n", site_title);

    let mut current_section = String::new();
    for doc in docs {
        if doc.section != current_section {
            current_section = doc.section.clone();
            content.push_str(&format!("## {}\n\n", current_section));
        }
        let desc = doc
            .frontmatter
            .description
            .as_deref()
            .unwrap_or("Documentation page");
        content.push_str(&format!("- [{}]({}) - {}\n", doc.title, doc.rel_html_path, desc));
    }

    DiscoveredMarkdown {
        rel_md_path: PathBuf::from("index.md"),
        rel_html_path: "index.html".to_string(),
        raw_content: content,
        title: "Overview".to_string(),
        section: "General".to_string(),
        frontmatter: FrontMatter {
            title: Some("Overview".to_string()),
            description: Some(format!("Overview and directory for {}", site_title)),
            ..Default::default()
        },
    }
}

pub fn scaffold_sample_content(input_dir: &Path) -> Result<()> {
    fs::create_dir_all(input_dir)?;

    let index_md = r#"---
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
"#;

    let guide_dir = input_dir.join("guide");
    fs::create_dir_all(&guide_dir)?;

    let getting_started_md = r#"---
title: "Getting Started Guide"
description: "How to use MDRender CLI flags, options, and workflows"
date: "2026-09-09"
author: "MDRender Team"
tags: [guide, cli, usage]
---

# Getting Started Guide

This guide explains how to convert your Markdown files into a complete HTML documentation site.

## Installation & Build

Build the MDRender CLI tool with Cargo:

```bash
cargo build --release
```

The resulting binary is located at `target/release/mdrender`.

---

## Command Line Usage

Run MDRender with default directories (`input/` -> `output/`):

```bash
./target/release/mdrender
```

### Custom Input and Output Folders

Specify custom folders using `-i` / `--input` and `-o` / `--output`:

```bash
mdrender --input ./my-docs --output ./public --title "Project Wiki"
```

### Clean Output Before Build

To ensure no orphaned files remain from deleted markdowns:

```bash
mdrender --clean
```

### Live Rebuilding (Watch Mode)

To automatically rebuild whenever markdown files change:

```bash
mdrender --watch
```

---

## Callout Alerts

MDRender recognizes standard GitHub alert blocks:

> [!TIP]
> You can mix markdown formatting like **bold text**, `code`, and [links](getting-started.html) inside callout boxes!

> [!WARNING]
> Ensure all relative links inside markdown point to `.html` destinations or match the rendered structure.

> [!CAUTION]
> Always run `--clean` if you rename files to avoid stale output.
"#;

    let theming_md = r#"---
title: "Themes and Modes"
description: "Details on available color themes and styling customization"
date: "2026-09-09"
author: "MDRender Team"
tags: [theming, css, darkmode]
---

# Themes & Customization

MDRender comes bundled with 5 beautifully designed themes, each supporting both Light and Dark modes.

## Available Themes

1. **GitHub**
   - Clean, modern, high-contrast palette inspired by GitHub's Primer design system.
2. **Dracula**
   - Iconic dark theme with rich purples, pinks, and cyans.
3. **Nord**
   - Elegant arctic blue-gray palette designed for eye comfort.
4. **Solarized**
   - Ethan Schoonover's scientifically crafted color palette (Light & Dark).
5. **Sepia / Paper**
   - Warm, low-contrast reader theme ideal for prolonged long-form reading.

## Shared Assets Architecture

All pages link to:

- `assets/style.css`
- `assets/theme.js`

Nested pages (e.g. `guide/theming.html`) automatically link to `../assets/style.css`, guaranteeing that assets are cached once by the browser and only regenerated when their contents change!
"#;

    fs::write(input_dir.join("index.md"), index_md)?;
    fs::write(guide_dir.join("getting-started.md"), getting_started_md)?;
    fs::write(guide_dir.join("theming.md"), theming_md)?;

    println!("  ✓ Created sample markdown documents in `{}`", input_dir.display());
    Ok(())
}
