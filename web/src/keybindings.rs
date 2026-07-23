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
    pub consume: bool,
}

pub fn reset_for_mode(state: &mut KeybindingState, mode: KeybindingMode) {
    state.pending = None;
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
    match mode {
        KeybindingMode::Vim => handle_vim(content, state, key, shift, alt, ctrl, cursor, selection),
        KeybindingMode::Emacs => handle_emacs(content, state, key, ctrl, cursor, selection),
        KeybindingMode::Standard => None,
    }
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
    if ctrl || alt {
        return None;
    }
    let (sel_start, sel_end) = selection.unwrap_or((cursor, cursor));

    match state.vim_mode {
        VimMode::Insert => {
            if key == "Escape" {
                state.pending = None;
                return Some(action_mode(cursor, VimMode::Normal));
            }
            return None;
        }
        VimMode::Visual => return handle_vim_visual(content, state, key, sel_start, sel_end, cursor),
        VimMode::Normal => {}
    }

    if let Some(pending) = state.pending.take() {
        if pending == 'd' && key == "d" {
            let line = line_index(content, cursor);
            let (_, _, text) = line_text(content, line);
            state.yank_register = text;
            let new_cursor = delete_line(content, line);
            return Some(changed(new_cursor));
        }
        if pending == 'y' && key == "y" {
            let line = line_index(content, cursor);
            let (_, _, text) = line_text(content, line);
            state.yank_register = text;
            if !state.yank_register.ends_with('\n') {
                state.yank_register.push('\n');
            }
            return Some(unchanged(cursor));
        }
        if pending == 'g' && key == "g" {
            return Some(unchanged(0));
        }
    }

    match (key, shift) {
        ("d", false) => {
            state.pending = Some('d');
            Some(unchanged(cursor))
        }
        ("y", false) => {
            state.pending = Some('y');
            Some(unchanged(cursor))
        }
        ("g", false) => {
            state.pending = Some('g');
            Some(unchanged(cursor))
        }
        ("Escape", _) => {
            state.pending = None;
            Some(action_mode(cursor, VimMode::Normal))
        }
        ("i", false) => {
            state.pending = None;
            Some(action_mode(cursor, VimMode::Insert))
        }
        ("a", false) => {
            state.pending = None;
            let pos = if cursor < content.chars().count() {
                move_right(content, cursor)
            } else {
                cursor
            };
            Some(action_mode(pos, VimMode::Insert))
        }
        ("A", _) | ("a", true) => {
            state.pending = None;
            Some(action_mode(line_end_char(content, cursor), VimMode::Insert))
        }
        ("I", _) | ("i", true) => {
            state.pending = None;
            Some(action_mode(line_start(content, cursor), VimMode::Insert))
        }
        ("o", false) => {
            state.pending = None;
            let new_cursor = insert_newline(content, cursor, false);
            Some(changed_mode(new_cursor, VimMode::Insert))
        }
        ("O", _) | ("o", true) => {
            state.pending = None;
            let new_cursor = insert_newline(content, cursor, true);
            Some(changed_mode(new_cursor, VimMode::Insert))
        }
        ("v", false) => {
            state.pending = None;
            Some(KeyAction {
                content_changed: false,
                cursor,
                selection: Some((cursor, cursor)),
                vim_mode: Some(VimMode::Visual),
                consume: true,
            })
        }
        ("h", false) => Some(unchanged(move_left(content, cursor))),
        ("l", false) => Some(unchanged(move_right(content, cursor))),
        ("k", false) => Some(unchanged(move_up(content, cursor))),
        ("j", false) => Some(unchanged(move_down(content, cursor))),
        ("w", false) => Some(unchanged(word_forward(content, cursor))),
        ("b", false) => Some(unchanged(word_backward(content, cursor))),
        ("0", false) => Some(unchanged(line_start(content, cursor))),
        ("$", false) => Some(unchanged(line_end_char(content, cursor))),
        ("x", false) => {
            if cursor >= content.chars().count() {
                return Some(unchanged(cursor));
            }
            let new_cursor = delete_char_at(content, cursor);
            Some(changed(new_cursor))
        }
        ("p", false) => {
            if state.yank_register.is_empty() {
                return Some(unchanged(cursor));
            }
            let line = line_index(content, cursor);
            let new_cursor = paste_line_below(content, line, &state.yank_register);
            Some(changed(new_cursor))
        }
        ("Home", _) => Some(unchanged(0)),
        ("End", _) => Some(unchanged(content.chars().count())),
        _ => {
            state.pending = None;
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
    sel_start: usize,
    sel_end: usize,
    _cursor: usize,
) -> Option<KeyAction> {
    match key {
        "h" => Some(select(sel_start, move_left(content, sel_end))),
        "l" => Some(select(sel_start, move_right(content, sel_end))),
        "k" => Some(select(sel_start, move_up(content, sel_end))),
        "j" => Some(select(sel_start, move_down(content, sel_end))),
        "Escape" => {
            state.pending = None;
            Some(action_mode(sel_start, VimMode::Normal))
        }
        "y" => {
            let (a, b) = ordered(sel_start, sel_end);
            state.yank_register = slice_chars(content, a, b);
            state.pending = None;
            Some(action_mode(sel_start, VimMode::Normal))
        }
        "d" | "x" => {
            let (a, b) = ordered(sel_start, sel_end);
            state.yank_register = kill_region(content, a, b);
            state.pending = None;
            Some(changed_mode(a.min(content.chars().count()), VimMode::Normal))
        }
        _ => None,
    }
}

fn handle_emacs(
    content: &mut String,
    state: &mut KeybindingState,
    key: &str,
    ctrl: bool,
    cursor: usize,
    selection: Option<(usize, usize)>,
) -> Option<KeyAction> {
    if !ctrl {
        return None;
    }
    let (sel_start, sel_end) = selection.unwrap_or((cursor, cursor));
    match key {
        "b" => Some(unchanged(move_left(content, cursor))),
        "f" => Some(unchanged(move_right(content, cursor))),
        "p" => Some(unchanged(move_up(content, cursor))),
        "n" => Some(unchanged(move_down(content, cursor))),
        "a" => Some(unchanged(line_start(content, cursor))),
        "e" => Some(unchanged(line_end_char(content, cursor))),
        "d" => {
            if cursor >= content.chars().count() {
                return Some(unchanged(cursor));
            }
            Some(changed(delete_char_at(content, cursor)))
        }
        "k" => {
            let end = line_end_char(content, cursor);
            if cursor == end {
                return Some(unchanged(cursor));
            }
            state.kill_ring = kill_region(content, cursor, end);
            Some(changed(cursor))
        }
        "w" => {
            if sel_start == sel_end {
                return None;
            }
            state.kill_ring = kill_region(content, sel_start, sel_end);
            Some(changed(sel_start.min(content.chars().count())))
        }
        "y" => {
            if state.kill_ring.is_empty() {
                return Some(unchanged(cursor));
            }
            Some(changed(insert_text(content, cursor, &state.kill_ring)))
        }
        _ => None,
    }
}

fn unchanged(cursor: usize) -> KeyAction {
    KeyAction {
        content_changed: false,
        cursor,
        selection: None,
        vim_mode: None,
        consume: true,
    }
}

fn changed(cursor: usize) -> KeyAction {
    KeyAction {
        content_changed: true,
        cursor,
        selection: None,
        vim_mode: None,
        consume: true,
    }
}

fn select(anchor: usize, head: usize) -> KeyAction {
    let (a, b) = ordered(anchor, head);
    KeyAction {
        content_changed: false,
        cursor: a,
        selection: Some((a, b)),
        vim_mode: None,
        consume: true,
    }
}

fn action_mode(cursor: usize, mode: VimMode) -> KeyAction {
    KeyAction {
        content_changed: false,
        cursor,
        selection: None,
        vim_mode: Some(mode),
        consume: true,
    }
}

fn changed_mode(cursor: usize, mode: VimMode) -> KeyAction {
    KeyAction {
        content_changed: true,
        cursor,
        selection: None,
        vim_mode: Some(mode),
        consume: true,
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

fn delete_line(content: &mut String, line_idx: usize) -> usize {
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
    let len = content[byte..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
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

    #[test]
    fn vim_dd() {
        let mut text = "hello\nworld".to_string();
        let mut state = KeybindingState::default();
        handle_vim(&mut text, &mut state, "d", false, false, false, 0, None);
        let action = handle_vim(&mut text, &mut state, "d", false, false, false, 0, None).unwrap();
        assert!(action.content_changed);
        assert_eq!(text, "world");
    }
}
