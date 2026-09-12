use pulldown_cmark::{
    html, CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TocItem {
    pub level: u32,
    pub title: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct RenderedMarkdown {
    #[allow(dead_code)]
    pub frontmatter: FrontMatter,
    pub title: String,
    pub html_content: String,
    pub toc: Vec<TocItem>,
}

/// Convert heading level enum to integer 1..6
fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Generate a URL-friendly slug from heading text
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut prev_dash = false;

    for c in text.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' {
            if !prev_dash && !slug.is_empty() {
                slug.push('-');
                prev_dash = true;
            }
        }
    }

    // Trim trailing dash
    if slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

/// Parse optional frontmatter block (`---` ... `---`)
pub fn parse_frontmatter(content: &str) -> (FrontMatter, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (FrontMatter::default(), content);
    }

    // Find end of first line
    let first_line_end = match trimmed.find('\n') {
        Some(idx) => idx,
        None => return (FrontMatter::default(), content),
    };

    if trimmed[..first_line_end].trim() != "---" {
        return (FrontMatter::default(), content);
    }

    let rest = &trimmed[first_line_end + 1..];
    let end_idx = match rest.find("\n---") {
        Some(idx) => idx,
        None => return (FrontMatter::default(), content),
    };

    let fm_str = &rest[..end_idx];
    let mut body_start = end_idx + 4; // skip \n---
    if body_start < rest.len() && rest.as_bytes()[body_start] == b'\n' {
        body_start += 1;
    }
    let body = &rest[body_start..];

    let mut fm = FrontMatter::default();

    for line in fm_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let mut val = val.trim().to_string();

            // Strip quotes if present
            if (val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\''))
            {
                if val.len() >= 2 {
                    val = val[1..val.len() - 1].to_string();
                }
            }

            match key.as_str() {
                "title" => fm.title = Some(val),
                "description" => fm.description = Some(val),
                "date" => fm.date = Some(val),
                "author" => fm.author = Some(val),
                "tags" => {
                    // Handle comma-separated or bracketed tags: [a, b, c]
                    let clean = val.trim_matches(|c| c == '[' || c == ']');
                    fm.tags = clean
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {
                    fm.extra.insert(key, val);
                }
            }
        }
    }

    (fm, body)
}

/// Parse and render markdown to HTML with enhanced features:
/// - GFM extensions: tables, footnotes, strikethrough, tasklists
/// - Automatic slug IDs and anchor links for headings
/// - Table of Contents extraction
/// - Code block container with copy button
/// - GitHub-style callouts/admonitions
pub fn render_markdown(raw_input: &str, fallback_title: &str) -> RenderedMarkdown {
    let (frontmatter, markdown_body) = parse_frontmatter(raw_input);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown_body, options);

    let mut toc = Vec::new();
    let mut detected_title = None;
    let mut heading_text_buffer = String::new();
    let mut in_heading = false;
    let mut current_heading_level = HeadingLevel::H1;
    let mut slug_counts: HashMap<String, usize> = HashMap::new();

    let mut transformed_events: Vec<Event> = Vec::new();

    for event in parser {
        match event {
            // Heading start
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_heading_level = level;
                heading_text_buffer.clear();
            }

            // Text inside heading or elsewhere
            Event::Text(ref text) => {
                if in_heading {
                    heading_text_buffer.push_str(text);
                } else {
                    transformed_events.push(event);
                }
            }

            // Code inside heading
            Event::Code(ref code) => {
                if in_heading {
                    heading_text_buffer.push_str(code);
                } else {
                    transformed_events.push(event);
                }
            }

            // Heading end
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let title_text = heading_text_buffer.trim().to_string();
                let num_level = heading_level_to_u32(current_heading_level);

                if detected_title.is_none() && num_level == 1 {
                    detected_title = Some(title_text.clone());
                }

                // Generate unique slug
                let base_slug = slugify(&title_text);
                let count = slug_counts.entry(base_slug.clone()).or_insert(0);
                *count += 1;
                let final_slug = if *count > 1 {
                    format!("{}-{}", base_slug, count)
                } else {
                    base_slug
                };

                // Add to TOC (H2, H3, H4)
                if num_level >= 2 && num_level <= 4 {
                    toc.push(TocItem {
                        level: num_level,
                        title: title_text.clone(),
                        id: final_slug.clone(),
                    });
                }

                // Push raw HTML heading with ID and anchor link
                let tag_name = format!("h{}", num_level);
                let heading_html = format!(
                    "<{tag} id=\"{id}\">{title}<a href=\"#{id}\" class=\"header-anchor\" aria-label=\"Link to section\">#</a></{tag}>",
                    tag = tag_name,
                    id = final_slug,
                    title = html_escape(&title_text)
                );
                transformed_events.push(Event::Html(CowStr::from(heading_html)));
            }

            // Code block start
            Event::Start(Tag::CodeBlock(ref kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let l = lang.trim();
                        if l.is_empty() { "text" } else { l }
                    }
                    CodeBlockKind::Indented => "text",
                };

                let wrapper_html = format!(
                    "<div class=\"code-block-wrapper\">\n  <div class=\"code-block-header\">\n    <span class=\"code-lang\">{}</span>\n    <button class=\"copy-btn\" type=\"button\" aria-label=\"Copy code\">Copy</button>\n  </div>\n",
                    html_escape(lang)
                );
                transformed_events.push(Event::Html(CowStr::from(wrapper_html)));
                transformed_events.push(event);
            }

            // Code block end
            Event::End(TagEnd::CodeBlock) => {
                transformed_events.push(event);
                transformed_events.push(Event::Html(CowStr::from("</div>\n")));
            }

            // Table start: wrap in overflow container
            Event::Start(Tag::Table(_)) => {
                transformed_events.push(Event::Html(CowStr::from("<div class=\"table-wrapper\">\n")));
                transformed_events.push(event);
            }

            // Table end
            Event::End(TagEnd::Table) => {
                transformed_events.push(event);
                transformed_events.push(Event::Html(CowStr::from("</div>\n")));
            }

            // Pass through all other events
            _ => {
                transformed_events.push(event);
            }
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed_events.into_iter());

    // Post-process GitHub-style alerts:
    // <blockquote><p>[!NOTE] ...</p></blockquote>
    let processed_html = process_callouts(&html_output);

    let final_title = frontmatter
        .title
        .clone()
        .or(detected_title)
        .unwrap_or_else(|| fallback_title.to_string());

    RenderedMarkdown {
        frontmatter,
        title: final_title,
        html_content: processed_html,
        toc,
    }
}

