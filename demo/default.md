---
title: omd 功能演示
description: 脚注、TOC、YAML、复杂 Mermaid 与图片放大测试
---

# omd 功能演示

欢迎使用 **omd** Markdown 编辑器！本文档展示常用与**复杂图表**示例，便于测试预览、主题切换与导出。

> **测试提示**：点击右上角 **🌙 / ☀️** 切换浅色/深色主题，下方甘特图、饼图等应正常重绘（无 `Syntax error`）。

---

## 1. 文本格式

| 格式 | 语法 | 效果 |
|------|------|------|
| 粗体 | `**粗体**` | **粗体** |
| 斜体 | `*斜体*` | *斜体* |
| 删除线 | `~~删除~~` | ~~删除~~ |
| 行内代码 | `` `code` `` | `code` |
| 链接 | `[文字](url)` | [Rust 官网](https://www.rust-lang.org) |

---

## 2. 任务列表

- [x] 实时预览
- [x] Mermaid 多类型图表
- [x] LaTeX 数学公式
- [x] 导出 HTML / PDF
- [ ] 继续探索…

---

## 3. 代码块

```rust
fn main() {
    println!("Hello, omd!");
}
```

---

## 4. LaTeX 数学公式

行内：$E = mc^2$，块级：

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$

---

## 5. Mermaid — 基础图表

### 流程图

```mermaid
flowchart TD
    A[编写 Markdown] --> B{实时预览}
    B --> C[插入图片]
    B --> D[渲染图表]
    C --> E[导出 / 保存]
    D --> E
```

### 时序图

```mermaid
sequenceDiagram
    participant 用户
    participant 编辑器
    participant 预览区
    用户->>编辑器: 输入文字
    编辑器->>预览区: 即时渲染
    预览区-->>用户: 显示结果
```

---

## 6. Mermaid — 复杂图表（主题切换测试）

### 甘特图（含中文任务名）

```mermaid
gantt
    title omd 开发里程碑
    dateFormat YYYY-MM-DD
    section 基础
    W1 项目骨架           :w1, 2026-01-06, 7d
    W2 编辑体验           :w2, after w1, 7d
    section 导出与 Web
    W3 导出 HTML/PWA      :w3, after w2, 7d
    W4 多标签与 PDF       :w4, after w3, 7d
    section 质量
    W5 测试与文档         :w5, after w4, 7d
```

### 饼图

```mermaid
pie title 功能投入占比
    "编辑体验" : 35
    "导出格式" : 25
    "Web/Android" : 25
    "文档测试" : 15
```

### 状态图

```mermaid
stateDiagram-v2
    [*] --> 编辑中
    编辑中 --> 预览中: 输入变更
    预览中 --> 编辑中: 继续修改
    预览中 --> 已导出: 导出 HTML/PDF
    已导出 --> [*]
```

### 类图

```mermaid
classDiagram
    class Editor {
        +String content
        +render()
        +save()
    }
    class Preview {
        +renderMarkdown()
        +renderMermaid()
    }
    class Export {
        +toHtml()
        +toPdf()
    }
    Editor --> Preview : 更新
    Preview --> Export : 导出
```

### ER 图

```mermaid
erDiagram
    DOCUMENT ||--o{ TAB : contains
    DOCUMENT {
        string id
        string title
        string content
    }
    TAB {
        string id
        string filename
    }
```

---

## 7. 表格与图片

脚注示例：omd 是轻量编辑器[^note]。

[^note]: 支持桌面、Web 与 Android 三端。

| 平台 | 保存方式 |
|------|----------|
| 桌面 | 磁盘文件 |
| Web | localStorage / 下载 |
| Android | WebView + 下载 |

![Rust Logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg)

*点击图片可放大预览（Web / 桌面）。*

---

## 8. 快捷操作

| 功能 | 桌面 | Web |
|------|------|-----|
| 新建 | `Ctrl+N` | 顶部「新建」 |
| 保存 | `Ctrl+S` | 「下载」 |
| 查找 | `Ctrl+F` | `Ctrl+F` |
| 导出 PDF | 工具栏 📕 | 「导出 PDF」 |
| 多标签 | — | 标签栏 `+` |

---

**开始编辑吧！** 修改任意内容，预览即时更新。🦀
