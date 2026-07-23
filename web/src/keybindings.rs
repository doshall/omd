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
            Self::Standard => "标准",
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
    pub emacs_prefix: Option<usize>,
    pub command_buffer: String,
    pub block_anchor: Option<BlockPos>,
    pub block_head: Option<BlockPos>,
    pub active_block: Option<BlockRect>,
    pub use_system_clipboard: bool,
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
    pub consume: bool,
    pub hint: Option<String>,
    pub command_result: Option<VimCommandResult>,
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
    state.vim_mode = match mode {
        KeybindingMode::Vim => VimMode::Normal,
        _ => VimMode::Insert,
    };
}

pub fn handle_keydown(
    content: &mut String,
    state: &mut KeybindingState,
    mode: KeybindingMode,
    key: &str,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    if mode == KeybindingMode::Standard {
        return None;
    }
    let ctrl = ctrl || meta;
    let action = match mode {
        KeybindingMode::Vim => {
            if state.vim_mode == VimMode::Command {
                return handle_command_input(
                    state, key, shift, content, cursor,
                );
            }
            if state.vim_mode == VimMode::Insert && key != "Escape" {
                return None;
            }
            handle_vim(content, state, key, shift, alt, ctrl, cursor, selection)
        }
        KeybindingMode::Emacs => handle_emacs(content, state, key, ctrl, alt, cursor, selection),
        KeybindingMode::Standard => None,
    };
    if let Some(ref act) = action {
        if state.macro_recording.is_some() && mode == KeybindingMode::Vim && act.consume {
            record_macro_key(state, key, shift);
        }
    }
    action
}

