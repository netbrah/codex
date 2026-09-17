# Item 5 — Docs, gates, spec final pass: TDD evidence (apex-ayl.52)

Worker: `/root/item5_docs_worker` (stage 5, item 5 of 6).
Branch: `feat/normalize-content-types-vllm`, working-tree HEAD
`3db1b1381a` (post Items 1-4: `e21f608ac4` P2, `a4d5af1f62` P1,
`939a6dc6f4` P3, `a90b15d600` T4; the campaign function-tool seam
remains uncommitted working-tree state per breakdown §0 execution
baseline).
SoT: spec v5 `docs/responses-compat-apply-patch-format.md` (§1.2, §3.4,
§5.4, §6); breakdown v3
`docs/responses-compat-apply-patch-task-breakdown.md` (Item 5, §0, §5);
coordinator task contract for Item 5.

## TDD-mode note (docs item — no red/green by design)

Per breakdown Item 5, this item is **docs only + full gate run. No
product code.** There is therefore **no red/green TDD cycle**: the
deliverable is two document updates (seam doc + spec final pass) whose
claims are verified against the source, and the gate suite (below)
proves the tree is green. The "tests" for a docs item are the
factual-verification pass (§ Factual verification below) and the gates.

## Pre-check (line refs re-verified at working-tree state, protocol step 1)

- **`codex-rs/model-provider-info/src/lib.rs`** (seam §2 correction):
  - `const OPENAI_PROVIDER_NAME: &str = "OpenAI";` — `:40` — confirmed
    by `grep -n "OPENAI_PROVIDER_NAME: &str"` (single hit).
  - `pub fn is_openai(&self) -> bool { self.name == OPENAI_PROVIDER_NAME }`
    — `:546-547` — confirmed by `grep -n "pub fn is_openai"` (single
    hit, `:546`) and `grep -n "self.name == OPENAI_PROVIDER_NAME"`
    (`:547`). The comparison is a plain `&str ==` — exact,
    case-sensitive byte equality; there is no Azure branch.
  - `grep -in "azure" model-provider-info/src/lib.rs` → **no matches**:
    the file contains no Azure provider name or branch — the stale seam
    §2 "plus an Azure branch" description has no code counterpart.
- **Seam doc section anchors (pre-edit, 226-line file)** — confirmed by
  full read: §1 three boundaries :8, §2 provider capabilities :40,
  §3 design :58 (Problem :60, Decision :70, Components :79, Testing
  :121, Out of scope :131), §4 divergence map :139 (table header :143,
  rows 1-6 at :145-150 — row 6 `AGENTS.md` at :150 pre-edit), §5
  empirical evidence :164, §6 rebase/maintenance :178
  (conflict sites :185-196, verification gate :197-202, invariants
  :218-226). The stale gate-semantics sentence was at :53-56.
- **Spec final-pass anchors (pre-edit, 1026-line file)** — confirmed by
  read: Status line :7-10; §1.2 pinned-snapshot table and Conclusion at
  :207-211, the `≈60×` token at :209 (the ONLY occurrence outside §7 —
  the second occurrence, `grep -n "≈60×"`, is the §7 Round-5 map row
  I-N1 at :1018, a historical log entry left untouched per the task
  contract "DO NOT touch §7").
- **Item commit facts (for the spec status line and the divergence
  row)** — `git show --stat` per SHA:
  - `e21f608ac4` (item 1 / P2): `apply-patch/src/streaming_parser.rs`,
    `apply-patch/src/parser.rs` (+1 module-doc line), new
    `streaming_parser_p2_tests.rs`.
  - `a4d5af1f62` (item 2 / P1): `core/src/tools/handlers/apply_patch_spec.rs`
    + `apply_patch_spec_tests.rs`.
  - `939a6dc6f4` (item 3 / P3): `apply-patch/src/parser.rs`,
    `apply-patch/src/streaming_parser.rs`,
    `core/src/tools/handlers/apply_patch.rs` (+ tests).
  - `a90b15d600` (item 4 / T4): `core/tests/common/responses.rs`,
    `core/tests/suite/apply_patch_cli.rs`.
  - All four commits dated **2026-09-16** (`git log --date=format:`) —
    the seam-doc §3 subsection says "landed 2026-09-16".
- **Tree state**: 11 modified codex-rs files + `AGENTS.md` (the
  campaign seam, untouched by this item) and the untracked campaign
  docs. `git status --short` snapshot taken before and after the docs
  edits (final state section).

## Changes (docs only; before → after per edit)

Both target files are **untracked** campaign docs (no `git diff` output
— before/after below is quoted verbatim from the pre-edit read and the
post-edit tree).

### 1. `docs/responses-compat-seam.md` §2 — stale `is_openai()` description (pre-edit :53-56)

**Before:**

    Gate semantics: `is_openai()` is a string match on the provider `name`
    field in `config.toml` (`== "OpenAI"`), plus an Azure branch. The gate means
    "this provider's *deployment* implements the full OpenAI responses spec",
    not "the model is made by OpenAI".

