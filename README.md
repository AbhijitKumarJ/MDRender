# MDRender
Markdown Renderer

A fast, modern Markdown-to-HTML documentation converter written in Rust. It recursively parses Markdown files from an input directory, preserves directory structure, generates responsive HTML files with full Light/Dark mode and multi-theme support, and manages shared assets in an `assets/` subfolder.

---

## Features

- **🚀 High Performance**: Native Rust implementation using `pulldown-cmark`.
- **🎨 Multi-Theme Support**: Includes 5 built-in themes:
  - **GitHub**: Clean, modern default.
  - **Dracula**: Rich high-contrast dark theme with vibrant accents.
  - **Nord**: Arctic bluish-gray palette designed for focus and eye comfort.
  - **Solarized**: Precision-tuned developer color scheme.
  - **Sepia / Paper**: Warm, relaxed palette for long-form reading.
- **🌓 Light & Dark Modes**:
  - System mode auto-detection (`prefers-color-scheme`).
  - Manual toggle between `Light`, `Dark`, and `Auto`.
  - Anti-FOUC inline hydration script preventing page flicker on reload.
  - Persisted in browser `localStorage`.
- **📁 Shared Assets Architecture**:
  - CSS (`assets/style.css`) and JavaScript (`assets/theme.js`) are written once to `<output>/assets/` and updated only when their contents change.
  - Relative asset links are computed automatically based on file nesting depth (e.g. `./assets/`, `../assets/`, `../../assets/`), making the generated files completely portable offline (`file://`) and on any static web host.
- **🗂️ Recursive Directory Traversal**:
  - Scans `--input` folder recursively.
  - Copies non-markdown resources (images, diagrams, downloads) to the output directory.
  - Automatically generates an `index.html` directory portal if no root `index.md` exists.
- **🛠️ Rich Markdown Features**:
  - GitHub Flavored Markdown (tables, strikethrough, task lists, footnotes).
  - Code blocks with language labels and one-click copy buttons.
  - GitHub-style callouts/admonitions (`[!NOTE]`, `[!TIP]`, `[!WARNING]`, `[!IMPORTANT]`, `[!CAUTION]`).
  - Heading slugs with hoverable `#` anchor links.
  - In-page Table of Contents (TOC) with scroll-based active section highlighting.
  - Frontmatter metadata parsing (title, description, date, author, tags).
- **🔄 Live Watch Mode**: Automatically watches for changes and rebuilds instantly.

---

## Installation & Building

Compile with Cargo:

```bash
cargo build --release
```

The compiled binary will be placed at `target/release/mdrender`.

---

## CLI Usage

### Basic Usage

Convert all markdown files in `./input` to `./output`:

```bash
mdrender
```

Or run directly with cargo:

```bash
cargo run --release
```

### CLI Flags and Options

```
Usage: mdrender [OPTIONS]

Options:
  -i, --input <INPUT>    Input directory containing markdown files [default: input]
  -o, --output <OUTPUT>  Output directory for generated HTML and assets [default: output]
  -t, --title <TITLE>    Site title displayed in header and browser tab [default: "MDRender Docs"]
      --clean            Clean output directory before building
  -w, --watch            Watch input directory for changes and automatically rebuild
      --init             Scaffold sample markdown files in the input directory
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

**Custom directories and title:**
```bash
mdrender -i ./docs -o ./dist -t "My Project Docs"
```

**Clean build:**
```bash
mdrender --clean
```

**Live watch mode for authoring:**
```bash
mdrender --watch
```

**Scaffold sample documentation:**
```bash
mdrender --init
```

---

### Interactive Web UI Dashboard

MDRender features a built-in graphical Web UI dashboard with zero external dependencies. It lets you visually manage documents, write in a split-view Markdown editor with live preview, build the static site, archive folders, and browse past archives:

```bash
# Launch the web dashboard on default port (http://127.0.0.1:3000)
mdrender ui

# Launch and automatically open your default browser
mdrender ui --open

# Custom port or host
mdrender ui -p 8080 --host 0.0.0.0

