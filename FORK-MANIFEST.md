# Codex-Combined Fork — Divergence Manifest (pull ratchet)

> **Purpose:** Track every fork-specific file and modification that must survive
> upstream merges (openai/codex main → this branch). After every ratchet, verify
> against this manifest and update "Last verified".
>
> **Last verified:** 2026-09-19 — base `f3da3861c5`
> (`reference/upstream-openai`) unchanged since the 2026-09-16 upstream
> merge @ `7bcd344fa7` (ratchet verified 2026-09-17; that audit block
> below stands as-is); manifest re-derived at fork
> HEAD `9ffeb7db7d` (apex-xt2.7 item 6, campaign closure). Since 10d14858c7:
> xt2.9 item 3 on-prem catalogue spec doc (`c2c1b8a620`), xt2.10
> output-normalization seam + spec (`2b60e20ac3`/`edbc5150e1`), xt2.11 item C
> lax-retry spec (`0c560d5460`), and xt2.7 items 2-5 tool-args ratchet
> (`cf1f8bb8ae`, `d96703897f`, `a81aa1d3a0`, `9ffeb7db7d`).
> Both remotes (netbrah/codex + APEX/codex) in sync at `9ffeb7db7d`; SCS
> ff-synced (the item-6 docs commit re-pushes both remotes and re-syncs
> SCS, so this holds after landing).
> All counts below = first-hand `git diff f3da3861c5..9ffeb7db7d
> --name-status` (A=70, M=79; 149 total).

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

## Fork-only files (69 added; no upstream equivalent — merge risk: none)

Count = `git diff f3da3861c5..9ffeb7db7d --name-status` minus this manifest
(self-reference). `docs/tool-args-ratchet-spec.md` (the apex-xt2.7 campaign
record) lands in the item-6 commit alongside this re-derivation and is not in
this count.

### Code (19) — MUST SURVIVE

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

| `codex-rs/core/src/tools/handlers/args_parse.rs` | shared tool-args parse funnel: teachable parse errors with tool + parameter attribution (apex-xt2.7 item 3) |
| `codex-rs/core/src/tools/handlers/args_parse_tests.rs` | tests for above |
| `codex-rs/core/src/tools/code_mode/wait_handler_tests.rs` | strict-int timeout coercion tests (item 2) + deny_unknown_fields tests (item 4) |
| `codex-rs/core/src/tools/handlers/multi_agents/wait_tests.rs` | v1 wait strict-int + D3 deny-attr tests (items 2/4) |
| `codex-rs/core/src/tools/handlers/multi_agents_v2/wait_tests.rs` | v2 wait strict-int coercion tests (item 2) |
| `codex-rs/core/src/tools/handlers/request_permissions_tests.rs` | D2 deny_unknown_fields tests (item 4) |
| `codex-rs/core/src/tools/handlers/sleep_tests.rs` | strict-int duration coercion tests (items 2/4) |
| `codex-rs/core/src/tools/handlers/test_sync_tests.rs` | strict-int coercion tests (items 2/4) |
| `codex-rs/core/src/tools/handlers/unified_exec/write_stdin_tests.rs` | strict-int + exec env-fold tests (items 2/4) |
| `codex-rs/core/tests/suite/tool_args_ratchet.rs` | campaign integration suite: ratchet coverage across the shared funnel (items 2-4) |

### Docs (50) — KEEP (campaign record; zero merge risk)
- `docs/responses-compat-seam.md` — seam SoT incl. §6 conflict-site map
  (the authoritative reference this manifest complements)
- `docs/responses-compat-apply-patch-task-breakdown.md`,
  `docs/responses-compat-apply-patch-format.md` — campaign plan + format SoT
- `docs/reviews/apply-patch-format-spec-*.md` (15),
  `docs/reviews/apply-patch-task-breakdown-*.md` (6),
  `docs/reviews/impl-item*.md` (21) — multi-agent review audit trail
- `docs/vllm-glm-toolcall-research.md` — vLLM source research (prior seat)
- `docs/tool-args-type-error-census.md` — type-error census + fix plan (apex-xt2.2)

- `docs/onprem-mode-catalog-spec.md` — on-prem mode catalogue spec (apex-xt2.9
  item 3, `c2c1b8a620`)
- `docs/apply-patch-lax-retry-spec.md` — apply-patch lax-retry spec v1.4 +
  live-pin re-derivation + acceptance record (apex-xt2.11 item C,
  `0c560d5460`)
