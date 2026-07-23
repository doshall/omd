# 配置参考

本文档汇总 omd 项目中所有可配置项，包括构建配置、运行时参数和持久化存储。

## Rust 工具链

### `rust-toolchain.toml`

```toml
[toolchain]
channel = "stable"
```

进入项目目录时，`rustup` 自动切换到 stable 工具链。当前最低要求 **Rust 1.85+**。

---

## 桌面版配置

### `Cargo.toml`

#### 包信息

| 字段 | 值 | 说明 |
|------|-----|------|
| `name` | `omd` | 包名与二进制名 |
| `version` | `0.3.0` | 语义化版本 |
| `edition` | `2021` | Rust 版本 |
| `license` | `MIT` | 开源许可证 |

#### 依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `eframe` | 0.29 | 应用框架（窗口、事件循环、持久化） |
| `egui` | 0.29 | GUI 控件 |
| `egui_extras` | 0.29 | 图片加载（`image` feature） |
| `pulldown-cmark` | 0.12 | Markdown 解析 |
| `rfd` | 0.15 | 原生文件对话框 |
| `serde` | 1.x | 状态序列化 |

#### Release 优化

```toml
[profile.release]
lto = true          # 链接时优化，减小体积、提升性能
codegen-units = 1   # 单编译单元，配合 LTO 效果更好
strip = true        # 剥离调试符号
```

### 窗口配置（`src/main.rs`）

```rust
eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])      // 初始窗口大小
        .with_min_inner_size([640.0, 480.0])   // 最小窗口大小
        .with_title("omd — Markdown Editor"),
    ..Default::default()
}
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| 初始宽度 | 1200 px | 首次启动窗口宽度 |
| 初始高度 | 800 px | 首次启动窗口高度 |
| 最小宽度 | 640 px | 窗口可缩放的最小宽度 |
| 最小高度 | 480 px | 窗口可缩放的最小高度 |

### 应用状态（`OmdApp`）

通过 eframe persistence 自动序列化保存：

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `content` | `String` | 示例文档 | 编辑区 Markdown 文本 |
| `file_path` | `Option<PathBuf>` | `None` | 当前文件路径 |
| `modified` | `bool` | `false` | 是否有未保存修改 |
| `dark_mode` | `bool` | `true` | 深色主题开关 |
| `show_preview` | `bool` | `true` | 预览区显示开关 |
| `split_ratio` | `f32` | `0.5` | 编辑区宽度比例（0.2–0.8） |
| `status_message` | `String` | 空 | 状态栏临时消息 |
| `status_timer` | `f32` | `0.0` | 消息显示倒计时（秒） |
| `editor_settings` | `EditorSettings` | 见下表 | 编辑器行为与外观设置 |

#### `EditorSettings`（桌面版持久化）

| 字段 | 默认值 | 说明 |
|------|--------|------|
| `show_line_numbers` | `true` | 显示行号栏 |
| `highlight_current_line` | `true` | 高亮当前行 |
| `show_minimap` | `true` | 显示 Minimap |
| `sync_scroll` | `true` | 分栏时同步滚动 |
| `preview_syntax_highlight` | `true` | 预览区代码块语法高亮 |
| `editor_syntax_highlight` | `false` | 编辑区 Markdown 语法高亮 |
| `focus_mode` | `false` | 专注模式（隐藏工具栏与预览） |
| `editor_font_size` | `14.0` | 编辑区字号（px） |
| `editor_line_height` | `1.6` | 编辑区行高倍数 |
| `preview_font_size` | `15.0` | 预览区字号（px） |
| `show_undo_redo_hint` | `true` | Ctrl+Z / Ctrl+Y 状态栏提示 |
| `keybinding_mode` | `"standard"` | 键位模式：`standard` / `vim` / `emacs` |
| `vim_show_block_highlight` | `true` | Vim 模式下高亮 Visual Block 选区 |
| `vim_use_system_clipboard` | `true` | Vim 模式下 `"` / `"a` 与系统剪贴板寄存器 `"+` / `"*` 同步 |
| `auto_save_enabled` | `true` | 自动保存到磁盘（仅已保存路径的文件） |
| `auto_save_interval_secs` | `30` | 停止编辑后多少秒触发自动保存（5–300） |

打开方式：**View → Settings…**（`F11` 切换专注模式，`Esc` 退出）

#### 文件操作

| 功能 | 说明 |
|------|------|
| 拖拽插入图片 | 将 PNG/JPG/GIF/WebP/SVG/BMP 拖入编辑区 |
| 未保存确认 | 关闭窗口、退出、新建、打开时若有修改会弹出对话框 |
| 自动保存 | 对已保存到磁盘的文件，在设置延迟后自动写入 |
| 导出 HTML | **File → Export HTML…** 或工具栏 **📤**，生成独立 HTML 文件 |

#### Vim 模式参考

启用 `keybinding_mode: "vim"` 后可用：

| 类别 | 功能 |
|------|------|
| 移动 | `hjkl`、`w`/`b`/`e`、`0`/`$`、`gg`/`G`、数字前缀（如 `3j`） |
| 编辑 | `dd`/`yy`/`dw`/`cw`/`cc`/`D`、`p`/`P`、`>>`/`<<` |
| Visual Block | `Ctrl+V` 进入，`hjkl` 扩展，`y`/`d` 复制或删除列块 |
| 命令行 | `:`（桌面 `Shift+;`）输入 Ex 命令 |
| 寄存器 | `"a`…`"z`、`"+`、`"*`、`"0`、`:reg` 查看 |
| 宏 | `qa` 录制 → `q` 停止 → `@a` / `@@` 回放 |
| Ex 命令 | `:w` `:q` `:42` `:1,5d` `:g/pat/d` `:v/pat/d` `:%s/old/new/g` `:set number` |

