# Codex-Combined Fork — Divergence Manifest (pull ratchet)

> **Purpose:** Track every fork-specific file and modification that must survive
> upstream merges (openai/codex main → this branch). After every ratchet, verify
> against this manifest and update "Last verified".
>
> **Last verified:** 2026-09-19 (UTC) — base `f3da3861c5`
> (`reference/upstream-openai`) unchanged since the 2026-09-16 upstream
> merge @ `7bcd344fa7` (ratchet verified 2026-09-17; that audit block
> below stands, except the provider.rs and tools/src/lib.rs notes, both
> rewritten at this re-derivation); manifest re-derived at fork
> HEAD `10d14858c7` (apex-xt2.9 item 2). Since 2026-09-17: xt2.7 strict_int
> helpers, xt2.9 item 1 on-prem catalogue (qwen3.8-27b + v2 flips), xt2.11
> A/B apply-patch shape-repair, xt2.9 item 2 Proactive-default seam.
> Both remotes (netbrah/codex + APEX/codex) in sync at `10d14858c7`; SCS
> ff-synced (the item-3 commit re-pushes both remotes and re-syncs SCS, so
> this holds after landing).
> All counts below = first-hand `git diff f3da3861c5..10d14858c7
> --name-status` (A=57, M=35).

## Ratchet audit 2026-09-17 (pre-merge, entity-level via weave)

Dry-run merge `origin/main` → `feat/normalize-content-types-vllm` (193 upstream
commits): **9 of 10 shared files clean** (high/very_high confidence), **0
dangling references, 0 renames**, 1 flagged entity.

| file | verdict | note |
|---|---|---|
| `codex-api/src/endpoint/mod.rs` | clean very_high | our `pub mod content_type_compat;` line vs upstream -1 line |
| `codex-api/src/endpoint/responses.rs` | clean high | upstream DELETED `ResponsesEndpoint` enum+impl (Guardian now routes via `/responses` with headers, eeded5ba1a); **nothing in merged output references it** — no DANGLING. Our `normalize_content_types` flag + body-encode branch auto-merge |
| `codex-api/tests/clients.rs` | clean very_high | — |
| `core/src/client.rs` | clean high, **2 advisories** | upstream big refactor (~450L: `ClientRouting` enum, `WebsocketContinuation` added; `impl ModelClient` + `impl ModelClientSession` edited by both sides on different siblings) — confirm coexistence at compile + `just test -p codex-core` post-merge |
| `core/src/tools/spec_plan.rs` | clean high | upstream reorganized 16 fns; our import + apply_patch tool-type hunks auto-merge |
| `core/tests/common/responses.rs` | clean very_high | — |
| `model-provider/src/provider.rs` | clean very_high | upstream deleted `enforce_managed_residency` (unreferenced in output); our `!is_openai()` capability flags auto-merge (note corrected 2026-09-19: the `llm_proxy*` providers never lived in this repo — they are operator profile TOMLs) |
| `models-manager/src/model_info.rs` | clean very_high | upstream personality refactor (`with_config_overrides`, `model_info_from_slug`, `local_model_messages` edited); `model_info_from_slug` still exists — our new test compiles |
| `models-manager/src/model_info_tests.rs` | **1 CONFLICT (benign)** | flagged fn `baked_personality_section_is_preserved_without_enabled_explicit_none` "both modified" — but WE never touched it: upstream renamed it (dropped `personality_enabled`) + switched to `codex_prompts::render_model_instructions`. **Resolution: take upstream version of that fn; keep our two additive changes** (the `ApplyPatchToolType` import + `fallback_model_info_enables_apply_patch_for_unknown_models` test at EOF) |
| `tools/src/lib.rs` | clean very_high | upstream added `output_schema` module; our `strict_int` module + 8 re-exports (xt2.7) and the `flatten_namespace_specs` re-export (ayl.52) auto-merge |

Upstream 193-commit themes touching our seams: `personality_enabled` config
removal + `render_model_instructions` API (models-manager, codex_prompts
crate); Guardian endpoint consolidation (codex-api); client.rs transport
refactor; spec_plan reorganization; tools crate `output_schema`.

## Fork-only files (56 added; no upstream equivalent — merge risk: none)

Count = `git diff f3da3861c5..10d14858c7 --name-status` minus this manifest
(self-reference). `docs/onprem-mode-catalog-spec.md` (the apex-xt2.9 campaign
record) lands in the item-3 commit alongside this re-derivation and is not in
this count.

