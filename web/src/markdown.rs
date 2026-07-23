use pulldown_cmark::{html, Options, Parser};

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    transform_mermaid_blocks(&html_output)
}

/// Convert fenced mermaid code blocks to `<div class="mermaid">` for mermaid.js.
fn transform_mermaid_blocks(html: &str) -> String {
    let marker = "<pre><code class=\"language-mermaid\">";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(marker) {
        out.push_str(&rest[..start]);
        let after = &rest[start + marker.len()..];
        if let Some(end) = after.find("</code></pre>") {
            let code = html_escape(&after[..end]);
            out.push_str("<div class=\"mermaid\">");
            out.push_str(&code);
            out.push_str("</div>");
            rest = &after[end + "</code></pre>".len()..];
        } else {
            out.push_str(marker);
            rest = after;
            break;
        }
    }
    out.push_str(rest);
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn word_count(text: &str) -> usize {
    text.split_whitespace().filter(|w| !w.is_empty()).count()
}

pub fn line_count(text: &str) -> usize {
    if text.is_empty() {
        1
    } else {
        text.lines().count()
    }
}

pub fn wrap_selection(text: &str, start: usize, end: usize, wrapper: &str) -> String {
    let start = start.min(text.len());
    let end = end.max(start).min(text.len());
    let selected = &text[start..end];

    let (open, close) = match wrapper {
        "**" => ("**", "**"),
        "*" => ("*", "*"),
        "~~" => ("~~", "~~"),
        "`" => ("`", "`"),
        "[]()" => ("[", "](url)"),
        _ => (wrapper, ""),
    };

    format!("{}{open}{selected}{close}{}", &text[..start], &text[end..])
}

pub fn prefix_lines(text: &str, start: usize, end: usize, prefix: &str) -> String {
    let start = start.min(text.len());
    let end = end.max(start).min(text.len());

    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[end..]
        .find('\n')
        .map(|i| end + i)
        .unwrap_or(text.len());

    let block = &text[line_start..line_end];
    let prefixed: String = block
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!("{}{}{}", &text[..line_start], prefixed, &text[line_end..])
}

pub fn insert_at_cursor(text: &str, cursor: usize, insertion: &str) -> String {
    let cursor = cursor.min(text.len());
    format!("{}{}{}", &text[..cursor], insertion, &text[cursor..])
}

pub fn image_markdown(alt: &str, url: &str) -> String {
    format!("\n![{alt}]({url})\n")
}