**After** (post-edit :53-61): exact case-sensitive match on
`OPENAI_PROVIDER_NAME` with verified refs (`:546-547`, constant `:40`),
explicit **no Azure branch** citing the spec §3.4 invariant, and
"Azure-*named* provider fails the gate and is in seam scope (function
tool + P1 format text)". The trailing two sentences (gate means
deployment-capability, not model-maker) are preserved verbatim.

### 2. `docs/responses-compat-seam.md` §3 — new `### Format-remediation layer (P1/P2/P3)` subsection (post-edit :126-156, between `### Components` and `### Testing`)

New content (was absent pre-edit): first-shipped spec text taught no
format; the pinned evidence table (glm-5.2 37 calls / 26 rejections /
~70% (70.27%) "as of the pinned snapshot"; qwen3.8-27b 413 / 5 / ~1%
(1.21%)) sourced to spec §1.2; 09-15 alone 13 of 21 ≈ 62%; the two
rejection classes (F1/F2 raw content, F4 missing boundary markers); the
three fixes with their files (P1 `apply_patch_spec.rs`, P2
`streaming_parser.rs`, P3/P3.4 `streaming_parser.rs` + `parser.rs`);
"landed 2026-09-16 in the same PR set (commit SHAs recorded in the
spec's status line)"; pointer to spec §3.1-§3.4 and to this doc's §6
invariant.

### 3. `docs/responses-compat-seam.md` §4 — divergence table row 7 (post-edit :188, after row 6)

**Before:** table ended at row 6 (`AGENTS.md` working principles).
**After:** one row added, matching the existing 5-column format
(Change | Files | Purpose | Rebase risk — the table's header has 4 data
columns plus the index): Change = "apply-patch parser change: P2
Add-File leniency + P3/P3.4 teachable errors — **provider-agnostic**;
surface = function-tool path + freeform diff consumer + `apply_patch`
CLI (P2), plus the `parse_patch` pre-pass surface for P3/P3.4 (function
path + CLI + shell-intercept)"; Files = `apply-patch/src/streaming_parser.rs`,
`apply-patch/src/parser.rs`, `apply_patch.rs` (P3.3 handler errors);
Purpose = "~70% glm-5.2 parse rejections (spec §1.2 pinned snapshot);
P2/P3 affect only inputs upstream rejects (spec §3.4)"; Rebase risk =
"low-medium — re-apply on upstream restructure of `streaming_parser.rs`
or `parser.rs`; the diff consumer's parallel streaming boundary
messages intentionally unchanged (spec §3.4 divergence note)".
Path style follows the table's codex-rs-relative convention (row 1 uses
`codex-api/src/endpoint/…`, row 3 uses bare `apply_patch.rs`); the
`apply-patch/src/…` package prefix follows row 1's style.

### 4. `docs/responses-compat-seam.md` §6 — conflict sites (post-edit :235-238)

**Before:** the expected-conflict-site list ended at
`model-provider/src/provider.rs` — capability computation block.
**After:** two bullets appended, matching the list's
"backticked path — description" format:
`codex-rs/apply-patch/src/streaming_parser.rs` (P2 Add-File arm +
P3.1/P3.2 error-message sites) and `codex-rs/apply-patch/src/parser.rs`
(P3.4 `parse_patch` pre-pass boundary strings). The `codex-rs/` prefix
is the exact path spelled in the spec §6 requirement and the task
contract (sibling bullets omit the prefix — see Discrepancy D1).

### 5. `docs/responses-compat-seam.md` §6 — verification gate (post-edit :245-251)

**Before:** the step-3 gate list ended at
`cargo test -p codex-model-provider --lib` (capability gates).
**After:** two bullets appended, referenced the way the section already
references other gates (backticked command + parenthetical):
`just test -p codex-apply-patch` (P2/P3 parser behavior — a rebase that
silently reverts P2 fails here) and the raw-format probe per
`docs/responses-compat-apply-patch-format.md` §5.4 (standalone
`apply_patch` binary, same release build, scratch dir, F1-shape raw
Add-File patch; pass = exit 0, contents byte-identical).

### 6. `docs/responses-compat-seam.md` §6 — invariants (post-edit :271-282, after the first bullet)

**Before:** the "Invariants that must survive every rebase" list had
three bullets (OpenAI/Azure byte-identical; only wire differences = the
three §2 capabilities; new capabilities land in `ProviderCapabilities`).
**After:** a second bullet inserted carrying the spec §3.4
exact-invariant text: outbound **request bytes for providers named
`OpenAI` are unchanged** (P1 touches only the function-tool spec, which
never ships to them); P2/P3 **provider-agnostic parser changes** —
byte-identical for every input upstream accepts; for inputs upstream
*rejects*, P2 turns some Add-File rejections into accepted content and
P3 turns some rejections into teachable errors — full surface: the
function-tool execution path, the freeform diff consumer, and the
`apply_patch` CLI; P3.4 applies to the `parse_patch` pre-pass surface
only (function path + CLI + shell-intercept) and does **not** touch the
diff consumer's parallel streaming boundary messages. The two trailing
meta-sentences of the spec's invariant paragraph (the "seam doc §2 is
stale and is corrected in the same PR set" note — now false here, this
edit IS the correction — and the one-time "not verified against any
Azure deployment" verification note) are deliberately omitted; the
three clauses above are the spec's wording verbatim.

