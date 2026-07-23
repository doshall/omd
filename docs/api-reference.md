# API 参考

本文档描述 omd 各模块的公开接口，供开发者扩展或集成时参考。

## 桌面版模块

### `src/main.rs`

程序入口，无公开 API。

```rust
fn main() -> eframe::Result<()>
```

初始化 eframe 窗口并启动 `OmdApp` 事件循环。

---

### `src/app.rs`

#### `OmdApp`

主应用结构体，实现 `eframe::App` trait。

```rust
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OmdApp {
    pub(crate) content: String,
    pub(crate) file_path: Option<PathBuf>,
    // ... 其他字段
}
```

##### 构造

```rust
impl OmdApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self
}
```

- 安装 egui 图片加载器
- 从 eframe storage 恢复持久化状态
- 应用主题

##### `eframe::App` 实现

| 方法 | 说明 |
|------|------|
| `update(&mut self, ctx, frame)` | 每帧 UI 渲染与事件处理 |
| `save(&mut self, storage)` | 关闭时持久化状态 |

##### 内部方法（私有）

| 方法 | 说明 |
|------|------|
| `new_file()` | 清空内容，重置路径 |
| `open_file()` | rfd 文件对话框打开 |
| `save_file()` | 保存到当前路径 |
| `save_file_as()` | 另存为对话框 |
| `insert_formatting(wrapper)` | 工具栏插入格式 |
| `insert_line_prefix(prefix)` | 工具栏插入行前缀 |
| `insert_image()` | 选择图片文件并插入 Markdown |
| `render_toolbar(ui)` | 渲染工具栏 |
| `render_editor(ui)` | 渲染编辑区 |
| `render_preview(ui)` | 渲染预览区 |
| `render_status_bar(ui)` | 渲染状态栏 |

---

### `src/markdown.rs`

Markdown 解析与 egui 预览渲染。

#### `render_preview`

```rust
pub fn render_preview(
    ui: &mut Ui,
    markdown: &str,
    base_path: Option<&Path>,
)
```

将 Markdown 文本渲染到 egui UI 中。

| 参数 | 说明 |
|------|------|
| `ui` | egui UI 上下文 |
| `markdown` | Markdown 源码 |
| `base_path` | 当前文件所在目录，用于解析相对图片路径 |

**渲染元素：**

| Event | 渲染方式 |
|-------|----------|
| Heading H1–H6 | `RichText` 不同字号 + 加粗 |
| Paragraph | `ui.label()` |
| Bold / Italic / Strikethrough | `RichText` 样式 |
| Link | `ui.hyperlink_to()` |
| Image | `egui::Image::new()` via egui_extras |
| Code (inline) | 等宽字体 + 背景色 |
| Code block | 带边框的 Frame + 等宽 Label |
| BlockQuote | 缩进 + 弱色文字 |
| List (ordered/unordered) | `•` 或 `1.` 前缀 |
| Task list | `☑` / `☐` + 可选删除线 |
| Table | `egui::Grid` 条纹表格 |
| Rule | `ui.separator()` |
| HTML | 灰色斜体纯文本 |

#### `wrap_selection`

```rust
pub fn wrap_selection(
    text: &mut String,
    cursor_range: Range<usize>,
    wrapper: &str,
)
```

在选中文字两侧插入格式标记（原地修改）。

| `wrapper` | 展开为 |
|-----------|--------|
| `"**"` | `**选中**` |
| `"*"` | `*选中*` |
| `"~~"` | `~~选中~~` |
| `` "`" `` | `` `选中` `` |
| `"[]()"` | `[选中](url)` |

#### `prefix_lines`

```rust
pub fn prefix_lines(
    text: &mut String,
    cursor_range: Range<usize>,
    prefix: &str,
)
```

在选中行（或光标所在行）的行首插入前缀。

#### `word_count` / `line_count`

```rust
pub fn word_count(text: &str) -> usize
pub fn line_count(text: &str) -> usize
```

| 函数 | 计算方式 |
|------|----------|
| `word_count` | 按空白分隔的非空 token 数 |
| `line_count` | 行数；空文本返回 1 |

