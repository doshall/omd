use std::cell::RefCell;

use wasm_bindgen_futures::{spawn_local, JsFuture};

thread_local! {
    static CLIPBOARD_CACHE: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn remember_text(text: &str) {
    CLIPBOARD_CACHE.with(|c| *c.borrow_mut() = Some(text.to_string()));
}

pub fn cached_text() -> Option<String> {
    CLIPBOARD_CACHE.with(|c| c.borrow().clone())
}

pub fn write_text(text: &str) {
    remember_text(text);
    let owned = text.to_string();
    spawn_local(async move {
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let _ = JsFuture::from(clipboard.write_text(&owned)).await;
        }
    });
}

pub async fn read_text_async() -> Option<String> {
    let window = web_sys::window()?;
    let clipboard = window.navigator().clipboard();
    let value = JsFuture::from(clipboard.read_text()).await.ok()?;
    let text: String = value.as_string()?;
    remember_text(&text);
    Some(text)
}

pub fn refresh_cache() {
    spawn_local(async {
        let _ = read_text_async().await;
    });
}
