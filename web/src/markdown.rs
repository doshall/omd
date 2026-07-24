use omd_common::{markdown_to_html_with_toc, parse_front_matter, resolve_title};

pub fn prepare_markdown(input: &str) -> (Option<omd_common::FrontMatter>, String) {
    let (fm, body) = parse_front_matter(input);
    (fm, body.to_string())
}

pub fn markdown_to_html(markdown: &str) -> String {
    let (_, body) = parse_front_matter(markdown);
    markdown_to_html_with_toc(body, true).0
}

pub fn export_title(filename: &str, markdown: &str) -> String {
    let (fm, body) = parse_front_matter(markdown);
    resolve_title(fm.as_ref(), Some(filename), body)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mermaid_transform_keeps_arrows() {
        let md = "```mermaid\nflowchart TD\n    A --> B\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("<div class=\"mermaid\">"));
        assert!(!html.contains("&amp;gt;"));
    }

    #[test]
    fn front_matter_title_for_export() {
        let md = "---\ntitle: From YAML\n---\n\n# Heading\n";
        assert_eq!(export_title("doc.md", md), "From YAML");
    }
}
