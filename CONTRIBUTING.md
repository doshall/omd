# 贡献指南

感谢你有兴趣为 omd 做出贡献！本文档说明如何参与项目开发。

## 行为准则

- 尊重所有贡献者
- 接受建设性批评
- 关注对社区最有利的事情

## 如何贡献

### 报告 Bug

在 [GitHub Issues](https://github.com/doshall/omd/issues) 提交时，请包含：

1. **环境信息**：操作系统、Rust 版本、桌面版或 Web 版
2. **复现步骤**：从启动到出现问题的完整操作
3. **期望行为** vs **实际行为**
4. **截图或日志**（如有）

### 提出功能建议

在 Issue 中描述：

- 使用场景
- 建议的实现方式（可选）
- 是否愿意自行实现

### 提交代码

1. Fork 仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 编写代码并确保通过编译
4. 提交变更：`git commit -m "feat: 描述你的改动"`
5. 推送分支：`git push origin feature/your-feature`
6. 创建 Pull Request

## 开发环境

### 前置要求

- Rust stable（1.85+，见 `rust-toolchain.toml`）
- 桌面版：OpenGL 支持的图形环境
- Web 版：`trunk`、`wasm32-unknown-unknown` 目标

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Web 版额外依赖
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

### 构建与测试

```bash
# 桌面版
cargo build
cargo run
cargo clippy -- -D warnings   # 推荐

# Web 版
cd web
trunk serve                   # 开发
trunk build --release         # 发布
```

### 代码规范

- 遵循 Rust 官方风格：`cargo fmt`
- 通过 Clippy 检查：`cargo clippy`
- 保持改动范围最小，一个 PR 解决一个问题
- 新增功能需更新相关文档
- 提交信息使用英文或中文，格式建议：
  - `feat:` 新功能
  - `fix:` Bug 修复
  - `docs:` 文档
  - `refactor:` 重构
  - `chore:` 构建/工具

## 项目结构

```
omd/
├── src/           # 桌面版源码
├── web/           # Web 版源码（独立 Cargo 项目）
├── docs/          # 项目文档
└── README.md
```

桌面版与 Web 版是**独立子项目**，共享设计理念但代码分开维护。修改 Markdown 渲染逻辑时，需分别更新 `src/markdown.rs` 和 `web/src/markdown.rs`。

## Pull Request 检查清单

- [ ] 代码可编译（桌面版和/或 Web 版）
- [ ] 无新增 Clippy 警告
- [ ] 已更新相关文档
- [ ] 已更新 CHANGELOG.md（如有用户可见变更）
- [ ] PR 描述清晰说明了改动内容和动机

## 许可证

贡献的代码将以 [MIT License](LICENSE) 发布。
