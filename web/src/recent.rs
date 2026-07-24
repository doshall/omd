use serde::{Deserialize, Serialize};

pub const STORAGE_RECENT: &str = "omd-web-recent";
pub const MAX_RECENT: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentEntry {
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RecentFiles {
    pub entries: Vec<RecentEntry>,
}

impl RecentFiles {
    pub fn push(&mut self, filename: String) {
        self.entries.retain(|e| e.filename != filename);
        self.entries.insert(0, RecentEntry { filename });
        self.entries.truncate(MAX_RECENT);
    }
}

pub fn load_recent() -> RecentFiles {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_RECENT).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn save_recent(recent: &RecentFiles) {
    if let Ok(json) = serde_json::to_string(recent) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = storage.set_item(STORAGE_RECENT, &json);
        }
    }
}
