# Codex CLI (XLI) — Development Instructions

Fork of OpenAI Codex (Rust) with native Anthropic `/messages` wire protocol.
Primary branch: `dev`. Upstream: `upstream/main` (openai/codex).

Proprietary layer (`feat/embed-assets`) adds deploy scripts and corp
proxy config on top of this branch. Read that branch's CLAUDE.md for corp context.
This file covers dev-branch workflow only.

---

## Build & Test

```bash
cd codex-rs
cargo check                          # type check (fastest)
cargo test -p codex-core -p codex-api  # unit + integration tests
cargo test -p codex-core --test all    # full suite including regression guards
```

---

## Unit Tests

```bash
cd codex-rs

# All unit + integration
cargo test -p codex-core -p codex-api

# Single module (faster)
cargo test -p codex-core messages_wire
cargo test -p codex-core extract_developer_blocks
cargo test -p codex-core anthropic_thinking

# After config changes
cargo run -p codex-core --bin codex-write-config-schema
```

**Test mandate**: every change needs tests — unit tests, regression guards,
CapturingTransport for request body assertions. See `feat/embed-assets`
CLAUDE.md §Test Patterns for full skeletons.

---

## Live E2E Tests

The live e2e tests are **env-var gated**: `live_messages` auto-skips when
`CODEX_LLM_PROXY_KEY` or `CODEX_PROXY_BASE_URL` is not set. `proxy_e2e_messages`
requires `CODEX_PROXY_E2E=1` explicitly. No corp URLs are hardcoded on this branch.

### Set these in your shell profile to enable live e2e:

```bash
export CODEX_LLM_PROXY_KEY="<your-key>"          # or ANTHROPIC_API_KEY
export CODEX_PROXY_BASE_URL="<your-endpoint>"     # or ANTHROPIC_BASE_URL
```

For a corp proxy, set `CODEX_PROXY_BASE_URL` to the proxy URL
(documented in the proprietary branch's CLAUDE.md — not here).

### Run live e2e tests

```bash
cd codex-rs

# Live /messages wire smoke (auto-skips if env not set)
cargo test -p codex-core --test all -- live_messages --ignored --test-threads=1

# Full proxy e2e through LiteLLM stack (explicit gate required)
CODEX_PROXY_E2E=1 \
CODEX_PROXY_BASE_URL="<your-endpoint>" \
  cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1

# Live /responses wire (GPT models)
cargo test -p codex-core --test live_cli -- --ignored --test-threads=1
```

### What live e2e covers

| Test | Command | What it tests | Skip condition |
|------|---------|---------------|----------------|
| `live_messages` | `--test all -- live_messages --ignored` | Full /messages wire: spawn binary, prompt, SSE stream | `CODEX_LLM_PROXY_KEY` empty |
| `proxy_e2e_messages` | `--test proxy_e2e_messages` | LiteLLM → Claude: tool_use, thinking, streaming | `CODEX_PROXY_E2E` not set |
| `live_cli` | `--test live_cli -- --ignored` | /responses wire: GPT via proxy | `ANTHROPIC_API_KEY` empty |

### When to run e2e

| Change | Run e2e |
|--------|---------|
| `messages_wire.rs` | Yes — `live_messages` |
| `client.rs` stream_messages_api | Yes — `live_messages` |
| `codex-api/sse/messages.rs` | Yes — `live_messages` |
| `/responses` wire changes | Yes — `live_cli` |
| `compact.rs`, config | No — unit tests enough |
| Test-only changes | No |

---

## Architecture

```
config (WireApi::Messages) → client.rs (stream_messages_api, line ~1159)
  → messages_wire.rs (ResponseItem[] → Anthropic messages[])
  → codex-api/endpoint/messages.rs (HTTP, auth headers)
  → codex-api/sse/messages.rs (SSE: content_block_* → ResponseEvent)
```

### Key files

| Area | File | Detail |
|------|------|--------|
| Wire dispatch | `core/src/client.rs` | `stream()` routes by WireApi |
| History translator | `core/src/messages_wire.rs` | `extract_developer_blocks()` line 231 |
| HTTP client | `codex-api/src/endpoint/messages.rs` | auth headers, streaming |
| SSE parser | `codex-api/src/sse/messages.rs` | content_block events |
| Wire enum | `core/src/model_provider_info.rs` | WireApi::Responses / Messages |
| Compaction | `core/src/compact.rs` | 70/30 split, PRESERVE_RECENT_FRACTION |
| Thinking | `core/src/client.rs` line 2110 | `anthropic_thinking_param()` |
| Regression guards | `core/src/messages_wire_regression_tests.rs` | null-space field guards |
| Developer role | `core/src/client.rs` line 1197 | W-7/BREAK-1: system[] injection |
| Live messages e2e | `core/tests/suite/live_messages.rs` | S-013 smoke harness |

---

## Proprietary Layer

Proprietary content (corp proxy URL, deploy scripts, ONTAP skills) lives ONLY on
`feat/embed-assets`. Never commit corp-specific content to `dev`.

If you need the corp proxy URL, env var values, or full test instructions, read
`feat/embed-assets` CLAUDE.md.
