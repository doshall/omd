# 更新日志

本文件记录 omd 项目的所有重要变更，格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

## [未发布]

## [0.5.0] - 2026-07-24

### 新增 — 格式与文件

- **脚注 / TOC**：共享 `omd-common` 解析，预览与导出 HTML 自动生成目录
- **YAML Front Matter**：支持 `title` / `description`，导出标题优先使用元数据
- **桌面多标签页**：标签栏 + eframe 持久化
- **最近打开文件**：桌面 File 菜单；Web 最近文件栏（跳转已开标签）
- **图片点击放大**：Web 灯箱；桌面预览弹窗

### 变更

- 新增 `omd-common` crate；示例 `demo/default.md` 含脚注与 Front Matter
- 版本号升至 **0.5.0**

## [0.4.1] - 2026-07-24

### 变更

- **示例文档**：统一为 `demo/default.md`，含甘特图、饼图、状态图、类图、ER 图等复杂 Mermaid

## [0.4.0] - 2026-07-24

### 新增 — 文档与格式

- **LaTeX 数学公式**（Web / Android / 导出 HTML）：KaTeX 渲染 `$...$` 与 `$$...$$`；桌面预览显示 TeX 源码样式
- **导出 PDF**（桌面 / Web / Android）：生成打印优化 HTML，浏览器「打印 → 另存为 PDF」
- **Web 多标签页**：标签栏新建 / 切换 / 关闭，`localStorage` 持久化（最多 20 个标签）

### 修复

- **Web Mermaid 主题切换**：切换浅色/深色时用 `mermaid.render()` 从 `data-mermaid-source` 重绘，避免甘特图等图表出现 `Syntax error in text`

### 变更

- `scripts/fetch-web-assets.sh` 增加 KaTeX 离线资源
- 版本号升至 **0.4.0**
- Service Worker 缓存版本升至 `v4`

## [0.3.1] - 2026-07-24

### 新增

- **Web / Android 未保存确认**：离开页面前提示、新建/打开前确认；下载后视为已保存；状态栏文件名显示 `*`
- **Android 退出确认**：返回键检测未下载修改并弹窗

### 修复

- **Web Mermaid 渲染**：修复代码块二次 HTML 转义导致 `Syntax error in text`（`-->` 被错误编码为 `--&amp;gt;`）
- 导出 HTML 中的 Mermaid 块同步修复（桌面 / Web）

## [0.3.0] - 2026-07-23

### 新增 — 桌面版

- **拖拽插入图片**：将图片文件拖入编辑区，在光标处插入 `![alt](path)`
- **关闭前未保存确认**：关闭窗口、退出、新建或打开文件时，若有未保存修改则提示保存 / 不保存 / 取消
- **自动保存到磁盘**：设置项可开启，按延迟将已打开文件写入磁盘（仅对已保存路径的文件生效）

### 新增 — 导出与 Web

- **导出 HTML**（桌面 / Web）：将 Markdown 导出为独立 HTML 文件，含语法高亮与 Mermaid 图表
- **PWA**（Web）：`manifest.webmanifest`、Service Worker 离线缓存、应用图标，可安装到主屏幕
- **CI**：`ci.yml` PR/主分支测试与 Web 构建；`pages.yml` 自动部署 Web 到 GitHub Pages

### 变更

- 桌面版 **File → Export HTML…** 与工具栏 **📤** 按钮
- Web 版头部 **导出 HTML** 按钮（与「下载」并列）

### 修复

- **CI / Pages / Release**：构建前自动下载 `mermaid.min.js`，修复 Web 构建失败
- **GitHub Pages**：https://doshall.github.io/omd/ 已上线

## [0.2.0] - 2026-07-23

### 新增 — 编辑体验（A 类）

- **查找 / 替换**（桌面 / Web / Android）：`Ctrl+F` / `Ctrl+H`，支持区分大小写、全部替换
- **行号与当前行高亮**（桌面 / Web / Android）：编辑区左侧行号栏，光标所在行背景高亮
- **编辑区与预览区同步滚动**（桌面 / Web / Android）：分栏模式下按滚动比例联动
- **预览区代码块语法高亮**（桌面 / Web / Android）：基于 syntect / highlight.js
- **编辑区语法高亮**（桌面 / Web / Android）：Markdown 着色，设置项控制，默认关闭
- **滚动条 Minimap**（桌面 / Web / Android）：编辑区右侧文档缩略导航，点击或拖拽定位
- **编辑器设置面板**（桌面 / Web / Android）：行号、Minimap、同步滚动、字号行高、专注模式等可配置项
- **撤销 / 重做状态提示**（桌面 / Web / Android）：设置项控制，状态栏提示

### 新增 — Vim / Emacs 键位

- **Vim 模式**（桌面 / Web / Android）：Normal / Insert / Visual / Visual Block / Command
- Visual Block：`Ctrl+V` 列选、`y`/`d`/`c`/`p`/`P`/`~`/`u`/`U`/`>`/`<`、`I`/`A` 块插入
- Ex 命令：`:g/pat/d`、`:g/pat/s`、`:g/pat/norm`（含 `@a` 宏回放）、`:1,5d`、`:reg` 等
- 系统剪贴板寄存器 `"+` / `"*`（可配置）
- **Emacs 模式**：`Ctrl+S`/`Ctrl+R` 增量搜索（全匹配 overlay 高亮）、`Ctrl+Space` 标记等
- 文档：[Vim 键位参考](docs/vim-keybindings.md)

### 新增 — 平台专项

- **桌面版 Mermaid**：纯 Rust 渲染（mermaid-rs-renderer + resvg）
- **桌面版图片粘贴**：`Ctrl+V` 剪贴板截图（Base64 嵌入）
- 一键构建脚本 `scripts/build-android.sh`
- Mermaid.js / highlight.js 离线打包（`web/assets/`）
- [Android 版指南](docs/android.md)

### 修复

- **Web 版页面空白**：修正 Trunk 静态资源路径（`mermaid.min.js` 等）

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

[未发布]: https://github.com/doshall/omd/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/doshall/omd/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/doshall/omd/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/doshall/omd/releases/tag/v0.1.0
