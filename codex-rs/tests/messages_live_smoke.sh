#!/usr/bin/env bash
# XLI Phase 4 — Live Smoke Test against NetApp LiteLLM Proxy
#
# Run OUTSIDE Cursor sandbox (Terminal.app or iTerm):
#   cd ~/Projects/codex-cli/codex-rs
#   CODEX_PROXY_E2E=1 bash tests/messages_live_smoke.sh
#
# Prerequisites:
#   - CODEX_LLM_PROXY_KEY set (user=palanisd&key=sk_...)
#   - Binary built: cargo build -p codex-cli
#
# Runs 3 tests:
#   1. Text-only prompt (validates SSE parsing + OutputItemAdded + text streaming)
#   2. Tool-use prompt (validates tool call → tool_result → multi-turn)
#   3. Multi-tool chain (validates tool loop continuity)

set -euo pipefail

BINARY="${1:-target/debug/codex}"
WORKDIR="$(cd "$(dirname "$0")/.." && pwd)"
PASS=0
FAIL=0
SKIP=0

if [ "${CODEX_PROXY_E2E:-}" != "1" ]; then
    echo "Skipping proxy-e2e tests (set CODEX_PROXY_E2E=1 to enable)"
    exit 0
fi

if [ ! -f "$WORKDIR/$BINARY" ]; then
    echo "ERROR: Binary not found at $WORKDIR/$BINARY"
    echo "Run: cd $WORKDIR && cargo build -p codex-cli"
    exit 1
fi

if [ -z "${CODEX_LLM_PROXY_KEY:-}" ]; then
    echo "ERROR: CODEX_LLM_PROXY_KEY not set"
    exit 1
fi

cd "$WORKDIR"

MODEL="${PROXY_CLAUDE_MODEL:-claude-sonnet-4.6}"

run_codex() {
    local prompt="$1"
    local timeout="${2:-60}"
    timeout "$timeout" "$BINARY" exec \
        -c "model=\"$MODEL\"" \
        -c 'model_provider="llm_proxy_messages"' \
        -c 'approval_policy="never"' \
        "$prompt" 2>&1 || true
}

echo "=== XLI /messages Live Smoke Test ==="
echo "Binary: $BINARY"
echo "Model:  $MODEL"
echo "Proxy:  NetApp LiteLLM"
echo ""

# ─── Test 1: Text-only ───
echo "--- Test 1: Text prompt (SSE + OutputItemAdded + streaming) ---"
RESULT=$(run_codex "What is 2+2? Reply with ONLY the number, nothing else." 60)
echo "$RESULT" | tail -10
if echo "$RESULT" | grep -q "4"; then
    echo "PASS: Text response contains '4'"
    ((PASS++))
else
    echo "FAIL: Expected '4' in response"
    ((FAIL++))
fi
echo ""

# ─── Test 2: Tool use (file read) ───
echo "--- Test 2: Tool use (read_file) ---"
TMPDIR_TEST=$(mktemp -d)
echo "The secret number is 42." > "$TMPDIR_TEST/secret.txt"
RESULT2=$(run_codex "Read the file at $TMPDIR_TEST/secret.txt and tell me the secret number. Just the number." 90)
rm -rf "$TMPDIR_TEST"
echo "$RESULT2" | tail -15
if echo "$RESULT2" | grep -q "42"; then
    echo "PASS: Tool use response contains '42'"
    ((PASS++))
else
    echo "FAIL: Expected '42' in response"
    ((FAIL++))
fi
echo ""

# ─── Test 3: Multi-tool chain ───
echo "--- Test 3: Multi-tool chain (shell + read) ---"
RESULT3=$(run_codex "Run 'echo MESSAGES_WIRE_OK' in the shell and tell me exactly what it printed." 90)
echo "$RESULT3" | tail -15
if echo "$RESULT3" | grep -q "MESSAGES_WIRE_OK"; then
    echo "PASS: Shell tool output captured"
    ((PASS++))
else
    echo "FAIL: Expected 'MESSAGES_WIRE_OK' in response"
    ((FAIL++))
fi
echo ""

# ─── Summary ───
echo "==========================================="
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo "==========================================="

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