fn handle_vim(
    content: &mut String,
    state: &mut KeybindingState,
    key: &str,
    shift: bool,
    alt: bool,
    ctrl: bool,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    if state.macro_recording.is_some() && key == "q" && !shift && !ctrl && !alt {
        let reg = state.macro_recording.take();
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            consume: true,
            hint: Some(format!("macro {} stopped", reg.unwrap_or('a'))),
            command_result: None,
        });
    }

    match state.vim_mode {
        VimMode::Insert => {
            if key == "Escape" {
                state.pending = Pending::None;
                state.count = 0;
                return Some(action_mode(cursor, VimMode::Normal));
            }
            return None;
        }
        VimMode::Visual => {
            return handle_vim_visual(content, state, key, shift, cursor, selection);
        }
        VimMode::VisualBlock => {
            return handle_vim_visual_block(content, state, key, cursor);
        }
        VimMode::Command => return None,
        VimMode::Normal => {}
    }

    if let Pending::RegisterSelect = state.pending {
        state.pending = Pending::None;
        if let Some(c) = key_char_lower(key) {
            if c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '*' {
                state.registers.pending = Some(c);
                return Some(unchanged(cursor));
            }
        }
    }

    if key == ":" && !ctrl && !alt && state.pending == Pending::None {
        state.command_buffer.clear();
        return Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: Some(VimMode::Command),
            consume: true,
            hint: None,
            command_result: None,
        });
    }

    if ctrl && key_matches(key, "v", false) {
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
            consume: true,
            hint: None,
            command_result: None,
        });
    }

    if key == "\"" && !ctrl && !alt && state.pending == Pending::None {
        state.pending = Pending::RegisterSelect;
        return Some(unchanged(cursor));
    }

    // @macro replay
    if key == "@" && !ctrl && !alt && state.pending == Pending::None {
        state.pending = Pending::Operator('@');
        return Some(unchanged(cursor));
    }
    if let Pending::Operator('@') = state.pending {
        state.pending = Pending::None;
        if key == "@" {
            if let Some(reg) = state.last_macro {
                return replay_macro(content, state, reg, cursor, selection);
            }
            return Some(unchanged(cursor));
        }
        if let Some(c) = key_char_lower(key) {
            if c.is_ascii_lowercase() {
                return replay_macro(content, state, c, cursor, selection);
            }
        }
    }

    // Macro start: q{letter}
    if key == "q" && !shift && !ctrl && !alt && state.pending == Pending::None && state.count == 0 {
        state.pending = Pending::Operator('q');
        return Some(unchanged(cursor));
    }
    if let Pending::Operator('q') = state.pending {
        state.pending = Pending::None;
        if let Some(c) = key_char_lower(key) {
            if c.is_ascii_lowercase() {
                state.macro_recording = Some(c);
                state.macros.insert(c, Vec::new());
                return Some(KeyAction {
                    content_changed: false,
                    cursor,
                    selection: None,
                    block_selection: None,
                    vim_mode: None,
                    consume: true,
                    hint: Some(format!("recording @{c}")),
                    command_result: None,
                });
            }
        }
    }

    // Repeat
    if key == "." && !shift && !ctrl && !alt {
        return apply_repeatable(content, state, cursor);
    }

    // Count prefix
    if !shift && !ctrl && !alt {
        if let Some(d) = key_digit(key) {
            if state.count == 0 && d == 0 && state.pending == Pending::None {
                return Some(unchanged(line_start(content, cursor)));
            }
            state.count = state.count.saturating_mul(10).saturating_add(d).min(9999);
            return Some(unchanged(cursor));
        }
    }

    // Pending operator completions
    if let Pending::Operator(op) = state.pending {
        state.pending = Pending::None;
        let count = take_count(state);
        return match op {
            'd' if key == "d" => {
                let line = line_index(content, cursor);
                let (_, _, text) = line_text(content, line);
                yank_into(state, text);
                let new_cursor = delete_lines(content, line, count);
                state.repeatable = Some(Repeatable::DeleteLines(count));
                Some(changed(new_cursor))
            }
            'y' if key == "y" => {
                let line = line_index(content, cursor);
                let mut yanked = String::new();
                for i in 0..count {
                    let (_, _, text) = line_text(content, line + i);
                    yanked.push_str(&text);
                    yanked.push('\n');
                }
                yank_into(state, yanked);
                state.repeatable = Some(Repeatable::YankLines(count));
                Some(unchanged(cursor))
            }
            'c' if key == "c" => {
                let line = line_index(content, cursor);
                for _ in 0..count {
                    let (_, _, text) = line_text(content, line);
                    yank_into(state, text);
                    delete_line(content, line);
                }
                state.repeatable = Some(Repeatable::ChangeLine);
                Some(changed_mode(line_start(content, cursor), VimMode::Insert))
            }
            'g' if key == "g" => {
                if count <= 1 {
                    Some(unchanged(0))
                } else {
                    Some(unchanged(cursor))
                }
            }
            'd' if key_matches(key, "w", shift) => {
                let pos = cursor;
                for _ in 0..count {
                    let end = word_forward(content, pos);
                    yank_into(state, kill_region(content, pos, end));
                }
                state.repeatable = Some(Repeatable::DeleteWord(count));
                Some(changed(cursor))
            }
            'c' if key_matches(key, "w", shift) => {
                for _ in 0..count {
                    let end = word_forward(content, cursor);
                    yank_into(state, kill_region(content, cursor, end));
                }
                state.repeatable = Some(Repeatable::ChangeWord);
                Some(changed_mode(cursor, VimMode::Insert))
            }
            'd' if key == "l" || key == "$" => {
                let end = line_end_char(content, cursor);
                yank_into(state, kill_region(content, cursor, end));
                state.repeatable = Some(Repeatable::DeleteToEol);
                Some(changed(cursor))
            }
            '>' if key == ">" => {
                let line = line_index(content, cursor);
                for i in 0..count {
                    indent_line(content, line + i, "    ");
                }
                state.repeatable = Some(Repeatable::IndentLines(count, true));
                Some(changed(cursor))
            }
            '<' if key == "<" => {
                let line = line_index(content, cursor);
                for i in 0..count {
                    let _ = unindent_line(content, line + i);
                }
                state.repeatable = Some(Repeatable::IndentLines(count, false));
                Some(changed(cursor))
            }
            _ => None,
        };
    }

    if let Pending::Replace = state.pending {
        state.pending = Pending::None;
        if let Some(ch) = key_char_lower(key) {
            let c = if shift {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            if cursor < content.chars().count() {
                delete_char_at(content, cursor);
            }
            insert_text(content, cursor, &c.to_string());
            state.repeatable = Some(Repeatable::ReplaceChar(c));
            return Some(changed(cursor));
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
            if let Some(ch) = key_char_lower(key) {
                let c = if shift {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                };
                state.last_find = Some((c, forward, till));
                if let Some(pos) = find_on_line(content, cursor, c, forward, till) {
                    return Some(unchanged(pos));
                }
            }
            return Some(unchanged(cursor));
        }
    }

    if let Pending::Operator('g') = state.pending {
        if key_matches(key, "g", shift) {
            state.pending = Pending::None;
            take_count(state);
            return Some(unchanged(0));
        }
        state.pending = Pending::None;
    }

    if ctrl || alt {
        if key != "Escape"
            && !(shift && matches_key_ci(key, &["i", "a", "o", "g"]))
        {
            return None;
        }
    }

    match (key, shift) {
        ("d", false) => {
            state.pending = Pending::Operator('d');
            Some(unchanged(cursor))
        }
        ("y", false) => {
            state.pending = Pending::Operator('y');
            Some(unchanged(cursor))
        }
        ("c", false) => {
            state.pending = Pending::Operator('c');
            Some(unchanged(cursor))
        }
        ("G", _) | ("g", true) => {
            take_count(state);
            Some(unchanged(content.chars().count()))
        }
        ("g", false) => {
            state.pending = Pending::Operator('g');
            Some(unchanged(cursor))
        }
        ("r", false) => {
            state.pending = Pending::Replace;
            Some(unchanged(cursor))
        }
        ("f", false) => {
            state.pending = Pending::FindForward;
            Some(unchanged(cursor))
        }
        ("F", _) | ("f", true) => {
            state.pending = Pending::FindBackward;
            Some(unchanged(cursor))
        }
        ("t", false) => {
            state.pending = Pending::TillForward;
            Some(unchanged(cursor))
        }
        ("T", _) | ("t", true) => {
            state.pending = Pending::TillBackward;
            Some(unchanged(cursor))
        }
        (";", false) => {
            if let Some((ch, forward, till)) = state.last_find {
                if let Some(pos) = find_on_line(content, cursor, ch, forward, till) {
                    return Some(unchanged(pos));
                }
            }
            Some(unchanged(cursor))
        }
        (",", false) => {
            if let Some((ch, forward, till)) = state.last_find {
                if let Some(pos) = find_on_line(content, cursor, ch, !forward, till) {
                    return Some(unchanged(pos));
                }
            }
            Some(unchanged(cursor))
        }
        (">", true) if state.pending == Pending::None => {
            state.pending = Pending::Operator('>');
            Some(unchanged(cursor))
        }
        ("<", true) if state.pending == Pending::None => {
            state.pending = Pending::Operator('<');
            Some(unchanged(cursor))
        }
        ("Escape", _) => {
            state.pending = Pending::None;
            state.count = 0;
            Some(action_mode(cursor, VimMode::Normal))
        }
        ("I", _) | ("i", true) => {
            state.pending = Pending::None;
            Some(action_mode(line_start(content, cursor), VimMode::Insert))
        }
        ("i", false) => {
            state.pending = Pending::None;
            Some(action_mode(cursor, VimMode::Insert))
        }
        ("A", _) | ("a", true) => {
            state.pending = Pending::None;
            Some(action_mode(line_end_char(content, cursor), VimMode::Insert))
        }
        ("a", false) => {
            state.pending = Pending::None;
            let pos = if cursor < content.chars().count() {
                move_right(content, cursor)
            } else {
                cursor
            };
            Some(action_mode(pos, VimMode::Insert))
        }
        ("O", _) | ("o", true) => {
            state.pending = Pending::None;
            let new_cursor = insert_newline(content, cursor, true);
            Some(changed_mode(new_cursor, VimMode::Insert))
        }
        ("o", false) => {
            state.pending = Pending::None;
            let line = line_index(content, cursor);
            let (_, end, _) = line_text(content, line);
            let insert_at = if end < content.chars().count() {
                end + 1
            } else {
                end
            };
            let new_cursor = insert_text(content, insert_at, "\n");
            Some(changed_mode(new_cursor, VimMode::Insert))
        }
        ("v", false) => {
            state.pending = Pending::None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: Some((cursor, cursor)),
                block_selection: None,
                vim_mode: Some(VimMode::Visual),
                consume: true,
                hint: None,
                command_result: None,
            })
        }
        ("h", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            move_left(content, p)
        }))),
        ("l", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            move_right(content, p)
        }))),
        ("k", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            move_up(content, p)
        }))),
        ("j", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            move_down(content, p)
        }))),
        ("w", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            word_forward(content, p)
        }))),
        ("b", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            word_backward(content, p)
        }))),
        ("e", false) => Some(unchanged(repeat_n(cursor, take_count(state), |p| {
            word_end(content, p)
        }))),
        ("^", _) => Some(unchanged(first_nonblank(content, cursor))),
        ("$", _) => Some(unchanged(line_end_char(content, cursor))),
        ("0", false) => Some(unchanged(line_start(content, cursor))),
        ("x", false) => {
            let count = take_count(state);
            let mut pos = cursor;
            for _ in 0..count {
                if pos >= content.chars().count() {
                    break;
                }
                pos = delete_char_at(content, pos);
            }
            state.repeatable = Some(Repeatable::DeleteChars(count));
            Some(changed(pos))
        }
        ("P", _) | ("p", true) => {
            let Some(text) = paste_text(state) else {
                return Some(unchanged(cursor));
            };
            if text.is_empty() {
                return Some(unchanged(cursor));
            }
            let new_cursor = insert_text(content, cursor, &text);
            Some(changed(new_cursor))
        }
        ("p", false) => {
            let Some(text) = paste_text(state) else {
                return Some(unchanged(cursor));
            };
            if text.is_empty() {
                return Some(unchanged(cursor));
            }
            let line = line_index(content, cursor);
            let new_cursor = paste_line_below(content, line, &text);
            Some(changed(new_cursor))
        }
        ("u", false) => Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            consume: true,
            hint: Some("使用 Ctrl+Z 撤销".to_string()),
            command_result: None,
        }),
        ("Home", _) => Some(unchanged(0)),
        ("End", _) => Some(unchanged(content.chars().count())),
        _ => {
            state.pending = Pending::None;
            if state.vim_mode == VimMode::Normal && key.len() == 1 {
                Some(unchanged(cursor))
            } else {
                None
            }
        }
    }
}

