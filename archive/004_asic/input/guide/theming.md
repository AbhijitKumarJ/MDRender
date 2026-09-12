---
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
