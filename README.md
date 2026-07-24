# omd

**omd** 是一款使用 Rust 编写的轻量级 Markdown 编辑器，提供**桌面版**、**Web 版**和 **Android 版**（当前版本 **v0.9.1**）。

**在线体验**：https://doshall.github.io/omd/

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](rust-toolchain.toml)

## 特性

- 实时 Markdown 预览（图片、表格、任务列表、Mermaid 图表）
- 深色 / 浅色主题
- 格式化工具栏与快捷键
- 查找 / 替换、行号、Minimap、同步滚动、语法高亮
- Vim / Emacs 可选键位模式
- 图片插入（URL、本地文件、粘贴、拖拽）
- 脚注 / TOC、YAML Front Matter、LaTeX（KaTeX）、导出 HTML / PDF
- PlantUML / Graphviz 图表、自定义预览 CSS
- 多语言界面（中/英）、Web 拼写检查与无障碍（ARIA）
- 多标签页、最近文件、项目文件夹侧边栏、图片点击放大
- 桌面版：文件管理、自动保存、关闭前确认
- Web 版：PWA 可安装、localStorage + IndexedDB 自动保存

## 版本对比

| | 桌面版 | Web 版 | Android 版 |
|---|--------|--------|------------|
| **目录** | 项目根目录 | `web/` | `android/` |
| **适用场景** | 本地电脑 | 浏览器 | 手机 APK |
| **技术栈** | egui / eframe | Leptos / WASM | WebView + WASM |
| **Mermaid** | ✅ | ✅ | ✅ |
| **图片粘贴** | ✅ | ✅ | ✅ |
| **离线** | ✅ | ✅ PWA | ✅ |
| **自动保存** | ✅ 磁盘（可配置） | localStorage + IndexedDB | localStorage + IndexedDB |
| **导出 HTML** | ✅ | ✅ | ✅ |
| **Release APK** | — | — | ✅ GitHub Releases |

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

### Android 版

```bash
export ANDROID_HOME=/path/to/android-sdk
echo "sdk.dir=$ANDROID_HOME" > android/local.properties
./scripts/build-android.sh
# 安装: adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

详见 [Android 版指南](docs/android.md)。要求 Android SDK API 35、JDK 17+。

## 文档

完整文档见 [docs/README.md](docs/README.md)（文档中心）。

| 分类 | 文档 |
|------|------|
| **入门** | [用户指南](docs/user-guide.md) · [桌面版](docs/desktop.md) · [Web 版](docs/web.md) · [Android 版](docs/android.md) |
| **参考** | [Markdown 语法](docs/markdown-syntax.md) · [版本对比](docs/comparison.md) · [配置](docs/configuration.md) |
| **开发** | [架构](docs/architecture.md) · [开发指南](docs/development.md) · [API](docs/api-reference.md) |
| **运维** | [部署指南](docs/deployment.md) · [安全说明](docs/security.md) |
| **其他** | [FAQ](docs/faq.md) · [发布说明](docs/release-notes.md) · [路线图](docs/roadmap.md) · [贡献指南](CONTRIBUTING.md) · [CHANGELOG](CHANGELOG.md) |

## 项目结构

```
omd/
├── src/                    # 桌面版（egui）
│   ├── main.rs
│   ├── app.rs
│   ├── markdown.rs
│   ├── mermaid.rs          # Mermaid 图表渲染
│   └── clipboard.rs        # 剪贴板图片
├── web/                    # Web 版（Leptos + WASM）
├── android/                # Android 版（WebView APK）
├── scripts/build-android.sh
├── docs/                   # 项目文档
├── Cargo.toml
└── README.md
```

## 技术栈

| 组件 | 桌面版 | Web 版 | Android 版 |
|------|--------|--------|------------|
| GUI | [eframe](https://github.com/emilk/egui) / [egui](https://github.com/emilk/egui) | [Leptos](https://leptos.dev/) | Android WebView |
| Markdown | [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) | pulldown-cmark (HTML) | 同 Web 版 |
| 图表 | mermaid-rs-renderer | [Mermaid.js](https://mermaid.js.org/) | Mermaid.js（离线打包） |
| 构建 | cargo | [Trunk](https://trunkrs.dev/) | Gradle + `build-android.sh` |

## 快捷键（桌面版）

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+N` | 新建文件 |
| `Ctrl+O` | 打开文件 |
| `Ctrl+S` | 保存 |
| `Ctrl+Shift+S` | 另存为 |
| `Ctrl+V` | 粘贴剪贴板图片 |

## 贡献

欢迎贡献！请阅读 [贡献指南](CONTRIBUTING.md) 和 [开发指南](docs/development.md)。

## 许可证

[MIT License](LICENSE) © 2026 omd contributors
