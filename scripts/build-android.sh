#!/usr/bin/env bash
# Build omd Android APK: Web WASM bundle → Android assets → Gradle assemble
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WEB="$ROOT/web"
ANDROID="$ROOT/android"
ASSETS="$ANDROID/app/src/main/assets"
MERMAID_URL="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"

echo "==> [1/4] Download offline Mermaid.js (if missing)"
mkdir -p "$WEB/assets"
if [[ ! -f "$WEB/assets/mermaid.min.js" ]]; then
  curl -fsSL -o "$WEB/assets/mermaid.min.js" "$MERMAID_URL"
fi

echo "==> [2/4] Build Web WASM bundle (Trunk release)"
cd "$WEB"
if ! command -v trunk &>/dev/null; then
  echo "Installing trunk..."
  cargo install trunk --locked
fi
rustup target add wasm32-unknown-unknown 2>/dev/null || true
env -u NO_COLOR trunk build --release

echo "==> [3/4] Copy dist/ to Android assets"
rm -rf "$ASSETS"
mkdir -p "$ASSETS"
cp -a "$WEB/dist/." "$ASSETS/"
echo "    Assets: $(du -sh "$ASSETS" | cut -f1)"

echo "==> [4/4] Build Android APK"
cd "$ANDROID"
if [[ ! -f "./gradlew" ]]; then
  if command -v gradle &>/dev/null; then
    gradle wrapper --gradle-version 8.11.1
  else
    echo "Error: gradlew not found. Install Gradle or Android Studio, then run:"
    echo "  cd android && gradle wrapper"
    exit 1
  fi
fi
chmod +x ./gradlew
./gradlew assembleDebug

APK="$ANDROID/app/build/outputs/apk/debug/app-debug.apk"
if [[ -f "$APK" ]]; then
  echo ""
  echo "✅ Build complete!"
  echo "   APK: $APK"
  echo "   Install: adb install -r $APK"
else
  echo "❌ APK not found at expected path"
  exit 1
fi
