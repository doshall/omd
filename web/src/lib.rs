mod markdown;

use leptos::ev::Event;
use leptos::html::{Input, Textarea};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, HtmlInputElement, HtmlTextAreaElement};

const STORAGE_CONTENT: &str = "omd-web-content";
const STORAGE_THEME: &str = "omd-web-theme";
const STORAGE_VIEW: &str = "omd-web-view";

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

fn download_file(content: &str, filename: &str) {
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
    let _ = blob_props.set_type("text/markdown;charset=utf-8");

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

    apply_theme(initial_dark);

    let (content, set_content) = signal(initial_content);
    let (dark_mode, set_dark_mode) = signal(initial_dark);
    let (view_mode, set_view_mode) = signal(initial_view);
    let (filename, set_filename) = signal("document.md".to_string());
    let (saved_hint, set_saved_hint) = signal(false);
    let textarea_ref = NodeRef::<Textarea>::new();
    let file_input_ref = NodeRef::<Input>::new();
    let image_input_ref = NodeRef::<Input>::new();

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

        set_saved_hint.set(true);
        let set_saved_hint = set_saved_hint.clone();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1500).await;
            set_saved_hint.set(false);
        });
    });

    // Render mermaid diagrams after preview updates
    Effect::new(move |_| {
        let _ = content.get();
        let _ = dark_mode.get();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(50).await;
            omd_render_mermaid();
        });
    });

    let preview_html = move || markdown::markdown_to_html(&content.get());

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
                let name = file.name();
                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                    if let Ok(result) = reader_clone.result() {
                        if let Some(text) = result.as_string() {
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
        <div id="app">
            <header class="header">
                <h1><span>"omd"</span>" Web"</h1>
                <div class="header-actions">
                    <button class="btn" on:click=move |_| {
                        set_content.set(String::new());
                        set_filename.set("document.md".to_string());
                    }>"新建"</button>
                    <button class="btn" on:click=move |_| {
                        if let Some(input) = file_input_ref.get() {
                            input.click();
                        }
                    }>"打开"</button>
                    <button class="btn btn-primary" on:click=move |_| {
                        download_file(&content.get(), &filename.get());
                    }>"下载"</button>
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

            <div class=move || format!("main {}", view_mode.get().css_class())>
                <div class="pane editor-pane">
                    <div class="pane-header">"编辑"</div>
                    <textarea
                        node_ref=textarea_ref
                        prop:value=move || content.get()
                        on:input=move |ev| {
                            let el: HtmlTextAreaElement = ev.target().unwrap().unchecked_into();
                            set_content.set(el.value());
                        }
                        on:paste=on_paste
                        on:drop=on_drop
                        on:dragover=on_drag_over
                        placeholder="在此输入 Markdown，可粘贴或拖入图片..."
                        spellcheck="false"
                    ></textarea>
                </div>
                <div class="divider"></div>
                <div class="pane preview-pane">
                    <div class="pane-header">"预览"</div>
                    <div class="preview-content" inner_html=preview_html></div>
                </div>
            </div>

            <footer class="status-bar">
                <span>
                    {move || {
                        let (lines, words, chars) = stats();
                        format!("行 {lines} · 字 {words} · 字符 {chars}")
                    }}
                </span>
                <span>
                    {move || filename.get()}
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
