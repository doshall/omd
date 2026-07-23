# omd

**omd** 是一款使用 Rust 编写的轻量级 Markdown 编辑器。

提供两个版本：

| 版本 | 目录 | 适用场景 |
|------|------|----------|
| **桌面版** | 项目根目录 | 本地电脑，原生 GUI |
| **Web 版** | `web/` | 浏览器、手机 Cursor |

---

## Web 版（浏览器 / 手机）

在浏览器中直接使用，无需安装桌面环境。

### 功能

- 实时 Markdown 预览（**图片**、**Mermaid 图表**）
- 插入图片：工具栏 🖼 / 粘贴截图 / 拖拽上传 / URL
- 导入 / 下载 `.md` 文件
- 深色 / 浅色主题
- 分栏 / 仅编辑 / 仅预览三种视图
- 移动端响应式布局

### 运行

```bash
# 安装构建工具（首次）
cargo install trunk --locked
rustup target add wasm32-unknown-unknown

# 开发模式（热重载）
cd web
trunk serve
# 浏览器打开 http://127.0.0.1:8080

# 发布构建
trunk build --release
# 静态文件在 web/dist/，可部署到任意静态托管
```

### 部署

将 `web/dist/` 目录上传到 GitHub Pages、Cloudflare Pages、Vercel 等静态托管即可。

---

## 桌面版（原生 GUI）

基于 [egui](https://github.com/emilk/egui) 构建，需要桌面图形环境。

### 功能

- 实时预览、文件新建/打开/保存
- 格式化工具栏、深色/浅色主题
- 快捷键：`Ctrl+N` / `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S`

### 运行

```bash
cargo run

# 发布构建
cargo build --release
./target/release/omd
```

### 环境要求

- Rust 1.85+
- Linux / macOS / Windows 桌面环境（Wayland / X11 / 原生窗口）

---

## 项目结构

```
omd/
├── Cargo.toml          # 桌面版依赖
├── src/                # 桌面版源码
│   ├── main.rs
│   ├── app.rs
│   └── markdown.rs
└── web/                # 浏览器版
    ├── Cargo.toml
    ├── Trunk.toml
    ├── index.html
    ├── style.css
    └── src/
        ├── lib.rs
        └── markdown.rs
```

## 技术栈

| 组件 | 桌面版 | Web 版 |
|------|--------|--------|
| GUI | eframe / egui | Leptos (WASM) |
| Markdown | pulldown-cmark | pulldown-cmark (HTML) |
| 文件 | rfd 原生对话框 | 浏览器导入/下载 |
| 构建 | cargo | trunk |

## 许可证

MIT License — 详见 [LICENSE](LICENSE)。
