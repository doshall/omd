# 更新日志

本文件记录 omd 项目的所有重要变更，格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

## [未发布]

### 新增
- **查找 / 替换**（桌面 / Web / Android）：`Ctrl+F` / `Ctrl+H`，支持区分大小写、全部替换
- **行号与当前行高亮**（桌面 / Web / Android）：编辑区左侧行号栏，光标所在行背景高亮
- **编辑区与预览区同步滚动**（桌面 / Web / Android）：分栏模式下按滚动比例联动
- **预览区代码块语法高亮**（桌面 / Web / Android）：基于 syntect / highlight.js
- **编辑器设置面板**（桌面 / Web / Android）：行号、Minimap、同步滚动、字号行高、专注模式等可配置项
- **Vim / Emacs 键位模式**：`:g/pat/norm`、Visual Block `c` 列修改、Emacs `Ctrl+S`/`Ctrl+R` 增量搜索
- **滚动条 Minimap**（桌面 / Web / Android）：编辑区右侧文档缩略导航，点击或拖拽定位
- **桌面版 Mermaid**：纯 Rust 渲染（mermaid-rs-renderer + resvg）
- **桌面版图片粘贴**：`Ctrl+V` 剪贴板截图（Base64 嵌入）
- 一键构建脚本 `scripts/build-android.sh`
- Mermaid.js 离线打包（`web/assets/mermaid.min.js`）
- [Android 版指南](docs/android.md)

### 变更
- 合并所有 feature 分支至 `main`，三版本统一维护
- Web 版 `Trunk.toml` 设置 `public_url = "./"` 支持 Android assets 加载
- Web 版支持 `omd-web-filename` localStorage 键（Android 打开文件时保留文件名）

## [0.1.0] - 2026-07-23

### 新增 — 桌面版
- 基于 egui/eframe 的原生 GUI Markdown 编辑器
- 左右分栏实时预览，可拖拽调整比例
- 文件操作：新建、打开、保存、另存为
- 格式化工具栏（粗体、斜体、删除线、代码、链接、图片、标题、列表、引用）
- 深色 / 浅色主题切换
- 快捷键：`Ctrl+N` / `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S`
- 状态栏：行数、字数、字符数、文件路径
- 图片预览（本地路径与网络 URL，via egui_extras）
- 工具栏插入本地图片
- 应用状态持久化（窗口布局、主题偏好）
- 默认示例文档展示全部功能

### 新增 — Web 版
- 基于 Leptos + WASM 的浏览器 Markdown 编辑器
- 实时 HTML 预览（pulldown-cmark）
- Mermaid 流程图、时序图渲染
- 图片：URL 插入、本地上传（Base64）、粘贴截图、拖拽
- 自动保存到 localStorage
- 导入 / 下载 `.md` 文件
- 三种视图：分栏 / 仅编辑 / 仅预览
- 深色 / 浅色主题（Mermaid 同步适配）
- 移动端响应式布局
- 默认示例文档展示全部功能

### 技术栈
- 桌面：eframe 0.29、egui 0.29、egui_extras、mermaid-rs-renderer、resvg、arboard、pulldown-cmark 0.12、rfd 0.15
- Web：Leptos 0.7、pulldown-cmark 0.12、Trunk 0.21、Mermaid.js 11

[未发布]: https://github.com/doshall/omd/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/doshall/omd/releases/tag/v0.1.0
