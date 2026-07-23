# 开发指南

本文档面向希望参与 omd 开发的贡献者，涵盖环境搭建、项目结构、开发流程和调试技巧。

## 环境搭建

### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

项目通过 `rust-toolchain.toml` 指定 stable 工具链，进入项目目录时会自动切换。

### 2. 克隆仓库

```bash
git clone https://github.com/doshall/omd.git
cd omd
```

### 3. 桌面版依赖

```bash
# 验证编译
cargo build

# Linux 图形库（如需运行 GUI）
sudo apt install libxkbcommon-x11-0
```

### 4. Web 版依赖

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked

cd web
trunk build
```

## 项目结构详解

```
omd/
├── Cargo.toml              # 桌面版包配置
├── Cargo.lock              # 桌面版依赖锁定
├── rust-toolchain.toml     # Rust 工具链版本
├── .gitignore
│
├── src/                    # 桌面版源码
│   ├── main.rs             # 入口：eframe 窗口初始化
│   ├── app.rs              # OmdApp 结构体与 UI 逻辑（~540 行）
│   └── markdown.rs         # egui 预览渲染器（~350 行）
│
├── web/                    # Web 版（独立 Cargo 项目）
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── Trunk.toml          # Trunk 构建/服务配置
│   ├── index.html          # HTML 入口 + Mermaid 脚本
│   ├── style.css           # 全局样式
│   └── src/
│       ├── lib.rs          # Leptos 应用（~500 行）
│       └── markdown.rs     # HTML 转换工具（~100 行）
│
├── docs/                   # 项目文档
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── LICENSE
```

## 开发工作流

### 桌面版

```bash
# 开发运行（自动重编译需外部工具如 cargo-watch）
cargo run

# 使用 cargo-watch 实现热重载
cargo install cargo-watch
cargo watch -x run

# 检查
cargo check
cargo clippy
cargo fmt -- --check
```

### Web 版

```bash
cd web

# 开发服务器（自带热重载）
env -u NO_COLOR trunk serve

# 仅构建
env -u NO_COLOR trunk build

# 发布构建
env -u NO_COLOR trunk build --release
```

> **注意**：若 `trunk` 报 `invalid value '1' for '--no-color'`，使用 `env -u NO_COLOR` 前缀。

### 同时开发两个版本

两个版本是独立的 Cargo 项目，可分别在不同终端中运行：

```bash
# 终端 1：桌面版
cargo run

# 终端 2：Web 版
cd web && trunk serve
```

## 代码组织原则

### 桌面版 `app.rs`

| 区域 | 职责 |
|------|------|
| `OmdApp` 结构体 | 应用状态定义 |
| `impl OmdApp` | 文件操作、工具栏、主题 |
| `impl eframe::App for OmdApp` | 每帧 update 循环、面板布局 |
| `DEFAULT_CONTENT` | 启动时的示例文档 |

添加新功能时：

1. 在 `OmdApp` 中添加状态字段
2. 在 `render_toolbar()` 或菜单中添加 UI
3. 在 `update()` 中处理逻辑
4. 如需预览支持，修改 `markdown.rs`

### 桌面版 `markdown.rs`

预览渲染器使用**状态机模式**：

```rust
struct PreviewState { /* 解析上下文 */ }

impl PreviewState {
    fn handle_event(&mut self, ui: &mut Ui, event: Event<'_>) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => { ... }
            Event::Text(text) => { ... }
            // ...
        }
    }
}
```

添加新 Markdown 元素支持：

1. 在 `PreviewState` 中添加状态字段
2. 在 `handle_event()` 中添加对应的 `Event::Start` / `Event::End` / `Event::Text` 处理
3. 渲染为对应的 egui 控件

### Web 版 `lib.rs`

Leptos 组件化结构：

```rust
#[component]
fn App() -> impl IntoView {
    // 信号定义
    let (content, set_content) = signal(...);

    // Effect 副作用
    Effect::new(move |_| { ... });

    // 事件处理闭包
    let on_file_change = move |ev| { ... };

    // 视图
    view! { <div>...</div> }
}
```

添加新功能：

1. 添加 signal 状态
2. 编写事件处理闭包
3. 在 `view!` 宏中添加 UI 元素
4. 如需预览支持，修改 `web/src/markdown.rs`

### Web 版 `markdown.rs`

纯函数工具集：

```rust
pub fn markdown_to_html(markdown: &str) -> String { ... }
pub fn wrap_selection(...) -> String { ... }
pub fn insert_at_cursor(...) -> String { ... }
pub fn image_markdown(alt: &str, url: &str) -> String { ... }
```

## 调试技巧

### 桌面版

```bash
# 详细日志
RUST_LOG=debug cargo run

# 运行时 panic 回溯
RUST_BACKTRACE=1 cargo run
```

egui 内置调试面板：按 `F12` 或菜单中可开启（取决于 egui 版本配置）。

### Web 版

1. 浏览器开发者工具（F12）→ Console 查看 WASM panic
2. `console_error_panic_hook` 已在 `main()` 中启用，panic 会输出到控制台
3. Trunk 开发服务器支持 source map

```bash
# 查看 WASM 大小
ls -lh web/dist/*.wasm
```

### 常见问题

| 问题 | 解决 |
|------|------|
| `edition2024` 错误 | 升级 Rust：`rustup update stable` |
| egui 编译慢 | 正常，首次编译约 2–3 分钟 |
| WASM 构建失败 | 确认 `wasm32-unknown-unknown` 已安装 |
| Leptos 闭包 Clone 错误 | 使用独立函数 + 克隆 signal |

## 测试

当前项目暂无自动化测试。手动测试检查清单：

### 桌面版

- [ ] 启动显示默认示例
- [ ] 编辑文字，预览实时更新
- [ ] 新建/打开/保存/另存为
- [ ] 工具栏各按钮
- [ ] 插入本地图片，预览显示
- [ ] 主题切换
- [ ] 快捷键 Ctrl+N/O/S
- [ ] 分栏拖拽调整
- [ ] 关闭重开，状态恢复

### Web 版

- [ ] 页面加载显示默认示例
- [ ] 编辑文字，预览实时更新
- [ ] 新建/打开/下载
- [ ] 工具栏各按钮
- [ ] 图片 URL/上传/粘贴/拖拽
- [ ] Mermaid 图表渲染
- [ ] 三种视图模式
- [ ] 主题切换（Mermaid 同步）
- [ ] 刷新页面，内容恢复
- [ ] 手机浏览器布局

### 回归测试脚本

可快速验证基本功能：

```bash
#!/bin/bash
set -e
echo "=== 桌面版编译 ==="
cargo build --release

echo "=== Web 版编译 ==="
cd web && env -u NO_COLOR trunk build --release && cd ..

echo "=== Clippy 检查 ==="
cargo clippy -- -D warnings 2>/dev/null || echo "Clippy 有警告，请检查"

echo "=== 全部通过 ==="
```

## 发布流程

### 桌面版

```bash
cargo build --release
# 二进制：target/release/omd
```

可配合 GitHub Actions 构建多平台二进制并发布 Release。

### Web 版

```bash
cd web
trunk build --release
# 静态文件：web/dist/
```

部署到静态托管。详见 [部署指南](deployment.md)。

### 版本号

更新以下文件中的版本号：

- `Cargo.toml`（桌面版）
- `web/Cargo.toml`（Web 版）
- `CHANGELOG.md`

## 相关文档

- [架构设计](architecture.md)
- [贡献指南](../CONTRIBUTING.md)
- [部署指南](deployment.md)
