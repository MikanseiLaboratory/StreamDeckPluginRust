#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
PLUGIN_OUT="$ROOT/artifacts/plugin"
mkdir -p "$PLUGIN_OUT"
PACK=1 OUTPUT_DIR="$PLUGIN_OUT" "$ROOT/samples/counter-sample/publish.sh"
echo "Plugin package: $PLUGIN_OUT"
