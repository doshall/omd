use pulldown_cmark::{html, HeadingLevel, Options, Parser, TagEnd};
use std::collections::HashMap;

use crate::locale::{t, Locale};

#[derive(Debug, Clone, Copy)]
pub struct TocOptions {
    pub max_level: HeadingLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub level: u8,
    pub title: String,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownRenderOptions {
    pub include_toc: bool,
    pub enable_footnotes: bool,
    pub locale: Locale,
}

impl Default for MarkdownRenderOptions {
    fn default() -> Self {
        Self {
            include_toc: true,
            enable_footnotes: true,
            locale: Locale::default(),
        }
    }
}

pub fn markdown_options() -> Options {
    markdown_options_with(true)
}

pub fn markdown_options_with(enable_footnotes: bool) -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_MATH);
    if enable_footnotes {
        options.insert(Options::ENABLE_FOOTNOTES);
    }
    options
}

pub fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !last_dash && !slug.is_empty() {
                slug.push('-');
                last_dash = true;
            }
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

pub fn collect_headings(markdown: &str) -> Vec<TocEntry> {
    let options = markdown_options();
    let parser = Parser::new_ext(markdown, options);
    let mut headings = Vec::new();
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut in_heading = false;
    let mut heading_level = HeadingLevel::H1;
    let mut heading_buf = String::new();

    for event in parser {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = level;
                heading_buf.clear();
            }
            pulldown_cmark::Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    let base = slugify(&heading_buf);
                    let count = slug_counts.entry(base.clone()).or_insert(0);
                    *count += 1;
                    let id = if *count == 1 {
                        base
                    } else {
                        format!("{base}-{}", *count)
                    };
                    headings.push(TocEntry {
                        level: heading_level as u8,
                        title: heading_buf.trim().to_string(),
                        id,
                    });
                }
                in_heading = false;
            }
            pulldown_cmark::Event::Text(text) if in_heading => heading_buf.push_str(&text),
            _ => {}
        }
    }
    headings
}

pub fn markdown_to_html(markdown: &str) -> String {
    markdown_to_html_with_options(markdown, MarkdownRenderOptions::default())
}

pub fn markdown_to_html_with_toc(markdown: &str, include_toc: bool) -> (String, Vec<TocEntry>) {
    markdown_to_html_with_options_parts(
        markdown,
        MarkdownRenderOptions {
            include_toc,
            ..Default::default()
        },
    )
}

pub fn markdown_to_html_with_options(
    markdown: &str,
    options: MarkdownRenderOptions,
) -> String {
    markdown_to_html_with_options_parts(markdown, options).0
}