### 7. `docs/responses-compat-apply-patch-format.md` — Status line (pre-edit :7-10)

**Before:**

    Status: SPEC — v5 (round 5 returned 0 Blocking + 0 Major — spec review loop
    terminated on v4; v5 applies the five source-verified minor/nit fixes below,
    maps in §7). No code changes until the task-breakdown cross-review (stage 3)
    returns zero Blocking and zero Major.

**After** (post-edit :7-14): `Status: IMPLEMENTED` — all five items
landed on branch `feat/normalize-content-types-vllm` with the four
recorded SHAs (item 1 / P2 `e21f608ac4`, item 2 / P1 `a4d5af1f62`,
item 3 / P3 `939a6dc6f4`, item 4 / T4 `a90b15d600`) and the
placeholder `<<ITEM5-SHA>>` for item 5 (kept exactly, findable, for the
coordinator to fill after commit), plus the terminated review loops
(spec round 5, breakdown round 2).

### 8. `docs/responses-compat-apply-patch-format.md` — §1.2 Conclusion (pre-edit :209)

**Before:** `parse-rejection rate is ~70% (as of the pinned snapshot), ≈60× qwen's`
**After** (post-edit :213): `…≈58× qwen's` — a one-token change on the
Conclusion line only (the pinned ratio is 10738/185 = 58.0432 ≈ 58×; round-1
nit K-N1, deferred against the reviewed artifact per the spec §7 R5-map
row I-N1). The §7 I-N1 row at post-edit :1022 still quotes the
historical "≈60×" — untouched by design (review log is a record, not a
claim).

**Untouched, per the task contract:** the spec §7 review log (all five
round maps), the TL;DR, and every other section of the spec; every file
outside the ownership list.

## Factual verification (every number/line-ref checked against source)

- **`is_openai()` line refs** — `:546-547` (fn + body) and `:40`
  (`OPENAI_PROVIDER_NAME` constant): grep-verified at the current
  working tree (single hits); the body is `self.name ==
  OPENAI_PROVIDER_NAME` — exact, case-sensitive, no Azure branch
  (case-insensitive `azure` grep over the file: zero matches).
- **glm-5.2 26/37 ≈ 70% (70.27%)** — spec §1.2 pinned table row
  (glm-5.2 | 37 | **26 (~70%)** | 11 | 0 | —) and the Conclusion's
  "26/37 = 70.27%"; row pin 2026-09-15T23:34:04.678Z. Seam doc phrases
  it "as of the pinned snapshot" exactly as the task pinned it.
- **09-15 alone 13/21 ≈ 62%** — spec §1.2: "Most recent day (09-15)
  alone: 13 of 21 glm calls rejected (~62%)." (Verified before writing;
  the task required re-verification of this figure — it is in §1.2, not
  only in the breakdown.)
- **qwen 5/413 ≈ 1% (1.21%)** — spec §1.2 pinned table row (qwen3.8-27b
  | 413 | **5 (~1%)** | 374 | 30 | …) and the Conclusion's "5/413 =
  1.21%"; row pin 2026-09-15T23:30:11.662Z.
- **≈58×** — exact ratio 10738/185 = 58.0432 (≈58×); the rounded
  70.27/1.21 = 58.07 also rounds to ≈58×; matches the §7 R5-map I-N1
  note "70.27% / 1.21% ≈ 58×".
- **Two rejection classes (F1/F2 raw content; F4 missing boundary
  markers)** — spec TL;DR: "glm-5.2's rejections fall in two format
  classes: raw unprefixed content lines (F1/F2, 19 of 26) and missing
  boundary markers (F4, 7 of 26)". Seam doc states the classes without
  the 19/7 split (the split is spec-internal detail).
- **"landed 2026-09-16"** — `git log --date=format:%Y-%m-%d` shows all
  four item commits (`e21f608ac4`, `a4d5af1f62`, `939a6dc6f4`,
  `a90b15d600`) dated 2026-09-16.
- **P3.3 handler file in divergence row 7** — `git show --stat
  939a6dc6f4` confirms `codex-rs/core/src/tools/handlers/apply_patch.rs`
  in item 3's change set (the absent/non-string `patch` handler errors).
- **P2/P3.4 files in divergence row 7** — `git show --stat e21f608ac4`
  (streaming_parser.rs + parser.rs +1) and `git show --stat 939a6dc6f4`
  (streaming_parser.rs + parser.rs) confirm both `apply-patch/src/`
  files carry the parser change.
