# apex-xt2.13 item C — KF/ENV registry (known-failure + environment-bound test rows)

Status: DRAFT v1 (item C of apex-xt2.13; closes the bead)
Branch: feat/normalize-content-types-vllm
Audience: any future full-suite run (campaign gates, ratchet re-runs, release-build
acceptance) that needs to separate **new regressions** from **known rows** without
re-deriving the RUN-2 / item A / item B forensics.

## 0. Purpose and rules of use

- **KF** = known flake / known deterministic pre-existing failure: measured, root-caused
  or class-adjudicated in this campaign; tracked here (and in the bead note), never
  deleted, never "fixed" by loosening an unrelated assertion.
- **ENV** = environment-bound: the test is correct and passes where the environment
  has the capability; it fails on the SCS build plane for a missing capability.
- A failed name **in this registry** = known; record the row id in the run note.
- A failed name **not in this registry** = NEW: classify (N2 / KF / ENV / other) and
  adjudicate per §7 escalation rules before it is ever added here.
- KF entries require an isolate re-run as evidence (green isolated = flake; red
  isolated = deterministic → new item/bead, not a KF row); the exec-process
  deadline family (K3–K6) is the exception — it stays KF even when red in isolate
  (item A spec §5.5 — see §7 rule 3). Rows registered as pre-existing before
  the campaign's isolate practice (K1, K2, K5 — RUN-2 catalogue / item A gate-5
  KF registry) are exempt from the isolate re-run requirement; their basis is
  the RUN-2 catalogue + the bead `apex-xt2.13` KF registry notes.
- All rows below were measured on SCS (`10.234.218.97`, `/x/eng/ai_engineering/APEX/codex`,
  toolchain 1.94.1-x86_64) — the build plane — during items A/B of this bead.

Provenance:
- RUN-2 catalogue — bead `apex-xt2.13` note 2026-09-20 (214 failed tests total across 14 failing targets
  (189 in app-server/core/TUI suites; 25 in exec-server/http-client/skills/
  cli-daemon/exec/responses_headers/ext-agent/linux-sandbox/v8-poc)); per-test data `/tmp/r2_classification.json`,
  `/tmp/r2_panic_locs.json`.
- Item A (commit `3cd2efebaf`): spec `docs/xt213-n1-helper-spec.md` (v3.1), report
  `/tmp/xt213-itema-tdd-report.md`, residue table `/tmp/xt213_itema_residue_table.txt`
  (124 rows), gate logs `/tmp/xt213_itema_gate{1..5}.log` + `/tmp/xt213_itema_g4*.log`.
- Item B (commit `51e1b313de`): spec `docs/xt213-b-fcat-spec.md` (v1.3.1), report
  `/tmp/xt213-itemb-tdd-report.md`, gate logs `/tmp/xt213_itemb_gate*.log`, probe logs
  `/tmp/xt213_itemb_probe3_s{1,2}.log`.

## 1. ENV rows (environment-bound)

| id | test | suite | failure signature | env requirement | evidence |
|---|---|---|---|---|---|
| E1 | `app::tests::background_task_defaults_tests::review_regression_agents_overview_creation_is_fresh_but_returning_is_not` (`codex-rs/tui/src/app/tests/background_task_defaults_tests.rs:46`) | codex-tui `--lib` (item B row 15) | crossterm `reader source not set` (crossterm `read.rs:61:30`) | TTY | item B G3 full-lib run (`/tmp/xt213_itemb_gate3.log`): red on SCS (headless); red at G3 (failed-set member of the predicted 4872/2/3 variant) |
| E2 | `responses_stream_includes_turn_metadata_header_for_git_workspace_e2e` (`codex-rs/core/tests/responses_headers.rs`, test def :457; panic assert :691:9) | codex-core `--test responses_headers` | git-origin URL assert: workspace origin `https://repomirror-rtp.eng.netapp.com/github/openai/codex` vs expected `https://github.com/openai/codex` (SCS git remotes route via repomirror; NOT TLS/real-cert) | network + git workspace with an un-mirrored github.com origin | suite = 4 passed / 1 failed at BOTH RUN-2 and item A G4D (`/tmp/xt213_itema_g4d_headers.log`); baseline unchanged through the campaign (expected per item A spec gate 4) |

