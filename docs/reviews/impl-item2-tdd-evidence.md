# Item 2 — P1 function-tool spec text: TDD evidence (apex-ayl.52)

Worker: `/root/item2_p1_worker` (stage 5, item 2 of 6).
Branch: `feat/normalize-content-types-vllm`, working-tree HEAD `49d584e82e`.
SoT: spec v5 `docs/responses-compat-apply-patch-format.md` (§3.1, §4 T2);
coordinator task contract for Item 2 (P1).

## Pre-check (line refs verified at current working-tree state)

- Ownership files re-verified before editing:
  - `create_apply_patch_freeform_tool` `apply_patch_spec.rs:13` — untouched.
  - `create_apply_patch_function_tool` `apply_patch_spec.rs:40` (pre-edit;
    post-edit :79) — the seam's function tool, kept as-is; only its two
    description literals changed.
  - Pre-edit literals: `patch` argument description `apply_patch_spec.rs:44`;
    tool description `:57`.
  - Test module declared `#[path = "apply_patch_spec_tests.rs"]`
    `apply_patch_spec.rs:70-71` (post-edit :108-109).
  - Pre-edit test `create_apply_patch_function_tool_matches_expected_spec`
    `apply_patch_spec_tests.rs:40` (post-edit :45), expected tool
    description `:45` (post-edit :50), expected `patch` argument `:52`
    (post-edit :57).
  - Capability gate confirmed present and left untouched:
    `spec_plan.rs:1257-1271` and `model-provider/src/provider.rs:372`
    (function path non-OpenAI only). No gate change in this item.
