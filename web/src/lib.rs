mod editor_highlight;
mod export;
mod find_replace;
mod vim_ex;
mod keybindings;
mod clipboard;
mod line_gutter;
mod markdown;
mod minimap;
mod settings;
mod sync_scroll;
mod unsaved;

use leptos::ev::Event;
use leptos::html::{Canvas, Div, Input, Textarea};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, HtmlInputElement, HtmlTextAreaElement, KeyboardEvent, MouseEvent};

const STORAGE_CONTENT: &str = "omd-web-content";
const STORAGE_THEME: &str = "omd-web-theme";
const STORAGE_VIEW: &str = "omd-web-view";
const STORAGE_FILENAME: &str = "omd-web-filename";

use settings::EditorSettings;
use keybindings::{KeybindingMode, KeybindingState, VimMode};

const DEFAULT_CONTENT: &str = r#"# omd Web 功能演示

欢迎使用 **omd** 浏览器版 Markdown 编辑器！本文档展示全部功能，可直接编辑体验。

---

## 1. 文本格式

| 格式 | 语法 | 效果 |
|------|------|------|
| 粗体 | `**粗体**` | **粗体** |
| 斜体 | `*斜体*` | *斜体* |
| 删除线 | `~~删除~~` | ~~删除~~ |
| 行内代码 | `` `code` `` | `code` |
| 链接 | `[文字](url)` | [Rust 官网](https://www.rust-lang.org) |

> 工具栏快捷按钮：**B** 粗体 · **I** 斜体 · **S** 删除线 · **</>** 代码 · **🔗** 链接

---

## 2. 标题与结构

### 三级标题
#### 四级标题

- 无序列表项 A
- 无序列表项 B
  - 嵌套子项

1. 有序列表第一步
2. 有序列表第二步

> 引用块：Markdown 让写作更高效。工具栏 **❝** 可快速插入引用。

---

## 3. 任务列表

- [x] 实时预览
- [x] 自动保存到浏览器
- [x] 导入 / 下载 `.md` 文件
- [x] 图片与 Mermaid 图表
- [ ] 继续探索更多功能…

---

## 4. 代码块

```rust
fn main() {
    println!("Hello, omd Web!");
}
```

---

## 5. 表格

| 功能 | 操作 | 说明 |
|------|------|------|
| 新建 | 顶部「新建」 | 清空编辑器 |
| 打开 | 顶部「打开」 | 导入 `.md` 文件 |
| 下载 | 顶部「下载」 | 导出当前内容 |
| 主题 | 🌙 / ☀️ | 深色 / 浅色切换 |
| 视图 | ⊞ ✎ 👁 | 分栏 / 仅编辑 / 仅预览 |

---

## 6. 图片

### URL 图片
![Rust Logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg)

### 插入方式
- **🖼** 工具栏：从相册或文件选择（Base64 嵌入）
- **🌐** 工具栏：输入图片 URL
- **粘贴**：在编辑区 `Ctrl+V` / 长按粘贴截图
- **拖拽**：将图片文件拖入编辑区

---

## 7. Mermaid 图表

### 流程图
```mermaid
flowchart TD
    A[编写 Markdown] --> B{实时预览}
    B --> C[插入图片]
    B --> D[渲染图表]
    C --> E[下载 / 自动保存]
    D --> E
```

### 时序图
```mermaid
sequenceDiagram
    participant 用户
    participant 编辑器
    participant 预览区
    用户->>编辑器: 输入文字
    编辑器->>预览区: 即时渲染
    预览区-->>用户: 显示结果
```

---

## 8. 视图与主题

| 按钮 | 模式 | 适用场景 |
|------|------|----------|
| ⊞ | 分栏 | 电脑宽屏，边写边看 |
| ✎ | 仅编辑 | 手机竖屏，专注写作 |
| 👁 | 仅预览 | 阅读成品效果 |

点击右上角 **🌙** / **☀️** 切换深色与浅色主题，Mermaid 图表会同步适配。

---

## 9. 自动保存

编辑内容会自动保存到浏览器 **localStorage**，刷新页面后恢复。状态栏显示行数、字数、字符数及「已自动保存」提示。

---

**开始编辑吧！** 修改任意文字，右侧预览即时更新。🦀
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = omdRenderMermaid)]
    fn omd_render_mermaid();

    #[wasm_bindgen(js_namespace = window, js_name = omdHighlightCode)]
    fn omd_highlight_code();

    #[wasm_bindgen(js_namespace = window, js_name = omdApplyTheme)]
    fn omd_apply_theme(dark: bool);
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Split,
    EditorOnly,
    PreviewOnly,
}

impl ViewMode {
    fn css_class(self) -> &'static str {
        match self {
            ViewMode::Split => "",
            ViewMode::EditorOnly => "editor-only",
            ViewMode::PreviewOnly => "preview-only",
        }
    }
}

fn load_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
}

fn save_storage(key: &str, value: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item(key, value);
    }
}

fn apply_theme(dark: bool) {
    omd_apply_theme(dark);
}

fn download_blob(content: &str, filename: &str, mime: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(content));

    let blob_props = BlobPropertyBag::new();
    let _ = blob_props.set_type(mime);

    let blob = match Blob::new_with_str_sequence_and_options(&parts, &blob_props) {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    if let Ok(a) = document.create_element("a") {
        let _ = a.set_attribute("href", &url);
        let _ = a.set_attribute("download", filename);
        a.set_class_name("file-input-hidden");
        if let Some(body) = document.body() {
            let _ = body.append_child(&a);
            if let Some(html_el) = a.dyn_ref::<web_sys::HtmlElement>() {
                html_el.click();
            }
            let _ = body.remove_child(&a);
        }
        let _ = web_sys::Url::revoke_object_url(&url);
    }
}

fn download_file(content: &str, filename: &str) {
    download_blob(content, filename, "text/markdown;charset=utf-8");
}

fn textarea_selection(el: &HtmlTextAreaElement) -> (usize, usize) {
    let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
    let end = el.selection_end().ok().flatten().unwrap_or(0) as usize;
    (start, end)
}

fn insert_image_into(
    current: &str,
    set_content: WriteSignal<String>,
    textarea_ref: &NodeRef<Textarea>,
    alt: &str,
    url: &str,
) {
    let md = markdown::image_markdown(alt, url);
    let new_text = if let Some(el) = textarea_ref.get() {
        let (start, _) = textarea_selection(&el);
        markdown::insert_at_cursor(current, start, &md)
    } else {
        format!("{current}{md}")
    };
    set_content.set(new_text);
}

