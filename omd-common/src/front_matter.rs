#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub raw: String,
}

/// Split YAML front matter (`---` delimited) from Markdown body.
pub fn parse_front_matter(input: &str) -> (Option<FrontMatter>, &str) {
    let trimmed = input.strip_prefix('\u{feff}').unwrap_or(input);
    if !trimmed.starts_with("---") {
        return (None, input);
    }
    let after_first = trimmed.strip_prefix("---").unwrap_or(trimmed);
    let after_first = after_first.strip_prefix('\n').unwrap_or(after_first);
    let Some(end) = after_first.find("\n---") else {
        return (None, input);
    };
    let yaml = &after_first[..end];
    let body = after_first[end + 4..].trim_start_matches('\n');
    (Some(parse_yaml_block(yaml)), body)
}

fn parse_yaml_block(yaml: &str) -> FrontMatter {
    let mut fm = FrontMatter {
        raw: yaml.to_string(),
        ..Default::default()
    };
    for line in yaml.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("title:") {
            fm.title = Some(unquote_yaml_value(value.trim()));
        } else if let Some(value) = line.strip_prefix("description:") {
            fm.description = Some(unquote_yaml_value(value.trim()));
        }
    }
    fm
}

fn unquote_yaml_value(value: &str) -> String {
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value[1..value.len().saturating_sub(1)].to_string()
    } else {
        value.to_string()
    }
}

pub fn resolve_title(
    front_matter: Option<&FrontMatter>,
    filename: Option<&str>,
    markdown_body: &str,
) -> String {
    if let Some(fm) = front_matter {
        if let Some(title) = fm.title.as_deref().filter(|t| !t.is_empty()) {
            return title.to_string();
        }
    }
    if let Some(name) = filename {
        let stem = name
            .strip_suffix(".md")
            .or_else(|| name.strip_suffix(".markdown"))
            .or_else(|| name.strip_suffix(".txt"))
            .unwrap_or(name);
        if !stem.is_empty() && stem != "document" {
            return stem.to_string();
        }
    }
    markdown_body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("# ").map(str::trim)
        })
        .filter(|title| !title.is_empty())
        .unwrap_or("document")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_from_front_matter() {
        let md = "---\ntitle: My Doc\ndescription: hello\n---\n\n# Body\n";
        let (fm, body) = parse_front_matter(md);
        let fm = fm.unwrap();
        assert_eq!(fm.title.as_deref(), Some("My Doc"));
        assert_eq!(fm.description.as_deref(), Some("hello"));
        assert!(body.starts_with("# Body"));
        assert_eq!(
            resolve_title(Some(&fm), Some("other.md"), body),
            "My Doc"
        );
    }
}
