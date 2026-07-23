use crate::markdown;
use eframe::egui;
use std::path::PathBuf;

const DEFAULT_CONTENT: &str = "# Welcome to omd

**omd** is a lightweight Markdown editor written in Rust.

## Features

- Live preview
- File open / save
- Toolbar shortcuts
- Dark & light themes

## Try it

Edit this document and see the preview update in real time.

### Code example

```rust
fn main() {
    println!(\"Hello, omd!\");
}
```

### Table

| Feature   | Status |
|-----------|--------|
| Editor    | ✅     |
| Preview   | ✅     |
| File I/O  | ✅     |

> Markdown makes writing documentation a pleasure.

- [x] Task list support
- [ ] More themes (coming soon)

---

Happy writing! 🦀
";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmdApp {
    content: String,
    file_path: Option<PathBuf>,
    modified: bool,
    dark_mode: bool,
    show_preview: bool,
    split_ratio: f32,
    status_message: String,
    status_timer: f32,
}

impl Default for OmdApp {
    fn default() -> Self {
        Self {
            content: DEFAULT_CONTENT.to_string(),
            file_path: None,
            modified: false,
            dark_mode: true,
            show_preview: true,
            split_ratio: 0.5,
            status_message: String::new(),
            status_timer: 0.0,
        }
    }
}

impl OmdApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        app.apply_theme(&cc.egui_ctx);
        app
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
        self.status_timer = 4.0;
    }

    fn title(&self) -> String {
        let name = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let modified = if self.modified { " *" } else { "" };
        format!("{name}{modified} — omd")
    }

    fn new_file(&mut self) {
        self.content = String::new();
        self.file_path = None;
        self.modified = false;
        self.set_status("New file created");
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "txt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.content = content;
                    self.file_path = Some(path.clone());
                    self.modified = false;
                    self.set_status(format!("Opened {}", path.display()));
                }
                Err(e) => {
                    self.set_status(format!("Failed to open: {e}"));
                }
            }
        }
    }

    fn save_file(&mut self) -> bool {
        if let Some(path) = &self.file_path {
            let path = path.clone();
            self.write_to_path(&path)
        } else {
            self.save_file_as()
        }
    }

    fn save_file_as(&mut self) -> bool {
        let mut dialog = rfd::FileDialog::new().add_filter("Markdown", &["md", "markdown"]);

        if let Some(path) = &self.file_path {
            dialog = dialog.set_file_name(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("document.md"),
            );
        } else {
            dialog = dialog.set_file_name("document.md");
        }

        if let Some(path) = dialog.save_file() {
            self.write_to_path(&path)
        } else {
            false
        }
    }

    fn write_to_path(&mut self, path: &PathBuf) -> bool {
        match std::fs::write(path, &self.content) {
            Ok(()) => {
                self.file_path = Some(path.clone());
                self.modified = false;
                self.set_status(format!("Saved to {}", path.display()));
                true
            }
            Err(e) => {
                self.set_status(format!("Save failed: {e}"));
                false
            }
        }
    }

    fn toolbar_button(ui: &mut egui::Ui, label: &str, tooltip: &str) -> egui::Response {
        ui.add(egui::Button::new(label).min_size(egui::vec2(32.0, 24.0)))
            .on_hover_text(tooltip)
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📄 New").clicked() {
                self.new_file();
            }
            if ui.button("📂 Open").clicked() {
                self.open_file();
            }
            if ui.button("💾 Save").clicked() {
                self.save_file();
            }
            if ui.button("💾 Save As").clicked() {
                self.save_file_as();
            }

            ui.separator();

            if Self::toolbar_button(ui, "B", "Bold (**text**)").clicked() {
                self.insert_formatting("**");
            }
            if Self::toolbar_button(ui, "I", "Italic (*text*)").clicked() {
                self.insert_formatting("*");
            }
            if Self::toolbar_button(ui, "S", "Strikethrough (~~text~~)").clicked() {
                self.insert_formatting("~~");
            }
            if Self::toolbar_button(ui, "</>", "Inline code (`code`)").clicked() {
                self.insert_formatting("`");
            }
            if Self::toolbar_button(ui, "🔗", "Link ([text](url))").clicked() {
                self.insert_formatting("[]()");
            }

            ui.separator();

            if Self::toolbar_button(ui, "H1", "Heading 1").clicked() {
                self.insert_line_prefix("# ");
            }
            if Self::toolbar_button(ui, "H2", "Heading 2").clicked() {
                self.insert_line_prefix("## ");
            }
            if Self::toolbar_button(ui, "•", "Bullet list").clicked() {
                self.insert_line_prefix("- ");
            }
            if Self::toolbar_button(ui, "1.", "Numbered list").clicked() {
                self.insert_line_prefix("1. ");
            }
            if Self::toolbar_button(ui, "❝", "Blockquote").clicked() {
                self.insert_line_prefix("> ");
            }

            ui.separator();

            if ui
                .selectable_label(self.show_preview, "👁 Preview")
                .clicked()
            {
                self.show_preview = !self.show_preview;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .selectable_label(self.dark_mode, if self.dark_mode { "🌙" } else { "☀️" })
                    .on_hover_text("Toggle dark/light theme")
                    .clicked()
                {
                    self.dark_mode = !self.dark_mode;
                    self.apply_theme(ui.ctx());
                }
            });
        });
    }

    fn insert_formatting(&mut self, wrapper: &str) {
        let len = self.content.len();
        markdown::wrap_selection(&mut self.content, 0..len, wrapper);
        self.modified = true;
    }

    fn insert_line_prefix(&mut self, prefix: &str) {
        let len = self.content.len();
        markdown::prefix_lines(&mut self.content, 0..len, prefix);
        self.modified = true;
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        let font_id = egui::FontId::monospace(14.0);
        let text_color = ui.visuals().text_color();

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.content)
                        .font(font_id)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .text_color(text_color)
                        .lock_focus(true),
                );

                if response.changed() {
                    self.modified = true;
                }
            });
    }

    fn render_preview(&self, ui: &mut egui::Ui) {
        let content = self.content.clone();
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());
                ui.add_space(8.0);
                markdown::render_preview(ui, &content);
            });
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let words = markdown::word_count(&self.content);
            let lines = markdown::line_count(&self.content);
            let chars = self.content.chars().count();

            ui.label(format!("Lines: {lines}  Words: {words}  Chars: {chars}"));

            if let Some(path) = &self.file_path {
                ui.separator();
                ui.label(path.display().to_string());
            }

            if !self.status_message.is_empty() && self.status_timer > 0.0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.status_message);
                });
            }
        });
    }
}

