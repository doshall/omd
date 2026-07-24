#!/usr/bin/env bash
# Fetch optional Web offline assets (gitignored) required for Trunk builds in CI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSETS="$ROOT/web/assets"
CURL_RETRY="$ROOT/scripts/curl-retry.sh"
MERMAID_URL="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"
KATEX_VERSION="0.16.11"
KATEX_JS_URL="https://cdn.jsdelivr.net/npm/katex@${KATEX_VERSION}/dist/katex.min.js"
KATEX_CSS_URL="https://cdn.jsdelivr.net/npm/katex@${KATEX_VERSION}/dist/katex.min.css"
PAKO_URL="https://cdn.jsdelivr.net/npm/pako@2/dist/pako.min.js"
VIZ_URL="https://cdn.jsdelivr.net/npm/viz.js@2.1.2/viz.min.js"
VIZ_RENDER_URL="https://cdn.jsdelivr.net/npm/viz.js@2.1.2/full.render.js"
HLJS_JS_URL="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/highlight.min.js"
HLJS_CSS_LIGHT="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/styles/github.min.css"
HLJS_CSS_DARK="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/styles/github-dark.min.css"

mkdir -p "$ASSETS"

download_if_missing() {
  local name="$1"
  local url="$2"
  if [[ -f "$ASSETS/$name" ]]; then
    echo "==> $name already present"
    return
  fi
  echo "==> Downloading $name"
  bash "$CURL_RETRY" "$url" "$ASSETS/$name"
}

download_if_missing "mermaid.min.js" "$MERMAID_URL"
download_if_missing "katex.min.js" "$KATEX_JS_URL"
download_if_missing "katex.min.css" "$KATEX_CSS_URL"
download_if_missing "pako.min.js" "$PAKO_URL"
download_if_missing "viz.min.js" "$VIZ_URL"
download_if_missing "full.render.js" "$VIZ_RENDER_URL"
download_if_missing "highlight.min.js" "$HLJS_JS_URL"
download_if_missing "highlight-github.min.css" "$HLJS_CSS_LIGHT"
download_if_missing "highlight-github-dark.min.css" "$HLJS_CSS_DARK"