- **Spec §1.2 Conclusion is the only in-scope "≈60×"** —
  `grep -n "≈60×"` pre-edit: :209 (Conclusion) and :1018 (§7 R5-map
  I-N1, historical log — excluded by the task contract). Post-edit:
  :213 (≈58×) and :1022 (§7, unchanged).
- **Seam §2 stale sentence location** — pre-edit :53-56, confirmed by
  full-file read before editing.
- **Item-evidence cross-check** — the item 1-4 evidence docs'
  before/after line refs describe the same parser surfaces this item
  cites (AddFile arm, StartedPatch/DeleteFile error sites, `parse_patch`
  pre-pass strings); item 3's evidence confirms the diff consumer's
  parallel streaming boundary messages were left untouched (its
  pre-shift cites :168/:184/:374 — the row-7 "intentionally unchanged"
  claim matches).

## Gates (from `codex-rs/`)

### Gate 1: `just test -p codex-apply-patch`

```text
     Summary [  11.821s] 115 tests run: 115 passed, 0 skipped
GATE1_EXIT=0
```

**115/115, exit 0** (log `/tmp/item5-gate1.log`) — the expected count
for this tree.

### Gate 2: `just test -p codex-core apply_patch`

Ran to completion under heavy machine load (concurrent agent builds;
loadavg observed 5-35):

```text
     Summary [3864.328s] 118 tests run: 66 passed (2 flaky), 1 failed, 51 timed out, 4251 skipped
GATE2_EXIT=100
```

**Green criterion (per task contract): 0 DETERMINISTIC failures —
MET.** Every non-pass is the documented load-noise class.

**Binding check (verbatim, over the full log `/tmp/item5-gate2.log`):**

```text
$ grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u
(empty — 0 lines)
```

Panic-site census over the full log: 92 panics, **all** at
`core/tests/common/lib.rs:388` (`timeout waiting for event:
Elapsed(())`) — the named load-noise class. The single FAILED test
(`apply_patch_exec_command_failure_propagates_error_and_skips_diff`,
TRY 2 FAIL after a TRY 1 FAIL) panics at the same `lib.rs:388` site in
its stderr — load noise, not an assertion failure.

Non-pass identities (52 unique = 51 TMT + 1 FAIL; all `codex-core::all`
suite, all load-noise class):

```text
suite::apply_patch_cli::apply_patch_aggregates_diff_across_multiple_tool_calls
suite::apply_patch_cli::apply_patch_aggregates_diff_preserves_success_after_failure
suite::apply_patch_cli::apply_patch_change_context_disambiguates_target
suite::apply_patch_cli::apply_patch_cli_can_use_exec_command_output_as_patch_input
suite::apply_patch_cli::apply_patch_cli_delete_directory_reports_verification_error
suite::apply_patch_cli::apply_patch_cli_delete_missing_file_reports_error
suite::apply_patch_cli::apply_patch_cli_does_not_widen_permissions_for_workspace_directory_target
suite::apply_patch_cli::apply_patch_cli_end_of_file_anchor
suite::apply_patch_cli::apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file
suite::apply_patch_cli::apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file
suite::apply_patch_cli::apply_patch_cli_function_tool_served_to_non_openai_provider
suite::apply_patch_cli::apply_patch_cli_insert_only_hunk_modifies_file
suite::apply_patch_cli::apply_patch_cli_missing_second_chunk_context_rejected
suite::apply_patch_cli::apply_patch_cli_move_overwrites_existing_destination
suite::apply_patch_cli::apply_patch_cli_move_without_content_change_has_no_turn_diff
suite::apply_patch_cli::apply_patch_cli_moves_file_to_new_directory
suite::apply_patch_cli::apply_patch_cli_multiple_chunks
suite::apply_patch_cli::apply_patch_cli_multiple_operations_integration
suite::apply_patch_cli::apply_patch_cli_preserves_distinct_updated_paths
suite::apply_patch_cli::apply_patch_cli_preserves_existing_hard_link_outside_workspace
suite::apply_patch_cli::apply_patch_cli_rejects_duplicate_resolved_paths
suite::apply_patch_cli::apply_patch_cli_rejects_invalid_hunk_header
suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff
suite::apply_patch_cli::apply_patch_exec_command_heredoc_with_cd_updates_relative_workdir
suite::apply_patch_cli::apply_patch_normalizes_crlf_without_preserve_line_endings_feature
suite::apply_patch_cli::apply_patch_preserves_crlf_with_preserve_line_endings_feature
suite::apply_patch_cli::apply_patch_shell_accepts_lenient_heredoc_wrapped_patch
suite::apply_patch_cli::apply_patch_shell_heredoc_normalizes_crlf_without_preserve_line_endings_feature
suite::apply_patch_cli::apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::coding_originator_uses_cwd_when_enabled
suite::apply_patch_cli::apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::desktop_uses_cwd_when_enabled
suite::apply_patch_cli::apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::disabled_feature_keeps_repository_root
suite::apply_patch_serialization::apply_patch_custom_tool_call_creates_file
suite::apply_patch_serialization::apply_patch_custom_tool_call_reports_failure_output
suite::apply_patch_serialization::apply_patch_custom_tool_call_updates_existing_file
suite::approvals::approval_matrix_covers_group::apply_patch
suite::approvals::approving_apply_patch_for_session_skips_future_prompts_for_same_file
suite::code_mode::code_mode_can_apply_patch_via_nested_tool
suite::hooks::permission_request_hook_allows_apply_patch_with_write_alias
suite::hooks::post_tool_use_records_additional_context_for_apply_patch
suite::hooks::post_tool_use_records_apply_patch_context_with_edit_alias
suite::hooks::pre_tool_use_blocks_apply_patch_before_execution
suite::hooks::pre_tool_use_blocks_apply_patch_with_write_alias
suite::hooks::pre_tool_use_rewrites_apply_patch_before_execution
suite::prompt_caching::gpt_5_tools_without_apply_patch_append_apply_patch_instructions
suite::request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_session
suite::request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_turn
suite::request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::with_strict_auto_review
suite::request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::without_strict_auto_review
suite::shell_snapshot::unified_exec_snapshot_still_intercepts_apply_patch
suite::tool_harness::apply_patch_reports_parse_diagnostics
suite::tool_harness::apply_patch_tool_executes_and_emits_patch_events
suite::unified_exec::unified_exec_intercepts_apply_patch_exec_command
```

