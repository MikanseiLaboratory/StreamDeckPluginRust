#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$ROOT/../.." && pwd)"
PLUGIN_ID="dev.flowingspdg.countersample.rust"
PLUGIN_DIR="$ROOT/$PLUGIN_ID.sdPlugin"
OUTPUT_DIR="${OUTPUT_DIR:-$REPO_ROOT/artifacts/plugin}"
INSTALL="${INSTALL:-0}"
PACK="${PACK:-0}"

publish_target() {
  local target="$1"
  local rid="$2"
  local name="$3"
  rustup target add "$target"
  cargo build -p counter-sample --release --bin counter_sample_plugin --target "$target"
  mkdir -p "$PLUGIN_DIR/bin/$rid"
  src=$(find "$REPO_ROOT/target/$target/release" -maxdepth 1 -name 'counter_sample_plugin*' | head -n 1)
  cp "$src" "$PLUGIN_DIR/bin/$rid/$name"
}

cd "$REPO_ROOT"
cargo run -p counter-sample --bin typegen
(cd "$ROOT/pi" && { [ -d node_modules ] || npm install; } && npm run build)

publish_target x86_64-pc-windows-msvc win-x64 "$PLUGIN_ID.exe" || true
publish_target aarch64-apple-darwin osx-arm64 "$PLUGIN_ID" || true
publish_target x86_64-apple-darwin osx-x64 "$PLUGIN_ID" || true

echo "Published plugin bundle to $PLUGIN_DIR"

if [ "$PACK" = "1" ]; then
  mkdir -p "$OUTPUT_DIR"
  ZIP="$OUTPUT_DIR/$PLUGIN_ID.streamDeckPlugin"
  rm -f "$ZIP"
  if command -v streamdeck >/dev/null 2>&1; then
    streamdeck pack "$PLUGIN_DIR" --output "$OUTPUT_DIR" --force
  else
    (cd "$(dirname "$PLUGIN_DIR")" && zip -qr "$ZIP" "$(basename "$PLUGIN_DIR")")
  fi
  echo "Packed $ZIP"
fi

if [ "$INSTALL" = "1" ]; then
  DEST="$HOME/Library/Application Support/com.elgato.StreamDeck/Plugins/$PLUGIN_ID.sdPlugin"
  mkdir -p "$(dirname "$DEST")"
  rm -rf "$DEST"
  cp -R "$PLUGIN_DIR" "$DEST"
  echo "Installed plugin to $DEST"
  if command -v streamdeck >/dev/null 2>&1; then
    streamdeck restart "$PLUGIN_ID"
  fi
fi
