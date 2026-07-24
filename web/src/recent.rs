use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::idb;

pub const STORAGE_RECENT: &str = "omd-web-recent";
pub const MAX_RECENT: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentEntry {
    #[serde(default = "new_recent_id")]
    pub id: String,
    pub filename: String,
    /// Legacy inline content kept for migration from older versions.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RecentFiles {
    pub entries: Vec<RecentEntry>,
}

impl RecentFiles {
    pub fn push(&mut self, filename: String, content: String) {
        let id = new_recent_id();
        self.entries.retain(|e| e.filename != filename);
        let mut entry = RecentEntry {
            id: id.clone(),
            filename,
            content: String::new(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            entry.content = content;
        }
        self.entries.insert(0, entry);
        self.entries.truncate(MAX_RECENT);
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let _ = idb::put_string(&idb::recent_key(&id), &content).await;
        });
    }
}

pub async fn load_entry_content(entry: &RecentEntry) -> Option<String> {
    if !entry.content.is_empty() {
        return Some(entry.content.clone());
    }
    idb::get_string(&idb::recent_key(&entry.id))
        .await
        .ok()
        .flatten()
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
        let _ = crate::tabs::save_storage(STORAGE_RECENT, &json);
    }
}

fn new_recent_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let millis = js_sys::Date::now() as u64;
        let random = (js_sys::Math::random() * 1_000_000.0) as u64;
        format!("recent-{millis}-{random}")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("recent-test-{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_keeps_most_recent_first() {
        let mut recent = RecentFiles::default();
        recent.push("a.md".into(), "a".into());
        recent.push("b.md".into(), "b".into());
        assert_eq!(recent.entries[0].filename, "b.md");
        assert_eq!(recent.entries.len(), 2);
    }

    #[test]
    fn deduplicates_by_filename() {
        let mut recent = RecentFiles::default();
        recent.push("a.md".into(), "first".into());
        recent.push("b.md".into(), "b".into());
        recent.push("a.md".into(), "second".into());
        assert_eq!(recent.entries.len(), 2);
        assert_eq!(recent.entries[0].filename, "a.md");
    }
}
