# AGENTS.md — Codex CLI (XLI Rust Harness)

## What This Is

Fork of OpenAI Codex (Rust) with native Anthropic /messages wire protocol implementation.
This is the **Rust universal harness** — codename **XLI**.

## Why It Exists

Traditional CLI agents are monolithic: one harness, one wire, one model. XLI breaks that by cross-wiring the Codex Rust harness to speak multiple wire protocols. Today it speaks /responses (native from upstream OpenAI) and /messages (our novel 3,459-line addition). This means the same Rust binary can drive both GPT and Claude — and eventually Gemini when generateContent is added.

The `ResponseItem[]` enum is the **lingua franca**: all wire protocols translate to and from it at a single boundary. This keeps the harness core (tools, sandboxing, multi-agent, skills, guardian) wire-agnostic.

**Two tiers**:
- `dev` branch = PUBLIC tier (netbrah GitHub). All cross-wire implementations. Open-source and upstreamable.
- `feat/xli-embed-assets` = PROPRIETARY tier (NetApp XLI). A thin veneer on top of dev: deploy scripts, skills, build config. Maximize dev, minimize this layer. NEVER push to public remotes.

## Mission

Implement all three wire protocols natively in the Codex Rust harness with minimal null space. Cross-pollinate features from the TS harness (Apex/qwen-code). Distribute as NetApp XLI via Artifactory.

## Architecture

- **Language**: Rust (workspace of crates under `codex-rs/`)
- **Internal format**: `ResponseItem` enum — the lingua franca. Single conversion boundary at each wire.
- **Primary branch**: `dev` (public), `feat/xli-embed-assets` (proprietary)
- **Upstream**: openai/codex (remote: `upstream`, branch: `main`)

### Wire Protocol Status

| Wire | Status | Key Files |
|------|--------|-----------|
| OpenAI /responses | Native (upstream) | `codex-rs/codex-api/src/sse/responses.rs` |
| Anthropic /messages | Implemented (novel, 3459 lines added) | `messages_wire.rs`, `sse/messages.rs`, `endpoint/messages.rs` |
| Gemini generateContent | Not started | — |

### Key Source Files

| File | Role | Lines |
|------|------|-------|
| `codex-rs/core/src/messages_wire.rs` | Conversation translator: ResponseItem[] <-> Anthropic JSON | 752 |
| `codex-rs/codex-api/src/sse/messages.rs` | SSE parser with BlockTracker state machine | ~500 |
| `codex-rs/codex-api/src/endpoint/messages.rs` | MessagesClient HTTP client | ~80 |
| `codex-rs/core/src/client.rs` | ModelClient — request construction for both wires | ~1200 |
| `codex-rs/core/src/config/mod.rs` | Config with sampling, tool_choice, metadata | ~300 |
| `codex-rs/protocol/src/config_types.rs` | ToolChoice enum, SamplingParams struct | ~200 |

### Novel Additions Beyond /messages Wire

| Feature | File | Description |
|---------|------|-------------|
| WireApi enum | `model_provider_info.rs` | `Responses` / `Messages` config switch |
| Claude model registry | `client.rs` | Model-specific output caps (128K opus, 64K sonnet) |
| 70/30 compaction split | `compact.rs` | Preserves 70% recent, summarizes 30% oldest |
| XML compaction prompt | `compact.rs` | Claude-optimized summarization |
| Cache control | `messages_wire.rs` | Automatic ephemeral cache on last tool + system |
| Thinking/reasoning | `sse/messages.rs` | Full thinking + signature + redacted thinking |

## Known Null Space (Critical Gaps)

| Gap | Severity | Detail |
|-----|----------|--------|
| Developer role dropped | P0 | `messages_wire.rs:24` — `"developer" => continue`. Claude never sees AGENTS.md, permissions, personality. |
| stop_reason not parsed | P0 | `sse/messages.rs` — stop_reason from message_delta discarded. Cannot detect max_tokens truncation. |
| tool_choice hardcoded | P1 | Always None (server auto). Cannot force/suppress tool use. Sortie W-2 ready. |
| Sampling params None | P1 | temperature, top_p, top_k all hardcoded None. Sortie W-3 ready. |
| stop_sequences missing | P1 | Not on MessagesApiRequest struct. Sortie W-4 ready. |
| metadata.user_id missing | P2 | No per-user attribution. Sortie W-5 ready. |
| cache_creation_tokens dead | P2 | Parsed but #[allow(dead_code)]. Sortie W-6 ready. |

## Sortie Branches (In-Flight Work)

Active sortie implementations (not the old W-2..W-6 branches which were deleted):

| Branch | Feature | Status |
|--------|---------|--------|
| `seeder` | Sortie implementation | Active |
| `persimmon` | Sortie implementation | Active |
| `firewall` | Sortie implementation | Active |
| `find` | Sortie implementation | Active |
| `cent` | Sortie implementation | Active |
| `fahrenheit` | Sortie implementation | Active |

