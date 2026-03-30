# XLI (codex-cli) — Sortie Agent Instructions

You are working on the XLI Rust harness — a fork of OpenAI's Codex CLI that
adds Anthropic `/messages` wire protocol support. Primary branch is `dev`.

## Test Mandate (HARD RULE)

Every sortie MUST include tests. No exceptions.

### Unit Tests
- **Every code change requires co-located unit tests** in the same file or
  `*_tests.rs` companion (pattern: `#[cfg(test)] mod tests { ... }`)
- **Every bug fix requires a regression test** that fails before the fix and
  passes after
- **Test the actual behavior**, not just compilation. Assert values, check
  error paths, verify edge cases
- Aim for **3-5 tests per function changed** covering: happy path, error path,
  edge cases, None/empty inputs

### Integration Tests
- When changing `codex-api/` code: add/update tests in `codex-api/tests/`
- Use the `CapturingTransport` pattern (see `messages_metadata.rs`) to verify
  request bodies without network calls
- Use `FixtureSseTransport` (see `messages_end_to_end.rs`) for SSE response
  testing

### E2E / Live Tests
- When changing `/messages` wire behavior: update `live_messages.rs` and/or
  `proxy_e2e_messages.rs`
- Live tests are `#[ignore]` — they run with env vars, not in CI
- **Run live tests before declaring a sortie complete** when wire behavior changed

### Running Live Tests

Env vars are pre-configured in `~/.bashrc` and `~/.zshrc`. Run:

```bash
# Unit + integration tests (always run)
cd codex-rs && cargo test -p codex-core -p codex-api

# Proxy e2e tests (run when /messages wire changed)
CODEX_PROXY_E2E=1 \
CODEX_PROXY_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com" \
  cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1

# Live messages tests (run when /messages wire changed)
# ANTHROPIC_BASE_URL from ~/.bashrc is auto-detected; /v1 suffix auto-appended
cargo test -p codex-core --test all -- live_messages --ignored --test-threads=1

# Live CLI tests (run when /responses wire changed)
cargo test --test live_cli -- --ignored
```

Do NOT prompt the user before running tests. Run them autonomously.

## Key Files

| Area | Files |
|------|-------|
| Messages wire translator | `core/src/messages_wire.rs` |
| Messages SSE parser | `codex-api/src/sse/messages.rs` |
| Messages HTTP client | `codex-api/src/endpoint/messages.rs` |
| Client dispatch | `core/src/client.rs` (stream_messages_api, build_responses_request) |
| Config (sampling, wire_api) | `core/src/config/mod.rs`, `core/src/config/types.rs` |
| Model provider info | `core/src/model_provider_info.rs` (WireApi enum) |
| Regression guards | `core/src/messages_wire_regression_tests.rs` |
| Live e2e tests | `core/tests/suite/live_messages.rs` |
| Proxy e2e tests | `exec/tests/proxy_e2e_messages.rs` |

## Code Review Reference

All open issues are tracked in:
`~/Projects/cli-ops/docs/null-space-ops/reference/XLI-CODE-REVIEW.md`

Read Part I (Issues 1-22) and Part III (Test Plan) before starting any sortie.

## Branch Naming

Sortie branches use **auto-generated codenames** (not `apex/sortie/` prefixes).
Examples: `seeder`, `persimmon`, `firewall`, `find`, `cent`, `fahrenheit`.
The superset launcher assigns names. Check all branches with `git branch`.

## Build & Test

```bash
cd codex-rs

# Build
cargo check           # fast type check
cargo build           # full build

# Test (scoped)
cargo test -p codex-core --lib        # core lib tests only
cargo test -p codex-api               # API crate tests
cargo test --test all                 # integration test suite

# Schema regeneration (after config changes)
cargo run -p codex-core --bin codex-write-config-schema
```