impl eframe::App for OmdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));
        if self.status_timer > 0.0 {
            self.status_timer -= ctx.input(|i| i.unstable_dt);
            if self.status_timer <= 0.0 {
                self.status_message.clear();
            }
        }

        ctx.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl {
                if i.key_pressed(egui::Key::S) {
                    if i.modifiers.shift {
                        self.save_file_as();
                    } else {
                        self.save_file();
                    }
                }
                if i.key_pressed(egui::Key::O) {
                    self.open_file();
                }
                if i.key_pressed(egui::Key::N) {
                    self.new_file();
                }
            }
        });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.show_preview, "Show Preview")
                        .changed()
                    {}
                    if ui.checkbox(&mut self.dark_mode, "Dark Mode").changed() {
                        self.apply_theme(ctx);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About omd").clicked() {
                        self.set_status("omd v0.1.0 — Rust Markdown Editor");
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.render_toolbar(ui);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_preview {
                let available = ui.available_width();
                let left_width = available * self.split_ratio;

                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(left_width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading("Editor");
                            ui.separator();
                            self.render_editor(ui);
                        },
                    );

                    let separator = ui.separator();
                    if separator.dragged() {
                        let delta = separator.drag_delta().x;
                        self.split_ratio =
                            ((left_width + delta) / available).clamp(0.2, 0.8);
                    }

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading("Preview");
                            ui.separator();
                            self.render_preview(ui);
                        },
                    );
                });
            } else {
                ui.heading("Editor");
                ui.separator();
                self.render_editor(ui);
            }
        });

        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
