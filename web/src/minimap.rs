use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlTextAreaElement};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Normal,
    Heading,
    CodeFence,
    CodeBody,
    Image,
    Blockquote,
    List,
    Rule,
}

impl LineKind {
    fn rgb(self, dark: bool) -> &'static str {
        match (self, dark) {
            (LineKind::Normal, false) => "#ced4da",
            (LineKind::Normal, true) => "#495057",
            (LineKind::Heading, false) => "#4dabf7",
            (LineKind::Heading, true) => "#74c0fc",
            (LineKind::CodeFence | LineKind::CodeBody, false) => "#dee2e6",
            (LineKind::CodeFence | LineKind::CodeBody, true) => "#373a40",
            (LineKind::Image, _) => "#48a868",
            (LineKind::Blockquote, false) => "#adb5bd",
            (LineKind::Blockquote, true) => "#868e96",
            (LineKind::List, false) => "#b0b8c0",
            (LineKind::List, true) => "#5c6370",
            (LineKind::Rule, false) => "#adb5bd",
            (LineKind::Rule, true) => "#6c757d",
        }
    }
}

pub fn analyze_lines(content: &str) -> Vec<LineKind> {
    let mut in_code = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            lines.push(LineKind::CodeFence);
            continue;
        }
        if in_code {
            lines.push(LineKind::CodeBody);
            continue;
        }
        if trimmed.starts_with('#') {
            lines.push(LineKind::Heading);
        } else if trimmed.starts_with("![") {
            lines.push(LineKind::Image);
        } else if trimmed.starts_with('>') {
            lines.push(LineKind::Blockquote);
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
            || (trimmed
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
                && trimmed.contains(". "))
        {
            lines.push(LineKind::List);
        } else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            lines.push(LineKind::Rule);
        } else {
            lines.push(LineKind::Normal);
        }
    }

    if lines.is_empty() {
        lines.push(LineKind::Normal);
    }
    lines
}

const MINIMAP_WIDTH_CSS: f64 = 52.0;

pub fn paint(
    canvas: &HtmlCanvasElement,
    textarea: &HtmlTextAreaElement,
    content: &str,
    dark: bool,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let dpr = window.device_pixel_ratio();
    let css_height = textarea.client_height().max(1) as f64;
    let width_px = (MINIMAP_WIDTH_CSS * dpr).round() as u32;
    let height_px = (css_height * dpr).round() as u32;

    canvas.set_width(width_px);
    canvas.set_height(height_px);
    let _ = canvas
        .style()
        .set_property("width", &format!("{MINIMAP_WIDTH_CSS}px"));
    let _ = canvas.style().set_property("height", &format!("{css_height}px"));

    let Some(ctx) = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    else {
        return;
    };

    ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0).ok();
    let bg = if dark { "#141517" } else { "#f1f3f5" };
    ctx.set_fill_style_str(bg);
    ctx.fill_rect(0.0, 0.0, MINIMAP_WIDTH_CSS, css_height);

    let lines = analyze_lines(content);
    let line_count = lines.len().max(1);
    let row_h = (css_height / line_count as f64).max(1.0);

    for (i, kind) in lines.iter().enumerate() {
        ctx.set_fill_style_str(kind.rgb(dark));
        ctx.fill_rect(3.0, i as f64 * row_h, MINIMAP_WIDTH_CSS - 6.0, row_h);
    }

    let scroll_height = textarea.scroll_height().max(1) as f64;
    let client_height = textarea.client_height().max(1) as f64;
    let scroll_top = textarea.scroll_top() as f64;
    let scroll_max = (scroll_height - client_height).max(1.0);

    let view_h = (client_height / scroll_height * css_height).clamp(8.0, css_height);
    let view_top = (scroll_top / scroll_max * (css_height - view_h)).clamp(0.0, css_height - view_h);

    ctx.set_fill_style_str(if dark {
        "rgba(116, 192, 252, 0.18)"
    } else {
        "rgba(13, 110, 253, 0.15)"
    });
    ctx.fill_rect(1.0, view_top, MINIMAP_WIDTH_CSS - 2.0, view_h);

    ctx.set_stroke_style_str(if dark { "#74c0fc" } else { "#0d6efd" });
    ctx.set_line_width(1.5);
    ctx.stroke_rect(1.0, view_top, MINIMAP_WIDTH_CSS - 2.0, view_h);
}

pub fn scroll_from_pointer(
    canvas: &HtmlCanvasElement,
    textarea: &HtmlTextAreaElement,
    offset_y: f64,
) {
    let height = canvas.client_height().max(1) as f64;
    let ratio = (offset_y / height).clamp(0.0, 1.0);
    let scroll_max = (textarea.scroll_height() - textarea.client_height()).max(0);
    textarea.set_scroll_top((ratio * scroll_max as f64) as i32);
}