fn handle_vim_visual(
    content: &mut String,
    state: &mut KeybindingState,
    key: &str,
    shift: bool,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    let (sel_start, sel_end) = selection.unwrap_or((cursor, cursor));

    if key_matches(key, "f", shift) {
        state.pending = Pending::FindBackward;
        return Some(unchanged(sel_end));
    }

    match key {
        "h" => Some(select(sel_start, move_left(content, sel_end))),
        "l" => Some(select(sel_start, move_right(content, sel_end))),
        "k" => Some(select(sel_start, move_up(content, sel_end))),
        "j" => Some(select(sel_start, move_down(content, sel_end))),
        "Escape" => {
            state.pending = Pending::None;
            Some(action_mode(sel_start, VimMode::Normal))
        }
        "y" => {
            let (a, b) = ordered(sel_start, sel_end);
            yank_into(state, slice_chars(content, a, b));
            state.pending = Pending::None;
            Some(action_mode(sel_start, VimMode::Normal))
        }
        "d" | "x" => {
            let (a, b) = ordered(sel_start, sel_end);
            yank_into(state, kill_region(content, a, b));
            state.pending = Pending::None;
            Some(changed_mode(
                sel_start.min(content.chars().count()),
                VimMode::Normal,
            ))
        }
        _ => None,
    }
}

