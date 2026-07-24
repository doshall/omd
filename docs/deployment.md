# 部署指南

本文档说明如何将 omd Web 版部署到各种静态托管平台，以及桌面版的发布方式。

## Web 版部署

Web 版构建产物为纯静态文件（HTML + WASM + CSS + JS），可部署到任何静态文件服务器。

### 构建发布包

```bash
# 若尚未下载离线 Mermaid（gitignore），先执行：
bash scripts/fetch-web-assets.sh

cd web
trunk build --release
```

输出目录 `web/dist/`：

```
dist/
├── index.html              # 入口页面
├── omd-web-<hash>.js       # WASM 胶水代码
├── omd-web-<hash>_bg.wasm  # WebAssembly 二进制
└── style-<hash>.css        # 样式表
```

### 通用部署步骤

1. 构建发布包（见上）
2. 将 `dist/` 目录内容上传到托管平台
3. 配置服务器支持 WASM MIME 类型
4. 访问部署 URL

### WASM MIME 类型

服务器必须将 `.wasm` 文件的 Content-Type 设为 `application/wasm`：

**Nginx：**

```nginx
types {
    application/wasm wasm;
}
```

**Apache：**

```apache
AddType application/wasm .wasm
```

大多数现代静态托管平台（GitHub Pages、Cloudflare Pages、Vercel）已自动处理。

---

## GitHub Pages

**在线演示**：https://doshall.github.io/omd/

### 自动部署（推荐，已启用）

仓库已配置 `.github/workflows/pages.yml`：向 `main` 推送 `web/**` 或该 workflow 文件时，自动构建并部署到 GitHub Pages。

**一次性设置**（仓库管理员）：

1. 打开 [Settings → Pages](https://github.com/doshall/omd/settings/pages)
2. **Build and deployment → Source** 选择 **GitHub Actions**
3. 保存后，下次 workflow 成功即可访问站点

**构建流程要点**：

- 运行 `scripts/fetch-web-assets.sh` 下载 `web/assets/mermaid.min.js`（该文件在 `.gitignore` 中，CI 需先拉取）
- `trunk build --release` 生成 `web/dist/`
- `actions/deploy-pages` 发布产物

查看部署状态：[Actions → Deploy Web to GitHub Pages](https://github.com/doshall/omd/actions/workflows/pages.yml)

### 手动部署

若需自行上传静态文件：

```bash
bash scripts/fetch-web-assets.sh
cd web
trunk build --release

# 将 dist/ 内容推送到 gh-pages 分支，或上传到任意静态托管
```

访问：`https://doshall.github.io/omd/`（项目站路径为 `/omd/`）

---

## Cloudflare Pages

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Pages → Create a project → Connect to Git
3. 构建配置：

| 设置 | 值 |
|------|-----|
| Build command | `cd web && trunk build --release` |
| Build output directory | `web/dist` |
| Environment variable | `TRUNK_BUILD_NO_SRI=true`（如需要） |

4. 部署环境变量中添加 Rust 安装（或使用自定义 Docker 镜像）

### 使用 wrangler CLI

```bash
cd web
trunk build --release
npx wrangler pages deploy dist --project-name=omd
```

---

## Vercel

1. 导入 GitHub 仓库
2. 构建设置：

| 设置 | 值 |
|------|-----|
| Framework Preset | Other |
| Build Command | `cd web && cargo install trunk --locked && rustup target add wasm32-unknown-unknown && trunk build --release` |
| Output Directory | `web/dist` |

3. Deploy

---

## Docker 自托管

### Dockerfile

```dockerfile
# 构建阶段
FROM rust:1.83-bookworm AS builder
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked
WORKDIR /app
COPY web/ ./web/
RUN cd web && trunk build --release

# 运行阶段
FROM nginx:alpine
COPY --from=builder /app/web/dist /usr/share/nginx/html
RUN echo 'types { application/wasm wasm; }' >> /etc/nginx/mime.types
EXPOSE 80
```

### 构建与运行

```bash
docker build -t omd-web .
docker run -d -p 8080:80 omd-web
```

访问 `http://localhost:8080`

---

## Nginx 自托管

```bash
cd web && trunk build --release
sudo cp -r dist/* /var/www/omd/
```

Nginx 配置：

```nginx
server {
    listen 80;
    server_name omd.example.com;
    root /var/www/omd;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location ~* \.wasm$ {
        types { application/wasm wasm; }
    }

    # 缓存静态资源
    location ~* \.(js|css|wasm)$ {
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
```

---

## 子路径部署

若部署在子路径（如 `https://example.com/omd/`），需配置 Trunk 的 `public_url`：

```toml
# web/Trunk.toml
[build]
public_url = "/omd/"
```

然后重新构建。

---

## Mermaid CDN 离线化

默认从 CDN 加载 Mermaid.js。本地开发与 CI 构建前可执行：

```bash
bash scripts/fetch-web-assets.sh
```

该脚本将 Mermaid 下载到 `web/assets/mermaid.min.js`（已在 `.gitignore` 中，不提交仓库）。`index.html` 已配置本地副本与 CDN 回退。

---

## 桌面版发布

### 构建各平台二进制

```bash
# Linux
cargo build --release
# → target/release/omd

# Windows（交叉编译或 CI）
# macOS（交叉编译或 CI）
```

### GitHub Releases

推送 `v*` tag 时自动构建三平台桌面二进制、Web 压缩包与 **Android APK**（`omd-android-vX.Y.Z.apk`），并创建 GitHub Release。详见 [发布说明](release-notes.md)。

**Web 构建**（本地与 CI 统一）：

```bash
bash scripts/fetch-web-assets.sh   # 离线 JS/CSS（含重试）
bash scripts/trunk-build.sh        # Trunk release；wasm-opt 504 时自动重试/回退
```

**补发失败的历史 Release**（如 wasm-opt 504、Linux 缺少 GTK 依赖）：

1. 打开 [Actions → Release](https://github.com/doshall/omd/actions/workflows/release.yml)
2. 点击 **Run workflow**
3. 输入 tag 名称（如 `v0.2.0`）并运行

`v0.3.0` 及之后的 tag 推送会使用当前 `main` 上的 workflow（含 `scripts/fetch-web-assets.sh`）。

手动发布：

1. 创建 tag：`git tag v0.3.0 && git push origin v0.3.0`
2. 等待 Release workflow 完成
3. 在 [Releases](https://github.com/doshall/omd/releases) 页面查看产物

### 多平台 CI 示例

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: omd-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/omd*
```

---

## 性能优化建议

| 优化 | 说明 |
|------|------|
| `trunk build --release` | 启用 wasm-opt 压缩 |
| CDN 加速 | 静态资源走 CDN |
| Gzip/Brotli | 服务器启用压缩（WASM 可压缩 50%+） |
| 缓存策略 | JS/WASM/CSS 设长期缓存（hash 文件名已支持） |
| HTTP/2 | 多路复用加速加载 |

典型 Web 版加载大小：

| 资源 | 约大小 |
|------|--------|
| WASM | 1–2 MB（压缩后 400–800 KB） |
| JS 胶水 | 50–100 KB |
| CSS | 5–10 KB |
| Mermaid.js | 1.5 MB（CDN） |

---

## 相关文档

- [Web 版指南](web.md)
- [开发指南](development.md)
- [常见问题](faq.md)
