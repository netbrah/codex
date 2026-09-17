# Responses-Compat Seam — Divergence Map and Design

This document defines the boundary ("seam") between this fork of codex and
non-OpenAI deployments of the OpenAI Responses wire, records every change that
diverges from `openai/codex`, and specifies how new compatibility work is
designed, applied, and re-verified after upstream rebases.

## 1. The three boundaries

```
┌────────────────────────┐    ┌────────────────────────┐    ┌─────────────────────────────┐
│ HARNESS (this code)    │    │ RESPONSES WIRE         │    │ DEPLOYMENT                  │
│                        │    │ (HTTP /v1/responses)   │    │                             │
│ • turn loop, tool      │───▶│ • request: model,      │───▶│ A. raw OpenAI / Azure       │
│   registry, sandbox,   │    │   input, tools,        │    │   full spec: custom tools,  │
│   compaction logic,    │◀───│   headers              │◀───│   grammar constraints,      │
│   multi-agent runtime  │    │ • response: SSE items  │    │   server-side compaction    │
│ • ProviderCapabilities │    │                        │    │ B. LiteLLM proxy            │
│   (the seam knob)      │    │                        │    │   pass-through w/ quirks:   │
└────────────────────────┘    └────────────────────────┘    │   strips x-codex-* headers, │
                                                            │   route allowlist           │
                                                            │ C. vLLM responses-compat    │
                                                            │   function tools only: no   │
                                                            │   custom tools, no compact  │
                                                            └─────────────────────────────┘
```

1. **Harness ↔ model**: per-model behavior (prompts, tool mode, multi-agent
   version, apply_patch flavor) is driven by the model catalog
   (`models-manager/models.json`, embedded at compile time) plus
   `model_info_from_slug` fallback metadata for unknown slugs.
2. **Harness ↔ wire**: the request/response shape codex emits. This fork
   normalizes two wire details for non-OpenAI providers (content part types,
   namespaced tool shapes) and, per the design below, one tool flavor
   (apply_patch freeform → function).
3. **Wire ↔ deployment**: what the serving stack actually implements.
   Empirically verified against the corporate llm-proxy (LiteLLM) on
   2026-09-13 — see §5.

## 2. Provider capabilities — the seam knob

`model-provider/src/provider.rs` computes `ProviderCapabilities` per
configured provider. These are the ONLY supported divergence points; new
compatibility work must land here, not in scattered `if !is_openai()`
branches.

| Capability | Gate | Effect | Status |
|---|---|---|---|
| `normalize_content_types` | `!is_openai()` | Rewrites `input_text`/`output_text` content parts to `text` before encoding (vLLM/SGLang accept `text`) | shipped (commit `b4d4b12`) |
| `flatten_namespace_tools` | `!is_openai()` | Expands `namespace` tool specs into individual function tools with dotted names; router splits dotted names back | shipped (commit `197ea16`) |
| `apply_patch_function_tool` | `!is_openai()` | Emits `apply_patch` as a JSON-schema **function tool** instead of a **custom (freeform + lark grammar) tool** | IMPLEMENTED (items 1-3, 2026-09-16) |

Gate semantics: `is_openai()` is an exact, case-sensitive match of the
provider `name` field in `config.toml` against `OPENAI_PROVIDER_NAME`
— `model-provider-info/src/lib.rs:546-547` (constant
`OPENAI_PROVIDER_NAME: &str = "OpenAI"` at `:40`). There is **no Azure
branch** (`docs/responses-compat-apply-patch-format.md` §3.4 invariant):
any Azure-*named* provider fails the gate and is in seam scope
(function tool + P1 format text). The gate means "this provider's
*deployment* implements the full OpenAI responses spec", not "the model
is made by OpenAI".

## 3. Design: apply_patch function tool for non-OpenAI providers

### Problem (empirically established 2026-09-13)

`apply_patch` for non-catalog models uses `ToolSpec::Freeform` — a
`type: "custom"` Responses tool carrying a Lark grammar. Grammar-constrained
custom tools are an **OpenAI server-side feature**. Verified against the
corporate llm-proxy + on-prem qwen (vLLM): the deployment accepts the custom
tool definition but never emits a `custom_tool_call`; the model writes the
patch as a plain text message. The harness therefore can never execute the
patch — the tool is dead weight for such deployments.

### Decision

For providers with `apply_patch_function_tool`, emit `apply_patch` as a
standard **function tool** with a single required `patch` string argument
(the full `*** Begin Patch ... *** End Patch` text). Function calls are the
one tool mechanism every responses-compat deployment implements (it is what
all other codex tools use). Providers named `OpenAI` are untouched: they
keep the native custom+grammar path (spec §3.4 invariant: outbound
request bytes for `OpenAI`-named providers are unchanged; `is_openai()`
is name-anchored, so Azure-*named* providers take the function tool, as
pinned by the capability test).

