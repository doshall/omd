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

/// Insert `text` at `col` on every line in `rect` (bottom-up to preserve indices).
pub fn insert_block_column(content: &mut String, rect: BlockRect, col: usize, text: &str) -> usize {
    if text.is_empty() {
        return block_pos_to_char_index(
            content,
            BlockPos {
                line: rect.line_start,
                col,
            },
        );
    }
    for line in (rect.line_start..=rect.line_end).rev() {
        let (start, end) = line_col_range(content, line);
        let line_len = end.saturating_sub(start);
        let insert_col = col.min(line_len);
        let char_pos = start + insert_col;
        let (sb, _) = char_range_to_bytes(content, char_pos, char_pos);
        content.insert_str(sb, text);
    }
    block_pos_to_char_index(
        content,
        BlockPos {
            line: rect.line_start,
            col: col + text.chars().count(),
        },
    )
}

/// Delete one character before `col` on every line in `rect`.
pub fn delete_block_column_char(content: &mut String, rect: BlockRect, col: usize) -> usize {
    if col == 0 {
        return block_pos_to_char_index(
            content,
            BlockPos {
                line: rect.line_start,
                col: 0,
            },
        );
    }
    for line in (rect.line_start..=rect.line_end).rev() {
        let (start, end) = line_col_range(content, line);
        let line_len = end.saturating_sub(start);
        let del_col = col.saturating_sub(1).min(line_len);
        if del_col < line_len {
            let (sb, eb) = char_range_to_bytes(content, start + del_col, start + del_col + 1);
            content.replace_range(sb..eb, "");
        }
    }
    block_pos_to_char_index(
        content,
        BlockPos {
            line: rect.line_start,
            col: col.saturating_sub(1),
        },
    )
}

