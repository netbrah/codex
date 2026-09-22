# apex-xt2.15 — Upstream ratchet f3da3861c5 → 78245b47af

Status: DRAFT v0.3.1 (r1 + r2 + r3 dual seats complete — r1 A
0B/5M/6m/3n, B 0B/5M/4m/2n; r2 A 0B/2M/5m/4n, B 0B/1M/5m/5n; r3 A
0B/0M/4m/0n, B 0B/0M/2m/1n; all findings mapped in §10; r3 minors
fixed as v0.3.1; awaiting execution per §5)
Author: coordinator (charlie-mike campaign, 2026-09-22)

## 1. Purpose

Periodic upstream pull ratchet for the on-prem `codex-combined` fork
(branch `feat/normalize-content-types-vllm`). Merge 129 upstream commits
from `reference/upstream-openai` (f3da3861c5) to 78245b47af, adjudicate
the both-sides semantic surface, re-gate, and rotate both deployment
planes (Mac `codex-combined-v2` + SCS `codex-combined`).

## 2. Scope facts (measured 2026-09-22 at HEAD 645be103f5)

- Upstream range: `f3da3861c5..78245b47af` = **129 commits**, 983 files.
  (Upstream `origin/main` has since advanced to 5c07856e25; the ratchet
  target stays pinned at 78245b47af per the campaign plan — the delta
  rolls into the next ratchet.)
