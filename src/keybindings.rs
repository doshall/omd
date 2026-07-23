use eframe::egui::{self, Key, Modifiers};

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
}

impl VimMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
        }
    }
}

#[derive(Clone, Default)]
pub struct KeybindingState {
    pub vim_mode: VimMode,
    pub pending: Option<char>,
    pub yank_register: String,
    pub kill_ring: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyAction {
    pub content_changed: bool,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    pub vim_mode: Option<VimMode>,
    pub status: Option<String>,
}

impl KeyAction {
    fn cursor_only(cursor: usize) -> Self {
        Self {
            content_changed: false,
            cursor,
            selection: None,
            vim_mode: None,
            status: None,
        }
    }

    fn with_selection(cursor: usize, selection: (usize, usize)) -> Self {
        Self {
            content_changed: false,
            cursor,
            selection: Some(selection),
            vim_mode: None,
            status: None,
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

pub fn move_left(content: &str, pos: usize) -> usize {
    pos.saturating_sub(1).min(content.chars().count())
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
    if i == 0 && chars[0].is_whitespace() {
        0
    } else {
        i
    }
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

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        1
    } else {
        content.chars().filter(|&c| c == '\n').count() + 1
    }
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

pub fn selection_range(cursor: usize, selection: Option<(usize, usize)>) -> (usize, usize) {
    selection.unwrap_or((cursor, cursor))
}

pub fn handle_vim(
    content: &mut String,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    if !modifiers.alt && !modifiers.shift && !modifiers.ctrl && !modifiers.command {
        // allow Shift for capital letter commands below
    } else if key != Key::Escape && !(modifiers.shift && matches!(key, Key::I | Key::A | Key::O)) {
        return None;
    }

    let (sel_start, sel_end) = selection_range(cursor, selection);

    match state.vim_mode {
        VimMode::Insert => {
            if key == Key::Escape {
                state.pending = None;
                return Some(KeyAction {
                    content_changed: false,
                    cursor,
                    selection: None,
                    vim_mode: Some(VimMode::Normal),
                    status: Some("NORMAL".to_string()),
                });
            }
            return None;
        }
        VimMode::Visual => {
            let new_sel = match key {
                Key::H => Some((sel_start, move_left(content, sel_end))),
                Key::L => Some((sel_start, move_right(content, sel_end))),
                Key::K => Some((sel_start, move_up(content, sel_end))),
                Key::J => Some((sel_start, move_down(content, sel_end))),
                Key::Escape => {
                    state.pending = None;
                    return Some(KeyAction {
                        content_changed: false,
                        cursor: sel_start,
                        selection: None,
                        vim_mode: Some(VimMode::Normal),
                        status: Some("NORMAL".to_string()),
                    });
                }
                Key::Y => {
                    let (a, b) = if sel_start <= sel_end {
                        (sel_start, sel_end)
                    } else {
                        (sel_end, sel_start)
                    };
                    let (start_b, end_b) = char_range_to_bytes(content, a, b);
                    state.yank_register = content[start_b..end_b].to_string();
                    state.pending = None;
                    return Some(KeyAction {
                        content_changed: false,
                        cursor: sel_start,
                        selection: None,
                        vim_mode: Some(VimMode::Normal),
                        status: Some("NORMAL".to_string()),
                    });
                }
                Key::D | Key::X => {
                    let killed = kill_region(content, sel_start, sel_end);
                    state.yank_register = killed;
                    state.pending = None;
                    return Some(KeyAction {
                        content_changed: true,
                        cursor: sel_start.min(content.chars().count()),
                        selection: None,
                        vim_mode: Some(VimMode::Normal),
                        status: Some("NORMAL".to_string()),
                    });
                }
                _ => None,
            };
            if let Some((start, end)) = new_sel {
                let (a, b) = if start <= end { (start, end) } else { (end, start) };
                return Some(KeyAction::with_selection(a, (a, b)));
            }
            return None;
        }
        VimMode::Normal => {}
    }

    if let Some(pending) = state.pending.take() {
        if pending == 'd' && key == Key::D {
            let line = line_index(content, cursor);
            let (start, end, text) = line_text(content, line);
            state.yank_register = text;
            let new_cursor = delete_line(content, line);
            return Some(KeyAction {
                content_changed: true,
                cursor: new_cursor.min(content.chars().count()),
                selection: None,
                vim_mode: None,
                status: None,
            });
        }
        if pending == 'y' && key == Key::Y {
            let line = line_index(content, cursor);
            let (_, _, text) = line_text(content, line);
            state.yank_register = text.clone();
            if !state.yank_register.ends_with('\n') {
                state.yank_register.push('\n');
            }
            return Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
                vim_mode: None,
                status: None,
            });
        }
        if pending == 'g' && key == Key::G {
            return Some(KeyAction::cursor_only(0));
        }
    }

    match key {
        Key::D => {
            state.pending = Some('d');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Y => {
            state.pending = Some('y');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::G => {
            state.pending = Some('g');
            Some(KeyAction::cursor_only(cursor))
        }
        Key::Escape => {
            state.pending = None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
                vim_mode: Some(VimMode::Normal),
                status: Some("NORMAL".to_string()),
            })
        }
        Key::I if modifiers.shift => {
            state.pending = None;
            let pos = line_start(content, cursor);
            Some(KeyAction {
                content_changed: false,
                cursor: pos,
                selection: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::I => {
            state.pending = None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::A if modifiers.shift => {
            state.pending = None;
            let pos = line_end_char(content, cursor);
            Some(KeyAction {
                content_changed: false,
                cursor: pos,
                selection: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::A => {
            state.pending = None;
            let pos = if cursor < content.chars().count() {
                move_right(content, cursor)
            } else {
                cursor
            };
            Some(KeyAction {
                content_changed: false,
                cursor: pos,
                selection: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::O if modifiers.shift => {
            state.pending = None;
            let line = line_index(content, cursor);
            let new_cursor = insert_newline(content, cursor, true);
            let _ = line;
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::O => {
            state.pending = None;
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
                vim_mode: Some(VimMode::Insert),
                status: Some("INSERT".to_string()),
            })
        }
        Key::V => {
            state.pending = None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: Some((cursor, cursor)),
                vim_mode: Some(VimMode::Visual),
                status: Some("VISUAL".to_string()),
            })
        }
        Key::H => Some(KeyAction::cursor_only(move_left(content, cursor))),
        Key::L => Some(KeyAction::cursor_only(move_right(content, cursor))),
        Key::K => Some(KeyAction::cursor_only(move_up(content, cursor))),
        Key::J => Some(KeyAction::cursor_only(move_down(content, cursor))),
        Key::W => Some(KeyAction::cursor_only(word_forward(content, cursor))),
        Key::B => Some(KeyAction::cursor_only(word_backward(content, cursor))),
        Key::Num0 => Some(KeyAction::cursor_only(line_start(content, cursor).min(content.chars().count()))),
        Key::Num4 if modifiers.shift => Some(KeyAction::cursor_only(line_end_char(content, cursor))),
        Key::X => {
            if cursor >= content.chars().count() {
                return Some(KeyAction::cursor_only(cursor));
            }
            let new_cursor = delete_char_at(content, cursor);
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                vim_mode: None,
                status: None,
            })
        }
        Key::P => {
            if state.yank_register.is_empty() {
                return Some(KeyAction::cursor_only(cursor));
            }
            let line = line_index(content, cursor);
            let new_cursor = paste_line_below(content, line, &state.yank_register);
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                vim_mode: None,
                status: None,
            })
        }
        Key::Home => Some(KeyAction::cursor_only(0)),
        Key::End => Some(KeyAction::cursor_only(content.chars().count())),
        _ => {
            state.pending = None;
            None
        }
    }
}

pub fn handle_emacs(
    content: &mut String,
    state: &mut KeybindingState,
    key: Key,
    modifiers: Modifiers,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    if !modifiers.ctrl && !modifiers.command {
        return None;
    }
    let ctrl = modifiers.ctrl || modifiers.command;
    let (sel_start, sel_end) = selection_range(cursor, selection);
    let has_selection = sel_start != sel_end;

    match key {
        Key::B if ctrl => Some(KeyAction::cursor_only(move_left(content, cursor))),
        Key::F if ctrl => Some(KeyAction::cursor_only(move_right(content, cursor))),
        Key::P if ctrl => Some(KeyAction::cursor_only(move_up(content, cursor))),
        Key::N if ctrl => Some(KeyAction::cursor_only(move_down(content, cursor))),
        Key::A if ctrl => Some(KeyAction::cursor_only(
            line_start(content, cursor).min(content.chars().count()),
        )),
        Key::E if ctrl => Some(KeyAction::cursor_only(line_end_char(content, cursor))),
        Key::D if ctrl => {
            if cursor >= content.chars().count() {
                return Some(KeyAction::cursor_only(cursor));
            }
            let new_cursor = delete_char_at(content, cursor);
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                vim_mode: None,
                status: None,
            })
        }
        Key::K if ctrl => {
            let end = line_end_char(content, cursor);
            if cursor == end {
                return Some(KeyAction::cursor_only(cursor));
            }
            state.kill_ring = kill_region(content, cursor, end);
            Some(KeyAction {
                content_changed: true,
                cursor,
                selection: None,
                vim_mode: None,
                status: None,
            })
        }
        Key::W if ctrl => {
            if !has_selection {
                return None;
            }
            state.kill_ring = kill_region(content, sel_start, sel_end);
            Some(KeyAction {
                content_changed: true,
                cursor: sel_start.min(content.chars().count()),
                selection: None,
                vim_mode: None,
                status: None,
            })
        }
        Key::Y if ctrl => {
            if state.kill_ring.is_empty() {
                return Some(KeyAction::cursor_only(cursor));
            }
            let new_cursor = insert_text(content, cursor, &state.kill_ring);
            Some(KeyAction {
                content_changed: true,
                cursor: new_cursor,
                selection: None,
                vim_mode: None,
                status: None,
            })
        }
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
        for event in &input.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                let result = match mode {
                    KeybindingMode::Vim => {
                        if state.vim_mode == VimMode::Insert {
                            if *key == Key::Escape {
                                handle_vim(content, state, *key, *modifiers, cursor, selection)
                            } else {
                                None
                            }
                        } else {
                            handle_vim(content, state, *key, *modifiers, cursor, selection)
                        }
                    }
                    KeybindingMode::Emacs => handle_emacs(content, state, *key, *modifiers, cursor, selection),
                    KeybindingMode::Standard => None,
                };
                if let Some(act) = result {
                    input.consume_key(*modifiers, *key);
                    action = Some(act);
                    break;
                }
            }
        }
    });

    if let Some(act) = &action {
        if let Some(vim_mode) = act.vim_mode {
            state.vim_mode = vim_mode;
        }
        let sel = act
            .selection
            .map(|(a, b)| egui::text::CCursorRange::two(
                egui::text::CCursor::new(a),
                egui::text::CCursor::new(b),
            ))
            .unwrap_or_else(|| {
                egui::text::CCursorRange::one(egui::text::CCursor::new(act.cursor))
            });
        text_state.cursor.set_char_range(Some(sel));
        text_state.store(ctx, text_edit_id);
    }

    action
}

pub fn reset_for_mode(state: &mut KeybindingState, mode: KeybindingMode) {
    state.pending = None;
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
    fn vim_dd() {
        let mut text = "hello\nworld".to_string();
        let mut state = KeybindingState::default();
        let a1 = handle_vim(&mut text, &mut state, Key::D, Modifiers::NONE, 0, None).unwrap();
        assert!(!a1.content_changed);
        let a2 = handle_vim(&mut text, &mut state, Key::D, Modifiers::NONE, 0, None).unwrap();
        assert!(a2.content_changed);
        assert_eq!(text, "world");
    }
}