- `docs/tool-output-normalization-spec.md` — tool-output normalization spec
  (apex-xt2.10, `edbc5150e1`)

## Fork modifications to shared files (79)

### 10 also modified upstream in this window — per-file strategy (table above)

### 69 we modified, upstream UNTOUCHED in the merged 2026-09-17 window (split re-derived 2026-09-19 at 9ffeb7db7d)

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
| `codex-rs/core/src/tools/handlers/mod.rs` | apply_patch handler module wiring (`FunctionApplyPatchHandler` re-export); landed in item 3: teachable parse error in `parse_arguments` with tool + parameter attribution (apex-xt2.7 CODEX-TOOLARGS-FIX-1) | MUST SURVIVE |
| `codex-rs/core/src/tools/router.rs` | apply_patch routing | MUST SURVIVE |
| `codex-rs/core/tests/suite/apply_patch_cli.rs` | CLI-level seam tests | MUST SURVIVE |
| `codex-rs/core/tests/suite/multi_agent_mode.rs` | fork Proactive-default expectation rows (apex-xt2.9 item 2, spec §7.5 rows 1–4, 10) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/core/tests/suite/step_settings.rs` | fork Proactive-default expectation rows (apex-xt2.9 item 2, spec §7.5; final len==2 double-occurrence assert) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/core/tests/suite/subagent_notifications.rs` | fork Proactive-default expectation row (apex-xt2.9 item 2, spec §7.5) + item-4 deny-attr expectation update | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |

| `codex-rs/app-server/tests/suite/v2/turn_start.rs` | row 6/7 developer-message asserts via test-local `normalized_developer_message_texts` helper (fork wire normalization; apex-xt2.9 item 2, spec §7.5) | MUST SURVIVE (re-derive at ratchet; expectations encode fork behavior) |
| `codex-rs/model-provider/src/amazon_bedrock/mod.rs` | capability line | MUST SURVIVE |
| `codex-rs/models-manager/models.json` | 5 fork-added entries (net +438/−18 over base): fork-added glm-5.2 + grok-4.6 and re-added gpt-5.4-mini + gpt-5.2 (`799ec98053`, 374+/18−; the gpt pair was retired upstream in `eb7bd64ef9`) + qwen3.8-27b with the on-prem-trio v2 flips (apex-xt2.9 item 1, `c5c61277af`, 66+/2−) — all three on-prem entries at `multi_agent_version: "v2"`; the baseline commit also modified 8 pre-existing entries (base_instructions payloads; gpt-5.4/gpt-5.5 additionally carry the `model_messages` `{{ personality }}` template rewrite) — registered in the seam SoT (`docs/responses-compat-seam.md` §4, row 5); a manifest-only ratchet must not drop them | MUST SURVIVE (watch: upstream owns this file — re-apply the on-prem trio as one reviewed hunk at ratchet; the gpt-5.4-mini/gpt-5.2 re-adds are deliberate operator-directed fork entries from the baseline commit — keep unless the operator prunes them) |
| `codex-rs/models-manager/src/manager_tests.rs` | catalog tests (ayl.52 + apex-xt2.9 item 1 T1a/T1b) | MUST SURVIVE |
| `codex-rs/protocol/src/tool_name.rs` | `ToolName::from_response_fields` (recovers namespace from flattened dotted tool names; vLLM) + `mod tests` decl | MUST SURVIVE |
| `codex-rs/tools/src/tool_spec.rs`, `tool_spec_tests.rs` | function-tool spec type for apply_patch + `flatten_namespace_specs` | MUST SURVIVE |
| `AGENTS.md` | 116-line delta (`799ec98053`): a 107-line "Working Principles for AI Agents" SDD doc (mandatory spec → multi-agent review → TDD workflow, principles 0–4) PREPENDED + a claude-mem harness context block APPENDED (committed at operator direction) | merge-careful (upstream also edits AGENTS.md; the SDD doc is operator-mandated and must survive — re-derive at ratchet) |

Landed 10d14858c7..9ffeb7db7d (apex-xt2.7 items 2-5; all "MUST SURVIVE" —
the tool-args ratchet is fork behavior upstream has no coverage for):

