use eframe::egui::{self, Context};
use crate::keybindings::KeybindingMode;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
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
    pub keybinding_mode: KeybindingMode,
    /// Highlight Visual Block selection (Vim mode only).
    pub vim_show_block_highlight: bool,
    /// Sync `"+` / `"*` registers with the system clipboard (Vim mode only).
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
            keybinding_mode: KeybindingMode::Standard,
            vim_show_block_highlight: true,
            vim_use_system_clipboard: true,
        }
    }
}

impl EditorSettings {
    pub fn editor_line_height_px(&self) -> f32 {
        self.editor_font_size * self.editor_line_height
    }
}

pub fn render_settings_window(ctx: &Context, open: &mut bool, settings: &mut EditorSettings) {
    let mut open_flag = *open;
    egui::Window::new("Settings")
        .open(&mut open_flag)
        .default_width(360.0)
        .show(ctx, |ui| {
            ui.heading("Editor");
            ui.checkbox(&mut settings.show_line_numbers, "Show line numbers");
            ui.checkbox(
                &mut settings.highlight_current_line,
                "Highlight current line",
            );
            ui.checkbox(&mut settings.show_minimap, "Show minimap");
            ui.add(
                egui::Slider::new(&mut settings.editor_font_size, 10.0..=24.0).text("Font size"),
            );
            ui.add(
                egui::Slider::new(&mut settings.editor_line_height, 1.2..=2.2)
                    .text("Line height"),
            );

            ui.separator();
            ui.heading("Preview");
            ui.checkbox(&mut settings.sync_scroll, "Sync scroll with editor (split view)");
            ui.checkbox(
                &mut settings.preview_syntax_highlight,
                "Syntax highlight code blocks",
            );
            ui.checkbox(
                &mut settings.editor_syntax_highlight,
                "Syntax highlight in editor",
            );
            ui.add(
                egui::Slider::new(&mut settings.preview_font_size, 12.0..=22.0)
                    .text("Preview font size"),
            );

            ui.separator();
            ui.heading("Focus & hints");
            ui.checkbox(
                &mut settings.focus_mode,
                "Focus mode (hide toolbars and preview)",
            );
            ui.label(egui::RichText::new("Shortcut: F11 toggle, Esc exit").small().weak());
            ui.checkbox(
                &mut settings.show_undo_redo_hint,
                "Show undo / redo hints in status bar",
            );

            ui.separator();
            ui.heading("Keybindings");
            egui::ComboBox::from_label("Mode")
                .selected_text(settings.keybinding_mode.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut settings.keybinding_mode,
                        KeybindingMode::Standard,
                        "Standard",
                    );
                    ui.selectable_value(&mut settings.keybinding_mode, KeybindingMode::Vim, "Vim");
                    ui.selectable_value(
                        &mut settings.keybinding_mode,
                        KeybindingMode::Emacs,
                        "Emacs",
                    );
                });
            if settings.keybinding_mode == KeybindingMode::Vim {
                ui.checkbox(
                    &mut settings.vim_show_block_highlight,
                    "Highlight Visual Block selection",
                );
                ui.checkbox(
                    &mut settings.vim_use_system_clipboard,
                    "Use system clipboard for \"+ / \"* registers",
                );
                ui.label(
                    egui::RichText::new(
                        "Vim: hjkl · Ctrl+V block · I/A/C · :g/pat/norm · :cmd · Emacs: C-Space · C-s/C-r search",
                    )
                    .small()
                    .weak(),
                );
            }
            if settings.keybinding_mode == KeybindingMode::Emacs {
                ui.label(
                    egui::RichText::new(
                        "Emacs: Ctrl+b/f/n/p/a/e · Ctrl+u prefix · Alt+b/f · M-d kill word",
                    )
                    .small()
                    .weak(),
                );
            }
        });
    *open = open_flag;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_match_features() {
        let s = EditorSettings::default();
        assert!(s.show_line_numbers);
        assert!(s.sync_scroll);
        assert!((s.editor_line_height_px() - 22.4).abs() < 0.01);
    }
}
