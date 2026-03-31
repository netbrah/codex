# Codex CLI (XLI) — Development Instructions

Fork of OpenAI Codex (Rust) with native Anthropic `/messages` wire protocol.
Primary branch: `dev`. Upstream: `upstream/main` (openai/codex).

---

## Build & Test

```bash
cd codex-rs

# Type check (fastest — no linking)
cargo check

# Unit + integration tests (run after every change)
cargo test -p codex-core -p codex-api

# Full test suite (all integration tests)
cargo test -p codex-core --test all

# Specific test module
cargo test -p codex-core messages_wire
cargo test -p codex-core extract_developer_blocks
cargo test -p codex-core anthropic_thinking

# After config struct changes (regenerates schema)
cargo run -p codex-core --bin codex-write-config-schema
```

---

## Architecture

```
config (WireApi::Messages) -> client.rs (stream_messages_api)
  -> messages_wire.rs (history translator: ResponseItem[] -> Anthropic messages[])
  -> codex-api/endpoint/messages.rs (HTTP client: builds MessagesApiRequest)
  -> codex-api/sse/messages.rs (SSE parser: content_block_* events -> ResponseEvent)
  -> ResponseEvent (shared type with /responses wire)
```

### Key Files

| Area | File | Notes |
|------|------|-------|
| Wire dispatch | `core/src/client.rs` | `stream()` routes by WireApi; `stream_messages_api()` at line ~1159 |
| History translator | `core/src/messages_wire.rs` | ResponseItem[] -> Anthropic messages[]; `extract_developer_blocks()` line 231 |
| HTTP client | `codex-api/src/endpoint/messages.rs` | MessagesApiRequest, auth headers, streaming |
| SSE parser | `codex-api/src/sse/messages.rs` | Parses content_block_start/delta/stop, message_delta |
| Wire enum | `core/src/model_provider_info.rs` | WireApi::Responses / WireApi::Messages |
| Compaction | `core/src/compact.rs` | 70/30 split; PRESERVE_RECENT_FRACTION = 0.30 |
| Thinking param | `core/src/client.rs` line 2110 | anthropic_thinking_param() |
| Regression guards | `core/src/messages_wire_regression_tests.rs` | Field-by-field null-space guards |
| Developer role fix | `core/src/client.rs` line 1197-1220 | W-7/BREAK-1: injects developer-role into system[] |

---

## Test Mandate (NON-OPTIONAL)

**Every sortie MUST include tests. No exceptions.**

| Change type | Required tests |
|-------------|---------------|
| New function | 3-5 unit tests: happy path, None/empty input, error path, edge case |
| Bug fix | Regression test that asserts wrong behavior before fix, passes after |
| New wire field | Unit test asserting field present in serialized JSON; test for absent case |
| Converter change | Round-trip: build ResponseItem, call translator, assert output JSON fields |
| SSE parser change | Feed raw SSE bytes via CapturingTransport, assert ResponseEvent sequence |
| Breaking change | Regression guard in messages_wire_regression_tests.rs |

### Test Patterns

```rust
// Standard unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_does_the_thing() {
        let input = vec![/* ... */];
        let result = my_function(&input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "expected");
    }

    #[test]
    fn returns_empty_on_empty_input() {
        let result = my_function(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn skips_non_matching_roles() {
        // Build a system-role item — extract_developer_blocks should ignore it
        let input = ResponseItem { role: "system".into(), /* ... */ };
        let result = extract_developer_blocks(&[input]);
        assert!(result.is_empty(), "system role should be ignored");
    }
}
```

```rust
// CapturingTransport — test actual HTTP request body
use codex_core::test_support::CapturingTransport;

let transport = CapturingTransport::new();
let client = ModelClient::new_with_transport(config, transport.clone());
client.stream(prompt, items).await;
let body: serde_json::Value = serde_json::from_str(&transport.last_request_body()).unwrap();
assert_eq!(body["system"][0]["type"], "text");
assert!(body["system"][0]["text"].as_str().unwrap().contains("developer content"));
```

```rust
// Regression guard — put in messages_wire_regression_tests.rs
#[test]
fn w7_developer_blocks_appear_in_system_param() {
    // MUST FAIL if extract_developer_blocks() is removed or emptied
    let input = vec![/* developer-role ResponseItem with text "AGENTS.md content" */];
    let blocks = extract_developer_blocks(&input);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], "AGENTS.md content here");
}
```

---

## Live Proxy E2E Tests (Corp Network Required)

### Environment Setup

All env vars are pre-configured in `~/.bashrc`. Confirm before running:

```bash
echo $CODEX_LLM_PROXY_KEY      # must be non-empty
echo $CODEX_PROXY_BASE_URL     # https://llm-proxy-api.ai.eng.netapp.com
```

### Live Messages Test (S-013)

