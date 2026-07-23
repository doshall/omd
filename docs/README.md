# omd 文档中心

欢迎来到 omd 项目文档。本目录包含项目的完整技术文档。

## 按角色阅读

### 我是新用户

1. [用户指南](user-guide.md) — 了解功能和基本操作
2. [桌面版指南](desktop.md) 或 [Web 版指南](web.md) — 选择你的版本
3. [Markdown 语法支持](markdown-syntax.md) — 写作参考
4. [常见问题](faq.md) — 遇到问题先看这里

### 我想部署 Web 版

1. [Web 版指南](web.md) — 构建与运行
2. [部署指南](deployment.md) — GitHub Pages / Docker / Nginx
3. [配置参考](configuration.md) — Trunk、localStorage、主题
4. [安全说明](security.md) — CSP、CDN、XSS

### 我想参与开发

1. [贡献指南](../CONTRIBUTING.md) — 流程与规范
2. [开发指南](development.md) — 环境搭建与调试
3. [架构设计](architecture.md) — 模块与数据流
4. [API 参考](api-reference.md) — 公开接口文档
5. [配置参考](configuration.md) — 所有可配置项

### 我想选型对比

- [版本功能对比](comparison.md) — 桌面版 vs Web 版完整矩阵

## 文档索引

### 用户文档

| 文档 | 说明 |
|------|------|
| [用户指南](user-guide.md) | 功能概览、界面介绍、通用操作 |
| [桌面版指南](desktop.md) | 安装、菜单、工具栏、快捷键、图片 |
| [Web 版指南](web.md) | 浏览器运行、自动保存、Mermaid、移动端 |
| [Markdown 语法支持](markdown-syntax.md) | 支持的语法、GFM 扩展、Mermaid |
| [版本功能对比](comparison.md) | 桌面版与 Web 版功能矩阵 |
| [常见问题](faq.md) | FAQ 与故障排除 |

### 开发者文档

| 文档 | 说明 |
|------|------|
| [架构设计](architecture.md) | 技术架构、模块划分、数据流 |
| [开发指南](development.md) | 环境搭建、工作流、调试、测试 |
| [API 参考](api-reference.md) | 模块公开接口 |
| [配置参考](configuration.md) | Cargo、Trunk、持久化、主题 |
| [部署指南](deployment.md) | 多平台部署方案 |
| [安全说明](security.md) | 安全模型与最佳实践 |
| [路线图](roadmap.md) | 未来规划 |

### 项目根目录

| 文件 | 说明 |
|------|------|
| [README.md](../README.md) | 项目简介与快速开始 |
| [CHANGELOG.md](../CHANGELOG.md) | 版本更新记录 |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | 贡献指南 |
| [SECURITY.md](../SECURITY.md) | 安全政策 |
| [LICENSE](../LICENSE) | MIT 开源许可证 |

## 文档结构图

```
docs/
├── README.md              ← 你在这里（文档索引）
├── user-guide.md          用户入门
├── desktop.md             桌面版
├── web.md                 Web 版
├── markdown-syntax.md     语法参考
├── comparison.md          版本对比
├── faq.md                 常见问题
├── architecture.md        架构设计
├── development.md         开发指南
├── api-reference.md       API 文档
├── configuration.md       配置参考
├── deployment.md          部署指南
├── security.md            安全说明
└── roadmap.md             路线图
```

## 术语表

| 术语 | 说明 |
|------|------|
| **桌面版** | 基于 egui 的原生 GUI 应用，位于项目根目录 |
| **Web 版** | 基于 Leptos + WASM 的浏览器应用，位于 `web/` |
| **GFM** | GitHub Flavored Markdown，含表格、任务列表等扩展 |
| **Trunk** | Rust WASM 构建工具，用于 Web 版 |
| **eframe** | egui 的应用框架，提供窗口和持久化 |
| **pulldown-cmark** | Rust Markdown 解析库 |
| **Mermaid** | 图表描述语言，Web 版支持渲染 |
| **localStorage** | 浏览器本地存储，Web 版用于自动保存 |
| **WASM** | WebAssembly，Web 版的运行格式 |
