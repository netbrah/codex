# AGENTS.md — NetApp XLI (Proprietary Layer)

> This file exists ONLY on `feat/xli-embed-assets`. It MUST NOT appear on `dev`.
> `dev` = public engine. This branch = thin NetApp veneer (~5 commits on top of dev).

---

## Architecture

```
feat/xli-embed-assets  <- YOU ARE HERE (NetApp XLI distribution)
        ^ rebase
      dev               <- public engine (netbrah/codex, open-source)
        ^ absorbs
  upstream/main (openai/codex)
```

- **Lingua franca**: `ResponseItem[]` enum (Codex internal type)
- **Wire dispatch**: `WireApi::Responses` (native) or `WireApi::Messages` (our addition)
- **Hub**: `~/Projects/cli-ops/` — C2 coordination, sortie board, feature registry

### Wire Protocol Status

| Wire | Status | Implementation |
|------|--------|----------------|
| /responses | Native | openai/codex default |
| /messages | Shipped (CA-2..CA-8) | stream_messages_api() in client.rs |
| generateContent | Not started | — |

---

## What's Proprietary (This Branch Only)

| Path | Content |
|------|---------|
| `deploy/build.sh` | Release build script |
| `deploy/npm/` | npm package scaffolding for XLI distribution |
| `deploy/skills/ontap-dev-guide/` | ONTAP-specific skill |
| `AGENTS.md` (this file) | Proprietary context + proxy config |
| `CLAUDE.md` | Corp-specific agent instructions with proxy URLs |

---

## Proxy Configuration

```bash
# Set in ~/.bashrc — already configured on corp machines
export CODEX_LLM_PROXY_KEY="<from NetApp vault>"
export CODEX_PROXY_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com"

# Also works for direct Anthropic auth via proxy:
export ANTHROPIC_API_KEY="$CODEX_LLM_PROXY_KEY"
export ANTHROPIC_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com"
```

---

## Sortie Instructions — ALL AGENTS READ THIS

Read `CLAUDE.md` for the full test mandate, test patterns, e2e instructions, and sortie completion gate. This file has the architecture context; CLAUDE.md has the operational runbook.

### Rules
1. Work ONLY on your assigned sortie branch (`apex/sortie/...`)
2. Do NOT merge to dev or feat/xli-embed-assets — that is C2's job
3. Do NOT revert commits from other agents
4. Every commit must have a conventional prefix: `feat:`, `fix:`, `test:`, `refactor:`

### Quick Test Reference

```bash
cd codex-rs

# Unit tests (every change)
cargo test -p codex-core -p codex-api

# Live /messages e2e (wire-layer sorties)
cargo test -p codex-core --test all -- live_messages --ignored --test-threads=1

# Proxy e2e (full stack — LiteLLM -> Vertex AI -> Claude)
CODEX_PROXY_E2E=1 CODEX_PROXY_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com" \
  cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1
```

---

## Hub Reference

| What you need | Where to find it |
|---------------|-----------------|
| Big picture | `~/Projects/cli-ops/AGENTS.md` |
| Feature registry (CA-1..CA-9) | `~/Projects/cli-ops/AGENTS.md` Feature Registry |
| Next sorties | `~/Projects/cli-ops/sortie-board/SORTIE-BOARD.md` |
| Active branch status | `~/Projects/cli-ops/sortie-board/active-sorties.md` |
| XLI null-space gaps | `~/Projects/cli-ops/docs/null-space-ops/reference/` |
| Features to port to Apex | `~/Projects/cli-ops/docs/feature-delta/03-cross-implementation.md` |
| XLI feature delta | `~/Projects/cli-ops/docs/feature-delta/02-codex-harness-additions.md` |
| How ops work | `~/Projects/cli-ops/docs/03-OPERATIONAL-MODEL.md` |

---

## Rebase Procedure

```bash
git checkout feat/xli-embed-assets
git fetch origin
git rebase dev
# deploy/ files never conflict with codex-rs/ changes
git push origin feat/xli-embed-assets --force-with-lease
```