ENV classes named in item A spec §7 (from the RUN-2 catalogue) beyond the two
measured rows above: exec-server load, TLS real-cert, skills walk, crossterm TTY,
air-gap, bwrap `--argv0`, v8 ptrcomp. Where a row from one of these classes first
re-appears in a measured run, add it to the table with its test name + signature
instead of carrying class names indefinitely.

## 2. KF rows — `codex-core --lib`

Exec-process deadline family + previously-registered lib names. Measured at item A
gate 3B (2581 passed / 5 failed / 1 ignored) and gate 4C (same family, one
within-family swap); the union of both `--lib` runs = 6 unique names.

| id | test | class | signature | status at measurement |
|---|---|---|---|---|
| K1 | `session::tests::user_shell_commands_do_not_inherit_managed_network_proxy` | KF-registered (pre-existing; module prefix corrected in item A gate 5) | — | red (both runs) |
| K2 | `session::turn::tests::post_sampling_token_estimate_is_disabled_by_always_on_sinks` | KF-registered (pre-existing; module prefix corrected in item A gate 5) | — | red (both runs) |
| K3 | `unified_exec::tests::unified_exec_timeouts` | exec-process deadline | `mod_tests.rs:474` "expected process id" | red in gate 3B, 4C, AND isolate re-run g5f (`/tmp/xt213_itema_iso_g5f*.log`) → KF per spec §5.5 |
| K4 | `unified_exec::tests::unified_exec_persists_across_requests` | exec-process deadline | — | red in gate 3B; green in gate 4C (within-family flake — in-family, passed this round per bead note) |
| K5 | `unified_exec::tests::reusing_completed_process_returns_unknown_process` | exec-process deadline (NEW in RUN-2; added to KF registry in item A gate 5) | — | red (both runs) |
| K6 | `unified_exec::tests::multi_unified_exec_sessions` | exec-process deadline (new at gate 4C; within-family swap vs K4) | — | red in gate 4C only; isolate GREEN @ iso3b (SCS /tmp/xt213_iso3_multisess.log; bead KF registry note) → flake |

Note: item-6-era baseline was 2582/4/1 at `9ffeb7db7d`; the 2581/5/1 measured at
`f443aced98` reflects campaign commits between the two legs. The RUN-2 catalogue is
the authoritative baseline for this item (item A TDD report, gate 3B note).

## 3. KF rows — flake-confirmed by isolate re-run (suite tests)

| id | test | suite | RUN-2 / first-red signature | isolate evidence | note |
|---|---|---|---|---|---|
| F1 | `compact_remote::remote_compact_v2_rewrites_multiple_trailing_function_call_outputs::automatic` | codex-core `--test all` | panic `compact_remote.rs:1335` (RUN-2, plain-`OpenAI` provider — normalization off ⇒ item A change provably no-op for its body) | g5a: 1 passed (1.43s), `/tmp/xt213_itema_iso_g5a*.log`; also greened at item A G4B | already in the bead KF registry; flake confirmed |
| F2 | `guardian_context_budget::review_preserves_user_instructions_until_request_budgeting::complete_instructions_fit` | codex-core `--test all` | panic `guardian_context_budget.rs:165` (`delta_context.contains(&followup)`, RUN-2) | g5b: 2 passed (3.34s; filter matched 2 names) | plain-`OpenAI` provider ⇒ no-op reasoning as F1; flake confirmed |
| F3 | `attestation::attestation_generate_round_trip_adds_header_to_responses_websocket_handshake` | codex-app-server `--test all` (NEW-in-gate4, not in RUN-2) | `attestation.rs:161` `(left == right)` | g5g: 1 passed (2.21s) | flake; first observed red at item A G4A |
| F4 | `app_server_session::rollout_history::tests::cached_legacy_resume_revalidates_history_across_migration_settings` (`codex-rs/tui/src/app_server_session/rollout_history_tests.rs:285`) | codex-tui `--lib` (item B row 13) | `rollout_history_tests.rs:341` `4 == 3` (request-id delta +3 vs +2, RUN-2) | PROBE-3 both sides green isolated: s1 fork tree GREEN 5.30s; s2 = same fork commit with upstream 9-row `models.json` swapped in (rebuild + run) GREEN 5.46s (item B spec §2.4) (`/tmp/xt213_itemb_probe3_s{1,2}.log`); red at item B G3 only under full-suite load | VERDICT: load-sensitive flake, out of item B scope (item B spec §2.4). Escalation: P2 bead ONLY on red quiet-isolated re-run. Instrumentation candidates (for a future item): `rollout_history.rs:195-208` retry `ThreadResume` arm (guard :198; reissue :204); `history.rs:249-251` legacy-hydration `thread_read`; `history.rs:257` paginated-hydration `thread_turns_page` (enclosing `hydrate_initial_thread_history` :236; prime candidate); `now_or_never` guard at `rollout_history_tests.rs:335`; the stale candidate cites in spec B §8 Round-1 C4 note (:428-430) are superseded by this row |

