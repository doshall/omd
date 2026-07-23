pub fn line_count(content: &str) -> usize {
    if content.is_empty() {
        1
    } else {
        content.chars().filter(|&c| c == '\n').count() + 1
    }
}

/// Map a UTF-16 selection offset (as returned by `textarea.selectionStart`) to a line index.
pub fn line_index_at_utf16(content: &str, utf16_offset: u32) -> usize {
    let mut utf16_pos = 0u32;
    let mut line = 0usize;
    for ch in content.chars() {
        if utf16_pos >= utf16_offset {
            return line;
        }
        if ch == '\n' {
            line += 1;
        }
        utf16_pos += ch.len_utf16() as u32;
    }
    line
}

pub fn highlight_top_px(
    line: usize,
    scroll_top: f64,
    font_size: f64,
    line_height: f64,
) -> f64 {
    let pad = font_size * 0.857;
    pad + line as f64 * font_size * line_height - scroll_top
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_line_index() {
        let text = "foo\nbar";
        assert_eq!(line_index_at_utf16(text, 0), 0);
        assert_eq!(line_index_at_utf16(text, 4), 1);
    }
}