| file | what we changed | class |
|---|---|---|
| `codex-rs/core/src/tools/code_mode/wait_handler.rs` | strict-int timeout coercion (item 2) + deny_unknown_fields on model-facing struct (item 4) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/multi_agents/wait.rs` | strict-int timeout coercion (item 2) + D3 deny_unknown_fields (item 4) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/sleep.rs` | strict-int duration coercion (item 2) + deny_unknown_fields (item 4) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/test_sync.rs` | strict-int coercion (item 2) + deny_unknown_fields (item 4) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/unified_exec.rs` | strict-int coercion (item 2) + exec_command env fold (item 4) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/unified_exec/write_stdin.rs` | strict-int coercion (item 2) + deny_unknown_fields (item 4) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/unified_exec/exec_command.rs` | teachable parse-error funnel site (item 3 shared-funnel sweep) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/unified_exec_tests.rs` | sweep test updates: strict coercion / teachable errors / deny attrs (items 2-4) | MUST SURVIVE |
| `codex-rs/core/tests/suite/mod.rs` | `mod tool_args_ratchet;` decl (item 2) | MUST SURVIVE |
| `codex-rs/rollout-trace/src/reducer/tool/terminal.rs`, `codex-rs/rollout-trace/src/reducer/tool/terminal_tests.rs` | D5 mirror: rollout-trace terminal reducer + tests reflecting the strict-coerced tool args (item 2) | MUST SURVIVE (mirror of core item-2 behavior) |
| `codex-rs/core/src/tools/handlers/dynamic.rs` | teachable parse-error funnel site (item 3) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/mcp_resource.rs`, `codex-rs/core/src/tools/handlers/mcp_resource/{list_mcp_resource_templates,list_mcp_resources,read_mcp_resource}.rs`, `codex-rs/core/src/tools/handlers/mcp_resource_tests.rs` | teachable parse-error funnel sites (item 3) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/multi_agents/{close_agent,resume_agent,send_input,spawn}.rs`, `codex-rs/core/src/tools/handlers/multi_agents_tests.rs` | teachable parse-error funnel sites (item 3) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/multi_agents_v2/{followup_task,interrupt_agent,list_agents,send_message,spawn,wait}.rs` | teachable parse-error funnel sites (item 3); `wait.rs` also strict-int timeout coercion (item 2) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/{plan,request_permissions,request_plugin_install,request_user_input,request_user_input_async,request_user_input_spec,send_message_to_user_async,view_image,wait_for_environment}.rs` | teachable parse-error funnel sites (item 3); `request_plugin_install.rs` + `request_user_input_spec.rs` also carry item-4 deny_unknown_fields | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/request_plugin_install_tests.rs` | item-4 deny-attr D2 tests (core side) | MUST SURVIVE |
| `codex-rs/tools/src/request_plugin_install.rs`, `codex-rs/tools/src/request_plugin_install_tests.rs` | item-4 deny_unknown_fields on the model-facing spec struct + tests (tools side) | MUST SURVIVE |
| `codex-rs/core/tests/suite/direct_tool_metadata.rs`, `codex-rs/core/tests/suite/tool_harness.rs` | funnel expectation updates (item 3) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/multi_agents_spec.rs` | (C) v2 spec-level negative field guidance in collaboration tool descriptions — 4 additive one-liners, file stays 878 LoC (item 5, ratchet C) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/multi_agents_spec_tests.rs` | ratchet-C negative-guidance test + exact-string description pin (item 5) | MUST SURVIVE |

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
6. **Tool-args ratchet (apex-xt2.7)** — fork-owned hardening of model-facing
   tool arguments: `strict_int` coercion helpers (fork-only, re-exported
   from `tools/src/lib.rs`), teachable parse errors with tool + parameter
   attribution through the shared funnel (`handlers/args_parse.rs`),
   deny_unknown_fields on the model-facing args structs, and v2
   spec-level negative field guidance (`multi_agents_spec.rs`); the
   rollout-trace terminal reducer mirrors the coerced behavior (D5). All
   rows registered above; spec SoT = `docs/tool-args-ratchet-spec.md`.

## Pending known future divergence (re-checked 2026-09-19)

- `apex-xt2.7` CODEX-TOOLARGS-FIX-1: **fully landed 2026-09-19** — strict-int
  coercion (helpers at `02beac536d`; 11 fields via items 2-3), teachable
  parse error in `handlers/mod.rs::parse_arguments` (item 3), deny_unknown
  fields on model-facing structs (item 4), v2 spec-level negative guidance
  (item 5) — commits `cf1f8bb8ae`, `d96703897f`, `a81aa1d3a0`,
  `9ffeb7db7d`; every row registered above. Spec SoT
  `docs/tool-args-ratchet-spec.md` lands in the item-6 commit alongside
  this re-derivation (not in the counts above). Nothing of this work is
  still queued.

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