fn handle_emacs(
    content: &mut String,
    state: &mut KeybindingState,
    key: &str,
    ctrl: bool,
    alt: bool,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    let (sel_start, sel_end) = selection.unwrap_or((cursor, cursor));
    let has_selection = sel_start != sel_end;

    if ctrl && key == "u" {
        state.emacs_prefix = Some(state.emacs_prefix.unwrap_or(0) * 4);
        return Some(unchanged(cursor));
    }

    let prefix = state.emacs_prefix.take().unwrap_or(1).max(1);

    if alt {
        return match key {
            "b" | "B" => Some(unchanged(repeat_n(cursor, prefix, |p| {
                word_backward(content, p)
            }))),
            "f" | "F" => Some(unchanged(repeat_n(cursor, prefix, |p| {
                word_forward(content, p)
            }))),
            "<" => Some(unchanged(0)),
            ">" => Some(unchanged(content.chars().count())),
            "d" | "D" => {
                let end = word_forward(content, cursor);
                let killed = kill_region(content, cursor, end);
                push_kill_ring(state, killed);
                Some(changed(cursor))
            }
            _ => None,
        };
    }

    if !ctrl {
        return None;
    }

    match key {
        "b" => Some(unchanged(repeat_n(cursor, prefix, |p| move_left(content, p)))),
        "f" => Some(unchanged(repeat_n(cursor, prefix, |p| move_right(content, p)))),
        "p" => Some(unchanged(repeat_n(cursor, prefix, |p| move_up(content, p)))),
        "n" => Some(unchanged(repeat_n(cursor, prefix, |p| move_down(content, p)))),
        "a" => Some(unchanged(line_start(content, cursor))),
        "e" => Some(unchanged(line_end_char(content, cursor))),
        "v" => Some(unchanged(
            repeat_n(cursor, prefix, |p| move_down(content, p)).min(content.chars().count()),
        )),
        "d" => {
            let mut pos = cursor;
            for _ in 0..prefix {
                if pos >= content.chars().count() {
                    break;
                }
                pos = delete_char_at(content, pos);
            }
            Some(changed(pos))
        }
        "k" => {
            let end = line_end_char(content, cursor);
            if cursor == end {
                return Some(unchanged(cursor));
            }
            let killed = kill_region(content, cursor, end);
            push_kill_ring(state, killed);
            Some(changed(cursor))
        }
        "w" => {
            if !has_selection {
                return None;
            }
            let killed = kill_region(content, sel_start, sel_end);
            push_kill_ring(state, killed);
            Some(changed(sel_start.min(content.chars().count())))
        }
        "y" => {
            if let Some(text) = yank_kill_ring(state) {
                let new_cursor = insert_text(content, cursor, text);
                return Some(changed(new_cursor));
            }
            Some(unchanged(cursor))
        }
        "/" | "\\" => Some(KeyAction {
            content_changed: false,
            cursor,
            selection: None,
            block_selection: None,
            vim_mode: None,
            consume: true,
            hint: Some("使用 Ctrl+Z 撤销".to_string()),
            command_result: None,
        }),
        _ => None,
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
        let (key, shift, alt, ctrl) = parse_stored_key(&key_str)?;
        if let Some(action) = handle_vim(content, state, &key, shift, alt, ctrl, pos, sel) {
            pos = action.cursor;
            sel = action.selection;
            last_action = Some(action);
        }
    }
    last_action
}

