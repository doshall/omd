use eframe::egui::{self, Context};
use crate::keybindings::KeybindingMode;
use omd_common::{t, Locale};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(default)]
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
    /// Periodically save open files to disk when they have unsaved changes.
    pub auto_save_enabled: bool,
    /// Seconds of inactivity before auto-save writes to disk.
    pub auto_save_interval_secs: u32,
    /// Show auto-generated table of contents in preview and export.
    pub show_toc: bool,
    /// Parse and render Markdown footnotes.
    pub enable_footnotes: bool,
    /// Compress pasted/uploaded images before embedding as data URLs.
    pub compress_images: bool,
    /// Max width in pixels when compressing images.
    pub max_image_width: u32,
    /// JPEG quality (1–100) when compressing images.
    pub image_quality: u8,
    /// Keep running in the system tray when the window is closed.
    pub minimize_to_tray_on_close: bool,
    /// Register global shortcuts (Ctrl+Shift+O show, Ctrl+Shift+N new).
    pub enable_global_shortcuts: bool,
    /// Interface language for menus and settings.
    pub locale: Locale,
    /// Custom CSS injected into exported HTML.
    pub custom_preview_css: String,
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
            auto_save_enabled: true,
            auto_save_interval_secs: 30,
            show_toc: true,
            enable_footnotes: true,
            compress_images: true,
            max_image_width: 1920,
            image_quality: 85,
            minimize_to_tray_on_close: true,
            enable_global_shortcuts: true,
            locale: Locale::default(),
            custom_preview_css: String::new(),
        }
    }
}

impl EditorSettings {
    pub fn editor_line_height_px(&self) -> f32 {
        self.editor_font_size * self.editor_line_height
    }
}

pub fn render_settings_window(ctx: &Context, open: &mut bool, settings: &mut EditorSettings) {
    let locale = settings.locale;
    let mut open_flag = *open;
    egui::Window::new(t(locale, "settings"))
        .open(&mut open_flag)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.heading(t(locale, "section_appearance"));
            egui::ComboBox::from_label(t(locale, "locale"))
                .selected_text(match settings.locale {
                    Locale::Zh => t(locale, "locale_zh"),
                    Locale::En => t(locale, "locale_en"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.locale, Locale::Zh, t(locale, "locale_zh"));
                    ui.selectable_value(&mut settings.locale, Locale::En, t(locale, "locale_en"));
                });
            ui.label(t(locale, "custom_preview_css"));
            ui.add(
                egui::TextEdit::multiline(&mut settings.custom_preview_css)
                    .desired_rows(4)
                    .font(egui::TextStyle::Monospace),
            );
            ui.label(
                egui::RichText::new(t(locale, "custom_preview_css_hint"))
                    .small()
                    .weak(),
            );

            ui.separator();
            ui.heading(t(locale, "section_editor"));
            ui.checkbox(&mut settings.show_line_numbers, t(locale, "show_line_numbers"));
            ui.checkbox(
                &mut settings.highlight_current_line,
                t(locale, "highlight_current_line"),
            );
            ui.checkbox(&mut settings.show_minimap, t(locale, "show_minimap"));
            ui.add(
                egui::Slider::new(&mut settings.editor_font_size, 10.0..=24.0)
                    .text(t(locale, "font_size")),
            );
            ui.add(
                egui::Slider::new(&mut settings.editor_line_height, 1.2..=2.2)
                    .text(t(locale, "line_height")),
            );

            ui.separator();
            ui.heading(t(locale, "section_preview"));
            ui.checkbox(&mut settings.sync_scroll, t(locale, "sync_scroll"));
            ui.checkbox(
                &mut settings.preview_syntax_highlight,
                t(locale, "preview_syntax_highlight"),
            );
            ui.checkbox(
                &mut settings.editor_syntax_highlight,
                t(locale, "editor_syntax_highlight"),
            );
            ui.add(
                egui::Slider::new(&mut settings.preview_font_size, 12.0..=22.0)
                    .text(t(locale, "preview_font_size")),
            );
            ui.checkbox(&mut settings.show_toc, t(locale, "show_toc"));
            ui.checkbox(&mut settings.enable_footnotes, t(locale, "enable_footnotes"));

            ui.separator();
            ui.heading(t(locale, "section_images"));
            ui.checkbox(&mut settings.compress_images, t(locale, "compress_images"));
            ui.add_enabled(
                settings.compress_images,
                egui::Slider::new(&mut settings.max_image_width, 320..=4096)
                    .logarithmic(true)
                    .text("Max width (px)"),
            );
            ui.add_enabled(
                settings.compress_images,
                egui::Slider::new(&mut settings.image_quality, 40..=100).text("JPEG quality"),
            );

            ui.separator();
            ui.heading(t(locale, "section_files"));
            ui.checkbox(&mut settings.auto_save_enabled, t(locale, "auto_save"));
            ui.add_enabled(
                settings.auto_save_enabled,
                egui::Slider::new(&mut settings.auto_save_interval_secs, 5..=300)
                    .logarithmic(true)
                    .text(t(locale, "auto_save_delay")),
            );
            ui.label(
                egui::RichText::new(t(locale, "auto_save_hint"))
                    .small()
                    .weak(),
            );

            ui.separator();
            ui.heading(t(locale, "section_desktop"));
            ui.checkbox(
                &mut settings.minimize_to_tray_on_close,
                t(locale, "minimize_to_tray"),
            );
            ui.checkbox(
                &mut settings.enable_global_shortcuts,
                t(locale, "global_shortcuts"),
            );
            ui.label(
                egui::RichText::new(t(locale, "tray_hint"))
                    .small()
                    .weak(),
            );

            ui.separator();
            ui.heading(t(locale, "section_focus"));
            ui.checkbox(
                &mut settings.focus_mode,
                t(locale, "focus_mode"),
            );
            ui.label(
                egui::RichText::new(t(locale, "focus_mode_hint"))
                    .small()
                    .weak(),
            );
            ui.checkbox(
                &mut settings.show_undo_redo_hint,
                t(locale, "undo_redo_hint"),
            );

            ui.separator();
            ui.heading(t(locale, "keybinding_mode"));
            egui::ComboBox::from_label(t(locale, "keybinding_mode"))
                .selected_text(settings.keybinding_mode.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut settings.keybinding_mode,
                        KeybindingMode::Standard,
                        t(locale, "keybinding_standard"),
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
                    t(locale, "vim_block_highlight"),
                );
                ui.checkbox(
                    &mut settings.vim_use_system_clipboard,
                    t(locale, "vim_system_clipboard"),
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
