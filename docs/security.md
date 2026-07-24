# 安全说明

本文档描述 omd 的安全模型、已知风险和安全最佳实践。

## 安全模型概览

```
┌─────────────────────────────────────────────────┐
│                   用户数据                       │
├────────────────────┬────────────────────────────┤
│     桌面版          │         Web 版              │
│                    │                            │
│  文件 → 本地磁盘    │  内容 → localStorage + IndexedDB │
│  图片 → 本地/网络   │  图片 → Base64 / URL        │
│  无网络通信*        │  Mermaid → CDN 加载         │
│  无数据上传         │  无后端服务器                │
└────────────────────┴────────────────────────────┘

* 网络图片和 egui_extras 图片加载器会发起 HTTP 请求
```

omd 是**纯客户端**应用，不向任何 omd 官方服务器发送用户数据。

## 桌面版

### 数据存储

- 编辑内容保存在用户指定的本地文件路径
- 应用状态（主题、分栏比例等）保存在系统应用数据目录
- 不会自动上传或同步到云端

### 网络访问

桌面版在以下情况会发起网络请求：

| 场景 | 说明 |
|------|------|
| Markdown 中的 `https://` 图片 | egui_extras 图片加载器请求 URL |
| 用户点击预览区超链接 | 系统默认浏览器打开 |

### 文件系统

- 文件读写通过 rfd 原生对话框，由用户显式选择路径
- 插入图片时写入绝对路径到 Markdown 文本
- 不会扫描或访问对话框以外的路径

### 建议

- 不要打开来源不明的 `.md` 文件后立即点击其中的链接
- 网络图片 URL 应来自可信来源

---

## Web 版

### XSS（跨站脚本）风险

Web 版使用 `inner_html` 将 pulldown-cmark 生成的 HTML 注入预览区：

```rust
<div class="preview-content" inner_html=preview_html></div>
```

**风险：** 若 Markdown 中包含恶意 HTML/JS，可能在预览区执行。

**缓解措施：**

| 措施 | 说明 |
|------|------|
| pulldown-cmark 默认行为 | 大部分 HTML 标签被转义为纯文本 |
| 无 `<script>` 执行 | inner_html 不执行 `<script>` 标签 |
| 编辑区为 textarea | 源码以纯文本编辑，不解析 HTML |

**残留风险：**

- pulldown-cmark 允许部分内联 HTML 通过
- `javascript:` 协议的链接可能被渲染为 `<a href="javascript:...">`
- 建议在不受信任的 Markdown 源上谨慎使用预览功能

### localStorage 安全

| 风险 | 说明 |
|------|------|
| 同源策略 | 仅同域名下的脚本可访问 |
| XSS 窃取 | 若页面存在 XSS 漏洞，攻击者可读取 localStorage |
| 无加密 | 内容以明文存储 |
| 容量限制 | 约 5–10 MB |

**建议：**

- 不要在 Web 版中编辑高度敏感内容（密码、密钥等）
- 公共电脑上使用后清除浏览器数据
- 定期下载备份 `.md` 文件

### 第三方 CDN

Web 版从 jsDelivr 加载 Mermaid.js：

```html
<script src="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"></script>
```

| 风险 | 缓解 |
|------|------|
| CDN 被篡改 | 使用 SRI（Subresource Integrity）哈希 |
| CDN 不可用 | 离线部署时替换为本地文件 |
| 供应链攻击 | 锁定版本号（当前 `@11`） |

**增强安全性（推荐生产部署）：**

```html
<script
  src="https://cdn.jsdelivr.net/npm/mermaid@11.4.0/dist/mermaid.min.js"
  integrity="sha384-..."
  crossorigin="anonymous"
></script>
```

### 图片安全

| 方式 | 风险 |
|------|------|
| URL 图片 | 加载时向第三方服务器发起请求（泄露 IP、Referer） |
| Base64 嵌入 | 无网络请求，但增大文档体积 |
| 粘贴/上传 | 图片数据保留在 localStorage 中 |

### Content Security Policy（CSP）

自托管部署时建议设置 CSP 头：

```nginx
add_header Content-Security-Policy "
    default-src 'self';
    script-src 'self' cdn.jsdelivr.net;
    style-src 'self' 'unsafe-inline';
    img-src 'self' data: https:;
    connect-src 'self';
" always;
```

### WASM 安全

- WASM 模块在浏览器沙箱中运行
- 仅能访问 web-sys 声明的浏览器 API
- 无法直接访问文件系统（通过 File API 需用户授权）

---

## 依赖安全

### 审计依赖

```bash
# 桌面版
cargo audit

# 安装 cargo-audit
cargo install cargo-audit
```

### 锁定版本

- `Cargo.lock` 已纳入版本控制
- `web/Cargo.lock` 已纳入版本控制
- 定期更新依赖并检查安全公告

---

## 报告安全问题

如发现安全漏洞，请**不要**在公开 Issue 中讨论。

请通过以下方式私下报告：

1. 在 GitHub 仓库创建 **Security Advisory**（Settings → Security → Advisories）
2. 或发送邮件给仓库维护者

我们会在确认后尽快修复并发布安全更新。

详见 [SECURITY.md](../SECURITY.md)。

## 相关文档

- [部署指南](deployment.md)
- [配置参考](configuration.md)
- [常见问题](faq.md)
