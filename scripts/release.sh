#!/usr/bin/env bash
# 0.3.0+ release artifacts locally (desktop + web).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
OUT="$ROOT/target/release-artifacts"
mkdir -p "$OUT"

echo "==> omd release build v${VERSION}"

echo "==> [1/2] Desktop (release)"
cargo build --release
cp target/release/omd "$OUT/omd-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"

echo "==> [2/2] Web (trunk release)"
cd web
if ! command -v trunk &>/dev/null; then
  echo "Installing trunk..."
  cargo install trunk --locked
fi
rustup target add wasm32-unknown-unknown 2>/dev/null || true
env -u NO_COLOR trunk build --release
tar -czf "$OUT/omd-web-dist-v${VERSION}.tar.gz" -C dist .
cd "$ROOT"

echo ""
echo "Done. Artifacts in: $OUT"
ls -lh "$OUT"