#[component]
fn App() -> impl IntoView {
    let initial_content = load_storage(STORAGE_CONTENT).unwrap_or_else(|| DEFAULT_CONTENT.to_string());
    let initial_dark = load_storage(STORAGE_THEME).map(|t| t == "dark").unwrap_or(true);
    let initial_view = match load_storage(STORAGE_VIEW).as_deref() {
        Some("editor") => ViewMode::EditorOnly,
        Some("preview") => ViewMode::PreviewOnly,
        _ => ViewMode::Split,
    };

    let initial_settings = EditorSettings::load();
    settings::apply_editor_css(&initial_settings);

    apply_theme(initial_dark);

    let (content, set_content) = signal(initial_content);
    let (dark_mode, set_dark_mode) = signal(initial_dark);
    let (view_mode, set_view_mode) = signal(initial_view);
    let (editor_settings, set_editor_settings) = signal(initial_settings);
    let (keybinding_state, set_keybinding_state) = signal(KeybindingState::default());
    let (settings_open, set_settings_open) = signal(false);
    let (undo_hint, set_undo_hint) = signal(String::new());
    let (filename, set_filename) = signal(
        load_storage(STORAGE_FILENAME).unwrap_or_else(|| "document.md".to_string()),
    );
    let initial_snapshot = content.get_untracked();
    let (saved_snapshot, set_saved_snapshot) = signal(initial_snapshot.clone());
    let (saved_hint, set_saved_hint) = signal(false);
    let (find_open, set_find_open) = signal(false);
    let (find_replace_mode, set_find_replace_mode) = signal(false);
    let (find_query, set_find_query) = signal(String::new());
    let (replace_query, set_replace_query) = signal(String::new());
    let (find_case_sensitive, set_find_case_sensitive) = signal(false);
    let (find_match_index, set_find_match_index) = signal(0usize);
    let (current_line, set_current_line) = signal(0usize);
    let (editor_char_cursor, set_editor_char_cursor) = signal(0usize);
    let (editor_scroll_top, set_editor_scroll_top) = signal(0.0f64);
    let textarea_ref = NodeRef::<Textarea>::new();
    let highlight_layer_ref = NodeRef::<Div>::new();
    let line_gutter_ref = NodeRef::<Div>::new();
    let preview_ref = NodeRef::<Div>::new();
    let minimap_ref = NodeRef::<Canvas>::new();
    let minimap_drag = RwSignal::new(false);
    let scroll_sync_guard = RwSignal::new(false);
    let file_input_ref = NodeRef::<Input>::new();
    let image_input_ref = NodeRef::<Input>::new();
    let command_input_ref = NodeRef::<Input>::new();

    // Auto-save to localStorage
    Effect::new(move |_| {
        let text = content.get();
        let dark = dark_mode.get();
        let view = view_mode.get();

        save_storage(STORAGE_CONTENT, &text);
        save_storage(STORAGE_THEME, if dark { "dark" } else { "light" });
        save_storage(
            STORAGE_VIEW,
            match view {
                ViewMode::Split => "split",
                ViewMode::EditorOnly => "editor",
                ViewMode::PreviewOnly => "preview",
            },
        );
        let name = filename.get();
        save_storage(STORAGE_FILENAME, &name);

        set_saved_hint.set(true);
        let set_saved_hint = set_saved_hint.clone();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1500).await;
            set_saved_hint.set(false);
        });
    });

    Effect::new(move |_| {
        let s = editor_settings.get();
        s.save();
        settings::apply_editor_css(&s);
    });

    Effect::new(move |_| {
        let hint = undo_hint.get();
        if hint.is_empty() {
            return;
        }
        let set_undo_hint = set_undo_hint.clone();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1500).await;
            set_undo_hint.set(String::new());
        });
    });

    // Render mermaid diagrams after preview updates
    Effect::new(move |_| {
        let _ = content.get();
        let _ = dark_mode.get();
        let syntax = editor_settings.get().preview_syntax_highlight;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(50).await;
            omd_render_mermaid();
            if syntax {
                omd_highlight_code();
            }
        });
    });

    Effect::new({
        move |_| {
            crate::clipboard::refresh_cache();
        }
    });

    Effect::new({
        let content = content.clone();
        let saved_snapshot = saved_snapshot.clone();
        move |_| {
            let modified = unsaved::is_modified(&content.get(), &saved_snapshot.get());
            unsaved::set_unsaved_warning(modified);
        }
    });

    Effect::new({
        let command_input_ref = command_input_ref.clone();
        let keybinding_state = keybinding_state.clone();
        move |_| {
            if keybinding_state.get().vim_mode == VimMode::Command {
                let command_input_ref = command_input_ref.clone();
                spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(0).await;
                    if let Some(input) = command_input_ref.get() {
                        let _ = input.focus();
                    }
                });
            }
        }
    });

    let repaint_minimap = {
        let minimap_ref = minimap_ref.clone();
        let textarea_ref = textarea_ref.clone();
        let content = content.clone();
        let dark_mode = dark_mode.clone();
        move || {
            if let (Some(canvas), Some(ta)) = (minimap_ref.get(), textarea_ref.get()) {
                minimap::paint(&canvas, &ta, &content.get_untracked(), dark_mode.get_untracked());
            }
        }
    };

    Effect::new({
        let repaint = repaint_minimap.clone();
        move |_| {
            let _ = content.get();
            let _ = dark_mode.get();
            repaint();
        }
    });

    let update_cursor_line = {
        let textarea_ref = textarea_ref.clone();
        let content = content.clone();
        let set_current_line = set_current_line.clone();
        let set_editor_char_cursor = set_editor_char_cursor.clone();
        move || {
            if let Some(ta) = textarea_ref.get() {
                let offset = ta.selection_start().ok().flatten().unwrap_or(0);
                let text = content.get_untracked();
                let line = line_gutter::line_index_at_utf16(&text, offset);
                set_current_line.set(line);
                set_editor_char_cursor.set(line_gutter::utf16_to_char_index(&text, offset));
            }
        }
    };

    let on_editor_scroll = {
        let repaint = repaint_minimap.clone();
        let line_gutter_ref = line_gutter_ref.clone();
        let highlight_layer_ref = highlight_layer_ref.clone();
        let textarea_ref = textarea_ref.clone();
        let preview_ref = preview_ref.clone();
        let view_mode = view_mode.clone();
        let scroll_sync_guard = scroll_sync_guard.clone();
        let set_editor_scroll_top = set_editor_scroll_top.clone();
        move |_| {
            if let Some(ta) = textarea_ref.get() {
                if let Some(gutter) = line_gutter_ref.get() {
                    gutter.set_scroll_top(ta.scroll_top());
                }
                if let Some(layer) = highlight_layer_ref.get() {
                    layer.set_scroll_top(ta.scroll_top());
                    layer.set_scroll_left(ta.scroll_left());
                }
                set_editor_scroll_top.set(ta.scroll_top() as f64);
            }
            if !scroll_sync_guard.get_untracked()
                && view_mode.get_untracked() == ViewMode::Split
                && editor_settings.get_untracked().sync_scroll
            {
                if let (Some(ta), Some(preview_el)) = (textarea_ref.get(), preview_ref.get()) {
                    if let Some(preview) = sync_scroll::as_html_element(&preview_el) {
                        scroll_sync_guard.set(true);
                        sync_scroll::sync_editor_to_preview(&ta, &preview);
                        scroll_sync_guard.set(false);
                    }
                }
            }
            repaint();
        }
    };

    let on_preview_scroll = {
        let textarea_ref = textarea_ref.clone();
        let preview_ref = preview_ref.clone();
        let line_gutter_ref = line_gutter_ref.clone();
        let highlight_layer_ref = highlight_layer_ref.clone();
        let view_mode = view_mode.clone();
        let scroll_sync_guard = scroll_sync_guard.clone();
        let set_editor_scroll_top = set_editor_scroll_top.clone();
        let repaint = repaint_minimap.clone();
        move |_| {
            if scroll_sync_guard.get_untracked()
                || view_mode.get_untracked() != ViewMode::Split
                || !editor_settings.get_untracked().sync_scroll
            {
                return;
            }
            if let (Some(ta), Some(preview_el)) = (textarea_ref.get(), preview_ref.get()) {
                if let Some(preview) = sync_scroll::as_html_element(&preview_el) {
                    scroll_sync_guard.set(true);
                    sync_scroll::sync_preview_to_editor(&preview, &ta);
                    if let Some(gutter) = line_gutter_ref.get() {
                        gutter.set_scroll_top(ta.scroll_top());
                    }
                    if let Some(layer) = highlight_layer_ref.get() {
                        layer.set_scroll_top(ta.scroll_top());
                        layer.set_scroll_left(ta.scroll_left());
                    }
                    set_editor_scroll_top.set(ta.scroll_top() as f64);
                    scroll_sync_guard.set(false);
                }
            }
            repaint();
        }
    };

    let on_editor_click = {
        let update_cursor_line = update_cursor_line.clone();
        move |_: MouseEvent| update_cursor_line()
    };

    let on_editor_keyup = {
        let update_cursor_line = update_cursor_line.clone();
        move |_: KeyboardEvent| update_cursor_line()
    };

    let on_editor_select = {
        let update_cursor_line = update_cursor_line.clone();
        move |_: Event| update_cursor_line()
    };

    let on_textarea_keydown = {
        let content = content.clone();
        let set_content = set_content.clone();
        let editor_settings = editor_settings.clone();
        let keybinding_state = keybinding_state.clone();
        let set_keybinding_state = set_keybinding_state.clone();
        let set_editor_settings = set_editor_settings.clone();
        let set_undo_hint = set_undo_hint.clone();
        let textarea_ref = textarea_ref.clone();
        let find_open = find_open.clone();
        let filename = filename.clone();
        let set_saved_snapshot = set_saved_snapshot.clone();
        move |ev: KeyboardEvent| {
            if ev.key() == "F11" {
                ev.prevent_default();
                set_editor_settings.update(|s| s.focus_mode = !s.focus_mode);
            }
            if editor_settings.get_untracked().focus_mode && ev.key() == "Escape" {
                set_editor_settings.update(|s| s.focus_mode = false);
            }
            if editor_settings.get_untracked().show_undo_redo_hint
                && (ev.ctrl_key() || ev.meta_key())
            {
                if ev.key() == "z" {
                    set_undo_hint.set(if ev.shift_key() {
                        "重做".to_string()
                    } else {
                        "撤销".to_string()
                    });
                } else if ev.key() == "y" {
                    set_undo_hint.set("重做".to_string());
                }
            }

            if find_open.get_untracked() {
                return;
            }

            let mode = editor_settings.get_untracked().keybinding_mode;
            if mode == KeybindingMode::Standard {
                return;
            }

            let Some(ta) = textarea_ref.get() else {
                return;
            };

            let mut text = content.get_untracked();
            let start_utf16 = ta.selection_start().ok().flatten().unwrap_or(0);
            let end_utf16 = ta.selection_end().ok().flatten().unwrap_or(start_utf16);
            let cursor = line_gutter::utf16_to_char_index(&text, start_utf16);
            let sel_start =
                line_gutter::utf16_to_char_index(&text, start_utf16.min(end_utf16));
            let sel_end =
                line_gutter::utf16_to_char_index(&text, start_utf16.max(end_utf16));
            let selection = if start_utf16 != end_utf16 {
                Some((sel_start, sel_end))
            } else {
                None
            };

            let mut kb = keybinding_state.get_untracked();
            kb.use_system_clipboard = editor_settings
                .get_untracked()
                .vim_use_system_clipboard;
            let Some(action) = keybindings::handle_keydown(
                &mut text,
                &mut kb,
                mode,
                &ev.key(),
                ev.ctrl_key(),
                ev.shift_key(),
                ev.alt_key(),
                ev.meta_key(),
                cursor,
                selection,
            ) else {
                return;
            };

            if action.consume {
                ev.prevent_default();
            }
            if action.content_changed {
                set_content.set(text.clone());
            }
            if let Some(vim_mode) = action.vim_mode {
                kb.vim_mode = vim_mode;
            }
            if let Some(hint) = action.hint {
                set_undo_hint.set(hint);
            }
            if let Some(result) = action.command_result {
                if let Some(show) = result.line_numbers {
                    set_editor_settings.update(|s| s.show_line_numbers = show);
                }
                if result.request_save {
                    let name = filename.get_untracked();
                    download_file(&text, &name);
                    set_saved_snapshot.set(text.clone());
                }
            }
            set_keybinding_state.set(kb);
            let ta_ref = textarea_ref.clone();
            let cursor = action.cursor;
            let selection = action.selection;
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(0).await;
                if let Some(ta) = ta_ref.get() {
                    let value = ta.value();
                    line_gutter::set_char_selection(&ta, &value, cursor, selection);
                }
            });
        }
    };

    let minimap_scroll_at = {
        let minimap_ref = minimap_ref.clone();
        let textarea_ref = textarea_ref.clone();
        let repaint = repaint_minimap.clone();
        move |offset_y: f64| {
            if let (Some(canvas), Some(ta)) = (minimap_ref.get(), textarea_ref.get()) {
                minimap::scroll_from_pointer(&canvas, &ta, offset_y);
                repaint();
            }
        }
    };

    let on_minimap_down = {
        let minimap_scroll_at = minimap_scroll_at.clone();
        let minimap_drag = minimap_drag.clone();
        move |ev: MouseEvent| {
            minimap_drag.set(true);
            minimap_scroll_at(ev.offset_y() as f64);
        }
    };

    let on_minimap_move = {
        let minimap_scroll_at = minimap_scroll_at.clone();
        let minimap_drag = minimap_drag.clone();
        move |ev: MouseEvent| {
            if minimap_drag.get_untracked() {
                minimap_scroll_at(ev.offset_y() as f64);
            }
        }
    };

    let on_minimap_up = move |_: MouseEvent| {
        minimap_drag.set(false);
    };

    let go_to_find_match = {
        let content = content.clone();
        let find_query = find_query.clone();
        let find_case_sensitive = find_case_sensitive.clone();
        let textarea_ref = textarea_ref.clone();
        let set_find_match_index = set_find_match_index.clone();
        let set_current_line = set_current_line.clone();
        move |index: usize| {
            let ranges = find_replace::find_ranges(
                &content.get_untracked(),
                &find_query.get_untracked(),
                find_case_sensitive.get_untracked(),
            );
            if let Some(&(start, end)) = ranges.get(index) {
                if let Some(ta) = textarea_ref.get() {
                    find_replace::select_range(&ta, start, end);
                    let offset = ta.selection_start().ok().flatten().unwrap_or(0);
                    let line = line_gutter::line_index_at_utf16(
                        &content.get_untracked(),
                        offset,
                    );
                    set_current_line.set(line);
                }
                set_find_match_index.set(index);
            }
        }
    };

    let step_find_match = {
        let go_to_find_match = go_to_find_match.clone();
        let content = content.clone();
        let find_query = find_query.clone();
        let find_case_sensitive = find_case_sensitive.clone();
        let find_match_index = find_match_index.clone();
        move |forward: bool| {
            let ranges = find_replace::find_ranges(
                &content.get_untracked(),
                &find_query.get_untracked(),
                find_case_sensitive.get_untracked(),
            );
            if ranges.is_empty() {
                return;
            }
            let idx = find_match_index.get_untracked();
            let next = if forward {
                (idx + 1) % ranges.len()
            } else {
                (idx + ranges.len() - 1) % ranges.len()
            };
            go_to_find_match(next);
        }
    };

    Effect::new({
        let set_find_open = set_find_open.clone();
        let set_find_replace_mode = set_find_replace_mode.clone();
        let find_open = find_open.clone();
        let step_find_match = step_find_match.clone();
        move |_| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let set_find_open = set_find_open.clone();
            let set_find_replace_mode = set_find_replace_mode.clone();
            let find_open = find_open.clone();
            let step_find_match = step_find_match.clone();
            let closure = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
                let ctrl = ev.ctrl_key() || ev.meta_key();
                if ctrl && ev.key().eq_ignore_ascii_case("f") {
                    ev.prevent_default();
                    set_find_open.set(true);
                    set_find_replace_mode.set(false);
                }
                if ctrl && ev.key().eq_ignore_ascii_case("h") {
                    ev.prevent_default();
                    set_find_open.set(true);
                    set_find_replace_mode.set(true);
                }
                if find_open.get_untracked() && ev.key() == "Escape" {
                    set_find_open.set(false);
                }
                if find_open.get_untracked() && ev.key() == "Enter" {
                    ev.prevent_default();
                    step_find_match(!ev.shift_key());
                }
                if ev.key() == "F3" {
                    ev.prevent_default();
                    step_find_match(ev.shift_key());
                }
            }) as Box<dyn FnMut(_)>);
            let _ = window.add_event_listener_with_callback(
                "keydown",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();
        }
    });

    let find_match_label = move || {
        let query = find_query.get();
        if query.is_empty() {
            return "—".to_string();
        }
        let ranges = find_replace::find_ranges(
            &content.get(),
            &query,
            find_case_sensitive.get(),
        );
        if ranges.is_empty() {
            "0/0".to_string()
        } else {
            format!(
                "{}/{}",
                find_match_index.get() + 1,
                ranges.len()
            )
        }
    };

    let preview_html = move || markdown::markdown_to_html(&content.get());
    let editor_highlight_html = move || {
        if editor_settings.get().editor_syntax_highlight {
            editor_highlight::lines_to_html(&content.get())
        } else {
            String::new()
        }
    };

    let stats = move || {
        let text = content.get();
        (
            markdown::line_count(&text),
            markdown::word_count(&text),
            text.chars().count(),
        )
    };

    let insert_format = move |wrapper: &'static str| {
        if let Some(el) = textarea_ref.get() {
            let (start, end) = textarea_selection(&el);
            let new_text = markdown::wrap_selection(&content.get(), start, end, wrapper);
            set_content.set(new_text);
        }
    };

    let insert_prefix = move |prefix: &'static str| {
        if let Some(el) = textarea_ref.get() {
            let (start, end) = textarea_selection(&el);
            let new_text = markdown::prefix_lines(&content.get(), start, end, prefix);
            set_content.set(new_text);
        }
    };

    let on_image_file = {
        let set_content = set_content.clone();
        let textarea_ref = textarea_ref.clone();
        let content = content.clone();
        move |ev: Event| {
        let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let name = file.name();
                let alt = std::path::Path::new(&name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image")
                    .to_string();
                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let set_content = set_content.clone();
                let textarea_ref = textarea_ref.clone();
                let current = content.get_untracked();
                let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                    if let Ok(result) = reader_clone.result() {
                        if let Some(data_url) = result.as_string() {
                            insert_image_into(&current, set_content, &textarea_ref, &alt, &data_url);
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                let _ = reader.read_as_data_url(&file);
            }
        }
        input.set_value("");
        }
    };

    let on_paste = {
        let set_content = set_content.clone();
        let textarea_ref = textarea_ref.clone();
        let content = content.clone();
        move |ev: Event| {
            let ev: web_sys::ClipboardEvent = ev.unchecked_into();
            if let Some(dt) = ev.clipboard_data() {
                let items = dt.items();
                for i in 0..items.length() {
                    if let Some(item) = items.get(i) {
                        if item.type_().starts_with("image/") {
                            ev.prevent_default();
                            if let Ok(Some(file)) = item.get_as_file() {
                                let reader = web_sys::FileReader::new().unwrap();
                                let reader_clone = reader.clone();
                                let set_content = set_content.clone();
                                let textarea_ref = textarea_ref.clone();
                                let current = content.get_untracked();
                                let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                                    if let Ok(result) = reader_clone.result() {
                                        if let Some(data_url) = result.as_string() {
                                            insert_image_into(
                                                &current,
                                                set_content,
                                                &textarea_ref,
                                                "image",
                                                &data_url,
                                            );
                                        }
                                    }
                                }) as Box<dyn FnMut(_)>);
                                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                onload.forget();
                                let _ = reader.read_as_data_url(&file);
                            }
                            return;
                        }
                    }
                }
                if let Ok(text) = dt.get_data("text/plain") {
                    if !text.is_empty() {
                        crate::clipboard::remember_text(&text);
                    }
                }
            }
        }
    };

    let on_drop = {
        let set_content = set_content.clone();
        let textarea_ref = textarea_ref.clone();
        let content = content.clone();
        move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        if let Some(dt) = ev.data_transfer() {
            if let Some(files) = dt.files() {
                if let Some(file) = files.get(0) {
                    if file.type_().starts_with("image/") {
                        let alt = file
                            .name()
                            .split('.')
                            .next()
                            .unwrap_or("image")
                            .to_string();
                        let reader = web_sys::FileReader::new().unwrap();
                        let reader_clone = reader.clone();
                        let set_content = set_content.clone();
                        let textarea_ref = textarea_ref.clone();
                        let current = content.get_untracked();
                        let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                            if let Ok(result) = reader_clone.result() {
                                if let Some(data_url) = result.as_string() {
                                    insert_image_into(&current, set_content, &textarea_ref, &alt, &data_url);
                                }
                            }
                        }) as Box<dyn FnMut(_)>);
                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                        onload.forget();
                        let _ = reader.read_as_data_url(&file);
                    }
                }
            }
        }
        }
    };

    let on_drag_over = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
    };

    let on_file_change = move |ev: Event| {
        let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let set_content = set_content.clone();
                let set_filename = set_filename.clone();
                let set_saved_snapshot = set_saved_snapshot.clone();
                let name = file.name();
                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                    if let Ok(result) = reader_clone.result() {
                        if let Some(text) = result.as_string() {
                            set_saved_snapshot.set(text.clone());
                            set_content.set(text);
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                let _ = reader.read_as_text(&file);
                set_filename.set(name);
            }
        }
        input.set_value("");
    };

    view! {
        <div id="app" class=move || if editor_settings.get().focus_mode { "focus-mode" } else { "" }>
            <header class="header">
                <h1><span>"omd"</span>" Web"</h1>
                <div class="header-actions">
                    <button class="btn" on:click={
                        let content = content.clone();
                        let saved_snapshot = saved_snapshot.clone();
                        let set_content = set_content.clone();
                        let set_filename = set_filename.clone();
                        let set_saved_snapshot = set_saved_snapshot.clone();
                        move |_| {
                            if unsaved::is_modified(&content.get(), &saved_snapshot.get())
                                && !unsaved::confirm_discard_changes()
                            {
                                return;
                            }
                            set_content.set(String::new());
                            set_filename.set("document.md".to_string());
                            set_saved_snapshot.set(String::new());
                        }
                    }>"新建"</button>
                    <button class="btn" on:click={
                        let content = content.clone();
                        let saved_snapshot = saved_snapshot.clone();
                        let file_input_ref = file_input_ref.clone();
                        move |_| {
                            if unsaved::is_modified(&content.get(), &saved_snapshot.get())
                                && !unsaved::confirm_discard_changes()
                            {
                                return;
                            }
                            if let Some(input) = file_input_ref.get() {
                                input.click();
                            }
                        }
                    }>"打开"</button>
                    <button class="btn btn-primary" on:click={
                        let content = content.clone();
                        let filename = filename.clone();
                        let set_saved_snapshot = set_saved_snapshot.clone();
                        move |_| {
                            let text = content.get();
                            let name = filename.get();
                            download_file(&text, &name);
                            set_saved_snapshot.set(text);
                        }
                    }>"下载"</button>
                    <button class="btn" on:click=move |_| {
                        let md = content.get();
                        let name = filename.get();
                        let dark = dark_mode.get();
                        let title = export::export_title(&name, &md);
                        let html = export::export_html_document(&md, &title, dark);
                        download_blob(
                            &html,
                            &export::html_filename(&name),
                            "text/html;charset=utf-8",
                        );
                    }>"导出 HTML"</button>
                    <button class="btn btn-icon" title="设置"
                        on:click=move |_| set_settings_open.set(true)>"⚙"</button>
                    <button class="btn btn-icon" title="切换主题"
                        on:click=move |_| {
                            let next = !dark_mode.get();
                            set_dark_mode.set(next);
                            apply_theme(next);
                        }
                    >
                        {move || if dark_mode.get() { "🌙" } else { "☀️" }}
                    </button>
                </div>
            </header>

            <input type="file" accept=".md,.markdown,.txt" class="file-input-hidden"
                node_ref=file_input_ref on:change=on_file_change />
            <input type="file" accept="image/*" class="file-input-hidden"
                node_ref=image_input_ref on:change=on_image_file />

            <div class="toolbar">
                <button class="btn btn-icon" on:click=move |_| insert_format("**") title="粗体">"B"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("*") title="斜体">"I"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("~~") title="删除线">"S"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("`") title="行内代码">"</>"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("[]()") title="链接">"🔗"</button>
                <button class="btn btn-icon" title="插入图片"
                    on:click=move |_| {
                        if let Some(input) = image_input_ref.get() {
                            input.click();
                        }
                    }
                >"🖼"</button>
                <button class="btn btn-icon" title="图片 URL"
                    on:click={
                        let set_content = set_content.clone();
                        let textarea_ref = textarea_ref.clone();
                        let content = content.clone();
                        move |_| {
                        if let Some(url) = web_sys::window()
                            .and_then(|w| w.prompt_with_message("输入图片 URL:").ok().flatten())
                            .filter(|s| !s.is_empty())
                        {
                            insert_image_into(&content.get(), set_content, &textarea_ref, "image", &url);
                        }
                        }
                    }
                >"🌐"</button>
                <button class="btn btn-icon" on:click=move |_| insert_prefix("# ") title="标题">"H"</button>
                <button class="btn btn-icon" on:click=move |_| insert_prefix("- ") title="列表">"•"</button>
                <button class="btn btn-icon" on:click=move |_| insert_prefix("> ") title="引用">"❝"</button>
                <button class=move || format!("btn btn-icon {}", if view_mode.get() == ViewMode::Split { "active" } else { "" })
                    on:click=move |_| set_view_mode.set(ViewMode::Split) title="分栏">"⊞"</button>
                <button class=move || format!("btn btn-icon {}", if view_mode.get() == ViewMode::EditorOnly { "active" } else { "" })
                    on:click=move |_| set_view_mode.set(ViewMode::EditorOnly) title="仅编辑">"✎"</button>
                <button class=move || format!("btn btn-icon {}", if view_mode.get() == ViewMode::PreviewOnly { "active" } else { "" })
                    on:click=move |_| set_view_mode.set(ViewMode::PreviewOnly) title="仅预览">"👁"</button>
            </div>

            {move || (keybinding_state.get().vim_mode == VimMode::Command).then(|| {
                let set_keybinding_state = set_keybinding_state.clone();
                let set_content = set_content.clone();
                let set_editor_settings = set_editor_settings.clone();
                let set_undo_hint = set_undo_hint.clone();
                let content = content.clone();
                let filename = filename.clone();
                let set_saved_snapshot = set_saved_snapshot.clone();
                let command_input_ref = command_input_ref.clone();
                let textarea_ref = textarea_ref.clone();
                view! {
                    <div class="command-bar">
                        <label class="command-field">
                            ":"
                            <input
                                type="text"
                                class="command-input"
                                node_ref=command_input_ref
                                prop:value=move || keybinding_state.get().command_buffer
                                on:input=move |ev| {
                                    let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                    set_keybinding_state.update(|kb| {
                                        kb.command_buffer = el.value();
                                    });
                                }
                                on:keydown=move |ev: KeyboardEvent| {
                                    if ev.key() == "Enter" {
                                        ev.prevent_default();
                                        let mut text = content.get_untracked();
                                        let mut kb = keybinding_state.get_untracked();
                                        let start_utf16 = textarea_ref
                                            .get()
                                            .and_then(|ta| ta.selection_start().ok().flatten())
                                            .unwrap_or(0);
                                        let cursor =
                                            line_gutter::utf16_to_char_index(&text, start_utf16);
                                        if let Some(action) =
                                            keybindings::execute_command(&mut kb, &mut text, cursor)
                                        {
                                            if action.content_changed {
                                                set_content.set(text.clone());
                                            }
                                            if let Some(hint) = action.hint {
                                                set_undo_hint.set(hint);
                                            }
                                            if let Some(result) = action.command_result {
                                                if let Some(show) = result.line_numbers {
                                                    set_editor_settings
                                                        .update(|s| s.show_line_numbers = show);
                                                }
                                                if result.request_save {
                                                    let name = filename.get_untracked();
                                                    download_file(&text, &name);
                                                    set_saved_snapshot.set(text.clone());
                                                }
                                            }
                                            set_keybinding_state.set(kb);
                                            let ta_ref = textarea_ref.clone();
                                            let cursor = action.cursor;
                                            spawn_local(async move {
                                                gloo_timers::future::TimeoutFuture::new(0).await;
                                                if let Some(ta) = ta_ref.get() {
                                                    let value = ta.value();
                                                    line_gutter::set_char_selection(
                                                        &ta, &value, cursor, None,
                                                    );
                                                }
                                            });
                                        }
                                    }
                                    if ev.key() == "Escape" {
                                        ev.prevent_default();
                                        set_keybinding_state.update(|kb| {
                                            kb.command_buffer.clear();
                                            kb.vim_mode = VimMode::Normal;
                                        });
                                    }
                                }
                                placeholder="w · q · g/pat/d · g/pat/s/o/n/g · g/pat/norm I-  · set number · reg"
                            />
                        </label>
                    </div>
                }
            })}

            {move || find_open.get().then(|| {
                let step_find_match = step_find_match.clone();
                let set_find_open = set_find_open.clone();
                let set_find_query = set_find_query.clone();
                let set_replace_query = set_replace_query.clone();
                let set_find_case_sensitive = set_find_case_sensitive.clone();
                let set_find_match_index = set_find_match_index.clone();
                let set_content = set_content.clone();
                let content = content.clone();
                let find_query = find_query.clone();
                let replace_query = replace_query.clone();
                let find_case_sensitive = find_case_sensitive.clone();
                let find_match_index = find_match_index.clone();
                let go_to_find_match = go_to_find_match.clone();
                view! {
                    <div class="find-bar">
                        <label class="find-field">
                            "查找"
                            <input
                                type="text"
                                class="find-input"
                                prop:value=move || find_query.get()
                                on:input=move |ev| {
                                    let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                    set_find_query.set(el.value());
                                    set_find_match_index.set(0);
                                }
                            />
                        </label>
                        <label class="find-case">
                            <input
                                type="checkbox"
                                prop:checked=move || find_case_sensitive.get()
                                on:change=move |ev| {
                                    let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                    set_find_case_sensitive.set(el.checked());
                                    set_find_match_index.set(0);
                                }
                            />
                            "区分大小写"
                        </label>
                        <span class="find-count">{find_match_label}</span>
                        <button class="btn btn-icon" type="button" title="上一个 (Shift+Enter)"
                            on:click=move |_| step_find_match(false)>"↑"</button>
                        <button class="btn btn-icon" type="button" title="下一个 (Enter)"
                            on:click=move |_| step_find_match(true)>"↓"</button>
                        {move || find_replace_mode.get().then(|| {
                            let set_content = set_content.clone();
                            let find_query = find_query.clone();
                            let replace_query = replace_query.clone();
                            let find_case_sensitive = find_case_sensitive.clone();
                            let find_match_index = find_match_index.clone();
                            let go_to_find_match = go_to_find_match.clone();
                            view! {
                                <>
                                    <label class="find-field">
                                        "替换"
                                        <input
                                            type="text"
                                            class="find-input"
                                            prop:value=move || replace_query.get()
                                            on:input=move |ev| {
                                                let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                set_replace_query.set(el.value());
                                            }
                                        />
                                    </label>
                                    <button class="btn" type="button" on:click=move |_| {
                                        let mut text = content.get_untracked();
                                        let idx = find_match_index.get_untracked();
                                        let q = find_query.get_untracked();
                                        let r = replace_query.get_untracked();
                                        let cs = find_case_sensitive.get_untracked();
                                        if find_replace::replace_at(&mut text, &q, &r, cs, idx).is_some() {
                                            set_content.set(text.clone());
                                            let ranges = find_replace::find_ranges(&text, &q, cs);
                                            let new_idx = idx.min(ranges.len().saturating_sub(1));
                                            go_to_find_match(new_idx);
                                        }
                                    }>"替换"</button>
                                    <button class="btn" type="button" on:click=move |_| {
                                        let mut text = content.get_untracked();
                                        let q = find_query.get_untracked();
                                        let r = replace_query.get_untracked();
                                        find_replace::replace_all(&mut text, &q, &r, find_case_sensitive.get_untracked());
                                        set_content.set(text);
                                        set_find_match_index.set(0);
                                    }>"全部替换"</button>
                                </>
                            }
                        })}
                        <button class="btn btn-icon find-close" type="button" title="关闭 (Esc)"
                            on:click=move |_| set_find_open.set(false)>"✕"</button>
                    </div>
                }
            })}

            {move || settings_open.get().then(|| {
                let set_editor_settings = set_editor_settings.clone();
                let set_keybinding_state = set_keybinding_state.clone();
                let set_settings_open = set_settings_open.clone();
                view! {
                    <div class="settings-backdrop" on:click=move |_| set_settings_open.set(false)>
                        <div class="settings-panel" on:click=move |ev: MouseEvent| { ev.stop_propagation(); }>
                            <h2>"编辑器设置"</h2>
                            <h3>"编辑区"</h3>
                            <label class="settings-row">
                                "显示行号"
                                <input type="checkbox" prop:checked=move || editor_settings.get().show_line_numbers
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.show_line_numbers = el.checked());
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "高亮当前行"
                                <input type="checkbox" prop:checked=move || editor_settings.get().highlight_current_line
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.highlight_current_line = el.checked());
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "显示 Minimap"
                                <input type="checkbox" prop:checked=move || editor_settings.get().show_minimap
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.show_minimap = el.checked());
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "字号"
                                <input type="range" min="10" max="24" step="1"
                                    prop:value=move || editor_settings.get().editor_font_size.to_string()
                                    on:input=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        if let Ok(v) = el.value().parse::<f32>() {
                                            set_editor_settings.update(|s| s.editor_font_size = v);
                                        }
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "行高"
                                <input type="range" min="12" max="22" step="1"
                                    prop:value=move || (editor_settings.get().editor_line_height * 10.0).round().to_string()
                                    on:input=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        if let Ok(v) = el.value().parse::<f32>() {
                                            set_editor_settings.update(|s| s.editor_line_height = v / 10.0);
                                        }
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "编辑区语法高亮"
                                <input type="checkbox" prop:checked=move || editor_settings.get().editor_syntax_highlight
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.editor_syntax_highlight = el.checked());
                                    }
                                />
                            </label>
                            <h3>"预览"</h3>
                            <label class="settings-row">
                                "同步滚动"
                                <input type="checkbox" prop:checked=move || editor_settings.get().sync_scroll
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.sync_scroll = el.checked());
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "代码块语法高亮"
                                <input type="checkbox" prop:checked=move || editor_settings.get().preview_syntax_highlight
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.preview_syntax_highlight = el.checked());
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "预览字号"
                                <input type="range" min="12" max="22" step="1"
                                    prop:value=move || editor_settings.get().preview_font_size.to_string()
                                    on:input=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        if let Ok(v) = el.value().parse::<f32>() {
                                            set_editor_settings.update(|s| s.preview_font_size = v);
                                        }
                                    }
                                />
                            </label>
                            <h3>"专注与提示"</h3>
                            <label class="settings-row">
                                "专注模式"
                                <input type="checkbox" prop:checked=move || editor_settings.get().focus_mode
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.focus_mode = el.checked());
                                    }
                                />
                            </label>
                            <p class="settings-hint">"快捷键：F11 切换，Esc 退出"</p>
                            <label class="settings-row">
                                "撤销/重做提示"
                                <input type="checkbox" prop:checked=move || editor_settings.get().show_undo_redo_hint
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        set_editor_settings.update(|s| s.show_undo_redo_hint = el.checked());
                                    }
                                />
                            </label>
                            <label class="settings-row">
                                "键位模式"
                                <select
                                    prop:value=move || match editor_settings.get().keybinding_mode {
                                        KeybindingMode::Standard => "standard",
                                        KeybindingMode::Vim => "vim",
                                        KeybindingMode::Emacs => "emacs",
                                    }
                                    on:change=move |ev| {
                                        let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                        let mode = match el.value().as_str() {
                                            "vim" => KeybindingMode::Vim,
                                            "emacs" => KeybindingMode::Emacs,
                                            _ => KeybindingMode::Standard,
                                        };
                                        set_editor_settings.update(|s| s.keybinding_mode = mode);
                                        set_keybinding_state.update(|state| keybindings::reset_for_mode(state, mode));
                                    }
                                >
                                    <option value="standard">"标准"</option>
                                    <option value="vim">"Vim"</option>
                                    <option value="emacs">"Emacs"</option>
                                </select>
                            </label>
                            {move || (editor_settings.get().keybinding_mode == KeybindingMode::Vim).then(|| view! {
                                <>
                                    <label class="settings-row">
                                        "Visual Block 高亮"
                                        <input type="checkbox" prop:checked=move || editor_settings.get().vim_show_block_highlight
                                            on:change=move |ev| {
                                                let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                set_editor_settings.update(|s| s.vim_show_block_highlight = el.checked());
                                            }
                                        />
                                    </label>
                                    <label class="settings-row">
                                        "系统剪贴板寄存器 (+/*)"
                                        <input type="checkbox" prop:checked=move || editor_settings.get().vim_use_system_clipboard
                                            on:change=move |ev| {
                                                let el: HtmlInputElement = ev.target().unwrap().unchecked_into();
                                                set_editor_settings.update(|s| s.vim_use_system_clipboard = el.checked());
                                            }
                                        />
                                    </label>
                                </>
                            })}
                            <p class="settings-hint">"Vim: Ctrl+V · I/A/C 块编辑 · :g/pat/s/o/n/g · :g/pat/norm · Emacs: C-Space mark · C-s 搜索 · C-o/C-j/C-t · M-w"</p>
                            <div class="settings-actions">
                                <button class="btn btn-primary" type="button"
                                    on:click=move |_| set_settings_open.set(false)>"完成"</button>
                            </div>
                        </div>
                    </div>
                }
            })}

            <div class=move || format!("main {}", view_mode.get().css_class())>
                <div class="pane editor-pane">
                    <div class="pane-header">"编辑"</div>
                    <div class="editor-with-minimap">
                        <div class="editor-with-gutter">
                            {move || editor_settings.get().show_line_numbers.then(|| view! {
                                <div class="line-gutter" node_ref=line_gutter_ref>
                                    {move || {
                                        let count = line_gutter::line_count(&content.get());
                                        let active = current_line.get();
                                        (0..count)
                                            .map(|line| {
                                                let class = if line == active {
                                                    "line active"
                                                } else {
                                                    "line"
                                                };
                                                view! {
                                                    <div class=class>{line + 1}</div>
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </div>
                            })}
                            <div class="editor-input-wrap">
                                {move || {
                                    let settings = editor_settings.get();
                                    let kb = keybinding_state.get();
                                    (settings.keybinding_mode == KeybindingMode::Vim
                                        && settings.vim_show_block_highlight
                                        && kb.vim_mode == VimMode::VisualBlock)
                                        .then(|| kb.active_block)
                                        .flatten()
                                        .map(|block| {
                                            let scroll = editor_scroll_top.get();
                                            let fs = f64::from(settings.editor_font_size);
                                            let lh = f64::from(settings.editor_line_height);
                                            (block.line_start..=block.line_end)
                                                .map(|line| {
                                                    let style = line_gutter::block_highlight_style(
                                                        line,
                                                        block.col_start,
                                                        block.col_end,
                                                        scroll,
                                                        fs,
                                                        lh,
                                                    );
                                                    view! { <div class="vim-block-highlight" style=style></div> }
                                                })
                                                .collect_view()
                                        })
                                }}
                                {move || {
                                    let settings = editor_settings.get();
                                    let kb = keybinding_state.get();
                                    if settings.keybinding_mode != KeybindingMode::Emacs {
                                        return None;
                                    }
                                    let isearch = kb.emacs_isearch?;
                                    if isearch.query.is_empty() {
                                        return None;
                                    }
                                    let content_val = content.get();
                                    let matches =
                                        keybindings::isearch_all_matches(&content_val, &isearch.query);
                                    let cursor = editor_char_cursor.get();
                                    let scroll = editor_scroll_top.get();
                                    let fs = f64::from(settings.editor_font_size);
                                    let lh = f64::from(settings.editor_line_height);
                                    Some(
                                        matches
                                            .into_iter()
                                            .map(|(start, end)| {
                                                let pos = vim_ex::pos_to_block_pos(&content_val, start);
                                                let end_col = vim_ex::pos_to_block_pos(
                                                    &content_val,
                                                    end.saturating_sub(1),
                                                )
                                                .col
                                                    + 1;
                                                let class = if start == cursor {
                                                    "isearch-match-current"
                                                } else {
                                                    "isearch-match"
                                                };
                                                let style = line_gutter::isearch_highlight_style(
                                                    pos.line,
                                                    pos.col,
                                                    end_col,
                                                    scroll,
                                                    fs,
                                                    lh,
                                                );
                                                view! { <div class=class style=style></div> }
                                            })
                                            .collect_view(),
                                    )
                                }}
                                {move || editor_settings.get().highlight_current_line.then(|| view! {
                                    <div
                                        class="current-line-highlight"
                                        style:top=move || {
                                            let s = editor_settings.get();
                                            format!(
                                                "{}px",
                                                line_gutter::highlight_top_px(
                                                    current_line.get(),
                                                    editor_scroll_top.get(),
                                                    f64::from(s.editor_font_size),
                                                    f64::from(s.editor_line_height),
                                                )
                                            )
                                        }
                                    ></div>
                                })}
                                {move || editor_settings.get().editor_syntax_highlight.then(|| view! {
                                    <div
                                        class="editor-highlight-layer"
                                        node_ref=highlight_layer_ref
                                        inner_html=editor_highlight_html
                                    ></div>
                                })}
                                <textarea
                                    node_ref=textarea_ref
                                    class:syntax-highlight=move || editor_settings.get().editor_syntax_highlight
                                    prop:value=move || content.get()
                                    on:input=move |ev| {
                                        let el: HtmlTextAreaElement = ev.target().unwrap().unchecked_into();
                                        set_content.set(el.value());
                                        update_cursor_line();
                                    }
                                    on:keydown=on_textarea_keydown
                                    on:scroll=on_editor_scroll
                                    on:click=on_editor_click
                                    on:keyup=on_editor_keyup
                                    on:select=on_editor_select
                                    on:paste=on_paste
                                    on:drop=on_drop
                                    on:dragover=on_drag_over
                                    placeholder="在此输入 Markdown，可粘贴或拖入图片..."
                                    spellcheck="false"
                                ></textarea>
                            </div>
                        </div>
                        {move || editor_settings.get().show_minimap.then(|| view! {
                            <canvas
                                node_ref=minimap_ref
                                class="editor-minimap"
                                on:mousedown=on_minimap_down
                                on:mousemove=on_minimap_move
                                on:mouseup=on_minimap_up
                                on:mouseleave=on_minimap_up
                            ></canvas>
                        })}
                    </div>
                </div>
                <div class="divider"></div>
                <div class="pane preview-pane">
                    <div class="pane-header">"预览"</div>
                    <div
                        class="preview-content"
                        node_ref=preview_ref
                        on:scroll=on_preview_scroll
                        inner_html=preview_html
                    ></div>
                </div>
            </div>

            <footer class="status-bar">
                <span>
                    {move || {
                        let (lines, words, chars) = stats();
                        let mode = editor_settings.get().keybinding_mode;
                        let kb = keybinding_state.get();
                        let mode_label = match mode {
                            KeybindingMode::Vim => {
                                let mut label = format!(" · Vim:{}", kb.vim_mode.label());
                                if kb.count > 0 {
                                    label.push_str(&format!(" · {}", kb.count));
                                }
                                if let Some(reg) = kb.macro_recording {
                                    label.push_str(&format!(" · REC @{reg}"));
                                }
                                label
                            }
                            KeybindingMode::Emacs => {
                                if let Some(ref isearch) = kb.emacs_isearch {
                                    let dir = if isearch.forward {
                                        "I-search"
                                    } else {
                                        "I-search backward"
                                    };
                                    format!(" · {dir}:{}", isearch.query)
                                } else {
                                    " · Emacs".to_string()
                                }
                            }
                            KeybindingMode::Standard => String::new(),
                        };
                        format!("行 {lines} · 字 {words} · 字符 {chars}{mode_label}")
                    }}
                </span>
                <span>
                    {move || {
                        let hint = undo_hint.get();
                        let name = filename.get();
                        let modified = unsaved::is_modified(&content.get(), &saved_snapshot.get());
                        let display = if modified {
                            format!("{name} *")
                        } else {
                            name
                        };
                        if hint.is_empty() {
                            display
                        } else {
                            format!("{display} · {hint}")
                        }
                    }}
                    <span class=move || format!("saved-hint {}", if saved_hint.get() { "show" } else { "" })>
                        " · 已自动保存"
                    </span>
                </span>
            </footer>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
