use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

pub const STORAGE_SETTINGS: &str = "omd-web-settings";

fn load_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

fn save_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item(key, value);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditorSettings {
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub show_minimap: bool,
    pub sync_scroll: bool,
    pub preview_syntax_highlight: bool,
    pub editor_syntax_highlight: bool,
    pub focus_mode: bool,
    pub editor_font_size: f32,
    pub editor_line_height: f32,
    pub preview_font_size: f32,
    pub show_undo_redo_hint: bool,
    pub keybinding_mode: crate::keybindings::KeybindingMode,
    pub vim_show_block_highlight: bool,
    pub vim_use_system_clipboard: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            highlight_current_line: true,
            show_minimap: true,
            sync_scroll: true,
            preview_syntax_highlight: true,
            editor_syntax_highlight: false,
            focus_mode: false,
            editor_font_size: 14.0,
            editor_line_height: 1.6,
            preview_font_size: 15.0,
            show_undo_redo_hint: true,
            keybinding_mode: crate::keybindings::KeybindingMode::Standard,
            vim_show_block_highlight: true,
            vim_use_system_clipboard: true,
        }
    }
}

impl EditorSettings {
    pub fn editor_line_height_px(&self) -> f64 {
        f64::from(self.editor_font_size) * f64::from(self.editor_line_height)
    }

    pub fn load() -> Self {
        load_storage(STORAGE_SETTINGS)
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            save_storage(STORAGE_SETTINGS, &json);
        }
    }
}

pub fn apply_editor_css(settings: &EditorSettings) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(root) = doc.document_element() {
            if let Ok(html) = root.dyn_into::<web_sys::HtmlElement>() {
                let style = html.style();
                let _ = style.set_property(
                    "--editor-font-size",
                    &format!("{}px", settings.editor_font_size),
                );
                let _ = style.set_property(
                    "--editor-line-height",
                    &settings.editor_line_height.to_string(),
                );
                let _ = style.set_property(
                    "--preview-font-size",
                    &format!("{}px", settings.preview_font_size),
                );
                let pad = settings.editor_font_size * 0.857;
                let _ = style.set_property("--editor-pad", &format!("{pad}px"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrip_json() {
        let s = EditorSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let parsed: EditorSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }
}
