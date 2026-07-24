use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProjectFolder {
    pub root: Option<PathBuf>,
}

impl ProjectFolder {
    pub fn rescan(&self) -> Vec<PathBuf> {
        self.root
            .as_ref()
            .map(|root| scan_markdown_files(root))
            .unwrap_or_default()
    }
}

pub fn is_markdown_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("md") | Some("markdown") | Some("txt")
    )
}

/// List markdown files in a single directory (non-recursive), sorted by name.
pub fn scan_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_markdown_file(&path) {
                files.push(path);
            }
        }
    }
    files.sort_by(|a, b| {
        a.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .cmp(
                b.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(""),
            )
    });
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("omd-project-test-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scans_flat_markdown_files() {
        let dir = temp_dir();
        fs::write(dir.join("a.md"), "# A").unwrap();
        fs::write(dir.join("b.txt"), "B").unwrap();
        fs::write(dir.join("skip.png"), "x").unwrap();
        let sub = dir.join("nested");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("hidden.md"), "nested").unwrap();

        let files = scan_markdown_files(&dir);
        assert_eq!(files.len(), 2);
        assert!(files[0].ends_with("a.md"));
        assert!(files[1].ends_with("b.txt"));

        let _ = fs::remove_dir_all(dir);
    }
}
