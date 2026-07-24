use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::idb;

pub const STORAGE_TABS: &str = "omd-web-tabs";
pub const STORAGE_TABS_META: &str = "omd-web-tabs-meta";
const LEGACY_CONTENT: &str = "omd-web-content";
const LEGACY_FILENAME: &str = "omd-web-filename";
const MAX_TABS: usize = 20;
const INLINE_STORAGE_THRESHOLD: usize = 1_500_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tab {
    pub id: String,
    pub filename: String,
    pub content: String,
    pub saved_snapshot: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TabStore {
    pub tabs: Vec<Tab>,
    pub active_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TabMeta {
    id: String,
    filename: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TabStoreMeta {
    tabs: Vec<TabMeta>,
    active_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TabDoc {
    content: String,
    saved_snapshot: String,
}

impl TabStore {
    pub fn new_tab(content: String, filename: String) -> Tab {
        let id = new_tab_id();
        Tab {
            id: id.clone(),
            filename,
            content: content.clone(),
            saved_snapshot: content,
        }
    }

    pub fn default_with_content(content: String, filename: String) -> Self {
        let tab = Self::new_tab(content, filename);
        let active_id = tab.id.clone();
        Self {
            tabs: vec![tab],
            active_id,
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.active_id)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let id = self.active_id.clone();
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn tab_label(tab: &Tab) -> String {
        let stem = tab
            .filename
            .strip_suffix(".md")
            .or_else(|| tab.filename.strip_suffix(".markdown"))
            .or_else(|| tab.filename.strip_suffix(".txt"))
            .unwrap_or(tab.filename.as_str());
        if !stem.is_empty() && stem != "document" {
            return stem.to_string();
        }
        tab.content
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                trimmed.strip_prefix("# ").map(str::trim)
            })
            .filter(|title| !title.is_empty())
            .unwrap_or("未命名")
            .to_string()
    }

    pub fn add_tab(&mut self, content: String, filename: String) -> bool {
        if self.tabs.len() >= MAX_TABS {
            return false;
        }
        let tab = Self::new_tab(content, filename);
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
        let removed_id = self.tabs[idx].id.clone();
        self.tabs.remove(idx);
        if self.active_id == id {
            let next = idx.min(self.tabs.len().saturating_sub(1));
            self.active_id = self.tabs[next].id.clone();
        }
        if idb::storage_uses_idb() {
            let key = idb::tab_key(&removed_id);
            #[cfg(target_arch = "wasm32")]
            spawn_local(async move {
                let _ = idb::delete_string(&key).await;
            });
        }
        true
    }

    pub fn switch_tab(&mut self, id: &str) -> bool {
        if !self.tabs.iter().any(|t| t.id == id) {
            return false;
        }
        self.active_id = id.to_string();
        true
    }
}

pub fn load_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

pub fn save_storage(key: &str, value: &str) -> bool {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        storage.set_item(key, value).is_ok()
    } else {
        false
    }
}

pub fn needs_idb_hydration() -> bool {
    idb::storage_uses_idb() || load_storage(STORAGE_TABS_META).is_some()
}

pub fn load_tab_store(default_content: String, default_filename: String) -> TabStore {
    if needs_idb_hydration() {
        return TabStore::default_with_content(String::new(), "document.md".to_string());
    }

    if let Some(json) = load_storage(STORAGE_TABS) {
        if let Ok(store) = serde_json::from_str::<TabStore>(&json) {
            if !store.tabs.is_empty() && store.tabs.iter().any(|t| t.id == store.active_id) {
                return store;
            }
        }
    }

    let content = load_storage(LEGACY_CONTENT).unwrap_or(default_content);
    let filename = load_storage(LEGACY_FILENAME).unwrap_or(default_filename);
    TabStore::default_with_content(content, filename)
}

pub fn persist_tab_store(store: &TabStore) {
    if let Ok(json) = serde_json::to_string(store) {
        if !idb::storage_uses_idb() && json.len() < INLINE_STORAGE_THRESHOLD && save_storage(STORAGE_TABS, &json)
        {
            return;
        }
    }
    persist_tab_store_idb(store);
}

fn persist_tab_store_idb(store: &TabStore) {
    let meta = TabStoreMeta {
        tabs: store
            .tabs
            .iter()
            .map(|tab| TabMeta {
                id: tab.id.clone(),
                filename: tab.filename.clone(),
            })
            .collect(),
        active_id: store.active_id.clone(),
    };
    if let Ok(json) = serde_json::to_string(&meta) {
        let _ = save_storage(STORAGE_TABS_META, &json);
        let _ = save_storage(STORAGE_TABS, "");
    }
    idb::enable_idb_storage();

    let tabs: Vec<(String, String, String)> = store
        .tabs
        .iter()
        .map(|tab| {
            (
                tab.id.clone(),
                tab.content.clone(),
                tab.saved_snapshot.clone(),
            )
        })
        .collect();
    #[cfg(target_arch = "wasm32")]
    spawn_local(async move {
        for (id, content, saved_snapshot) in tabs {
            if let Ok(doc) = serde_json::to_string(&TabDoc {
                content,
                saved_snapshot,
            }) {
                let _ = idb::put_string(&idb::tab_key(&id), &doc).await;
            }
        }
    });
}

pub async fn hydrate_tab_store_from_idb() -> Option<TabStore> {
    let meta_json = load_storage(STORAGE_TABS_META)?;
    let meta: TabStoreMeta = serde_json::from_str(&meta_json).ok()?;
    let mut tabs = Vec::with_capacity(meta.tabs.len());
    for tab_meta in meta.tabs {
        let doc_json = idb::get_string(&idb::tab_key(&tab_meta.id))
            .await
            .ok()
            .flatten()?;
        let doc: TabDoc = serde_json::from_str(&doc_json).ok()?;
        tabs.push(Tab {
            id: tab_meta.id,
            filename: tab_meta.filename,
            content: doc.content,
            saved_snapshot: doc.saved_snapshot,
        });
    }
    if tabs.is_empty() {
        return None;
    }
    if !tabs.iter().any(|tab| tab.id == meta.active_id) {
        return None;
    }
    Some(TabStore {
        tabs,
        active_id: meta.active_id,
    })
}

pub async fn migrate_inline_tabs_to_idb_if_needed() -> Option<TabStore> {
    if idb::storage_uses_idb() || load_storage(STORAGE_TABS_META).is_some() {
        return None;
    }
    let json = load_storage(STORAGE_TABS)?;
    let store: TabStore = serde_json::from_str(&json).ok()?;
    if store.tabs.is_empty() {
        return None;
    }
    if json.len() < INLINE_STORAGE_THRESHOLD {
        return None;
    }
    persist_tab_store_idb(&store);
    Some(store)
}

fn new_tab_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let millis = js_sys::Date::now() as u64;
        let random = (js_sys::Math::random() * 1_000_000.0) as u64;
        format!("tab-{millis}-{random}")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("tab-test-{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_label_from_filename() {
        let tab = Tab {
            id: "a".into(),
            filename: "notes.md".into(),
            content: String::new(),
            saved_snapshot: String::new(),
        };
        assert_eq!(TabStore::tab_label(&tab), "notes");
    }

    #[test]
    fn close_tab_switches_active() {
        let mut store = TabStore::default_with_content("a".into(), "a.md".into());
        store.add_tab("b".into(), "b.md".into());
        let second_id = store.active_id.clone();
        let first_id = store.tabs[0].id.clone();
        store.switch_tab(&first_id);
        assert!(store.close_tab(&first_id));
        assert_eq!(store.active_id, second_id);
    }

    #[test]
    fn cannot_close_last_tab() {
        let mut store = TabStore::default_with_content("a".into(), "a.md".into());
        let id = store.tabs[0].id.clone();
        assert!(!store.close_tab(&id));
    }
}
