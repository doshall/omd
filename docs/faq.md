# 常见问题（FAQ）

## 通用

### omd 是什么？

omd 是一款用 Rust 编写的轻量级 Markdown 编辑器，提供**桌面版**、**Web 版**和 **Android 版**。

### 桌面版和 Web 版有什么区别？

| | 桌面版 | Web 版 | Android 版 |
|---|--------|--------|------------|
| 运行方式 | 本地应用 | 浏览器 | 原生 APK |
| 文件管理 | 直接读写磁盘 | 导入/下载 | 文件关联打开 |
| Mermaid | ✅ | ✅ | ✅ |
| 图片粘贴 | ✅ | ✅ | ✅ |
| 自动保存 | ✅ 磁盘（可配置） | 自动（localStorage） | 自动（localStorage） |
| 导出 HTML | ✅ | ✅ | — |
| 在线演示 | — | https://doshall.github.io/omd/ | — |
| 手机使用 | ❌ | ✅ 响应式 | ✅ |

详见 [用户指南](user-guide.md#版本选择)。

### 支持哪些操作系统？

- **桌面版**：Linux、macOS、Windows（需图形环境）
- **Web 版**：任何有现代浏览器的设备

### 免费吗？

是的，omd 以 MIT 许可证开源，免费使用和修改。

### 还有哪些功能可以加？当前优先做什么？

核心编辑能力已比较完整，没有「必须马上加」的功能。完整扩展清单与实现顺序见 [路线图](roadmap.md)。

**当前优先方向：A 类 — 编辑体验**，推荐顺序：

1. 查找 / 替换
2. 预览区代码块语法高亮
3. 行号与当前行高亮
4. **滚动条 Minimap**（文档缩略导航）
5. 编辑区与预览区同步滚动

---

## 桌面版

### 启动报错：`Library libxkbcommon-x11.so could not be loaded`

Linux 缺少 X11 键盘库：

```bash
sudo apt install libxkbcommon-x11-0
```

### 在远程服务器上运行看不到窗口？

桌面版是原生 GUI 应用，窗口显示在运行它的机器上。通过 SSH 连接远程服务器时无法看到界面。

解决方案：
- 在本地电脑运行桌面版
- 使用 Web 版（`cd web && trunk serve`）
- 使用 X11 转发（`ssh -X`，体验较差）

### 图片无法显示？

1. 检查路径是否正确（相对路径基于当前文件所在目录）
2. 网络图片确认 URL 可访问
3. 确认图片格式受支持（PNG、JPG、GIF、WebP、SVG、BMP）

### 如何打开特定文件？

```bash
# 未来版本可能支持
omd /path/to/file.md

# 当前：启动后 File → Open
```

### 关闭时未保存的内容会丢失吗？

eframe persistence 会保存编辑区内容。但建议养成 `Ctrl+S` 保存习惯。

---

## Web 版

### 页面空白或 WASM 加载失败？

1. 确认浏览器支持 WebAssembly（Chrome 57+、Firefox 52+、Safari 11+）
2. 检查控制台错误信息
3. 重新构建：`cd web && trunk build --release`
4. 确认服务器正确设置 WASM MIME 类型

### 刷新后内容还在吗？

是的，Web 版自动保存到 localStorage。除非：
- 使用了浏览器隐私/无痕模式
- 手动清除了浏览器数据
- localStorage 已满

### 如何清除自动保存的内容？

- 点击「新建」清空
- 或浏览器设置 → 清除网站数据

### Mermaid 图表不显示？

1. 确认代码块语言为 `mermaid`（不是 ` ``` ` 无语言标记）
2. 检查网络能否访问 `cdn.jsdelivr.net`
3. 查看浏览器控制台 Mermaid 错误
4. 确认 Mermaid 语法正确

### 手机上怎么用？

**方式一：局域网访问**

```bash
cd web && trunk serve
# 手机浏览器访问 http://<电脑IP>:8080
```

**方式二：在线演示或公网部署**

直接访问 https://doshall.github.io/omd/ ，或自行部署 `web/dist/` 到静态托管。

**方式三：手机 Cursor 浏览器**

Cloud Agent 远程运行无法显示 GUI，需本地运行或部署到公网。

### 上传的图片太大怎么办？

Base64 嵌入会显著增大文档体积。建议：
- 小图标/截图：Base64 可接受
- 大图片：使用 URL 引用（🌐 按钮）
- 定期下载备份 `.md` 文件

### `trunk` 命令报错？

```bash
# NO_COLOR 环境变量冲突
env -u NO_COLOR trunk serve

# 未安装 trunk
cargo install trunk --locked

# 未安装 WASM 目标
rustup target add wasm32-unknown-unknown
```

---

## 开发

### 需要什么 Rust 版本？

1.85+（stable）。项目包含 `rust-toolchain.toml` 自动管理。

### 如何贡献代码？

参见 [贡献指南](../CONTRIBUTING.md) 和 [开发指南](development.md)。

### 两个版本的 Markdown 渲染器为什么不共享？

渲染目标不同：
- 桌面版输出 egui 原生控件
- Web 版输出 HTML 字符串

未来可考虑抽取共享的解析逻辑，但渲染层需保持独立。

### 如何添加新的 Markdown 语法支持？

1. 确认 pulldown-cmark 是否支持该语法
2. 桌面版：在 `src/markdown.rs` 的 `PreviewState::handle_event()` 中添加处理
3. Web 版：pulldown-cmark HTML 输出通常自动支持；特殊处理在 `web/src/markdown.rs` 中添加

---

## 其他

### 支持导出 HTML 吗？

支持。桌面版：**File → Export HTML…** 或工具栏 **📤**；Web 版：顶部 **导出 HTML**。生成独立 HTML 文件，含 Mermaid 与代码高亮。

### 支持导出 PDF 吗？

暂不支持。可通过浏览器打印功能（Web 版）或第三方工具转换。

### 支持多标签页吗？

暂不支持。一次编辑一个文档。

### 数据安全吗？

- **桌面版**：文件存储在本地磁盘，不上传任何数据
- **Web 版**：内容保存在浏览器 localStorage，不发送到服务器

详见 [安全说明](security.md)。

### 桌面版和 Web 版怎么选？

参见 [版本功能对比](comparison.md)。

### 如何配置主题颜色？

- **桌面版**：使用 egui 内置深色/浅色方案，暂不支持自定义
- **Web 版**：修改 `web/style.css` 中的 CSS 变量，参见 [配置参考](configuration.md)

### localStorage 存了哪些数据？

| 键名 | 内容 |
|------|------|
| `omd-web-content` | Markdown 文本 |
| `omd-web-theme` | 主题偏好 |
| `omd-web-view` | 视图模式 |

### 如何离线使用 Web 版？

1. `cd web && trunk build --release`
2. 将 `dist/` 部署到本地服务器或直接用浏览器打开
3. Mermaid 需本地化 CDN，参见 [部署指南](deployment.md)

### 在哪里报告问题？

[GitHub Issues](https://github.com/doshall/omd/issues)

## 相关文档

- [用户指南](user-guide.md)
- [安全说明](security.md)
- [版本功能对比](comparison.md)
- [配置参考](configuration.md)
- [路线图](roadmap.md)
- [开发指南](development.md)
