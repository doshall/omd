use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

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

pub fn select_range(textarea: &HtmlTextAreaElement, start: usize, end: usize) {
    let len = textarea.value().len() as u32;
    let start = (start as u32).min(len);
    let end = (end as u32).min(len);
    textarea.set_selection_start(Some(start)).ok();
    textarea.set_selection_end(Some(end)).ok();
    textarea.focus().ok();
}

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