- Text extraction (mechanical, no retyping): python sliced the two fenced
  blocks out of spec §3.1 (first ``` after each label heading up to the
  closing ```), printed `repr()`, and spliced those exact bytes into the
  Rust source. Verified byte-identical against the spec file AFTER
  splicing (regex extraction from the compiled source vs fresh slice of
  the spec):
  - tool description: 126 chars / 21 words (126 bytes)
  - `patch` argument description: 2,198 chars / 380 words
    (2,204 bytes = 2,198 + 6 extra bytes for the three multi-byte chars —
    two `—` em-dashes and one `→` (r1 seat B n-1)
  - total: 401 words ✓ (matches spec §3.1 "Length (v2, measured)")
  - no `\` or `"` inside either text → plain `&str` literals, no escapes.
- Tree state before starting: `just test -p codex-core apply_patch` →
  `Summary [ 561.015s] 110 tests run: 92 passed (2 flaky), 4 failed,
  14 timed out, 4251 skipped` (see "Baseline failure note"); the four
  `apply_patch_spec` unit tests all PASS.

## Baseline failure note (pre-existing; NOT caused by this item)

Baseline run (unmodified tree) had 4 integration failures, all with the
identical signature `timeout waiting for event: Elapsed(())` at
`core/tests/common/lib.rs:388`:

```text
suite::apply_patch_cli::apply_patch_cli_delete_missing_file_reports_error      (FAIL TRY 1 → PASS TRY 2)
suite::apply_patch_cli::apply_patch_custom_tool_streaming_emits_updated_changes (FAIL TRY 1, FAIL TRY 2)
suite::apply_patch_cli::apply_patch_emits_turn_diff_event_with_unified_diff     (FAIL TRY 2)
suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff (FAIL TRY 1, FAIL TRY 2)
suite::apply_patch_cli::apply_patch_exec_command_heredoc_with_cd_emits_turn_diff (FAIL TRY 2)
```

Characterization (unmodified tree, before any Item 2 edit):
- Isolated re-run of the 4 non-flip tests: 3 of 4 PASSED
  (`4 tests run: 3 passed, 1 failed`); the one flip
  (`delete_missing_file_reports_error`) already passed its own TRY 2
  in the baseline run.
- `apply_patch_exec_command_failure_propagates_error_and_skips_diff`
  failed 4 consecutive times (2 baseline + 2 isolated single-test
  re-runs), always the same event-wait timeout. It is a
  `skip_if_no_network!` test that executes a real shell heredoc under
  load; not resolved by this item (out of file ownership).
- Machine was oversubscribed during all runs: 14 cores,
  `vm.loadavg` 16–23 (unrelated `grok-build-responses` build running in
  parallel). The repo's own `.config/nextest.toml` documents exactly
  this: "Higher concurrency causes integration test timeouts under
  resource contention on common developer machines"
  (`slow-timeout = { period = "30s", terminate-after = 2 }`,
  `retries = 1`).
- The identity of the flaking integration test changes run to run
  (baseline 4 vs RED-run 1 different test), while the new unit test
  fails deterministically — consistent with load, not logic.

## Sub-case T2.1 — drift-guard test → RED

Action: appended test
`create_apply_patch_function_tool_teaches_patch_format_in_argument_description`
(`apply_patch_spec_tests.rs:84-110`, post-edit) asserting the 7 exact
literal substrings from spec §3.1 against the `patch` argument
description extracted from `create_apply_patch_function_tool(false)`:
`first line is `*** Begin Patch``, `real newline characters`, `bare '+'`,
`at most one hunk per patch`, `starts with '+'`, `multiple `@@` chunks`,
`must change at least one line`. No char-count assertions (per contract;
mechanical verification recorded in the pre-check instead).

Test command: `just test -p codex-core apply_patch` (from `codex-rs/`).

RED (captured, verbatim):

```text
  TRY 1 FAIL [   0.274s] (───────) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_teaches_patch_format_in_argument_description
  stdout ───

    running 1 test
    test tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_teaches_patch_format_in_argument_description ... FAILED

    failures:

    failures:
        tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_teaches_patch_format_in_argument_description

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2533 filtered out; finished in 0.07s

  stderr ───

    thread 'tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_teaches_patch_format_in_argument_description' (84260872) panicked at core/src/tools/handlers/apply_patch_spec_tests.rs:100:9:
    patch argument description missing: first line is `*** Begin Patch`
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(Also `TRY 2 FAIL [0.608s]`, same panic line/message — deterministic.)
RED-run summary: `Summary [ 665.583s] 111 tests run: 95 passed (15 flaky),
2 failed, 14 timed out, 4251 skipped` — the 2nd failure is
`apply_patch_cli_updates_file_appends_trailing_newline` (TRY 2 only), a
baseline-PASS test, i.e. the load flake identity changed again.

## Sub-case — implement spec §3.1 consts → T2.1 GREEN

Changes in `apply_patch_spec.rs` (both consts file-local `private`, no new
public API; stored directly above the function per spec §3.1):

- `:35-37` — `APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION: &str` (single line,
  126 chars, verbatim spec §3.1).
- `:39-72` — `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION: &str`
  (multi-line literal, :42-:72, 2,198 chars, verbatim spec §3.1).
- `:83` — `patch` argument literal replaced by
  `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION.to_string()`.
- `:96` — tool description literal replaced by
  `APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION.to_string()`.
- Function shape unchanged: one required string argument `patch`,
  optional `environment_id`, `strict: false`,
  `additionalProperties: false`.

T2.3 (intentional change, not a regression): the two expected literals in
`create_apply_patch_function_tool_matches_expected_spec`
(`apply_patch_spec_tests.rs:50` tool description, `:57` `patch` argument)
updated to the exact spec §3.1 text — referenced through the same consts
(the test lives in the same module tree via `use super::*`), mirroring
the file's existing pattern of referencing `APPLY_PATCH_LARK_GRAMMAR`
rather than duplicating the grammar text. Single source of truth; the
drift guard (T2.1) pins the text to the spec.

GREEN (captured): `just test -p codex-core apply_patch_spec` →

```text
        PASS [   0.041s] (1/5) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_teaches_patch_format_in_argument_description
        PASS [   0.041s] (2/5) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_freeform_tool_matches_expected_spec
        PASS [   0.041s] (3/5) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_matches_expected_spec
        PASS [   0.044s] (4/5) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_freeform_tool_includes_environment_id_when_requested
        PASS [   0.046s] (5/5) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_includes_environment_id_when_requested
────────────
     Summary [   0.065s] 5 tests run: 5 passed, 4357 skipped
```

## Sub-case T2.2 — self-consistency round-trip (green at write time)

Test added: `create_apply_patch_function_tool_example_round_trips_through_parser`
(`apply_patch_spec_tests.rs:112-162`). Extraction per the PINNED rule
(spec §4 T2.2): split the argument description at the first line that is
EXACTLY `Example:` (a naive substring search for `*** Begin Patch` is
wrong — the first sentence contains that phrase); the remainder (to end of
description) must end
with the line `*** End Patch`; the remainder is fed to
`codex_apply_patch::parse_patch` (codex-core already depends on
codex-apply-patch) and the whole `ApplyPatchArgs` is asserted by
object equality:

- `Hunk::AddFile { path: notes/todo.md, contents: "# TODO\n\n1. ship the fix\n" }`
- `Hunk::UpdateFile { path: src/main.rs, move_path: None, chunks: [UpdateFileChunk {
  change_context: Some("fn main"), old_lines: ["    old_call();", "    shared();"],
  new_lines: ["    new_call();", "    shared();"], context_line_indices: [(1, 1)],
  is_end_of_file: false }] }`
- `patch: example` (verbatim, parser trims only), `workdir: None`,
  `environment_id: None`.

It PASSED at write time (no text drift between the embedded example and
the spec §3.1 Example block):

```text
        PASS [   0.049s] (6/6) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_example_round_trips_through_parser
────────────
     Summary [   0.066s] 6 tests run: 6 passed, 4357 skipped
```

## Sub-case T2.4 — freeform tool spec unchanged

No edits to `create_apply_patch_freeform_tool` or its tests. Both
existing freeform tests green in every run above:
`create_apply_patch_freeform_tool_matches_expected_spec` and
`create_apply_patch_freeform_tool_includes_environment_id_when_requested`.

## Per-sub-case run counts (`just test -p codex-core apply_patch_spec`)

| state | unit tests | result |
| --- | --- | --- |
| baseline (pre-edit) | 4 | 4 passed |
| after T2.1 test (RED) | 5 | 1 failed (drift guard), 4 passed |
| after implementation + T2.3 | 5 | 5 passed |
| after T2.2 test (final) | 6 | 6 passed |

Full filter counts: baseline 110 → RED 111 → final 112.

## Gates (after final green) — run from `codex-rs/`

1. `just test -p codex-core apply_patch`:

```text
        PASS [   0.201s] ( 22/112) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_example_round_trips_through_parser
        PASS [   0.159s] ( 23/112) codex-core tools::handlers::apply_patch_spec::tests::create_apply_patch_function_tool_teaches_patch_format_in_argument_description
────────────
     Summary [ 471.641s] 112 tests run: 98 passed (14 flaky), 14 timed out, 4251 skipped
```

   0 deterministic failures — 98 pass, and the 14 "timed out"
   entries are slow-timeout terminations that time out on BOTH tries
   (`TRY 2 TMT` in the raw log; `slow-timeout period=30s,
   terminate-after=2`) — they do not end green, and the non-zero exit
   comes from those 14 terminal timeouts. The 14 TMT identities match
   the pre-Item-2 baseline name-for-name (re-verified by seat B in
   impl-item2-r1-seatB.md), and the repo's `.config/nextest.toml`
   attributes this class to resource contention. This is a pre-existing
   environmental condition, not an Item 2 failure. (r1 seat B m-1
   correction.) Both new unit tests
   PASS (22/112, 23/112); all four pre-existing unit tests PASS; both
   freeform tests PASS (T2.4).

2. `just fix -p codex-core`:

```text
    Checking codex-core v0.0.0 (/Users/palanisd/Projects/upstream/codex/codex-rs/core)
       Fixed core/src/tools/handlers/apply_patch_spec.rs (1 fix)
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
       Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 41s
```

3. `just fmt`:

```text
exit 0 — no output; idempotent (normalized one whitespace-only line left by the gate 2 fix in `apply_patch_spec.rs`; no string literal touched — post-fmt byte-identity re-verified, see pre-check).
```

## Final diff (item scope)

`git diff --stat` scoped to the ownership files (note: the working tree
also carries the uncommitted Item 1 seam in the same two files, so this
stat is the combined seam+Item 2 delta vs HEAD):

```text
 codex-rs/core/src/tools/handlers/apply_patch_spec.rs     |  78 +++++++++++++
 codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs | 125 +++++++++++++++++++++
 2 files changed, 203 insertions(+)
```

Item 2's own delta, file by file:

- `codex-rs/core/src/tools/handlers/apply_patch_spec.rs`:
  - +2 file-local private consts above `create_apply_patch_function_tool`
    (:35-37 tool description, 1 line; :39-72 `patch` argument, multi-line
    literal) — verbatim spec §3.1 bytes.
  - 2 literal → const swaps (:83, :96); function shape untouched.
- `codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs`:
  - :1-7 imports (+`codex_apply_patch::{ApplyPatchArgs, Hunk,
    UpdateFileChunk, parse_patch}`, `std::path::PathBuf`).
  - T2.3: expected literals at :50/:57 → spec §3.1 consts (intentional
    change, not a regression).
  - :84-110 new T2.1 drift-guard test (7 substrings).
  - :112-162 new T2.2 self-consistency round-trip test (whole-object
    `assert_eq!` via `pretty_assertions`).
- `docs/reviews/impl-item2-tdd-evidence.md`: new (this file).

All other working-tree modifications (AGENTS.md, codex-api/*,
apply_patch.rs, apply_patch_tests.rs, mod.rs, spec_plan.rs,
model-provider/*, models-manager/*) are pre-existing campaign seam state
— untouched by this item.

## Discrepancy notes (for coordinator)

1. Baseline integration failures (see "Baseline failure note"): 4
   `suite::*` failures at tree start, all `timeout waiting for event`
   event-wait timeouts; 3 of 4 pass on isolated re-run and all 4 pass in
   the post-implementation gate run (gate 1 shows 0 deterministic
   failures, same 14 TMTs as baseline).
   `apply_patch_exec_command_failure_propagates_error_and_skips_diff`
   failed 4 consecutive times under load (14-core box, loadavg 16–23,
   concurrent unrelated build); it needs a quiet-machine or CI
   confirmation — NOT attributable to Item 2 (text-only change in tool
   descriptions; mocked-model test).
2. Gate 1 (`just test -p codex-core apply_patch`) exits non-zero purely
   from the 14 first-try slow-timeout terminations (recovered on retry);
   identical TMT count in the pre-Item-2 baseline run. Coordinator may
   want to re-run the gate on a quiet machine or in CI for a clean
   zero-exit record.
3. T2.3 implemented as const references (not duplicated 2,198-char
   literals) in the expected `ResponsesApiTool`, mirroring the existing
   `APPLY_PATCH_LARK_GRAMMAR.to_string()` pattern in the freeform
   expected-spec test; the task contract's "update the two expected
   literals to the spec §3.1 exact text" is satisfied (the expected
   values now carry the spec §3.1 exact text) with a single source of
   truth. Flagging for the review loop in case a literal-duplicate form
   is preferred.
4. Gate 2 (`just fix -p codex-core`, mandated) auto-applied a
   `let_and_return` fix in `apply_patch_spec.rs` (an ownership file)
   that collapsed the PRE-EXISTING uncommitted seam's
   `let freeform = ToolSpec::Freeform(...); freeform` (freeform tool
   builder, Item 1's function-tool seam) back into the direct
   `ToolSpec::Freeform(...)` return — i.e. that region of
   `create_apply_patch_freeform_tool` now matches HEAD except one
   whitespace-only blank line at `apply_patch_spec.rs:22` (absent in
   HEAD; no functional impact — emitted freeform ToolSpec
   byte-identical, clippy/fmt-stable). (r1 seat B m-2 correction.)
   Semantics identical (clippy let_and_return); the freeform spec text
   and both freeform tests are untouched (T2.4); no campaign doc
   references the binding. Kept because re-introducing the let-binding
   would be re-fixed by every subsequent `just fix` run.
5. The same gate-2 run auto-fixed a pre-existing unused import
   (`wiremock::matchers::body_json`) in
   `core/tests/suite/openai_file_mcp.rs` — OUTSIDE this item's
   ownership. That file was clean vs HEAD at task start (verified in
   the initial `git status`), so it was restored to its pre-gate state
   with `git checkout --` afterwards (no seam content lost; the fix
   target was an import line, not a seam line). Coordinator: that
   unused import is real (clippy will re-fix it on the next
   `just fix -p codex-core`); suggest folding the one-line removal into
   a seam commit or a follow-up.
