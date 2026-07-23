use eframe::egui::{self, Key, Modifiers};
use std::collections::HashMap;

use crate::vim_ex::{self, BlockPos, BlockRect, RegisterFile, VimCommandResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
pub enum KeybindingMode {
    #[default]
    Standard,
    Vim,
    Emacs,
}

impl KeybindingMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Vim => "Vim",
            Self::Emacs => "Emacs",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VimMode {
    #[default]
    Normal,
    Insert,
    Visual,
    VisualBlock,
    Command,
}

impl VimMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::VisualBlock => "VISUAL BLOCK",
            Self::Command => "COMMAND",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Pending {
    #[default]
    None,
    Operator(char),
    Replace,
    FindForward,
    FindBackward,
    TillForward,
    TillBackward,
    RegisterSelect,
}

#[derive(Clone, Debug)]
enum Repeatable {
    DeleteLines(usize),
    DeleteChars(usize),
    DeleteWord(usize),
    DeleteToEol,
    ChangeLine,
    ChangeWord,
    ReplaceChar(char),
    IndentLines(usize, bool),
    YankLines(usize),
}

#[derive(Clone, Default)]
pub struct KeybindingState {
    pub vim_mode: VimMode,
    pending: Pending,
    pub count: usize,
    pub registers: RegisterFile,
    pub kill_ring: Vec<String>,
    last_find: Option<(char, bool, bool)>,
    repeatable: Option<Repeatable>,
    pub macro_recording: Option<char>,
    pub macros: HashMap<char, Vec<String>>,
    pub last_macro: Option<char>,
    emacs_prefix: Option<usize>,
    pub command_buffer: String,
    pub block_anchor: Option<BlockPos>,
    pub block_head: Option<BlockPos>,
    pub active_block: Option<BlockRect>,
    pub use_system_clipboard: bool,
    /// Visual Block insert: `(rect, insert_column)`.
    pub block_insert: Option<(BlockRect, usize)>,
    pub emacs_mark: Option<usize>,
    pub emacs_isearch: Option<EmacsIsearch>,
}

#[derive(Clone, Debug, Default)]
pub struct EmacsIsearch {
    pub query: String,
    pub forward: bool,
    pub anchor: usize,
}

impl KeybindingState {
    pub fn yank_text(&self) -> &str {
        &self.registers.unnamed
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyAction {
    pub content_changed: bool,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    pub block_selection: Option<BlockRect>,
    pub vim_mode: Option<VimMode>,
    pub status: Option<String>,
    pub command_result: Option<VimCommandResult>,
}

impl KeyAction {
    fn cursor_only(cursor: usize) -> Self {
        Self {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            status: None,
            command_result: None,
        }
    }

    fn with_selection(cursor: usize, selection: (usize, usize)) -> Self {
        Self {
            content_changed: false,
            cursor,
            selection: Some(selection),
            block_selection: None,
            vim_mode: None,
            status: None,
            command_result: None,
        }
    }

    fn changed(cursor: usize) -> Self {
        Self {
            content_changed: true,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            status: None,
            command_result: None,
        }
    }
}

pub fn line_index(content: &str, pos: usize) -> usize {
    content.chars().take(pos).filter(|&c| c == '\n').count()
}

pub fn line_start(content: &str, pos: usize) -> usize {
    let pos = pos.min(content.chars().count());
    let mut start = 0usize;
    for (i, c) in content.chars().enumerate().take(pos) {
        if c == '\n' {
            start = i + 1;
        }
    }
    start
}

pub fn line_end_char(content: &str, pos: usize) -> usize {
    let pos = pos.min(content.chars().count());
    content
        .chars()
        .skip(pos)
        .position(|c| c == '\n')
        .map(|i| pos + i)
        .unwrap_or_else(|| content.chars().count())
}

pub fn first_nonblank(content: &str, pos: usize) -> usize {
    let start = line_start(content, pos);
    let end = line_end_char(content, pos);
    for (i, c) in content.chars().enumerate().skip(start).take(end.saturating_sub(start)) {
        if !c.is_whitespace() {
            return i;
        }
    }
    start
}

pub fn move_left(content: &str, pos: usize) -> usize {
    pos.saturating_sub(1)
}

pub fn move_right(content: &str, pos: usize) -> usize {
    (pos + 1).min(content.chars().count())
}

pub fn move_up(content: &str, pos: usize) -> usize {
    let start = line_start(content, pos);
    if start == 0 {
        return 0;
    }
    let prev_start = line_start(content, start.saturating_sub(1));
    let col = pos - start;
    (prev_start + col).min(line_end_char(content, prev_start))
}

pub fn move_down(content: &str, pos: usize) -> usize {
    let start = line_start(content, pos);
    let end = line_end_char(content, pos);
    if end >= content.chars().count() {
        return pos;
    }
    let next_start = end + 1;
    let col = pos - start;
    (next_start + col).min(line_end_char(content, next_start))
}

pub fn word_forward(content: &str, pos: usize) -> usize {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    if pos >= len {
        return len;
    }
    let mut i = pos;
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }
    while i < len && !chars[i].is_whitespace() {
        i += 1;
    }
    i
}

pub fn word_backward(content: &str, pos: usize) -> usize {
    let chars: Vec<char> = content.chars().collect();
    if pos == 0 {
        return 0;
    }
    let mut i = pos.saturating_sub(1);
    while i > 0 && chars[i].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

pub fn word_end(content: &str, pos: usize) -> usize {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    if pos >= len.saturating_sub(1) {
        return len.saturating_sub(1);
    }
    let mut i = if pos < len && !chars[pos].is_whitespace() {
        pos
    } else {
        pos + 1
    };
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }
    while i + 1 < len && !chars[i + 1].is_whitespace() {
        i += 1;
    }
    i.min(len.saturating_sub(1))
}

pub fn line_text(content: &str, line_idx: usize) -> (usize, usize, String) {
    let mut current = 0usize;
    for (idx, line) in content.split_inclusive('\n').enumerate() {
        let char_len = line.chars().count();
        if idx == line_idx {
            let body = line.strip_suffix('\n').unwrap_or(line);
            return (current, current + body.chars().count(), body.to_string());
        }
        current += char_len;
    }
    (current, current, String::new())
}

pub fn delete_line(content: &mut String, line_idx: usize) -> usize {
    let line_count = content.chars().filter(|&c| c == '\n').count() + 1;
    if line_idx >= line_count {
        return content.chars().count();
    }
    let (start, end, _) = line_text(content, line_idx);
    let delete_end = if end < content.chars().count() {
        end + 1
    } else {
        end
    };
    let (start_b, end_b) = char_range_to_bytes(content, start, delete_end);
    content.replace_range(start_b..end_b, "");
    start.min(content.chars().count())
}

pub fn insert_newline(content: &mut String, pos: usize, before: bool) -> usize {
    let line = line_index(content, pos);
    let insert_at = if before {
        let (start, _, _) = line_text(content, line);
        start
    } else {
        let (_, end, _) = line_text(content, line);
        if end < content.chars().count() {
            end + 1
        } else {
            end
        }
    };
    let byte = char_index_to_byte(content, insert_at);
    content.insert(byte, '\n');
    insert_at
}

pub fn delete_char_at(content: &mut String, pos: usize) -> usize {
    if pos >= content.chars().count() {
        return pos;
    }
    let byte = char_index_to_byte(content, pos);
    let len = content[byte..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
    content.replace_range(byte..byte + len, "");
    pos
}

pub fn kill_region(content: &mut String, start: usize, end: usize) -> String {
    let (a, b) = if start <= end { (start, end) } else { (end, start) };
    let (start_b, end_b) = char_range_to_bytes(content, a, b);
    let killed = content[start_b..end_b].to_string();
    content.replace_range(start_b..end_b, "");
    killed
}

pub fn insert_text(content: &mut String, pos: usize, text: &str) -> usize {
    let byte = char_index_to_byte(content, pos);
    content.insert_str(byte, text);
    pos + text.chars().count()
}

pub fn paste_line_below(content: &mut String, line_idx: usize, text: &str) -> usize {
    let mut line = text.to_string();
    if !line.ends_with('\n') {
        line.push('\n');
    }
    let (_, end, _) = line_text(content, line_idx);
    let insert_at = if end < content.chars().count() {
        end + 1
    } else {
        end
    };
    insert_text(content, insert_at, &line);
    insert_at + line.chars().count().saturating_sub(1)
}

fn char_index_to_byte(content: &str, char_idx: usize) -> usize {
    content
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(content.len())
}

fn char_range_to_bytes(content: &str, start: usize, end: usize) -> (usize, usize) {
    (
        char_index_to_byte(content, start),
        char_index_to_byte(content, end),
    )
}

fn selection_range(cursor: usize, selection: Option<(usize, usize)>) -> (usize, usize) {
    selection.unwrap_or((cursor, cursor))
}

fn clear_block_state(state: &mut KeybindingState) {
    state.block_anchor = None;
    state.block_head = None;
    state.active_block = None;
    state.block_insert = None;
}

fn join_line(content: &mut String, cursor: usize) -> usize {
    let end = line_end_char(content, cursor);
    if end >= content.chars().count() {
        return cursor;
    }
    delete_char_at(content, end);
    let prev = if end > 0 {
        content.chars().nth(end - 1)
    } else {
        None
    };
    let next = content.chars().nth(end);
    if prev != Some(' ') && next != Some(' ') && next.is_some() {
        return insert_text(content, end, " ");
    }
    cursor.min(end)
}

fn open_line(content: &mut String, cursor: usize) -> usize {
    let pos = line_end_char(content, cursor);
    insert_text(content, pos, "\n");
    cursor
}

fn transpose_chars(content: &mut String, cursor: usize) -> usize {
    if cursor < 2 {
        return cursor;
    }
    let a = cursor - 2;
    let b = cursor - 1;
    let chars: Vec<char> = content.chars().collect();
    if a >= chars.len() || b >= chars.len() {
        return cursor;
    }
    let mut swapped = chars;
    swapped.swap(a, b);
    *content = swapped.into_iter().collect();
    cursor
}

fn chars_match_at(content: &str, start: usize, query: &str) -> bool {
    content
        .chars()
        .skip(start)
        .take(query.chars().count())
        .eq(query.chars())
}

fn isearch_find(content: &str, query: &str, from: usize, forward: bool) -> Option<usize> {
    if query.is_empty() {
        return None;
    }
    let q_len = query.chars().count();
    let total = content.chars().count();
    if q_len == 0 || total < q_len {
        return None;
    }
    let max_start = total.saturating_sub(q_len);
    if forward {
        for i in from..=max_start {
            if chars_match_at(content, i, query) {
                return Some(i);
            }
        }
        for i in 0..from.min(max_start + 1) {
            if chars_match_at(content, i, query) {
                return Some(i);
            }
        }
    } else {
        for i in (0..=from.min(max_start)).rev() {
            if chars_match_at(content, i, query) {
                return Some(i);
            }
        }
        for i in (from..=max_start).rev() {
            if chars_match_at(content, i, query) {
                return Some(i);
            }
        }
    }
    None
}

fn isearch_status(query: &str, forward: bool) -> String {
    let dir = if forward { "I-search" } else { "I-search backward" };
    format!("{dir}: {query}")
}

fn isearch_selection(content: &str, cursor: usize, query: &str) -> Option<(usize, usize)> {
    if query.is_empty() {
        return None;
    }
    if chars_match_at(content, cursor, query) {
        Some((cursor, cursor + query.chars().count()))
    } else {
        None
    }
}

fn isearch_action(content: &str, cursor: usize, query: &str, forward: bool, status: Option<String>) -> KeyAction {
    KeyAction {
        content_changed: false,
        cursor,
        selection: isearch_selection(content, cursor, query),
        block_selection: None,
        vim_mode: None,
        status: status.or_else(|| Some(isearch_status(query, forward))),
        command_result: None,
    }
}

pub fn isearch_all_matches(content: &str, query: &str) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    let q_len = query.chars().count();
    let total = content.chars().count();
    if q_len == 0 || total < q_len {
        return Vec::new();
    }
    let max_start = total.saturating_sub(q_len);
    let mut matches = Vec::new();
    for i in 0..=max_start {
        if chars_match_at(content, i, query) {
            matches.push((i, i + q_len));
        }
    }
    matches
}

pub fn handle_emacs_isearch(
    content: &str,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
    text: Option<&str>,
) -> Option<KeyAction> {
    let ctrl = modifiers.ctrl || modifiers.command;

    if let Some(text) = text.filter(|t| !t.is_empty()) {
        if state.emacs_isearch.is_some() {
            let isearch = state.emacs_isearch.get_or_insert(EmacsIsearch {
                query: String::new(),
                forward: true,
                anchor: cursor,
            });
            isearch.query.push_str(text);
            let forward = isearch.forward;
            let query = isearch.query.clone();
            let anchor = isearch.anchor;
            let from = if forward { anchor } else { cursor };
            let new_cursor = isearch_find(content, &query, from, forward).unwrap_or(cursor);
            return Some(isearch_action(content, new_cursor, &query, forward, None));
        }
        return None;
    }

    if let Some(mut isearch) = state.emacs_isearch.take() {
        match key {
            Key::Escape => {
                state.emacs_isearch = None;
                return Some(KeyAction {
                    content_changed: false,
                    cursor: isearch.anchor,
                    selection: None,
                    block_selection: None,
                    vim_mode: None,
                    status: Some("Quit".to_string()),
                    command_result: None,
                });
            }
            Key::G if ctrl => {
                state.emacs_isearch = None;
                return Some(KeyAction {
                    content_changed: false,
                    cursor: isearch.anchor,
                    selection: None,
                    block_selection: None,
                    vim_mode: None,
                    status: Some("Quit".to_string()),
                    command_result: None,
                });
            }
            Key::Enter => {
                return Some(isearch_action(content, cursor, &isearch.query, isearch.forward, None));
            }
            Key::Backspace => {
                isearch.query.pop();
                if isearch.query.is_empty() {
                    let forward = isearch.forward;
                    let anchor = isearch.anchor;
                    state.emacs_isearch = Some(isearch);
                    return Some(isearch_action(content, anchor, "", forward, None));
                }
                let forward = isearch.forward;
                let query = isearch.query.clone();
                let anchor = isearch.anchor;
                let new_cursor =
                    isearch_find(content, &query, anchor, forward).unwrap_or(isearch.anchor);
                state.emacs_isearch = Some(isearch);
                return Some(isearch_action(content, new_cursor, &query, forward, None));
            }
            Key::S if ctrl => {
                isearch.forward = true;
                let query = isearch.query.clone();
                let from = cursor.saturating_add(1);
                let new_cursor = isearch_find(content, &query, from, true).unwrap_or(cursor);
                state.emacs_isearch = Some(isearch);
                return Some(isearch_action(content, new_cursor, &query, true, None));
            }
            Key::R if ctrl => {
                isearch.forward = false;
                let query = isearch.query.clone();
                let from = cursor.saturating_sub(1);
                let new_cursor = isearch_find(content, &query, from, false).unwrap_or(cursor);
                state.emacs_isearch = Some(isearch);
                return Some(isearch_action(content, new_cursor, &query, false, None));
            }
            _ => {
                state.emacs_isearch = Some(isearch);
                return None;
            }
        }
    }

    if ctrl && key == Key::S {
        state.emacs_isearch = Some(EmacsIsearch {
            query: String::new(),
            forward: true,
            anchor: cursor,
        });
        return Some(isearch_action(content, cursor, "", true, None));
    }

    if ctrl && key == Key::R {
        state.emacs_isearch = Some(EmacsIsearch {
            query: String::new(),
            forward: false,
            anchor: cursor,
        });
        return Some(isearch_action(content, cursor, "", false, None));
    }

    None
}

fn yank_into(state: &mut KeybindingState, text: String) {
    let reg = state.registers.take_pending().unwrap_or('"');
    state.registers.yank(Some(reg), text.clone());
    if state.use_system_clipboard {
        if reg == '"' || reg.is_ascii_lowercase() || reg == '+' || reg == '*' {
            state.registers.store_clipboard_register(&text);
            crate::clipboard::set_clipboard_text(&text);
        }
    }
}

fn paste_text(state: &KeybindingState) -> Option<String> {
    let reg = state.registers.pending.unwrap_or('"');
    if state.use_system_clipboard && (reg == '+' || reg == '*' || reg == '"') {
        if let Some(text) = crate::clipboard::clipboard_text() {
            return Some(text);
        }
    }
    state.registers.get(reg).map(|s| s.to_string())
}

fn take_count(state: &mut KeybindingState) -> usize {
    let n = if state.count == 0 { 1 } else { state.count };
    state.count = 0;
    n
}

fn repeat_n<F: Fn(usize) -> usize>(mut pos: usize, count: usize, f: F) -> usize {
    for _ in 0..count {
        pos = f(pos);
    }
    pos
}

fn delete_lines(content: &mut String, line: usize, count: usize) -> usize {
    let mut line_idx = line;
    let mut cursor = line_start(content, char_index_at_line(content, line_idx));
    for _ in 0..count {
        if line_idx >= line_count(content) {
            break;
        }
        cursor = delete_line(content, line_idx);
    }
    cursor
}

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        1
    } else {
        content.chars().filter(|&c| c == '\n').count() + 1
    }
}

