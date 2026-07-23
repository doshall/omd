# omd

**omd** 是一款使用 Rust 编写的轻量级 Markdown 编辑器，基于 [egui](https://github.com/emilk/egui) 构建，提供实时预览与文件管理功能。

## 功能特性

- **实时预览** — 编辑区与预览区并排显示，输入即时渲染
- **文件操作** — 新建、打开、保存、另存为（支持 `.md`、`.markdown`、`.txt`）
- **格式化工具栏** — 粗体、斜体、删除线、行内代码、链接、标题、列表、引用
- **主题切换** — 深色 / 浅色模式
- **快捷键**
  - `Ctrl+N` — 新建文件
  - `Ctrl+O` — 打开文件
  - `Ctrl+S` — 保存
  - `Ctrl+Shift+S` — 另存为
- **状态栏** — 行数、字数、字符数及当前文件路径

## 环境要求

- Rust 1.85+（推荐使用 stable 工具链）
- Linux 桌面环境（Wayland / X11）及 OpenGL 支持

## 构建与运行

```bash
# 克隆仓库后
cargo run

# 发布构建
cargo build --release
./target/release/omd
```

## 项目结构

```
omd/
├── Cargo.toml          # 项目依赖
├── rust-toolchain.toml # Rust 工具链配置
└── src/
    ├── main.rs         # 程序入口
    ├── app.rs          # 主应用逻辑与 UI
    └── markdown.rs     # Markdown 解析与预览渲染
```

## 技术栈

| 组件 | 用途 |
|------|------|
| [eframe](https://github.com/emilk/egui) / [egui](https://github.com/emilk/egui) | 跨平台 GUI 框架 |
| [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) | Markdown 解析 |
| [rfd](https://github.com/PolyMeilex/rfd) | 原生文件对话框 |

## 许可证

MIT License — 详见 [LICENSE](LICENSE)。