fn parse_stored_key(s: &str) -> Option<(String, bool, bool, bool)> {
    if let Ok(n) = s.parse::<usize>() {
        return Some((n.to_string(), false, false, false));
    }
    if s.len() == 1 {
        let ch = s.chars().next()?;
        if ch.is_ascii_alphabetic() {
            let shift = ch.is_ascii_uppercase();
            return Some((ch.to_string(), shift, false, false));
        }
        return Some((s.to_string(), false, false, false));
    }
    match s {
        "Escape" | "Home" | "End" | ";" | "," | "@" | ">" | "<" | "$" | "^" => {
            Some((s.to_string(), false, false, false))
        }
        _ => None,
    }
}

fn record_macro_key(state: &mut KeybindingState, key: &str, shift: bool) {
    if let Some(reg) = state.macro_recording {
        let name = key_name(key, shift);
        state.macros.entry(reg).or_default().push(name);
    }
}

fn key_name(key: &str, shift: bool) -> String {
    if let Some(d) = key_digit(key) {
        return d.to_string();
    }
    if key.len() == 1 {
        let ch = key.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            return if shift {
                ch.to_ascii_uppercase().to_string()
            } else {
                ch.to_ascii_lowercase().to_string()
            };
        }
    }
    key.to_string()
}

fn apply_repeatable(
    content: &mut String,
    state: &mut KeybindingState,
    cursor: usize,
) -> Option<KeyAction> {
    let rep = state.repeatable.clone()?;
    match rep {
        Repeatable::DeleteLines(n) => {
            let line = line_index(content, cursor);
            let new_cursor = delete_lines(content, line, n);
            Some(changed(new_cursor))
        }
        Repeatable::DeleteChars(n) => {
            let mut pos = cursor;
            for _ in 0..n {
                pos = delete_char_at(content, pos);
            }
            Some(changed(pos))
        }
        Repeatable::DeleteWord(n) => {
            let pos = cursor;
            for _ in 0..n {
                let end = word_forward(content, pos);
                kill_region(content, pos, end);
            }
            Some(changed(cursor))
        }
        Repeatable::DeleteToEol => {
            let end = line_end_char(content, cursor);
            yank_into(state, kill_region(content, cursor, end));
            Some(changed(cursor))
        }
        Repeatable::ChangeLine => {
            let line = line_index(content, cursor);
            let (_, _, text) = line_text(content, line);
            yank_into(state, text);
            let new_cursor = delete_line(content, line);
            Some(changed_mode(new_cursor, VimMode::Insert))
        }
        Repeatable::ChangeWord => {
            let end = word_forward(content, cursor);
            yank_into(state, kill_region(content, cursor, end));
            Some(changed_mode(cursor, VimMode::Insert))
        }
        Repeatable::ReplaceChar(ch) => {
            if cursor < content.chars().count() {
                delete_char_at(content, cursor);
            }
            let new_cursor = insert_text(content, cursor, &ch.to_string());
            Some(changed(new_cursor.saturating_sub(1)))
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
            Some(changed(cursor))
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
            Some(unchanged(cursor))
        }
    }
}

