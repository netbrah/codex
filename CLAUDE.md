# Codex CLI — Development Instructions

Fork of OpenAI Codex (Rust) with native Anthropic `/messages` wire protocol.
Primary branch: `dev`. Upstream: `upstream/main`.

## Build & Test

```bash
cd codex-rs
cargo check                                    # type check
cargo test -p codex-core -p codex-api          # unit + integration
cargo test -p codex-core --test all            # full integration suite
cargo run -p codex-core --bin codex-write-config-schema  # after config changes
```

## Test Mandate

Every code change MUST include tests:
- **Unit tests**: 3-5 per function changed (happy path, error, edge cases)
- **Regression tests**: every bug fix gets a test that fails-before, passes-after
- **Integration tests**: use CapturingTransport for request body verification
- **E2E tests**: update live test files when wire behavior changes

## Key Files

| Area | Files |
|------|-------|
| Messages wire translator | `core/src/messages_wire.rs` |
| Messages SSE parser | `codex-api/src/sse/messages.rs` |
| Messages HTTP client | `codex-api/src/endpoint/messages.rs` |
| Client dispatch | `core/src/client.rs` |
| Config | `core/src/config/mod.rs`, `core/src/config/types.rs` |
| WireApi enum | `core/src/model_provider_info.rs` |
| Regression guards | `core/src/messages_wire_regression_tests.rs` |

## Architecture

```
config (WireApi::Messages) → client.rs (stream_messages_api)
  → messages_wire.rs (history translator)
  → codex-api endpoint/messages.rs (HTTP client)
  → codex-api sse/messages.rs (SSE parser)
  → ResponseEvent (shared with responses wire)
```

## Proprietary Layer

Proprietary content (deploy scripts, corp-specific skills, internal URLs)
lives ONLY on `feat/xli-embed-assets`. NEVER on `dev`.
See the embed branch CLAUDE.md for corp-specific instructions.