fn char_index_at_line(content: &str, line: usize) -> usize {
    let (start, _, _) = line_text(content, line);
    start
}

fn indent_line(content: &mut String, line: usize, spaces: &str) {
    let (start, _, _) = line_text(content, line);
    insert_text(content, start, spaces);
}

fn unindent_line(content: &mut String, line: usize) -> bool {
    let (start, end, text) = line_text(content, line);
    let stripped = text.strip_prefix("    ").or_else(|| text.strip_prefix('\t'));
    if let Some(rest) = stripped {
        let remove_len = text.len() - rest.len();
        let (start_b, end_b) = char_range_to_bytes(content, start, start + remove_len);
        content.replace_range(start_b..end_b, "");
        true
    } else {
        let _ = (start, end);
        false
    }
}

fn find_on_line(content: &str, pos: usize, ch: char, forward: bool, till: bool) -> Option<usize> {
    let start = line_start(content, pos);
    let end = line_end_char(content, pos);
    let slice: Vec<(usize, char)> = content
        .chars()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect();
    if forward {
        for (i, c) in slice.iter().skip(pos.saturating_sub(start)) {
            if *c == ch {
                let target = if till { i.saturating_sub(1) } else { *i };
                return Some(target.max(start));
            }
        }
    } else {
        for (i, c) in slice.iter().take(pos.saturating_sub(start) + 1).rev() {
            if *c == ch {
                let target = if till {
                    (*i + 1).min(end)
                } else {
                    *i
                };
                return Some(target);
            }
        }
    }
    None
}