详见 [Vim 键位参考](vim-keybindings.md)。

#### 持久化存储位置

eframe 将状态保存到系统应用数据目录：

| 平台 | 路径 |
|------|------|
| Linux | `~/.local/share/omd/` |
| macOS | `~/Library/Application Support/omd/` |
| Windows | `%APPDATA%\omd\` |

存储格式为 RON（Rusty Object Notation）。

### Markdown 解析选项

桌面版和 Web 版均启用以下 pulldown-cmark 扩展：

```rust
options.insert(Options::ENABLE_STRIKETHROUGH);  // ~~删除线~~
options.insert(Options::ENABLE_TABLES);          // | 表格 |
options.insert(Options::ENABLE_TASKLISTS);      // - [x] 任务
```

---

## Web 版配置

### `web/Cargo.toml`

#### 包信息

| 字段 | 值 |
|------|-----|
| `name` | `omd-web` |
| `crate-type` | `["cdylib", "rlib"]` |

#### 主要依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `leptos` | 0.7 (csr) | WASM UI 框架 |
| `pulldown-cmark` | 0.12 (html) | Markdown → HTML |
| `wasm-bindgen` | 0.2 | Rust/JS 互操作 |
| `web-sys` | 0.3 | 浏览器 API |
| `gloo-timers` | 0.3 | 异步定时器 |

### `web/Trunk.toml`

```toml
[build]
target = "index.html"    # 构建入口 HTML
dist = "dist"            # 输出目录

[watch]
ignore = ["dist"]        # 监视时忽略的目录

[serve]
addresses = ["0.0.0.0"]  # 开发服务器监听地址（允许局域网访问）
port = 8080              # 开发服务器端口
```

#### 子路径部署

若部署在非根路径（如 `/omd/`），添加：

```toml
[build]
public_url = "/omd/"
```

### PWA（渐进式 Web 应用）

| 文件 | 说明 |
|------|------|
| `manifest.webmanifest` | 应用名称、图标、主题色、`standalone` 显示模式 |
| `sw.js` | Service Worker，缓存静态资源以支持离线访问 |
| `assets/icon-192.png` / `icon-512.png` | 主屏幕图标 |
| `assets/apple-touch-icon.png` | iOS 主屏幕图标 |

安装方式：在支持的浏览器中打开 Web 版，选择「安装应用」或「添加到主屏幕」。

### localStorage 键

Web 版使用以下 localStorage 键持久化状态：

| 键名 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `omd-web-content` | `string` | 示例文档 | Markdown 编辑内容 |
| `omd-web-theme` | `string` | `"dark"` | 主题：`"dark"` 或 `"light"` |
| `omd-web-view` | `string` | `"split"` | 视图：`"split"` / `"editor"` / `"preview"` |
| `omd-web-settings` | `string` (JSON) | 默认见 `EditorSettings` | 编辑器设置（行号、Minimap、字号等） |

#### `EditorSettings`（Web / Android，JSON）

与桌面版字段相同，通过界面 **⚙ 设置** 修改并自动保存。

#### 清除存储

```javascript
// 浏览器控制台执行
localStorage.removeItem('omd-web-content');
localStorage.removeItem('omd-web-theme');
localStorage.removeItem('omd-web-view');
```

### 外部 CDN 依赖

`web/index.html` 从 CDN 加载：

| 资源 | URL | 用途 |
|------|-----|------|
| Mermaid.js 11 | `cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js` | 图表渲染 |

离线部署需替换为本地文件，参见 [部署指南](deployment.md#mermaid-cdn-离线化)。

### Mermaid 初始化配置

```javascript
mermaid.initialize({
    startOnLoad: false,
    theme: 'dark' | 'default',   // 跟随应用主题
    securityLevel: 'loose',
});
```

### CSS 主题变量

定义在 `web/style.css`：

| 变量 | 浅色值 | 深色值 | 用途 |
|------|--------|--------|------|
| `--bg` | `#f8f9fa` | `#1a1b1e` | 页面背景 |
| `--surface` | `#ffffff` | `#25262b` | 面板背景 |
| `--border` | `#dee2e6` | `#373a40` | 边框 |
| `--text` | `#212529` | `#e9ecef` | 主文字 |
| `--text-muted` | `#6c757d` | `#adb5bd` | 次要文字 |
| `--accent` | `#0d6efd` | `#4dabf7` | 强调色 |
| `--toolbar-bg` | `#e9ecef` | `#2c2e33` | 工具栏背景 |
| `--preview-code-bg` | `#f1f3f5` | `#2c2e33` | 代码块背景 |

切换方式：`document.documentElement.setAttribute('data-theme', 'dark')`

### 响应式断点

| 断点 | 布局变化 |
|------|----------|
| `> 768px` | 左右分栏 |
| `≤ 768px` | 上下分栏，按钮放大至 36px |

---

## 环境变量

### 构建时

| 变量 | 影响 | 建议 |
|------|------|------|
| `NO_COLOR` | 与 trunk 0.21 冲突 | 使用 `env -u NO_COLOR trunk serve` |
| `RUST_LOG` | 桌面版日志级别 | `RUST_LOG=debug cargo run` |
| `RUST_BACKTRACE` | panic 回溯 | `RUST_BACKTRACE=1 cargo run` |

### 运行时（桌面版）

| 变量 | 说明 |
|------|------|
| `WAYLAND_DISPLAY` | Wayland 显示服务器 |
| `DISPLAY` | X11 显示服务器（如 `:0`） |

---

## 相关文档

- [开发指南](development.md)
- [架构设计](architecture.md)
- [部署指南](deployment.md)
