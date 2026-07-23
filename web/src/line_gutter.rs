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

pub fn char_index_to_utf16(content: &str, char_idx: usize) -> u32 {
    content
        .chars()
        .take(char_idx)
        .map(|c| c.len_utf16() as u32)
        .sum()
}

pub fn utf16_to_char_index(content: &str, utf16: u32) -> usize {
    let mut pos = 0u32;
    for (i, ch) in content.chars().enumerate() {
        if pos >= utf16 {
            return i;
        }
        pos += ch.len_utf16() as u32;
    }
    content.chars().count()
}

pub fn block_highlight_style(
    line: usize,
    col_start: usize,
    col_end: usize,
    scroll_top: f64,
    font_size: f64,
    line_height: f64,
) -> String {
    let top = highlight_top_px(line, scroll_top, font_size, line_height);
    let width_cols = col_end.saturating_sub(col_start).max(1);
    format!(
        "top:{top}px;left:calc(var(--editor-pad) + {col_start}ch);width:calc({width_cols}ch);height:calc(var(--editor-font-size) * var(--editor-line-height));"
    )
}

pub fn set_char_selection(
    textarea: &web_sys::HtmlTextAreaElement,
    content: &str,
    cursor: usize,
    selection: Option<(usize, usize)>,
) {
    let (start, end) = if let Some((a, b)) = selection {
        (
            char_index_to_utf16(content, a),
            char_index_to_utf16(content, b),
        )
    } else {
        let p = char_index_to_utf16(content, cursor);
        (p, p)
    };
    textarea.set_selection_start(Some(start)).ok();
    textarea.set_selection_end(Some(end)).ok();
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
