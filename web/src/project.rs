use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{File, FileList, FileReader};

thread_local! {
    static PROJECT_FILES: RefCell<HashMap<String, File>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectEntry {
    pub name: String,
    pub relative_path: String,
}

pub fn is_markdown_filename(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".txt")
}

pub fn folder_label_from_entries(entries: &[ProjectEntry]) -> Option<String> {
    entries
        .first()
        .and_then(|e| e.relative_path.split('/').next())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub fn ingest_file_list(files: &FileList) -> Vec<ProjectEntry> {
    let mut entries = Vec::new();
    PROJECT_FILES.with(|store| {
        store.borrow_mut().clear();
        for index in 0..files.length() {
            let Some(file) = files.get(index) else {
                continue;
            };
            let relative_path = webkit_relative_path(&file).unwrap_or_else(|| file.name());
            if !is_markdown_filename(&relative_path) {
                continue;
            }
            let name = file.name();
            store.borrow_mut().insert(relative_path.clone(), file);
            entries.push(ProjectEntry {
                name,
                relative_path,
            });
        }
    });
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    entries
}

pub fn load_project_file(relative_path: &str, on_loaded: impl FnOnce(Option<String>) + 'static) {
    let Some(file) = PROJECT_FILES.with(|store| store.borrow().get(relative_path).cloned()) else {
        on_loaded(None);
        return;
    };
    let reader = match FileReader::new() {
        Ok(reader) => reader,
        Err(_) => {
            on_loaded(None);
            return;
        }
    };
    let reader_clone = reader.clone();
    let callback = RefCell::new(Some(on_loaded));
    let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
        let text = reader_clone.result().ok().and_then(|r| r.as_string());
        if let Some(cb) = callback.borrow_mut().take() {
            cb(text);
        }
    }) as Box<dyn FnMut(_)>);
    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    let _ = reader.read_as_text(&file);
}

fn webkit_relative_path(file: &File) -> Option<String> {
    js_sys::Reflect::get(file, &"webkitRelativePath".into())
        .ok()
        .and_then(|v| v.as_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_name_filter() {
        assert!(is_markdown_filename("notes.md"));
        assert!(is_markdown_filename("README.MARKDOWN"));
        assert!(!is_markdown_filename("photo.png"));
    }

    #[test]
    fn folder_label_from_first_entry() {
        let entries = vec![ProjectEntry {
            name: "a.md".into(),
            relative_path: "my-project/docs/a.md".into(),
        }];
        assert_eq!(folder_label_from_entries(&entries).as_deref(), Some("my-project"));
    }
}
