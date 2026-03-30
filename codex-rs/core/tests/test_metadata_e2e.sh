#!/usr/bin/env bash
# Live E2E test harness for W-5: metadata.user_id propagation
#
# Usage:
#   ./test_metadata_e2e.sh <PROXY_BASE_URL> <API_KEY>
#
# Example:
#   ./test_metadata_e2e.sh https://litellm.corp.example.com sk-xxxxx
#
# Prerequisites:
#   - curl with JSON support
#   - jq installed
#
# This script sends a real Anthropic Messages API request through a proxy
# with metadata.user_id set and verifies:
# 1. The request is accepted (HTTP 200)
# 2. The response contains a valid message
# 3. The metadata field doesn't cause rejection

set -euo pipefail

BASE_URL="${1:?Usage: $0 <BASE_URL> <API_KEY>}"
API_KEY="${2:?Usage: $0 <BASE_URL> <API_KEY>}"
USER_ID="${3:-$(whoami)}"

echo "=== W-5 E2E Test: metadata.user_id ==="
echo "Base URL: $BASE_URL"
echo "User ID:  $USER_ID"
echo ""

# -----------------------------------------------------------------------
# Test 1: Request with metadata.user_id is accepted
# -----------------------------------------------------------------------
echo "--- Test 1: metadata.user_id accepted ---"

RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "${BASE_URL}/v1/messages" \
    -H "Content-Type: application/json" \
    -H "x-api-key: ${API_KEY}" \
    -H "anthropic-version: 2023-06-01" \
    -d '{
        "model": "claude-sonnet-4.6",
        "max_tokens": 16,
        "stream": false,
        "metadata": {
            "user_id": "'"${USER_ID}"'"
        },
        "messages": [
            {"role": "user", "content": "Say the number 42 and nothing else."}
        ]
    }')

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    echo "  PASS: HTTP 200 — request with metadata.user_id accepted"
else
    echo "  FAIL: HTTP $HTTP_CODE"
    echo "  Body: $BODY"
    exit 1
fi

# Verify response has expected structure
MSG_ID=$(echo "$BODY" | jq -r '.id // empty')
if [ -n "$MSG_ID" ]; then
    echo "  PASS: Response has message id: $MSG_ID"
else
    echo "  FAIL: No message id in response"
    echo "  Body: $BODY"
    exit 1
fi

STOP_REASON=$(echo "$BODY" | jq -r '.stop_reason // empty')
echo "  INFO: stop_reason=$STOP_REASON"

# -----------------------------------------------------------------------
# Test 2: Request without metadata is still accepted (backwards compat)
# -----------------------------------------------------------------------
echo ""
echo "--- Test 2: no metadata (backwards compat) ---"

RESPONSE2=$(curl -s -w "\n%{http_code}" \
    -X POST "${BASE_URL}/v1/messages" \
    -H "Content-Type: application/json" \
    -H "x-api-key: ${API_KEY}" \
    -H "anthropic-version: 2023-06-01" \
    -d '{
        "model": "claude-sonnet-4.6",
        "max_tokens": 16,
        "stream": false,
        "messages": [
            {"role": "user", "content": "Say yes."}
        ]
    }')

HTTP_CODE2=$(echo "$RESPONSE2" | tail -1)
if [ "$HTTP_CODE2" = "200" ]; then
    echo "  PASS: HTTP 200 — request without metadata still works"
else
    echo "  FAIL: HTTP $HTTP_CODE2"
    exit 1
fi

# -----------------------------------------------------------------------
# Test 3: Streaming request with metadata.user_id
# -----------------------------------------------------------------------
echo ""
echo "--- Test 3: streaming with metadata.user_id ---"

STREAM_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "${BASE_URL}/v1/messages" \
    -H "Content-Type: application/json" \
    -H "x-api-key: ${API_KEY}" \
    -H "anthropic-version: 2023-06-01" \
    -d '{
        "model": "claude-sonnet-4.6",
        "max_tokens": 16,
        "stream": true,
        "metadata": {
            "user_id": "'"${USER_ID}"'"
        },
        "messages": [
            {"role": "user", "content": "Say hello."}
        ]
    }')

HTTP_CODE3=$(echo "$STREAM_RESPONSE" | tail -1)
SSE_BODY=$(echo "$STREAM_RESPONSE" | sed '$d')

if [ "$HTTP_CODE3" = "200" ]; then
    echo "  PASS: HTTP 200 — streaming with metadata.user_id accepted"
else
    echo "  FAIL: HTTP $HTTP_CODE3"
    echo "  Body: $SSE_BODY"
    exit 1
fi

# Check for message_start event in SSE stream
if echo "$SSE_BODY" | grep -q "event: message_start"; then
    echo "  PASS: SSE stream contains message_start event"
else
    echo "  FAIL: No message_start in SSE stream"
    exit 1
fi

if echo "$SSE_BODY" | grep -q "event: message_stop"; then
    echo "  PASS: SSE stream completed with message_stop"
else
    echo "  WARN: No message_stop in SSE stream (may be truncated)"
fi

# -----------------------------------------------------------------------
# Test 4: metadata with thinking enabled
# -----------------------------------------------------------------------
echo ""
echo "--- Test 4: metadata + thinking ---"

THINKING_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "${BASE_URL}/v1/messages" \
    -H "Content-Type: application/json" \
    -H "x-api-key: ${API_KEY}" \
    -H "anthropic-version: 2023-06-01" \
    -H "anthropic-beta: interleaved-thinking-2025-05-14" \
    -d '{
        "model": "claude-sonnet-4.6",
        "max_tokens": 4096,
        "stream": false,
        "thinking": {"type": "enabled", "budget_tokens": 2048},
        "metadata": {
            "user_id": "'"${USER_ID}"'"
        },
        "messages": [
            {"role": "user", "content": "Say 1."}
        ]
    }')

HTTP_CODE4=$(echo "$THINKING_RESPONSE" | tail -1)
if [ "$HTTP_CODE4" = "200" ]; then
    echo "  PASS: HTTP 200 — metadata + thinking accepted"
else
    echo "  FAIL: HTTP $HTTP_CODE4"
    BODY4=$(echo "$THINKING_RESPONSE" | sed '$d')
    echo "  Body: $BODY4"
    exit 1
fi

echo ""
echo "=== ALL E2E TESTS PASSED ==="
echo "Evidence: metadata.user_id=$USER_ID accepted on all request variants"