### Code (9) — MUST SURVIVE

| file | purpose |
|---|---|
| `codex-rs/codex-api/src/endpoint/content_type_compat.rs` | seam core: content-type normalization + agent-message translation for non-OpenAI providers (apex-ayl.52) |
| `codex-rs/codex-api/src/endpoint/content_type_compat_tests.rs` | tests for above |
| `codex-rs/apply-patch/src/streaming_parser_p2_tests.rs` | apply-patch P2 lenient-parser tests |
| `codex-rs/apply-patch/src/parser_shape_retry_tests.rs` | apply-patch strict-first shape-repair pre-pass tests (apex-xt2.11 item A) |
| `codex-rs/protocol/src/tool_name_tests.rs` | `ToolName::from_response_fields` (flattened dotted-name namespace recovery) tests |
| `codex-rs/core/src/fork_defaults.rs` | on-prem Proactive delegation default seam (apex-xt2.9 item 2) |
| `codex-rs/core/src/session/multi_agent_mode_selection_tests.rs` | selection tests for the fork default (apex-xt2.9 item 2) |
| `codex-rs/tools/src/strict_int.rs` | strict numeric coercion helpers for tool args (apex-xt2.7) |
| `codex-rs/tools/src/strict_int_tests.rs` | tests for above |

### Docs (47) — KEEP (campaign record; zero merge risk)
- `docs/responses-compat-seam.md` — seam SoT incl. §6 conflict-site map
  (the authoritative reference this manifest complements)
- `docs/responses-compat-apply-patch-task-breakdown.md`,
  `docs/responses-compat-apply-patch-format.md` — campaign plan + format SoT
- `docs/reviews/apply-patch-format-spec-*.md` (15),
  `docs/reviews/apply-patch-task-breakdown-*.md` (6),
  `docs/reviews/impl-item*.md` (21) — multi-agent review audit trail
- `docs/vllm-glm-toolcall-research.md` — vLLM source research (prior seat)
- `docs/tool-args-type-error-census.md` — type-error census + fix plan (apex-xt2.2)

## Fork modifications to shared files (35)

### 10 also modified upstream in this window — per-file strategy (table above)

### 25 we modified, upstream UNTOUCHED in the merged 2026-09-17 window (re-derive the split at the next ratchet)

