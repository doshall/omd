use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_TABS: usize = 20;
pub const MAX_RECENT: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabDocument {
    pub id: String,
    pub content: String,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TabStore {
    pub tabs: Vec<TabDocument>,
    pub active_id: String,
}

impl TabStore {
    pub fn new_document(content: String) -> Self {
        let tab = TabDocument {
            id: new_tab_id(),
            content,
            file_path: None,
            modified: false,
        };
        let active_id = tab.id.clone();
        Self {
            tabs: vec![tab],
            active_id,
        }
    }

    pub fn active_tab(&self) -> Option<&TabDocument> {
        self.tabs.iter().find(|t| t.id == self.active_id)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TabDocument> {
        let id = self.active_id.clone();
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn tab_label(tab: &TabDocument) -> String {
        if let Some(path) = &tab.file_path {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                return name.to_string();
            }
        }
        tab.content
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                trimmed.strip_prefix("# ").map(str::trim)
            })
            .filter(|t| !t.is_empty())
            .unwrap_or("未命名")
            .to_string()
    }

    pub fn add_tab(&mut self, content: String) -> bool {
        if self.tabs.len() >= MAX_TABS {
            return false;
        }
        let tab = TabDocument {
            id: new_tab_id(),
            content,
            file_path: None,
            modified: false,
        };
        self.active_id = tab.id.clone();
        self.tabs.push(tab);
        true
    }

    pub fn close_tab(&mut self, id: &str) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        let idx = match self.tabs.iter().position(|t| t.id == id) {
            Some(i) => i,
            None => return false,
        };
        self.tabs.remove(idx);
        if self.active_id == id {
            let next = idx.min(self.tabs.len().saturating_sub(1));
            self.active_id = self.tabs[next].id.clone();
        }
        true
    }

    pub fn switch_tab(&mut self, id: &str) -> bool {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_id = id.to_string();
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RecentFiles {
    pub paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn push(&mut self, path: PathBuf) {
        self.paths.retain(|p| p != &path);
        self.paths.insert(0, path);
        self.paths.truncate(MAX_RECENT);
    }

    pub fn open_path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(PathBuf::as_path)
    }
}

fn new_tab_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("tab-{millis}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_tab_switches_active() {
        let mut store = TabStore::new_document("a".into());
        store.add_tab("b".into());
        let second = store.active_id.clone();
        let first = store.tabs[0].id.clone();
        store.switch_tab(&first);
        assert!(store.close_tab(&first));
        assert_eq!(store.active_id, second);
    }
}
