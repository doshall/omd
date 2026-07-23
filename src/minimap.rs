use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

/// Semantic kind of a source line for minimap coloring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub fn color(self, visuals: &egui::Visuals) -> Color32 {
        match self {
            LineKind::Normal => visuals.weak_text_color().gamma_multiply(0.35),
            LineKind::Heading => visuals.selection.bg_fill,
            LineKind::CodeFence | LineKind::CodeBody => visuals.code_bg_color,
            LineKind::Image => Color32::from_rgb(72, 160, 96),
            LineKind::Blockquote => visuals.weak_text_color().gamma_multiply(0.55),
            LineKind::List => visuals.weak_text_color().gamma_multiply(0.45),
            LineKind::Rule => visuals.widgets.noninteractive.bg_stroke.color,
        }
    }
}

/// Classify each line of a Markdown document for minimap rendering.
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

pub const MINIMAP_WIDTH: f32 = 52.0;
pub const EDITOR_LINE_HEIGHT: f32 = 22.4; // 14px × 1.6 line-height

pub enum MinimapAction {
    None,
    ScrollToRatio(f32),
}

/// Draw the minimap strip and return scroll interaction if any.
pub fn show_minimap(
    ui: &mut Ui,
    lines: &[LineKind],
    scroll_offset_y: f32,
    viewport_height: f32,
    content_height: f32,
) -> MinimapAction {
    let height = viewport_height.max(1.0);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(MINIMAP_WIDTH, height), Sense::click_and_drag());

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, ui.visuals().widgets.noninteractive.bg_stroke.color),
    );

    let line_count = lines.len().max(1);
    let row_h = (height / line_count as f32).max(1.0);

    for (i, kind) in lines.iter().enumerate() {
        let y = rect.top() + i as f32 * row_h;
        let row_rect = Rect::from_min_size(Pos2::new(rect.left() + 2.0, y), Vec2::new(rect.width() - 4.0, row_h));
        painter.rect_filled(row_rect, 0.0, kind.color(ui.visuals()));
    }

    let scroll_max = (content_height - viewport_height).max(1.0);
    let view_h = (viewport_height / content_height.max(1.0) * height).clamp(8.0, height);
    let view_top = (scroll_offset_y / scroll_max * (height - view_h)).clamp(0.0, height - view_h);
    let viewport_rect = Rect::from_min_size(
        Pos2::new(rect.left() + 1.0, rect.top() + view_top),
        Vec2::new(rect.width() - 2.0, view_h),
    );
    painter.rect_stroke(
        viewport_rect,
        1.0,
        Stroke::new(
            1.5_f32,
            ui.visuals().selection.stroke.color.gamma_multiply(0.9),
        ),
    );
    painter.rect_filled(
        viewport_rect,
        1.0,
        ui.visuals().selection.bg_fill.gamma_multiply(0.15),
    );

    if let Some(pos) = response.interact_pointer_pos() {
        if response.clicked() || response.dragged() {
            let ratio = ((pos.y - rect.top()) / height).clamp(0.0, 1.0);
            return MinimapAction::ScrollToRatio(ratio);
        }
    }

    MinimapAction::None
}

pub fn content_height_from_lines(line_count: usize) -> f32 {
    line_count.max(1) as f32 * EDITOR_LINE_HEIGHT
}

pub fn apply_scroll_ratio(
    ctx: &egui::Context,
    scroll_id: egui::Id,
    ratio: f32,
    content_height: f32,
    viewport_height: f32,
) {
    let scroll_max = (content_height - viewport_height).max(0.0);
    let mut state = egui::scroll_area::State::load(ctx, scroll_id).unwrap_or_default();
    state.offset.y = ratio * scroll_max;
    state.store(ctx, scroll_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_headings_and_code() {
        let md = "# Title\n\n```rust\nfn main() {}\n```\n";
        let lines = analyze_lines(md);
        assert_eq!(lines[0], LineKind::Heading);
        assert_eq!(lines[2], LineKind::CodeFence);
        assert_eq!(lines[3], LineKind::CodeBody);
        assert_eq!(lines[4], LineKind::CodeFence);
    }
}
