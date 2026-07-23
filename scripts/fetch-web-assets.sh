#!/usr/bin/env bash
# Fetch optional Web offline assets (gitignored) required for Trunk builds in CI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSETS="$ROOT/web/assets"
MERMAID_URL="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"

mkdir -p "$ASSETS"

if [[ ! -f "$ASSETS/mermaid.min.js" ]]; then
  echo "==> Downloading mermaid.min.js"
  curl -fsSL -o "$ASSETS/mermaid.min.js" "$MERMAID_URL"
else
  echo "==> mermaid.min.js already present"
fi