fn key_char(key: Key) -> Option<char> {
    match key {
        Key::A => Some('a'),
        Key::B => Some('b'),
        Key::C => Some('c'),
        Key::D => Some('d'),
        Key::E => Some('e'),
        Key::F => Some('f'),
        Key::G => Some('g'),
        Key::H => Some('h'),
        Key::I => Some('i'),
        Key::J => Some('j'),
        Key::K => Some('k'),
        Key::L => Some('l'),
        Key::M => Some('m'),
        Key::N => Some('n'),
        Key::O => Some('o'),
        Key::P => Some('p'),
        Key::Q => Some('q'),
        Key::R => Some('r'),
        Key::S => Some('s'),
        Key::T => Some('t'),
        Key::U => Some('u'),
        Key::V => Some('v'),
        Key::W => Some('w'),
        Key::X => Some('x'),
        Key::Y => Some('y'),
        Key::Z => Some('z'),
        _ => None,
    }
}

fn key_digit(key: Key) -> Option<usize> {
    match key {
        Key::Num0 => Some(0),
        Key::Num1 => Some(1),
        Key::Num2 => Some(2),
        Key::Num3 => Some(3),
        Key::Num4 => Some(4),
        Key::Num5 => Some(5),
        Key::Num6 => Some(6),
        Key::Num7 => Some(7),
        Key::Num8 => Some(8),
        Key::Num9 => Some(9),
        _ => None,
    }
}

fn key_name(key: Key, modifiers: Modifiers) -> String {
    if let Some(d) = key_digit(key) {
        return d.to_string();
    }
    if let Some(c) = key_char(key) {
        let ch = if modifiers.shift {
            c.to_ascii_uppercase()
        } else {
            c
        };
        return ch.to_string();
    }
    match key {
        Key::Escape => "Escape".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::Semicolon => ";".to_string(),
        Key::Comma => ",".to_string(),
        _ => format!("{key:?}"),
    }
}

fn record_macro_key(state: &mut KeybindingState, key: Key, modifiers: Modifiers) {
    if let Some(reg) = state.macro_recording {
        let name = key_name(key, modifiers);
        state.macros.entry(reg).or_default().push(name);
    }
}

fn push_kill_ring(state: &mut KeybindingState, text: String) {
    if text.is_empty() {
        return;
    }
    state.kill_ring.push(text);
}

fn yank_kill_ring(state: &KeybindingState) -> Option<&str> {
    state.kill_ring.last().map(|s| s.as_str())
}

