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

const DEFAULT_CONTENT: &str = r#"# Welcome to omd Web

**omd** 浏览器版 Markdown 编辑器，可在手机与电脑上使用。

## 功能

- 实时预览
- 自动保存到浏览器
- 导入 / 导出 `.md` 文件
- 深色 / 浅色主题

## 示例

```rust
fn main() {
    println!("Hello, omd!");
}
```

| 平台 | 支持 |
|------|------|
| 浏览器 | ✅ |
| 手机 | ✅ |

> 在手机上也能愉快地写 Markdown！
"#;

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
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(html) = doc.document_element() {
            let _ = html.set_attribute("data-theme", if dark { "dark" } else { "light" });
        }
    }
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

            <div class="toolbar">
                <button class="btn btn-icon" on:click=move |_| insert_format("**") title="粗体">"B"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("*") title="斜体">"I"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("~~") title="删除线">"S"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("`") title="行内代码">"</>"</button>
                <button class="btn btn-icon" on:click=move |_| insert_format("[]()") title="链接">"🔗"</button>
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
                        placeholder="在此输入 Markdown..."
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