---

## Web 版模块

### `web/src/lib.rs`

#### 入口

```rust
#[wasm_bindgen(start)]
pub fn main()
```

- 设置 `console_error_panic_hook`
- `leptos::mount::mount_to_body(App)` 挂载 UI

#### `App` 组件

```rust
#[component]
fn App() -> impl IntoView
```

Leptos 根组件，管理所有响应式状态。

##### 信号（Signals）

| 信号 | 类型 | 说明 |
|------|------|------|
| `content` | `String` | 编辑区文本 |
| `dark_mode` | `bool` | 主题 |
| `view_mode` | `ViewMode` | 视图模式 |
| `filename` | `String` | 当前文件名 |
| `saved_hint` | `bool` | 保存提示 |

##### `ViewMode` 枚举

```rust
enum ViewMode {
    Split,        // 分栏
    EditorOnly,   // 仅编辑
    PreviewOnly,  // 仅预览
}
```

##### 外部 JS 函数

```rust
extern "C" {
    fn omd_render_mermaid();      // 渲染 Mermaid 图表
    fn omd_apply_theme(bool);     // 切换主题
}
```

##### 辅助函数

| 函数 | 说明 |
|------|------|
| `load_storage(key)` | 从 localStorage 读取 |
| `save_storage(key, value)` | 写入 localStorage |
| `download_file(content, filename)` | 触发浏览器下载 |
| `insert_image_into(...)` | 在光标处插入图片 Markdown |
| `textarea_selection(el)` | 获取 textarea 选区 |

---

### `web/src/markdown.rs`

纯函数工具集，无状态。

#### `markdown_to_html`

```rust
pub fn markdown_to_html(markdown: &str) -> String
```

Markdown → HTML 字符串，包含 Mermaid 块转换。

处理流程：
1. pulldown-cmark 解析（启用 GFM 扩展）
2. `html::push_html()` 生成 HTML
3. `transform_mermaid_blocks()` 将 `<pre><code class="language-mermaid">` 转为 `<div class="mermaid">`

#### `wrap_selection`

```rust
pub fn wrap_selection(text: &str, start: usize, end: usize, wrapper: &str) -> String
```

与桌面版逻辑相同，但返回新字符串（不可变）。

#### `prefix_lines`

```rust
pub fn prefix_lines(text: &str, start: usize, end: usize, prefix: &str) -> String
```

#### `insert_at_cursor`

```rust
pub fn insert_at_cursor(text: &str, cursor: usize, insertion: &str) -> String
```

在指定光标位置插入文本。

#### `image_markdown`

```rust
pub fn image_markdown(alt: &str, url: &str) -> String
```

生成 `![alt](url)` 格式，前后带换行。

#### `word_count` / `line_count`

与桌面版相同。

---

## pulldown-cmark 配置

两个版本共享的解析选项：

```rust
let mut options = Options::empty();
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_TASKLISTS);
let parser = Parser::new_ext(markdown, options);
```

### 未启用的扩展

以下 pulldown-cmark 选项**未启用**：

| 选项 | 效果 |
|------|------|
| `ENABLE_FOOTNOTES` | 脚注 |
| `ENABLE_SMART_PUNCTUATION` | 智能标点 |
| `ENABLE_HEADING_ATTRIBUTES` | 标题属性 |
| `ENABLE_MATH` | 数学公式（需额外 feature） |

---

## 扩展指南

### 添加新的 Markdown 元素（桌面版）

1. 在 `PreviewState` 中添加状态字段
2. 在 `handle_event()` 匹配新的 `Event` 变体
3. 渲染为 egui 控件

### 添加新的 Markdown 元素（Web 版）

- 标准 GFM 元素：pulldown-cmark HTML 输出通常自动支持
- 自定义块（如 Mermaid）：在 `markdown_to_html()` 后添加后处理

### 添加新工具栏按钮

**桌面版：** `app.rs` → `render_toolbar()`

**Web 版：** `lib.rs` → `view!` 中的 `.toolbar` 区域

## 相关文档

- [架构设计](architecture.md)
- [开发指南](development.md)
- [配置参考](configuration.md)