# Shorthand flag
mdrender --ui
```

Features available in the Web UI:
- 📁 **Active File Explorer**: Create, edit, rename, and delete Markdown documents.
- ✍️ **Split-Screen Editor**: Side-by-side editing with quick Markdown formatting buttons (H1/H2, tables, GFM callout alerts, code blocks, task lists).
- 👁️ **Live Site Preview**: Direct iframe view of the rendered static site (`output/index.html`).
- 🔨 **One-Click Build**: Rebuild documentation directly from the web interface.
- 📦 **One-Click Archive**: Archive active content to numbered folders (e.g. `001_GraphTutorials`) and clean `input/output` while preserving reusable assets.
- 🗄️ **Archive Explorer & One-Click Restore**: Browse past archive packages and restore any archive back to `input/` with one click.
- 🌓 **Dashboard Dark/Light Mode**: Matches your preference.

---

### Archive Command

MDRender includes an `archive` command that copies the current `input` and `output` folders into a numbered subfolder in `archive/` (e.g. `001_Tutorials`, `002_RustBasics`, `003_GraphTutorials`). After copying, it cleans `input` and `output`, **preserving reusable assets** (`output/assets/`) so they don't need to be regenerated:

```bash
# Archive with a preferred name (automatically numbered like 001_GraphTutorials)
mdrender archive GraphTutorials

# Or explicitly pass the name with sequence number
mdrender archive 003_GraphTutorials

# Customize input, output, or archive destinations
mdrender archive GraphTutorials -i ./my-input -o ./my-output --archive-dir ./backup

# Keep input and output files without cleaning
mdrender archive GraphTutorials --no-clean
```

Options for `mdrender archive`:
```
Usage: mdrender archive [OPTIONS] <NAME>

Arguments:
  <NAME>  Preferred name for the archive (e.g. GraphTutorials)

Options:
      --archive-dir <ARCHIVE_DIR>  Archive root directory [default: archive]
  -i, --input <INPUT>              Input directory to archive and clean [default: input]
  -o, --output <OUTPUT>            Output directory to archive and clean [default: output]
      --no-clean                   Do not clean input and output folders after copy
  -h, --help                       Print help
```

---

## Markdown Formatting Guide

### Frontmatter

You can include optional YAML frontmatter at the top of any markdown document:

```markdown
---
title: "Custom Page Title"
description: "Page description for search engines and overview"
date: "2026-09-09"
author: "Your Name"
tags: [rust, markdown, html]
---

# Your Content Starts Here
```

### GitHub Callout Alerts

```markdown
> [!NOTE]
> Helpful background information or context.

> [!TIP]
> Helpful tips and recommendations.

> [!WARNING]
> Important alerts about potential pitfalls.
```

### Code Blocks

````markdown
```rust
fn main() {
    println!("Hello from MDRender!");
}
```
````

Each code block automatically includes a language badge and an interactive "Copy" button.

---

## Project Structure

```
MDRender/
├── Cargo.toml               # Package configuration & dependencies
├── templates/
│   ├── style.css            # Responsive CSS styles and 5 themes (Light & Dark)
│   └── theme.js             # Theme switcher, mode toggle, copy button, TOC observer
├── src/
│   ├── main.rs              # CLI entry point
│   ├── config.rs            # Clap CLI argument definitions
│   ├── assets.rs            # Asset writing & change detection
│   ├── markdown.rs          # Pulldown-cmark GFM parser, TOC & callouts
│   ├── html.rs              # HTML page generator & relative path resolver
│   ├── builder.rs           # Recursive build runner & directory scanner
│   └── watcher.rs           # File watcher for --watch mode
├── input/                   # Default markdown source folder
│   ├── index.md             # Site home page
│   ├── images/              # Static media copied to output
│   └── guide/               # Subdirectory showing directory mirroring
│       ├── getting-started.md
│       └── theming.md
└── output/                  # Generated static HTML site
    ├── index.html
    ├── assets/
    │   ├── style.css        # Common stylesheet
    │   └── theme.js         # Common script
    ├── images/
    └── guide/
        ├── getting-started.html
        └── theming.html
```

---

## License

MIT / Apache 2.0

