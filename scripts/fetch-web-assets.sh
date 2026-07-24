#!/usr/bin/env bash
# Fetch optional Web offline assets (gitignored) required for Trunk builds in CI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSETS="$ROOT/web/assets"
MERMAID_URL="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"
KATEX_VERSION="0.16.11"
KATEX_JS_URL="https://cdn.jsdelivr.net/npm/katex@${KATEX_VERSION}/dist/katex.min.js"
KATEX_CSS_URL="https://cdn.jsdelivr.net/npm/katex@${KATEX_VERSION}/dist/katex.min.css"

mkdir -p "$ASSETS"

if [[ ! -f "$ASSETS/mermaid.min.js" ]]; then
  echo "==> Downloading mermaid.min.js"
  curl -fsSL -o "$ASSETS/mermaid.min.js" "$MERMAID_URL"
else
  echo "==> mermaid.min.js already present"
fi

if [[ ! -f "$ASSETS/katex.min.js" ]]; then
  echo "==> Downloading katex.min.js"
  curl -fsSL -o "$ASSETS/katex.min.js" "$KATEX_JS_URL"
else
  echo "==> katex.min.js already present"
fi

if [[ ! -f "$ASSETS/katex.min.css" ]]; then
  echo "==> Downloading katex.min.css"
  curl -fsSL -o "$ASSETS/katex.min.css" "$KATEX_CSS_URL"
else
  echo "==> katex.min.css already present"
fi