fn handle_vim_visual_block(
    content: &mut String,
    state: &mut KeybindingState,
    key: &str,
    cursor: usize,
) -> Option<KeyAction> {
    let anchor = state
        .block_anchor
        .unwrap_or_else(|| vim_ex::pos_to_block_pos(content, cursor));
    let mut head = state.block_head.unwrap_or(anchor);

    match key {
        "h" => head.col = head.col.saturating_sub(1),
        "l" => head.col += 1,
        "k" if head.line > 0 => head.line -= 1,
        "j" => head.line += 1,
        "Escape" => {
            state.block_anchor = None;
            state.block_head = None;
            state.active_block = None;
            return Some(action_mode(cursor, VimMode::Normal));
        }
        "y" => {
            let rect = BlockRect::from_positions(anchor, head);
            yank_into(state, vim_ex::yank_block(content, rect));
            state.block_anchor = None;
            state.block_head = None;
            state.active_block = None;
            return Some(action_mode(cursor, VimMode::Normal));
        }
        "d" | "x" => {
            let rect = BlockRect::from_positions(anchor, head);
            yank_into(state, vim_ex::yank_block(content, rect));
            let new_cursor = vim_ex::delete_block(content, rect);
            state.block_anchor = None;
            state.block_head = None;
            state.active_block = None;
            return Some(changed_mode(new_cursor, VimMode::Normal));
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
        consume: true,
        hint: None,
        command_result: None,
    })
}

pub fn handle_command_input(
    state: &mut KeybindingState,
    key: &str,
    shift: bool,
    content: &mut String,
    cursor: usize,
) -> Option<KeyAction> {
    if state.vim_mode != VimMode::Command {
        return None;
    }
    match key {
        "Escape" => {
            state.command_buffer.clear();
            Some(action_mode(cursor, VimMode::Normal))
        }
        "Backspace" => {
            state.command_buffer.pop();
            Some(unchanged(cursor))
        }
        "Enter" => execute_command(state, content, cursor),
        _ => {
            if let Some(c) = key_char_lower(key) {
                let ch = if shift {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                state.command_buffer.push(ch);
                return Some(unchanged(cursor));
            }
            if let Some(d) = key_digit(key) {
                state
                    .command_buffer
                    .push(char::from_digit(d as u32, 10).unwrap());
                return Some(unchanged(cursor));
            }
            None
        }
    }
}

pub fn execute_command(
    state: &mut KeybindingState,
    content: &mut String,
    cursor: usize,
) -> Option<KeyAction> {
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
        vim_ex::execute_vim_command(&cmd, content, cursor)
    };
    let new_cursor = result.cursor.unwrap_or(cursor);
    Some(KeyAction {
        content_changed: result.content_changed,
        cursor: new_cursor,
        selection: None,
        block_selection: None,
        vim_mode: Some(VimMode::Normal),
        consume: true,
        hint: Some(result.status.clone()),
        command_result: Some(result),
    })
}

fn yank_into(state: &mut KeybindingState, text: String) {
    let reg = state.registers.take_pending().unwrap_or('"');
    state.registers.yank(Some(reg), text.clone());
    if state.use_system_clipboard {
        if reg == '"' || reg.is_ascii_lowercase() || reg == '+' || reg == '*' {
            state.registers.store_clipboard_register(&text);
            crate::clipboard::write_text(&text);
        }
    }
}

