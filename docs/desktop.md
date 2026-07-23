# 桌面版指南

omd 桌面版是基于 **egui/eframe** 构建的原生 GUI 应用，适合在本地电脑上编辑和管理 Markdown 文件。

## 系统要求

| 平台 | 要求 |
|------|------|
| **Linux** | Wayland 或 X11，OpenGL 支持；需 `libxkbcommon-x11` 等库 |
| **macOS** | 10.15+ |
| **Windows** | Windows 10+ |

### Linux 依赖

```bash
# Ubuntu/Debian
sudo apt install libxkbcommon-x11-0 libxkbcommon0

# Fedora
sudo dnf install libxkbcommon-x11
```

### Rust 工具链

```bash
rustup default stable   # 需要 1.85+
```

## 安装与运行

### 从源码构建

```bash
git clone https://github.com/doshall/omd.git
cd omd
cargo build --release
./target/release/omd
```

### 开发模式

```bash
cargo run
```

开发模式编译更快，但运行性能较低，适合调试。

## 界面详解

### 菜单栏

#### File（文件）

| 菜单项 | 快捷键 | 功能 |
|--------|--------|------|
| New | `Ctrl+N` | 创建空白文档，清除当前内容 |
| Open… | `Ctrl+O` | 打开 `.md` / `.markdown` / `.txt` 文件 |
| Save | `Ctrl+S` | 保存到当前路径；无路径时等同另存为 |
| Save As… | `Ctrl+Shift+S` | 选择新路径保存 |
| Exit | — | 退出应用 |

#### View（视图）

| 菜单项 | 功能 |
|--------|------|
| Show Preview | 切换预览区显示/隐藏 |
| Dark Mode | 切换深色/浅色主题 |

#### Help（帮助）

| 菜单项 | 功能 |
|--------|------|
| About omd | 在状态栏显示版本信息 |

### 工具栏

```
📄 New | 📂 Open | 💾 Save | 💾 Save As
─────────────────────────────────────
B | I | S | </> | 🔗 | 🖼
─────────────────────────────────────
H1 | H2 | • | 1. | ❝
─────────────────────────────────────
👁 Preview                                    🌙/☀️
```

| 按钮 | 悬停提示 | 说明 |
|------|----------|------|
| B | Bold (**text**) | 插入粗体标记 |
| I | Italic (*text*) | 插入斜体标记 |
| S | Strikethrough (~~text~~) | 插入删除线 |
| </> | Inline code (`code`) | 插入行内代码 |
| 🔗 | Link ([text](url)) | 插入链接模板 |
| 🖼 | Image (![alt](path)) | 打开文件选择器插入图片 |
| H1 | Heading 1 | 行首插入 `# ` |
| H2 | Heading 2 | 行首插入 `## ` |
| • | Bullet list | 行首插入 `- ` |
| 1. | Numbered list | 行首插入 `1. ` |
| ❝ | Blockquote | 行首插入 `> ` |
| 👁 | — | 切换预览区 |
| 🌙/☀️ | Toggle theme | 切换主题 |

### 编辑区

- 等宽字体显示 Markdown 源码
- 支持多行编辑、滚动
- 自动获得焦点

### 预览区

- 实时渲染 Markdown 为 egui 原生控件
- 支持：标题、段落、列表、引用、代码块、表格、任务列表、链接、图片
- 图片支持本地路径（相对/绝对）和网络 URL
- 拖拽中间分隔线调整左右比例（20%–80%）

### 状态栏

```
Lines: 42  Words: 256  Chars: 1024    /path/to/document.md
```

### 窗口标题

```
document.md * — omd
```

- 显示当前文件名
- `*` 表示有未保存修改

## 图片功能

### 插入本地图片

1. 点击工具栏 **🖼**
2. 在文件对话框中选择图片
3. 插入格式：`![文件名](/absolute/path/to/image.png)`
4. 预览区自动加载并显示

如果当前文件已保存，文件对话框默认打开文件所在目录。

### 支持的图片格式

PNG、JPG/JPEG、GIF、WebP、SVG、BMP

### 路径解析规则

| Markdown 中的路径 | 解析方式 |
|-------------------|----------|
| `https://...` | 直接从网络加载 |
| `http://...` | 直接从网络加载 |
| `data:...` | Base64 数据 URL |
| `/absolute/path` | 绝对路径 |
| `./relative/path` | 相对于当前文件所在目录 |
| `relative/path` | 相对于当前文件所在目录 |

### 网络图片

在 Markdown 中直接书写：

```markdown
![Rust Logo](https://www.rust-lang.org/static/images/rust-logo-blk.svg)
```

需要网络连接才能加载。

## 应用状态持久化

eframe 的 persistence 功能会自动保存：

- 深色/浅色主题偏好
- 编辑区内容（关闭时）
- 分栏比例
- 预览区开关状态

下次启动时自动恢复。

## 发布构建优化

`Cargo.toml` 中 release 配置：

```toml
[profile.release]
lto = true          # 链接时优化
codegen-units = 1   # 单代码生成单元
strip = true        # 剥离调试符号
```

典型二进制大小约 15–25 MB（含 egui 运行时）。

## 已知限制

- 不支持 Mermaid 图表渲染
- 不支持从剪贴板粘贴图片
- 代码块无语法高亮（预览区显示纯文本）
- 工具栏格式按钮作用于全文（非精确选区）
- 不支持 LaTeX 数学公式
- 脚注语法不渲染

## 故障排除

### 启动失败：`Library libxkbcommon-x11.so could not be loaded`

```bash
sudo apt install libxkbcommon-x11-0
```

### 窗口无法显示

确认运行在图形环境中（非纯 SSH 终端）。远程服务器需 X11 转发或 VNC。

### 图片无法加载

- 检查文件路径是否正确
- 网络图片确认可访问
- 查看终端是否有加载错误输出

## 相关文档

- [用户指南](user-guide.md)
- [Markdown 语法支持](markdown-syntax.md)
- [架构设计](architecture.md)
- [常见问题](faq.md)