- Fork side: 38 commits since the merge base, 161 files touched.
- Both-sides intersection: **31 files** (bead's earlier "13" is stale).
- Text conflicts (`git merge-tree --write-tree`): **3 files**
  - codex-rs/core/src/tools/handlers/request_permissions.rs
  - codex-rs/core/tests/suite/apply_patch_cli.rs
  - codex-rs/core/tests/suite/snapshots/all__suite__scenarios__astra_async_question_and_answer.snap
  Everything else auto-merges, including the xt2.14 fix file
  (codex-rs/core/src/tools/handlers/apply_patch.rs).
- r1 re-verification (seats A+B): every number above re-derives exactly
  from git (seat A facts-check table, 21/21 PASS-or-noted); seat B
  additionally ran a scratch `git worktree` + `git merge --no-commit` —
  all 31 auto-merges are structurally sound (zero dangling refs to the
  upstream-moved `agent::control`→`agent::types` cluster, zero leftover
  fork-local symbols, signature changes consistent).
- models.json: **zero upstream changes** in range (diff + log both
  empty) → the 14-model fork catalog survives byte-for-byte; all new
  ModelInfo/ToolMessage fields are serde-defaulted/optional so the fork
  rows deserialize (seat B MODELS-JSON). R3 downgraded accordingly.
- Cargo.lock: net delta = workspace-internal dep edges only (e.g.
  `+ "vt100"`, `+ "rand 0.9.3"`); 0 packages added/removed/version-
  changed → SCS offline vendor dir stays valid, no vendor-refresh step
  (seat A fact 19).
- Config schema: fork added no ConfigToml fields (config_toml.rs
  untouched fork-side) → upstream config.schema.json changes auto-merge;
  no `just write-config-schema` step needed (seat A fact 21).
- Toolchain: codex-rs/rust-toolchain.toml = 1.95.0 at both range ends;
  the SCS plane builds on 1.94.1 via the sacred env (§5 recipe) —
  dual-plane record, no new toolchain risk (seat A fact 20).
- Precedent: the previous ratchet (apex-xt2.8) landed as a merge commit
  ("merge upstream main f3da3861c5 (193 commits) into
  feat/normalize-content-types-vllm (apex-xt2.8)") **plus follow-up
  fixup commits** (2fb993185d merge fix, 9ac22bb79c manifest bump).
  Planned shape for this ratchet (r2 N8/A-N2 fix): a single merge
  commit — the pre-commit Mac gate loop (step 4) lands its fixes
  INSIDE it — plus the step-9 ratchet commit (reference/manifest
  advance); a §9.2 post-push hotfix commit only if an SCS gate fails
  (r1 A-F6).

## 3. Fork invariants that MUST survive the merge (semantic checklist)

Authoritative ledger: **FORK-MANIFEST.md is the fork's file-level
MUST-SURVIVE ledger; where this §3 and the manifest disagree, the
manifest wins.** Every manifest MUST-SURVIVE row not named explicitly in
this spec is still in scope for the §4 walk (r1 A-F2 fix).

1. **On-prem provider/catalog**: 14-model bundled catalog
   (models-manager/models.json) incl. the on-prem trio (glm-5.2,
   grok-4.6, qwen3.8-27b) with `multi_agent_version: v2`. r1-verified:
   byte-for-byte survival this ratchet (§2). **Untagged provider view:
   the view itself is out-of-repo operator TOML (manifest item 3); the
   in-repo seam is the three `!is_openai()`-derived
   `ProviderCapabilities` flags in model-provider/src/provider.rs —
   `normalize_content_types`, `flatten_namespace_tools`,
   `apply_patch_function_tool` — all three MUST survive** (r1 A-F2 fix;
   v0.1's "untagged provider view" wording was unmappable and protected
   only 1 of the 3 flags).
2. **v2 agent system**: agent_type-based worker seats, role config
   layer (agent/role.rs — r1-verified 0/0 commits both sides in range,
   seat B m4), spawn/followup/send-message/list handlers; the
   proactive-vs-explicit multi-agent policy text (prompts/src/
   model_messages/multi_agent.rs — r1-verified 0/0 both sides;
   upstream's single-side prompts/src/model_messages.rs +29
   auto-applies). **Build-default seam (r1 A-gap-8): core/src/
   fork_defaults.rs (fork-only) + the
   `|| crate::fork_defaults::proactive_delegation_default()` disjunct
   inside `effective_multi_agent_mode` in core/src/session/
   multi_agents.rs (both-sides) — "upstream wins" would silently flip
   the build default; MUST survive.**
3. **apply_patch compatibility** (xt2.14 + ayl.52 P1–P3/T4 + xt2.11
   A+B; r1 A-gaps 2–7 fix):
   - FunctionApplyPatchHandler accepts Function AND Custom payloads
     (Custom = verbatim, no JSON sniff); matches_kind Function|Custom;
     3-arm patch-text extraction.
   - `apply_patch_payload_command` stays byte-identical to upstream —
     re-verify post-merge against the NEW upstream version (r1:
     currently byte-identical to BOTH f3da3861c5 and 78245b47af, seat A
     fact 10).
   - ayl.52 P1: function-tool spec text (tools/handlers/
     apply_patch_spec.rs + apply_patch_spec_tests.rs — fork-only this
     range, auto-preserved; manifest L113/L174) + the
     `apply_patch_function_tool` provider flag (both-sides; covered by
     §3.1).
   - ayl.52 P2: Add-File leniency (apply-patch/src/streaming_parser.rs
     + streaming_parser_p2_tests.rs + parser.rs — fork-only this range).
   - ayl.52 P3: teachable errors (apply-patch/src/parser.rs;
     core/tools/handlers/apply_patch.rs both-sides; args_parse.rs
     funnel fork-only; manifest L141/L191).
   - ayl.52 T4: integration tests on the renamed non-OpenAI provider
     (core/tests/suite/apply_patch_cli.rs) — TEXT CONFLICT #2; an
     "upstream wins" resolution silently deletes the fork's 138 test
     lines (manifest L117).
   - xt2.11 item A (lenient pre-pass) + item B (repair note as leading
     tool-output line): core/tools/handlers/apply_patch.rs +
     apply_patch_tests.rs + apply_patch_spec_tests.rs (manifest
     L105/L114).
   - Freeform fallback: `apply_patch_tool_type = Some(Freeform)` for
     unknown models (models-manager/src/model_info.rs 1-line fork hunk +
     import; test fallback_model_info_enables_apply_patch_for_unknown_
     models in model_info_tests.rs — BOTH both-sides; auto-merge on
     disjoint lines, seat B P1.4; manifest L38).
4. **Content-type normalization seam** (xt2.10): codex-api
   content_type_compat.rs function_call_output output-array handling;
   the `!is_openai` gate in provider.rs is one of the three flags in
   §3.1.
5. **Tool-args ratchet** (xt2.7 items 2–5) + **TUI snapshot/catalog
   surface** (xt2.13 item B) + **KF/ENV catalogue registry** (xt2.13
   item C alone; docs/xt213-c-kf-env-registry.md).
6. **On-prem mode** (docs/onprem-mode-catalog-spec.md behavior) and the
   TUI model-selection snapshot reflecting the fork catalog (r1: the
   snapshot auto-merges but is STALE — upstream 547c9a1aad switched the
   popup to catalog display_name → MUST regenerate at G7; §4.3).
7. **Accepted upstream deltas (pre-recorded 2026-09-22 per r1 B-m3/n1 —
   NOT invariants; "upstream wins" stands)**:
   - 78d4d983d3: `supports_reasoning_effort_updates: false` added to
     `model_info_from_slug`; reasoning-effort `ConfigurationUpdate`
     items are stripped for every model lacking the flag — ALL 14 fork
     models (none set it), and the `is_openai()` gate means on-prem
     never takes the override path. Recorded in §7; redteam set gains
     an effort-change probe (§6a).
   - 841b5490b2: non-Option `policy_context()` signature change deletes
     the "apply_patch requires an executor cwd" error arm (behavior on
     error paths; upstream test
     intercepted_apply_patch_updates_absolute_target_after_turn_cwd_is_
     removed auto-merges into apply_patch_cli.rs and runs in G2).
   - b244e9bafe/841b5490b2 exec_command.rs process-ID reorder:
     allocation moved after preparation; early-error paths no longer
     release a pre-allocated ID. No fork interaction (fork hunks are
     parse renames); covered by G2; recorded in §7.

## 4. Both-sides semantic adjudication (all 31 files)

### 4.1 Pre-baked conflict resolutions (r1 seat B scratch-merge
verified — the executor MUST NOT pick a side for the 3 conflicts)

- **Conflict #1 — tools/handlers/request_permissions.rs**: take
  UPSTREAM's binding (non-Option `policy_context()` per 841b5490b2) +
  keep the FORK's parse rename. Only compile-valid option: post-merge
  `FileSystemSandboxContext::policy_context()` returns
  `FileSystemSandboxPolicyContext` (non-Option; file-system/src/lib.rs
  L449 in the merged tree), so the fork side's `.ok_or_else(...)` does
  not compile. Merged conflict region (scratch L83-93):
  ```rust
      let context = sandbox_context.policy_context();
      let mut arguments: Value = parse_arguments("request_permissions", &arguments)?;
      resolve_permission_path_strings(&mut arguments, &context)?;
      let mut args: RequestPermissionsArgs =
          serde_json::from_value(arguments).map_err(|err| {
              FunctionCallError::RespondToModel(format!(
                  "failed to parse arguments for request_permissions: {err}"
              ))
          })?;
  ```
  The fork's `RequestPermissionsEnvironmentArgs` parse rename (L70) and
  the EOF `mod tests;` + request_permissions_tests.rs auto-merge and
  MUST be kept. §7 row: "upstream non-Option policy context wins
  (forced); fork teachable-error rename preserved."
- **Conflict #2 — core/tests/suite/apply_patch_cli.rs**: two pure
  addition regions; UNION both:
  (a) Imports at old L51: keep upstream's
  `use core_test_support::is_wine_exec_test_environment;` (after
  `assert_regex_match`) AND the fork's
  `use core_test_support::responses::ResponseMock;` (first in the
  `responses::` group, case-sensitive rustfmt order, matching the fork's
  original placement). Dropping either breaks compilation:
  ResponseMock = fork's mount_apply_patch_function_call helper + T4
  tests; is_wine_exec_test_environment = upstream's MXC test.
  (b) After `mount_apply_patch` (old L243): keep BOTH additions —
  upstream's mxc_config_routes_command_and_patch_to_the_windows_executor
  test + the fork's mount_apply_patch_function_call helper (suggest
  fork helper first — it sits where the mount_apply_patch_model_output
  expectations anchor it; no mutual dependency).
  Verify post-resolution: upstream's
  intercepted_apply_patch_updates_absolute_target_after_turn_cwd_is_
  removed (auto-merged ~L1004) and the fork's three T4 tests (auto-
  merge at EOF) both intact; both test sets depend on surviving fork
  seams (function-tool capability, cwd-policy preservation) — run in G2.
