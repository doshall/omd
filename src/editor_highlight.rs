use eframe::egui::{self, Color32, FontId, Painter, Pos2, Rect};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdTokenKind {
    Plain,
    Heading,
    CodeFence,
    CodeBody,
    InlineCode,
    Bold,
    Italic,
    Link,
    Image,
    List,
    Blockquote,
    Url,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdToken {
    pub kind: MdTokenKind,
    pub text: String,
}

pub fn tokenize(content: &str) -> Vec<Vec<MdToken>> {
    let mut in_code = false;
    let mut lines = Vec::new();

    for line in content.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        if in_code {
            if body.trim_start().starts_with("```") {
                in_code = false;
                lines.push(vec![MdToken {
                    kind: MdTokenKind::CodeFence,
                    text: line.to_string(),
                }]);
            } else {
                lines.push(vec![MdToken {
                    kind: MdTokenKind::CodeBody,
                    text: line.to_string(),
                }]);
            }
            continue;
        }

        if body.trim_start().starts_with("```") {
            in_code = true;
            lines.push(vec![MdToken {
                kind: MdTokenKind::CodeFence,
                text: line.to_string(),
            }]);
            continue;
        }

        lines.push(tokenize_markdown_line(line));
    }

    if lines.is_empty() {
        lines.push(vec![MdToken {
            kind: MdTokenKind::Plain,
            text: String::new(),
        }]);
    }

    lines
}

fn tokenize_markdown_line(line: &str) -> Vec<MdToken> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        let hashes = trimmed.chars().take_while(|&c| c == '#').count();
        if hashes > 0 && hashes <= 6 {
            let rest = &trimmed[hashes..];
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
                return vec![MdToken {
                    kind: MdTokenKind::Heading,
                    text: line.to_string(),
                }];
            }
        }
    }

    if trimmed.starts_with(">") {
        return vec![MdToken {
            kind: MdTokenKind::Blockquote,
            text: line.to_string(),
        }];
    }

    if is_list_line(trimmed) {
        return vec![MdToken {
            kind: MdTokenKind::List,
            text: line.to_string(),
        }];
    }

    if trimmed.starts_with("![") {
        return vec![MdToken {
            kind: MdTokenKind::Image,
            text: line.to_string(),
        }];
    }

    if trimmed.contains("](") && trimmed.contains(')') {
        return vec![MdToken {
            kind: MdTokenKind::Link,
            text: line.to_string(),
        }];
    }

    tokenize_inline(line)
}

fn is_list_line(trimmed: &str) -> bool {
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
            && trimmed.contains(". ")
}

fn tokenize_inline(line: &str) -> Vec<MdToken> {
    let mut tokens = Vec::new();
    let mut rest = line;

    while !rest.is_empty() {
        if let Some(url) = strip_url_prefix(rest) {
            let (url_text, tail) = take_url(url);
            push_token(&mut tokens, MdTokenKind::Url, url_text);
            rest = tail;
            continue;
        }

        if rest.starts_with("**") {
            if let Some((inner, tail)) = take_wrapped(rest, "**") {
                push_token(&mut tokens, MdTokenKind::Bold, inner);
                rest = tail;
                continue;
            }
        }

        if rest.starts_with('*') && !rest.starts_with("**") {
            if let Some((inner, tail)) = take_wrapped(rest, "*") {
                push_token(&mut tokens, MdTokenKind::Italic, inner);
                rest = tail;
                continue;
            }
        }

        if rest.starts_with('`') {
            if let Some((inner, tail)) = take_wrapped(rest, "`") {
                push_token(&mut tokens, MdTokenKind::InlineCode, inner);
                rest = tail;
                continue;
            }
        }

        let next_special = rest[1..]
            .find(|c| matches!(c, '*' | '`' | 'h'))
            .map(|i| i + 1)
            .unwrap_or(rest.len());

        let (plain, tail) = rest.split_at(next_special);
        push_token(&mut tokens, MdTokenKind::Plain, plain);
        rest = tail;
    }

    if tokens.is_empty() {
        tokens.push(MdToken {
            kind: MdTokenKind::Plain,
            text: line.to_string(),
        });
    }

    tokens
}

fn push_token(tokens: &mut Vec<MdToken>, kind: MdTokenKind, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = tokens.last_mut() {
        if last.kind == kind {
            last.text.push_str(text);
            return;
        }
    }
    tokens.push(MdToken {
        kind,
        text: text.to_string(),
    });
}

