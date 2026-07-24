use leptos::html::Textarea;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use crate::settings::EditorSettings;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = omdCompressDataUrl)]
    fn omd_compress_data_url(data_url: &str, max_width: u32, quality: u8) -> js_sys::Promise;
}

pub fn insert_image_with_settings(
    current: &str,
    set_content: WriteSignal<String>,
    textarea_ref: NodeRef<Textarea>,
    settings: &EditorSettings,
    alt: &str,
    data_url: &str,
) {
    let settings = settings.clone();
    let current = current.to_string();
    let alt = alt.to_string();
    let data_url = data_url.to_string();
    if !settings.compress_images {
        super::insert_image_into(&current, set_content, &textarea_ref, &alt, &data_url);
        return;
    }
    spawn_local(async move {
        let url = compress_data_url(&data_url, settings.max_image_width, settings.image_quality)
            .await
            .unwrap_or(data_url);
        super::insert_image_into(&current, set_content, &textarea_ref, &alt, &url);
    });
}

async fn compress_data_url(data_url: &str, max_width: u32, quality: u8) -> Option<String> {
    let promise = omd_compress_data_url(data_url, max_width, quality);
    let result = JsFuture::from(promise).await.ok()?;
    result.as_string()
}