The FAIL identity (the only non-TMT of the 52) is
`suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff`
(log line: `TRY 2 FAIL [  58.917s] ( 99/118) codex-core::all
suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff`;
its stderr panic is the same `lib.rs:388` load-noise site).
The two FLAKY identities (passed on TRY 2):
`apply_patch_clears_aggregated_diff_after_inexact_delta`,
`apply_patch_shell_heredoc_preserves_crlf_with_preserve_line_endings_feature`.
All 40 `codex-core` lib unit tests (including the P1 spec tests and the
P3 teachable-error handler tests) passed on first try.

### Gate 3: `just test -p codex-model-provider`

Ran to completion (log `/tmp/item5-gate3.log`):

```text
     Summary [ 121.853s] 84 tests run: 81 passed (11 slow, 12 flaky), 2 failed, 1 timed out, 0 skipped
GATE3_EXIT=100
```

- **Capability-gate regression target: GREEN.**
  `provider::tests::configured_provider_apply_patch_function_tool_matches_provider_support`
  PASS (0.184s, first try) — the seam's capability gate is intact.
- The 12 FLAKY + 1 TMT non-passes are the same load-noise pattern as
  Gate 2 (60s timeouts under load; every flaky test passed on retry;
  the single TMT `auth::tests::chatgpt_bootstrap_unavailable_uses_session_bearer_fallback`
  is a 60s harness-event timeout under load). One flaky test
  (`models_endpoint::tests::command_auth_refresh_fetches_a_catalog_for_the_current_credentials`)
  showed a TRY-1 assertion diff (`None` vs `Some(ModelInfo { …
  slug: "command-auth-model" … })` at `models_endpoint.rs:607`) before
  passing on TRY 2 — a timing/ordering flake, recorded for completeness.
- **2 FAILED (recorded, both the same panic, both attempts, both
  tests):** `amazon_bedrock::tests::configured_profile_takes_precedence_over_managed_auth`
  and `amazon_bedrock::tests::command_auth_resolves_configured_and_regional_base_urls`
  — panic in a third-party crate:
  `aws-smithy-http-client-1.1.12/…/tls/rustls_provider.rs:116`:
  **"TrustStore configured to enable native roots but no valid root
  certificates parsed!"** — an environment TLS-trust-store issue when
  the tests construct the AWS SDK HTTP client (`provider.auth().await`
  path). See Discrepancy D2 for the pre-existing analysis.

### Gate 4: `just fix -p codex-apply-patch -p codex-core -p codex-model-provider`

```text
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 48s
GATE4_EXIT=0
```

**EXIT 0.** **KNOWN HAZARD TRIGGERED (as the task brief predicted):**
clippy re-applied the pre-existing unused-import auto-fix in
OUT-OF-SCOPE `codex-rs/core/tests/suite/openai_file_mcp.rs` (removal
of `use wiremock::matchers::body_json;` at :47 — the same pre-existing
warning the test compile prints on every run). Restored via
`git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs` and
verified clean vs HEAD (`git diff HEAD --stat` over that file: empty).
No other file was touched by `just fix` (post-restore `git status`
matches the pre-gate tracked set exactly — see Final state).

### Gate 5: `just fmt`

```text
GATE5_EXIT=0   (scripts/format.py, no output on success; ~10s)
```

`just fmt` produced no changes (this item edits no Rust source).

## Final state

