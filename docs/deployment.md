# 部署指南

本文档说明如何将 omd Web 版部署到各种静态托管平台，以及桌面版的发布方式。

## Web 版部署

Web 版构建产物为纯静态文件（HTML + WASM + CSS + JS），可部署到任何静态文件服务器。

### 构建发布包

```bash
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

### 方式一：手动部署

```bash
cd web
trunk build --release

# 将 dist/ 内容推送到 gh-pages 分支
cd dist
git init
git add .
git commit -m "deploy"
git remote add origin https://github.com/doshall/omd.git
git push -f origin main:gh-pages
```

访问：`https://doshall.github.io/omd/`

### 方式二：GitHub Actions 自动部署

创建 `.github/workflows/deploy-web.yml`：

```yaml
name: Deploy Web

on:
  push:
    branches: [main]
    paths: ['web/**']

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Add WASM target
        run: rustup target add wasm32-unknown-unknown

      - name: Install Trunk
        run: cargo install trunk --locked

      - name: Build
        run: cd web && trunk build --release

      - name: Setup Pages
        uses: actions/configure-pages@v4

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: web/dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

需在仓库 Settings → Pages 中启用 GitHub Pages，Source 选 GitHub Actions。

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

默认从 `cdn.jsdelivr.net` 加载 Mermaid.js。若部署环境无外网：

1. 下载 Mermaid 到 `web/assets/mermaid.min.js`
2. 修改 `index.html`：

```html
<!-- 替换 CDN 链接 -->
<script data-trunk rel="copy-file" href="assets/mermaid.min.js"></script>
```

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

1. 创建 tag：`git tag v0.2.0 && git push origin v0.2.0`
2. 在 GitHub Releases 页面创建 Release
3. 上传各平台二进制文件

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
