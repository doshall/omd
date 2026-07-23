use egui::Color32;
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub type HighlightLine = Vec<(Color32, String)>;

pub fn highlight_lines(code: &str, lang: Option<&str>, dark_mode: bool) -> Vec<HighlightLine> {
    let theme_name = if dark_mode {
        "base16-ocean.dark"
    } else {
        "InspiredGitHub"
    };
    let theme = &THEME_SET.themes[theme_name];

    let syntax = lang
        .and_then(|l| SYNTAX_SET.find_syntax_by_token(l))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    if code.is_empty() {
        lines.push(vec![(Color32::GRAY, String::new())]);
        return lines;
    }

    for line in LinesWithEndings::from(code) {
        let tokens = match highlighter.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = style.foreground;
                    (
                        Color32::from_rgb(fg.r, fg.g, fg.b),
                        text.to_string(),
                    )
                })
                .collect(),
            Err(_) => vec![(Color32::GRAY, line.to_string())],
        };
        lines.push(if tokens.is_empty() {
            vec![(Color32::GRAY, String::new())]
        } else {
            tokens
        });
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust_keyword() {
        let lines = highlight_lines("fn main() {}", Some("rust"), true);
        assert!(!lines.is_empty());
        let joined: String = lines[0].iter().map(|(_, t)| t.as_str()).collect();
        assert!(joined.contains("fn"));
    }
}