### Components

1. **Capability** — `apply_patch_function_tool: bool` added to
   `ProviderCapabilities` (`model-provider/src/provider.rs`), computed as
   `!is_openai()`. Mirrors the two existing compat capabilities.

2. **Spec** — `create_apply_patch_function_tool(include_environment_id: bool)
   -> ToolSpec` in `core/src/tools/handlers/apply_patch_spec.rs`:
   `ToolSpec::Function(ResponsesApiTool)` with:
   - `name: "apply_patch"`, `strict: false`
   - `description`: same edit-files guidance as the freeform spec, plus
     the placement rule ("The complete patch goes in the `patch`
     argument."); the `patch` argument description (P1, spec §3.1)
     carries "The ENTIRE patch as a single string" and the anti-wrapping
     rule (no markdown fences, code-block markers, or shell heredoc
     wrapper)
   - `parameters` (JsonSchema object):
     - `patch: string` (required) — the complete patch text
     - `environment_id: string` (optional) — only when
       `include_environment_id` is true

3. **Handler** — `FunctionApplyPatchHandler` in
   `core/src/tools/handlers/apply_patch.rs`:
   - `tool_name()` → `ToolName::plain("apply_patch")`
   - `spec()` → the function spec above
   - `handle_call` → parses `ToolPayload::Function { arguments }` as JSON,
     extracts `patch` (required) and `environment_id` (optional); missing
     `patch` or invalid JSON → `FunctionCallError::RespondToModel`
   - Delegates to the SAME shared execution path as the custom handler
     (extract the post-parse body of the existing `handle_call` into a
     private `run_apply_patch_text(...)` used by both handlers — no logic
     duplication, so future upstream fixes to patch execution apply to both)
   - `CoreToolRuntime::matches_kind` → `ToolPayload::Function { .. }`
   - Hooks: `pre/post_tool_use_payload` emit the same
     `{"command": <patch text>}` shape as the custom path so existing
     apply_patch hooks keep working; `with_updated_hook_input` writes the
     updated patch back into the Function arguments JSON
   - `create_diff_consumer` → `None` in v1 (streaming progress events for
     in-JSON-string deltas are a documented follow-up)

4. **Registration** — `core/src/tools/spec_plan.rs`, at the existing
   `environment_mode.has_environment() &&
   context.model_info.apply_patch_tool_type.is_some()` site: select
   `FunctionApplyPatchHandler` when the provider capability is set, else
   `ApplyPatchHandler`. Exactly one is registered per session.

### Format-remediation layer (P1/P2/P3)

The first-shipped function-tool spec text taught **no** patch format
("the complete patch goes in the `patch` argument as plain text"), so
format correctness depended purely on model priors. Measured on the live
vLLM/LiteLLM deployment, 2026-09-13..15 (the spec §1.2 pinned snapshot;
source: `docs/responses-compat-apply-patch-format.md` §1.2):

| Model | unique apply_patch calls | parse rejections | rate |
|---|---|---|---|
| glm-5.2 (vLLM-served) | 37 | 26 | **~70%** (70.27%), as of the pinned snapshot |
| qwen3.8-27b | 413 | 5 | **~1%** (1.21%) |

glm-5.2 on 09-15 alone: 13 of 21 calls rejected ≈ 62%. The rejections
fell in two format classes — raw unprefixed content lines (F1/F2) and
missing `*** Begin Patch`/`*** End Patch` boundary markers (F4) —
exactly what P1 teaches; P2/P3 are the defense-in-depth layer below it.
The fix landed 2026-09-16 in the same PR set (commit SHAs recorded in
the spec's status line):

- **P1** — the full patch format is taught in the function-tool `patch`
  argument description (non-OpenAI path only; `apply_patch_spec.rs`).
- **P2** — Add-File leniency in the streaming parser: raw/empty/
  whitespace content lines accepted verbatim (defense in depth;
  provider-agnostic; `streaming_parser.rs`).
- **P3** — teachable parse errors (bad hunk headers, Delete-File
  content, absent/non-string `patch`), including the `parse_patch`
  pre-pass boundary strings (P3.4; `streaming_parser.rs`, `parser.rs`),
  plus the P3.3 handler error split (`apply_patch.rs`).

The exact wording and invariant: spec §3.1-§3.4; the rebase invariant
is recorded under §6 of this doc.

### Testing

- Unit: function spec shape (with/without `environment_id`; required list)
- Unit: arguments extraction (valid, missing `patch`, invalid JSON,
  `environment_id` passthrough)
- Unit: capability gate (`!is_openai()` → true; OpenAI → false)
- Smoke against the REAL deployment (Principle 2, no substitute):
  wiretap capture shows `function` apply_patch; a qwen session creates a
  file via apply_patch; file content verified on disk.

### Out of scope (documented follow-ups)

- Streaming patch progress for the function flavor
- Server-side compaction for proxied OpenAI models (needs proxy-side route
  unblock + codex unary-compact path; separate work item)
- v2 multi-agent on non-catalog models (catalog `multi_agent_version` or
  `features.multi-agent-v2`; independent of this seam)

## 4. Divergence map (this fork vs openai/codex)

Code divergence, smallest rebase blast radius first:

| # | Change | Files | Purpose | Rebase risk |
|---|---|---|---|---|
| 1 | `normalize_content_types` capability + `content_type_compat` module | `codex-api/src/endpoint/{mod,responses}.rs`, `client.rs`, `provider.rs` | vLLM/SGLang want `text` parts, not `input_text`/`output_text` | medium — `responses.rs`/`client.rs` churn upstream |
| 2 | `flatten_namespace_tools` capability + `flatten_namespace_specs` | `tools/src/tool_spec.rs`, `protocol/src/tool_name.rs`, `core/src/tools/router.rs`, `client.rs` | vLLM does not implement `namespace` tool types; dotted names split back on tool-call return | medium — `client.rs` tool-construction block |
| 3 | `apply_patch_function_tool` capability + `FunctionApplyPatchHandler` | `provider.rs`, `apply_patch_spec.rs`, `apply_patch.rs`, `spec_plan.rs` | vLLM does not implement grammar-constrained custom tools; function form is universal | low-medium — new files + one registration site |
| 4 | Fallback model metadata: `apply_patch_tool_type = Some(Freeform)` for unknown slugs | `models-manager/src/model_info.rs` | non-catalog models (e.g. on-prem qwen) get a working apply_patch instead of none | low — one line, but semantically significant |
| 5 | `base_instructions` / personality `instructions_variables` on gpt-5.x catalog entries | `models-manager/models.json` | prompt content for GPT models (GPT-only; unrelated to the seam) | high — models.json churns constantly upstream |
| 6 | `AGENTS.md` working principles | repo root | process doc for agents operating in this repo | low |
| 7 | apply-patch parser change: P2 Add-File leniency + P3/P3.4 teachable errors — **provider-agnostic**; surface = function-tool path + freeform diff consumer + `apply_patch` CLI (P2), plus the `parse_patch` pre-pass surface for P3/P3.4 (function path + CLI + shell-intercept) | `apply-patch/src/streaming_parser.rs`, `apply-patch/src/parser.rs`, `apply_patch.rs` (P3.3 handler errors) | glm-5.2 (vLLM-served) rejected ~70% of apply_patch calls with parse errors (spec §1.2 pinned snapshot); P2/P3 affect only inputs upstream rejects (spec §3.4) | low-medium — re-apply on upstream restructure of `streaming_parser.rs` or `parser.rs`; the diff consumer's parallel streaming boundary messages intentionally unchanged (spec §3.4 divergence note) |

Operational (out-of-repo) seams discovered 2026-09-13, for reference:

- **llm-proxy route allowlist** (`cli-ops/upstream-infrastructure/llm-proxy`,
  `app/common/routes.py`): `ALLOWED_ROUTES` blocks `POST /responses/compact`
  (403 "Route is blocked"). Unblocking requires a proxy deploy.
- **LiteLLM header stripping**: `x-codex-turn-metadata` (carries
  `request_kind: "compaction"`) does not reach the upstream provider, so
  OpenAI server-side compaction is unreachable through the proxy.
- **vLLM custom-tool gap**: the on-prem qwen deployment accepts custom tool
  definitions but never emits `custom_tool_call` — this is what change #3
  works around.

## 5. Empirical evidence (2026-09-13, corporate llm-proxy)

| Probe | Method | Result |
|---|---|---|
| v2 header compaction | `POST /v1/responses` + `x-codex-turn-metadata` (`request_kind: compaction`), model gpt-5.6-sol | header dropped by LiteLLM; normal assistant reply returned |
| v1 compact endpoint | `POST /v1/responses/compact` (± litellm tags, ± /v1) | 403 "Route is blocked" (allowlist) |
| qwen custom tool call | `POST /v1/responses`, tools=[custom apply_patch + lark grammar], model qwen3.8-27b | status completed; model emits the patch as a plain `message`, never a `custom_tool_call` |
| qwen function tool call | all existing codex tools (function/namespaced) | works — the only universally supported tool mechanism |
| local (model-self) compaction on qwen | forced `model_auto_compact_token_limit=2500` session | compaction fires, session resumes from summary, task completes correctly |

Consequence: for non-OpenAI providers the harness must not depend on
server-side compaction, custom tools, or any OpenAI-only request field.
`ProviderCapabilities` is the single gate for all of it.

## 6. Rebase/maintenance workflow

How to pull upstream without losing the seam (proven 2026-09-13, 1011
commits in one step):

1. `git stash push` (tracked changes only; untracked tooling like
   `.codegraph/` can stay), then `git rebase origin/main`.
2. Expected conflict sites, in order of likelihood:
   - `codex-api/src/endpoint/{mod,responses}.rs` — upstream endpoint
     restructures (the old `compact` endpoint module was deleted upstream in
     #44273). Keep our `content_type_compat` module line and the
     `normalize_content_types` conditional.
   - `core/src/client.rs` — the tool-construction block
     (`(instructions, tools)`). Re-apply the `tools_ref` flattening onto
     upstream's version of the block; keep upstream's new logic.
   - `models-manager/models.json` — catalog churn; resolve by keeping both
     upstream's entries and our additions (they are usually disjoint per
     model entry).
   - `model-provider/src/provider.rs` — capability computation block.
   - `codex-rs/apply-patch/src/streaming_parser.rs` — P2 Add-File arm +
     P3.1/P3.2 error-message sites (the format-remediation layer, §3).
   - `codex-rs/apply-patch/src/parser.rs` — P3.4 `parse_patch` pre-pass
     boundary strings (the Begin/End patch error messages).
3. After rebase: `git stash pop`, then **the verification gate** (all must
   pass before a release build):
   - `cargo check -p codex-cli`
   - `cargo test -p codex-models-manager --lib` (fallback metadata)
   - `cargo test -p codex-core apply_patch` (spec + handler)
   - `cargo test -p codex-model-provider --lib` (capability gates)
   - `just test -p codex-apply-patch` (P2/P3 parser behavior — a rebase
     that silently reverts P2 fails here)
   - Raw-format probe per `docs/responses-compat-apply-patch-format.md`
     §5.4: the standalone `apply_patch` binary from the same release
     build, scratch dir, constructed raw Add-File patch in the F1 shape
     (no `+` prefixes, `# …` first content line) — pass = exit 0,
     contents byte-identical to the raw content
4. Release build: `cargo build --release -p codex-cli` (≈10 min warm).
5. Wire verification against the real deployment (never a substitute):
   - `python3 ~/bin/wiretap.py 9098` then run a one-turn qwen session with
     `-c model_providers.llm_proxy_qwen.base_url="http://127.0.0.1:9098/v1"`;
     confirm `apply_patch` appears as a **function** tool with a `patch`
     parameter and the vLLM shims are active (dotted MCP/multi-agent names,
     `text` content parts).
   - Functional: a qwen session must create a file via `apply_patch`
     (function call), verified on disk.
   - Compaction regression: forced low `model_auto_compact_token_limit`
     session completes correctly.
6. Keep the running production binary untouched until the new one passes
   5; install under a new name, then cut over; previous binary stays as
   rollback (see `Projects/upstream/codex-bin-backups/`).

Invariants that must survive every rebase:

- Outbound request bytes for providers named `OpenAI` are
  byte-identical to upstream (all capability flags false on
  `is_openai()`, which is name-anchored on `OpenAI`;
  Azure-*named* providers take the function tool per spec §3.4).
- The apply-patch format-remediation layer (P1/P2/P3) holds the spec's
  exact invariant (`docs/responses-compat-apply-patch-format.md` §3.4):
  outbound **request bytes for providers named `OpenAI` are unchanged**
  (P1 touches only the function-tool spec, which never ships to them).
  P2 and P3 are **provider-agnostic parser changes**: for every input
  upstream accepts, behavior is byte-identical; for inputs upstream
  *rejects*, P2 turns some Add-File rejections into accepted content
  and P3 turns some rejections into teachable errors — full surface:
  the function-tool execution path, the freeform diff consumer, and the
  `apply_patch` CLI. P3.4 applies to the `parse_patch` pre-pass surface
  only (function path + CLI + shell-intercept); it does **not** touch
  the diff consumer's parallel streaming boundary messages.
- The only wire differences for non-OpenAI providers are the three
  capabilities in §2 — nothing else may branch on `is_openai()`.
- New provider capabilities land in `ProviderCapabilities` and are covered
  by a unit test in `model-provider/src/provider.rs` (cases: OpenAI, Azure,
  custom).
