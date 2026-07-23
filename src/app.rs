use crate::clipboard;
use crate::find_replace::{self, FindBarState};
use crate::keybindings::{self, KeybindingMode, KeybindingState, VimMode};
use crate::line_gutter;
use crate::markdown::{self, PreviewContext};
use crate::mermaid::MermaidCache;
use crate::minimap::{self, MinimapAction};
use crate::editor_highlight;
use crate::settings::{self, EditorSettings};
use crate::sync_scroll::{ScrollMetrics, SyncController};
use eframe::egui;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnsavedPrompt {
    Close,
    New,
    Open,
}

enum UnsavedDialogAction {
    Save,
    Discard,
    Cancel,
}

const DEFAULT_CONTENT: &str = r#"# omd 桌面版功能演示

欢迎使用 **omd** 桌面版 Markdown 编辑器！本文档展示全部功能，可直接编辑体验。

---

## 1. 文本格式

| 格式 | 语法 | 效果 |
|------|------|------|
| 粗体 | `**粗体**` | **粗体** |
| 斜体 | `*斜体*` | *斜体* |
| 删除线 | `~~删除~~` | ~~删除~~ |
| 行内代码 | `` `code` `` | `code` |
| 链接 | `[文字](url)` | [Rust 官网](https://www.rust-lang.org) |

> 工具栏：**B** 粗体 · **I** 斜体 · **S** 删除线 · **</>** 代码 · **🔗** 链接

---

## 2. 标题与结构

### 三级标题
#### 四级标题

- 无序列表项 A
- 无序列表项 B

1. 有序列表第一步
2. 有序列表第二步

> 引用块：Markdown 让写作更高效。工具栏 **❝** 可快速插入引用。

---

## 3. 任务列表

- [x] 实时分栏预览
- [x] 文件新建 / 打开 / 保存
- [x] 本地图片插入与预览
- [x] Mermaid 图表渲染
- [x] 剪贴板粘贴图片（`Ctrl+V`）
- [x] 拖拽插入图片
- [x] 深色 / 浅色主题

---

## 4. 代码块

```rust
fn main() {
    println!("Hello, omd!");
}
```

---

## 5. Mermaid 图表

```mermaid
flowchart LR
    A[编辑] --> B[预览]
    B --> C[保存]
```

---

## 6. 表格

| 功能 | 快捷键 / 操作 | 说明 |
|------|---------------|------|
| 新建 | `Ctrl+N` / 菜单 | 创建空白文档 |
| 打开 | `Ctrl+O` / 📂 | 打开 `.md` 文件 |
| 保存 | `Ctrl+S` / 💾 | 保存当前文件 |
| 另存为 | `Ctrl+Shift+S` | 保存到新路径 |
| 插入图片 | 工具栏 🖼 / 拖拽 | 选择本地图片或拖入编辑区 |
| 粘贴图片 | `Ctrl+V` | 从剪贴板粘贴截图 |
| 查找 | `Ctrl+F` | 打开查找栏 |
| 替换 | `Ctrl+H` | 打开查找替换栏 |
| 切换主题 | 工具栏 🌙 / ☀️ | 深色 / 浅色模式 |
| 预览开关 | 工具栏 👁 | 显示 / 隐藏预览区 |

---

## 7. 图片

### 网络图片
![Rust Logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg)

### 本地图片
点击工具栏 **🖼**，选择本地图片文件，将自动插入 `![文件名](路径)` 并在预览区渲染。

支持格式：PNG、JPG、GIF、WebP、SVG、BMP

---

## 8. 菜单栏

- **File** — 新建、打开、保存、另存为、退出
- **View** — 显示预览、深色模式
- **Help** — 关于 omd

拖拽中间分隔线可调整编辑区与预览区宽度。

---

**开始编辑吧！** 修改任意文字，预览区即时更新。🦀
"#;

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
    editor_settings: EditorSettings,
    #[serde(skip)]
    settings_open: bool,
    #[serde(skip)]
    mermaid_cache: MermaidCache,
    #[serde(skip)]
    find_bar: FindBarState,
    #[serde(skip)]
    sync_scroll: SyncController,
    #[serde(skip)]
    keybinding_state: KeybindingState,
    #[serde(skip)]
    editor_text_edit_id: Option<egui::Id>,
    #[serde(skip)]
    last_keybinding_mode: KeybindingMode,
    #[serde(skip)]
    unsaved_prompt: Option<UnsavedPrompt>,
    #[serde(skip)]
    auto_save_timer: f32,
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
            editor_settings: EditorSettings::default(),
            settings_open: false,
            mermaid_cache: MermaidCache::default(),
            find_bar: FindBarState::default(),
            sync_scroll: SyncController::default(),
            keybinding_state: KeybindingState::default(),
            editor_text_edit_id: None,
            last_keybinding_mode: KeybindingMode::Standard,
            unsaved_prompt: None,
            auto_save_timer: 0.0,
        }
    }
}