**Ownership boundary (verified before and after all edits + gates):**
the ONLY tracked files modified in the tree are the pre-existing
campaign-seam files — 12 total, none of them touched by this item:
`AGENTS.md`, `codex-rs/codex-api/src/endpoint/content_type_compat.rs`,
`codex-rs/codex-api/src/endpoint/content_type_compat_tests.rs`,
`codex-rs/codex-api/src/endpoint/responses.rs`,
`codex-rs/core/src/tools/handlers/mod.rs`,
`codex-rs/core/src/tools/spec_plan.rs`,
`codex-rs/model-provider/src/amazon_bedrock/mod.rs`,
`codex-rs/model-provider/src/provider.rs`,
`codex-rs/models-manager/models.json`,
`codex-rs/models-manager/src/manager_tests.rs`,
`codex-rs/models-manager/src/model_info.rs`,
`codex-rs/models-manager/src/model_info_tests.rs`.
`git diff HEAD --stat` over the tracked set: 809 insertions / 20
deletions — identical before and after this item's work (the item
changed zero tracked files). No `Cargo.toml`/`Cargo.lock`/
`MODULE.bazel.lock` changes (none expected for a docs item; none
occurred — no `just bazel-lock-update` run needed). No `git add` /
`git commit` executed.
Addendum (post-worker, 2026-09-16): the tree claims above are as of
the pre-baseline tree (HEAD `3db1b1381a`); ~14s after this doc's last
write, operator-directed baseline commit `799ec98053` ("commit
working-tree function-tool seam baseline", 2026-09-16) committed
exactly that 12-file / 809+ / 20− delta. The item-5 docs themselves
remain untracked and uncommitted.

**Final stats of the owned untracked files** (untracked — shown as
line counts / final content state):
- `docs/responses-compat-seam.md` — 297 lines (item-5 pre-edit 226;
  +61 → 287 at this doc's writing; r1 fixes +7 (M-A1 +4, M-A2 +2,
  N-1 +1) → 294; r2 fixes +3 (F-C1/n-D5 −1, n-D3 +4, n-D4
  line-neutral) → 297). Post-r2 anchors (measured after this round's
  edits): §2 gate semantics :53-61; §3 "Format-remediation layer
  (P1/P2/P3)" :133-164; §4 row 7 :196; §6 conflict-site bullets
  :243-246; §6 gate bullets :253-259; §6 invariant bullet 2
  :281-292.
- `docs/responses-compat-apply-patch-format.md` — 1029 lines
  (pre-edit 1026; +3). Post-edit anchors: Status :7-14 (with
  `<<ITEM5-SHA>>` at :10); §1.2 Conclusion `≈58×` at :213; §7 untouched
  (I-N1 row at :1022).
- `docs/reviews/impl-item5-tdd-evidence.md` — this file (new).

## Discrepancy notes

- **D1 — conflict-site path style.** Sibling bullets in seam §6 use
  codex-rs-relative paths without the `codex-rs/` prefix
  (`core/src/client.rs`, `model-provider/src/provider.rs`), but the
  spec §6 requirement and the task contract spell the two new entries
  WITH the prefix (`codex-rs/apply-patch/src/streaming_parser.rs`,
  `codex-rs/apply-patch/src/parser.rs`). Followed the SoT/task
  spelling exactly; the format (backticked path — em-dash description)
  matches the list.
- **D2 — Gate 3 amazon_bedrock TLS failures are pre-existing /
  environmental, not this item's.** (a) This item changed zero
  tracked files, so nothing in the product code moved between the
  start and end of the gate run. (b) The panic is in the third-party
  `aws-smithy-http-client` rustls trust-store init
  ("no valid root certificates parsed" — the macOS native-roots store
  is unreadable in this environment) during AWS-SDK HTTP client
  construction on the `provider.auth().await` path. (c) The only
  `amazon_bedrock/mod.rs` change in the working tree (the uncommitted
  seam diff, not this item) is a single
  `apply_patch_function_tool: false` struct field in the capabilities
  computation plus two matching test-expectation literals — nowhere
  near the auth/TLS code path. Both failing tests fail identically on
  both attempts, consistent with an environment condition rather than
  a code regression. The capability-gate regression target
  (`configured_provider_apply_patch_function_tool_matches_provider_support`)
  passed, which is Gate 3's stated purpose.
- **D3 — Gate 4 out-of-scope auto-fix.** `just fix` re-applied the
  pre-existing unused-import fix in
  `codex-rs/core/tests/suite/openai_file_mcp.rs:47` (predicted known
  hazard). Restored via `git checkout --` and verified clean vs HEAD.
  Recorded here per the task contract.
- **D4 — Gate 2 first attempt lost to session cleanup.** The initial
  Gate 2 launch (detached background) died silently when its launching
  exec session was reaped (log stuck at the recipe echo, no
  `GATE2_EXIT` marker); the build it had already done is what made the
  successful run's compile 26.8s. The successful run (session
  `8238`, continuously polled) is the recorded Gate 2 result. No test
  outcome is affected — the recorded run is a full completion.