fn paste_text(state: &KeybindingState) -> Option<String> {
    let reg = state.registers.pending.unwrap_or('"');
    if state.use_system_clipboard && (reg == '+' || reg == '*' || reg == '"') {
        if let Some(text) = crate::clipboard::cached_text() {
            return Some(text);
        }
    }
    state.registers.get(reg).map(|s| s.to_string())
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

fn key_digit(key: &str) -> Option<usize> {
    match key {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        _ => None,
    }
}

fn key_char_lower(key: &str) -> Option<char> {
    if key.len() == 1 {
        let c = key.chars().next()?;
        if c.is_ascii_alphabetic() {
            return Some(c.to_ascii_lowercase());
        }
    }
    None
}

fn key_matches(key: &str, expected: &str, shift: bool) -> bool {
    if key.len() != 1 || expected.len() != 1 {
        return false;
    }
    let k = key.chars().next().unwrap();
    let e = expected.chars().next().unwrap();
    k.to_ascii_lowercase() == e.to_ascii_lowercase()
        && (k.is_ascii_uppercase() == shift || k == e)
}

fn matches_key_ci(key: &str, options: &[&str]) -> bool {
    if key.len() != 1 {
        return false;
    }
    let k = key.chars().next().unwrap().to_ascii_lowercase();
    options.iter().any(|o| o.chars().next().unwrap().to_ascii_lowercase() == k)
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

fn unchanged(cursor: usize) -> KeyAction {
    KeyAction {
        content_changed: false,
        cursor,
        selection: None,
        block_selection: None,
        vim_mode: None,
        consume: true,
        hint: None,
        command_result: None,
    }
}

fn changed(cursor: usize) -> KeyAction {
    KeyAction {
        content_changed: true,
        cursor,
        selection: None,
        block_selection: None,
        vim_mode: None,
        consume: true,
        hint: None,
        command_result: None,
    }
}

fn select(anchor: usize, head: usize) -> KeyAction {
    let (a, b) = ordered(anchor, head);
    KeyAction {
        content_changed: false,
        cursor: a,
        selection: Some((a, b)),
        block_selection: None,
        vim_mode: None,
        consume: true,
        hint: None,
        command_result: None,
    }
}

fn action_mode(cursor: usize, mode: VimMode) -> KeyAction {
    KeyAction {
        content_changed: false,
        cursor,
        selection: None,
        block_selection: None,
        vim_mode: Some(mode),
        consume: true,
        hint: None,
        command_result: None,
    }
}

fn changed_mode(cursor: usize, mode: VimMode) -> KeyAction {
    KeyAction {
        content_changed: true,
        cursor,
        selection: None,
        block_selection: None,
        vim_mode: Some(mode),
        consume: true,
        hint: None,
        command_result: None,
    }
}

fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

fn line_index(content: &str, pos: usize) -> usize {
    content.chars().take(pos).filter(|&c| c == '\n').count()
}

fn line_start(content: &str, pos: usize) -> usize {
    let pos = pos.min(content.chars().count());
    let mut start = 0usize;
    for (i, c) in content.chars().enumerate().take(pos) {
        if c == '\n' {
            start = i + 1;
        }
    }
    start
}

fn line_end_char(content: &str, pos: usize) -> usize {
    let pos = pos.min(content.chars().count());
    content
        .chars()
        .skip(pos)
        .position(|c| c == '\n')
        .map(|i| pos + i)
        .unwrap_or_else(|| content.chars().count())
}

fn first_nonblank(content: &str, pos: usize) -> usize {
    let start = line_start(content, pos);
    let end = line_end_char(content, pos);
    for (i, c) in content
        .chars()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
    {
        if !c.is_whitespace() {
            return i;
        }
    }
    start
}

fn move_left(_content: &str, pos: usize) -> usize {
    pos.saturating_sub(1)
}

fn move_right(content: &str, pos: usize) -> usize {
    (pos + 1).min(content.chars().count())
}

fn move_up(content: &str, pos: usize) -> usize {
    let start = line_start(content, pos);
    if start == 0 {
        return 0;
    }
    let prev_start = line_start(content, start.saturating_sub(1));
    let col = pos - start;
    (prev_start + col).min(line_end_char(content, prev_start))
}

fn move_down(content: &str, pos: usize) -> usize {
    let start = line_start(content, pos);
    let end = line_end_char(content, pos);
    if end >= content.chars().count() {
        return pos;
    }
    let next_start = end + 1;
    let col = pos - start;
    (next_start + col).min(line_end_char(content, next_start))
}

fn word_forward(content: &str, pos: usize) -> usize {
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

fn word_backward(content: &str, pos: usize) -> usize {
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

fn word_end(content: &str, pos: usize) -> usize {
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

fn line_text(content: &str, line_idx: usize) -> (usize, usize, String) {
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

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        1
    } else {
        content.chars().filter(|&c| c == '\n').count() + 1
    }
}

fn delete_line(content: &mut String, line_idx: usize) -> usize {
    if line_idx >= line_count(content) {
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

fn delete_lines(content: &mut String, line: usize, count: usize) -> usize {
    let line_idx = line;
    let mut cursor = line_start(content, char_index_at_line(content, line_idx));
    for _ in 0..count {
        if line_idx >= line_count(content) {
            break;
        }
        cursor = delete_line(content, line_idx);
    }
    cursor
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
    let (start, _, text) = line_text(content, line);
    let stripped = text.strip_prefix("    ").or_else(|| text.strip_prefix('\t'));
    if let Some(rest) = stripped {
        let remove_len = text.len() - rest.len();
        let (start_b, end_b) = char_range_to_bytes(content, start, start + remove_len);
        content.replace_range(start_b..end_b, "");
        true
    } else {
        let _ = start;
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

fn insert_newline(content: &mut String, pos: usize, before: bool) -> usize {
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

fn delete_char_at(content: &mut String, pos: usize) -> usize {
    if pos >= content.chars().count() {
        return pos;
    }
    let byte = char_index_to_byte(content, pos);
    let len = content[byte..]
        .chars()
        .next()
        .map(|c| c.len_utf8())
        .unwrap_or(0);
    content.replace_range(byte..byte + len, "");
    pos
}

fn kill_region(content: &mut String, start: usize, end: usize) -> String {
    let (a, b) = ordered(start, end);
    let (start_b, end_b) = char_range_to_bytes(content, a, b);
    let killed = content[start_b..end_b].to_string();
    content.replace_range(start_b..end_b, "");
    killed
}

fn insert_text(content: &mut String, pos: usize, text: &str) -> usize {
    let byte = char_index_to_byte(content, pos);
    content.insert_str(byte, text);
    pos + text.chars().count()
}

fn paste_line_below(content: &mut String, line_idx: usize, text: &str) -> usize {
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

fn slice_chars(content: &str, start: usize, end: usize) -> String {
    let (start_b, end_b) = char_range_to_bytes(content, start, end);
    content[start_b..end_b].to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn vim(
        content: &mut String,
        state: &mut KeybindingState,
        key: &str,
        cursor: usize,
    ) -> Option<KeyAction> {
        handle_vim(content, state, key, false, false, false, cursor, None)
    }

    #[test]
    fn vim_dd() {
        let mut text = "hello\nworld".to_string();
        let mut state = KeybindingState::default();
        vim(&mut text, &mut state, "d", 0);
        let action = vim(&mut text, &mut state, "d", 0).unwrap();
        assert!(action.content_changed);
        assert_eq!(text, "world");
    }

    #[test]
    fn vim_dd_with_count() {
        let mut text = "a\nb\nc\n".to_string();
        let mut state = KeybindingState::default();
        vim(&mut text, &mut state, "2", 0).unwrap();
        vim(&mut text, &mut state, "d", 0).unwrap();
        vim(&mut text, &mut state, "d", 0).unwrap();
        assert_eq!(text, "c\n");
    }

    #[test]
    fn vim_macro_record() {
        let mut text = "hi".to_string();
        let mut state = KeybindingState::default();
        vim(&mut text, &mut state, "q", 0).unwrap();
        vim(&mut text, &mut state, "a", 0).unwrap();
        assert!(state.macro_recording == Some('a'));
    }

    #[test]
    fn vim_count_j() {
        let mut text = "a\nb\nc".to_string();
        let mut state = KeybindingState::default();
        vim(&mut text, &mut state, "2", 0).unwrap();
        let action = vim(&mut text, &mut state, "j", 0).unwrap();
        assert_eq!(action.cursor, 4);
    }
}
