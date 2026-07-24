# 常见问题（FAQ）

## 通用

### omd 是什么？

omd 是一款用 Rust 编写的轻量级 Markdown 编辑器，提供**桌面版**、**Web 版**和 **Android 版**（当前 **v0.9.x**）。

### 三端怎么选？

| | 桌面版 | Web 版 | Android 版 |
|---|--------|--------|------------|
| 运行方式 | 本地应用 | 浏览器 | 原生 APK |
| 文件管理 | 直接读写磁盘 | 导入/下载 | 文件关联打开 |
| 离线 | ✅ | ✅ PWA/自托管 | ✅ |
| 导出 HTML | ✅ | ✅ | ✅ |
| 手机使用 | ❌ | ✅ 响应式 | ✅ |

详见 [三端功能对比](comparison.md) 与 [发布说明](release-notes.md)。

### 支持哪些操作系统？

- **桌面版**：Linux、macOS、Windows（需图形环境）
- **Web 版**：Chrome 57+、Firefox 52+、Safari 11+ 等支持 WASM 的浏览器
- **Android 版**：Android 8.0+（API 26+）

### 免费吗？

是的，MIT 许可证开源。

### 从哪里下载？

- **桌面 / Web 包 / APK**：[GitHub Releases](https://github.com/doshall/omd/releases)
- **Web 在线**：<https://doshall.github.io/omd/>
- **自行构建**：见 [用户指南](user-guide.md) 与 [Android 版指南](android.md)

---

## 桌面版

### 启动报错：`Library libxkbcommon-x11.so could not be loaded`

```bash
sudo apt install libxkbcommon-x11-0
```

### Linux 编译报错缺少 `glib-2.0` / GTK？

v0.8.1 起桌面版含系统托盘，需要开发库：

```bash
sudo apt install libgtk-3-dev libayatana-appindicator3-dev libxdo-dev
```

### 全局快捷键无效？

Linux 上仅 **X11** 会话可靠；Wayland 下系统可能拦截全局热键。可在设置中关闭「全局快捷键」。

### 如何用命令行打开文件？

```bash
omd /path/to/file.md
```

### PlantUML 不显示？

桌面预览通过 **plantuml.com** 加载 SVG，需要网络。离线时请导出 HTML 或在 Web/Android 端预览。

### 关闭时未保存的内容会丢失吗？

已保存到磁盘的文件：可配置自动保存。未保存路径的新文档：关闭前会提示；eframe 也会持久化部分会话状态，但仍建议 `Ctrl+S`。

---

## Web 版

### 页面空白或 WASM 加载失败？

1. **强制刷新**：`Ctrl+Shift+R`（Mac：`Cmd+Shift+R`）
2. 注销旧 **Service Worker**：开发者工具 → Application → Service Workers → Unregister
3. 清除该站 localStorage / IndexedDB 后重试
4. 本地开发：`bash scripts/fetch-web-assets.sh && bash scripts/trunk-build.sh`
5. 确认服务器对 `.wasm` 返回 `application/wasm`

### 刷新后内容还在吗？

是。小文档在 localStorage，大文档自动迁移到 IndexedDB。隐私模式或清空站点数据会丢失。

### Mermaid / PlantUML / Graphviz 不显示？

1. 代码块语言分别为 `mermaid`、`plantuml`、`graphviz` 或 `dot`
2. PlantUML 需访问 **plantuml.com**；离线环境仅 Mermaid / Graphviz（viz.js 已打包）可靠
3. 查看浏览器控制台错误

### CI 构建报 wasm-opt 504？

GitHub 下载 `binaryen` 偶发 504。项目已提供 `scripts/trunk-build.sh`（重试后回退为无 wasm-opt 构建）。本地与 CI 均应使用该脚本。

### 如何切换英文界面？

**⚙ 设置** → **界面语言** → English。

### 如何自定义预览样式？

**⚙ 设置** → **自定义预览 CSS**，规则同时作用于预览区与导出 HTML。

---

## Android 版

### 为什么没有出现在 GitHub Release？

v0.9.0 之前 Release 工作流**未包含** Android job；自 **v0.9.1** 起打 tag 会自动附带 `omd-android-vX.Y.Z.apk`。若某次 Release 失败，见 [发布说明 — 补发](release-notes.md#补发失败的-release)。

### 如何本地打包？

```bash
export ANDROID_HOME=/path/to/android-sdk
echo "sdk.dir=$ANDROID_HOME" > android/local.properties
./scripts/build-android.sh
```

### 与 Web 版功能一致吗？

Android 复用同一 WASM  bundle，功能与 Web 版基本一致（含多标签、侧边栏、导出 HTML 等）。

---

## 导出与格式

### 支持导出 HTML 吗？

三端均支持。桌面：**File → Export HTML**；Web/Android：顶部 **导出 HTML**。

### 支持导出 PDF 吗？

通过 **导出 PDF** 打开打印优化 HTML，在浏览器中选择「另存为 PDF」。

### 支持多标签页吗？

三端均支持多标签编辑。

---

## 数据与安全

- **桌面**：文件在本地磁盘，不上传
- **Web / Android**：内容存于浏览器 localStorage / IndexedDB

详见 [安全说明](security.md)。

---

## 开发

### 需要什么 Rust 版本？

1.85+（stable），见 `rust-toolchain.toml`。

### 如何贡献？

[贡献指南](../CONTRIBUTING.md)、[开发指南](development.md)。

---

## 相关文档

- [用户指南](user-guide.md)
- [三端功能对比](comparison.md)
- [发布说明](release-notes.md)
- [配置参考](configuration.md)
- [路线图](roadmap.md)
