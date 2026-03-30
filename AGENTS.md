# AGENTS.md — Codex CLI (XLI Rust Harness)

## What This Is

Fork of OpenAI Codex (Rust) with native Anthropic /messages wire protocol implementation.
This is the **Rust universal harness** — codename **XLI**.

## Architecture

- **Language**: Rust
- **Base**: `openai/codex` (upstream remote: `upstream`)
- **Novel work**: Anthropic /messages wire (`codex-rs/codex-api/src/sse/messages.rs`)
- **Primary branch**: `apex/messages-wire`
- **Upstream sync**: `main` tracks `upstream/main`

## Wire Protocol Status

| Wire | Status | Key Files |
|------|--------|-----------|
| OpenAI /responses | Native (upstream) | `codex-rs/codex-api/src/sse/responses.rs` |
| Anthropic /messages | Implemented (novel) | `codex-rs/codex-api/src/sse/messages.rs` |
| Gemini generateContent | Not started | — |

## Active Sortie Branches

These branches contain null-space closure implementations:
- `seeder`, `persimmon`, `firewall`, `find`, `cent`, `fahrenheit`

## Build

```bash
cd codex-rs
cargo build --release -p codex-cli

# Cross-compile for Linux
cargo zigbuild --release -p codex-cli --target x86_64-unknown-linux-musl
```

## Skills

Skills are baked into the binary at compile time via `include_dir!` macro.
Source: `codex-rs/skills/src/assets/samples/` (symlink-swapped during build).
At runtime, extracted to `~/.codex/skills/.system/`.

## Hub Reference

Coordination hub: `/Users/palanisd/Projects/cli-ops/`
Analysis docs, sortie board, and deployment configs live there.
