# Android 版指南

omd Android 版将 Web 版（Leptos + WASM）打包为原生 APK，通过 WebView 运行，完整支持移动端编辑体验。

## 特性

- 完整 Web 版功能：实时预览、Mermaid 图表、图片粘贴/上传、自动保存
- 离线运行（WASM + Mermaid 打包在 APK 内）
- 响应式移动端布局（默认「仅编辑」/ 上下分栏）
- 系统文件关联：从文件管理器打开 `.md` 文件
- 原生文件选择器（打开/上传图片）
- 启动闪屏、深色主题

## 系统要求

| 项目 | 要求 |
|------|------|
| Android 版本 | 8.0+（API 26+） |
| 架构 | arm64-v8a、armeabi-v7a、x86_64（通用 APK） |
| 存储空间 | 约 15 MB |

## 快速构建

### 前置依赖

1. **Rust** stable + `wasm32-unknown-unknown`
2. **Trunk**：`cargo install trunk --locked`
3. **JDK** 17+
4. **Android SDK**（API 35、Build-Tools 35）

### 一键构建

```bash
# 设置 Android SDK 路径
export ANDROID_HOME=/path/to/android-sdk
echo "sdk.dir=$ANDROID_HOME" > android/local.properties

# 构建 APK
./scripts/build-android.sh
```

输出：`android/app/build/outputs/apk/debug/app-debug.apk`

### 安装到手机

```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

### 手动分步构建

```bash
# 1. 下载离线 Mermaid
curl -fsSL -o web/assets/mermaid.min.js \
  https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js

# 2. 构建 Web WASM
cd web && trunk build --release && cd ..

# 3. 复制到 Android assets
rm -rf android/app/src/main/assets
mkdir -p android/app/src/main/assets
cp -a web/dist/. android/app/src/main/assets/

# 4. 构建 APK
cd android && ./gradlew assembleDebug
```

## 发布构建

```bash
cd android
./gradlew assembleRelease
```

发布版 APK 位于 `app/build/outputs/apk/release/`。发布到 Google Play 需配置签名密钥。

### 签名配置（可选）

```bash
keytool -genkey -v -keystore omd-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias omd
```

在 `android/keystore.properties`（不提交到 Git）中：

```properties
storeFile=../omd-release.jks
storePassword=your_password
keyAlias=omd
keyPassword=your_password
```

## 项目结构

```
android/
├── app/
│   ├── src/main/
│   │   ├── kotlin/dev/omd/app/MainActivity.kt   # WebView 容器
│   │   ├── res/                                  # 图标、主题、布局
│   │   ├── assets/                               # Web dist（构建时生成）
│   │   └── AndroidManifest.xml
│   └── build.gradle.kts
├── build.gradle.kts
├── settings.gradle.kts
├── gradle.properties
└── gradlew

scripts/
└── build-android.sh          # 一键构建脚本
```

## 架构说明

```
┌─────────────────────────────────────┐
│         Android APK                 │
│  ┌───────────────────────────────┐  │
│  │  MainActivity (Kotlin)        │  │
│  │  ├── WebView                  │  │
│  │  ├── 文件选择器回调            │  │
│  │  ├── 打开外部 .md Intent      │  │
│  │  └── 返回键导航               │  │
│  └──────────────┬────────────────┘  │
│                 │ file:///android_asset/
│  ┌──────────────▼────────────────┐  │
│  │  assets/ (Web dist)           │  │
│  │  ├── index.html               │  │
│  │  ├── *.wasm                   │  │
│  │  ├── *.js                     │  │
│  │  └── assets/mermaid.min.js    │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

Android 版不重复实现编辑器逻辑，而是复用 Web 版全部代码，仅添加原生壳层处理：

- WebView 配置（JavaScript、localStorage、文件访问）
- `onShowFileChooser` 桥接系统文件选择器
- `ACTION_VIEW` Intent 打开外部 Markdown 文件
- 外部链接在系统浏览器中打开

## 使用说明

### 编辑文档

启动应用即可编辑。内容自动保存到 WebView localStorage，与应用数据绑定。

### 打开外部文件

1. 在文件管理器中点击 `.md` 文件
2. 选择「omd」打开
3. 或通过应用内「打开」按钮选择文件

### 导出文档

点击工具栏「下载」将当前内容保存为 `.md` 文件到手机存储。

### 插入图片

- **🖼** 从相册选择
- **粘贴** 截图
- **🌐** 输入 URL

### 视图模式

手机上推荐使用 **✎ 仅编辑** 或默认分栏模式（上下布局）。

## 与 Web 版的区别

| 特性 | 浏览器 Web 版 | Android APK |
|------|--------------|-------------|
| 安装 | 无需安装 | 安装 APK |
| 离线 | 需部署后 | ✅ 完全离线 |
| 自动保存 | localStorage | localStorage（应用内） |
| 打开系统文件 | 文件选择器 | ✅ Intent + 文件选择器 |
| 添加到主屏幕 | 手动 | ✅ 原生图标 |
| 应用商店 | ❌ | 可上架 Play Store |

## 开发调试

### Android Studio

1. 先运行 `./scripts/build-android.sh` 生成 assets
2. 用 Android Studio 打开 `android/` 目录
3. 连接手机或启动模拟器
4. 点击 Run

### Chrome 远程调试

1. 手机开启开发者选项 → USB 调试
2. Chrome 访问 `chrome://inspect`
3. 检查 WebView 中的 omd 页面

### 常见问题

**白屏**

- 确认 `assets/` 目录已生成（运行构建脚本）
- 检查 logcat：`adb logcat | grep -i chromium`

**WASM 加载失败**

- 需要 Android 8.0+（API 26）及较新的 WebView
- 更新 Android System WebView

**文件选择器不弹出**

- 确认已授予存储权限（Android 13+ 使用系统 Photo Picker，无需权限）

## 相关文档

- [Web 版指南](web.md)
- [版本功能对比](comparison.md)
- [部署指南](deployment.md)
