use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

pub const STYLE_CSS: &str = include_str!("../templates/style.css");
pub const THEME_JS: &str = include_str!("../templates/theme.js");

/// Ensures that common assets (CSS and JS) are written to <output>/assets/
/// only when they do not exist or when content has changed.
pub fn ensure_assets(output_dir: &Path) -> Result<bool> {
    let assets_dir = output_dir.join("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(&assets_dir)
            .with_context(|| format!("Failed to create assets dir at {}", assets_dir.display()))?;
    }

    let css_path = assets_dir.join("style.css");
    let js_path = assets_dir.join("theme.js");

    let mut changed = false;

    if write_if_different(&css_path, STYLE_CSS.as_bytes())? {
        println!("  ✓ Wrote assets/style.css");
        changed = true;
    } else {
        println!("  • assets/style.css is up to date (unchanged)");
    }

    if write_if_different(&js_path, THEME_JS.as_bytes())? {
        println!("  ✓ Wrote assets/theme.js");
        changed = true;
    } else {
        println!("  • assets/theme.js is up to date (unchanged)");
    }

    Ok(changed)
}

/// Writes `content` to `path` only if `path` doesn't exist or its contents differ.
/// Returns true if file was created or overwritten, false if unchanged.
fn write_if_different(path: &Path, content: &[u8]) -> Result<bool> {
    if path.exists() {
        if let Ok(existing) = fs::read(path) {
            if existing == content {
                return Ok(false);
            }
        }
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write asset to {}", path.display()))?;
    Ok(true)
}

