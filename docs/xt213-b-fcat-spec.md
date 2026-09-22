# Spec — xt2.13 item B: TUI suite refresh — catalog-drift snapshots, catalog-name
# collision test data, and TUI-local post-normalization span-type filters

Status: FINAL v1.3.1 (0B/0M at spec review r2; v1.3 = r2 minor + nit fixes: §3.2
48/16 sweep count, §1 panic-footer range, §2.1 merge-base qualifier, §2.5/§5 G0
wording, §8 guard-line cite; v1.3.1 = G2d deviation D-1 adjudicated — picker
`expression:` metadata refresh approved, §3.1; review history §8/§9; C4 verdict §2.4 —
row 13 ruled LOAD-SENSITIVE FLAKE, out of scope; fix targets 13 of 15)
Scope: test-support + tracked snapshot files only. **No product code.**

## 1. Purpose

Bring the `codex-tui --lib` suite to a clean baseline at HEAD `3cd2efebaf` by fixing the
15 RUN-2 TUI failures whose root causes are (a) fork catalog growth, (b) fork catalog
entries colliding with synthetic test model names, or (c) TUI-local test helpers that
filter wire spans for the pre-normalization `input_text` type only (item A's N1 class,
TUI-local copies). One failure is SCS-environment-only (crossterm TTY) and is catalogued
for item C, not fixed here.

Authoritative measurement: RUN-2 full-suite log `/tmp/xt213_fullsuite_run2.log` (TUI lib
section lines 39832–45403; result line 45403: `FAILED. 4859 passed; 15 failed; 3
ignored`), panic blocks extracted at section lines 5310–5552; fork catalog
`codex-rs/models-manager/models.json` (14 entries) vs upstream reference
`reference/upstream-openai` (9 entries; fork-only: `gpt-5.2`, `gpt-5.4-mini`, `glm-5.2`,
`grok-4.6`, `qwen3.8-27b`). The xt2.15 pull (`f3da3861c5` → `78245b47af`, the current
origin/main tip) does NOT touch `safety_buffering.rs` or `model_catalog.rs`; among this
spec's five files it modifies `recap_generation_tests.rs` and the picker `.snap`
(divergence record in §4/§7).

### 1.1 Measured population — all 15 TUI lib failures at RUN-2

| # | Test (module::name) | Panic site | Class | Disposition |
|---|---|---|---|---|
| 1 | `chatwidget::tests::popups_and_settings::model_selection_popup_snapshot` | insta runtime.rs:719 @ popups_and_settings.rs:3309 | C1 catalog-snapshot | FIX (§3.1) |
| 2 | `chatwidget::tests::popups_and_settings::model_picker_refreshes_startup_catalog` | insta runtime.rs:719 @ popups_and_settings.rs:3393 | C1 | FIX (§3.1) |
| 3 | `chatwidget::tests::popups_and_settings::model_picker_queued_all_models_uses_refreshed_catalog` | insta runtime.rs:719 @ popups_and_settings.rs:3423 | C1 | FIX (§3.1) |
| 4 | `chatwidget::tests::popups_and_settings::model_picker_refresh_preserves_dismissal_and_reasoning_submenu` | insta runtime.rs:719 @ popups_and_settings.rs:3544 | C1 | FIX (§3.1) |
| 5 | `app::tests::model_catalog::model_migration_prompt_shows_for_hidden_model` | model_catalog.rs:531 `expect("upgrade configured")` | C2 name-collision | FIX (§3.2) |
| 6 | `app::tests::model_catalog::retired_model_migration_respects_catalog_metadata` | model_catalog.rs:201 `left==right` (Some(catalog gpt-5.4-mini upgrade) vs None) | C2 | FIX (§3.2) |
| 7 | `app::tests::safety_buffering::safety_retry_can_retry_a_first_turn_a_second_time` | safety_buffering.rs:1037 `[] == [RETRY_PROMPT]` | C3 span-filter | FIX (§3.3) |
| 8 | `app::tests::safety_buffering::safety_retry_forks_first_turn_and_continues_without_duplicating_prompt` | safety_buffering.rs:1037 `[] == [PREV, RETRY]` | C3 | FIX (§3.3) |
| 9 | `app::tests::safety_buffering::safety_retry_forks_after_the_previous_turn_and_uses_faster_settings` | safety_buffering.rs:1037 `[] == [PREV, RETRY]` | C3 | FIX (§3.3) |
| 10 | `app::tests::safety_buffering::safety_retry_preserves_a_committed_steer_from_the_interrupted_turn` | safety_buffering.rs:1037 `[] == [PREV, RETRY, STEER]` | C3 | FIX (§3.3) |
| 11 | `app::tests::safety_buffering::safety_retry_replays_older_interruption_notices` | safety_buffering.rs:1037 `[] == [PREV, RETRY]` | C3 | FIX (§3.3) |
| 12 | `app::tests::recap_generation::recap_generation_uses_bounded_structured_request_and_inserts_result` | recap_generation_tests.rs:202 `expect("recap prompt")` | C3 span-filter | FIX (§3.3) |
| 13 | `app_server_session::rollout_history::tests::cached_legacy_resume_revalidates_history_across_migration_settings` | rollout_history_tests.rs:341 `4 == 3` (request-id delta +3 vs +2) | C4 request-count | KF-FLAKE → item C catalogue, NO FIX (§2.4 verdict) |
| 14 | `app::tests::safety_buffering::agents_overview_acknowledges_inactive_steer_before_interrupt` | insta runtime.rs:782 inline-snapshot-in-loop | C5 insta-dup | FIX (§3.4) |
| 15 | `app::tests::background_task_defaults_tests::review_regression_agents_overview_creation_is_fresh_but_returning_is_not` | crossterm read.rs:61 "reader source not set" | C6 ENV-TTY | item C catalogue, NO FIX (§7) |

