# AGENTS.md — NetApp XLI (Proprietary Layer)

> This file exists ONLY on `feat/xli-embed-assets`. It MUST NOT appear on `dev`.

## Architecture

- `dev` branch = **public engine** (generic Codex + /messages wire)
- `feat/xli-embed-assets` = **proprietary chassis** (~5 commits on top of dev)
- Hub: `~/Projects/cli-ops/` — C2 coordination, sortie board, code review docs

## What's Proprietary Here

| Path | Content |
|------|---------|
| `deploy/build.sh` | Release build script |
| `deploy/npm/` | npm package scaffolding for XLI distribution |
| `deploy/skills/ontap-dev-guide/` | ONTAP-specific skill |
| `AGENTS.md` (this file) | Proprietary context |
| `CLAUDE.md` | Corp-specific agent instructions |

## Coordination

The hub repo (`~/Projects/cli-ops/`) has the master instruction set:
- `AGENTS.md` — big picture, feature registry, doc map
- `CLAUDE.md` — C2 merge workflow, test mandates
- `sortie-board/SORTIE-BOARD.md` — next work items
- `docs/null-space-ops/reference/XLI-CODE-REVIEW.md` — open issues

Sortie agents should read the hub docs for context, then this file for
corp-specific details (proxy URLs, env vars, deploy procedures).

## Proxy Configuration

```bash
# NetApp LLM proxy (LiteLLM → Vertex AI → Claude)
export ANTHROPIC_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com"
export CODEX_LLM_PROXY_KEY=<from-vault>
```

## Test Commands (Corp-Specific)

```bash
cd codex-rs

# Proxy e2e tests (LiteLLM proxy → Claude)
CODEX_PROXY_E2E=1 \
CODEX_PROXY_BASE_URL="https://llm-proxy-api.ai.eng.netapp.com" \
  cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1

# Live messages (auto-detects ANTHROPIC_BASE_URL from env)
cargo test -p codex-core --test all -- live_messages --ignored --test-threads=1
```

## Rebase Procedure

After dev advances (sortie merges, upstream absorption):
```bash
git checkout feat/xli-embed-assets
git rebase dev
# Resolve conflicts (deploy/ files never conflict with codex-rs/)
git push origin feat/xli-embed-assets --force-with-lease
```