pub fn markdown_to_html_with_options_parts(
    markdown: &str,
    options: MarkdownRenderOptions,
) -> (String, Vec<TocEntry>) {
    let headings = collect_headings(markdown);
    let parser_options = markdown_options_with(options.enable_footnotes);
    let parser = Parser::new_ext(markdown, parser_options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    let html_output = transform_diagram_blocks(&html_output);
    let html_output = inject_heading_ids(&html_output, &headings);

    let toc_html = if options.include_toc && !headings.is_empty() {
        render_toc_html(&headings, options.locale)
    } else {
        String::new()
    };

    (format!("{toc_html}{html_output}"), headings)
}

fn inject_heading_ids(html: &str, headings: &[TocEntry]) -> String {
    let mut out = html.to_string();
    for entry in headings {
        let open = format!("<h{}>", entry.level);
        let open_id = format!("<h{} id=\"{}\">", entry.level, entry.id);
        if let Some(pos) = out.find(&open) {
            out.replace_range(pos..pos + open.len(), &open_id);
        }
    }
    out
}

fn render_toc_html(headings: &[TocEntry], locale: Locale) -> String {
    let label = t(locale, "toc");
    let mut out = format!(
        "<nav class=\"toc\" aria-label=\"{label}\"><strong>{label}</strong><ul>"
    );
    for entry in headings {
        let indent = (entry.level.saturating_sub(1) as usize) * 16;
        out.push_str(&format!(
            "<li style=\"margin-left:{indent}px\"><a href=\"#{}\">{}</a></li>",
            entry.id,
            html_escape_text(&entry.title)
        ));
    }
    out.push_str("</ul></nav>");
    out
}

fn html_escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn transform_diagram_blocks(html: &str) -> String {
    let html = transform_codeblock(html, "mermaid", "mermaid");
    let html = transform_codeblock(&html, "plantuml", "plantuml");
    let html = transform_codeblock(&html, "graphviz", "graphviz");
    transform_codeblock(&html, "dot", "graphviz")
}

fn transform_codeblock(html: &str, language: &str, class_name: &str) -> String {
    let marker = format!("<pre><code class=\"language-{language}\">");
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(&marker) {
        out.push_str(&rest[..start]);
        let after = &rest[start + marker.len()..];
        if let Some(end) = after.find("</code></pre>") {
            let code = &after[..end];
            out.push_str(&format!("<div class=\"{class_name}\">"));
            out.push_str(code);
            out.push_str("</div>");
            rest = &after[end + "</code></pre>".len()..];
        } else {
            out.push_str(&marker);
            rest = after;
            break;
        }
    }
    out.push_str(rest);
    out
}

/// Replace pulldown-cmark disabled task checkboxes with interactive ones.
pub fn make_task_lists_interactive(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;
    let mut task_index = 0usize;

    while let Some(pos) = rest.find("<input") {
        result.push_str(&rest[..pos]);
        let after = &rest[pos..];
        if let Some(end) = after.find('>') {
            let tag = &after[..=end];
            rest = &after[end + 1..];
            if tag.contains("type=\"checkbox\"") {
                let checked = tag.contains("checked");
                let checked_attr = if checked { " checked" } else { "" };
                result.push_str(&format!(
                    "<input type=\"checkbox\" data-omd-task=\"{task_index}\" class=\"omd-task\"{checked_attr}/>"
                ));
                task_index += 1;
            } else {
                result.push_str(tag);
            }
        } else {
            result.push_str(rest);
            return result;
        }
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::Locale;

    #[test]
    fn interactive_task_lists() {
        let md = "- [ ] todo\n- [x] done\n";
        let html = markdown_to_html(md);
        let interactive = make_task_lists_interactive(&html);
        assert!(interactive.contains("data-omd-task=\"0\""));
        assert!(interactive.contains("data-omd-task=\"1\""));
        assert!(!interactive.contains("disabled"));
    }

    #[test]
    fn footnotes_render() {
        let md = "Text[^1]\n\n[^1]: footnote body";
        let html = markdown_to_html(md);
        assert!(html.contains("footnote") || html.contains("fnref"));
    }

    #[test]
    fn toc_contains_heading_links() {
        let md = "# Alpha\n\n## Beta\n\nParagraph.";
        let (html, entries) = markdown_to_html_with_toc(md, true);
        assert_eq!(entries.len(), 2);
        assert!(html.contains("class=\"toc\""));
        assert!(html.contains("href=\"#alpha\""));
        assert!(html.contains("<h1 id=\"alpha\">"));
    }

    #[test]
    fn toc_can_be_disabled() {
        let md = "# Alpha\n\nBody.";
        let html = markdown_to_html_with_options(
            md,
            MarkdownRenderOptions {
                include_toc: false,
                enable_footnotes: true,
                ..Default::default()
            },
        );
        assert!(!html.contains("class=\"toc\""));
    }

    #[test]
    fn footnotes_can_be_disabled() {
        let md = "Text[^1]\n\n[^1]: footnote body";
        let html = markdown_to_html_with_options(
            md,
            MarkdownRenderOptions {
                include_toc: true,
                enable_footnotes: false,
                ..Default::default()
            },
        );
        assert!(!html.contains("class=\"footnotes\""));
        assert!(!html.contains("fnref"));
    }

    #[test]
    fn plantuml_blocks_transform() {
        let md = "```plantuml\n@startuml\nA -> B\n@enduml\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("class=\"plantuml\""));
    }

    #[test]
    fn graphviz_blocks_transform() {
        let md = "```dot\ndigraph { a -> b }\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("class=\"graphviz\""));
    }

    #[test]
    fn toc_uses_locale_label() {
        let md = "# Alpha\n\nBody.";
        let html = markdown_to_html_with_options(
            md,
            MarkdownRenderOptions {
                locale: Locale::En,
                ..Default::default()
            },
        );
        assert!(html.contains("aria-label=\"Contents\""));
    }
}