fn apply_repeatable(content: &mut String, state: &mut KeybindingState, cursor: usize) -> Option<KeyAction> {
    let rep = state.repeatable.clone()?;
    match rep {
        Repeatable::DeleteLines(n) => {
            let line = line_index(content, cursor);
            let new_cursor = delete_lines(content, line, n);
            Some(KeyAction::changed(new_cursor))
        }
        Repeatable::DeleteChars(n) => {
            let mut pos = cursor;
            for _ in 0..n {
                pos = delete_char_at(content, pos);
            }
            Some(KeyAction::changed(pos))
        }
        Repeatable::DeleteWord(n) => {
            let mut pos = cursor;
            for _ in 0..n {
                let end = word_forward(content, pos);
                kill_region(content, pos, end);
            }
            Some(KeyAction::changed(cursor))
        }
        Repeatable::DeleteToEol => {
            let end = line_end_char(content, cursor);
            yank_into(state, kill_region(content, cursor, end));
            Some(KeyAction::changed(cursor))
        }
        Repeatable::ChangeLine => {
            let line = line_index(content, cursor);
            let (_, _, text) = line_text(content, line);
            yank_into(state, text);
            let new_cursor = delete_line(content, line);
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Repeatable::ChangeWord => {
            let end = word_forward(content, cursor);
            yank_into(state, kill_region(content, cursor, end));
            Some(KeyAction {
                content_changed: true,
                cursor,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Repeatable::ReplaceChar(ch) => {
            if cursor < content.chars().count() {
                delete_char_at(content, cursor);
            }
            let new_cursor = insert_text(content, cursor, &ch.to_string());
            Some(KeyAction::changed(new_cursor.saturating_sub(1)))
        }
        Repeatable::IndentLines(n, increase) => {
            let line = line_index(content, cursor);
            for i in 0..n {
                if increase {
                    indent_line(content, line + i, "    ");
                } else {
                    let _ = unindent_line(content, line + i);
                }
            }
            Some(KeyAction::changed(cursor))
        }
        Repeatable::YankLines(n) => {
            let line = line_index(content, cursor);
            let mut yanked = String::new();
            for i in 0..n {
                let (_, _, text) = line_text(content, line + i);
                yanked.push_str(&text);
                yanked.push('\n');
            }
            yank_into(state, yanked);
            Some(KeyAction::cursor_only(cursor))
        }
    }
}

fn replay_macro(
    content: &mut String,
    state: &mut KeybindingState,
    reg: char,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    let keys = state.macros.get(&reg)?.clone();
    if keys.is_empty() {
        return None;
    }
    state.last_macro = Some(reg);
    let mut pos = cursor;
    let mut sel = selection;
    let mut last_action = None;
    for key_str in keys {
        let (key, modifiers) = parse_stored_key(&key_str)?;
        if let Some(action) = handle_vim(content, state, key, modifiers, pos, sel) {
            pos = action.cursor;
            sel = action.selection;
            last_action = Some(action);
        }
    }
    last_action
}

fn replay_macro_at(
    content: &mut String,
    state: &mut KeybindingState,
    reg: char,
    cursor: usize,
) -> bool {
    let keys = match state.macros.get(&reg) {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return false,
    };
    state.last_macro = Some(reg);
    let mut pos = cursor;
    let mut changed = false;
    for key_str in keys {
        let Some((key, modifiers)) = parse_stored_key(&key_str) else {
            continue;
        };
        if let Some(action) = handle_vim(content, state, key, modifiers, pos, None) {
            if action.content_changed {
                changed = true;
            }
            pos = action.cursor;
        }
    }
    changed
}

fn parse_stored_key(s: &str) -> Option<(Key, Modifiers)> {
    if let Ok(n) = s.parse::<usize>() {
        return Some((match n {
            0 => Key::Num0,
            1 => Key::Num1,
            2 => Key::Num2,
            3 => Key::Num3,
            4 => Key::Num4,
            5 => Key::Num5,
            6 => Key::Num6,
            7 => Key::Num7,
            8 => Key::Num8,
            9 => Key::Num9,
            _ => return None,
        }, Modifiers::NONE));
    }
    if s.len() == 1 {
        let ch = s.chars().next()?;
        let key = key_char(match ch.to_ascii_lowercase() {
            'a' => Key::A,
            'b' => Key::B,
            'c' => Key::C,
            'd' => Key::D,
            'e' => Key::E,
            'f' => Key::F,
            'g' => Key::G,
            'h' => Key::H,
            'i' => Key::I,
            'j' => Key::J,
            'k' => Key::K,
            'l' => Key::L,
            'm' => Key::M,
            'n' => Key::N,
            'o' => Key::O,
            'p' => Key::P,
            'q' => Key::Q,
            'r' => Key::R,
            's' => Key::S,
            't' => Key::T,
            'u' => Key::U,
            'v' => Key::V,
            'w' => Key::W,
            'x' => Key::X,
            'y' => Key::Y,
            'z' => Key::Z,
            _ => return None,
        })?;
        let modifiers = if ch.is_ascii_uppercase() {
            Modifiers::SHIFT
        } else {
            Modifiers::NONE
        };
        return Some((match ch.to_ascii_lowercase() {
            'a' => Key::A,
            'b' => Key::B,
            'c' => Key::C,
            'd' => Key::D,
            'e' => Key::E,
            'f' => Key::F,
            'g' => Key::G,
            'h' => Key::H,
            'i' => Key::I,
            'j' => Key::J,
            'k' => Key::K,
            'l' => Key::L,
            'm' => Key::M,
            'n' => Key::N,
            'o' => Key::O,
            'p' => Key::P,
            'q' => Key::Q,
            'r' => Key::R,
            's' => Key::S,
            't' => Key::T,
            'u' => Key::U,
            'v' => Key::V,
            'w' => Key::W,
            'x' => Key::X,
            'y' => Key::Y,
            'z' => Key::Z,
            _ => return None,
        }, modifiers));
    }
    match s {
        "Escape" => Some((Key::Escape, Modifiers::NONE)),
        "Home" => Some((Key::Home, Modifiers::NONE)),
        "End" => Some((Key::End, Modifiers::NONE)),
        ";" => Some((Key::Semicolon, Modifiers::NONE)),
        "," => Some((Key::Comma, Modifiers::NONE)),
        _ => None,
    }
}

pub fn handle_vim(
    content: &mut String,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    if state.macro_recording.is_some() && key == Key::Q && modifiers.is_none() {
        let reg = state.macro_recording.take();
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            command_result: None,
            vim_mode: None,
            status: Some(format!("macro {} stopped", reg.unwrap_or('a'))),
        });
    }

    match state.vim_mode {
        VimMode::Insert => {
            if let Some((rect, col)) = state.block_insert {
                if key == Key::Escape {
                    clear_block_state(state);
                    state.pending = Pending::None;
                    state.count = 0;
                    return Some(KeyAction {
                        content_changed: false,
                        cursor,
                        selection: None,
                        block_selection: None,
                        command_result: None,
                        vim_mode: Some(VimMode::Normal),
                        status: Some("NORMAL".to_string()),
                    });
                }
                if key == Key::Backspace {
                    let new_col = col.saturating_sub(1);
                    let new_cursor = vim_ex::delete_block_column_char(content, rect, col);
                    state.block_insert = Some((rect, new_col));
                    return Some(KeyAction {
                        content_changed: true,
                        cursor: new_cursor,
                        selection: None,
                        block_selection: Some(rect),
                        vim_mode: Some(VimMode::Insert),
                        status: None,
                        command_result: None,
                    });
                }
                return None;
            }
            if key == Key::Escape {
                state.pending = Pending::None;
                state.count = 0;
                return Some(KeyAction {
                    content_changed: false,
                    cursor,
                    selection: None,
            block_selection: None,
            command_result: None,
                    vim_mode: Some(VimMode::Normal),
                    status: Some("NORMAL".to_string()),
                });
            }
            return None;
        }
        VimMode::Visual => {
            return handle_vim_visual(content, state, key, modifiers, cursor, selection);
        }
        VimMode::VisualBlock => {
            return handle_vim_visual_block(content, state, key, modifiers, cursor);
        }
        VimMode::Command => return None,
        VimMode::Normal => {}
    }

    if let Pending::RegisterSelect = state.pending {
        state.pending = Pending::None;
        if let Some(c) = key_char(key) {
            if c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '*' {
                state.registers.pending = Some(c);
                return Some(KeyAction::cursor_only(cursor));
            }
        }
    }

    if key == Key::Semicolon && modifiers.shift && state.pending == Pending::None {
        state.command_buffer.clear();
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: Some(VimMode::Command),
            status: Some("COMMAND".to_string()),
            command_result: None,
        });
    }

    if modifiers.ctrl && key == Key::V {
        let pos = vim_ex::pos_to_block_pos(content, cursor);
        state.block_anchor = Some(pos);
        state.block_head = Some(pos);
        let rect = BlockRect::from_positions(pos, pos);
        state.active_block = Some(rect);
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: Some(rect),
            vim_mode: Some(VimMode::VisualBlock),
            status: Some("VISUAL BLOCK".to_string()),
            command_result: None,
        });
    }

    // @macro replay (Shift+2 on US keyboard)
    if key == Key::Num2 && modifiers.shift && state.pending == Pending::None {
        state.pending = Pending::Operator('@');
        return Some(KeyAction::cursor_only(cursor));
    }
    if let Pending::Operator('@') = state.pending {
        state.pending = Pending::None;
        if key == Key::Num2 && modifiers.shift {
            if let Some(reg) = state.last_macro {
                return replay_macro(content, state, reg, cursor, selection);
            }
            return Some(KeyAction::cursor_only(cursor));
        }
        if let Some(c) = key_char(key) {
            if c.is_ascii_lowercase() {
                return replay_macro(content, state, c, cursor, selection);
            }
        }
    }

    // Repeat
    if key == Key::Q && modifiers.is_none() && state.pending == Pending::None && state.count == 0 {
        state.pending = Pending::Operator('q');
        return Some(KeyAction::cursor_only(cursor));
    }

    if let Pending::Operator('q') = state.pending {
        state.pending = Pending::None;
        if let Some(c) = key_char(key) {
            if c.is_ascii_lowercase() {
                state.macro_recording = Some(c);
                state.macros.insert(c, Vec::new());
                return Some(KeyAction {
                    content_changed: false,
                    cursor,
                    selection: None,
            block_selection: None,
            command_result: None,
                    vim_mode: None,
                    status: Some(format!("recording @{c}")),
                });
            }
        }
    }

    // Macro start: q{letter}
    if key == Key::Period && modifiers.is_none() {
        return apply_repeatable(content, state, cursor);
    }

    // Count prefix
    if modifiers.is_none() {
        if let Some(d) = key_digit(key) {
            if state.count == 0 && d == 0 && state.pending == Pending::None {
                return Some(KeyAction::cursor_only(line_start(content, cursor)));
            }
            state.count = state.count.saturating_mul(10).saturating_add(d).min(9999);
            return Some(KeyAction::cursor_only(cursor));
        }
    }

    let (sel_start, sel_end) = selection_range(cursor, selection);

    // Pending operator completions
    if let Pending::Operator(op) = state.pending {
        state.pending = Pending::None;
        let count = take_count(state);
        return match (op, key, modifiers) {
            ('d', Key::D, _) => {
                let line = line_index(content, cursor);
                let (_, _, text) = line_text(content, line);
                yank_into(state, text);
                let new_cursor = delete_lines(content, line, count);
                state.repeatable = Some(Repeatable::DeleteLines(count));
                Some(KeyAction::changed(new_cursor))
            }
            ('y', Key::Y, _) => {
                let line = line_index(content, cursor);
                let mut yanked = String::new();
                for i in 0..count {
                    let (_, _, text) = line_text(content, line + i);
                    yanked.push_str(&text);
                    yanked.push('\n');
                }
                yank_into(state, yanked);
                state.repeatable = Some(Repeatable::YankLines(count));
                Some(KeyAction::cursor_only(cursor))
            }
            ('c', Key::C, _) => {
                let line = line_index(content, cursor);
                for _ in 0..count {
                    let (_, _, text) = line_text(content, line);
                    yank_into(state, text);
                    delete_line(content, line);
                }
                state.repeatable = Some(Repeatable::ChangeLine);
                Some(KeyAction {
                    content_changed: true,
                    cursor: line_start(content, cursor),
                    selection: None,
            block_selection: None,
            command_result: None,
                    vim_mode: Some(VimMode::Insert),
                    status: Some("INSERT".to_string()),
                })
            }
            ('g', Key::G, _) => {
                if count <= 1 {
                    Some(KeyAction::cursor_only(0))
                } else {
                    Some(KeyAction::cursor_only(cursor))
                }
            }
            ('d', Key::W, _) => {
                let mut pos = cursor;
                for _ in 0..count {
                    let end = word_forward(content, pos);
                    yank_into(state, kill_region(content, pos, end));
                }
                state.repeatable = Some(Repeatable::DeleteWord(count));
                Some(KeyAction::changed(cursor))
            }
            ('c', Key::W, _) => {
                for _ in 0..count {
                    let end = word_forward(content, cursor);
                    yank_into(state, kill_region(content, cursor, end));
                }
                state.repeatable = Some(Repeatable::ChangeWord);
                Some(KeyAction {
                    content_changed: true,
                    cursor,
                    selection: None,
            block_selection: None,
            command_result: None,
                    vim_mode: Some(VimMode::Insert),
                    status: Some("INSERT".to_string()),
                })
            }
            ('d', Key::L, _) => {
                let end = line_end_char(content, cursor);
                yank_into(state, kill_region(content, cursor, end));
                state.repeatable = Some(Repeatable::DeleteToEol);
                Some(KeyAction::changed(cursor))
            }
            ('d', Key::Num4, m) if m.shift => {
                let end = line_end_char(content, cursor);
                yank_into(state, kill_region(content, cursor, end));
                state.repeatable = Some(Repeatable::DeleteToEol);
                Some(KeyAction::changed(cursor))
            }
            ('>', Key::Period, m) if m.shift => {
                let line = line_index(content, cursor);
                for i in 0..count {
                    indent_line(content, line + i, "    ");
                }
                state.repeatable = Some(Repeatable::IndentLines(count, true));
                Some(KeyAction::changed(cursor))
            }
            ('<', Key::Comma, m) if m.shift => {
                let line = line_index(content, cursor);
                for i in 0..count {
                    let _ = unindent_line(content, line + i);
                }
                state.repeatable = Some(Repeatable::IndentLines(count, false));
                Some(KeyAction::changed(cursor))
            }
            _ => None,
        };
    }

    if let Pending::Replace = state.pending {
        state.pending = Pending::None;
        if let Some(ch) = key_char(key) {
            let c = if modifiers.shift { ch.to_ascii_uppercase() } else { ch };
            if cursor < content.chars().count() {
                delete_char_at(content, cursor);
            }
            insert_text(content, cursor, &c.to_string());
            state.repeatable = Some(Repeatable::ReplaceChar(c));
            return Some(KeyAction::changed(cursor));
        }
    }

    for (pending, forward, till) in [
        (Pending::FindForward, true, false),
        (Pending::FindBackward, false, false),
        (Pending::TillForward, true, true),
        (Pending::TillBackward, false, true),
    ] {
        if state.pending == pending {
            state.pending = Pending::None;
            if let Some(ch) = key_char(key) {
                let c = if modifiers.shift { ch.to_ascii_uppercase() } else { ch };
                state.last_find = Some((c, forward, till));
                if let Some(pos) = find_on_line(content, cursor, c, forward, till) {
                    return Some(KeyAction::cursor_only(pos));
                }
            }
            return Some(KeyAction::cursor_only(cursor));
        }
    }

    // gg handled via g then g
    if let Pending::Operator('g') = state.pending {
        if key == Key::G {
            state.pending = Pending::None;
            take_count(state);
            return Some(KeyAction::cursor_only(0));
        }
        state.pending = Pending::None;
    }

    if !modifiers.alt && !modifiers.ctrl && !modifiers.command {
        // ok
    } else if key != Key::Escape && !(modifiers.shift && matches!(key, Key::I | Key::A | Key::O | Key::G)) {
        return None;
    }

    match key {
        Key::D => {
            state.pending = Pending::Operator('d');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Y => {
            state.pending = Pending::Operator('y');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::C => {
            state.pending = Pending::Operator('c');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::G if modifiers.shift => {
            take_count(state);
            Some(KeyAction::cursor_only(content.chars().count()))
        }
        Key::G => {
            state.pending = Pending::Operator('g');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::R => {
            state.pending = Pending::Replace;
            Some(KeyAction::cursor_only(cursor))
        }
        Key::F => {
            state.pending = Pending::FindForward;
            Some(KeyAction::cursor_only(cursor))
        }
        Key::T => {
            state.pending = Pending::TillForward;
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Semicolon => {
            if let Some((ch, forward, till)) = state.last_find {
                if let Some(pos) = find_on_line(content, cursor, ch, forward, till) {
                    return Some(KeyAction::cursor_only(pos));
                }
            }
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Comma if !modifiers.shift => {
            if let Some((ch, forward, till)) = state.last_find {
                if let Some(pos) = find_on_line(content, cursor, ch, !forward, till) {
                    return Some(KeyAction::cursor_only(pos));
                }
            }
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Period if modifiers.shift && state.pending == Pending::None => {
            state.pending = Pending::Operator('>');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Comma if modifiers.shift && state.pending == Pending::None => {
            state.pending = Pending::Operator('<');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Escape => {
            state.pending = Pending::None;
            state.count = 0;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
            })
        }
        Key::I if modifiers.shift => {
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor: line_start(content, cursor),
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::I => {
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::A if modifiers.shift => {
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor: line_end_char(content, cursor),
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::A => {
            state.pending = Pending::None;
            let pos = if cursor < content.chars().count() {
                move_right(content, cursor)
            } else {
                cursor
            };
            Some(KeyAction {
                content_changed: false,
                cursor: pos,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::O if modifiers.shift => {
            state.pending = Pending::None;
            let new_cursor = insert_newline(content, cursor, true);
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::O => {
            state.pending = Pending::None;
            let line = line_index(content, cursor);
            let (_, end, _) = line_text(content, line);
            let insert_at = if end < content.chars().count() {
                end + 1
            } else {
                end
            };
            let new_cursor = insert_text(content, insert_at, "\n");
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::V => {
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: Some((cursor, cursor)),
                block_selection: None,
                command_result: None,
                vim_mode: Some(VimMode::Visual),
                status: Some("VISUAL".to_string()),
            })
        }
        Key::H => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            move_left(content, p)
        }))),
        Key::L => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            move_right(content, p)
        }))),
        Key::K => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            move_up(content, p)
        }))),
        Key::J => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            move_down(content, p)
        }))),
        Key::W => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            word_forward(content, p)
        }))),
        Key::B => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            word_backward(content, p)
        }))),
        Key::E => Some(KeyAction::cursor_only(repeat_n(cursor, take_count(state), |p| {
            word_end(content, p)
        }))),
        Key::Num6 if modifiers.shift => Some(KeyAction::cursor_only(first_nonblank(content, cursor))),
        Key::Num4 if modifiers.shift => Some(KeyAction::cursor_only(line_end_char(content, cursor))),
        Key::X => {
            let count = take_count(state);
            let mut pos = cursor;
            for _ in 0..count {
                if pos >= content.chars().count() {
                    break;
                }
                pos = delete_char_at(content, pos);
            }
            state.repeatable = Some(Repeatable::DeleteChars(count));
            Some(KeyAction::changed(pos))
        }
        Key::P if modifiers.shift => {
            let Some(text) = paste_text(state) else {
                return Some(KeyAction::cursor_only(cursor));
            };
            if text.is_empty() {
                return Some(KeyAction::cursor_only(cursor));
            }
            let new_cursor = insert_text(content, cursor, &text);
            Some(KeyAction::changed(new_cursor))
        }
        Key::P => {
            let Some(text) = paste_text(state) else {
                return Some(KeyAction::cursor_only(cursor));
            };
            if text.is_empty() {
                return Some(KeyAction::cursor_only(cursor));
            }
            let line = line_index(content, cursor);
            let new_cursor = paste_line_below(content, line, &text);
            Some(KeyAction::changed(new_cursor))
        }
        Key::U => Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            command_result: None,
            vim_mode: None,
            status: Some("Use Ctrl+Z to undo".to_string()),
        }),
        Key::Home => Some(KeyAction::cursor_only(0)),
        Key::End => Some(KeyAction::cursor_only(content.chars().count())),
        _ => {
            state.pending = Pending::None;
            None
        }
    }
}