Fix-target population: **13 of 15** — rows 1–12 and 14. Row 13 is a load-sensitive
flake (§2.4 verdict) and row 15 is SCS-env only; both go to the item C catalogue.

## 2. Root causes (coordinator-verified)

### 2.1 C1 — picker snapshot predates fork catalog growth (rows 1–4)

All four tests assert ONE shared snapshot file,
`tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__model_selection_popup.snap`
(tracked; last touched upstream by `eb7bd64ef9` as of the merge-base — the pull
range re-touches it, see §4). It records a 5-model picker
(`gpt-6-astra`, `gpt-5.6-sol`, `gpt-5.6-terra`, `gpt-5.6-luna`, `gpt-5.5 (current)`).
`TEST_MODEL_PRESETS` (`tui/src/test_support.rs:12`) is built from the BUNDLED
`models-manager/models.json` — i.e. the fork's 14-entry catalog — and the fork added
three picker-visible rows after the snapshot was recorded: `gpt-5.2` (prio 29),
`glm-5.2` (50), `grok-4.6` (51). RUN-2 insta diffs (all four identical): +6 lines,
exactly those three models with their two-line wrapped descriptions, inserted after
`gpt-5.5`; everything else byte-identical. `codex-auto-review` (43) is hidden
(visibility=hide) — consistent. `qwen3.8-27b` (52) is visibility=list yet ABSENT from
the RUN-2 render — see OQ-1 in §3.1.

### 2.2 C2 — fork catalog entries collide with synthetic test model names (rows 5–6)

`model_presets_with_test_upgrades()` (model_catalog.rs:123) clones the `gpt-5.5` preset,
renames it to `gpt-5.2` (id+model), stamps a synthetic upgrade, and pushes it LAST.
Upstream's catalog has no `gpt-5.2` and no `gpt-5.4-mini` (upstream `eb7bd64ef9`
removed retired entries; the upstream `models.json` carries an upgrade entry for
`gpt-5.4` only), so upstream `.find(|p| p.model == "gpt-5.2")` resolves the synthetic
entry. The fork catalog added real `gpt-5.2` (no upgrade, prio 29) and `gpt-5.4-mini`
(upgrade → `gpt-5.6-luna`, retirement 2026-08-31, prio 23) entries — both occupy
earlier vector positions than the pushed synthetic entry (`.find()` is vector-order
over the priority-sorted preset vector with the synthetic appended LAST), so
`.find()` now resolves the real entries:

- row 6: `expected = current.upgrade.clone()` = None (real gpt-5.2), while
  `model_upgrade_for_migration("gpt-5.4-mini", …)` finds the REAL gpt-5.4-mini entry
  first (vector order — the synthetic, a renamed gpt-5.5 clone keeping prio 12, is
  pushed LAST) and returns its catalog upgrade
  (retirement Some(2026-08-31T19:00:00Z) — observed in the RUN-2 diff, which the
  hard-coded fallback at `startup_prompts.rs:117-135` could not produce, since that
  path returns `retirement_at: None`). `assert_eq!(Some(…), None)` red.
- row 5: `.find(p.model == "gpt-5.2")` → real entry → `current.upgrade` is None →
  `expect("upgrade configured")` panics.

The helper's other consumer (`model_migration_prompt_only_shows_for_deprecated_models`,
model_catalog.rs:176) is unaffected by renaming the synthetic entry: it queries by the
plain slugs `gpt-5.2`/`gpt-5.4` and its `any(upgrade.id == target)` check still finds the
synthetic upgrade regardless of the synthetic entry's own model name (verified against
`should_show_model_migration_prompt`, startup_prompts.rs:138-172; the test is green in
RUN-2 and stays green — no change to it).

