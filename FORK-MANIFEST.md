# Codex-Combined Fork — Divergence Manifest (pull ratchet)

> **Purpose:** Track every fork-specific file and modification that must survive
> upstream merges (openai/codex main → this branch). After every ratchet, verify
> against this manifest and update "Last verified".
>
> **Last verified:** 2026-09-17 (UTC) — base `f3da3861c5`
> (`reference/upstream-openai`) = origin/main HEAD, MERGED into branch
> `feat/normalize-content-types-vllm` @ `7bcd344fa7` (+ fork-ops commits);
> post-merge gate pack 3-run classified clean (2 upstream-side flaky tests
> documented, reproduced on pure upstream f3da3861c5); both remotes in sync
> (netbrah/codex + APEX/codex); SCS ff-synced.

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
| `model-provider/src/provider.rs` | clean very_high | upstream deleted `enforce_managed_residency` (unreferenced in output); our 3 llm_proxy provider entries auto-merge |
| `models-manager/src/model_info.rs` | clean very_high | upstream personality refactor (`with_config_overrides`, `model_info_from_slug`, `local_model_messages` edited); `model_info_from_slug` still exists — our new test compiles |
| `models-manager/src/model_info_tests.rs` | **1 CONFLICT (benign)** | flagged fn `baked_personality_section_is_preserved_without_enabled_explicit_none` "both modified" — but WE never touched it: upstream renamed it (dropped `personality_enabled`) + switched to `codex_prompts::render_model_instructions`. **Resolution: take upstream version of that fn; keep our two additive changes** (the `ApplyPatchToolType` import + `fallback_model_info_enables_apply_patch_for_unknown_models` test at EOF) |
| `tools/src/lib.rs` | clean very_high | upstream added `output_schema` module; our +1 re-export line auto-merges |

Upstream 193-commit themes touching our seams: `personality_enabled` config
removal + `render_model_instructions` API (models-manager, codex_prompts
crate); Guardian endpoint consolidation (codex-api); client.rs transport
refactor; spec_plan reorganization; tools crate `output_schema`.

## Fork-only files (50 added; no upstream equivalent — merge risk: none)

### Code (4) — MUST SURVIVE

| file | purpose |
|---|---|
| `codex-rs/codex-api/src/endpoint/content_type_compat.rs` | seam core: content-type normalization + agent-message translation for non-OpenAI providers (apex-ayl.52) |
| `codex-rs/codex-api/src/endpoint/content_type_compat_tests.rs` | tests for above |
| `codex-rs/apply-patch/src/streaming_parser_p2_tests.rs` | apply-patch P2 lenient-parser tests |
| `codex-rs/protocol/src/tool_name_tests.rs` | tool-name constant tests |

### Docs (46) — KEEP (campaign record; zero merge risk)
- `docs/responses-compat-seam.md` — seam SoT incl. §6 conflict-site map
  (the authoritative reference this manifest complements)
- `docs/responses-compat-apply-patch-task-breakdown.md`,
  `docs/responses-compat-apply-patch-format.md` — campaign plan + format SoT
- `docs/reviews/apply-patch-format-spec-*.md` (11 spec review rounds),
  `docs/reviews/apply-patch-task-breakdown-*.md`,
  `docs/reviews/impl-item5-*.md` — multi-agent review audit trail
- `docs/vllm-glm-toolcall-research.md` — vLLM source research (prior seat)
- `docs/tool-args-type-error-census.md` — type-error census + fix plan (apex-xt2.2)

## Fork modifications to shared files (27)

### 10 also modified upstream in this window — per-file strategy (table above)

### 17 we modified, upstream UNTOUCHED in this window (conflict risk: none this ratchet)

| file | what we changed | class |
|---|---|---|
| `codex-rs/apply-patch/src/parser.rs` | P2 lenient AddFile state (accepts unprefixed content lines) | MUST SURVIVE |
| `codex-rs/apply-patch/src/streaming_parser.rs` | streaming parse support for P2 | MUST SURVIVE |
| `codex-rs/apply-patch/tests/suite/tool.rs` | P2 tests | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/apply_patch.rs` | function-tool apply_patch handler (item 1) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` | format-teaching patch-param description (P1) | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs`, `apply_patch_tests.rs` | tests | MUST SURVIVE |
| `codex-rs/core/src/tools/handlers/mod.rs` | apply_patch handler module wiring | MUST SURVIVE |
| `codex-rs/core/src/tools/router.rs` | apply_patch routing | MUST SURVIVE |
| `codex-rs/core/tests/suite/apply_patch_cli.rs` | CLI-level seam tests | MUST SURVIVE |
| `codex-rs/model-provider/src/amazon_bedrock/mod.rs` | capability line | MUST SURVIVE |
| `codex-rs/models-manager/models.json` | +392L: glm-5.2 / qwen3.8-27b catalog entries | MUST SURVIVE (watch: upstream may start owning this file — if so, re-apply entries at ratchet) |
| `codex-rs/models-manager/src/manager_tests.rs` | catalog tests | MUST SURVIVE |
| `codex-rs/protocol/src/tool_name.rs` | apply_patch tool-name constant | MUST SURVIVE |
| `codex-rs/tools/src/tool_spec.rs`, `tool_spec_tests.rs` | function-tool spec type for apply_patch | MUST SURVIVE |
| `AGENTS.md` | appended APEX campaign-state section | re-append ours on any upstream edit (keep upstream's content too) |

## MUST SURVIVE (feature level)

1. **Content-type seam** — `normalize_content_types` on `ResponsesOptions`,
   body-encode branch in `responses.rs`, `content_type_compat` module:
   non-OpenAI providers get normalized content parts + translated agent
   messages. Gate must stay provider-scoped (OpenAI/Azure byte-identical).
2. **apply_patch function-tool seam** — format-teaching spec (P1), lenient
   P2 parser, function handler + routing + tool-name/spec plumbing
   (apex-ayl.52; acceptance green on Mac + SCS 2026-09-17).
3. **On-prem provider entries** — `llm_proxy`, `llm_proxy_untagged`,
   `llm_proxy_qwen` in `model-provider/src/provider.rs` (http_headers
   placement is load-bearing: tag header only on the tagged provider).
4. **Model catalog** — glm-5.2 / qwen3.8-27b entries in `models.json` +
   `apply_patch_tool_type` default (Freeform) in the model-info fallback path.
5. **multi_agent_v2** — UPSTREAM code (verified 2026-09-17: unmodified in
   `git diff reference/upstream-openai..HEAD`); we only consume it. Keep it
   upstream-shaped; any fork edit lands as a shared-file modification and
   must be registered here.

## Pending known future divergence (not landed as of 2026-09-17)

- `apex-xt2.7` CODEX-TOOLARGS-FIX-1 (queued, model-agnostic ratchets — see
  `docs/tool-args-type-error-census.md`): strict numeric coercion on 11
  fields (`core/src/tools/handlers/unified_exec.rs`,
  `unified_exec/write_stdin.rs`, `multi_agents_v2/wait.rs`, `sleep.rs`,
  `code_mode/wait_handler.rs`), teachable parse error
  (`handlers/mod.rs::parse_arguments`), v2 tool spec teaching
  (`multi_agents_spec.rs` + `multi_agents_v2/` specs). All on currently
  UNMODIFIED upstream files — add rows to the table above after it lands.

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