/// Converts GitHub-style callouts in blockquotes:
/// `<blockquote>\n<p>[!NOTE]\ncontent...</p>\n</blockquote>`
fn process_callouts(html: &str) -> String {
    let callout_types = [
        ("NOTE", "callout-note", "ℹ️", "Note"),
        ("TIP", "callout-tip", "💡", "Tip"),
        ("IMPORTANT", "callout-important", "🟣", "Important"),
        ("WARNING", "callout-warning", "⚠️", "Warning"),
        ("CAUTION", "callout-caution", "🛑", "Caution"),
    ];

    let mut result = html.to_string();

    for (tag, class_name, icon, label) in callout_types {
        let pattern_upper = format!("<blockquote>\n<p>[!{}]", tag);
        let pattern_lower = format!("<blockquote>\n<p>[!{}]", tag.to_lowercase());

        for pattern in [&pattern_upper, &pattern_lower] {
            while let Some(start_pos) = result.find(pattern.as_str()) {
                let after_prefix = start_pos + pattern.len();
                // Find matching </blockquote>
                if let Some(end_rel) = result[after_prefix..].find("</blockquote>") {
                    let end_pos = after_prefix + end_rel + "</blockquote>".len();
                    let inner_content = &result[after_prefix..after_prefix + end_rel];
                    
                    // Remove optional leading <br> or newline or whitespace
                    let clean_inner = inner_content
                        .trim_start_matches(|c: char| c.is_whitespace() || c == '<' || c == 'b' || c == 'r' || c == '>' || c == '/');

                    let replacement = format!(
                        "<div class=\"callout {}\">\n  <div class=\"callout-title\"><span>{}</span> {}</div>\n  <p>{}</div>",
                        class_name, icon, label, clean_inner
                    );

                    result.replace_range(start_pos..end_pos, &replacement);
                } else {
                    break;
                }
            }
        }
    }

    result
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
    fn test_slugify() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Quick-Start Guide (v1.0)"), "quick-start-guide-v1-0");
        assert_eq!(slugify("   Spaces   "), "spaces");
    }

    #[test]
    fn test_frontmatter() {
        let md = "---\ntitle: Test Page\ndate: 2026-09-09\ntags: [rust, cli]\n---\n# Main Heading\nContent";
        let (fm, body) = parse_frontmatter(md);
        assert_eq!(fm.title.as_deref(), Some("Test Page"));
        assert_eq!(fm.date.as_deref(), Some("2026-09-09"));
        assert_eq!(fm.tags, vec!["rust", "cli"]);
        assert!(body.starts_with("# Main Heading"));
    }

    #[test]
    fn test_markdown_rendering() {
        let md = "# Title\n\n## Section One\n\nSome text with `inline code`.\n\n```rust\nfn main() {}\n```\n";
        let rendered = render_markdown(md, "Fallback");
        assert_eq!(rendered.title, "Title");
        assert_eq!(rendered.toc.len(), 1);
        assert_eq!(rendered.toc[0].title, "Section One");
        assert_eq!(rendered.toc[0].id, "section-one");
        assert!(rendered.html_content.contains("code-block-wrapper"));
        assert!(rendered.html_content.contains("copy-btn"));
    }
}
