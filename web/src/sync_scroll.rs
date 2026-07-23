use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlTextAreaElement};

pub fn scroll_ratio(scroll_top: i32, scroll_height: i32, client_height: i32) -> f64 {
    let max = (scroll_height - client_height).max(1) as f64;
    (scroll_top as f64 / max).clamp(0.0, 1.0)
}

pub fn set_scroll_from_ratio(el: &HtmlElement, ratio: f64) {
    let max = (el.scroll_height() - el.client_height()).max(0) as f64;
    let _ = el.set_scroll_top((ratio.clamp(0.0, 1.0) * max) as i32);
}

pub fn sync_editor_to_preview(ta: &HtmlTextAreaElement, preview: &HtmlElement) {
    let ratio = scroll_ratio(ta.scroll_top(), ta.scroll_height(), ta.client_height());
    set_scroll_from_ratio(preview, ratio);
}

pub fn sync_preview_to_editor(preview: &HtmlElement, ta: &HtmlTextAreaElement) {
    let ratio = scroll_ratio(
        preview.scroll_top(),
        preview.scroll_height(),
        preview.client_height(),
    );
    let max = (ta.scroll_height() - ta.client_height()).max(0) as f64;
    let _ = ta.set_scroll_top((ratio * max) as i32);
}

pub fn as_html_element(node: &web_sys::Element) -> Option<HtmlElement> {
    node.dyn_ref::<HtmlElement>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_bounds() {
        assert!((scroll_ratio(0, 100, 50) - 0.0).abs() < 0.001);
        assert!((scroll_ratio(50, 100, 50) - 1.0).abs() < 0.001);
    }
}