fn handle_vim_visual(
    content: &mut String,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    let (sel_start, sel_end) = selection_range(cursor, selection);
    let count = if state.count == 0 { 1 } else { state.count };
    state.count = 0;

    if key == Key::F && modifiers.shift {
        state.pending = Pending::FindBackward;
        return Some(KeyAction::cursor_only(sel_end));
    }

    match key {
        Key::H => Some(KeyAction::with_selection(
            sel_start,
            (sel_start, move_left(content, sel_end)),
        )),
        Key::L => Some(KeyAction::with_selection(
            sel_start,
            (sel_start, move_right(content, sel_end)),
        )),
        Key::K => Some(KeyAction::with_selection(
            sel_start,
            (sel_start, move_up(content, sel_end)),
        )),
        Key::J => Some(KeyAction::with_selection(
            sel_start,
            (sel_start, move_down(content, sel_end)),
        )),
        Key::Escape => {
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor: sel_start,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
            })
        }
        Key::Y => {
            let (a, b) = ordered(sel_start, sel_end);
            let (start_b, end_b) = char_range_to_bytes(content, a, b);
            yank_into(state, content[start_b..end_b].to_string());
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor: sel_start,
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
            })
        }
        Key::D | Key::X => {
            let (a, b) = ordered(sel_start, sel_end);
            yank_into(state, kill_region(content, a, b));
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: true,
                cursor: sel_start.min(content.chars().count()),
                selection: None,
            block_selection: None,
            command_result: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
            })
        }
        _ => None,
    }
}

