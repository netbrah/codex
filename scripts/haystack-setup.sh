#!/usr/bin/env bash
# haystack-setup.sh — one-time setup for the Haystack indexed code-search backend.
#
# Usage: ./scripts/haystack-setup.sh
#
# Expects the Haystack binary (and optional ctags binary) to be present in a
# ./bin/ directory relative to this script.  The binary is NOT included in
# the repository (it is ~24 MB and proprietary).

set -euo pipefail

# ---------------------------------------------------------------------------
# Platform detection
# ---------------------------------------------------------------------------
ARCH=$(uname -m)
OS=$(uname -s)

if [[ "$OS" != "Linux" ]]; then
  echo "error: haystack-setup.sh only supports Linux (got $OS)" >&2
  exit 1
fi

case "$ARCH" in
  x86_64)  PLATFORM="linux-amd64" ;;
  aarch64) PLATFORM="linux-arm64" ;;
  *)
    echo "error: unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

echo "Detected platform: $PLATFORM"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_SRC_DIR="$SCRIPT_DIR/bin"

if [[ ! -f "$BIN_SRC_DIR/haystack" ]]; then
  echo "error: haystack binary not found at $BIN_SRC_DIR/haystack" >&2
  echo "Please download the Haystack binary for $PLATFORM and place it at $BIN_SRC_DIR/haystack" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Directory setup
# ---------------------------------------------------------------------------
HAYSTACK_HOME="$HOME/.haystack-codex"
BIN_DIR="$HAYSTACK_HOME/bin"
mkdir -p "$BIN_DIR"

# ---------------------------------------------------------------------------
# Detect NFS home directory and redirect data to local disk if needed
# ---------------------------------------------------------------------------
DATA_DIR=""
if grep -qF " $HOME " /proc/mounts 2>/dev/null && grep -qE " (nfs|nfs4|cifs|afs|lustre|gpfs) " /proc/mounts 2>/dev/null; then
  echo "Detected NFS home directory — searching for local scratch space..."
  for candidate in "/local/$USER" "/scratch/$USER" "/var/tmp/$USER-haystack-codex" "/tmp/$USER-haystack-codex"; do
    if mkdir -p "$candidate" 2>/dev/null; then
      DATA_DIR="$candidate/data"
      echo "Using local data directory: $DATA_DIR"
      break
    fi
  done
fi

if [[ -z "$DATA_DIR" ]]; then
  DATA_DIR="$HAYSTACK_HOME/data"
fi

mkdir -p "$DATA_DIR"

# ---------------------------------------------------------------------------
# Install binaries
# ---------------------------------------------------------------------------
cp "$BIN_SRC_DIR/haystack" "$BIN_DIR/haystack"
chmod +x "$BIN_DIR/haystack"

if [[ -f "$BIN_SRC_DIR/ctags" ]]; then
  cp "$BIN_SRC_DIR/ctags" "$BIN_DIR/ctags"
  chmod +x "$BIN_DIR/ctags"
fi

echo "Installed binaries to $BIN_DIR"

# ---------------------------------------------------------------------------
# Write config.yaml
# ---------------------------------------------------------------------------
CONFIG_FILE="$HAYSTACK_HOME/config.yaml"
cat > "$CONFIG_FILE" <<EOF
server:
  port: 13136
  host: 127.0.0.1
data_dir: $DATA_DIR
EOF
echo "Wrote config: $CONFIG_FILE"

# ---------------------------------------------------------------------------
# Systemd user service
# ---------------------------------------------------------------------------
SYSTEMD_DIR="$HOME/.config/systemd/user"
mkdir -p "$SYSTEMD_DIR"

SERVICE_FILE="$SYSTEMD_DIR/haystack-codex.service"
cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=Haystack indexed code search (codex integration)
After=network.target

[Service]
ExecStart=$BIN_DIR/haystack server run --config $CONFIG_FILE
Restart=always
RestartSec=3
Environment=HOME=$HOME

[Install]
WantedBy=default.target
EOF
echo "Wrote systemd service: $SERVICE_FILE"

systemctl --user daemon-reload
systemctl --user enable haystack-codex.service
systemctl --user restart haystack-codex.service
echo "Started haystack-codex systemd service"

# ---------------------------------------------------------------------------
# Wait for health check
# ---------------------------------------------------------------------------
HAYSTACK_URL="http://127.0.0.1:13136"
echo -n "Waiting for Haystack to become healthy"
for i in $(seq 1 30); do
  if curl -sf "$HAYSTACK_URL/health" >/dev/null 2>&1; then
    echo " OK"
    break
  fi
  echo -n "."
  sleep 1
  if [[ $i -eq 30 ]]; then
    echo ""
    echo "warning: Haystack did not become healthy within 30 seconds." >&2
    echo "Check: systemctl --user status haystack-codex" >&2
  fi
done

# ---------------------------------------------------------------------------
# Export CODEX_HAYSTACK_URL in shell rc files
# ---------------------------------------------------------------------------
EXPORT_LINE="export CODEX_HAYSTACK_URL=$HAYSTACK_URL"

for RC in "$HOME/.bashrc" "$HOME/.zshrc"; do
  if [[ -f "$RC" ]] && ! grep -qF "CODEX_HAYSTACK_URL" "$RC"; then
    echo "" >> "$RC"
    echo "# Added by haystack-setup.sh" >> "$RC"
    echo "$EXPORT_LINE" >> "$RC"
    echo "Added CODEX_HAYSTACK_URL to $RC"
  fi
done

echo ""
echo "Setup complete! Restart your shell or run:"
echo "  $EXPORT_LINE"
echo "to enable Haystack search in your current session."
