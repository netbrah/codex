#!/usr/bin/env bash
# Proxy E2E Pre-Commit Hook
#
# Runs /messages wire integration tests against a live LiteLLM proxy.
# Only runs when explicitly opted in via CODEX_PROXY_E2E=1.
#
# Usage:
#   CODEX_PROXY_E2E=1 bash codex-rs/scripts/proxy-e2e-precommit.sh

set -euo pipefail

if [ "${CODEX_PROXY_E2E:-}" != "1" ]; then
    echo "Skipping proxy-e2e tests (set CODEX_PROXY_E2E=1 to enable)"
    exit 0
fi

if [ -z "${CODEX_LLM_PROXY_KEY:-}" ]; then
    echo "ERROR: CODEX_LLM_PROXY_KEY not set"
    exit 1
fi

cd "$(dirname "$0")/.."

echo "Building codex..."
cargo build -p codex-cli 2>&1 | tail -3

echo ""
echo "Running proxy-e2e tests (sequential to avoid rate limits)..."
ANTHROPIC_API_KEY="$CODEX_LLM_PROXY_KEY" \
cargo test --test proxy_e2e -- --test-threads=1 --nocapture \
    2>&1 | tee /tmp/proxy-e2e-results.txt

EXIT=${PIPESTATUS[0]}
echo ""
if [ "$EXIT" -eq 0 ]; then
    echo "PROXY-E2E: ALL TESTS PASSED"
else
    echo "PROXY-E2E FAILED — see /tmp/proxy-e2e-results.txt"
fi
exit "$EXIT"