fn strip_url_prefix(s: &str) -> Option<&str> {
    if let Some(idx) = s.find("http://") {
        return Some(&s[idx..]);
    }
    if let Some(idx) = s.find("https://") {
        return Some(&s[idx..]);
    }
    None
}

fn take_url(s: &str) -> (&str, &str) {
    let end = s
        .char_indices()
        .find(|(_, c)| c.is_whitespace() || matches!(*c, ')' | ']' | '>' | '"'))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s.split_at(end)
}

fn take_wrapped<'a>(s: &'a str, marker: &str) -> Option<(&'a str, &'a str)> {
    let close = s[marker.len()..].find(marker)? + marker.len();
    let end = close + marker.len();
    Some((&s[..end], &s[end..]))
}

pub fn token_color(kind: MdTokenKind, visuals: &egui::Visuals, dark_mode: bool) -> Color32 {
    match kind {
        MdTokenKind::Plain => visuals.text_color(),
        MdTokenKind::Heading => visuals.strong_text_color(),
        MdTokenKind::CodeFence | MdTokenKind::CodeBody => visuals.weak_text_color(),
        MdTokenKind::InlineCode => {
            if dark_mode {
                Color32::from_rgb(152, 195, 121)
            } else {
                Color32::from_rgb(175, 36, 96)
            }
        }
        MdTokenKind::Bold => visuals.strong_text_color(),
        MdTokenKind::Italic => visuals.weak_text_color(),
        MdTokenKind::Link | MdTokenKind::Url => visuals.hyperlink_color,
        MdTokenKind::Image => Color32::from_rgb(72, 160, 96),
        MdTokenKind::List => visuals.weak_text_color(),
        MdTokenKind::Blockquote => visuals.weak_text_color().gamma_multiply(0.85),
    }
}

pub fn paint_backdrop_in_rect(
    painter: &Painter,
    clip_rect: Rect,
    origin: Pos2,
    content: &str,
    font_id: &FontId,
    line_height: f32,
    dark_mode: bool,
    visuals: &egui::Visuals,
    layout_no_wrap: &impl Fn(String, FontId, Color32) -> Arc<egui::Galley>,
) {
    let mut y = origin.y;
    for line in tokenize(content) {
        if y + line_height >= clip_rect.top() && y <= clip_rect.bottom() {
            let mut x = origin.x;
            for token in line {
                let color = token_color(token.kind, visuals, dark_mode);
                let galley = layout_no_wrap(token.text.clone(), font_id.clone(), color);
                if x < clip_rect.right() {
                    painter.galley(Pos2::new(x, y), galley.clone(), color);
                }
                x += galley.size().x;
            }
        }
        y += line_height;
    }
}

pub fn lines_to_html(content: &str) -> String {
    let mut html = String::new();
    for (line_idx, line) in tokenize(content).into_iter().enumerate() {
        if line_idx > 0 {
            html.push('\n');
        }
        for token in line {
            let class = token_class(token.kind);
            html.push_str(&format!(
                "<span class=\"{class}\">{}</span>",
                html_escape(&token.text)
            ));
        }
    }
    html
}

fn token_class(kind: MdTokenKind) -> &'static str {
    match kind {
        MdTokenKind::Plain => "md-plain",
        MdTokenKind::Heading => "md-heading",
        MdTokenKind::CodeFence | MdTokenKind::CodeBody => "md-code-block",
        MdTokenKind::InlineCode => "md-code",
        MdTokenKind::Bold => "md-bold",
        MdTokenKind::Italic => "md-italic",
        MdTokenKind::Link | MdTokenKind::Url => "md-link",
        MdTokenKind::Image => "md-image",
        MdTokenKind::List => "md-list",
        MdTokenKind::Blockquote => "md-blockquote",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_heading() {
        let lines = tokenize("# Hello\n");
        assert_eq!(lines[0][0].kind, MdTokenKind::Heading);
    }

    #[test]
    fn tokenizes_code_fence() {
        let md = "```rust\nfn main() {}\n```\n";
        let lines = tokenize(md);
        assert_eq!(lines[0][0].kind, MdTokenKind::CodeFence);
        assert_eq!(lines[1][0].kind, MdTokenKind::CodeBody);
    }
}