fn handle_vim_visual_block(
    content: &mut String,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
) -> Option<KeyAction> {
    let anchor = state.block_anchor.unwrap_or_else(|| vim_ex::pos_to_block_pos(content, cursor));
    let mut head = state.block_head.unwrap_or(anchor);

    match key {
        Key::H => head.col = head.col.saturating_sub(1),
        Key::L => head.col += 1,
        Key::K if head.line > 0 => head.line -= 1,
        Key::J => head.line += 1,
        Key::Escape => {
            clear_block_state(state);
            return Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
                block_selection: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
                command_result: None,
            });
        }
        Key::Y => {
            let rect = BlockRect::from_positions(anchor, head);
            yank_into(state, vim_ex::yank_block(content, rect));
            clear_block_state(state);
            return Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
                block_selection: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
                command_result: None,
            });
        }
        Key::D | Key::X => {
            let rect = BlockRect::from_positions(anchor, head);
            yank_into(state, vim_ex::yank_block(content, rect));
            let new_cursor = vim_ex::delete_block(content, rect);
            clear_block_state(state);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
                command_result: None,
            });
        }
        Key::I => {
            let rect = BlockRect::from_positions(anchor, head);
            state.block_insert = Some((rect, rect.col_start));
            state.active_block = Some(rect);
            let new_cursor = vim_ex::block_pos_to_char_index(
                content,
                BlockPos {
                    line: rect.line_start,
                    col: rect.col_start,
                },
            );
            return Some(KeyAction {
                content_changed: false,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT (block)".to_string()),
                command_result: None,
            });
        }
        Key::A => {
            let rect = BlockRect::from_positions(anchor, head);
            state.block_insert = Some((rect, rect.col_end));
            state.active_block = Some(rect);
            let new_cursor = vim_ex::block_pos_to_char_index(
                content,
                BlockPos {
                    line: rect.line_start,
                    col: rect.col_end,
                },
            );
            return Some(KeyAction {
                content_changed: false,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT (block)".to_string()),
                command_result: None,
            });
        }
        Key::C => {
            let rect = BlockRect::from_positions(anchor, head);
            yank_into(state, vim_ex::yank_block(content, rect));
            let _ = vim_ex::delete_block(content, rect);
            let insert_rect = BlockRect {
                line_start: rect.line_start,
                line_end: rect.line_end,
                col_start: rect.col_start,
                col_end: rect.col_start,
            };
            state.block_insert = Some((insert_rect, rect.col_start));
            state.active_block = Some(insert_rect);
            let new_cursor = vim_ex::block_pos_to_char_index(
                content,
                BlockPos {
                    line: rect.line_start,
                    col: rect.col_start,
                },
            );
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(insert_rect),
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT (block)".to_string()),
                command_result: None,
            });
        }
        Key::Period if modifiers.shift => {
            let rect = BlockRect::from_positions(anchor, head);
            let new_cursor = vim_ex::indent_block_lines(content, rect);
            state.block_head = Some(head);
            state.active_block = Some(rect);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::VisualBlock),
                status: None,
                command_result: None,
            });
        }
        Key::Comma if modifiers.shift => {
            let rect = BlockRect::from_positions(anchor, head);
            let new_cursor = vim_ex::unindent_block_lines(content, rect);
            state.block_head = Some(head);
            state.active_block = Some(rect);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::VisualBlock),
                status: None,
                command_result: None,
            });
        }
        Key::P if !modifiers.shift => {
            let rect = BlockRect::from_positions(anchor, head);
            let Some(text) = paste_text(state) else {
                return None;
            };
            let new_cursor = vim_ex::paste_block_column(content, rect, &text, false);
            state.block_head = Some(head);
            state.active_block = Some(rect);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::VisualBlock),
                status: None,
                command_result: None,
            });
        }
        Key::P if modifiers.shift => {
            let rect = BlockRect::from_positions(anchor, head);
            let Some(text) = paste_text(state) else {
                return None;
            };
            let new_cursor = vim_ex::paste_block_column(content, rect, &text, true);
            state.block_head = Some(head);
            state.active_block = Some(rect);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::VisualBlock),
                status: None,
                command_result: None,
            });
        }
        Key::U if modifiers.shift => {
            let rect = BlockRect::from_positions(anchor, head);
            let new_cursor = vim_ex::transform_case_block(content, rect, true);
            state.block_head = Some(head);
            state.active_block = Some(rect);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::VisualBlock),
                status: None,
                command_result: None,
            });
        }
        Key::U => {
            let rect = BlockRect::from_positions(anchor, head);
            let new_cursor = vim_ex::transform_case_block(content, rect, false);
            state.block_head = Some(head);
            state.active_block = Some(rect);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                block_selection: Some(rect),
                vim_mode: Some(VimMode::VisualBlock),
                status: None,
                command_result: None,
            });
        }
        _ => {}
    }

    state.block_head = Some(head);
    let rect = BlockRect::from_positions(anchor, head);
    state.active_block = Some(rect);
    let cursor = vim_ex::block_pos_to_char_index(content, head);
    Some(KeyAction {
        content_changed: false,
        cursor,
        selection: None,
        block_selection: Some(rect),
        vim_mode: Some(VimMode::VisualBlock),
        status: None,
        command_result: None,
    })
}