## 4. Deterministic red — new-item rows (NOT KF; each has its own bead)

All three: deterministic red in isolate re-run, provably no-op under the plain-`OpenAI`
path (or no span-helper involvement), pre-existing (not caused by item A/B changes).
File a bead, do not KF-register.

| id | test | signature (isolate) | bead |
|---|---|---|---|
| D1 | `personality::config_personality_none_sends_no_personality` | g5c red — `personality.rs:88`: `{{ personality }}` placeholder present in top-level `instructions` string (not a span-helper site) | `apex-xt2.16` |
| D2 | `subagent_notifications::spawned_full_history_v2_child_uses_model_precedence_without_dropping_context::full_fork_restores_explicit_policy_after_proactive_transition` | g5d red — `subagent_notifications.rs:1725`: "proactive policy should share a developer message with unrelated context" (content assert) | `apex-xt2.17` |
| D3 | `unified_exec::unified_exec_formats_large_output_summary` (NEW-in-gate4) | g5e red — `unified_exec.rs:3517`: "regex did not match actual value" | `apex-xt2.18` |

## 5. N2 / item A2 cross-reference (out of KF/ENV scope)

Item A gate 5 bucketed the 124-row residue (`/tmp/xt213_itema_residue_table.txt`;
per-test panic-site data `/tmp/r2_panic_locs.json`). These rows belong to item A2
(per-file N2 rewrites: tool-output-shape / agent-message-conversion /
raw-item-shape / namespace-tools asserts), not to this registry:

- N2-namespace-tools: 16 (+4 astra insta-hash pins = 20 tool-visibility). Exemplar:
  `mcp_resource::orchestrator_skill_can_read_referenced_resource_without_an_executor`
  (`mcp_resource.rs:380`, `tool_by_name("skills","list").is_some()` — identical site
  at RUN-2 and item A G4A).
- N2-raw-item-shape: 35 (body compare) + 4 (insta snap) + 3 (injected items) +
  2 (instructions) + 4 (model request). Includes
  `manual_compact_twice_preserves_latest_user_messages` — the 1/21 non-clearing
  "clearable" N1 name: it now panics at the insta shape snapshot
  (`manual_compact_with_history_shapes`, `vendor/insta/src/runtime.rs:719`) instead
  of its RUN-2 helper site `compact.rs:4045` ⇒ reclassified N2-raw-item-shape, not
  an N1 residual. N1 residue measured = 0 (no surviving failure panics at a
  `message_input_texts*` helper site).
- N2-tool-output-shape / exec-process: 1. N2? (behavioral expect): 2.
- LDR? (load/deadline, to be re-adjudicated per-name): 18 + 19 (exec-process
  deadline class) + 3 (approval-flow timing).
- other (RUN-2 catalogue): 6.
- 17 N1+N2 partial names (all `multi_agent_v2_developer_instructions`, app-server):
  17/17 still red at item A G4A (spec expected at most 1 to clear — measured truth).
