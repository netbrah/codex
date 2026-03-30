# Messages Wire Branch — Comprehensive Code Review

**Branch:** `apex/messages-wire`
**Diff:** 3,459 lines added, 42 removed across 33 files (11 commits)
**Date:** 2026-03-25

---

## Architecture Summary

The branch adds Anthropic `/messages` wire protocol support to codex-rs alongside the existing OpenAI `/responses` wire. The stack is:

```
config (WireApi::Messages) -> client.rs (stream_messages_api)
  -> messages_wire.rs (history translator)
  -> codex-api endpoint/messages.rs (HTTP client)
  -> codex-api sse/messages.rs (SSE parser)
  -> ResponseEvent (shared with responses wire)
```

Plus: structured XML compaction prompt, 70/30 split logic, Claude model registry entries, and E2E test harness.

---

## Issues Found

### P0 — Correctness

#### 1. `developer` role messages silently mapped to `user` (messages_wire.rs:24)

```rust
let anthropic_role = match role.as_str() {
    "system" => continue,
    "user" => "user",
    "assistant" => "assistant",
    _ => "user",  // <-- developer messages become user messages
};
```

Codex uses `developer` role for system-level instructions (permissions, personality, AGENTS.md). The catch-all `_ => "user"` silently converts these to user messages, which:
- Pollutes the user conversation with system instructions
- Can confuse Claude about what is user input vs system directives
- Differs from the intended `system` prompt placement

**Fix options:** Either extract `developer` messages into the `system` parameter (like `system` messages are skipped), or explicitly `continue` to skip them since they're already injected via `base_instructions`.

#### 2. 70/30 split adds synthetic assistant ack but the compact.rs code has `MIN_ITEMS_FOR_SPLIT = 20`

The code adds a synthetic assistant message `"Understood. Continuing from where we left off."` (compact.rs:218-224) when the split is active. This is fine for Claude, but:
- The ack message has no corresponding user message before it if the preserved items start with a user message — creating `[summary(user), ack(assistant), user_msg, ...]` which violates strict alternation for some providers
- Worse: the `MIN_ITEMS_FOR_SPLIT = 20` threshold means short sessions use the old path while long ones use the new path, creating two different post-compact history shapes depending on session length. This makes tests brittle and behavior inconsistent.

**Note:** The 12 failing tests from the previous attempt are likely because most test histories are under 20 items, so they never exercise the split path — and the tests that DO exceed 20 items see the new shape and fail.

#### 3. `anthropic_max_output_tokens` uses substring matching (client.rs, last hunk)

```rust
fn anthropic_max_output_tokens(slug: &str) -> u32 {
    if slug.contains("opus") {
        128_000
    } else if slug.contains("haiku") {
        8_192
    } else {
        64_000
    }
}
```

A model slug like `my-opus-proxy-haiku` or `company/opus-tuned` would match incorrectly. This is documented as "intentionally a safety cap" but is fragile. A user-defined model named `opus-clone` that isn't actually Opus would get 128K max tokens.

#### 4. Token usage `total_tokens` calculation may be wrong (sse/messages.rs:461)

```rust
total_tokens: input + output,
```

Comment says "Anthropic's input_tokens already includes cache reads". This is incorrect — Anthropic's `input_tokens` does NOT include `cache_read_input_tokens` in newer API versions. The actual total should be `input + cache_read + output`. Check the Anthropic API docs for the version pinned in `anthropic-version: 2023-06-01`.

#### 5. `should_use_remote_compact_task` now excludes Messages wire from remote compact (compact.rs:57)

```rust
pub(crate) fn should_use_remote_compact_task(provider: &ModelProviderInfo) -> bool {
    provider.is_openai() && provider.wire_api == WireApi::Responses
}
```

This means Claude models always use inline compaction (local LLM summarization), never remote. This is correct since Anthropic doesn't have a `/responses/compact` endpoint, but the function name `should_use_remote_compact_task` doesn't make this obvious. Consider adding a comment.

### P1 — API/Protocol

#### 6. Missing `anthropic-beta` header for extended thinking

Anthropic's extended thinking feature requires `anthropic-beta: interleaved-thinking-2025-05-14` header (or similar). The code sends `anthropic-version: 2023-06-01` but no beta header. This may cause thinking blocks to not be returned by the API.

#### 7. `MessagesApiRequest` path is hardcoded to `"messages"` (endpoint/messages.rs:105)

The provider base URL is expected to end with `/v1`. For proxies that route to `/v1/messages`, this works. For proxies that expect `/messages` at root, this fails. The comment at line 98-106 acknowledges this but doesn't handle it.

#### 8. `stream_messages_api` doesn't set `x-codex-turn-state` header (client.rs)

The Responses API path sets various headers (`OPENAI_BETA_HEADER`, `X_CODEX_TURN_STATE_HEADER`). The Messages path only sets `x-codex-turn-metadata`. This might be intentional (those are OpenAI-specific) but should be documented.

#### 9. `map_api_error` change for RateLimit may break retry logic (api_bridge.rs)

```rust
-ApiError::RateLimit(msg) => CodexErr::Stream(msg, None),
+ApiError::RateLimit(_msg) => CodexErr::RetryLimit(RetryLimitReachedError {
+    status: http::StatusCode::TOO_MANY_REQUESTS,
+    request_id: None,
+})
```

