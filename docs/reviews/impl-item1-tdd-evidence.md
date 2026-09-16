# Item 1 — P2 Add-File leniency: TDD evidence (apex-ayl.52)

Worker: `/root/item1_p2_worker` (stage 5, item 1 of 6).
Branch: `feat/normalize-content-types-vllm`, working-tree HEAD `197ea1642c`.
SoT: spec v5 `docs/responses-compat-apply-patch-format.md` (§3.2, §3.4, §4 T1);
breakdown v3 `docs/responses-compat-apply-patch-task-breakdown.md` (Item 1, §0, §4, §5).

## Pre-check (protocol §4 step 1)

- Line refs re-verified at working-tree HEAD before editing:
  - AddFile arm `streaming_parser.rs`:198-215 (header check :199, `+` branch
    :202-208, trailing `Err(InvalidHunkError …)` :209-214) — confirmed.
  - P3.1 StartedPatch message :193, P3.2 DeleteFile message :222, boundary
    messages :168 / :184 / :374 — confirmed present and left untouched.
  - Ledger #2 assert at :838 inside `test_streaming_patch_parser_returns_errors`
    (fn :814) — confirmed; ledger #1 (:828) and #3 (:848) left untouched.
  - Surviving asserts :819 / :784 / :797 — confirmed.
- Tree green before starting: `just test -p codex-apply-patch` →
  `Summary 99 tests run: 99 passed, 0 skipped`.

## Sub-case 1 (T1.6 / ledger #2) — rewrite to new expectation → RED

Action: rewrote ledger #2 (assert :835-842 in the existing inline `mod tests`)
from `Err(InvalidHunkError { message: "'bad' is not a valid hunk header. …",
line_number: 3 })` to `Ok(vec![AddFile { path: PathBuf::from("file.txt"),
contents: "bad\n".to_string() }])` per spec §4 T1.6 / ledger #2 ("Add-File raw
line (e.g. `bad`) → `Ok`, content `bad\n`").

Test command: `just test -p codex-apply-patch streaming_parser`

RED (captured):

```text
  TRY 2 FAIL [   0.031s] (11/11) codex-apply-patch streaming_parser::tests::test_streaming_patch_parser_returns_errors
  stderr ───
    thread 'streaming_parser::tests::test_streaming_patch_parser_returns_errors' (83813689) panicked at apply-patch/src/streaming_parser.rs:835:9:
    assertion failed: `(left == right)`

    Diff < left / right > :
    <Err(
    <    InvalidHunkError {
    <        message: "'bad' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'",
    <        line_number: 3,
    <    },
    >Ok(
    >    [
    >        AddFile {
    >            path: "file.txt",
    >            contents: "bad\n",
    >        },
    >    ],
     )

     Summary [   0.093s] 11 tests run: 10 passed, 1 failed, 88 skipped
error: recipe `test` failed on line 88 with exit code 100
```

This is the red state for the core implementation (task sub-case 1).

## Sub-case 2 — implement lenient AddFile arm per spec §3.2 → GREEN

Change (kept inside the existing `AddFile` arm; `+` branch and header check
untouched; state machine un-restructured):

- Replaced the trailing `Err(InvalidHunkError "not a valid hunk header" …)`
  with: any other line (not structural, not `+`-prefixed) is appended verbatim
  (`line`, then `\n`) as Add-File content; arm returns `Ok(())`.