impl OmdApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

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

    fn mark_modified(&mut self) {
        self.modified = true;
        self.auto_save_timer = 0.0;
    }

    fn request_new_file(&mut self) {
        if self.modified {
            self.unsaved_prompt = Some(UnsavedPrompt::New);
        } else {
            self.new_file();
        }
    }

    fn request_open_file(&mut self) {
        if self.modified {
            self.unsaved_prompt = Some(UnsavedPrompt::Open);
        } else {
            self.open_file();
        }
    }

    fn request_close(&mut self, ctx: &egui::Context) {
        if self.modified {
            self.unsaved_prompt = Some(UnsavedPrompt::Close);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn apply_unsaved_prompt(&mut self, ctx: &egui::Context, prompt: UnsavedPrompt) {
        match prompt {
            UnsavedPrompt::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            UnsavedPrompt::New => self.new_file(),
            UnsavedPrompt::Open => self.open_file(),
        }
    }

    fn fulfill_unsaved_prompt(&mut self, ctx: &egui::Context, action: UnsavedDialogAction) {
        let Some(prompt) = self.unsaved_prompt.take() else {
            return;
        };
        match action {
            UnsavedDialogAction::Save => {
                if !self.save_file() {
                    self.unsaved_prompt = Some(prompt);
                    return;
                }
                self.apply_unsaved_prompt(ctx, prompt);
            }
            UnsavedDialogAction::Discard => self.apply_unsaved_prompt(ctx, prompt),
            UnsavedDialogAction::Cancel => {}
        }
    }

    fn render_unsaved_dialog(&mut self, ctx: &egui::Context) {
        let Some(prompt) = self.unsaved_prompt else {
            return;
        };

        let message = match prompt {
            UnsavedPrompt::Close => "Save changes before closing?",
            UnsavedPrompt::New => "Save changes before creating a new file?",
            UnsavedPrompt::Open => "Save changes before opening another file?",
        };

        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(message);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.fulfill_unsaved_prompt(ctx, UnsavedDialogAction::Save);
                    }
                    if ui.button("Don't Save").clicked() {
                        self.fulfill_unsaved_prompt(ctx, UnsavedDialogAction::Discard);
                    }
                    if ui.button("Cancel").clicked() {
                        self.fulfill_unsaved_prompt(ctx, UnsavedDialogAction::Cancel);
                    }
                });
            });
    }

    fn tick_auto_save(&mut self, dt: f32) {
        if !self.editor_settings.auto_save_enabled || !self.modified || self.file_path.is_none() {
            return;
        }

        self.auto_save_timer += dt;
        let interval = self.editor_settings.auto_save_interval_secs.max(1) as f32;
        if self.auto_save_timer < interval {
            return;
        }

        if let Some(path) = self.file_path.clone() {
            if self.write_to_path(&path) {
                self.set_status(format!("Auto-saved to {}", path.display()));
            }
        }
        self.auto_save_timer = 0.0;
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<egui::DroppedFile> =
            ctx.input(|i| i.raw.dropped_files.clone());
        if dropped.is_empty() {
            return;
        }

        let cursor = self.editor_cursor(ctx);
        let mut next_cursor = cursor;
        let mut inserted = 0usize;

        for file in dropped {
            if let Some(path) = file.path {
                if !markdown::is_image_path(&path) {
                    continue;
                }
                let path_str = path.display().to_string();
                let alt = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image");
                let md = markdown::image_markdown(alt, &path_str);
                self.content = markdown::insert_at_cursor(&self.content, next_cursor, &md);
                next_cursor += md.len();
                inserted += 1;
            }
        }

        if inserted > 0 {
            self.mark_modified();
            self.set_status(format!("Inserted {inserted} image(s)"));
        }
    }

    fn insert_image_markdown(&mut self, ctx: &egui::Context, alt: &str, url: &str) {
        let cursor = self.editor_cursor(ctx);
        let md = markdown::image_markdown(alt, url);
        self.content = markdown::insert_at_cursor(&self.content, cursor, &md);
        self.mark_modified();
        self.set_status(format!("Inserted image: {url}"));
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
            if Self::toolbar_button(ui, "🖼", "Image (![alt](path))").clicked() {
                self.insert_image(ui.ctx());
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
        self.mark_modified();
    }

    fn insert_line_prefix(&mut self, prefix: &str) {
        let len = self.content.len();
        markdown::prefix_lines(&mut self.content, 0..len, prefix);
        self.mark_modified();
    }

    fn insert_image(&mut self, ctx: &egui::Context) {
        let mut dialog = rfd::FileDialog::new().add_filter(
            "Images",
            &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"],
        );
        if let Some(dir) = self.file_path.as_ref().and_then(|p| p.parent()) {
            dialog = dialog.set_directory(dir);
        }
        if let Some(path) = dialog.pick_file() {
            let path_str = path.display().to_string();
            let alt = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            self.insert_image_markdown(ctx, alt, &path_str);
        }
    }

    fn try_paste_image(&mut self, ctx: &egui::Context) -> bool {
        if let Some(data_url) = clipboard::clipboard_image_data_url() {
            self.insert_image_markdown(ctx, "image", &data_url);
            self.set_status("Pasted image from clipboard");
            return true;
        }
        false
    }

    fn editor_cursor(&self, ctx: &egui::Context) -> usize {
        if let Some(id) = self.editor_text_edit_id {
            if let Some(state) = egui::text_edit::TextEditState::load(ctx, id) {
                if let Some(range) = state.cursor.char_range() {
                    return range.primary.index;
                }
            }
        }
        0
    }

    fn apply_key_action(&mut self, ctx: &egui::Context, action: keybindings::KeyAction) {
        if action.content_changed {
            self.mark_modified();
        }
        if let Some(status) = action.status {
            self.set_status(status);
        }
        if let Some(result) = action.command_result {
            if result.request_save {
                self.save_file();
            }
            if let Some(show) = result.line_numbers {
                self.editor_settings.show_line_numbers = show;
            }
        }
        if let Some(id) = self.editor_text_edit_id {
            if let Some(mut state) = egui::text_edit::TextEditState::load(ctx, id) {
                let sel = action
                    .selection
                    .map(|(a, b)| {
                        egui::text::CCursorRange::two(
                            egui::text::CCursor::new(a),
                            egui::text::CCursor::new(b),
                        )
                    })
                    .unwrap_or_else(|| {
                        egui::text::CCursorRange::one(egui::text::CCursor::new(action.cursor))
                    });
                state.cursor.set_char_range(Some(sel));
                state.store(ctx, id);
            }
        }
    }

    fn handle_find_bar_actions(&mut self, actions: find_replace::FindBarOutput) {
        if actions.close {
            self.find_bar.close();
            return;
        }
        if actions.find_next {
            if !self.find_bar.advance_match(&self.content, true) {
                self.set_status("No matches found");
            }
        }
        if actions.find_prev {
            if !self.find_bar.advance_match(&self.content, false) {
                self.set_status("No matches found");
            }
        }
        if actions.replace_one {
            let idx = self.find_bar.match_index;
            if self.find_bar.query.is_empty() {
                return;
            }
            let replacement = self.find_bar.replace.clone();
            if let Some((start, end)) = find_replace::replace_at(
                &mut self.content,
                &self.find_bar.query,
                &replacement,
                self.find_bar.case_sensitive,
                idx,
            ) {
                self.mark_modified();
                self.find_bar.pending_selection = Some((start, end));
                self.find_bar.match_index = idx.min(
                    self.find_bar.match_ranges(&self.content).len().saturating_sub(1),
                );
                self.set_status("Replaced 1 occurrence");
            } else {
                self.set_status("No matches found");
            }
        }
        if actions.replace_all {
            if self.find_bar.query.is_empty() {
                return;
            }
            let replacement = self.find_bar.replace.clone();
            let count = find_replace::replace_all(
                &mut self.content,
                &self.find_bar.query,
                &replacement,
                self.find_bar.case_sensitive,
            );
            if count > 0 {
                self.mark_modified();
            }
            self.find_bar.reset_match_index();
            self.set_status(format!("Replaced {count} occurrence(s)"));
        }
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) -> ScrollMetrics {
        let line_height = self.editor_settings.editor_line_height_px();
        let font_id = egui::FontId::monospace(self.editor_settings.editor_font_size);
        let highlight_syntax = self.editor_settings.editor_syntax_highlight;
        let text_color = if highlight_syntax {
            egui::Color32::TRANSPARENT
        } else {
            ui.visuals().text_color()
        };
        let available_height = ui.available_height();
        let line_kinds = minimap::analyze_lines(&self.content);
        let content_height = line_kinds.len().max(1) as f32 * line_height;
        let line_count = line_gutter::line_count(&self.content);
        let text_edit_id = ui.id().with(find_replace::EDITOR_ID_SALT);
        let current_line =
            line_gutter::current_line_from_state(ui.ctx(), text_edit_id, &self.content);

        let mut scroll_id = egui::Id::NULL;
        let mut scroll_offset_y = 0.0_f32;
        let mut viewport_height = available_height;
        let mut content_size_y = content_height;

        let minimap_reserve = if self.editor_settings.show_minimap {
            minimap::MINIMAP_WIDTH + 4.0
        } else {
            0.0
        };

        ui.horizontal(|ui| {
            let editor_width = (ui.available_width() - minimap_reserve).max(120.0);

            let scroll = ui
                .allocate_ui_with_layout(
                    egui::vec2(editor_width, available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("omd_editor_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let content_width = ui.available_width();
                                if self.editor_settings.highlight_current_line {
                                    line_gutter::paint_current_line_highlight(
                                        ui,
                                        current_line,
                                        content_width,
                                        line_gutter::TEXTEDIT_TOP_PAD,
                                        line_height,
                                    );
                                }

                                if self.editor_settings.keybinding_mode == KeybindingMode::Vim
                                    && self.editor_settings.vim_show_block_highlight
                                    && self.keybinding_state.vim_mode == VimMode::VisualBlock
                                {
                                    if let Some(block) = self.keybinding_state.active_block {
                                        let gutter = if self.editor_settings.show_line_numbers {
                                            line_gutter::GUTTER_WIDTH
                                        } else {
                                            0.0
                                        };
                                        let char_width = self.editor_settings.editor_font_size;
                                        line_gutter::paint_block_selection(
                                            ui,
                                            block,
                                            gutter,
                                            char_width,
                                            line_gutter::TEXTEDIT_TOP_PAD,
                                            line_height,
                                            4.0,
                                        );
                                    }
                                }

                                if self.editor_settings.keybinding_mode == KeybindingMode::Emacs {
                                    if let Some(ref isearch) = self.keybinding_state.emacs_isearch {
                                        if !isearch.query.is_empty() {
                                            let cursor_char = egui::text_edit::TextEditState::load(
                                                ui.ctx(),
                                                text_edit_id,
                                            )
                                            .and_then(|s| s.cursor.char_range())
                                            .map(|r| r.primary.index)
                                            .unwrap_or(0);
                                            let matches = keybindings::isearch_all_matches(
                                                &self.content,
                                                &isearch.query,
                                            );
                                            let gutter = if self.editor_settings.show_line_numbers
                                            {
                                                line_gutter::GUTTER_WIDTH
                                            } else {
                                                0.0
                                            };
                                            let char_width = self.editor_settings.editor_font_size;
                                            line_gutter::paint_isearch_matches(
                                                ui,
                                                &self.content,
                                                &matches,
                                                cursor_char,
                                                gutter,
                                                char_width,
                                                line_gutter::TEXTEDIT_TOP_PAD,
                                                line_height,
                                                4.0,
                                            );
                                        }
                                    }
                                }

                                ui.horizontal_top(|ui| {
                                    if self.editor_settings.show_line_numbers {
                                        line_gutter::show(
                                            ui,
                                            line_count,
                                            current_line,
                                            &font_id,
                                            line_height,
                                        );
                                    }

                                    if self.last_keybinding_mode != self.editor_settings.keybinding_mode {
                                        keybindings::reset_for_mode(
                                            &mut self.keybinding_state,
                                            self.editor_settings.keybinding_mode,
                                        );
                                        self.last_keybinding_mode = self.editor_settings.keybinding_mode;
                                    }
                                    self.keybinding_state.use_system_clipboard =
                                        self.editor_settings.vim_use_system_clipboard;

                                    if self.editor_settings.keybinding_mode != KeybindingMode::Standard
                                        && !self.find_bar.open
                                    {
                                        if let Some(action) = keybindings::process_egui_input(
                                            ui.ctx(),
                                            &mut self.content,
                                            text_edit_id,
                                            self.editor_settings.keybinding_mode,
                                            &mut self.keybinding_state,
                                        ) {
                                            self.apply_key_action(ui.ctx(), action);
                                        }
                                    }

                                    let editor_inner_height = line_count.max(1) as f32 * line_height
                                        + line_gutter::TEXTEDIT_TOP_PAD * 2.0;
                                    let editor_width = ui.available_width();
                                    let (editor_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(editor_width, editor_inner_height),
                                        egui::Sense::click(),
                                    );

                                    if highlight_syntax {
                                        let visuals = ui.visuals().clone();
                                        let origin = editor_rect.min
                                            + egui::vec2(
                                                line_gutter::TEXTEDIT_TOP_PAD,
                                                line_gutter::TEXTEDIT_TOP_PAD,
                                            );
                                        ui.fonts(|fonts| {
                                            editor_highlight::paint_backdrop_in_rect(
                                                &ui.painter().with_clip_rect(editor_rect),
                                                editor_rect,
                                                origin,
                                                &self.content,
                                                &font_id,
                                                line_height,
                                                self.dark_mode,
                                                &visuals,
                                                &|text, font, color| {
                                                    fonts.layout_no_wrap(text, font, color)
                                                },
                                            );
                                        });
                                    }

                                    let response = ui.put(
                                        editor_rect,
                                        egui::TextEdit::multiline(&mut self.content)
                                            .id_salt(find_replace::EDITOR_ID_SALT)
                                            .font(font_id.clone())
                                            .desired_width(f32::INFINITY)
                                            .lock_focus(true)
                                            .text_color(text_color)
                                            .frame(true)
                                            .margin(egui::Margin::same(line_gutter::TEXTEDIT_TOP_PAD)),
                                    );

                                    self.editor_text_edit_id = Some(response.id);

                                    if response.changed() {
                                        self.mark_modified();
                                    }

                                    if let Some((start, end)) =
                                        self.find_bar.pending_selection.take()
                                    {
                                        let mut state = egui::text_edit::TextEditState::load(
                                            ui.ctx(),
                                            response.id,
                                        )
                                        .unwrap_or_default();
                                        state.cursor.set_char_range(Some(
                                            egui::text::CCursorRange::two(
                                                egui::text::CCursor::new(start),
                                                egui::text::CCursor::new(end),
                                            ),
                                        ));
                                        state.store(ui.ctx(), response.id);
                                    }
                                });
                            })
                    },
                )
                .inner;

            scroll_id = scroll.id;
            scroll_offset_y = scroll.state.offset.y;
            viewport_height = scroll.inner_rect.height();
            content_size_y = scroll.content_size.y;

            if self.editor_settings.show_minimap {
                let minimap_action = ui
                    .allocate_ui_with_layout(
                        egui::vec2(minimap::MINIMAP_WIDTH, available_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            minimap::show_minimap(
                                ui,
                                &line_kinds,
                                scroll_offset_y,
                                viewport_height,
                                content_height,
                            )
                        },
                    )
                    .inner;

                if let MinimapAction::ScrollToRatio(ratio) = minimap_action {
                    minimap::apply_scroll_ratio(
                        ui.ctx(),
                        scroll_id,
                        ratio,
                        content_height,
                        viewport_height,
                    );
                }
            }
        });

        ScrollMetrics {
            id: scroll_id,
            offset_y: scroll_offset_y,
            content_height: content_size_y.max(viewport_height),
            viewport_height,
        }
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) -> ScrollMetrics {
        let content = self.content.clone();
        let base_path = self.file_path.as_ref().and_then(|p| p.parent());
        let mut ctx = PreviewContext {
            dark_mode: self.dark_mode,
            base_path,
            mermaid_cache: &mut self.mermaid_cache,
            preview_syntax_highlight: self.editor_settings.preview_syntax_highlight,
            preview_font_size: self.editor_settings.preview_font_size,
        };
        let scroll = egui::ScrollArea::both()
            .id_salt("omd_preview_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());
                ui.add_space(8.0);
                markdown::render_preview(ui, &content, &mut ctx);
            });

        ScrollMetrics {
            id: scroll.id,
            offset_y: scroll.state.offset.y,
            content_height: scroll.content_size.y.max(scroll.inner_rect.height()),
            viewport_height: scroll.inner_rect.height(),
        }
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let words = markdown::word_count(&self.content);
            let lines = markdown::line_count(&self.content);
            let chars = self.content.chars().count();

            ui.label(format!("Lines: {lines}  Words: {words}  Chars: {chars}"));

            if self.editor_settings.keybinding_mode == KeybindingMode::Vim {
                ui.separator();
                let mut label = format!("Vim: {}", self.keybinding_state.vim_mode.label());
                if self.keybinding_state.count > 0 {
                    label.push_str(&format!(" · {}", self.keybinding_state.count));
                }
                if let Some(reg) = self.keybinding_state.macro_recording {
                    label.push_str(&format!(" · REC @{reg}"));
                }
                ui.label(label);
            } else if self.editor_settings.keybinding_mode == KeybindingMode::Emacs {
                ui.separator();
                let label = if let Some(ref isearch) = self.keybinding_state.emacs_isearch {
                    let dir = if isearch.forward {
                        "I-search"
                    } else {
                        "I-search backward"
                    };
                    format!("{dir}: {}", isearch.query)
                } else {
                    "Emacs".to_string()
                };
                ui.label(label);
            }

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

        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && self.modified {
            if self.unsaved_prompt.is_none() {
                self.unsaved_prompt = Some(UnsavedPrompt::Close);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        self.handle_dropped_files(ctx);
        self.tick_auto_save(ctx.input(|i| i.unstable_dt));

        if self.status_timer > 0.0 {
            self.status_timer -= ctx.input(|i| i.unstable_dt);
            if self.status_timer <= 0.0 {
                self.status_message.clear();
            }
        }

        let mut find_next = false;
        let mut find_prev = false;

        ctx.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl {
                if i.key_pressed(egui::Key::V) {
                    self.try_paste_image(ctx);
                }
                if i.key_pressed(egui::Key::F) {
                    self.find_bar.open_find(false);
                }
                if i.key_pressed(egui::Key::H) {
                    self.find_bar.open_find(true);
                }
                if i.key_pressed(egui::Key::S) {
                    if i.modifiers.shift {
                        self.save_file_as();
                    } else {
                        self.save_file();
                    }
                }
                if i.key_pressed(egui::Key::O) {
                    self.request_open_file();
                }
                if i.key_pressed(egui::Key::N) {
                    self.request_new_file();
                }
                if self.editor_settings.show_undo_redo_hint {
                    if i.key_pressed(egui::Key::Z) {
                        if i.modifiers.shift {
                            self.set_status("Redo");
                        } else {
                            self.set_status("Undo");
                        }
                    }
                    if i.key_pressed(egui::Key::Y) {
                        self.set_status("Redo");
                    }
                }
            }
            if i.key_pressed(egui::Key::F11) {
                self.editor_settings.focus_mode = !self.editor_settings.focus_mode;
            }
            if self.editor_settings.focus_mode && i.key_pressed(egui::Key::Escape) {
                self.editor_settings.focus_mode = false;
            }
            if self.find_bar.open {
                if i.key_pressed(egui::Key::Escape) {
                    self.find_bar.close();
                }
                if i.key_pressed(egui::Key::Enter) {
                    if i.modifiers.shift {
                        find_prev = true;
                    } else {
                        find_next = true;
                    }
                }
            }
            if i.key_pressed(egui::Key::F3) {
                if i.modifiers.shift {
                    find_prev = true;
                } else {
                    find_next = true;
                }
            }
        });

        if find_next {
            if !self.find_bar.advance_match(&self.content, true) {
                self.set_status("No matches found");
            }
        }
        if find_prev {
            if !self.find_bar.advance_match(&self.content, false) {
                self.set_status("No matches found");
            }
        }

        let focus_mode = self.editor_settings.focus_mode;
        let show_chrome = !focus_mode;

        if show_chrome {
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            self.request_new_file();
                            ui.close_menu();
                        }
                        if ui.button("Open…").clicked() {
                            self.request_open_file();
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
                            self.request_close(ctx);
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        if ui.button("Find…").clicked() {
                            self.find_bar.open_find(false);
                            ui.close_menu();
                        }
                        if ui.button("Replace…").clicked() {
                            self.find_bar.open_find(true);
                            ui.close_menu();
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
                        ui.separator();
                        if ui.button("Settings…").clicked() {
                            self.settings_open = true;
                            ui.close_menu();
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
        }

        if show_chrome {
            egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                self.render_toolbar(ui);
            });
        }

        if show_chrome && self.find_bar.open {
            egui::TopBottomPanel::top("find_bar").show(ctx, |ui| {
                let actions = find_replace::render_find_bar(ui, &mut self.find_bar, &self.content);
                self.handle_find_bar_actions(actions);
            });
        }

        if show_chrome
            && self.editor_settings.keybinding_mode == KeybindingMode::Vim
            && self.keybinding_state.vim_mode == keybindings::VimMode::Command
        {
            let cursor = self.editor_cursor(ctx);
            egui::TopBottomPanel::top("vim_command_bar").show(ctx, |ui| {
                if let Some(action) = keybindings::render_vim_command_bar(
                    ui,
                    &mut self.keybinding_state,
                    &mut self.content,
                    cursor,
                ) {
                    self.apply_key_action(ctx, action);
                }
            });
        }

        if show_chrome {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                self.render_status_bar(ui);
            });
        }

        settings::render_settings_window(ctx, &mut self.settings_open, &mut self.editor_settings);

        self.render_unsaved_dialog(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if focus_mode {
                self.render_editor(ui);
                return;
            }

            if self.show_preview {
                let available = ui.available_width();
                let left_width = available * self.split_ratio;

                ui.horizontal(|ui| {
                    let mut editor_metrics = ScrollMetrics {
                        id: egui::Id::NULL,
                        offset_y: 0.0,
                        content_height: 1.0,
                        viewport_height: 1.0,
                    };
                    let mut preview_metrics = editor_metrics;

                    ui.allocate_ui_with_layout(
                        egui::vec2(left_width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.heading("Editor");
                            ui.separator();
                            editor_metrics = self.render_editor(ui);
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
                            preview_metrics = self.render_preview(ui);
                        },
                    );

                    if self.editor_settings.sync_scroll {
                        self.sync_scroll
                            .sync(ui.ctx(), editor_metrics, preview_metrics);
                    }
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