### 2.3 C3 — TUI-local span-type filters predate normalization (rows 7–12)

Item A widened the shared `core_test_support` helpers to accept post-normalization span
types (`input_text | text`). The TUI crate carries LOCAL copies of the same narrow
filter, used by tests whose provider is NOT `OpenAI` (so
`normalize_content_types`, `codex-api/src/endpoint/content_type_compat.rs:33`, gated by
`!is_openai()` at `model-provider/src/provider.rs:399`, IS active and relabels
`input_text`→`text` on the wire):

- `user_input_texts` (safety_buffering.rs:438-450) filters
  `span type == Some("input_text")` (:447). Provider `safety-retry-test`
  (safety_buffering.rs:31, name "Interrupt test") → rows 7–11 all observe `[]` at the
  shared assert :1037.
- `recap_generation_tests.rs:200` inline-finds
  `item["type"] == "input_text"` in the last user content; the test's `MODEL` is
  `gpt-5.2` (recap_generation_tests.rs:18) under a non-OpenAI mock provider → the
  recap prompt part arrives as `text` → find returns None → `expect("recap prompt")`
  at :202.

Identical failure class to item A's N1, different helper locations — hence a separate
item (item A's spec §6 confined itself to the shared helpers + one local twin).

### 2.4 C4 — rollout-history request-count drift (row 13) — VERDICT: load-sensitive flake, out of scope

`assert_eq!(app_server.next_request_id, next_request_id + 2)` (rollout_history_tests.rs:341)
observed a +3 delta at RUN-2. Static analysis (coordinator): the entire resume path
(`tui/src/app_server_session/**`, `app-server/src`, `rollout/src`) is byte-identical to
the upstream reference (`git diff reference/upstream-openai..HEAD --stat` → no files);
the test file is also unmodified (upstream green since `ac192cd793` #43178). The fork's
only data input reaching this path is the bundled catalog (normalization rewrites
bodies, not request counts).

SCS probe (data-only, no code changes; Mac logs `/tmp/xt213_itemb_probe3_s{1,2}.log`):
- stage 1 — fresh isolated re-run at HEAD, fork 14-row catalog: GREEN (`1 passed`,
  5.30s, 4876 filtered).
- stage 2 — same commit, upstream 9-row `models.json` swapped in (rebuild + run):
  GREEN (`1 passed`, 5.46s, 4876 filtered).
- stage 3 — restore: `models.json` byte-identical to HEAD (hash
  `b3ea545b7080ee079a2eb172c8b1f8a5c0c7f377`).

Verdict: the row is a load-sensitive flake, NOT catalog- or fork-driven — green in
isolation under BOTH the fork and the upstream catalog, red only under the RUN-2
full-suite parallel load. The +3 delta is consistent with timing-sensitive request-id
accounting under suite-level concurrency (mechanism unconfirmed; the exercised path is
byte-identical upstream and green). Disposition: NO FIX in item B; carried to the item C
KF registry with both probe data points. Escalation rule: raise a P2 bead (parent
`apex-xt2`, label `codex-combined`) only if the test goes red again in a quiet isolated
run.

### 2.5 C5 — shared inline snapshot + insta's global duplicate guard (row 14)

`active_turn_interrupt_is_nonblocking_and_coalesces_repeated_requests` (WithinTask) and
`agents_overview_acknowledges_inactive_steer_before_interrupt` (BetweenTasks) both call
`interrupt_after_inactive_steer` (safety_buffering.rs:182), which ends with the inline
assertion `insta::assert_snapshot!(…composer_text_with_pending(), @"")` at :400. Insta
1.46.3 (lock == upstream lock == vendored copy) keys its `prevent_inline_duplicate`
guard (vendor/insta/src/runtime.rs:776-790) on
`{function_name}|{file}|{line}` with `function_name` = the ENCLOSING function's type
path (`_function_name!`, vendor/insta/src/macros.rs:4-19) — identical for both test
entries — and stores it in a process-global `INLINE_DUPLICATES` set that is NEVER
cleared (only refs: runtime.rs:32, :778). Whichever of the two tests runs second in a
`cargo test -p codex-tui --lib` process panics with "Insta does not allow inline
snapshot assertions in loops" — scheduling order, not name order; RUN-2 hit it on
the agents_overview test. The macro `insta::allow_duplicates!`
(vendor/insta/src/macros.rs:608, `#[macro_export]`) exists in 1.46.3 and is the
sanctioned escape; both executions assert the same value (`""`), so allow-duplicate
recording is safe. Upstream stays green because its test path is `cargo nextest run` (the local `test`
recipe, justfile:88; CI is likewise nextest-based) — one process per test, so the global set never accumulates across
the twins: row 14 is a single-process `cargo test` harness artifact, NOT a fork
divergence. The xt2.15 pull does NOT touch this file (verified against
`f3da3861c5..78245b47af`), so the fix persists as fork divergence (item C manifest).

## 3. Change set (test-support + tracked snapshots only)

Confinement: exactly these paths may be modified. No product code, no `Cargo.toml`/
`Cargo.lock`, no `BUILD.bazel`, no wire-shape changes.

### 3.1 C1 — refresh shared picker snapshot (rows 1–4)

File: `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__model_selection_popup.snap`
(one file; all four tests share it). Regenerate on SCS (gate G2) by running the four
tests with `INSTA_UPDATE=always`, then VERIFY the resulting diff is EXACTLY: six added
lines after the `gpt-5.5 (current)` block —
`  6. gpt-5.2` + wrapped "Optimized for professional work and long-running agents."
description, `  7. glm-5.2` + "Zhipu GLM-5.2 — state-of-the-art agentic coding
capabilities.", `  8. grok-4.6` + "xAI Grok-4.6 — flagship model built for coding and
agentic tasks." — plus (approved deviation D-1, coordinator-adjudicated at G2d, v1.3.1)
the `expression:` metadata line: the re-record FORCES
`expression: "render_bottom_popup(&chat, 80)"` → `expression: popup`, because the
current test source passes a local `popup` variable through
`assert_chatwidget_snapshot!` (chatwidget/tests.rs:194-200 records the value token);
the legacy quoted form is not reproducible from the present source and would re-drift
on every future re-record. The `expression:` line is insta display metadata — the
assertion compares only the content below `---`. Any OTHER hunk ⇒ stop, report, do
not commit (hash-pin lesson from xt2.7 item 6).

**OQ-1 (picker population) — RESOLVED at v1.1.** The picker list is a
`ListSelectionView` capped at `MAX_POPUP_ROWS = 8` visible rows
(`tui/src/bottom_pane/popup_consts.rs:13`,
`list_selection_view.rs:505-506` `max_visible_rows(len) = MAX_POPUP_ROWS.min(len)`).
Nine models are picker-visible in the fork catalog (upstream 5 + `gpt-5.2`, `glm-5.2`,
`grok-4.6`, `qwen3.8-27b` — all `visibility: "list"`), so the rendered snapshot shows
exactly the first 8; `qwen3.8-27b` (priority 52, ninth) is clipped by the scroll
viewport, which is why RUN-2's render contains 8 models. The six-line expectation above
is therefore DETERMINISTIC; the G2 re-record remains the authoritative byte source, and
any deviation from the six-line diff is a stop-and-report condition.

### 3.2 C2 — de-collide synthetic test model names (rows 5–6)

File: `codex-rs/tui/src/app/tests/model_catalog.rs` (test module only):

1. `model_presets_with_test_upgrades()` :131-132 — synthetic rename target `"gpt-5.2"` →
   `"test-gpt-5.2"` (both `id` and `model`). The synthetic entry keeps its upgrade stamp
   (id `gpt-5.5`, key `hide_test_migration_prompt`) unchanged.
2. `retired_model_migration_respects_catalog_metadata` :196 find `"gpt-5.2"` →
   `"test-gpt-5.2"`; :198-199 rename target `"gpt-5.4-mini"` → `"test-gpt-5.4-mini"`
   (must not match the catalog nor the hard-coded fallback key); :202 and :211
   `model_upgrade_for_migration("gpt-5.4-mini", …)` → `"test-gpt-5.4-mini"`; :208 find
   `"gpt-5.4-mini"` → `"test-gpt-5.4-mini"`.
3. `model_migration_prompt_shows_for_hidden_model` :522 find `"gpt-5.2"` →
   `"test-gpt-5.2"`.

File: `codex-rs/tui/src/app/tests/snapshots/codex_tui__app__tests__model_catalog__model_migration_prompt_shows_for_hidden_model.snap`
— re-record on SCS (gate G2, `INSTA_UPDATE=always`); expected diff: the two lines
mentioning the synthetic slug read `test-gpt-5.2` instead of `gpt-5.2`
("We recommend switching from test-gpt-5.2 to gpt-5.5." / "You can continue using
test-gpt-5.2 if you prefer."); the `Introducing GPT-5.5.` header and description line are
unchanged (target display name comes from the REAL gpt-5.5 entry, as before).

NO changes to: `model_migration_prompt_only_shows_for_deprecated_models` (:176-190,
including its real-row `gpt-5.2` reference at :178), the real-catalog tests using
`"gpt-5.4-mini"`/`"gpt-5.4"` pairs (:373, :465-502), or any other `"gpt-5.2"` usage —
the full TUI-crate sweep at HEAD is 48 sites in 16 files (re-derivable:
`grep -rn '"gpt-5.2"' codex-rs/tui/src`), and every `"gpt-5.2"` site outside the four
C2 rename sites in model_catalog.rs (:131-132, :196, :522) is left untouched:
model_catalog.rs's :178 real-row reference; test fixtures green in RUN-2
(thread_title_tests, session_lifecycle_requests, background_task_defaults_tests,
agents_overview_actions_tests, popups_and_settings, slash_commands,
collaboration_catalog_tests, chatwidget/tests/helpers, chatwidget/tests/app_server,
status_and_layout, reasoning_defaults_tests, temporary_structured_request_tests,
app_server_session's test module) — recap_generation_tests' MODEL const (:18) excepted,
whose test was red in RUN-2 for the §3.3 reason and is fixed by this spec; and the
production `model_popups.rs:498` `starts_with` check.

### 3.3 C3 — widen the two TUI-local span filters (rows 7–12)

1. `codex-rs/tui/src/app/tests/safety_buffering.rs:447` —
   the filter line becomes
   `.filter(|span| matches!(span.get("type").and_then(Value::as_str), Some("input_text" | "text")))`
   (byte-matches item A's committed widened shared filters,
   `core/tests/common/responses.rs:236,258`; one-line change in the local
   `user_input_texts` helper).
2. `codex-rs/tui/src/app/tests/recap_generation_tests.rs:200` —
   `.find(|item| item["type"] == "input_text")` →
   `.find(|item| matches!(item["type"].as_str(), Some("input_text" | "text")))`.

No other call sites of either filter exist in the TUI crate (grep-verified).

### 3.4 C5 — allow the duplicated inline snapshot (row 14)

File: `codex-rs/tui/src/app/tests/safety_buffering.rs` :400 — wrap the trailing inline
assertion:

```rust
insta::allow_duplicates! {
    insta::assert_snapshot!(app.chat_widget.composer_text_with_pending(), @"");
}
```

(the macro is `#[macro_export]`ed in insta 1.46.3; the wrap adds two lines of braces).
Both calling tests
observe the same value `""`, so the allow-duplicate path records matching snapshots.

### 3.5 C4 — no change (row 13 out of scope, §2.4 verdict)

No code, snapshot, or manifest change for row 13 in this item. The probe ruled it a
load-sensitive flake (green in isolation under both catalogs, §2.4). It is carried to
the item C KF registry with the probe logs; a P2 bead is raised only on a red
quiet-isolated re-run (escalation rule, §2.4).

## 4. Risks and mitigations

- **Snapshot refresh picks up UNRELATED drift.** Mitigation: §3.1 byte-exact diff
  expectation + coordinator byte review of both `.snap` diffs before commit;
  `INSTA_UPDATE=always` scoped to the five target tests only (never a suite-wide
  accept — item-A astra lesson).
- **C2 rename changes recorded copy text beyond the slug.** Mitigation: §3.2 expected
  diff is two lines only; the `Introducing GPT-5.5.` header is target-display-name
  driven (real entry), verified against `migration_copy_for_models` semantics.
- **C3 widening hides a genuine wire regression** (a part that SHOULD be `input_text`
  arriving as something else entirely). Mitigation: the widening accepts exactly the two
  post-normalization text types (item A precedent, r3-reviewed); `encrypted_content` and
  other part types still fail the filter; row 12's downstream `contains`/`matches`
  asserts (:203-222) still validate the prompt CONTENT, not just its type.
- **C5 wrap masks a real loop regression.** Mitigation: the wrapped assertion is a
  fixed-value inline snapshot (`@""`) at a single shared call site; behavior is
  validated by the surrounding asserts (turn-completed, pending-interrupt cleared,
  request-id delta) which are untouched.
- **Divergence growth vs upstream.** All FIVE touched files are upstream-owned
  (zero-diff vs the reference before this change). Verified against the actual pull
  range `f3da3861c5..78245b47af`: it modifies two of the five —
  `recap_generation_tests.rs` (the §3.3 recap half meets an upstream-modified file at
  pull time) and the picker `.snap` (upstream rewrites it in-range — capitalized
  display names, subtitle line removed — so the §3.1 six-line re-record collides with
  the upstream re-record at pull time); it does NOT touch `safety_buffering.rs`,
  `model_catalog.rs`, or the hidden-model `.snap` — §3.2/§3.4 persist as fork
  divergence. All five recorded in FORK-MANIFEST at the ratchet re-derivation
  (item C / xt2.15), NOT in this commit's file set.
- **SCS-only verification.** All gates run on SCS (build plane); the Mac cannot build
  the fork (no rusty_v8 prebuilt). Mac-side verification = byte diff review + rustfmt
  `--check` on the final file shapes only.

## 5. Gates (all on SCS, tree @ `3cd2efebaf` + item-B diff)

SCS recipe (build-verified, item A): `cd /x/eng/ai_engineering/APEX/codex/codex-rs` with
`CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=1.94.1-x86_64-unknown-linux-gnu
CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
CODEX_SKIP_BWRAP_BUILD=1 RUSTY_V8_ARCHIVE=/x/eng/ai_engineering/APEX/rusty-v8/librusty_v8_ptrcomp_sandbox_release_x86_64-unknown-linux-gnu.a.gz
RUSTY_V8_SRC_BINDING_PATH=/x/eng/ai_engineering/APEX/rusty-v8/src_binding_ptrcomp_sandbox_release_x86_64-unknown-linux-gnu.rs`.

- **G0 baseline re-measure** — run the 13 fix-target test names (unique-substring
  filters, per the filter-semantics note below) at HEAD with no diff. Expected: rows
  1–12 RED at the RUN-2 panic sites; row 14 GREEN in isolation — by construction, its
  duplicate-guard panic requires BOTH twins of the shared helper in one process
  (§2.5); the twin `active_turn_interrupt_is_nonblocking_and_coalesces_repeated_requests`
  (WithinTask, green in RUN-2) proves the `@""` value for its path, and row 14's own
  isolation green proves it for the BetweenTasks path. Mandatory baseline
  reproduction of row 14's red mode (same binary, no rebuild): ONE 2-filter invocation
  with row 14 + that twin → exactly ONE of the two reds at the duplicate
  inline-snapshot assert (whichever runs second). Purpose: re-confirm the RUN-2 measurement before touching
  anything. (Row 13 is not a fix target — its baseline evidence is the probe stage-1
  log, §2.4.)
- **G1 code-only, snapshots stale** — apply §3.2/§3.3/§3.4 (§3.5 is no-change) but
  NOT any `.snap` change. Re-run the fix-target names. Expected: rows 1–5 STILL RED:
  rows 1–4 on the shared picker snapshot (proves the code change alone does not alter
  rendered picker content) and row 5 on its OWN snapshot — the renamed slug changes the
  recorded copy text, so the assertion now reaches the (stale) `assert_snapshot!` and
  fails there instead of at the :531 expect. Rows 6–12 and 14 GREEN at G1 (row 14 is
  green in isolation with or without §3.4 — its red mode needs both twins in one
  process; §3.4 is discriminated at G3, where BOTH twins must be green).

Filter semantics (all gates): test filters are UNIQUE NAME SUBSTRINGS passed after
`--` with NO `--exact` (a bare name with `--exact` matches 0 of 4877 — measured on
SCS, the TUI test's full path is e.g.
`app_server_session::rollout_history::tests::cached_…`). Each fix-target filter must
match exactly one test (worker asserts `running 1 test` per invocation, or the
correct per-suite count for multi-filter runs) before trusting the result.
- SCS hygiene: kill-interrupted cargo runs can leave `codex-rs/core/core.NNNN` dumps
  and stale `.snap.new` files in `codex-rs/core/tests/suite/snapshots/` (pre-existing
  since RUN-2); the worker removes new core dumps and never commits SCS-tree state.
- **G2a re-record picker snapshot** — ONE test (`model_selection_popup` substring)
  with `INSTA_UPDATE=always` → `running 1 test`, green, `.snap` rewritten. Authoritative
  byte source for the §3.1 expectation.
- **G2b re-record hidden-model snapshot** — ONE test
  (`model_migration_prompt_shows_for_hidden_model` substring) with
  `INSTA_UPDATE=always` → green, `.snap` rewritten.
- **G2c verification** — run all five (the 4 picker tests + the hidden-model test)
  with default insta mode → all 5 green; `find tui/src -name '*.snap.new'` empty.
- **G2d coordinator byte-review** — `git diff` of the two `.snap` files, scp'd to the
  Mac and reviewed BEFORE the worker reports done: picker diff = the six added fork
  rows per the §3.1 expectation (qwen stays clipped by MAX_POPUP_ROWS) + the D-1
  `expression:` metadata line (§3.1); no other byte changes. Hidden-model diff =
  exactly the two §3.2 slug lines.
- **G3 full TUI lib suite** — `cargo test -p codex-tui --lib` → `test result:` line
  `FAILED. 4873 passed; 1 failed; 3 ignored` (with any failure cargo prints FAILED,
  never ok; the 1 = ENV row 15) or, if row 13 flakes under suite load, `FAILED. 4872
  passed; 2 failed; 3 ignored` (the 2 = ENV row 15 + row 13 at its :341 delta assert).
  The failed name(s) must be EXACTLY {row 15} ∪ {row 13, only if flaked}, and BOTH C5
  twins (`agents_overview_acknowledges_inactive_steer_before_interrupt` +
  `active_turn_interrupt_is_nonblocking_and_coalesces_repeated_requests`) must be
  green — that is the §3.4 discriminator (without the wrap, the second-running twin
  reds at the duplicate assert).
- **G4 hygiene** — SCS rustfmt 1.94.1 `--check --edition 2024` over every changed
  `.rs` → clean. clippy: ENV WAIVER on SCS (component absent; CI enforces — recorded
  in the commit body, item-A precedent).

## 6. Acceptance

1. G0–G4 pass with persisted logs (Mac `/tmp/xt213_itemb_gate*.log`).
2. Both `.snap` diffs byte-match §3.1 (picker: exactly the six added lines; qwen stays
   clipped per OQ-1) and §3.2 (hidden-model: exactly the two slug lines);
   coordinator byte-review recorded in the TDD report.
3. TUI residue = exactly {row 15} (+ row 13 only if it flaked under G3 suite load;
   recorded in the gate log).
4. Mac tree after the item = exactly the confined file set of §3; no untracked
   snapshot pending files; `FORK-MANIFEST.md` untouched in this commit.

## 7. Out of scope

- Row 15 (crossterm ENV-TTY) → item C catalogue (no code fix; SCS has no TTY for
  crossterm's reader-source path).
- Row 13 (load-sensitive flake, §2.4 verdict) → item C KF registry with both probe
  data points and the escalation rule (P2 bead only on a red quiet-isolated re-run);
  no new bead now.
- All app/core suite residue (item C KF/ENV catalogue; A2 input list; beads
  `apex-xt2.16`/`.17`/`.18`).
- `FORK-MANIFEST.md` re-derivation (item C / xt2.15 ratchet time) — including the
  stale `:122` MUST-SURVIVE row noted by the item-A diff seats.
- The upstream `f3da3861c5` → `78245b47af` pull (xt2.15): it modifies
  `recap_generation_tests.rs` and the picker `.snap` (both in this spec's file set —
  pull-time merge/re-record per §4) and does NOT touch `safety_buffering.rs`,
  `model_catalog.rs`, or the hidden-model `.snap` (those persist as fork divergence —
  item C manifest).
- Any product-code change. Any catalog change (the 14-model catalog stands — operator
  decision; no GPT pruning).

## 8. Spec review — findings-round log

### Round 1 (two independent fresh seats, read-only; verdicts /tmp/xt213-itemb-seat-{A,B}-r1-verdict.md)

Seat A TALLY: B=0 M=2 m=2 n=1. Seat B TALLY: B=0 M=2 m=2 n=1. The two Majors concur
across seats. Coordinator re-verified every cited fact against tree/git before
revising (v1.2 = this revision).

| # | Finding (severity, seat) | Disposition |
|---|---|---|
| 1 | G0 "ALL red" unsatisfiable for row 14 — the C5 guard needs both twins in one process; isolated row 14 is deterministically green (Major, A+B) | FIXED — §5 G0 now expects rows 1–12 red + row 14 green isolated, plus a mandatory 2-filter reproduction of the red mode; G1/G3 annotate that §3.4 is discriminated at G3 (both twins green). |
| 2 | "Upstream deleted safety_buffering.rs / the xt2.15 pull deletes it" is FALSE — `6e7d4a4b45` is the `origin/#751` PR-branch tip (not an ancestor of main); origin/main tip = `78245b47af`, where the file EXISTS; pull range does not touch it or `model_catalog.rs` (Major, A+B) | FIXED — §1/§2.5/§4/§7 rewritten from verified git facts (`merge-base --is-ancestor` fails; `cat-file -e` EXISTS; targeted `--name-status` over the five files = {recap_generation_tests.rs M, picker .snap M}). |
| 3 | The pull range rewrites the picker `.snap` (capitalized display names, subtitle removed) — collides with the §3.1 re-record at pull time (Major-2 extension, B) | FIXED — recorded in §4/§7 for the item C manifest; upstream diff verified. |
| 4 | G3's expected `test result: ok.` line can never print with 1 failure (Minor, A) | FIXED — G3 expects `FAILED.` per cargo's format (counts re-verified: 4859+13+1=4873; 4859+13=4872). |
| 5 | §3.2's grep-sweep list incomplete — full sweep = 33 sites / 15 files at HEAD, incl. `model_popups.rs:498` production `starts_with` (Minor, A) | FIXED — §3.2 states the full-sweep claim with the re-derivation command; all non-C2 sites classified and untouched. |
| 6 | §2.2 mechanism wording — synthetic keeps gpt-5.5's prio 12 (not 29); `.find()` is vector-order (pushed last), not priority-sorted (Minor, B) | FIXED — §2.2 reworded (both bullet sites). |
| 7 | §2.5 "(alphabetically later)" is not the mechanism (scheduling order); upstream stays green via `cargo nextest run` (justfile:88) per-process isolation (Minor, B) | FIXED — §2.5 reworded + nextest note added. |
| 8 | §3.3 form `Some("input_text") \| Some("text")` vs item A's committed `Some("input_text" \| "text")` — cosmetic (Nit, B) | FIXED — §3.3 byte-matches item A's committed form (responses.rs:236,258). |
| 9 | §4 says "four" upstream-owned touched files — all FIVE are (3 .rs + both .snap, zero-diff vs reference) (Nit, A) | FIXED — §4 says all FIVE. |

C4 instrument note for item C (from seat B): the +3-vs-+2 delta narrows to exactly one
extra request among three timing-gated fallback sites — `rollout_history.rs:202-212`
(retry ThreadResume), `history.rs:241-244` (legacy-hydration thread_read),
`history.rs:250` (paginated-hydration — prime candidate: the migration race the test
drops its `now_or_never` guard — rollout_history_tests.rs:335 — into). Recommended as
KF-registry instrumentation for
item C; the probe already run is decisive on catalog-independence.

### Round 2 (two independent fresh seats on v1.2; verdicts /tmp/xt213-itemb-seat-{A,B}-r2-verdict.md)

Seat A TALLY: B=0 M=0 m=1 n=3. Seat B TALLY: B=0 M=0 m=1 n=2. **0B/0M reached at r2.**
Both seats verified all 9 r1 fixes against the real v1.2 text and independently
re-verified every new pull-range claim via git (origin/main tip 78245b47af, 6e7d4a4b45
on origin/#751 only, merge-base f3da3861c557, targeted 5-file `--name-status` =
{recap M, picker .snap M}, upstream picker diff -6/+5 @ commit 547c9a1aad #46503,
recap +1 @ :456, justfile:88, all five files zero-diff vs reference/upstream-openai).
Findings (fixed in v1.3):

| # | Finding (severity, seat) | Disposition |
|---|---|---|
| M-r2-1 | §3.2 "33 sites in 15 files" contradicts its own re-derivation command — actual sweep = 48 sites / 16 files; recap_generation_tests' MODEL const (:18) was red in RUN-2 (§3.3 reason), not green (Minor, A+B concur) | FIXED (v1.3) — count corrected to 48/16 (coordinator re-counted: 48 lines, 16 files); :18 reclassified; the four C2 rename sites (:131-132, :196, :522) stated explicitly. |
| n-r2-1 | G0 row-14: §8 says "mandatory" 2-filter repro, §5 says "(evidence)"; the twin-green-RUN-2 sentence only proves the WithinTask value match (Nit, B) | FIXED (v1.3) — §5 G0 now says "Mandatory" and splits the value-match proof (twin = WithinTask; row 14's own isolation green = BetweenTasks). |
| n-r2-2 | §2.5 cites justfile:88 for "its CI" — that is the local test recipe; CI is likewise nextest-based (Nit, B) | FIXED (v1.3) — reworded. |
| n-r2-3 | §1 panic-block locator ends at 5549; the last block's footer runs to 5552 (Nit, A) | FIXED (v1.3) — 5310–5552 (coordinator re-verified in the TUI section extract). |
| n-r2-4 | §8 C4 note: the test's `now_or_never` guard is at rollout_history_tests.rs:335 — no line was cited; add it (Nit, A) | FIXED (v1.3) — :335 cited (coordinator-verified). |
| n-r2-5 | §2.1 "last touched upstream by eb7bd64ef9" reads ambiguously against current origin/main (the in-range re-touch is recorded in §4) (Nit, A) | FIXED (v1.3) — "as of the merge-base" qualifier added. |

## 9. Convergence

- r1: B=0; Majors = G0/row-14 (A+B) + false upstream-deletion claim (A+B) + picker-
  snap pull-range rewrite (B) → all fixed in v1.2.
- r2: B=0, M=0, m=2 (identical content, A+B) + n=5 → **0B/0M REACHED**; all findings
  fixed in v1.3. Spec FINAL — TDD GO.
- v1.3.1 (post-r2, surfaced at G2d during implementation): deviation D-1 — the picker
  re-record also refreshes the `expression:` insta-metadata line, FORCED by the current
  macro (chatwidget/tests.rs:194-200; legacy quoted form not source-reproducible);
  content byte-exact. Coordinator adjudicated APPROVE at the G2d stop gate. Diff seats
  re-verify the forcing mechanism (diff brief D-1 note).
