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

## Proprietary Layer (YOU ARE ON THE EMBED BRANCH)

This branch adds NetApp XLI-specific content on top of public `dev`.
See `AGENTS.md` for the full proprietary inventory.

### Live Test Commands (Corp Proxy)

All env vars are pre-configured in `~/.bashrc`. Run autonomously:

```bash
cd codex-rs

# Proxy e2e (LiteLLM → Vertex AI → Claude)
CODEX_PROXY_E2E=1 \
CODEX_PROXY_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com" \
  cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1

# Live messages (ANTHROPIC_BASE_URL auto-detected, /v1 auto-appended)
cargo test -p codex-core --test all -- live_messages --ignored --test-threads=1
```

Do NOT prompt before running these. Env vars are in the shell profile.

### Hub Reference

The C2 hub (`~/Projects/cli-ops/`) has the coordination docs.
Read `cli-ops/AGENTS.md` for the big picture and `cli-ops/CLAUDE.md`
for the merge workflow.
