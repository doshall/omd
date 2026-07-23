# omd

**omd** 是一款使用 Rust 编写的轻量级 Markdown 编辑器，提供**桌面版**和 **Web 版**两个版本。

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](rust-toolchain.toml)

## 截图与特性

- 实时 Markdown 预览（图片、表格、任务列表、Mermaid 图表）
- 深色 / 浅色主题
- 格式化工具栏与快捷键
- 图片插入（URL、本地文件、粘贴、拖拽）
- 桌面版原生文件管理 / Web 版自动保存

## 版本对比

| | 桌面版 | Web 版 |
|---|--------|--------|
| **目录** | 项目根目录 | `web/` |
| **适用场景** | 本地电脑 | 浏览器、手机 |
| **技术栈** | egui / eframe | Leptos / WASM |
| **Mermaid** | ❌ | ✅ |
| **图片粘贴** | ❌ | ✅ |
| **自动保存** | 手动 `Ctrl+S` | localStorage |

## 快速开始

### 桌面版

```bash
cargo run

# 发布构建
cargo build --release
./target/release/omd
```

**要求**：Rust 1.85+、桌面图形环境（Wayland / X11 / 原生窗口）

### Web 版

```bash
cargo install trunk --locked
rustup target add wasm32-unknown-unknown

cd web
trunk serve
# 浏览器打开 http://127.0.0.1:8080
```

**发布构建**：`trunk build --release` → 部署 `web/dist/`

## 文档

完整文档见 [docs/README.md](docs/README.md)（文档中心）。

| 分类 | 文档 |
|------|------|
| **入门** | [用户指南](docs/user-guide.md) · [桌面版](docs/desktop.md) · [Web 版](docs/web.md) |
| **参考** | [Markdown 语法](docs/markdown-syntax.md) · [版本对比](docs/comparison.md) · [配置](docs/configuration.md) |
| **开发** | [架构](docs/architecture.md) · [开发指南](docs/development.md) · [API](docs/api-reference.md) |
| **运维** | [部署指南](docs/deployment.md) · [安全说明](docs/security.md) |
| **其他** | [FAQ](docs/faq.md) · [路线图](docs/roadmap.md) · [贡献指南](CONTRIBUTING.md) · [CHANGELOG](CHANGELOG.md) |

## 项目结构

```
omd/
├── src/                    # 桌面版源码
│   ├── main.rs             # 入口
│   ├── app.rs              # 应用逻辑与 UI
│   └── markdown.rs         # egui 预览渲染
├── web/                    # Web 版（独立 Cargo 项目）
│   ├── src/
│   │   ├── lib.rs          # Leptos 应用
│   │   └── markdown.rs     # HTML 转换
│   ├── index.html
│   ├── style.css
│   └── Trunk.toml
├── docs/                   # 项目文档
├── Cargo.toml
├── CHANGELOG.md
├── CONTRIBUTING.md
└── README.md
```

## 技术栈

| 组件 | 桌面版 | Web 版 |
|------|--------|--------|
| GUI 框架 | [eframe](https://github.com/emilk/egui) / [egui](https://github.com/emilk/egui) | [Leptos](https://leptos.dev/) |
| Markdown 解析 | [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) | pulldown-cmark (HTML) |
| 图片加载 | [egui_extras](https://github.com/emilk/egui/tree/master/crates/egui_extras) | 浏览器原生 |
| 图表 | — | [Mermaid.js](https://mermaid.js.org/) |
| 文件对话框 | [rfd](https://github.com/PolyMeilex/rfd) | 浏览器 File API |
| 构建工具 | cargo | [Trunk](https://trunkrs.dev/) |

## 快捷键（桌面版）

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+N` | 新建文件 |
| `Ctrl+O` | 打开文件 |
| `Ctrl+S` | 保存 |
| `Ctrl+Shift+S` | 另存为 |

## 贡献

欢迎贡献！请阅读 [贡献指南](CONTRIBUTING.md) 和 [开发指南](docs/development.md)。

## 许可证

[MIT License](LICENSE) © 2026 omd contributors
