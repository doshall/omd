use egui::{Align2, FontId, Id, Sense, Ui, Vec2};

use crate::vim_ex::BlockRect;

pub const GUTTER_WIDTH: f32 = 48.0;
const GUTTER_PAD_RIGHT: f32 = 8.0;
/// Matches egui `TextEdit` inner vertical padding.
pub const TEXTEDIT_TOP_PAD: f32 = 4.0;

pub fn line_count(content: &str) -> usize {
    if content.is_empty() {
        1
    } else {
        content.chars().filter(|&c| c == '\n').count() + 1
    }
}

pub fn line_index_at_char(content: &str, char_index: usize) -> usize {
    content
        .chars()
        .take(char_index)
        .filter(|&c| c == '\n')
        .count()
}

pub fn current_line_from_state(ctx: &egui::Context, text_edit_id: Id, content: &str) -> usize {
    if let Some(state) = egui::text_edit::TextEditState::load(ctx, text_edit_id) {
        if let Some(range) = state.cursor.char_range() {
            return line_index_at_char(content, range.primary.index);
        }
    }
    0
}

pub fn paint_block_selection(
    ui: &Ui,
    rect: BlockRect,
    gutter_width: f32,
    char_width: f32,
    top_pad: f32,
    line_height: f32,
    left_pad: f32,
) {
    let color = ui.visuals().selection.bg_fill.gamma_multiply(0.45);
    for line in rect.line_start..=rect.line_end {
        let y = top_pad + line as f32 * line_height;
        let x = gutter_width + left_pad + rect.col_start as f32 * char_width;
        let w = (rect.col_end.saturating_sub(rect.col_start)) as f32 * char_width;
        let block_rect = egui::Rect::from_min_size(
            egui::pos2(ui.min_rect().left() + x, ui.min_rect().top() + y),
            egui::vec2(w.max(char_width * 0.5), line_height),
        );
        ui.painter().rect_filled(block_rect, 0.0, color);
    }
}

pub fn paint_isearch_matches(
    ui: &Ui,
    content: &str,
    matches: &[(usize, usize)],
    current_cursor: usize,
    gutter_width: f32,
    char_width: f32,
    top_pad: f32,
    line_height: f32,
    left_pad: f32,
) {
    let dim = egui::Color32::from_rgb(230, 180, 0).gamma_multiply(0.35);
    let active = ui.visuals().selection.bg_fill.gamma_multiply(0.75);
    for (start, end) in matches {
        let start_pos = crate::vim_ex::pos_to_block_pos(content, *start);
        let line = start_pos.line;
        let y = top_pad + line as f32 * line_height;
        let x = gutter_width + left_pad + start_pos.col as f32 * char_width;
        let w = end.saturating_sub(*start).max(1) as f32 * char_width;
        let is_current = *start == current_cursor;
        let block_rect = egui::Rect::from_min_size(
            egui::pos2(ui.min_rect().left() + x, ui.min_rect().top() + y),
            egui::vec2(w, line_height),
        );
        ui.painter()
            .rect_filled(block_rect, 0.0, if is_current { active } else { dim });
    }
}

pub fn paint_current_line_highlight(
    ui: &Ui,
    current_line: usize,
    width: f32,
    top_pad: f32,
    line_height: f32,
) {
    let y = top_pad + current_line as f32 * line_height;
    let rect = egui::Rect::from_min_size(
        egui::pos2(ui.min_rect().left(), ui.min_rect().top() + y),
        egui::vec2(width, line_height),
    );
    ui.painter().rect_filled(
        rect,
        0.0,
        ui.visuals().selection.bg_fill.gamma_multiply(0.18),
    );
}

pub fn show(
    ui: &mut Ui,
    line_count: usize,
    current_line: usize,
    font_id: &FontId,
    line_height: f32,
) {
    let active_fill = ui.visuals().selection.bg_fill.gamma_multiply(0.35);
    let active_text = ui.visuals().strong_text_color();
    let inactive_text = ui.visuals().weak_text_color().gamma_multiply(0.75);
    let rows = line_count.max(1);

    ui.allocate_ui_with_layout(
        Vec2::new(GUTTER_WIDTH, rows as f32 * line_height),
        egui::Layout::top_down(egui::Align::RIGHT),
        |ui| {
            ui.set_width(GUTTER_WIDTH);
            for line in 0..rows {
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(GUTTER_WIDTH, line_height), Sense::hover());

                if line == current_line {
                    ui.painter().rect_filled(rect, 0.0, active_fill);
                }

                let color = if line == current_line {
                    active_text
                } else {
                    inactive_text
                };

                ui.painter().text(
                    rect.right_center() - Vec2::new(GUTTER_PAD_RIGHT, 0.0),
                    Align2::RIGHT_CENTER,
                    (line + 1).to_string(),
                    font_id.clone(),
                    color,
                );
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_lines() {
        assert_eq!(line_count(""), 1);
        assert_eq!(line_count("a"), 1);
        assert_eq!(line_count("a\nb"), 2);
        assert_eq!(line_count("a\nb\n"), 3);
    }

    #[test]
    fn maps_cursor_to_line() {
        let text = "foo\nbar\nbaz";
        assert_eq!(line_index_at_char(text, 0), 0);
        assert_eq!(line_index_at_char(text, 4), 1);
        assert_eq!(line_index_at_char(text, 8), 2);
    }
}