## Sortie Agent Instructions

If you are on a `sortie/` or feature branch, you are a sortie agent:
- Work ONLY on your assigned branch
- Commit with conventional commit messages
- Do NOT merge into `dev` — that is C2's job
- Do NOT switch branches
- Do NOT revert commits from other agents
- Run `cargo test -p codex-api` before final commit

### Sortie Completion Gate

Your FINAL COMMIT message or a `SORTIE-NOTES.md` in the repo root must include:

```
## Sortie Completion Notes
- Unit tests: [PASS/FAIL — cargo test -p codex-api / cargo test -p codex-protocol]
- Live e2e: [PASS/FAIL/SKIPPED — see below]
- Wire behavior changed: [yes/no — if yes, what field/event]
- New feature: [yes/no — if yes, suggest registry ID: CA-{N}, tier: public]
- Null-space gap closed: [yes/no — if yes, which gap, new remaining count]
- Cross-pollination: [yes/no — does the TS harness (Apex) need this?]
```

### Live E2E Testing (required for wire-layer sorties)

If your sortie changes wire behavior (/messages translation, SSE parsing, request construction), you MUST run the live e2e test before marking complete:

```bash
# /responses wire (native — upstream tests)
OPENAI_API_KEY=$OPENAI_API_KEY cargo test --test live_cli -- --ignored

# /messages wire — NO LIVE TEST EXISTS YET
# Instead, do a manual evidence ledger test:
# Send a real request through the proxy and record req/resp in your SORTIE-NOTES.md
# C2 will add it to the evidence ledger at merge time
```

**Known gap**: There is no automated live e2e test for the /messages wire (the novel wire this harness adds). The `live_cli.rs` only tests /responses. Until a /messages live test is created, wire-layer sortie agents must manually validate through the proxy and document the result.

C2 reads these notes at merge time and executes the doc updates (evidence ledger, feature registry, wire audit, KPIs). You focus on code; C2 handles the docs.

### Convergence (idempotent re-dispatch)

If you are re-dispatched on a branch that already has commits (e.g., previous agent crashed):
1. Read existing commits: `git log --oneline`
2. Check if `SORTIE-NOTES.md` exists and is partially filled → complete the missing fields
3. Check if tests were already run → verify results rather than re-running
4. Continue from where the previous agent left off — don't redo completed work

## Build & Test

```bash
cd codex-rs
cargo build --release -p codex-cli
cargo test -p codex-api        # messages + responses tests
cargo test -p codex-protocol   # config type tests
# codex-core tests need v8 — run outside sandbox

# Cross-compile for Linux
cargo zigbuild --release -p codex-cli --target x86_64-unknown-linux-musl
```

## Skills

Skills are baked into the binary at compile time via `include_dir!` macro.
Source: `codex-rs/skills/src/assets/samples/` (symlink-swapped during build).
At runtime, extracted to `~/.codex/skills/.system/`.

## Hub Reference

Coordination hub: `~/Projects/cli-ops/` — read `AGENTS.md` there for the full big picture, feature registry (public vs proprietary tracking), and four-dimension overview.

| What you need | Where to find it |
|---------------|-----------------|
| What to work on next | `~/Projects/cli-ops/sortie-board/SORTIE-BOARD.md` |
| What's in flight | `~/Projects/cli-ops/sortie-board/active-sorties.md` |
| XLI wire gaps | `~/Projects/cli-ops/docs/null-space-ops/MASTER.md` |
| Field-by-field audit | `~/Projects/cli-ops/docs/null-space-ops/reference/wire-audit-xli.md` |
| Roundtrip audit | `~/Projects/cli-ops/docs/null-space-ops/reference/wire-roundtrip-xli.md` |
| Features to port from Apex | `~/Projects/cli-ops/docs/feature-delta/03-cross-implementation.md` |
| How ops work (sortie lifecycle) | `~/Projects/cli-ops/docs/03-OPERATIONAL-MODEL.md` |
| KPIs and objectives | `~/Projects/cli-ops/docs/04-OBJECTIVES-AND-KPIs.md` |
| Build/deploy/merge procedures | `~/Projects/cli-ops/docs/05-RUNBOOK.md` (RB-3: sortie cycle, RB-5: XLI build) |

## Features to Port from Apex (TS)

| Feature | Source | Priority |
|---------|--------|----------|
| StreamingToolCallParser (truncation detect) | `openaiContentGenerator/streamingToolCallParser.ts` | P0 |
| cleanOrphanedToolCalls() | `openaiContentGenerator/converter.ts` | P1 |
| Modality gating with text placeholders | `openaiContentGenerator/converter.ts` | P2 |
| SchemaComplianceMode | `openaiContentGenerator/converter.ts` | P2 |
| Arena mode | `ArenaManager` | P2 |
