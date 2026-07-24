# 发布说明

本文档汇总各版本的**发布产物**与**三端差异**，供下载安装时参考。详细变更见 [CHANGELOG](../CHANGELOG.md)。

## 当前版本：v0.9.x

### 下载渠道

| 平台 | 获取方式 |
|------|----------|
| **桌面** | [GitHub Releases](https://github.com/doshall/omd/releases)：`omd-x86_64-unknown-linux-gnu`、`omd-x86_64-apple-darwin`、`omd-x86_64-pc-windows-msvc.exe` |
| **Web** | 在线：<https://doshall.github.io/omd/>；离线：Release 中的 `omd-web-dist.tar.gz` 或自行 `bash scripts/trunk-build.sh` |
| **Android** | Release 中的 `omd-android-vX.Y.Z.apk`；或本地 `./scripts/build-android.sh` |

### v0.9.0 主要功能

**格式扩展**

- PlantUML / Graphviz（`plantuml`、`graphviz`、`dot` 代码块）
- 自定义预览 CSS（预览 + 导出 HTML）

**打磨与补齐**

- 中/英界面（设置 → 界面语言）
- Web 拼写检查开关
- Web 无障碍（ARIA、对话框、实时区域）

**桌面**

- PlantUML 预览（plantuml.com SVG）
- 系统托盘与全局快捷键（v0.8.1 起）

**文件与项目**（v0.8.0 起）

- 项目文件夹侧边栏（桌面 + Web + Android）

### 发布检查清单（维护者）

1. 更新 `Cargo.toml` / `omd-common` / `omd-web` / `android/app/build.gradle.kts` 版本号
2. 填写 `CHANGELOG.md`
3. 推送 tag：`git tag vX.Y.Z && git push origin vX.Y.Z`
4. 确认 [Release workflow](https://github.com/doshall/omd/actions/workflows/release.yml) 全部 job 成功
5. 验证 Release 附件：三平台桌面二进制、Web 压缩包、Android APK

### 补发失败的 Release

若某次 Release 因 CI 失败未生成附件（例如 wasm-opt 504、Linux 缺少 GTK 依赖、macOS/Windows 误拉 `gtk`）：

1. 在 main 修复 CI 后合并（v0.9.2 起已修复跨平台 `desktop-shell` 依赖）
2. Actions → **Release** → **Run workflow**
3. 输入已有 tag（如 `v0.9.0` 或 `v0.9.1`）重新构建

## 历史版本摘要

| 版本 | 主题 |
|------|------|
| v0.9.x | 格式扩展、i18n、无障碍、Release 含 APK |
| v0.8.x | 项目侧边栏、托盘、全局快捷键 |
| v0.7.x | CLI 打开文件、图片压缩、任务列表可勾选 |
| v0.6.x | TOC/脚注开关、IndexedDB 大文档、最近文件正文 |
| v0.5.x | 脚注、Front Matter、多标签、图片灯箱 |

## 相关文档

- [CHANGELOG](../CHANGELOG.md)
- [部署指南](deployment.md)
- [三端功能对比](comparison.md)
- [Android 版指南](android.md)
