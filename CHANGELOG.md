# 更新日志

本文件记录 omd 项目的所有重要变更，格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

## [未发布]

### 新增
- 完善文档体系：配置参考、API 参考、安全说明、版本对比
- GitHub Issue/PR 模板
- SECURITY.md 安全政策
- 文档中心按角色分类导航、术语表、快速参考卡

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
- 桌面：eframe 0.29、egui 0.29、egui_extras、pulldown-cmark 0.12、rfd 0.15
- Web：Leptos 0.7、pulldown-cmark 0.12、Trunk 0.21、Mermaid.js 11

[未发布]: https://github.com/doshall/omd/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/doshall/omd/releases/tag/v0.1.0
