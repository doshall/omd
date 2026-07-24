/// Toggle the Nth task list item (0-based) between `[ ]` and `[x]`.
pub fn toggle_task_by_index(markdown: &str, task_index: usize) -> Option<String> {
    let mut current = 0usize;
    let mut lines: Vec<String> = markdown.lines().map(String::from).collect();
    for line in &mut lines {
        if let Some((indent, checked, rest)) = parse_task_line(line) {
            if current == task_index {
                let marker = if checked { "[ ]" } else { "[x]" };
                *line = format!("{indent}- {marker}{rest}");
                return Some(join_lines(&lines, markdown.ends_with('\n')));
            }
            current += 1;
        }
    }
    None
}

pub fn task_line_count(markdown: &str) -> usize {
    markdown
        .lines()
        .filter(|line| parse_task_line(line).is_some())
        .count()
}

fn parse_task_line(line: &str) -> Option<(String, bool, String)> {
    let trimmed = line.trim_start();
    let indent = line[..line.len() - trimmed.len()].to_string();
    if let Some(rest) = trimmed.strip_prefix("- [ ]") {
        return Some((indent, false, rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("- [x]") {
        return Some((indent, true, rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("- [X]") {
        return Some((indent, true, rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("* [ ]") {
        return Some((indent, false, rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("* [x]") {
        return Some((indent, true, rest.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("* [X]") {
        return Some((indent, true, rest.to_string()));
    }
    None
}

fn join_lines(lines: &[String], trailing_newline: bool) -> String {
    if trailing_newline {
        format!("{}\n", lines.join("\n"))
    } else {
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_second_task() {
        let md = "- [ ] one\n- [ ] two\n";
        let out = toggle_task_by_index(md, 1).unwrap();
        assert!(out.contains("- [ ] one"));
        assert!(out.contains("- [x] two"));
    }

    #[test]
    fn toggles_checked_to_unchecked() {
        let md = "- [x] done\n";
        let out = toggle_task_by_index(md, 0).unwrap();
        assert!(out.contains("- [ ] done"));
    }
}