- **Conflict #3 — core/tests/suite/snapshots/
  all__suite__scenarios__astra_async_question_and_answer.snap**:
  REGENERATE — do not pick a side. Both sides changed the same two
  tool-hash lines (fork 764538dff3 "ratchet C descriptions": developer
  d4f3e164→f80a5970, collaboration 43f195d9→4fffb2a2; upstream
  0a5b999169/dbf478850f/c5d079470e cluster: developer →9e8e744a,
  collaboration →91f6de0e) — each encodes only its own side's
  spec changes, so NEITHER published hash is the post-merge value.
  Keep upstream's header lines (incl. `assertion_line: 304` — the fork
  never touched tests/suite/scenarios.rs, so upstream's line number is
  authoritative) and upstream's user-message question-reply block
  (upstream-owned rendering). Take either side's hash-line text (identic
  apart from the hashes), then REGENERATE PRE-COMMIT on the Mac (r2
  N1 fix — the correct value must be in the merge commit, so
  regeneration cannot be pinned to the post-push SCS G2): run the
  astra scenario test locally,
  `just test -p codex-core -- astra_async_question_and_answer`, and
  accept the regenerated .snap via `cargo insta` (G7) BEFORE the
  merge commit. If the regenerated run reports a different assertion
  line, let insta update it — don't hand-edit (seat B n2). SCS G2 then
  CONFIRMS no drift: the same test must produce NO new .snap; a new
  snap = cross-plane drift → adjudicate per §9.1 before proceeding.
  Executor checklist for the accepted snap: tool list unchanged,
  question-reply block in new format, hashes updated (both published
  values wrong), assertion_line intact or insta-updated.

### 4.2 Per-file notes (r1-verified structural interactions)

- **spec_plan.rs** (highest-risk structural): upstream's catalog-
  override cluster (cc7591646e, 3e581ebca8, c5d079470e) DELETED the
  fork's local multi_agent_v2_handler (76 lines, old L1414) +
  MULTI_AGENT_V2_NAMESPACE_DESCRIPTION (the deletion resolves to
  3e581ebca8 alone — r2 B-N5 re-derivation; c026e7a622 does not touch
  spec_plan.rs) and moved them to NEW file core/src/tools/
  multi_agent_tool.rs (4-arg, catalog-override aware; const re-
  declared there); add_collaboration_tools now reads overrides
  via ResolvedModelMessages::from_model(...) and the import swapped to
  `use crate::tools::multi_agent_tool::multi_agent_v2_handler;`. The
  override surface further spans prompts/src/model_messages.rs +
  protocol/src/openai_models.rs (ToolMessage) — unlisted files, added
  to the walk below. Verified clean in the scratch merge: merged file
  has the upstream import at L57, NO leftover const/local-fn refs,
  fork's apply_patch_function_tool branch intact (old L1216 region),
  and the fork's `use crate::tools::registry::CoreToolRuntime;` + local
  2-arg handler are gone with zero dangling refs (grep-verified in the
  merged tree). Post-merge do NOT re-add the fork's local handler/const
  "because the fork used to have them" — the import from multi_agent_
  tool is the correct end state. The new override surface is inert for
  fork models (fork models.json has no `messages` key), so the fork's
  V2 description edits (9ffeb7db7d followup_task/wait_agent text)
  survive via the bundled descriptions.
- **apply_patch.rs** (xt2.14 fix file): auto-merge. Fork hunks (old
  L377–449: impl ApplyPatchHandler → run_apply_patch_text) and upstream
  hunks (L37 import; L552–567 execute_verified_patch) are disjoint.
  The fork's run_apply_patch_text delegates to the SAME
  execute_verified_patch upstream modified (fork did not duplicate it) —
  re-verify apply_patch_payload_command byte-shape per §3.3 against the
  NEW upstream version.
- **provider.rs**: upstream = gateway-OAuth plumbing (3d5b66c655,
  4d23af0975 — new trait method gateway_auth_manager() with default
  Ok(None); ConfiguredModelProvider field; create_model_provider
  restructured with early bedrock return), retry centralization
  (d5b29951ac), explicit catalog URLs (888be42a20 — model_catalog_url,
  discovery enable/disable tests); fork = the 3 ProviderCapabilities
  fields + Default + capabilities() (!self.info.is_openai()) + 1 test.
  Zero hunk overlap (upstream old L11/29/179/352/412/428/485/508/531/
  604/917/1304 vs fork old L49/59/390/724; scratch-merge clean). On-
  prem providers have gateway_oauth: None → gateway auth no-ops the
  gateway side; on-prem auth now flows through upstream's new
  api_auth()/api_auth_for_scope() composition — covered by G1 build +
  mock triad (§6a); the 3 capability fields are not visible to upstream
  code, no semantic coupling.