- Added the spec-mandated module doc note ("Lenient add-file content (P2)") at
  the top of `streaming_parser.rs` (spec §3.2 implementation note: "Update the
  module doc comment with a short 'lenient add-file content' note so the
  divergence is discoverable on rebase").

GREEN (captured): `just test -p codex-apply-patch streaming_parser` →

```text
        PASS [   0.060s] (11/11) codex-apply-patch streaming_parser::tests::test_streaming_patch_parser_returns_errors
────────────
     Summary [   0.061s] 11 tests run: 11 passed, 88 skipped
```

Item gate after core implementation: `just test -p codex-apply-patch` →

```text
     Summary [   2.844s] 99 tests run: 99 passed, 0 skipped
```

New P2 tests live in the new sibling file
`codex-rs/apply-patch/src/streaming_parser_p2_tests.rs`, declared in
`streaming_parser.rs` as
`#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"] mod
streaming_parser_p2_tests;` (module named `streaming_parser_p2_tests`, NOT
`tests`, per breakdown Item 1 step 6 — the file already has an inline
`mod tests`).

Red-state note for sub-cases 3-11: P2 is ONE behavior change (the AddFile
arm's trailing `Err` replaced by a verbatim-content branch), so there is
exactly ONE red state — the ledger #2 rewrite above (task sub-case 1).
Sub-cases 3-10 below therefore land as green locks: their behavior is the
consequence of the sub-case 2 implementation whose red was captured at
ledger #2 (breakdown §4 step 2: "RED if new behavior, green lock
otherwise"). Sub-cases 11 (T1.11) and 12 (T1.13) are regression locks with
no prior red, stated explicitly below.

## Sub-case 3 (T1.1) — F1 replay: raw markdown Add-File

Test added: `test_p2_add_file_raw_markdown_f1_replay_matches_canonical`
(`streaming_parser_p2_tests.rs:10`). Raw markdown content (`# H1` first
line, tables, blank lines) parses; `contents` asserted byte-identical to
the canonical `+`-prefixed version of the same file (whole-object equality
of the parsed hunk vectors, plus equality against the exact content
string).

Test command: `just test -p codex-apply-patch test_p2_add_file_raw_markdown_f1_replay_matches_canonical`

GREEN (captured):

```text
        PASS [   0.024s] (1/1) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_raw_markdown_f1_replay_matches_canonical
────────────
     Summary [   0.024s] 1 test run: 1 passed, 99 skipped
```

(Green lock: behavior from the sub-case 2 implementation; red = ledger #2.)

## Sub-case 4 (T1.2) — empty line inside Add-File

Test added: `test_p2_add_file_empty_line_becomes_empty_content_line`
(`streaming_parser_p2_tests.rs:42`). Patch content lines `a`, ``, `b` →
`contents == "a\n\nb\n"`.

Test command: `just test -p codex-apply-patch test_p2_add_file_empty_line_becomes_empty_content_line`

GREEN (captured, after the sub-case 3 file landed; re-verified in the
3-test P2 run):

```text
        PASS [   0.026s] (1/1) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_empty_line_becomes_empty_content_line
────────────
     Summary [   0.026s] 1 test run: 1 passed, 100 skipped
```

## Sub-case 5 (T1.3) — mixed `+`-prefixed and raw lines

Test added: `test_p2_add_file_mixed_prefixed_and_raw_lines_concatenate`
(`streaming_parser_p2_tests.rs:58`). Lines `+hello`, `world`, `+there` →
`contents == "hello\nworld\nthere\n"` (order preserved, `+` stripped from
prefixed lines, raw lines verbatim).

Test command: `just test -p codex-apply-patch streaming_parser_p2_tests`

GREEN (captured):

```text
        PASS [   0.035s] (1/3) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_mixed_prefixed_and_raw_lines_concatenate
        PASS [   0.035s] (2/3) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_raw_markdown_f1_replay_matches_canonical
        PASS [   0.037s] (3/3) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_empty_line_becomes_empty_content_line
────────────
     Summary [   0.038s] 3 tests run: 3 passed, 99 skipped
```

## Sub-case 6 (T1.7) — `*** ` non-marker lines, structural lines, padded End Patch

Tests added:
- `test_p2_add_file_raw_stars_line_matching_no_marker_is_content`
  (`streaming_parser_p2_tests.rs:75`): `*** Note: keep this line` → content.
- `test_p2_add_file_real_headers_inside_add_file_remain_structural`
  (`streaming_parser_p2_tests.rs:93`): `*** Add File: b.txt`,
  `*** Delete File: d.txt`, `*** Update File: u.txt` inside an Add-File
  each start their own hunk; whole-hunk-vector equality asserted (AddFile
  a.txt `x\n`, AddFile b.txt `y\n`, DeleteFile d.txt, UpdateFile u.txt with
  one chunk `z`).
- `test_p2_add_file_whitespace_padded_end_patch_is_structural`
  (`streaming_parser_p2_tests.rs:130`): `   *** End Patch` → structural
  (trimmed match), `finish()` Ok.

Test command: `just test -p codex-apply-patch test_p2_add_file_raw_stars_line_matching_no_marker_is_content test_p2_add_file_real_headers_inside_add_file_remain_structural test_p2_add_file_whitespace_padded_end_patch_is_structural`

GREEN (captured):

```text
        PASS [   0.023s] (1/3) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_real_headers_inside_add_file_remain_structural
        PASS [   0.022s] (2/3) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_whitespace_padded_end_patch_is_structural
        PASS [   0.025s] (3/3) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_raw_stars_line_matching_no_marker_is_content
────────────
     Summary [   0.027s] 3 tests run: 3 passed, 102 skipped
```

## Sub-case 7 (T1.8) — typo'd structural line swallowed as content

Test added: `test_p2_add_file_typoed_structural_line_swallowed_as_content`
(`streaming_parser_p2_tests.rs:153`). `*** Ad File: x` (typo) inside an
Add-File is swallowed as content and the rest of the patch applies
(pins the spec §3.2 silent-partial-apply class); `finish()` Ok with
contents `"*** Ad File: x\nreal content\n"`.

Test command: `just test -p codex-apply-patch test_p2_add_file_typoed_structural_line_swallowed_as_content`

GREEN (captured):

```text
        PASS [   0.021s] (1/1) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_typoed_structural_line_swallowed_as_content
────────────
     Summary [   0.021s] 1 test run: 1 passed, 105 skipped
```

## Sub-case 8 (T1.9) — lossy `++42` case; existing-path overwrite decision

Tests added:
- `test_p2_add_file_raw_plus_prefixed_line_strips_leading_plus`
  (`streaming_parser_p2_tests.rs:178`): raw line `++42 20 7946 0958` →
  content `+42 20 7946 0958\n` (pins the spec §3.2 named lossy case — a raw
  line starting with `+` is parsed as a prefixed line; the leading `+` is
  stripped, identical to today's canonical `+`-line behavior).
- `test_p2_add_file_to_existing_path_parses_without_existence_check`
  (`streaming_parser_p2_tests.rs:196`): at the parser level a raw Add-File
  parses as-is into an `AddFile` hunk — no existence check on any path
  (spec §3.2/§3.4 decision). The overwrite OUTCOME for an existing file is
  the apply layer's and is pinned (unmodified) by fixture scenario
  `011_add_overwrites_existing_file` (green in the gates below) and the
  core-suite test `apply_patch_cli_add_overwrites_existing_file` (out of
  this item's scope, untouched).

Test command: `just test -p codex-apply-patch test_p2_add_file_raw_plus_prefixed_line_strips_leading_plus test_p2_add_file_to_existing_path_parses_without_existence_check`

GREEN (captured):

```text
        PASS [   0.020s] (1/2) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_raw_plus_prefixed_line_strips_leading_plus
        PASS [   0.021s] (2/2) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_to_existing_path_parses_without_existence_check
────────────
     Summary [   0.022s] 2 tests run: 2 passed, 106 skipped
```

## Sub-case 9 (T1.9b) — last content line `*** End Patch`, both variants

Tests added:
- `test_p2_add_file_last_line_canonical_end_patch_is_content`
  (`streaming_parser_p2_tests.rs:215`): canonical `+*** End Patch` preserved
  verbatim as content — contents `"line1\n*** End Patch\n"` (matches the
  spec §3.2 probe on the unmodified binary).
- `test_p2_add_file_last_line_raw_end_patch_truncates`
  (`streaming_parser_p2_tests.rs:234`): raw unprefixed `*** End Patch` as
  the last content line truncates at that line — contents `"line1\n"`
  (lenient-form-only loss pinned by spec §3.2).

Test command: `just test -p codex-apply-patch test_p2_add_file_last_line_canonical_end_patch_is_content test_p2_add_file_last_line_raw_end_patch_truncates`

GREEN (captured):

```text
        PASS [   0.019s] (1/2) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_last_line_canonical_end_patch_is_content
        PASS [   0.020s] (2/2) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_last_line_raw_end_patch_truncates
────────────
     Summary [   0.021s] 2 tests run: 2 passed, 108 skipped
```

## Sub-case 10 (T1.10) — whitespace line, env-id in content, unclosed patch, CRLF

Tests added:
- `test_p2_add_file_whitespace_only_line_is_whitespace_content`
  (`streaming_parser_p2_tests.rs:251`): whitespace-only line `   ` →
  whitespace content `"   \n"` (verbatim; no trimming).
- `test_p2_add_file_environment_id_line_is_content`
  (`streaming_parser_p2_tests.rs:268`): `*** Environment ID: env1` inside
  Add-File → content; `environment_id()` stays `None` (env-id honored only
  in `StartedPatch`, pre-existing).
- `test_p2_add_file_unclosed_patch_finish_error_unchanged`
  (`streaming_parser_p2_tests.rs:287`): unclosed patch → `finish()` returns
  the unchanged `Err(InvalidPatchError("The last line of the patch must be
  '*** End Patch'"))`.
- `test_p2_add_file_crlf_raw_lines_are_lf_equivalent`
  (`streaming_parser_p2_tests.rs:302`): CRLF raw Add-File → LF-equivalent
  contents `"raw line\n"` (one trailing `\r` stripped per line by
  `push_delta`, unchanged path).

Test command: `just test -p codex-apply-patch test_p2_add_file_whitespace_only_line_is_whitespace_content test_p2_add_file_environment_id_line_is_content test_p2_add_file_unclosed_patch_finish_error_unchanged test_p2_add_file_crlf_raw_lines_are_lf_equivalent`

GREEN (captured):

```text
        PASS [   0.024s] (1/4) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_crlf_raw_lines_are_lf_equivalent
        PASS [   0.024s] (2/4) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_unclosed_patch_finish_error_unchanged
        PASS [   0.025s] (3/4) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_whitespace_only_line_is_whitespace_content
        PASS [   0.026s] (4/4) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_add_file_environment_id_line_is_content
────────────
     Summary [   0.027s] 4 tests run: 4 passed, 110 skipped
```

## Sub-case 11 (T1.11) — Update-File stays strict (regression lock)

**Regression lock: green at write time, NO prior red** (the Update arm was
not touched by P2; this test guards the spec §3.4 no-Update-leniency
decision).

Test added: `test_p2_update_file_raw_unprefixed_lines_still_rejected`
(`streaming_parser_p2_tests.rs:318`). Raw unprefixed line `bad` inside an
Update hunk → still rejected with the UNCHANGED message
`"Unexpected line found in update hunk: 'bad'. Every line should start
with ' ' (context line), '+' (added line), or '-' (removed line)"`
(line_number 3).

Test command: `just test -p codex-apply-patch test_p2_update_file_raw_unprefixed_lines_still_rejected`

GREEN (captured):

```text
        PASS [   0.024s] (1/1) codex-apply-patch streaming_parser::streaming_parser_p2_tests::test_p2_update_file_raw_unprefixed_lines_still_rejected
────────────
     Summary [   0.025s] 1 test run: 1 passed, 114 skipped
```

## Sub-case 12 (T1.13) — golden/canonical scope (regression lock)

**Regression lock: NO new test, NO prior red.** The 25 scenario fixtures
under `codex-rs/apply-patch/tests/fixtures/scenarios` and the
canonical-success parser tests must pass UNMODIFIED.

Verified:
- `git status --short -- codex-rs/apply-patch/tests/` → empty (fixtures
  and suite untouched).
- `ls codex-rs/apply-patch/tests/fixtures/scenarios/ | grep -c '^[0-9]'`
  → 25 fixture dirs.
- `suite::scenarios::test_apply_patch_scenarios` (runs all 25 fixtures)
  PASS in the final gate run below, as do all `parser.rs` canonical
  tests (`test_parse_patch`, `test_parse_patch_lenient`, …) in the same
  run.

## Gates (after final green) — run from `codex-rs/`

1. `just test -p codex-apply-patch`:

```text
        PASS [   2.731s] (115/115) codex-apply-patch::all suite::scenarios::test_apply_patch_scenarios
────────────
     Summary [   3.272s] 115 tests run: 115 passed, 0 skipped
```

   (99 pre-existing + 16 new P2 tests; includes ledger #1/:828 and #3/:848
   untouched-and-green in `test_streaming_patch_parser_returns_errors`,
   surviving asserts :819/:784/:797, all canonical parser tests, and the
   25 golden scenario fixtures.)

2. `just fix -p codex-apply-patch`:

```text
    Checking codex-apply-patch v0.0.0 (/Users/palanisd/Projects/upstream/codex/codex-rs/apply-patch)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 04s
```

   exit 0 — clippy clean, no fixes applied.

3. `just fmt`:

   exit 0 — no output; idempotent (cosmetic line-wrapping of two long
   `let patch =` literals in the new test file; semantics unchanged, per
   repo rule no test re-run after fix/fmt).

## Final diff (item scope)

`git diff --stat -- codex-rs/apply-patch/`:

```text
 codex-rs/apply-patch/src/streaming_parser.rs | 32 ++++++++++++++++++----------
 1 file changed, 21 insertions(+), 11 deletions(-)
```

New (untracked) files:

- `codex-rs/apply-patch/src/streaming_parser_p2_tests.rs` (16 tests)
- `docs/reviews/impl-item1-tdd-evidence.md` (this file)

`streaming_parser.rs` changes, hunk by hunk:
1. Module doc comment: short "Lenient add-file content (P2)" note at top
   of file (spec §3.2 implementation note mandates this note for rebase
   discoverability).
2. `AddFile` arm: trailing `Err(InvalidHunkError "not a valid hunk
   header" …)` replaced with the verbatim-content push + `Ok(())`
   (`+` branch and header check untouched; state machine not restructured;
   ~8 lines net).
3. One test-module declaration:
   `#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"] mod
   streaming_parser_p2_tests;`
4. Ledger #2 assertion rewrite (task sub-case 1) in the existing inline
   `mod tests`.

All other working-tree modifications (AGENTS.md, codex-api/*,
core/src/tools/handlers/*, spec_plan.rs, model-provider/*, models-manager/*)
are pre-existing campaign seam state — untouched by this item.

## Discrepancy note (for coordinator; no action taken)

Spec §3.2 implementation note says the P2 change "adds one clarifying line"
to `parser.rs`'s doc comment (grammar listing drift, seat A n2).
`parser.rs` is OUTSIDE this item's file ownership (breakdown Item 1 file
list: `streaming_parser.rs` + new sibling test file + existing inline
assert; task contract: "No other file may be modified"), and no item in
the breakdown assigns that parser.rs doc line. Worker therefore did NOT
touch `parser.rs`. Coordinator to decide: fold the one-line doc note into
this item's commit (it is doc-only, zero behavior), assign it to Item 5's
doc pass, or drop it.

## parser.rs doc line (spec §3.2 impl note, coordinator-directed)

Coordinator decision (2026-09-16): the spec §3.2 implementation note IS
part of the P2 change, so the line lands in THIS item's commit, recorded
as a documented, spec-mandated deviation from the breakdown's "AddFile arm
+ test files" DoD.

Exact line added to the `codex-rs/apply-patch/src/parser.rs` module doc
comment, immediately after the existing final doc line
("…allows for leading/trailing whitespace around patch markers."):

```text
//! The streaming parser's `AddFile` arm additionally accepts non-`+`-prefixed content lines verbatim (lenient add-file content); the grammar listing above remains the canonical form.
```

One line, `//!` style, doc-only — no reconciliation of the pre-existing
grammar/`.lark` drift (spec forbids it). `parser.rs` is the ONLY other
file touched by this item; its diff is exactly `+1` line.

Gate re-run after the doc line (from `codex-rs/`):

1. `just test -p codex-apply-patch`:

```text
     Summary [   3.465s] 115 tests run: 115 passed, 0 skipped
```

2. `just fix -p codex-apply-patch`:

```text
    Checking codex-apply-patch v0.0.0 (/Users/palanisd/Projects/upstream/codex/codex-rs/apply-patch)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.03s
```

   exit 0 — clippy clean.

3. `just fmt`: exit 0 (no reflow; `wrap_comments` is not enabled).

Line-ref note for downstream items: this doc line shifts every
`parser.rs` line reference by **+1** (Item 3's step-1 re-verify at HEAD
covers this; e.g. `test_parse_patch` :277 → :278, boundary strings
:268/:271 → :269/:272).