/// Apply a single normal-mode command at the start of `line` (for `:g/pat/norm`).
pub fn apply_norm_at_line(content: &mut String, line: usize, cmd: &str) -> bool {
    let trimmed = cmd.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed == "dd" {
        delete_line_at(content, line);
        return true;
    }
    if trimmed == ">>" {
        indent_line_at(content, line);
        return true;
    }
    if trimmed == "<<" {
        return unindent_line_at(content, line);
    }
    if trimmed == "x" {
        let (start, end) = line_col_range(content, line);
        if start < end {
            let (sb, eb) = char_range_to_bytes(content, start, start + 1);
            content.replace_range(sb..eb, "");
            return true;
        }
        return false;
    }
    if trimmed.starts_with('I') {
        let text: String = trimmed.chars().skip(1).collect();
        let (start, _) = line_col_range(content, line);
        let (sb, _) = char_range_to_bytes(content, start, start);
        content.insert_str(sb, &text);
        return true;
    }
    if trimmed.starts_with('A') {
        let text: String = trimmed.chars().skip(1).collect();
        let (_, end) = line_col_range(content, line);
        let (sb, _) = char_range_to_bytes(content, end, end);
        content.insert_str(sb, &text);
        return true;
    }
    if trimmed == "dw" {
        let (start, end) = line_col_range(content, line);
        let mut pos = start;
        while pos < end && content.chars().nth(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        let mut word_end = pos;
        if let Some(ch) = content.chars().nth(pos) {
            if ch.is_alphanumeric() || ch == '_' {
                while word_end < end {
                    let c = content.chars().nth(word_end).unwrap();
                    if !c.is_alphanumeric() && c != '_' {
                        break;
                    }
                    word_end += 1;
                }
            } else {
                word_end = pos + 1;
            }
        }
        if word_end > pos {
            let (sb, eb) = char_range_to_bytes(content, pos, word_end);
            content.replace_range(sb..eb, "");
            return true;
        }
        return false;
    }
    false
}

fn indent_line_at(content: &mut String, line: usize) {
    let (start, _) = line_col_range(content, line);
    let (sb, _) = char_range_to_bytes(content, start, start);
    content.insert_str(sb, "    ");
}

fn unindent_line_at(content: &mut String, line: usize) -> bool {
    let (start, end) = line_col_range(content, line);
    let prefix: String = content.chars().skip(start).take(4.min(end - start)).collect();
    if prefix == "    " {
        let (sb, eb) = char_range_to_bytes(content, start, start + 4);
        content.replace_range(sb..eb, "");
        return true;
    }
    if prefix.starts_with('\t') {
        let (sb, eb) = char_range_to_bytes(content, start, start + 1);
        content.replace_range(sb..eb, "");
        return true;
    }
    false
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
            '+' | '*' => self
                .named
                .get(&reg)
                .map(|s| s.as_str())
                .or(Some(&self.unnamed)),
            c => self.named.get(&c).map(|s| s.as_str()),
        }
    }

    pub fn store_clipboard_register(&mut self, text: &str) {
        self.named.insert('+', text.to_string());
        self.named.insert('*', text.to_string());
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
    if cmd.trim().is_empty() {
        return VimCommandResult {
            status: "Cancelled".to_string(),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }
    let trimmed = cmd.trim_start();

    if trimmed == "help" || trimmed == "h" {
        return VimCommandResult {
            status: "Commands: w q wq [n] [a,b]d g/pat/d g/pat/s/o/n/g g/pat/norm cmd v/pat/d %s/pat/rep/g set number|nonumber reg".to_string(),
            cursor: None,
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    if let Some(result) = parse_global_command(trimmed, content, cursor) {
        return result;
    }

    let (range_start, range_end, rest) = parse_line_range(trimmed);
    if let Some(result) = execute_ranged_command(range_start, range_end, rest, content, cursor) {
        return result;
    }

    execute_simple_command(trimmed, content, cursor)
}

fn execute_simple_command(cmd: &str, content: &mut String, cursor: usize) -> VimCommandResult {
    let trimmed = cmd;

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

/// Parse optional `start,end` prefix (1-based line numbers).
fn parse_line_range(cmd: &str) -> (Option<usize>, Option<usize>, &str) {
    let bytes = cmd.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return (None, None, cmd);
    }
    let start: usize = cmd[..i].parse().unwrap_or(0);
    if i < bytes.len() && bytes[i] == b',' {
        let after = &cmd[i + 1..];
        let end_len = after.chars().take_while(|c| c.is_ascii_digit()).count();
        if end_len == 0 {
            return (None, None, cmd);
        }
        let end: usize = after[..end_len].parse().unwrap_or(0);
        let rest = after[end_len..].trim_start();
        return (Some(start), Some(end), rest);
    }
    (Some(start), None, &cmd[i..])
}

fn execute_ranged_command(
    range_start: Option<usize>,
    range_end: Option<usize>,
    rest: &str,
    content: &mut String,
    cursor: usize,
) -> Option<VimCommandResult> {
    let (start, end) = match (range_start, range_end) {
        (Some(s), Some(e)) => (s, e),
        (Some(n), None) if rest.is_empty() => {
            return Some(VimCommandResult {
                status: format!("Line {n}"),
                cursor: Some(char_index_at_line(content, n.saturating_sub(1))),
                content_changed: false,
                request_save: false,
                line_numbers: None,
            });
        }
        (Some(n), None) if rest == "d" || rest == "delete" => {
            let (new_cursor, deleted) = delete_line_range(content, n, n);
            return Some(VimCommandResult {
                status: format!("{deleted} line(s) deleted"),
                cursor: Some(new_cursor.min(content.chars().count())),
                content_changed: deleted > 0,
                request_save: false,
                line_numbers: None,
            });
        }
        _ => return None,
    };

    let cmd = rest.trim();
    if cmd == "d" || cmd == "delete" {
        let (new_cursor, deleted) = delete_line_range(content, start, end);
        return Some(VimCommandResult {
            status: format!("{deleted} line(s) deleted"),
            cursor: Some(new_cursor.min(content.chars().count())),
            content_changed: deleted > 0,
            request_save: false,
            line_numbers: None,
        });
    }

    if cmd.is_empty() {
        return Some(VimCommandResult {
            status: format!("Lines {start}-{end}"),
            cursor: Some(char_index_at_line(content, start.saturating_sub(1))),
            content_changed: false,
            request_save: false,
            line_numbers: None,
        });
    }

    if let Some(result) = parse_substitute(cmd, content, cursor) {
        return Some(result);
    }

    None
}

fn char_index_at_line(content: &str, line: usize) -> usize {
    let mut current = 0usize;
    let mut pos = 0usize;
    for (i, ch) in content.chars().enumerate() {
        if current == line {
            return pos;
        }
        if ch == '\n' {
            current += 1;
            pos = i + 1;
        }
    }
    if current == line {
        pos
    } else {
        content.chars().count()
    }
}

fn delete_line_range(content: &mut String, start_line: usize, end_line: usize) -> (usize, usize) {
    let total = line_count(content);
    let start = start_line.saturating_sub(1).min(total.saturating_sub(1));
    let end = end_line.saturating_sub(1).min(total.saturating_sub(1));
    let lo = start.min(end);
    let hi = start.max(end);
    let count = hi - lo + 1;
    let cursor = char_index_at_line(content, lo);
    for _ in 0..count {
        if line_count(content) <= 1 && lo == 0 {
            content.clear();
            break;
        }
        delete_line_at(content, lo);
    }
    (cursor, count)
}

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        1
    } else {
        content.chars().filter(|&c| c == '\n').count() + 1
    }
}

fn delete_line_at(content: &mut String, line: usize) -> usize {
    let (start, end) = line_col_range(content, line);
    let delete_end = if end < content.chars().count() {
        end + 1
    } else if start > 0 {
        start - 1
    } else {
        end
    };
    let (sb, eb) = char_range_to_bytes(content, start.min(delete_end), end.max(delete_end));
    let cursor = start.min(delete_end);
    content.replace_range(sb..eb, "");
    cursor
}

fn parse_global_command(cmd: &str, content: &mut String, cursor: usize) -> Option<VimCommandResult> {
    let invert = cmd.starts_with("v/") || cmd.starts_with("g!");
    let body = if let Some(rest) = cmd.strip_prefix("g!") {
        rest
    } else if let Some(rest) = cmd.strip_prefix("g") {
        rest
    } else if let Some(rest) = cmd.strip_prefix("v") {
        rest
    } else {
        return None;
    };

    let sep = body.chars().next()?;
    if sep != '/' && sep != '#' {
        return None;
    }
    let after_sep = &body[sep.len_utf8()..];
    let end = after_sep.find(sep)?;
    let pattern = &after_sep[..end];
    let action = after_sep[end + sep.len_utf8()..].trim_start();

    if action == "d" || action == "delete" {
        return Some(global_delete_lines(content, cursor, pattern, invert));
    }

    if let Some((old, new, global)) = parse_substitute_action(action) {
        return Some(global_substitute_lines(
            content, cursor, pattern, invert, &old, &new, global,
        ));
    }

    if let Some(norm_cmd) = action.strip_prefix("norm") {
        let norm_cmd = norm_cmd.trim_start();
        return Some(global_norm_lines(
            content,
            cursor,
            pattern,
            invert,
            norm_cmd,
        ));
    }

    Some(VimCommandResult {
        status: format!("Unsupported global action: {action}"),
        cursor: Some(cursor),
        content_changed: false,
        request_save: false,
        line_numbers: None,
    })
}

fn global_delete_lines(
    content: &mut String,
    cursor: usize,
    pattern: &str,
    invert: bool,
) -> VimCommandResult {
    let mut lines_to_delete = Vec::new();
    let total = line_count(content);
    for line in 0..total {
        if line_matches_pattern(content, line, pattern, invert) {
            lines_to_delete.push(line);
        }
    }

    let deleted = lines_to_delete.len();
    for line in lines_to_delete.into_iter().rev() {
        delete_line_at(content, line);
    }

    VimCommandResult {
        status: format!("{deleted} line(s) deleted"),
        cursor: Some(cursor.min(content.chars().count())),
        content_changed: deleted > 0,
        request_save: false,
        line_numbers: None,
    }
}

fn global_norm_lines(
    content: &mut String,
    cursor: usize,
    line_pattern: &str,
    invert: bool,
    norm_cmd: &str,
) -> VimCommandResult {
    let mut lines = Vec::new();
    let total = line_count(content);
    for line in 0..total {
        if line_matches_pattern(content, line, line_pattern, invert) {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        return VimCommandResult {
            status: "No matching lines".to_string(),
            cursor: Some(cursor),
            content_changed: false,
            request_save: false,
            line_numbers: None,
        };
    }

    let mut applied = 0usize;
    if norm_cmd.trim() == "dd" {
        for line in lines.into_iter().rev() {
            delete_line_at(content, line);
            applied += 1;
        }
    } else {
        for line in lines.into_iter().rev() {
            if apply_norm_at_line(content, line, norm_cmd) {
                applied += 1;
            }
        }
    }

    VimCommandResult {
        status: format!("norm on {applied} line(s)"),
        cursor: Some(cursor.min(content.chars().count())),
        content_changed: applied > 0,
        request_save: false,
        line_numbers: None,
    }
}

fn global_substitute_lines(
    content: &mut String,
    cursor: usize,
    line_pattern: &str,
    invert: bool,
    old: &str,
    new: &str,
    global: bool,
) -> VimCommandResult {
    let total = line_count(content);
    let mut total_subs = 0usize;
    for line in 0..total {
        if !line_matches_pattern(content, line, line_pattern, invert) {
            continue;
        }
        let (start, end_idx) = line_col_range(content, line);
        let line_text: String = content.chars().skip(start).take(end_idx - start).collect();
        let (new_line, count) = substitute_in_line(&line_text, old, new, global);
        if count > 0 {
            let (sb, eb) = char_range_to_bytes(content, start, end_idx);
            content.replace_range(sb..eb, &new_line);
            total_subs += count;
        }
    }

    VimCommandResult {
        status: format!("{total_subs} substitution(s) on matching lines"),
        cursor: Some(cursor.min(content.chars().count())),
        content_changed: total_subs > 0,
        request_save: false,
        line_numbers: None,
    }
}

fn line_matches_pattern(content: &str, line: usize, pattern: &str, invert: bool) -> bool {
    let (start, end_idx) = line_col_range(content, line);
    let line_text: String = content.chars().skip(start).take(end_idx - start).collect();
    line_text.contains(pattern) != invert
}

fn parse_substitute_action(action: &str) -> Option<(String, String, bool)> {
    let rest = action.strip_prefix('s')?;
    let sep = rest.chars().next()?;
    if sep != '/' && sep != '#' {
        return None;
    }
    let body = &rest[sep.len_utf8()..];
    let parts: Vec<&str> = body.split(sep).collect();
    if parts.len() < 2 {
        return None;
    }
    let global = parts.get(2).is_some_and(|f| f.contains('g'));
    Some((parts[0].to_string(), parts[1].to_string(), global))
}

fn substitute_in_line(line: &str, old: &str, new: &str, global: bool) -> (String, usize) {
    if old.is_empty() {
        return (line.to_string(), 0);
    }
    if global {
        let count = line.matches(old).count();
        (line.replace(old, new), count)
    } else if let Some(replaced) = replace_first_literal(line, old, new) {
        (replaced, 1)
    } else {
        (line.to_string(), 0)
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

    #[test]
    fn deletes_line_range() {
        let mut text = "a\nb\nc\nd".to_string();
        let r = execute_vim_command("2,3d", &mut text, 0);
        assert!(r.content_changed);
        assert_eq!(text, "a\nd");
    }

    #[test]
    fn global_delete_matching_lines() {
        let mut text = "keep\nTODO fix\nkeep\nTODO bar".to_string();
        let r = execute_vim_command("g/TODO/d", &mut text, 0);
        assert!(r.content_changed);
        assert_eq!(text, "keep\nkeep");
    }

    #[test]
    fn inverse_global_delete() {
        let mut text = "a\nbb\nccc".to_string();
        let r = execute_vim_command("v/bb/d", &mut text, 0);
        assert!(r.content_changed);
        assert_eq!(text, "bb");
    }

    #[test]
    fn global_substitute_on_matching_lines() {
        let mut text = "foo bar\nbaz qux\nfoo again".to_string();
        let r = execute_vim_command("g/foo/s/foo/egg/g", &mut text, 0);
        assert!(r.content_changed);
        assert_eq!(text, "egg bar\nbaz qux\negg again");
    }

    #[test]
    fn insert_block_column_on_lines() {
        let mut text = "ab\nef\nij".to_string();
        let rect = BlockRect::from_positions(
            BlockPos { line: 0, col: 1 },
            BlockPos { line: 2, col: 1 },
        );
        insert_block_column(&mut text, rect, 1, "X");
        assert_eq!(text, "aXb\neXf\niXj");
    }

    #[test]
    fn global_norm_inserts_prefix() {
        let mut text = "foo\nbar\nfoo".to_string();
        let r = execute_vim_command("g/foo/norm I- ", &mut text, 0);
        assert!(r.content_changed);
        assert_eq!(text, "- foo\nbar\n- foo");
    }
}
