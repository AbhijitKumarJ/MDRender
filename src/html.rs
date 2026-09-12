use crate::markdown::{FrontMatter, TocItem};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NavItem {
    pub title: String,
    pub rel_html_path: String, // e.g. "index.html" or "guide/getting-started.html"
    pub section: String,       // e.g. "General" or "Guide"
}

#[derive(Debug, Clone)]
pub struct PageContext<'a> {
    pub site_title: &'a str,
    pub page_title: &'a str,
    pub frontmatter: &'a FrontMatter,
    pub content_html: &'a str,
    pub toc: &'a [TocItem],
    pub current_rel_path: &'a str, // e.g. "guide/getting-started.html"
    pub nav_items: &'a [NavItem],
}

/// Computes the relative asset prefix based on nesting depth.
/// e.g. "index.html" -> "./assets/"
///      "guide/intro.html" -> "../assets/"
///      "a/b/c.html" -> "../../assets/"
pub fn compute_assets_prefix(rel_path: &str) -> String {
    let path = Path::new(rel_path);
    let depth = path.parent().map(|p| p.components().count()).unwrap_or(0);
    if depth == 0 {
        "./assets/".to_string()
    } else {
        let mut prefix = String::new();
        for _ in 0..depth {
            prefix.push_str("../");
        }
        prefix.push_str("assets/");
        prefix
    }
}

/// Computes relative link from one file to another in the site.
/// e.g. from "guide/getting-started.html" to "index.html" -> "../index.html"
///      from "guide/getting-started.html" to "guide/faq.html" -> "faq.html"
///      from "index.html" to "guide/getting-started.html" -> "guide/getting-started.html"
pub fn compute_relative_link(from_file: &str, to_file: &str) -> String {
    if from_file == to_file {
        return "#".to_string();
    }

    let from_parent = Path::new(from_file).parent().unwrap_or_else(|| Path::new(""));
    let to_path = Path::new(to_file);

    let from_components: Vec<_> = from_parent.components().collect();
    let to_components: Vec<_> = to_path.components().collect();

    // Find common prefix length
    let mut common = 0;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    let mut result = PathBuf::new();
    // Go up for every component in from_parent after common
    for _ in common..from_components.len() {
        result.push("..");
    }
    // Go down into to_components
    for comp in &to_components[common..] {
        result.push(comp);
    }

    let res_str = result.to_string_lossy().to_string();
    if res_str.is_empty() {
        ".".to_string()
    } else {
        res_str
    }
}