- Reconciliation: the 124-row residue = the §5 bucket counts above
  (20+48+1+2+40+6 = 117) + 7 rows tracked in §1–§4 (3 KF-candidate +
  3 NEW-in-gate4 + 1 ENV, per the gate-5 bucket lines). The 17 N1+N2 partials
  are counted WITHIN the app-server 89, not additive.

## 6. FORK-MANIFEST pending rows (re-derive at ratchet time — xt2.15)

The manifest re-derivation itself is deliberately deferred to the upstream-ratchet
step; these rows are registered here so nothing is lost between now and then.

Pending rows to add/verify at ratchet (8 files — 3 item A + 5 item B; the two spec docs are not pending-manifest rows):
- Item A (`3cd2efebaf`): `codex-rs/core/tests/common/responses.rs` (widened span
  helpers + pin test), `codex-rs/core/tests/suite/pending_input.rs` (local twin
  widened; 13 call-site migration deferred to A2), `codex-rs/app-server/tests/
  suite/v2/turn_start.rs` (workaround helper deleted).
- Item B (`51e1b313de`): `codex-rs/tui/src/app/tests/model_catalog.rs`,
  `codex-rs/tui/src/app/tests/recap_generation_tests.rs`,
  `codex-rs/tui/src/app/tests/safety_buffering.rs`,
  `codex-rs/tui/src/app/tests/snapshots/...model_migration_prompt_shows_for_hidden_model.snap`,
  `codex-rs/tui/src/chatwidget/snapshots/...model_selection_popup.snap`.

Item-B upstream-collision specifics (pull range `f3da3861c5..78245b47af`):
- `model_selection_popup.snap` COLLIDES with the in-range upstream rewrite
  (`547c9a1aad`, #46503: capitalized display names + subtitle line removed, −6/+5)
  — expect a real 3-way at ratchet; item B's six content lines + D-1 metadata
  line must survive.
- `recap_generation_tests.rs` meets an upstream-modified file (upstream +1 at
  :456, non-adjacent to item B's :200 filter site) — likely trivial.
- `safety_buffering.rs`, `model_catalog.rs`, hidden-model `.snap` are NOT touched
  in range — they persist as fork divergence.
- D-1 (picker snap `expression:` metadata `popup`) is source-forced on any
  re-record (item B spec §3.1) — do not "fix" it back at ratchet.

Stale manifest row: `FORK-MANIFEST.md:122` — the `turn_start.rs` row names the
item-A-DELETED helper `normalized_developer_message_texts`; rewrite that row (the
surviving fork fact is the post-normalization developer-message assertion itself)
at ratchet time.

Reference ref: `reference/upstream-openai` still at `f3da3861c5` — advance to
`78245b47af` (origin/main tip) and update the manifest "Last verified" ONLY at
xt2.15.

## 7. Escalation rules (how future runs use this registry)

1. Full-suite run: a failed name matching a row here → record the row id in the run
   note; no action.
2. Failed name NOT in this registry → NEW: classify (N2 / KF / ENV / other) and
   adjudicate before adding it here.
3. KF registration requires an isolate re-run (same env as the full run, one cargo
   invocation, unique-substring filter, assert the `running N test(s)` line):
   - green isolated → flake → add a row in §2/§3 with the isolate evidence path.
   - red isolated → deterministic → file a new bead (see §4 pattern), not a KF
     row — EXCEPT the exec-process deadline family (K3–K6), which stays KF even
     when red in isolate per item A spec §5.5 ("plus the exec_process deadline
     names if their isolate re-runs still fail").
4. Flake-verdict rows (F1–F4): a RED quiet-isolated re-run escalates the row to a
   P2 bead (parent `apex-xt2`), and the row gains a "re-escalated" note with date.
5. ENV rows: green = no action. An ENV row failing in an environment that HAS the
   capability (e.g. E1 on a TTY box) escalates to a P2 bead.
6. Never silence a row by deleting or weakening its test; KF rows are tracked, not
   muted. If a KF row is ever fixed properly, move the row to a "retired" note with
   the fixing commit.
