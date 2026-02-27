#!/usr/bin/env bash
# haystack-setup.sh — One-time setup script for the Haystack code search server.
#
# Haystack is an optional, pre-indexed code search backend that Codex can use
# instead of ripgrep for large repositories. This script downloads the binary,
# writes a minimal configuration, and starts the server.
#
# Usage:
#   ./scripts/haystack-setup.sh [--port PORT] [--data-dir DIR]
#
# After setup, set the environment variable so Codex discovers the server:
#   export CODEX_HAYSTACK_URL="http://127.0.0.1:13135"

set -euo pipefail

# ── Defaults ──
HAYSTACK_PORT="${HAYSTACK_PORT:-13135}"
HAYSTACK_DATA_DIR="${HAYSTACK_DATA_DIR:-$HOME/.haystack}"
HAYSTACK_VERSION="${HAYSTACK_VERSION:-v0.3.3}"

# ── Parse flags ──
while [[ $# -gt 0 ]]; do
  case "$1" in
    --port)   HAYSTACK_PORT="$2";   shift 2 ;;
    --data-dir) HAYSTACK_DATA_DIR="$2"; shift 2 ;;
    --version) HAYSTACK_VERSION="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: $0 [--port PORT] [--data-dir DIR] [--version VERSION]"
      echo ""
      echo "Options:"
      echo "  --port      Port for the Haystack server (default: 13135)"
      echo "  --data-dir  Directory for Haystack data (default: ~/.haystack)"
      echo "  --version   Haystack release version (default: v0.3.3)"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# ── Platform detection ──
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64)  ARCH="amd64" ;;
  aarch64) ARCH="arm64" ;;
  *)
    echo "Error: unsupported architecture: $ARCH"
    echo "Haystack only supports linux-amd64 and linux-arm64."
    exit 1
    ;;
esac

if [ "$OS" != "linux" ]; then
  echo "Error: unsupported OS: $OS"
  echo "Haystack currently only supports Linux."
  exit 1
fi

PLATFORM="${OS}-${ARCH}"
echo "Detected platform: $PLATFORM"

# ── Directories ──
BIN_DIR="$HAYSTACK_DATA_DIR/bin"
mkdir -p "$BIN_DIR"

HAYSTACK_BIN="$BIN_DIR/haystack"

# ── Download ──
DOWNLOAD_URL="https://github.com/CodeTrek/haystack/releases/download/${HAYSTACK_VERSION}/haystack-${PLATFORM}-${HAYSTACK_VERSION}.zip"

if [ -x "$HAYSTACK_BIN" ]; then
  echo "Haystack binary already exists at $HAYSTACK_BIN"
else
  echo "Downloading Haystack ${HAYSTACK_VERSION} for ${PLATFORM}..."
  TMP_ZIP="$(mktemp)"
  if ! curl -fSL "$DOWNLOAD_URL" -o "$TMP_ZIP"; then
    echo "Error: failed to download from $DOWNLOAD_URL"
    rm -f "$TMP_ZIP"
    exit 1
  fi

  echo "Extracting..."
  unzip -o "$TMP_ZIP" -d "$BIN_DIR"
  rm -f "$TMP_ZIP"
  chmod +x "$HAYSTACK_BIN"
  echo "Installed Haystack to $HAYSTACK_BIN"
fi

# ── Configuration ──
CONFIG_FILE="$HAYSTACK_DATA_DIR/config.yaml"
if [ ! -f "$CONFIG_FILE" ]; then
  cat > "$CONFIG_FILE" <<EOF
global:
  data_path: ${HAYSTACK_DATA_DIR}/data
  port: ${HAYSTACK_PORT}
client:
  default_limit:
    max_results: 500
    max_results_per_file: 50
EOF
  echo "Wrote config to $CONFIG_FILE"
else
  echo "Config already exists at $CONFIG_FILE"
fi

# ── Start the server ──
echo "Starting Haystack server on port $HAYSTACK_PORT..."
"$HAYSTACK_BIN" server start &

# Wait a moment and verify
sleep 2
if curl -sf "http://127.0.0.1:${HAYSTACK_PORT}/health" > /dev/null 2>&1; then
  echo ""
  echo "✓ Haystack is running on http://127.0.0.1:${HAYSTACK_PORT}"
  echo ""
  echo "Add this to your shell profile to enable Codex integration:"
  echo ""
  echo "  export CODEX_HAYSTACK_URL=\"http://127.0.0.1:${HAYSTACK_PORT}\""
  echo ""
else
  echo ""
  echo "⚠ Haystack may still be starting. Check with:"
  echo "  curl http://127.0.0.1:${HAYSTACK_PORT}/health"
  echo ""
  echo "Once running, set:"
  echo "  export CODEX_HAYSTACK_URL=\"http://127.0.0.1:${HAYSTACK_PORT}\""
fi
