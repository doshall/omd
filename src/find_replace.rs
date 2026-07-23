use egui::{TextEdit, Ui};

pub const EDITOR_ID_SALT: &str = "omd_main_editor";

#[derive(Clone, Default)]
pub struct FindBarState {
    pub open: bool,
    pub replace_mode: bool,
    pub query: String,
    pub replace: String,
    pub case_sensitive: bool,
    pub match_index: usize,
    pub focus_find: bool,
    pub pending_selection: Option<(usize, usize)>,
}

impl FindBarState {
    pub fn open_find(&mut self, replace: bool) {
        self.open = true;
        self.replace_mode = replace;
        self.focus_find = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn match_ranges<'a>(&self, content: &'a str) -> Vec<(usize, usize)> {
        find_ranges(content, &self.query, self.case_sensitive)
    }

    pub fn advance_match(&mut self, content: &str, forward: bool) -> bool {
        let ranges = self.match_ranges(content);
        if ranges.is_empty() {
            self.pending_selection = None;
            return false;
        }
        if forward {
            self.match_index = (self.match_index + 1) % ranges.len();
        } else {
            self.match_index = (self.match_index + ranges.len() - 1) % ranges.len();
        }
        self.pending_selection = Some(ranges[self.match_index]);
        true
    }

    pub fn reset_match_index(&mut self) {
        self.match_index = 0;
        self.pending_selection = None;
    }
}

pub struct FindBarOutput {
    pub find_next: bool,
    pub find_prev: bool,
    pub close: bool,
    pub replace_one: bool,
    pub replace_all: bool,
}

/// Render the find/replace bar. Set flags on `output` from button clicks.
pub fn render_find_bar(ui: &mut Ui, state: &mut FindBarState, content: &str) -> FindBarOutput {
    let mut out = FindBarOutput {
        find_next: false,
        find_prev: false,
        close: false,
        replace_one: false,
        replace_all: false,
    };

    let ranges = state.match_ranges(content);
    let match_label = if state.query.is_empty() {
        "—".to_string()
    } else if ranges.is_empty() {
        "0/0".to_string()
    } else {
        format!("{}/{}", state.match_index + 1, ranges.len())
    };

    ui.horizontal(|ui| {
        ui.label("Find:");
        let response = ui.add(
            TextEdit::singleline(&mut state.query)
                .desired_width(160.0)
                .hint_text("Search…"),
        );
        if state.focus_find {
            response.request_focus();
            state.focus_find = false;
        }
        if response.changed() {
            state.reset_match_index();
        }

        if ui.checkbox(&mut state.case_sensitive, "Match case").changed() {
            state.reset_match_index();
        }

        ui.label(match_label);

        if ui.button("↑").on_hover_text("Previous (Shift+Enter)").clicked() {
            out.find_prev = true;
        }
        if ui.button("↓").on_hover_text("Next (Enter)").clicked() {
            out.find_next = true;
        }

        if state.replace_mode {
            ui.separator();
            ui.label("Replace:");
            ui.add(
                TextEdit::singleline(&mut state.replace)
                    .desired_width(120.0)
                    .hint_text("Replacement…"),
            );
            if ui.button("Replace").clicked() {
                out.replace_one = true;
            }
            if ui.button("Replace All").clicked() {
                out.replace_all = true;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("✕").on_hover_text("Close (Esc)").clicked() {
                out.close = true;
            }
        });
    });

    out
}

/// Find all non-overlapping byte ranges of `needle` in `haystack`.
pub fn find_ranges(haystack: &str, needle: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }

    let needle_chars: Vec<char> = needle.chars().collect();
    let mut ranges = Vec::new();
    let mut search_from = 0;

    while search_from < haystack.len() {
        if let Some(start) = find_from(haystack, search_from, &needle_chars, case_sensitive) {
            let end = start + needle.len();
            ranges.push((start, end));
            search_from = start + 1;
        } else {
            break;
        }
    }

    ranges
}

fn find_from(
    haystack: &str,
    from: usize,
    needle_chars: &[char],
    case_sensitive: bool,
) -> Option<usize> {
    if needle_chars.is_empty() {
        return None;
    }

    for (offset, _) in haystack[from..].char_indices() {
        let start = from + offset;
        if matches_at(haystack, start, needle_chars, case_sensitive) {
            return Some(start);
        }
    }
    None
}

fn matches_at(haystack: &str, byte_start: usize, needle_chars: &[char], case_sensitive: bool) -> bool {
    let mut hay_chars = haystack[byte_start..].chars();
    for &nc in needle_chars {
        let Some(c) = hay_chars.next() else {
            return false;
        };
        let eq = if case_sensitive {
            c == nc
        } else {
            c.eq_ignore_ascii_case(&nc)
        };
        if !eq {
            return false;
        }
    }
    true
}

/// Replace all matches; returns number of replacements.
pub fn replace_all(
    content: &mut String,
    needle: &str,
    replacement: &str,
    case_sensitive: bool,
) -> usize {
    let ranges = find_ranges(content, needle, case_sensitive);
    let count = ranges.len();
    for (start, end) in ranges.into_iter().rev() {
        content.replace_range(start..end, replacement);
    }
    count
}

/// Replace the match at `index` among current matches; returns new selection after replace.
pub fn replace_at(
    content: &mut String,
    needle: &str,
    replacement: &str,
    case_sensitive: bool,
    match_index: usize,
) -> Option<(usize, usize)> {
    let ranges = find_ranges(content, needle, case_sensitive);
    let (start, end) = *ranges.get(match_index)?;
    content.replace_range(start..end, replacement);
    let new_end = start + replacement.len();
    Some((start, new_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_case_insensitive() {
        let ranges = find_ranges("Hello hello HELLO", "hello", false);
        assert_eq!(ranges.len(), 3);
    }

    #[test]
    fn replace_all_count() {
        let mut s = "foo bar foo".to_string();
        let n = replace_all(&mut s, "foo", "baz", true);
        assert_eq!(n, 2);
        assert_eq!(s, "baz bar baz");
    }
}
