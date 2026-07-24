#!/usr/bin/env bash
# Build the Web bundle with Trunk; retry wasm-opt download failures (GitHub 504).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WEB="${ROOT}/web"
delays=(4 8 16 32)

cd "$WEB"

build_trunk() {
  env -u NO_COLOR trunk build --release "$@"
}

for i in "${!delays[@]}"; do
  attempt=$((i + 1))
  if build_trunk; then
    exit 0
  fi
  echo "trunk build failed (attempt ${attempt}/${#delays[@]}), retrying in ${delays[$i]}s..." >&2
  sleep "${delays[$i]}"
done

echo "trunk build failed after retries; building without wasm-opt" >&2
html="index.html"
bak="${html}.trunk-build.bak"
cp "$html" "$bak"
sed -i 's/ data-wasm-opt="[^"]*"//g; s/ data-wasm-opt-params="[^"]*"//g' "$html"
if build_trunk; then
  mv -f "$bak" "$html"
  exit 0
fi
mv -f "$bak" "$html"
exit 1
