---
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