pub fn render_html_page(ctx: &PageContext) -> String {
    let assets_prefix = compute_assets_prefix(ctx.current_rel_path);
    let home_link = compute_relative_link(ctx.current_rel_path, "index.html");

    // Group nav items by section
    let mut sections: BTreeMap<String, Vec<&NavItem>> = BTreeMap::new();
    for item in ctx.nav_items {
        sections
            .entry(item.section.clone())
            .or_default()
            .push(item);
    }

    // Build sidebar navigation HTML
    let mut nav_html = String::new();
    for (section_name, items) in sections {
        nav_html.push_str(&format!(
            "  <div class=\"nav-section-title\">{}</div>\n  <ul class=\"nav-list\">\n",
            html_escape(&section_name)
        ));
        for item in items {
            let is_active = item.rel_html_path == ctx.current_rel_path;
            let active_class = if is_active { " active" } else { "" };
            let href = compute_relative_link(ctx.current_rel_path, &item.rel_html_path);

            nav_html.push_str(&format!(
                "    <li class=\"nav-item\"><a class=\"nav-link{}\" href=\"{}\">{}</a></li>\n",
                active_class,
                html_escape(&href),
                html_escape(&item.title)
            ));
        }
        nav_html.push_str("  </ul>\n");
    }

    // Build TOC sidebar HTML
    let mut toc_html = String::new();
    if !ctx.toc.is_empty() {
        toc_html.push_str("<aside class=\"toc-sidebar\">\n");
        toc_html.push_str("  <div class=\"toc-title\">On this page</div>\n");
        toc_html.push_str("  <ul class=\"toc-list\">\n");
        for item in ctx.toc {
            toc_html.push_str(&format!(
                "    <li class=\"toc-item depth-{}\"><a class=\"toc-link\" href=\"#{}\">{}</a></li>\n",
                item.level,
                html_escape(&item.id),
                html_escape(&item.title)
            ));
        }
        toc_html.push_str("  </ul>\n");
        toc_html.push_str("</aside>\n");
    }

    // Metadata card if frontmatter has author, date, or tags
    let mut meta_card = String::new();
    let has_meta = ctx.frontmatter.date.is_some()
        || ctx.frontmatter.author.is_some()
        || !ctx.frontmatter.tags.is_empty();

    if has_meta {
        meta_card.push_str("<div class=\"metadata-box\">\n");
        if let Some(date) = &ctx.frontmatter.date {
            meta_card.push_str(&format!(
                "  <div class=\"metadata-item\"><span class=\"metadata-label\">Date:</span> <span>{}</span></div>\n",
                html_escape(date)
            ));
        }
        if let Some(author) = &ctx.frontmatter.author {
            meta_card.push_str(&format!(
                "  <div class=\"metadata-item\"><span class=\"metadata-label\">Author:</span> <span>{}</span></div>\n",
                html_escape(author)
            ));
        }
        if !ctx.frontmatter.tags.is_empty() {
            let tags_str = ctx
                .frontmatter
                .tags
                .iter()
                .map(|t| format!("<code>{}</code>", html_escape(t)))
                .collect::<Vec<_>>()
                .join(" ");
            meta_card.push_str(&format!(
                "  <div class=\"metadata-item\"><span class=\"metadata-label\">Tags:</span> <span>{}</span></div>\n",
                tags_str
            ));
        }
        meta_card.push_str("</div>\n");
    }

    let page_desc_meta = if let Some(desc) = &ctx.frontmatter.description {
        format!("<meta name=\"description\" content=\"{}\">\n", html_escape(desc))
    } else {
        String::new()
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{page_title} - {site_title}</title>
  {page_desc_meta}<link rel="stylesheet" href="{assets_prefix}style.css">
  <!-- Anti-FOUC theme pre-hydration -->
  <script>
    (function() {{
      const theme = localStorage.getItem('mdrender-theme') || 'github';
      const savedMode = localStorage.getItem('mdrender-mode') || 'auto';
      const mode = savedMode === 'auto'
        ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
        : savedMode;
      document.documentElement.setAttribute('data-theme', theme);
      document.documentElement.setAttribute('data-mode', mode);
    }})();
  </script>
</head>
<body>
  <div class="app-container">
    <header class="top-header">
      <div class="header-left">
        <button class="mobile-nav-toggle" type="button" aria-label="Toggle navigation">☰</button>
        <a class="site-title" href="{home_link}">
          <span>📖</span>
          <span>{site_title}</span>
        </a>
      </div>
      <div class="header-right">
        <select id="theme-select" class="control-select" aria-label="Select Theme">
          <option value="github">GitHub</option>
          <option value="dracula">Dracula</option>
          <option value="nord">Nord</option>
          <option value="solarized">Solarized</option>
          <option value="sepia">Sepia</option>
        </select>
        <button id="mode-toggle" class="control-btn mode-toggle-btn" type="button" aria-label="Toggle dark/light mode">
          💻 Auto
        </button>
      </div>
    </header>

    <div class="layout-body">
      <div class="sidebar-backdrop"></div>
      <aside class="sidebar">
        <div class="sidebar-search">
          <input type="text" class="search-input" placeholder="Search pages..." aria-label="Filter navigation">
        </div>
{nav_html}
      </aside>

      <main class="content-area">
{meta_card}        <article>
{content_html}
        </article>
        <footer class="page-footer">
          <span>Generated with <strong>MDRender</strong></span>
          <a href="#top" class="back-to-top">↑ Back to top</a>
        </footer>
      </main>

{toc_html}
    </div>
  </div>

  <script src="{assets_prefix}theme.js"></script>
</body>
</html>
"##,
        page_title = html_escape(ctx.page_title),
        site_title = html_escape(ctx.site_title),
        page_desc_meta = page_desc_meta,
        assets_prefix = assets_prefix,
        home_link = html_escape(&home_link),
        nav_html = nav_html,
        meta_card = meta_card,
        content_html = ctx.content_html,
        toc_html = toc_html,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_prefix() {
        assert_eq!(compute_assets_prefix("index.html"), "./assets/");
        assert_eq!(compute_assets_prefix("guide/intro.html"), "../assets/");
        assert_eq!(compute_assets_prefix("a/b/c.html"), "../../assets/");
    }

    #[test]
    fn test_relative_link() {
        assert_eq!(compute_relative_link("index.html", "index.html"), "#");
        assert_eq!(
            compute_relative_link("guide/getting-started.html", "index.html"),
            "../index.html"
        );
        assert_eq!(
            compute_relative_link("index.html", "guide/getting-started.html"),
            "guide/getting-started.html"
        );
        assert_eq!(
            compute_relative_link("guide/a.html", "guide/b.html"),
            "b.html"
        );
    }
}