Runs the full /messages wire against real Claude via the NetApp proxy:

```bash
cd codex-rs
cargo test -p codex-core --test all -- live_messages --ignored --test-threads=1
```

Auto-skips if `CODEX_LLM_PROXY_KEY` is empty. Spawns actual `codex` binary, verifies:
- exit code 0
- valid JSON event stream
- model responds with expected content

### Proxy E2E Messages Test (LiteLLM to Vertex AI to Claude)

```bash
cd codex-rs
CODEX_PROXY_E2E=1 CODEX_PROXY_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com" \
  cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1
```

Covers: streaming SSE parsing, tool_use round-trip, thinking/reasoning mode, Vertex AI routing.
Gated by `CODEX_PROXY_E2E=1` — safe to run in CI with that env set.

### Responses Wire Live Test (GPT via proxy)

```bash
cd codex-rs
cargo test -p codex-core --test live_cli -- --ignored --test-threads=1
```

### When to Run E2E

| Change | Run E2E |
|--------|---------|
| messages_wire.rs | Yes — live_messages |
| client.rs stream_messages_api | Yes — live_messages |
| SSE parser (sse/messages.rs) | Yes — live_messages |
| /responses wire changes | Yes — live_cli |
| compact.rs, config | No — unit tests sufficient |
| Test-only changes | No |

---

## Proprietary Layer — YOU ARE ON THE EMBED BRANCH

This branch (`feat/xli-embed-assets`) is a linear rebase stack on top of `dev`.

| Path | Content |
|------|---------|
| `deploy/build.sh` | Release build + packaging |
| `deploy/npm/` | npm package scaffolding for XLI |
| `deploy/skills/ontap-dev-guide/` | ONTAP-specific skill |
| `AGENTS.md` | Proprietary inventory (this branch only) |
| `CLAUDE.md` (this file) | Corp-specific instructions with proxy URLs |
| `deploy/npm/test/` | Home isolation env-bridging tests (S-040) |

**Never commit proprietary content to `dev`.** Proprietary = anything with corp URLs, internal paths, or NetApp branding.

### Home Isolation (S-040)

XLI defaults runtime state to `~/.xli` (not `~/.codex`).  The launcher
(`deploy/npm/bin/xli.js`) sets `CODEX_HOME` to `XLI_HOME` when the
operator hasn't explicitly set `CODEX_HOME`.

```bash
# Default state dir
XLI_HOME=~/.xli          # auto-set if unset

# To override:
export XLI_HOME=/custom/path

# Run env-bridging tests:
node --test deploy/npm/test/test-home-isolation.mjs
```

### Rebase Procedure

```bash
git checkout feat/xli-embed-assets
git fetch origin
git rebase dev
git push origin feat/xli-embed-assets --force-with-lease
```

---

## Sortie Instructions — ALL AGENTS READ THIS

### Rules
1. Work ONLY on your assigned branch (`apex/sortie/...`)
2. Do NOT switch to dev or feat/xli-embed-assets — C2 merges
3. Do NOT revert other agents' commits
4. Every commit: conventional prefix (`feat:`, `fix:`, `test:`, `refactor:`)

### Sortie Completion Gate

Final commit message OR `SORTIE-NOTES.md` MUST contain:

```
## Sortie Completion Notes
- Unit tests: [PASS/FAIL — cargo test -p codex-core]
- Live messages e2e: [PASS/FAIL/SKIPPED (not a wire change)]
- Proxy e2e: [PASS/FAIL/SKIPPED]
- Wire behavior changed: [yes/no — if yes: which field, which wire]
- New feature: [yes/no — if yes: suggest CA-{N}, tier: public/proprietary]
- Null-space gap closed: [yes/no — if yes: which field, remaining count]
- Cross-pollination: [yes/no — does Apex (TS/qwen-code) need this?]
- Regression risk: [low/medium/high — why]
```

### Convergence (Re-dispatch on existing branch)

```bash
git log --oneline          # what is done
cat SORTIE-NOTES.md        # what is missing
cargo test -p codex-core   # re-verify tests pass
# Fill in missing fields, continue from last step
```

---

## Hub Reference

| Question | File |
|----------|------|
| Big picture | ~/Projects/cli-ops/AGENTS.md |
| Feature registry (CA-1..CA-9) | ~/Projects/cli-ops/AGENTS.md Feature Registry section |
| Next work items | ~/Projects/cli-ops/sortie-board/SORTIE-BOARD.md |
| Active branches | ~/Projects/cli-ops/sortie-board/active-sorties.md |
| XLI null-space gaps | ~/Projects/cli-ops/docs/null-space/02-responses-to-messages.md |
| Features to port to Apex | ~/Projects/cli-ops/docs/feature-delta/03-cross-implementation.md |
| How ops work | ~/Projects/cli-ops/docs/03-OPERATIONAL-MODEL.md |