This changes rate limit errors from `Stream` (retried by the backoff loop in `run_compact_task_inner`) to `RetryLimit` (may not be retried). Check that `RetryLimitReachedError` is handled by retry logic in `ModelClientSession::stream()`.

### P2 — Code Quality

#### 10. `_stop_reason` is captured but never used (sse/messages.rs:97)

```rust
let mut _stop_reason: Option<String> = None;
```

The underscore prefix suppresses the unused-variable warning, but if this is meant to be used for distinguishing `end_turn` vs `tool_use` stop reasons, it should be wired up.

#### 11. `conversation_to_anthropic_messages` uses `unwrap_or(json!({}))` for argument parsing (messages_wire.rs:59)

```rust
let input_val: Value = serde_json::from_str(arguments).unwrap_or(json!({}));
```

If arguments are malformed JSON, this silently sends an empty `{}` input to the tool. Should at least log a warning.

#### 12. Thinking block uses `summary` field for thinking text (sse/messages.rs:384-387)

The `Reasoning` item stores the full thinking text in `summary` (a `Vec<ReasoningItemReasoningSummary>`) rather than `content` (which is `Option<Vec<ReasoningItemContent>>`). The `content` field exists exactly for this purpose. Using `summary` works but is semantically wrong — the thinking text is the full content, not a summary.

This also means the roundtrip in `messages_wire.rs` has to check `content` first, then fall back to `summary` (line 145-176), adding complexity.

#### 13. `memory_trace.rs` early-return for empty output is a workaround (memory_trace.rs)

```rust
if output.is_empty() && !prepared.is_empty() {
    tracing::debug!("memory summarization not available...");
    return Ok(Vec::new());
}
```

This silently discards trace data when the Messages wire doesn't support memory summarization. Users won't know their memories aren't being created. Should emit a user-visible warning at least once per session.

#### 14. Three identical `WireApi::Responses | WireApi::Messages` matches in display code

The same pattern is duplicated in:
- `exec/src/event_processor_with_human_output.rs`
- `tui/src/status/card.rs`
- `tui_app_server/src/status/card.rs`

When `WireApi` gains more variants, all three must be updated. Consider adding `fn supports_reasoning_effort(&self) -> bool` to `WireApi`.

#### 15. No exhaustive match for `WireApi` in key places

The `WireApi` enum now has two variants but several `match` expressions use `==` comparisons instead of exhaustive matches. If a third wire API is added, these won't produce compiler warnings:
- `should_use_remote_compact_task` (compact.rs:57) — uses `==`
- Display code mentioned above — uses `matches!` which is fine but not exhaustive

### P3 — Testing

#### 16. E2E tests are gated behind `CODEX_PROXY_E2E=1` with hardcoded NetApp proxy URL

```rust
const PROXY_BASE_URL: &str = "https://<your-llm-proxy>/v1";  // ← hardcoded; move to env var
```

This URL is specific to NetApp's internal infrastructure. If this branch is intended for upstream, the URL should be configurable via environment variable.

#### 17. No unit tests for `stream_messages_api` error paths

The `stream_messages_api` function handles auth recovery, transport errors, and rate limits. No tests exercise these paths. The Responses API path likely has these tests; parity is needed.

#### 18. `messages_end_to_end.rs` doesn't test prompt caching behavior

The `cache_control: {"type": "ephemeral"}` annotation is added to tools and system prompts, but no test verifies these are included in the serialized request body.

#### 19. No test for `developer` role handling in `conversation_to_anthropic_messages`

No test verifies what happens when `developer` role messages are in the input (Issue #1 above).

### P4 — Minor/Style

#### 20. `#[allow(clippy::too_many_arguments)]` on `stream_messages_api`

This mirrors the existing pattern on `stream_responses_api_http`, but both should ideally use a config/params struct.

#### 21. Two `.clone()` calls on `call_id` and `name` in SSE tool_use block_start (sse/messages.rs:231-233)

These clones are needed for the tracker and the emitted event, but the event could take references if the tracker owns the data. Minor allocation.

#### 22. `fixture_to_byte_stream` maps identity `.map(|b| b)` (sse/messages.rs:547)

```rust
let stream = ReaderStream::new(reader).map(|r| {
    r.map(|b| b)  // <-- redundant identity map
        .map_err(|e| codex_client::TransportError::Network(e.to_string()))
});
```

The `.map(|b| b)` is a no-op. Use `.map_err(...)` directly.

---

## Summary

| Severity | Count | Key Items |
|----------|-------|-----------|
| P0 Correctness | 5 | developer role leak, split threshold, token math, output cap matching, compact path |
| P1 API/Protocol | 4 | missing beta header, path handling, header parity, retry behavior change |
| P2 Code Quality | 6 | unused vars, silent fallbacks, semantic mismatch, duplication |
| P3 Testing | 4 | hardcoded URLs, missing error path tests, no developer role test |
| P4 Style | 3 | too_many_args, unnecessary clones, identity map |

**Recommendation:** Fix P0 items #1 (developer role) and #4 (token math) before merging. Item #2 (MIN_ITEMS_FOR_SPLIT threshold creating two code paths) is the root cause of the 12 test failures — consider removing the threshold and always using the split logic (with `find_compact_split_point` naturally handling small histories by returning `items.len()` when everything fits in 70%).
