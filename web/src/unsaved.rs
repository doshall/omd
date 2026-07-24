use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_namespace = window, js_name = omdSetUnsavedWarning)]
extern "C" {
    fn omd_set_unsaved_warning(enabled: bool);
}

pub fn is_modified(current: &str, baseline: &str) -> bool {
    current != baseline
}

pub fn set_unsaved_warning(modified: bool) {
    omd_set_unsaved_warning(modified);
}

pub fn confirm_discard_changes() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.confirm_with_message(
                "当前内容尚未下载保存到文件，确定放弃修改吗？",
            )
            .ok()
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_content_drift() {
        assert!(!is_modified("a", "a"));
        assert!(is_modified("a", "b"));
    }
}