- **tools/handlers/mod.rs**: upstream file_system_sandbox_policy_
  context_for_cwd → non-Option + apply_granted_turn_permissions adapted
  (841b5490b2); fork moved parse_arguments to the new args_parse.rs
  module (2-arg, tool-name attribution) + FunctionApplyPatchHandler
  export. Verified in scratch: merged mod.rs has the non-Option fn AND
  the args_parse re-export; every call site in the merged tree uses the
  2-arg form or the non-Option binding (zero leftovers). Do not
  resurrect the old 1-arg parse_arguments.
- **multi_agents_v2/{spawn,followup_task,send_message,list_agents}.rs,
  multi_agents{,_spec}.rs, multi_agents/spawn.rs (v1)**: upstream =
  import moves agent::control→agent::types (c026e7a622), Handler gains
  description_override + 2-arg new (c5d079470e), ListedAgent
  restructure (c6f5d9e9b5), static "Only call this tool for a concrete,
  bounded subtask…" line DELETED from the default description
  (16f49ccd7f/c5d079470e — model-visible behavior delta, covered by
  redteam acceptance); fork = 1-line parse renames (d96703897f),
  deny_unknown_fields (a81aa1d3a0), followup_task/wait_agent
  description text edits (9ffeb7db7d). All disjoint; usage-hint flow
  (resolve_usage_hints → SpawnAgentOptions.multi_agent_v2_usage_hints)
  byte-identical on both sides — fork role-seat semantics preserved.
- **model_info.rs**: same function, disjoint lines (gap L129–136) →
  auto-merge keeps both fields (scratch-verified:
  supports_reasoning_effort_updates: false AND
  apply_patch_tool_type: Some(Freeform) both present).
- **client.rs**: both-sides; carries the flatten_namespace_tools seam
  (§3.1) — line-by-line read in P1. **code_mode/wait_handler.rs,
  session/multi_agents.rs, unified_exec/exec_command.rs,
  config_tests.rs, core/src/lib.rs**: auto-merge, no r1-flagged
  interaction (exec_command.rs ID reorder = §3.7 pre-record).

### 4.3 The 31-file list (complete and path-verified in r2 — every
entry checked against the comm of both file lists; r1 A-F1 + r2 N2
fixes)

P1 (fork feature core — line-by-line both-sides read):
- tools/handlers/multi_agents_v2/{spawn,followup_task,send_message,
  list_agents}.rs
- tools/handlers/multi_agents_spec.rs, multi_agents_spec_tests.rs,
  multi_agents_tests.rs (r2 N2: the v0.2 brace expansion added a
  phantom 4th file, handlers/multi_agents.rs — exists but NOT both-
  sides)