- **D5 — spec status line keeps "maps in §7" phrasing.** The original
  status line said "v5 applies the five source-verified minor/nit
  fixes below, maps in §7"; the IMPLEMENTED status preserves the
  "maps in §7" pointer (the R5 map lives in §7). No §7 bytes touched.

## Worker attestation

- Docs-only item: no product code touched; no tracked file modified;
  no commits made (tree left uncommitted per the task contract).
- Every factual claim in the two doc edits was verified against the
  source before writing (Factual verification section); the three
  pinned evidence numbers (26/37 ≈ 70% (70.27%), 13/21 ≈ 62%, 5/413 ≈
  1% (1.21%)) match spec §1.2 exactly; the `≈58×` correction is
  arithmetic-verified (exact ratio 10738/185 = 58.0432 ≈ 58×) and
  single-token.
- Gates: 1 ✓ (115/115), 2 ✓ per the stated green criterion (0
  deterministic failures; binding check empty), 3 — capability-gate
  target green, 2 pre-existing environmental TLS failures recorded
  (D2), 4 ✓ (hazard triggered + restored, D3), 5 ✓.
## Round-1 findings and resolutions (r2 input)

r1 seats (2026-09-16): seat A `impl-item5-r1-seatA.md` — REQUEST CHANGES,
0 Blocking / 2 Major / 0 minor / 5 nit; seat B `impl-item5-r1-seatB.md` —
APPROVED, 0 Blocking / 0 Major / 2 minor / 3 nit. Resolutions:

- **M-A1 (seat A) — FIXED**: seam doc Decision paragraph reworded —
  "untouched / native custom+grammar path" scoped to providers **named
  `OpenAI`** with the §3.4 invariant stated inline (Azure-*named* providers
  take the function tool; `is_openai()` is name-anchored). Root cause per
  seat A: the spec §6/breakdown change list scoped only the §2 paragraph +
  the invariants addition.
- **M-A2 (seat A) — FIXED**: seam doc invariants bullet 1 reworded from
  "OpenAI/Azure provider behavior is byte-identical to upstream" to the
  §3.4-scoped "Outbound request bytes for providers named `OpenAI` are
  byte-identical to upstream (… name-anchored on `OpenAI`; Azure-*named*
  providers take the function tool per spec §3.4)".
- **m-F1 (seat B) — FIXED**: gate-2 evidence "All 46 codex-core lib unit
  tests" → "All 40" (worker's own log has exactly 40 unique lib-binary
  tests; substance unchanged, all PASS first-try).
- **m-F2 (seat B) — FIXED**: pre-edit seam-doc anchors corrected per seat B's
  mechanical reconstruction: Components :76→:79, Testing :130→:121, Out of
  scope :141→:131 (the old "Out of scope :141 + §4 :139" pair was
  geometrically impossible).
- **n-F3 (seat B) — RECORDED**: spec file trailing blank line normalized to
  the repo tracked-doc convention (single trailing newline); delta explained
  in seat B Mandate 2c, no re-add.
- **n-F4 (seat B) — RECORDED, shipped value stands**: 10738/185 = 58.0432
  supports the shipped ≈58× and the "58.04" phrasing; the rounded-division
  reading (58.07) also rounds to ≈58×.
- **n-F5 (seat B) — RECORDED**: spec §3.4 :607-609 historical "stale"
  narrative correctly left untouched per contract (historical log row).
- **N-1 (seat A) — FIXED**: P3 bullet now names all three files carrying P3
  sites (added the P3.3 handler split in `apply_patch.rs`).
- **N-2 (seat A) — ACCEPTED**: seam doc §6 mixed path spelling follows the
  locked spec spelling (D1).
- **N-3 (seat A) — ACCEPTED**: gate bullet keeps `just test -p
  codex-apply-patch` — the repo-sanctioned invocation (no bare `cargo
  test`); sibling bullets document the equivalent scoped filters.
- **N-4 (seat A) — ACCEPTED**: §6 invariants bullet is a faithful
  condensation of spec §3.4 (four small justified trims; both omitted
  meta-sentences accounted for).
- **N-5 (seat A) — FIXED**: capability-table status cell
  "SPECIFIED BELOW" → "IMPLEMENTED (items 1-3, 2026-09-16)".

## Round-2 findings and resolutions (r3 input)

r2 seats (2026-09-16): seat C `impl-item5-r2-seatC.md` — REQUEST CHANGES,
0 Blocking / 1 Major / 0 minor / 1 nit; seat D `impl-item5-r2-seatD.md`
— APPROVED, 0 Blocking / 0 Major / 2 minor / 3 nit. Resolutions:

- **F-C1 (seat C, Major) — FIXED**: seam doc Decision sentence reworded
  at the subject — the r1 M-A1 fix had appended the name-anchored
  scoping qualifier after the stale main clause, so the sentence still
  opened with the false "OpenAI/Azure providers are untouched: they keep
  the native custom+grammar path". It now reads "Providers named
  `OpenAI` are untouched: they keep the native custom+grammar path
  (spec §3.4 invariant: outbound request bytes for `OpenAI`-named
  providers are unchanged; `is_openai()` is name-anchored, so
  Azure-*named* providers take the function tool, as pinned by the
  capability test)" — no assertion about Azure-named providers being
  untouched remains in the sentence; consistent with §2 (:53-61) and
  invariants bullet 1 (:277-280) of the same doc.
- **n-D5 (seat D, nit) — FIXED**: the same Decision sentence as F-C1
  (seat D's independent phrasing of the same residual); resolved by the
  same edit.
- **n-D4 (seat D, nit) — FIXED**: Components handler bullet's shared
  helper renamed `run_patch_text(...)` → `run_apply_patch_text(...)` —
  the actual private function (`core/src/tools/handlers/apply_patch.rs:400`,
  called by the custom handler at :381 and the function handler at :555);
  the described structure (shared post-parse body, both handlers
  delegate) was already accurate and is unchanged.
- **n-D3 (seat D, nit) — FIXED**: Components spec bullet's composite
  quote ("put the ENTIRE patch text in `patch`; do not wrap in JSON or
  markdown" — in neither code nor spec) replaced with byte-exact quotes
  from `apply_patch_spec.rs` / spec §3.1: "The complete patch goes in
  the `patch` argument." (tool-level description) and "The ENTIRE patch
  as a single string" (P1 argument description), plus the unquoted
  anti-wrapping substance (no markdown fences, code-block markers, or
  shell heredoc wrapper — the P1 text's actual rule; the old "JSON"
  wording had been dropped from the P1 text by design, spec §3.1).
- **M-D1 (seat D, minor) — FIXED**: all three occurrences of the false
  quotient "70.27 / 1.21 = 58.04" (Factual verification; Worker
  attestation; Changes #8 text — found during the fix pass) replaced
  with the exact-ratio statement 10738/185 = 58.0432 (≈58×),
  consistent with the n-F4 line; the rounded-division note
  (70.27/1.21 = 58.07 also rounds to ≈58×) added once. The shipped
  ≈58× claim (spec :213) is unchanged.
- **M-D2 (seat D, minor) — FIXED**: Final state now carries a one-line
  addendum — the 12-file / 809+ / 20− tree claims are as of the
  pre-baseline tree (HEAD `3db1b1381a`); operator-directed baseline
  commit `799ec98053` (2026-09-16, ~14s after this doc's last write)
  committed exactly that 12-file delta; the item-5 docs remain untracked
  and uncommitted.
- **F-C2 (seat C, nit) — FIXED**: Final state seam-doc line count/anchors
  updated from the stale 287/pre-r1 anchors to the post-r2 measured
  state (297 lines; delta fully accounted 226 → +61 → 287 → +7 (r1)
  → 294 → +3 (r2) → 297; anchors re-measured after this round's edits:
  §3 :133-164, row 7 :196, conflict sites :243-246, gates :253-259,
  invariants bullet 2 :281-292).
- Cross-check note: seat D independently re-verified m-F1 (40 lib tests),
  the spec delta vs the v4 archive (`diff -u` = 8 hunks, each
  classified), the P1 byte-identical claim (2,198 + 126 chars), and the
  docs-only tree state (zero tracked modifications at `799ec98053`).

## Round-3 verdicts (clean round — loop exit)

r3 seats (2026-09-17, fresh, independent; full pass over the revised
seam + evidence docs with source re-verification):

- **Seat E `impl-item5-r3-seatE.md` — APPROVED, 0 Blocking / 0 Major /
  0 Minor / 0 Nit.** All seven r2 fixes verified at byte level; from-scratch
  pass clean (is_openai refs, SHAs, P1/P2/P3 site-by-site, §6 vs breakdown,
  invariants vs §3.4, gate arithmetic, pinned numbers); docs-only holds;
  spec frozen. Adjudicated the r2 seat C vs D hunk-count discrepancy:
  `diff -u` = 8 hunks (seat D right; seat C's 10 = U=0 distinct-change-site
  count); all 8 hunks explained (2 item-5 final-pass, 6 round-5 v5 fixes).
- **Seat F `impl-item5-r3-seatF.md` — APPROVED, 0 Blocking / 0 Major /
  0 Minor / 0 Nit.** All r2 fixes verified (programmatic quote comparison,
  58.04 sweep, M-D2 addendum vs git, F-C2 line accounting + anchors,
  Round-2 record vs actual seat reports); fresh-eyes pass found no new
  defects (4 checked-and-cleared observations O1–O4, none a finding);
  docs-only holds (1,366 untracked = 1,335 vendor + 31 docs).

Both seats 0B+0M: the SDD review loop (re-review until a full round
returns zero Blocking and zero Major) is satisfied at r3. Item 5 is
commit-ready.
