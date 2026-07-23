use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockPos {
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRect {
    pub line_start: usize,
    pub line_end: usize,
    pub col_start: usize,
    pub col_end: usize,
}

impl BlockRect {
    pub fn from_positions(a: BlockPos, b: BlockPos) -> Self {
        Self {
            line_start: a.line.min(b.line),
            line_end: a.line.max(b.line),
            col_start: a.col.min(b.col),
            col_end: a.col.max(b.col),
        }
    }
}

pub fn pos_to_block_pos(content: &str, pos: usize) -> BlockPos {
    let line = content.chars().take(pos).filter(|&c| c == '\n').count();
    let mut line_start = 0usize;
    for (i, c) in content.chars().enumerate().take(pos) {
        if c == '\n' {
            line_start = i + 1;
        }
    }
    BlockPos {
        line,
        col: pos - line_start,
    }
}

pub fn block_pos_to_char_index(content: &str, pos: BlockPos) -> usize {
    let (start, end) = line_col_range(content, pos.line);
    (start + pos.col).min(end)
}

pub fn line_col_range(content: &str, line: usize) -> (usize, usize) {
    let mut current_line = 0usize;
    let mut start = 0usize;
    for (i, ch) in content.chars().enumerate() {
        if current_line == line {
            let end = content
                .chars()
                .skip(start)
                .position(|c| c == '\n')
                .map(|p| start + p)
                .unwrap_or_else(|| content.chars().count());
            return (start, end);
        }
        if ch == '\n' {
            current_line += 1;
            start = i + 1;
        }
    }
    (start, content.chars().count())
}

pub fn yank_block(content: &str, rect: BlockRect) -> String {
    let mut out = String::new();
    for line in rect.line_start..=rect.line_end {
        let (start, end) = line_col_range(content, line);
        let line_len = end.saturating_sub(start);
        let col_start = rect.col_start.min(line_len);
        let col_end = rect.col_end.min(line_len);
        for (i, ch) in content.chars().skip(start).take(end - start).enumerate() {
            if i >= col_start && i < col_end {
                out.push(ch);
            }
        }
        if line < rect.line_end {
            out.push('\n');
        }
    }
    out
}

pub fn delete_block(content: &mut String, rect: BlockRect) -> usize {
    let mut cursor = block_pos_to_char_index(content, BlockPos {
        line: rect.line_start,
        col: rect.col_start,
    });
    for line in (rect.line_start..=rect.line_end).rev() {
        let (start, end) = line_col_range(content, line);
        let line_len = end.saturating_sub(start);
        let col_start = rect.col_start.min(line_len);
        let col_end = rect.col_end.min(line_len);
        if col_start < col_end {
            let del_start = start + col_start;
            let del_end = start + col_end;
            let (sb, eb) = char_range_to_bytes(content, del_start, del_end);
            content.replace_range(sb..eb, "");
            cursor = del_start;
        }
    }
    cursor
}

fn char_range_to_bytes(content: &str, start: usize, end: usize) -> (usize, usize) {
    let sb = content
        .char_indices()
        .nth(start)
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    let eb = content
        .char_indices()
        .nth(end)
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    (sb, eb)
}

#[derive(Clone, Default)]
pub struct RegisterFile {
    pub unnamed: String,
    pub named: HashMap<char, String>,
    pub pending: Option<char>,
    pub pending_quote: bool,
}

impl RegisterFile {
    pub fn yank(&mut self, reg: Option<char>, text: String) {
        let reg = reg.unwrap_or('"');
        if reg == '_' {
            return;
        }
        if reg == '"' || reg == '0' {
            self.unnamed = text.clone();
            self.named.insert('0', text);
            return;
        }
        self.named.insert(reg, text.clone());
        if reg.is_ascii_lowercase() {
            self.unnamed = text;
        }
    }

    pub fn get(&self, reg: char) -> Option<&str> {
        match reg {
            '"' => Some(&self.unnamed),
            '0' => self.named.get(&'0').map(|s| s.as_str()).or(Some(&self.unnamed)),
            '_' => Some(""),
            c => self.named.get(&c).map(|s| s.as_str()),
        }
    }

    pub fn take_pending(&mut self) -> Option<char> {
        self.pending.take()
    }

    pub fn format_all(&self) -> String {
        let mut lines = vec![format!("\" {}\n", truncate(&self.unnamed, 40))];
        if let Some(z) = self.named.get(&'0') {
            lines.push(format!("0 {}\n", truncate(z, 40)));
        }
        let mut keys: Vec<char> = self.named.keys().copied().filter(|c| *c != '0').collect();
        keys.sort();
        for k in keys {
            if let Some(v) = self.named.get(&k) {
                lines.push(format!("{k} {}\n", truncate(v, 40)));
            }
        }
        lines.join("")
    }
}

fn truncate(s: &str, max: usize) -> String {
    let chars: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        format!("{chars}…")
    } else {
        chars
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VimCommandResult {
    pub status: String,
    pub cursor: Option<usize>,
    pub content_changed: bool,
    pub request_save: bool,
    pub line_numbers: Option<bool>,
}

pub fn execute_vim_command(cmd: &str, content: &mut String, cursor: usize) -> VimCommandResult {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return VimCommandResult {
            status: "Cancelled".to_string(),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    if trimmed == "help" || trimmed == "h" {
        return VimCommandResult {
            status: "Commands: w q wq %s/pat/rep/g [n] set number|nonumber reg".to_string(),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    if trimmed == "w" || trimmed == "write" {
        return VimCommandResult {
            status: "Saving…".to_string(),
            cursor: None,
            content_changed: false,
            request_save: true,
            line_numbers: None,
        };
    }

    if trimmed == "q" || trimmed == "quit" {
        return VimCommandResult {
            status: "Use File → Exit or close window to quit".to_string(),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    if trimmed == "wq" || trimmed == "x" {
        return VimCommandResult {
            status: "Saving…".to_string(),
            cursor: None,
            content_changed: false,
            request_save: true,
            line_numbers: None,
        };
    }

    if trimmed == "reg" || trimmed == "registers" {
        return VimCommandResult {
            status: "See status message after :reg".to_string(),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    if let Some(rest) = trimmed.strip_prefix("set ") {
        return match rest.trim() {
            "number" | "nu" => VimCommandResult {
                status: "Line numbers on".to_string(),
                cursor: None,
                content_changed: false,
                request_save: false,
                line_numbers: Some(true),
            },
            "nonumber" | "nonu" => VimCommandResult {
                status: "Line numbers off".to_string(),
                cursor: None,
                content_changed: false,
                request_save: false,
                line_numbers: Some(false),
            },
            _ => VimCommandResult {
                status: format!("Unknown option: {rest}"),
                cursor: None,
                content_changed: false,
                request_save: false,
                line_numbers: None,
            },
        };
    }

    if let Some(line_num) = trimmed.parse::<usize>().ok() {
        let target = line_num.saturating_sub(1);
        let mut line = 0usize;
        let mut pos = 0usize;
        for (i, ch) in content.chars().enumerate() {
            if line == target {
                return VimCommandResult {
                    status: format!("Line {line_num}"),
                    cursor: Some(pos),
                    content_changed: false,
                    request_save: false,
                    line_numbers: None,
                };
            }
            if ch == '\n' {
                line += 1;
                pos = i + 1;
            }
        }
        if line == target {
            return VimCommandResult {
                status: format!("Line {line_num}"),
                cursor: Some(pos),
                content_changed: false,
                request_save: false,
                line_numbers: None,
            };
        }
        return VimCommandResult {
            status: format!("Invalid line number: {line_num}"),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    if let Some(result) = parse_substitute(trimmed, content, cursor) {
        return result;
    }

    VimCommandResult {
        status: format!("Unknown command: {trimmed}"),
        cursor: None,
        content_changed: false,
        request_save: false,
        line_numbers: None,
    }
}

fn parse_substitute(cmd: &str, content: &mut String, cursor: usize) -> Option<VimCommandResult> {
    let body = cmd.strip_prefix('%')?.strip_prefix('s')?;
    let body = body.strip_prefix('/').or_else(|| body.strip_prefix('#'))?;
    let sep = cmd.chars().nth(if cmd.starts_with('%') { 2 } else { 1 })?;

    let parts: Vec<&str> = body.split(sep).collect();
    if parts.len() < 2 {
        return None;
    }
    let pattern = parts[0];
    let replacement = parts[1];
    let global = parts.get(2).is_some_and(|f| f.contains('g'));

    if cmd.starts_with('%') || global {
        let count = replace_all_literal(content, pattern, replacement);
        return Some(VimCommandResult {
            status: format!("{count} substitution(s)"),
            cursor: Some(cursor.min(content.chars().count())),
            content_changed: count > 0,
            request_save: false,
            line_numbers: None,
        });
    }

    let line = content.chars().take(cursor).filter(|&c| c == '\n').count();
    let (start, end) = line_col_range(content, line);
    let (sb, eb) = char_range_to_bytes(content, start, end);
    let line_text = content[sb..eb].to_string();
    if let Some(new_line) = replace_first_literal(&line_text, pattern, replacement) {
        content.replace_range(sb..eb, &new_line);
        return Some(VimCommandResult {
            status: "1 substitution on line".to_string(),
            cursor: Some(cursor),
            content_changed: true,
            request_save: false,
            line_numbers: None,
        });
    }
    Some(VimCommandResult {
        status: "Pattern not found".to_string(),
        cursor: Some(cursor),
        content_changed: false,
        request_save: false,
        line_numbers: None,
    })
}

fn replace_first_literal(haystack: &str, needle: &str, rep: &str) -> Option<String> {
    haystack.replacen(needle, rep, 1).ne(haystack).then(|| haystack.replacen(needle, rep, 1))
}

fn replace_all_literal(content: &mut String, needle: &str, rep: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0usize;
    let mut search = 0usize;
    while search < content.len() {
        if let Some(idx) = content[search..].find(needle) {
            let abs = search + idx;
            content.replace_range(abs..abs + needle.len(), rep);
            count += 1;
            search = abs + rep.len();
        } else {
            break;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yanks_block_columns() {
        let text = "abcd\nefgh\nijkl";
        let rect = BlockRect::from_positions(
            BlockPos { line: 0, col: 1 },
            BlockPos { line: 2, col: 3 },
        );
        assert_eq!(yank_block(text, rect), "bc\nfg\njk");
    }

    #[test]
    fn goto_line_command() {
        let mut text = "a\nb\nc".to_string();
        let r = execute_vim_command("2", &mut text, 0);
        assert_eq!(r.cursor, Some(2));
    }
}