fn handle_vim_visual_block_text(
    content: &mut String,
    state: &mut KeybindingState,
    text: &str,
    cursor: usize,
) -> Option<KeyAction> {
    if text != "~" {
        return None;
    }
    let anchor = state
        .block_anchor
        .unwrap_or_else(|| vim_ex::pos_to_block_pos(content, cursor));
    let head = state.block_head.unwrap_or(anchor);
    let rect = BlockRect::from_positions(anchor, head);
    let new_cursor = vim_ex::toggle_case_block(content, rect);
    state.block_head = Some(head);
    state.active_block = Some(rect);
    Some(KeyAction {
        content_changed: true,
        cursor: new_cursor,
        selection: None,
        block_selection: Some(rect),
        vim_mode: Some(VimMode::VisualBlock),
        status: None,
        command_result: None,
    })
}

pub fn handle_vim_text(state: &mut KeybindingState, text: &str, cursor: usize) -> Option<KeyAction> {
    if state.vim_mode != VimMode::Normal {
        return None;
    }
    if text == "\"" {
        state.pending = Pending::RegisterSelect;
        return Some(KeyAction::cursor_only(cursor));
    }
    None
}

pub fn handle_command_key(
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    content: &mut String,
    cursor: usize,
) -> Option<KeyAction> {
    if state.vim_mode != VimMode::Command {
        return None;
    }
    match key {
        Key::Escape => {
            state.command_buffer.clear();
            return Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
                block_selection: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
                command_result: None,
            });
        }
        Key::Backspace => {
            state.command_buffer.pop();
            return Some(KeyAction::cursor_only(cursor));
        }
        Key::Enter => {
            let cmd = state.command_buffer.clone();
            state.command_buffer.clear();
            let result = if cmd.trim() == "reg" || cmd.trim() == "registers" {
                VimCommandResult {
                    status: state.registers.format_all(),
                    cursor: None,
                    content_changed: false,
                    request_save: false,
                    line_numbers: None,
                }
            } else {
                let mut replay = |content: &mut String, pos: usize, reg: char| {
                    replay_macro_at(content, state, reg, pos)
                };
                let mut replay_slot =
                    Some(&mut replay as &mut dyn FnMut(&mut String, usize, char) -> bool);
                vim_ex::execute_vim_command_ctx(&cmd, content, cursor, &mut replay_slot)
            };
            let new_cursor = result.cursor.unwrap_or(cursor);
            return Some(KeyAction {
                content_changed: result.content_changed,
                cursor: new_cursor,
                selection: None,
                block_selection: None,
                vim_mode: Some(VimMode::Normal),
                status: Some(result.status.clone()),
                command_result: Some(result),
            });
        }
        _ => {
            if let Some(c) = key_char(key) {
                if modifiers.shift {
                    state.command_buffer.push(c.to_ascii_uppercase());
                } else {
                    state.command_buffer.push(c);
                }
                return Some(KeyAction::cursor_only(cursor));
            }
            if let Some(d) = key_digit(key) {
                state.command_buffer.push(char::from_digit(d as u32, 10).unwrap());
                return Some(KeyAction::cursor_only(cursor));
            }
        }
    }
    None
}

pub fn render_vim_command_bar(
    ui: &mut egui::Ui,
    state: &mut KeybindingState,
    content: &mut String,
    cursor: usize,
) -> Option<KeyAction> {
    if state.vim_mode != VimMode::Command {
        return None;
    }
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(":");
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.command_buffer)
                .desired_width(ui.available_width() - 8.0)
                .hint_text("w · q · 42 · 1,5d · g/pat/d · g/pat/s/o/n/g · g/pat/norm · set number · reg"),
        );
        response.request_focus();
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            action = handle_command_key(state, Key::Enter, Modifiers::NONE, content, cursor);
        }
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            action = handle_command_key(state, Key::Escape, Modifiers::NONE, content, cursor);
        }
    });
    action
}

fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

pub fn handle_emacs(
    content: &mut String,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    let ctrl = modifiers.ctrl || modifiers.command;
    let alt = modifiers.alt;
    let (sel_start, sel_end) = selection_range(cursor, selection);
    let has_selection = sel_start != sel_end;

    if ctrl && key == Key::U {
        state.emacs_prefix = Some(state.emacs_prefix.unwrap_or(0) * 4);
        return Some(KeyAction::cursor_only(cursor));
    }

    let prefix = state.emacs_prefix.take().unwrap_or(1).max(1);

    if alt {
        return match key {
            Key::B => Some(KeyAction::cursor_only(repeat_n(cursor, prefix, |p| {
                word_backward(content, p)
            }))),
            Key::F => Some(KeyAction::cursor_only(repeat_n(cursor, prefix, |p| {
                word_forward(content, p)
            }))),
            Key::Comma => Some(KeyAction::cursor_only(0)),
            Key::Period => Some(KeyAction::cursor_only(content.chars().count())),
            Key::D => {
                let end = word_forward(content, cursor);
                let killed = kill_region(content, cursor, end);
                push_kill_ring(state, killed);
                Some(KeyAction::changed(cursor))
            }
            Key::W => {
                let mark = state.emacs_mark.unwrap_or(cursor);
                let (a, b) = ordered(mark, cursor);
                if a != b {
                    let text = kill_region(content, a, b);
                    push_kill_ring(state, text);
                }
                Some(KeyAction {
                    content_changed: false,
                    cursor,
                    selection: None,
                    block_selection: None,
                    vim_mode: None,
                    status: Some("Region copied".to_string()),
                    command_result: None,
                })
            }
            _ => None,
        };
    }

    if !ctrl {
        return None;
    }

    if key == Key::G {
        state.emacs_prefix = None;
        state.emacs_mark = None;
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            status: Some("Quit".to_string()),
            command_result: None,
        });
    }

    if key == Key::Space {
        state.emacs_mark = Some(cursor);
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            status: Some("Mark set".to_string()),
            command_result: None,
        });
    }

    match key {
        Key::B => Some(KeyAction::cursor_only(repeat_n(cursor, prefix, |p| {
            move_left(content, p)
        }))),
        Key::F => Some(KeyAction::cursor_only(repeat_n(cursor, prefix, |p| {
            move_right(content, p)
        }))),
        Key::P => Some(KeyAction::cursor_only(repeat_n(cursor, prefix, |p| {
            move_up(content, p)
        }))),
        Key::N => Some(KeyAction::cursor_only(repeat_n(cursor, prefix, |p| {
            move_down(content, p)
        }))),
        Key::A => Some(KeyAction::cursor_only(line_start(content, cursor))),
        Key::E => Some(KeyAction::cursor_only(line_end_char(content, cursor))),
        Key::V => Some(KeyAction::cursor_only(
            repeat_n(cursor, prefix, |p| move_down(content, p)).min(content.chars().count()),
        )),
        Key::D => {
            let mut pos = cursor;
            for _ in 0..prefix {
                if pos >= content.chars().count() {
                    break;
                }
                pos = delete_char_at(content, pos);
            }
            Some(KeyAction::changed(pos))
        }
        Key::K => {
            let end = line_end_char(content, cursor);
            if cursor == end {
                return Some(KeyAction::cursor_only(cursor));
            }
            let killed = kill_region(content, cursor, end);
            push_kill_ring(state, killed);
            Some(KeyAction::changed(cursor))
        }
        Key::W => {
            if !has_selection {
                return None;
            }
            let killed = kill_region(content, sel_start, sel_end);
            push_kill_ring(state, killed);
            Some(KeyAction::changed(sel_start.min(content.chars().count())))
        }
        Key::Y => {
            if let Some(text) = yank_kill_ring(state) {
                let new_cursor = insert_text(content, cursor, text);
                return Some(KeyAction::changed(new_cursor));
            }
            Some(KeyAction::cursor_only(cursor))
        }
        Key::O => Some(KeyAction::changed(open_line(content, cursor))),
        Key::J => Some(KeyAction::changed(join_line(content, cursor))),
        Key::T => Some(KeyAction::changed(transpose_chars(content, cursor))),
        Key::Slash | Key::Backslash => Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            command_result: None,
            vim_mode: None,
            status: Some("Use Ctrl+Z to undo".to_string()),
        }),
        _ => None,
    }
}