- session/multi_agents.rs (carries the fork_defaults disjunct — §3.2)
- tools/handlers/apply_patch.rs (xt2.14 fix vs upstream evolution)
- tools/handlers/request_permissions.rs (text conflict #1)
- tools/handlers/mod.rs, spec_plan.rs, code_mode/wait_handler.rs
- tools/handlers/view_image.rs (r2 N2: the both-sides file is the
  handlers/ one — up +25/−11, fork +2/−1; the suite-level file of the
  same name is unchanged on both sides and NOT in the set)
- client.rs, model-provider/src/provider.rs
- models-manager/{model_info,model_info_tests}.rs (Freeform fallback —
  §3.3)
- [A-F1 additions] core/src/lib.rs, tools/handlers/unified_exec/
  exec_command.rs, tools/handlers/multi_agents/spawn.rs (v1),
  core/src/config/config_tests.rs (upstream +123/−12, fork +75/−0;
  r2 N2: the v0.2 path core/tests/suite/config_tests.rs does not
  exist) — all auto-merge, all manifest MUST-SURVIVE.

P2 (tests/snapshots):
- core/tests/suite/apply_patch_cli.rs (text conflict #2)
- core/tests/suite/{mod,step_settings}.rs
- [B-m1 path fix] core/src/tools/handlers/{unified_exec_tests,
  request_plugin_install_tests}.rs — these live under
  core/src/tools/handlers/, NOT core/tests/suite/ (the suite-level
  files of those names are unchanged on both sides)
- tui snapshots: model_selection_popup.snap — auto-merges but STALE
  (upstream 547c9a1aad display_name switch; merged file keeps lowercase
  slug rows 6-8 while post-merge code renders GPT-5.2/GLM-5.2/Grok-4.6
  → TUI test fails until regenerated): MUST regenerate + review at G7;
  astra scenario snap (text conflict #3 — core suite, not TUI)
- tui/app/tests/recap_generation_tests.rs
- app-server/tests/suite/v2/turn_start.rs

Also walk (unlisted in v0.1; r1 A-F2/B-M5): core/src/tools/
multi_agent_tool.rs (new upstream — §4.2 note), prompts/src/
model_messages.rs (single-side +29 override hooks), protocol/src/
openai_models.rs (ToolMessage fields), core/src/fork_defaults.rs
(fork-only, §3.2).

Adjudication rule per file: upstream semantics win UNLESS it would
break a fork invariant (§3 or a FORK-MANIFEST.md MUST-SURVIVE row);
every kept-fork-hunk is recorded in §7 with rationale. No silent 3-way
"ours" or "theirs" resolution.

## 5. Merge mechanics (order fixed per r1 A-F3)

Push happens AFTER the Mac fast gates and BEFORE the SCS gates (the SCS
gates need the merge in-tree; their failure path is §9, not a silent
state). v0.1's "gates green at push time" was impossible under the only
feasible ordering — corrected here.

1. On feat/normalize-content-types-vllm from HEAD 645be103f5:
   `git merge --no-ff --no-commit 78245b47af`.
2. Resolve the 3 text conflicts per §4.1 (never pick a side); walk the
   §4.3 list (incl. the "also walk" files) for semantic drift; record
   every adjudication in §7 as it happens.
   Sub-steps: (a) diff the FORK-MANIFEST.md file table against the
   §4.3 31-set + also-walk files; record any delta in §7 (r1 A-F2
   operational step). (b) Conflict #3: after the text resolution,
   regenerate the astra snap locally pre-commit per §4.1 (the correct
   hash must be in the merge commit).
3. Mac compile sanity (JOBS=2): `cargo build -p codex-cli -p codex-exec`
   (same scope as SCS G1; run early to catch compile breaks before any
   push).
4. Mac fast gates G6 + G7/G7b (clippy; 27-test anchor; astra snap
   regen per §4.1; model_selection_popup insta regen).
5. Commit the merge: a SINGLE merge commit (r2 N8/A-N2: the step-4
   gate loop runs pre-commit, so its fixes land INSIDE the merge
   commit — no separate Mac-fixup; the xt2.8 manifest-bump follow-up is
   the step-9 ratchet commit; xt2.8's code-fix follow-up has no
   planned counterpart here — the §9.2 post-push hotfix is the only
   other commit path). Message:
   `merge upstream main 78245b47af (129 commits) into
   feat/normalize-content-types-vllm (apex-xt2.15)` + body listing the
   §7 adjudication highlights.
   Bazel (r1 A-F13): the merge carries 13 Cargo.toml + Cargo.lock
   changes → run `just bazel-lock-update`; if MODULE.bazel.lock
   changes, include it in the merge commit (AGENTS.md lockfile-drift
   rule). §2 fact: 0 package-level changes, so expect no SCS vendor
   impact (no vendor-refresh step).
6. Push to BOTH remotes (fork/netbrah github + APEX bitbucket — NEVER
   origin/openai); `git ls-remote` verify BOTH the fetch URL and the
   push URL (one remote name, two URLs). Mechanics per FORK-MANIFEST
   ratchet item 6 (L234–236): push to GitHub netbrah/codex via the
   full URL, and to bitbucket APEX/codex via the `fork` remote pushurl.
7. SCS ff-sync: `git pull --ff-only` (SCS remote `origin` = APEX
   bitbucket); verify the uncommitted `M codex-rs/.cargo/config.toml`
   is preserved — never commit it.
8. SCS gates G1–G5 + G2b/G4b (§6).
9. ONLY after all gates green: advance `reference/upstream-openai` to
   78245b47af and update FORK-MANIFEST.md ("Last verified" + ratchet
   row, incl. the deferred rows in docs/xt213-c-kf-env-registry.md §6:
   8 pending files, picker .snap collision vs 547c9a1aad, :122 stale
   row) + fold the §7 record
   into this spec. Reference/manifest advance happens at ratchet time
   only — never earlier.

SCS env recipe (r1 A-F5 — specified, not referenced):
- Every SCS cargo invocation uses the sacred env, verbatim from the
  xt2.14 SCS release build log header (/x/eng/ai_engineering/APEX/codex/
  scs-release-rebuild.log): CARGO_NET_OFFLINE=1, RUSTUP_TOOLCHAIN=1.94.1,
  CARGO_BUILD_JOBS (4 for builds), CARGO_INCREMENTAL=0,
  RUST_MIN_STACK=67108864, CODEX_SKIP_BWRAP_BUILD=1, RUSTY_V8_ARCHIVE
  + RUSTY_V8_SRC_BINDING_PATH (exact SCS values in the xt2.14 log
  header; also docs/xt213-b-fcat-spec.md:318–319 and docs/xt213-n1-
  helper-spec.md:170). Omitting CARGO_INCREMENTAL=0 or the RUSTY_V8_*
  pair causes surprise rebuilds.
- Toolchain reconciliation: repo pin = 1.95.0 (rust-toolchain.toml,
  unchanged in range); the SCS plane builds on 1.94.1 explicitly via
  RUSTUP_TOOLCHAIN=1.94.1. The SCS gates are the 1.94.1 record, the Mac
  gates the 1.95.0 record — BOTH planes must be green (dual-plane
  record, same as xt2.14).
- SCS background runs: `setsid <cmd> < /dev/null > log 2>&1 &` (bare
  nohup & holds the ssh channel); `timeout` exists on SCS; multiple
  positional test filters make libtest filter to the intersection
  (vacuous green) — use ONE filter per invocation.

## 6. Gate plan (order: Mac build + G6 + G7 → push + SCS sync → SCS G1–G5)

G1 (SCS): two-package debug build sanity `cargo build -p codex-cli
   -p codex-exec` (catches compile breaks in the 31-file surface fast).
G2 (SCS): `cargo test -p codex-core` full (changed-crate anchor; the
   astra scenario run here is the post-push drift CONFIRMATION — it
   must produce NO new .snap, because the correct hash was regenerated
   pre-commit on the Mac per §4.1; a new snap = cross-plane drift,
   adjudicate per §9.1 before proceeding). Adjudicate any failure vs
   the HEAD control per the xt2.14 protocol (HEAD control run +
   isolation re-runs before blaming the merge).
G2b (SCS, r1 A-F4 add): `cargo test -p codex-model-provider
   -p codex-models-manager` (both-sides: the 3 capability flags +
   Freeform fallback test).
G3 (SCS): `cargo test -p codex-exec` full suite (78/78 anchor).
G4 (SCS): `cargo test -p codex-api` — with the pre-existing 7-TMT
   adjudication (§8 R4; expected same-set TMTs, not new failures).
G4b (SCS, r1 A-F4 add): `cargo test -p codex-app-server -- turn_start`
   (the both-sides v2 anchor; full app-server run optional under load).
G5 (SCS): `rustfmt --check` on all both-sides files + owned files.
G6 (Mac): clippy, scope expanded (r1 A-F14 — the merge carries fork-
   owned lines in the other crates; core-only scope would leave them
   ungated): `just fix -p codex-core && just fix -p codex-model-provider
   && just fix -p codex-models-manager && just fix -p codex-tui`, then
   `just test -p codex-core -- tools::handlers::apply_patch::tests`
   (27-test module anchor), `just test -p codex-api --
   content_type_compat`, and the astra scenario test `just test -p
   codex-core -- astra_async_question_and_answer` (its .snap
   acceptance IS the pre-commit regeneration — §4.1).
G7 (Mac): TUI snapshot review for the 2 both-sides snapshots
   (`cargo insta` — model_selection_popup MUST-regen per §4.3; astra
   snap = the pre-commit regeneration per §4.1, reviewed here).
G7b (Mac, r1 A-F4 add): `just test -p codex-tui` full (fork catalog
   rows appear in multiple TUI snapshots — the xt2.13 item-B surface).
G8 (both planes): release rebuild + rotate + full redteam acceptance
   (§6a) — the binaries shipping to Mac/SCS must be post-merge.

### 6a. Post-merge rotation + acceptance (both planes)

SCS:
- `cargo build --release -p codex-cli --bin codex` (full sacred env
  recipe, CARGO_BUILD_JOBS=4).
- Mock triad on the release binary (xt214-harness, EXE_OVERRIDE
  wrapper `exe-release.sh` → `codex exec "$@"`): custom+test /
  function+test / builtin — all rc=0 + wire-shape + task_complete.
- Live: `codex-harness run.py --bin target/release/codex cp-01 cp-02`.
- Rotate: mv old bin/codex-combined → .bak-<prevdate>-<ts>, then the
  xt2.14 strip procedure: cp the new binary → tmp, `strip
  --strip-unneeded` the tmp copy, mv the stripped tmp →
  bin/codex-combined (mv-from-new-file = fresh inode; NEVER cp-
  overwrite an existing signed/rotated deploy file in place — see §8
  R1).
Mac:
- `CARGO_BUILD_JOBS=2 cargo build --release -p codex-cli --bin codex`.
- Cutover: timestamped backup of old v2 → codex-bin-backups/; back up
  the pre-existing ~/bin/codex-combined slot binary to
  codex-bin-backups/ (timestamped) BEFORE overwriting it (r2 N9);
  old v2 → ~/bin/codex-combined rollback slot, new →
  ~/bin/codex-combined-v2 via FRESH INODE (mv away the old path first,
  then cp; never cp-overwrite — §8 R1).
- Live acceptance: cp-01 + cp-02 via /tmp/codex-harness-mac
  (v0.3.3 bidirectional wiretap) + the xt2.10 MCP round-trip probe
  (glm, codegraph_explore, no-400, session recorded).
- Redteam set extension (r1 B-m3): **effort-change probe** — mid-turn
  reasoning-effort update on a fork model; post-merge expectation per
  78d4d983d3 = the ConfigurationUpdate is silently dropped (on-prem
  never takes the override path). The probe records the wire shape; any
  divergence is a ratchet blocker. Add the case to codex-harness.

## 7. Adjudication record (r1 pre-records + filled during execution)

| file / delta | resolution | rationale |
|---|---|---|
| conflict #1 request_permissions.rs | upstream non-Option binding + fork parse rename | §4.1 pre-baked; forced by 841b5490b2 |
| conflict #2 apply_patch_cli.rs | UNION imports + both post-mount additions | §4.1 pre-baked; T4's 138 lines survive (manifest L117) |
| conflict #3 astra snap | regenerate PRE-COMMIT on the Mac (§4.1); G2 confirms no drift; upstream header | §4.1 pre-baked; neither side's hash valid; r2 N1: the value must be in the merge commit |
| models.json | no change expected | §2 fact (0 upstream changes); if the diff is non-empty: STOP + re-adjudicate |
| core/src/tools/multi_agent_tool.rs (new) | take upstream as-is | fork's local handler/const moved here by 3e581ebca8 (catalog-override cluster); no fork edits |
| prompts/model_messages/multi_agent.rs | unchanged | 0/0 commits both sides (r1 B-m4) |
| 78d4d983d3 ConfigurationUpdate strip | upstream wins (all 14 fork models) | §3.7 pre-record; effort probe in §6a |
| 841b5490b2 cwd-error-arm drop | upstream wins | §3.7 pre-record; upstream test auto-merged, runs in G2 |
| b244e9bafe exec_command ID reorder | upstream wins | §3.7 pre-record; no fork interaction; G2 covers |
| (one row per kept-fork-hunk found during the §4 walk) | | |

## 8. Risks

R1. **Binary rotation footgun (learned 2026-09-22)**: cp-overwriting an
    existing code-signed binary in place (same inode) produced a kernel
    "Taskgated Invalid Signature" SIGKILL on the Mac (16KB-page kernel);
    byte-identical fresh-inode copies run fine. All rotations in §6a use
    mv-away-then-cp-fresh-inode. SCS x86-64 showed no such behavior, but
    the same procedure is used for symmetry.
R2. **v2 agent semantic drift**: the upstream range carries the
    agent::control→agent::types cluster move (c026e7a622) + the new
    multi_agent_tool.rs override surface; of the 31 both-sides files,
    3 conflict + 28 auto-merge, all verified structurally clean in the
    r1 scratch merge (zero dangling refs to moved types, zero leftover
    fork-local symbols, signature changes consistent). Mitigation:
    §4.2 per-file notes + §4.3 P1 line-by-line read + integration tests
    (multi_agents_tests.rs auto-merges; runs in G2).
R3. **Catalog regeneration**: VERIFIED NOT APPLICABLE at 78245b47af —
    zero upstream models.json changes in range; the 14-row fork catalog
    survives byte-for-byte (r1 B-m2). Standing re-check each future
    ratchet; the §7 STOP row enforces it.
R4. **codex-api 7 TMTs (files::upload_* + realtime_websocket_tls)**:
    Deterministic 60s TMTs on the Mac at HEAD 645be103f5 (isolation
    re-run confirmed; fork never touched files.rs). SCS control at the
    SAME HEAD: all 6 upload tests PASS in 0.33s
    (/x/eng/ai_engineering/APEX/scs-api-iso2.log, 2026-09-22) → the
    upload TMTs are Mac-environmental (wiremock localhost under this
    16KB-page kernel / load), NOT code or fork regressions. Gate G4
    counts them as TMT-not-fail on the Mac; SCS run is the green
    record. realtime_websocket_tls control recorded in scs-api-iso3.log.
    RESOLVED 2026-09-22: SCS control at the same HEAD passes ALL 7 —
    6/6 upload (0.33s) + realtime_tls (0.26s), both exit 0. All 7 TMTs
    are Mac-environmental; the gate green record for codex-api is the
    SCS run, Mac TMTs recorded as environmental (no P2 bead — fork code
    exonerated on both planes).
R5. **129-commit behavioral surface**: upstream changes outside the 31
    files can still shift behavior (prompts, compaction, sandbox).
    Mitigation: the full redteam acceptance set (§6a) is the on-prem
    behavioral anchor; any cp-01/cp-02/MCP-probe/effort-probe
    regression is a ratchet blocker, not a follow-up.
R6. **Mac resource contention**: operator workloads share the Mac;
    build at JOBS=2, gates scoped, SCS preferred for heavy gates.

## 9. Rollback (order fixed per r1 A-F3)

The push (step 6) happens only AFTER the Mac fast gates are green
(build + G6 + G7/G7b). The SCS gates then run against already-pushed
state; their failure path is:
1. Diagnose per the xt2.14 protocol (HEAD control / isolation re-runs
   before blaming the merge).
2. Fix forward on the branch (hotfix commit) → push both remotes → SCS
   ff-sync → re-run the failed gate.
3. If fix-forward is not feasible within the session:
   `git revert -m 1 <merge-sha>` (default — NEVER force-push without
   explicit operator approval), push both remotes, SCS ff-sync, re-gate
   next session.
4. Binaries: the G8 rotation happens only AFTER gates green, so gate
   failures need no binary rollback. Pre-rotation binaries are
   preserved on both planes (SCS .bak-*, Mac codex-bin-backups/*); a
   post-rotation rollback = re-link the previous binary (fresh inode,
   R1).
5. `git reset --hard 645be103f5` remains the local tree restore only
   (pre-merge state); it is NOT a published rollback.

## 10. Review log

- v0.1 draft: coordinator, 2026-09-22 (awaiting dual-seat review).
- r1 SEAT A (2026-09-22): **0B 5M 6m 3n** — /tmp/xt215-spec-seat-A-
  verdict.md. 21/21 facts-check PASS-or-noted (129/983/38/161/31/3
  conflicts exact; origin/main tip; sha anchors; 27-test anchor; R4;
  Cargo.lock/vendor; toolchain; schema).
- r1 SEAT B (2026-09-22): **0B 5M 4m 2n** — /tmp/xt215-spec-seat-B-
  verdict.md. Scratch merge of all 31 auto-merges verified structurally
  clean; per-file interaction maps (P1.1–P1.5 + P2 table); MODELS-JSON
  verification; CONFLICT-RESOLUTION-PREBAKED for all 3 conflicts.
- v0.2 (2026-09-22) — r1 finding → resolution map:
  - A-F1 (§4 list short by 4): v0.2 added the 4 files (core/src/lib.rs,
    multi_agents/spawn.rs, unified_exec/exec_command.rs,
    config_tests.rs); v0.3 (r2 N2) verified every entry against the
    comm: config_tests.rs corrected to core/src/config/config_tests.rs
    (the v0.2 path did not exist), view_image.rs corrected to
    core/src/tools/handlers/ (P1), the phantom multi_agents.rs brace
    removed — §4.3 now names exactly the 31 at correct paths.
  - A-F2 (manifest not normative; untagged view unmappable): §3 header
    names FORK-MANIFEST.md as the authoritative ledger with the manifest-
    wins rule; §3.1 reworded to the three named !is_openai() provider.
    rs flags; all 10 seat-A coverage-gap items folded into §3
    (flatten-namespace seam, ayl.52 P1–P3/T4, xt2.11 A+B, Freeform
    fallback, fork_defaults disjunct, untagged-view mapping).
  - A-F3 (push/gates ordering contradiction): §5 reordered (Mac fast
    gates → push → SCS gates) with the "gates green at push time" claim
    explicitly retracted; §9 specifies the post-push failure path
    (fix-forward → revert; force-push only on explicit operator
    approval).
  - A-F4 (missing gates): G2b (model-provider + models-manager), G4b
    (app-server turn_start), G7b (codex-tui full) added.
  - A-F5 (sacred recipe + toolchain unspecified): §5 specifies the env
    recipe (verbatim source = xt2.14 SCS build log header) + the
    1.94.1/1.95.0 dual-plane toolchain record + SCS background-run
    traps.
  - A-F10 (rotation under-specified): (b) ff-sync command + config.toml
    check resolved in §5 step 7 (v0.2); (a)/(c) — "stripped" undefined
    + Mac slot backup — resolved in v0.3 §6a (xt2.14 strip procedure +
    pre-overwrite slot backup).
  - A minors/nits: xt2.8 commit shape restated (§2 + §5 step 5, v0.3);
    §4.3 path fixes (unified_exec_tests/request_plugin_install_tests
    under handlers/; astra snap = core suite, not TUI; view_image +
    config_tests paths in v0.3); R2 reworded (3 conflict + 28
    auto-merge, scratch-verified); ayl.52 surface + 841b5490b2/
    b244e9bafe deltas into §3.7/§4.2/§7; clippy scope expanded (G6,
    A-F14); bazel-lock step (§5 step 5, 0-package-change fact in §2).
  - B-M1: conflict #2 pre-baked UNION resolution in §4.1.
  - B-M2: conflict #1 pre-baked (only compile-valid option, code block)
    in §4.1.
  - B-M3: conflict #3 "regenerate, do not pick a side" + executor
    checklist in §4.1.
  - B-M4: model_selection_popup "auto-merges but STALE → MUST
    regenerate at G7" in §4.3 (+ §3.6).
  - B-M5: spec_plan.rs per-file note + moved-file map (multi_agent_
    tool.rs, prompts/model_messages.rs, openai_models.rs) in §4.2/§4.3.
  - B-m1: two test-path corrections in §4.3. B-m2: R3 downgraded to
    verified-not-applicable + standing re-check. B-m3: 78d4d983d3
    pre-record (§3.7) + effort-change probe (§6a) + §7 row. B-m4: 0/0
    commit verifications recorded (§3.2). B-n1: exec_command ID reorder
    pre-record (§3.7/§7). B-n2: insta assertion_line handling rule in
    §4.1.
- r2 SEAT A (2026-09-22): **0B 2M 5m 4n** — /tmp/xt215-spec-seat-A-r2-
  verdict.md. 21/27 r1 findings fully resolved, 4 carried (A-F1
  MAJOR — misapplied paths; A-F7/A-F9/A-F10 MINOR residuals); NEW
  MAJOR: the astra hash is only producible post-push (G2/SCS) while
  its acceptance gate (G7/Mac) ran pre-commit — no step lands the
  regenerated .snap.
- r2 SEAT B (2026-09-22): **0B 1M 5m 5n** — /tmp/xt215-spec-seat-B-r2-
  verdict.md. All 12 r1 MAJORs verified resolved (pre-bake operative
  details byte-intact); the one MAJOR = the same astra ordering hole
  (found independently); MINORs = view_image path + phantom
  multi_agents.rs brace (N2), 13-vs-11 Cargo.toml (N3), xt2.13 item
  attribution (N4), RUSTY_V8 unnamed (N7), rotation residues (N9).
- v0.3 (2026-09-22) — r2 finding → resolution map (M = fix label, not
  severity; r2 severities: M1 MAJOR, M2 MAJOR-carried A-F1 + MINOR
  B-N2, M3–M5 MINOR):
  - M1 (A-N1/B-N1 astra ordering): §4.1 conflict #3 now regenerates
    PRE-COMMIT on the Mac (G6 astra run + G7 insta accept before the
    merge commit); G2 = post-push no-drift confirmation; §5 steps
    2(b)/4, §6 G2/G6/G7, §7 row all re-wired.
  - M2 (A-F1/B-N2 paths): §4.3 config_tests.rs → core/src/config/;
    view_image.rs → P1 handlers/; suite brace shrunk to {mod,
    step_settings}; phantom multi_agents.rs brace removed (family
    enumerated).
  - M3 (A-F9/B-N4): §3.5 attribution split (item B = TUI surface,
    item C = registry).
  - M4 (B-N7): RUSTY_V8_ARCHIVE + RUSTY_V8_SRC_BINDING_PATH named with
    xt213 doc cites.
  - M5 (A-F10/B-N9): §6a SCS strip procedure defined (cp → tmp, strip
    --strip-unneeded, mv) + Mac pre-overwrite slot backup.
  - Nits: commit shape restated (A-N2/B-N8); G6 cite A-F11→A-F14
    (A-N3/B-N6); 13 Cargo.toml (A-N4/B-N3); spec_plan deletion →
    3e581ebca8 (§4.2 + §7 row, B-N5); §3.3 heading gaps 2–7 (B-N6);
    §10 "§5.5" → "§5 step 5" (A-N5); item-C §6 → full doc cite (A-N6);
    G1 label + G6 command form (A-N7); dual-push cites FORK-MANIFEST
    ratchet item 6 (B-N10); A-F10 map entry added (B-N11).
- Awaiting r3 (confirmation round on the v0.3 fixes, scoped: per-fix
- r3 SEAT A (2026-09-22): **0B 0M 4m 0n** — /tmp/xt215-spec-seat-A-r3-
  verdict.md. All five v0.3 fixes verified present, correct, and
  genuinely resolving their r2 findings at every cited location; the
  30-hunk diff is fully accounted for except two flagged lines; the
  31-list re-derives byte-identically from git and §4.3 names exactly
  those 31 with no phantoms.
- r3 SEAT B (2026-09-22): **0B 0M 2m 1n** — /tmp/xt215-spec-seat-B-r3-
  verdict.md. All five r2 majors confirmed genuinely resolved (M1
  pre-commit rewiring verified across §4.1/§5/§6/§7 with no stale
  wiring; M2 paths verified against the tree plus an independently
  re-derived comm matching the canonical 31 byte-for-byte; M3/M4/M5
  present with every doc cite verified in-repo); all ten nits
  verified; 28 of 30 diff hunks map cleanly.
- v0.3.1 (2026-09-22) — r3 minor fixes (bookkeeping only; exact fixes
  specified by both r3 seats):
  - F1/3.1: §6a Mac duplicated `- CARGO_BUILD_JOBS=2 ...` build line
    removed.
  - F2/3.2: §5 step 2(a) map entry added (Nits list above).
  - F3/3.3: M1 cite "A-M2" → "A-N1" (seat A r2 ID).
  - F4: M-label header annotated with r2 severities (M = fix label,
    not severity).
- Awaiting execution per §5 (ratchet merge 78245b47af → gates → push
  → reference advance).
    step 2(a) manifest-diff operational step added to §5 step 2 (r1 A-F2
    residual note made explicit; map entry per r3 SEAT A F2 / SEAT B 3.2).