| file | what we changed | class |
|---|---|---|
| `codex-rs/apply-patch/src/lib.rs` | `ApplyPatchArgs.repair_note`: shape-repair pre-pass note surfaced to the model (apex-xt2.11 item A) | MUST SURVIVE |
| `codex-rs/apply-patch/src/parser.rs` | P2 lenient AddFile state (accepts unprefixed content lines) + strict-first shape-repair pre-pass for omitted Begin/End boundaries (apex-xt2.11 item A) | MUST SURVIVE |
| `codex-rs/apply-patch/src/streaming_parser.rs` | streaming parse support for P2 | MUST SURVIVE |
| `codex-rs/apply-patch/tests/suite/tool.rs` | P2 tests | MUST SURVIVE |
| `codex-rs/core/src/lib.rs` | `mod fork_defaults;` (on-prem Proactive default seam, apex-xt2.9 item 2) | MUST SURVIVE |
| `codex-rs/core/src/config/config_tests.rs` | T1c catalogue-v2 resolution test (apex-xt2.9 item 1) + T4 profile-layer × CLI `--enable` merge guard (apex-xt2.9 item 3) | MUST SURVIVE (T1c encodes fork catalogue behavior; T4 pins a profile-layer × CLI `--enable` layering scenario upstream has no coverage for — the merge machinery it exercises is byte-identical to upstream across base..HEAD) |
| `codex-rs/core/src/session/multi_agents.rs` | fork disjunct `|| crate::fork_defaults::proactive_delegation_default()` in `effective_multi_agent_mode` + `#[cfg(test)]` decl of `multi_agent_mode_selection_tests` (apex-xt2.9 item 2) | MUST SURVIVE (re-derive at ratchet if upstream reworks the selector) |
| `codex-rs/core/src/tools/handlers/apply_patch.rs` | function-tool apply_patch handler (ayl.52) + repair note as leading tool-output line (apex-xt2.11 item B) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` | format-teaching patch-param description (P1) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs`, `apply_patch_tests.rs` | seam tests (ayl.52) + repair-note tests (apex-xt2.11) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/mod.rs` | apply_patch handler module wiring (`FunctionApplyPatchHandler` re-export); queued same-file site: teachable parse error in `parse_arguments` (apex-xt2.7 CODEX-TOOLARGS-FIX-1 — extend this row's purpose when the hunk lands) | MUST SURVIVE |
| `codex-rs/core/src/tools/router.rs` | apply_patch routing | MUST SURVIVE |
| `codex-rs/core/tests/suite/apply_patch_cli.rs` | CLI-level seam tests | MUST SURVIVE |
| `codex-rs/core/tests/suite/multi_agent_mode.rs` | fork Proactive-default expectation rows (apex-xt2.9 item 2, spec §7.5 rows 1–4, 10) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/core/tests/suite/step_settings.rs` | fork Proactive-default expectation rows (apex-xt2.9 item 2, spec §7.5; final len==2 double-occurrence assert) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/core/tests/suite/subagent_notifications.rs` | fork Proactive-default expectation row (apex-xt2.9 item 2, spec §7.5) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/app-server/tests/suite/v2/turn_start.rs` | row 6/7 developer-message asserts via test-local `normalized_developer_message_texts` helper (fork wire normalization; apex-xt2.9 item 2, spec §7.5) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/model-provider/src/amazon_bedrock/mod.rs` | capability line | MUST SURVIVE |
| `codex-rs/models-manager/models.json` | 5 fork-added entries (net +438/−18 over base): fork-added glm-5.2 + grok-4.6 and re-added gpt-5.4-mini + gpt-5.2 (`799ec98053`, 374+/18−; the gpt pair was retired upstream in `eb7bd64ef9`) + qwen3.8-27b with the on-prem-trio v2 flips (apex-xt2.9 item 1, `c5c61277af`, 66+/2−) — all three on-prem entries at `multi_agent_version: "v2"`; the baseline commit also modified 8 pre-existing entries (base_instructions payloads; gpt-5.4/gpt-5.5 additionally carry the `model_messages` `{{ personality }}` template rewrite) — registered in the seam SoT (`docs/responses-compat-seam.md` §4, row 5); a manifest-only ratchet must not drop them | MUST SURVIVE (watch: upstream owns this file — re-apply the on-prem trio as one reviewed hunk at ratchet; the gpt-5.4-mini/gpt-5.2 re-adds are deliberate operator-directed fork entries from the baseline commit — keep unless the operator prunes them) |
| `codex-rs/models-manager/src/manager_tests.rs` | catalog tests (ayl.52 + apex-xt2.9 item 1 T1a/T1b) | MUST SURVIVE |
| `codex-rs/protocol/src/tool_name.rs` | `ToolName::from_response_fields` (recovers namespace from flattened dotted tool names; vLLM) + `mod tests` decl | MUST SURVIVE |
| `codex-rs/tools/src/tool_spec.rs`, `tool_spec_tests.rs` | function-tool spec type for apply_patch + `flatten_namespace_specs` | MUST SURVIVE |
| `AGENTS.md` | 116-line delta (`799ec98053`): a 107-line "Working Principles for AI Agents" SDD doc (mandatory spec → multi-agent review → TDD workflow, principles 0–4) PREPENDED + a claude-mem harness context block APPENDED (committed at operator direction) | merge-careful (upstream also edits AGENTS.md; the SDD doc is operator-mandated and must survive — re-derive at ratchet) |

## MUST SURVIVE (feature level)

1. **Content-type seam** — `normalize_content_types` on `ResponsesOptions`,
   body-encode branch in `responses.rs`, `content_type_compat` module:
   non-OpenAI providers get normalized content parts + translated agent
   messages. Gate must stay provider-scoped (OpenAI/Azure byte-identical).
2. **apply_patch function-tool seam** — format-teaching spec (P1), lenient
   P2 parser, function handler + routing + tool-name/spec plumbing
   (apex-ayl.52; acceptance green on Mac + SCS 2026-09-17), strict-first
   shape-repair pre-pass + repair-note surfacing (apex-xt2.11 A/B).
3. **On-prem providers + repo-side capabilities** — the `llm_proxy*`
   providers live in operator profile TOMLs on the deployment host
   (`[model_providers.*]` in `~/.codex/<name>.config.toml`; e.g.
   `llm_proxy_untagged` = proxy base_url + `CODEX_LLM_PROXY_KEY`,
   `supports_websockets = false`) — NOT in this repo; tag-header placement
   is load-bearing there (tag header only on the tagged provider). Repo-side
   seam: `!is_openai()`-anchored capability flags in
   `model-provider/src/provider.rs` (`normalize_content_types`,
   `flatten_namespace_tools`, `apply_patch_function_tool`).
4. **Model catalog** — glm-5.2 + grok-4.6 entries (`799ec98053`) and the
   qwen3.8-27b entry (apex-xt2.9 item 1, `c5c61277af`); all three on-prem
   entries at `multi_agent_version: "v2"` (the catalogue drives v2 without
   `--enable multi_agent_v2`) + `apply_patch_tool_type` default (Freeform)
   in the model-info fallback path.
5. **multi_agent_v2** — the upstream feature machinery is still unmodified,
   but the fork owns a Proactive-default seam (apex-xt2.9 item 2):
   `core/src/fork_defaults.rs` (fork-only) + one disjunct in
   `effective_multi_agent_mode` (`core/src/session/multi_agents.rs`) +
   test-expectation edits in `multi_agent_mode.rs`,
   `subagent_notifications.rs`, `step_settings.rs` and app-server
   `turn_start.rs` (all registered above). Keep the upstream machinery
   upstream-shaped; re-derive the disjunct at ratchet if upstream reworks
   the selector.

## Pending known future divergence (re-checked 2026-09-19)

- `apex-xt2.7` CODEX-TOOLARGS-FIX-1 (partially landed as of 2026-09-19):
  the `strict_int` coercion helpers landed at `02beac536d` (registered
  above: `tools/src/strict_int.rs` + re-exports). Still queued — see
  `docs/tool-args-type-error-census.md`: strict numeric coercion on 11
  fields (`core/src/tools/handlers/unified_exec.rs`,
  `unified_exec/write_stdin.rs`, `multi_agents_v2/wait.rs`, `sleep.rs`,
  `codex-rs/core/src/tools/code_mode/wait_handler.rs`), teachable parse error
  (`handlers/mod.rs::parse_arguments`), v2 tool spec teaching
  (`multi_agents_spec.rs` + `multi_agents_v2/` specs). 7 of 8 sites are on
  currently UNMODIFIED upstream files; the `handlers/mod.rs`
  (`parse_arguments`) site is in a file already a registered fork
  modification (row above). Add rows for the 7 unmodified files to the
  table above after it lands.

## Ratchet runbook (next pull)

1. `git fetch origin`; note advance:
   `git rev-list --count reference/upstream-openai..origin/main`.
2. Go/no-go (read-only): `weave_preview_merge(base_branch=origin/main,
   target_branch=feat/normalize-content-types-vllm)`; on any non-clean file,
   `weave_findings(..., file_path=<f>)` for typed conflicts / DANGLING /
   SHADOW before touching the tree.
3. `git merge --no-ff origin/main`. Resolution policy:
   - entities we never touched → take upstream's version;
   - fork-only additions (imports, tests, mod lines, provider entries,
     catalog rows) → keep;
   - true semantic collision in seam files (`responses.rs`, `client.rs`) →
     re-apply the seam hunks on upstream's version per the per-file strategy
     above; NEVER drop a MUST SURVIVE feature silently.
4. Gates before pushing: `cargo check` (affected crates first:
   codex-api, codex-core, codex-model-provider, codex-models-manager,
   codex-tools, codex-apply-patch), `just test -p codex-core -p
   codex-models-manager -p codex-model-provider -p codex-api`, `just fmt`.
   Load discipline: no full workspace `--all-features` builds on the Mac
   while other Rust builds run; cap `CARGO_BUILD_JOBS=4`.
5. Live wire check on a fresh build: `codex-harness` cp-01 (glm F1) + cp-02
   (qwen regression) — the seam must still show the function tool with the
   format-teaching description and all-`text` content parts.
6. Push BOTH remotes: netbrah/codex (GitHub) via full URL, bitbucket
   `APEX/codex` via the `fork` remote pushurl. **NEVER push to
   openai/codex (`origin` is read-only).**
7. Advance the parent pointer: `git branch -f
   reference/upstream-openai origin/main`; re-run the audit diff
   (`git diff reference/upstream-openai..HEAD`) — it must shrink to the fork
   delta only; update this file's "Last verified".