pub fn process_egui_input(
    ctx: &egui::Context,
    content: &mut String,
    text_edit_id: egui::Id,
    mode: KeybindingMode,
    state: &mut KeybindingState,
) -> Option<KeyAction> {
    if mode == KeybindingMode::Standard {
        return None;
    }

    if state.vim_mode == VimMode::Command {
        return None;
    }

    if !ctx.memory(|m| m.has_focus(text_edit_id)) {
        return None;
    }

    let mut text_state = egui::text_edit::TextEditState::load(ctx, text_edit_id)?;
    let range = text_state.cursor.char_range()?;
    let cursor = range.primary.index;
    let selection = if range.secondary.index != range.primary.index {
        Some((range.secondary.index, range.primary.index))
    } else {
        None
    };

    let mut action = None;
    ctx.input_mut(|input| {
        let events: Vec<_> = input
            .events
            .iter()
            .filter_map(|event| {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    Some((*key, *modifiers))
                } else {
                    None
                }
            })
            .collect();
        for (key, modifiers) in events {
            let result = match mode {
                KeybindingMode::Vim => {
                    if state.vim_mode == VimMode::Insert {
                        if state.block_insert.is_some() || key == Key::Escape {
                            handle_vim(content, state, key, modifiers, cursor, selection)
                        } else {
                            None
                        }
                    } else {
                        handle_vim(content, state, key, modifiers, cursor, selection)
                    }
                }
                KeybindingMode::Emacs => handle_emacs_isearch(
                    content,
                    state,
                    key,
                    modifiers,
                    cursor,
                    None,
                )
                .or_else(|| handle_emacs(content, state, key, modifiers, cursor, selection)),
                KeybindingMode::Standard => None,
            };
            if let Some(act) = result {
                input.consume_key(modifiers, key);
                if state.macro_recording.is_some() && mode == KeybindingMode::Vim {
                    record_macro_key(state, key, modifiers);
                }
                action = Some(act);
                break;
            }
        }
        if action.is_none() && mode == KeybindingMode::Emacs && state.emacs_isearch.is_some() {
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    if let Some(act) =
                        handle_emacs_isearch(content, state, Key::A, Modifiers::NONE, cursor, Some(text))
                    {
                        action = Some(act);
                        break;
                    }
                }
            }
        }
        if action.is_none() && mode == KeybindingMode::Vim && state.vim_mode == VimMode::Insert {
            if let Some((rect, col)) = state.block_insert {
                for event in &input.events {
                    if let egui::Event::Text(text) = event {
                        if !text.is_empty() {
                            let new_cursor =
                                vim_ex::insert_block_column(content, rect, col, text);
                            state.block_insert =
                                Some((rect, col + text.chars().count()));
                            action = Some(KeyAction {
                                content_changed: true,
                                cursor: new_cursor,
                                selection: None,
                                block_selection: Some(rect),
                                vim_mode: Some(VimMode::Insert),
                                status: None,
                                command_result: None,
                            });
                            break;
                        }
                    }
                }
            }
        }
        if action.is_none() && mode == KeybindingMode::Vim && state.vim_mode == VimMode::VisualBlock {
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    if let Some(act) =
                        handle_vim_visual_block_text(content, state, text, cursor)
                    {
                        action = Some(act);
                        break;
                    }
                }
            }
        }
        if action.is_none() && mode == KeybindingMode::Vim && state.vim_mode == VimMode::Normal {
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    if let Some(act) = handle_vim_text(state, text, cursor) {
                        action = Some(act);
                        break;
                    }
                }
            }
        }
    });

    if let Some(act) = &action {
        if let Some(vim_mode) = act.vim_mode {
            state.vim_mode = vim_mode;
        }
        if let Some(result) = &act.command_result {
            if result.request_save {
                // handled by app
            }
        }
        let sel = act
            .selection
            .map(|(a, b)| {
                egui::text::CCursorRange::two(
                    egui::text::CCursor::new(a),
                    egui::text::CCursor::new(b),
                )
            })
            .unwrap_or_else(|| {
                egui::text::CCursorRange::one(egui::text::CCursor::new(act.cursor))
            });
        text_state.cursor.set_char_range(Some(sel));
        text_state.store(ctx, text_edit_id);
    }

    action
}

pub fn reset_for_mode(state: &mut KeybindingState, mode: KeybindingMode) {
    state.pending = Pending::None;
    state.count = 0;
    state.emacs_prefix = None;
    state.macro_recording = None;
    state.command_buffer.clear();
    state.block_anchor = None;
    state.block_head = None;
    state.active_block = None;
    state.block_insert = None;
    state.emacs_mark = None;
    state.emacs_isearch = None;
    state.vim_mode = match mode {
        KeybindingMode::Vim => VimMode::Normal,
        _ => VimMode::Insert,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_on_lines() {
        let text = "ab\ncd";
        assert_eq!(move_down(text, 1), 4);
        assert_eq!(move_up(text, 4), 1);
    }

    #[test]
    fn deletes_line() {
        let mut text = "a\nb\nc".to_string();
        delete_line(&mut text, 1);
        assert_eq!(text, "a\nc");
    }

    #[test]
    fn vim_dd_with_count() {
        let mut text = "a\nb\nc\n".to_string();
        let mut state = KeybindingState::default();
        handle_vim(&mut text, &mut state, Key::Num2, Modifiers::NONE, 0, None).unwrap();
        handle_vim(&mut text, &mut state, Key::D, Modifiers::NONE, 0, None).unwrap();
        handle_vim(&mut text, &mut state, Key::D, Modifiers::NONE, 0, None).unwrap();
        assert_eq!(text, "c\n");
    }

    #[test]
    fn vim_macro_record() {
        let mut text = "hi".to_string();
        let mut state = KeybindingState::default();
        handle_vim(&mut text, &mut state, Key::Q, Modifiers::NONE, 0, None).unwrap();
        handle_vim(&mut text, &mut state, Key::A, Modifiers::NONE, 0, None).unwrap();
        assert!(state.macro_recording == Some('a'));
    }
}
