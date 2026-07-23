use egui::{Context, Id};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollMetrics {
    pub id: Id,
    pub offset_y: f32,
    pub content_height: f32,
    pub viewport_height: f32,
}

pub fn scroll_ratio(offset_y: f32, content_height: f32, viewport_height: f32) -> f32 {
    let max = (content_height - viewport_height).max(1.0);
    (offset_y / max).clamp(0.0, 1.0)
}

pub fn offset_for_ratio(ratio: f32, content_height: f32, viewport_height: f32) -> f32 {
    let max = (content_height - viewport_height).max(0.0);
    ratio.clamp(0.0, 1.0) * max
}

pub fn apply_ratio(
    ctx: &Context,
    scroll_id: Id,
    ratio: f32,
    content_height: f32,
    viewport_height: f32,
) {
    crate::minimap::apply_scroll_ratio(ctx, scroll_id, ratio, content_height, viewport_height);
}

/// Keeps editor/preview scroll positions aligned by ratio mapping.
#[derive(Clone, Copy, Debug, Default)]
pub struct SyncController {
    last_editor_y: f32,
    last_preview_y: f32,
    expected_preview_y: f32,
    expected_editor_y: f32,
}

impl SyncController {
    pub fn sync(&mut self, ctx: &Context, editor: ScrollMetrics, preview: ScrollMetrics) {
        let editor_changed = (editor.offset_y - self.last_editor_y).abs() > 0.5;
        let preview_user_scroll = (preview.offset_y - self.expected_preview_y).abs() > 1.0;

        if editor_changed {
            let ratio = scroll_ratio(
                editor.offset_y,
                editor.content_height,
                editor.viewport_height,
            );
            apply_ratio(
                ctx,
                preview.id,
                ratio,
                preview.content_height,
                preview.viewport_height,
            );
            self.expected_preview_y = offset_for_ratio(
                ratio,
                preview.content_height,
                preview.viewport_height,
            );
            self.expected_editor_y = editor.offset_y;
        } else if preview_user_scroll {
            let ratio = scroll_ratio(
                preview.offset_y,
                preview.content_height,
                preview.viewport_height,
            );
            apply_ratio(
                ctx,
                editor.id,
                ratio,
                editor.content_height,
                editor.viewport_height,
            );
            self.expected_editor_y = offset_for_ratio(
                ratio,
                editor.content_height,
                editor.viewport_height,
            );
            self.expected_preview_y = preview.offset_y;
        } else {
            self.expected_preview_y = preview.offset_y;
            self.expected_editor_y = editor.offset_y;
        }

        self.last_editor_y = editor.offset_y;
        self.last_preview_y = preview.offset_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_at_middle() {
        assert!((scroll_ratio(50.0, 200.0, 100.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn offset_round_trip() {
        let y = offset_for_ratio(0.25, 400.0, 100.0);
        assert!((scroll_ratio(y, 400.0, 100.0) - 0.25).abs() < 0.001);
    }
}
