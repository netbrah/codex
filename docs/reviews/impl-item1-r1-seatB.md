# Item 1 (P2 Add-File leniency, streaming parser) — implementation review, round 1, SEAT B

Bead `apex-ayl.52` · branch `feat/normalize-content-types-vllm` · working-tree
HEAD `197ea1642c` · repo `/Users/palanisd/Projects/upstream/codex`
Date: 2026-09-16 (EDT) · Reviewer: `/root/item1_review_seat_b`
Leaf reviewer (no subagents) · Read-only on the repo except this report

**Mandate (lead lenses: REPO CONVENTIONS, DIFF MINIMALITY, EVIDENCE
AUTHENTICITY).** Line-by-line convention audit of the item diff against the
repo-root `AGENTS.md` Rust/test conventions; test-module conventions; diff
minimality + Item-1 DoD; red/green evidence authenticity
(`docs/reviews/impl-item1-tdd-evidence.md`); independent re-run of the item
gates (`just test -p codex-apply-patch`, `just fix -p codex-apply-patch`);
commit-readiness. A second seat reviews spec conformance/semantics in depth;
this seat verified the full brief regardless.

**Files under review (item footprint — verified to be exactly these, §1):**

1. `codex-rs/apply-patch/src/streaming_parser.rs` — modified, +21/−11, 4 hunks
   (module doc note, AddFile arm, test-module declaration, ledger #2 rewrite)
2. `codex-rs/apply-patch/src/parser.rs` — modified, +1/−0 (one module-doc line)
3. `codex-rs/apply-patch/src/streaming_parser_p2_tests.rs` — new, 334 lines,
   16 tests
4. `docs/reviews/impl-item1-tdd-evidence.md` — new, 443 lines (worker TDD
   evidence, audited in §5)

**SoT:** breakdown v3 `docs/responses-compat-apply-patch-task-breakdown.md`
(Item 1 detail at :92-153, §0 conventions, §4 per-item protocol + review
brief at :405-514, §5 invariants at :458-482); spec v5 (LOCKED)
`docs/responses-compat-apply-patch-format.md` §3.2 at :387-478, §4 T1 at
:617-645; repo-root `AGENTS.md` (Rust/codex-rs + Tests sections).

## VERDICT

**APPROVED — 0 Blocking, 0 Major, 0 Minor, 3 Nit.**

The diff is exactly the P2 AddFile-arm change plus its mandated test
surface; canonical (`+`-prefixed) behavior is byte-identical to HEAD; the
red capture is internally consistent with the pre-change code at the
captured line numbers and the green side was re-verified by this seat's own
gate re-run (`115/115`); all AGENTS.md conventions hold; the evidence doc
discloses the green locks, the coordinator-directed `parser.rs` doc line,
and the re-run gates. The three nits are evidence-transcription and
documentation-ordering cosmetic issues; none blocks commit.

## Method

- `git status --porcelain`, `git diff --stat` / `--numstat` / per-hunk
  `git diff` for the two modified item files; `git show HEAD:<file>` for the
  pre-change state; `git diff --cached` (empty — nothing staged).
- Read every item file in full at line-numbered source:
  `streaming_parser.rs` (934 lines), `parser.rs` diff region, the new 334-line
  test file (all 16 test bodies), and the 443-line evidence doc.
- Diffed the AddFile arm against `git show HEAD` line-by-line; traced 8+ test
  bodies by hand against the implementation (T1.1, T1.2, T1.7 structural,
  T1.9b both variants, T1.10 ×4, T1.11).
- Re-derived the RED-capture line arithmetic (panic line `:835:9`) from the
  HEAD numbering plus the per-hunk deltas; checked nextest retry semantics
  against `codex-rs/.config/nextest.toml`.
- Ran the item gates myself from `codex-rs/`: `just test -p codex-apply-patch`
  and `just fix -p codex-apply-patch`; confirmed the tree was unmodified
  afterwards (identical `--numstat`).
- Greps for orphaned old-behavior assertions, TODO/FIXME/dbg leftovers,
  secrets in the evidence doc; fixture-dir count; staged-state check.

Every claim below is one this seat verified at the source. Line numbers are
to the working tree at review time (HEAD `197ea1642c` + item's uncommitted
changes).

## 1. Mandate 3 — footprint, diff minimality, DoD

### 1.1 Footprint is exactly the 4 item files

- `git status --porcelain` shows the two modified item files
  (`codex-rs/apply-patch/src/parser.rs`,
  `codex-rs/apply-patch/src/streaming_parser.rs`), the one new untracked item
  code file (`codex-rs/apply-patch/src/streaming_parser_p2_tests.rs`), and the
  untracked `docs/reviews/` (which contains the item evidence doc plus the
  pre-existing campaign review/spec/breakdown artifacts).
- All other modified files (`AGENTS.md`, `codex-rs/codex-api/*`,
  `codex-rs/core/src/tools/handlers/*`, `spec_plan.rs`,
  `model-provider/*`, `models-manager/*`) are the pre-existing campaign seam
  state named in the brief. I confirmed the `AGENTS.md` diff is the +116-line
  pre-existing "Working Principles for AI Agents" (SDD flow) block —
  unrelated to Item 1.
- `git diff --cached --stat` is empty: nothing staged.
- `git status --porcelain codex-rs/apply-patch/tests/` is empty: the 25
  golden fixture dirs and the suite are untouched (DoD: "golden scenarios
  unmodified" ✓). I counted the fixtures myself:
  `ls .../fixtures/scenarios/ | grep -c '^[0-9]'` → 25
  (`001`–`024` incl. two `020_*` dirs, plus `README.md`).

### 1.2 Per-file hunk audit — every line required by P2

`git diff --numstat -- codex-rs/apply-patch/`:

```text
1   0   codex-rs/apply-patch/src/parser.rs
21  11  codex-rs/apply-patch/src/streaming_parser.rs
```

`streaming_parser.rs` — exactly 4 hunks (verified hunk-by-hunk; the brief's
"~10 lines" arm description matches):

| Hunk | Location (new) | Δ | Content | Required by P2? |
|---|---|---|---|---|
| 1 | :1-8 | +8 | Module doc note "Lenient add-file content (P2) … See `docs/responses-compat-apply-patch-format.md` §3.2." | Yes — spec §3.2 implementation note mandates a short module-doc note for rebase discoverability |
| 2 | :206-222 | +5/−6 | AddFile arm: the trailing `Err(InvalidHunkError …)` (HEAD :209-214) replaced by the verbatim-content branch | Yes — the entire P2 change |
| 3 | :389-391 | +4 | `#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"] mod streaming_parser_p2_tests;` (+ separating blank line) | Yes — breakdown Item 1 step 6 mandates the sibling test module |
| 4 | :845-852 | +4/−5 | Ledger #2 assertion rewrite inside the existing inline `mod tests` | Yes — ledger #2, owned by Item 1 |

Total: +21/−11 — matches numstat exactly; **no other hunk exists** in the
file (verified: `grep -E "^@@"` on the diff returns exactly these 4).

Hunk 2 detail: the structural check (`handle_hunk_headers_and_end_patch`
call, :207-209) and the `+` branch (:210-216) are **byte-identical to HEAD**
(:202-208) — diffed line-by-line against `git show HEAD`; only the trailing
Err block changed. The new branch:

```rust
if let Some(AddFile { contents, .. }) = self.state.hunks.last_mut() {
    contents.push_str(line);
    contents.push('\n');
}
Ok(())
```

- appends the **raw** `line` + `\n` (verbatim per spec §3.2 — "raw line,
  then `\n`"; no trim, so indentation survives),
- guards with the same `if let Some(AddFile { .. }) = self.state.hunks.last_mut()`
  idiom the adjacent `+` branch uses (in `AddFile` mode the last hunk is
  necessarily the `AddFile` — the mode is set only when that hunk is pushed at
  :117-121 — so the guard is defensive, not a new control-flow path),
- removes the old terminal `Err` as spec §3.2 requires ("removed
  (unreachable after the two branches)").

`parser.rs` — exactly one added line (:26), doc-only, inside the module doc
comment, immediately after the pre-existing final doc line
(:25 "…allows for leading/trailing whitespace around patch markers."):

```text
//! The streaming parser's `AddFile` arm additionally accepts non-`+`-prefixed content lines verbatim (lenient add-file content); the grammar listing above remains the canonical form.
```

- Doc-only ✓ (numstat +1/−0), one line ✓, no grammar/`.lark` reconciliation
  ✓, no drive-by ✓.
- It is the spec §3.2 implementation-note line ("this change adds one
  clarifying line there, but does not attempt a full reconciliation"). The
  breakdown's Item-1 file list does not include `parser.rs`; the evidence
  doc records the worker's discrepancy note (evidence :390-400) and the
  coordinator's decision to fold it into this item (:402-420) as a
  documented, spec-mandated deviation. The brief itself pre-declares this
  fold. Verified present and bounded as described.

### 1.3 DoD check (breakdown Item 1 :143-153)

- Sub-cases all green: 16 new tests cover T1.1, T1.2, T1.3, T1.7 (×3), T1.8,
  T1.9 (×2), T1.9b (×2), T1.10 (×4), T1.11; ledger #2 (T1.6) rewritten green;
  T1.13 = the unmodified golden suite — all green in this seat's own gate
  re-run (§4). ✓
- Ledger #2 green: :845-852 now expects `Ok(vec![AddFile { path:
  PathBuf::from("file.txt"), contents: "bad\n".to_string() }])` — full-object
  `Vec<Hunk>` equality; in the existing inline `mod tests` (fn
  `test_streaming_patch_parser_returns_errors` :825), not moved. ✓
- Ledger #1 (:836-843, StartedPatch) and #3 (:854-862, DeleteFile) untouched —
  still expect the old messages (owned by Item 3); ledger #4 in
  `tests/suite/tool.rs:393` also untouched (verified: `git status` clean on
  `tests/`). ✓
- Parser tests still green and untouched: `git diff` on `parser.rs` is the
  single doc line; the parser test fns (e.g. `test_parse_patch`, now :278
  after the +1 doc-line shift — the shift claim verified against `git show
  HEAD` :277) pass in the gate re-run. ✓
- `git diff` limited to the AddFile arm + test files + the one documented
  doc line: ✓ (§1.2).
- "One commit" readiness: see §6.

## 2. Mandate 1 — convention audit (AGENTS.md Rust section), line by line

Audited every added/changed line of the two source files:

- **Inline `format!` args** (uninlined_format_args): no `format!` calls were
  added or changed; the new arm uses `push_str`/`push`. The new test file uses
  inline args where formatting (`format!("…{content}*** End Patch\n")` at
  :17, `format!("+{line}")` at :20). ✓
- **collapsible_if**: the new branch is a single `if let` with two statements,
  no nested `if`. ✓
- **Method refs over closures**: no closures added (the test's
  `.map(|line| format!("+{line}"))` is a formatting closure, not a method
  call — not subject to the lint). ✓
- **No bool / ambiguous `Option` params**: no new functions or parameters at
  all. ✓
- **`/*param_name*/` comments**: no opaque positional literals introduced. ✓
- **Exhaustive matches, no new wildcard arms**: the `match self.state.mode`
  is unchanged (all 6 arms explicit, :185-385); the new code is `if let`
  chains, no new `match` and no new wildcard. ✓
- **Private modules / explicit public API**: `mod streaming_parser_p2_tests`
  (:389-391) has no `pub`; no new `pub` items anywhere in the diff;
  `StreamingPatchParser`'s public surface (`push_delta`, `finish`,
  `environment_id`) is unchanged. ✓ (breakdown §5 invariant "No new public
  API surface" holds.)
- **No new small single-reference helpers**: none added. ✓
- **Module-size targets**: `streaming_parser.rs` non-test LoC = 387 (impl
  block ends :387; tests begin :389) vs 379 pre-change — well under the 500
  guidance. The new test code went to a sibling file per convention (334
  lines), which is exactly what keeps the implementation module small. ✓
- **No `#[async_trait]` / `#[allow(async_fn_in_trait)]`**: none. ✓
- **No drive-by edits / scope creep**: every hunk maps to a P2 requirement
  (§1.2 table); no reordering, no whitespace churn, no comment touch-ups
  outside the mandated note. ✓
- **Docs**: the module-doc note is the spec-mandated one; no `docs/` product
  documentation added (the evidence doc is a review artifact, allowed). ✓
- **Cargo/lock**: no `Cargo.toml`/`Cargo.lock`/`MODULE.bazel.lock` changes in
  the item footprint (verified in `git status`). ✓

## 3. Mandate 2 — test-module conventions

- **Sibling file + explicit `#[path]`** (AGENTS.md "Test Module
  organization"): the new module is declared at `streaming_parser.rs`:389-391
  as
  `#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"] mod
  streaming_parser_p2_tests;` — exactly the mandated shape, and verbatim what
  breakdown Item 1 step 6 prescribes. ✓
- **No name collision**: the file's pre-existing inline `#[cfg(test)] mod
  tests {` is at :393-394 (11 test fns); the new module is named
  `streaming_parser_p2_tests`, not `tests` — no duplicate-name compile error;
  the crate compiles and all tests ran in my gate re-run. ✓
- **Rewritten ledger #2 stayed in the existing inline module**: :845-852 is
  inside `mod tests` (fn `test_streaming_patch_parser_returns_errors` :825) —
  not moved into the new file. ✓
- **`pretty_assertions::assert_eq`**: imported at
  `streaming_parser_p2_tests.rs`:4 (`use pretty_assertions::assert_eq;`) and
  used by all 20 `assert_eq!` call sites across the 16 tests; the inline
  module's rewritten assertion uses the inline module's existing
  `pretty_assertions` import (:395). ✓
- **Whole-object equality** (AGENTS.md "Prefer deep equals comparisons"):
  every one of the 16 tests asserts on the complete `Vec<Hunk>` (whole hunk
  vectors incl. every field of each variant — e.g. the UpdateFile chunk in
  :115-125 asserts `change_context`/`old_lines`/`new_lines`/
  `context_line_indices`/`is_end_of_file`); the only non-Vec assertion is
  `parser.environment_id() == None` (:284), a whole-method result, not
  field-by-field poking. The ledger #2 rewrite asserts the full
  `AddFile { path, contents }` (both fields of the variant). No
  field-by-field comparisons found. ✓
- **No tests for statically-defined values**: all 16 tests exercise
  `push_delta`/`finish` behavior; none assert on constants or
  statically-derivable values. ✓
- **No negative tests for removed logic**: the removed AddFile trailing-Err
  path — I grepped the whole crate for the old message; the only remaining
  "not a valid hunk header" assertions are the DeleteFile arm code
  (`streaming_parser.rs`:229), ledger #1 (:839), ledger #3 (:858), and ledger
  #4 (`tests/suite/tool.rs:393`) — all surviving behaviors owned by Item 3
  (or unchanged code). **No test was left asserting the old AddFile raw-line
  error, and ledger #2's old assertion was REWRITTEN, not deleted** (diff
  hunk 4 shows the 5 old lines replaced by 4 new lines in place). ✓
- **Regression locks named as such**: T1.11
  (`test_p2_update_file_raw_unprefixed_lines_still_rejected` :318-333) locks
  *retained* Update-File strictness (spec §3.4 "no Update-File leniency") —
  a live behavior guard, not a negative test of removed logic; T1.13 is the
  unmodified golden suite. The evidence states "no prior red" for both
  (evidence :291, :314), as §4 of the breakdown requires for green locks. ✓
- **Test-to-sub-case coverage** (spec §4 T1, breakdown Item 1 step 5):

  | Sub-case | Test(s) in `streaming_parser_p2_tests.rs` |
  |---|---|
  | T1.1 F1 replay | `test_p2_add_file_raw_markdown_f1_replay_matches_canonical` (:10) |
  | T1.2 empty line | `test_p2_add_file_empty_line_becomes_empty_content_line` (:42) |
  | T1.3 mixed lines | `test_p2_add_file_mixed_prefixed_and_raw_lines_concatenate` (:59) |
  | T1.7 `*** ` non-marker / structural / padded End | (:76), (:93), (:131) |
  | T1.8 typo'd structural line | `test_p2_add_file_typoed_structural_line_swallowed_as_content` (:154) |
  | T1.9 `++42` lossy + existing-path parse | (:179), (:197) |
  | T1.9b canonical / raw `*** End Patch` last line | (:216), (:235) |
  | T1.10 whitespace / env-id / unclosed / CRLF | (:252), (:269), (:288), (:303) |
  | T1.11 Update stays strict | `test_p2_update_file_raw_unprefixed_lines_still_rejected` (:319) |
  | T1.13 golden scope | unmodified suite (no new test — as prescribed) |
  | T1.6 ledger #2 | inline `mod tests` :845-852 (rewritten) |

  All 10 Item-1 sub-cases covered; T1.4/T1.5/T1.12/T1.14 correctly belong to
  Item 3 and are absent here. 16 tests = 16 `#[test]` fns; the file contains
  nothing but the doc header, imports, and these tests. ✓

## 4. Mandate 5 — gate re-runs (independent, from `codex-rs/`)

- `just test -p codex-apply-patch` (which is `cargo nextest run --no-fail-fast`
  per the root `justfile` test recipe, `NEXTEST_PROFILE=local`):

  ```text
       Summary [   3.390s] 115 tests run: 115 passed, 0 skipped
  ```

  115/115 as expected (99 pre-existing + 16 new). The run includes
  `suite::scenarios::test_apply_patch_scenarios` (the 25-fixture golden
  scenario suite, PASS) and all canonical parser tests. Exit code 0.
- `just fix -p codex-apply-patch` (recipe: `cargo clippy --fix --tests
  --allow-dirty`): exit 0, "Finished `dev` profile … in 4.29s", zero
  warnings, clippy clean **incl. `--tests`** (the new test file is linted).
  No files were modified by the run — post-gate `git diff --numstat` on
  `codex-rs/apply-patch/` is identical to pre-gate (1/0 and 21/11).
- I did not run `just fmt` (not in my permitted gate set, and it writes);
  the evidence records `just fmt` exit 0 after the final green (evidence
  :352-356) and again after the parser.rs doc line (:438). The file
  state is consistent with a post-fmt tree: long unbreakable string
  literals remain (e.g. test-file :97, :329 — rustfmt does not split
  string literals at the repo's default `max_width` = 100,
  `codex-rs/rustfmt.toml` sets only edition/imports), and the wrapped
  `let patch =` / `format!(...)` assignment style rustfmt produces for long
  literals is present (e.g. :16-17, :23-24, :158-159).
- Other-seat concurrency note: gates were run sequentially by me after no
  other gate holder held the target lock; the Rust lock was respected,
  nothing was killed.

## 5. Mandate 4 — red/green evidence authenticity

Audited `docs/reviews/impl-item1-tdd-evidence.md` (443 lines, read in full)
as an input to verify, not a source of facts.

### 5.1 The RED (evidence :31-56) is a genuine capture of the NEW assertion
against the OLD code

- Command (:29): `just test -p codex-apply-patch streaming_parser` — at RED
  time the p2 module did not exist yet, so the filter matches exactly the
  11 fns of the inline `mod tests` (counted: 11) — matches "11 tests run"
  (:54).
- Panic site (:36): `apply-patch/src/streaming_parser.rs:835:9`.
  **Re-derived from the source:** at HEAD the ledger #2 `assert_eq!(` is at
  :835 (diff hunk `@@ -834,11 +845,10` puts old :834 = `let mut parser …`,
  old :835 = `assert_eq!(`). At RED time the only edit applied was the
  in-place rewrite of the expectation (which starts at old :837, below the
  macro call), so `assert_eq!(` remains at :835; column 9 is the
  8-space-indented macro position (matches the current file's indentation of
  that construct, now at :846). The final-tree position :846 = 835 + 8
  (module doc) − 1 (arm net) + 4 (test-module decl) — consistent. The
  evidence's own "assert :835-842" span (:23-24) is the same line numbering.
- Diff content (:39-52): `left` = `Err(InvalidHunkError { message: "'bad' is
  not a valid hunk header. Valid hunk headers: '*** Add File: {path}',
  '*** Delete File: {path}', '*** Update File: {path}'", line_number: 3 })`,
  `right` = `Ok([AddFile { path: "file.txt", contents: "bad\n" }])`.
  - `left` is the **old code's** actual output: the message string is
    byte-identical to the removed `format!` at HEAD :209-214 with
    `trimmed = "bad"`; `line_number: 3` is the third line of the pushed
    delta. I compared character-for-character.
  - `right` is the **new expectation** per spec §4 T1.6 — i.e. this is
    exactly "expected Ok / actual Err(InvalidHunkError) with the exact old
    message", as the brief requires for a genuine ledger RED.
- Retry semantics: `TRY 2 FAIL` (:34) is consistent with `retries = 1` in
  `codex-rs/.config/nextest.toml` (a deterministic assertion failure fails
  both attempts; `NEXTEST_PROFILE=local` inherits `default`).
- Summary (:54): "11 tests run: 10 passed, 1 failed, 88 skipped" → 99 total,
  matching the pre-check baseline green "99 tests run: 99 passed" (:18-19).
- (Nit N-1 below: the pasted FAIL line writes `codex-apply-patch
  streaming_parser::…` with a space where nextest's actual format — used in
  all of the evidence's own GREEN captures and in my gate re-run — is
  `codex-apply-patch::all streaming_parser::…`.)

### 5.2 Green locks are explicitly declared (breakdown §4 step 2)

- :95-102 states the red-state accounting: P2 is one behavior change, so
  there is exactly one red (ledger #2); sub-cases 3-10 "land as green
  locks"; sub-cases 11 (T1.11) and 12 (T1.13) are "regression locks with no
  prior red, stated explicitly below".
- T1.11 (:291): "**Regression lock: green at write time, NO prior red**"
  with the rationale (Update arm untouched; guards §3.4).
- T1.13 (:314): "**Regression lock: NO new test, NO prior red**" with the
  verification commands (git status on tests/, 25-dir count, scenario-suite
  pass in the final gate).
  ✓ The §4 protocol requirement ("sub-cases that are regression locks with
  no prior red say so explicitly in the record") is met.

### 5.3 Per-sub-case run arithmetic is consistent

Each sub-case's captured summary advances the total test count by exactly the
number of tests that sub-case adds (verified run-by-run):

| Evidence line | Sub-case | Tests added | Summary (run/passed/skipped) | Cumulative |
|---|---|---|---|---|
| :54 | RED ledger #2 | 0 (rewrite) | 11 / 10 / 88 (1 failed) | 99 |
| :78 | GREEN ledger #2 | 0 | 11 / 11 / 88 | 99 |
| :120 | T1.1 | +1 | 1 / 1 / 99 | 100 |
| :139 | T1.2 | +1 | 1 / 1 / 100 | 101 |
| :158 | T1.3 | +1 | 3 / 3 / 99 | 102 |
| :185 | T1.7 ×3 | +3 | 3 / 3 / 102 | 105 |
| :203 | T1.8 | +1 | 1 / 1 / 105 | 106 |
| :231 | T1.9 ×2 | +2 | 2 / 2 / 106 | 108 |
| :254 | T1.9b ×2 | +2 | 2 / 2 / 108 | 110 |
| :286 | T1.10 ×4 | +4 | 4 / 4 / 110 | 114 |
| :309 | T1.11 | +1 | 1 / 1 / 114 | 115 |
| :335 | gates | — | 115 / 115 / 0 | 115 |

99 (baseline) + 16 (new) = 115 — consistent end-to-end.

### 5.4 Gate outputs present and matching

- First gate set (:328-356): `just test -p codex-apply-patch` summary
  115/115 (:335) incl. `suite::scenarios::test_apply_patch_scenarios` PASS
  (:334); `just fix -p codex-apply-patch` "Checking … Finished", exit 0,
  clippy clean (:343-350); `just fmt` exit 0, idempotent (:352-356).
- Doc-line re-run set (:421-438): test 115/115 (:426), fix exit 0 (:429-436),
  fmt exit 0 (:438).
- My own re-run reproduces the test and fix results exactly (§4).

### 5.5 Final diff section matches reality

- :360-364: `git diff --stat -- codex-rs/apply-patch/` →
  "1 file changed, 21 insertions(+), 11 deletions(-)" — byte-for-byte my
  `--numstat` (21/11 on `streaming_parser.rs`); parser.rs +1 is covered by
  the later addendum.
- :365-370: untracked files = the p2 test file + the evidence doc — matches
  `git status`.
- :372-384: hunk-by-hunk description matches my hunk analysis (§1.2)
  (module doc; arm replacement with `+` branch/header check untouched; one
  test-module declaration; ledger #2 rewrite in the inline module).
- :386-388: the pre-existing seam files are listed as untouched by this item
  — matches `git status`.
- :440-442 line-ref note ("this doc line shifts every parser.rs line
  reference by +1; e.g. test_parse_patch :277 → :278") — verified against
  `git show HEAD:…parser.rs` :277 and the working tree :278.

### 5.6 Pre-check references all accurate at HEAD (evidence :8-19)

Arm :198-215 (header check :199, `+` branch :202-208, trailing Err :209-214),
P3.1 StartedPatch message :193, P3.2 DeleteFile message :222, boundary
messages :168/:184/:374, ledger #2 assert :838 in fn :814, ledger #1 :828 /
#3 :848, surviving asserts :819/:784/:797 — every one re-verified against
`git show HEAD:codex-rs/apply-patch/src/streaming_parser.rs`. ✓

### 5.7 Test bodies spot-checked against the evidence claims (≥4 required)

1. **T1.1** (test-file :10-39): evidence claims whole-object equality of the
   parsed hunk vectors **plus** equality against the exact content string —
   the test does both (`assert_eq!(raw_hunks, canonical_hunks)` :31 and
   `assert_eq!(raw_hunks, vec![AddFile { … }])` :32-38). Hand-traced: raw
   markdown lines (incl. blank lines) land verbatim via the new branch; the
   canonical `+`-prefixed version strips prefixes via the untouched branch →
   byte-identical contents. ✓
2. **T1.7 structural** (:93-128): evidence claims whole-hunk-vector equality
   (4 hunks incl. the UpdateFile chunk `z`). The test asserts the full
   `Vec<Hunk>` :101-127; the chunk shape (`old_lines ["z"]`, `new_lines
   ["z"]`, `context_line_indices [(0,0)]`, `is_end_of_file false`) matches
   `UpdateFileChunk::push_context_line` (`parser.rs`:138-143). ✓
3. **T1.9b both variants** (:216-232, :235-250): hand-traced — canonical
   `+*** End Patch` hits the `+` branch (structural check runs on the
   trimmed line, which is `+*** End Patch` ≠ `*** End Patch`) → content
   `line1\n*** End Patch\n`, matching the spec §3.2 probe; the raw variant
   hits the structural check first → truncation at `line1\n`. ✓
4. **T1.10 CRLF** (:303-317): `push_delta` strips one trailing `\r` per line
   before `process_line` (`streaming_parser.rs`:151) → `raw line\n`; the
   unchanged-path claim holds (that line is not in the diff). ✓
5. **T1.11** (:319-333): asserted message is byte-identical to the
   UpdateFile arm's error format (`streaming_parser.rs`:371) with line
   `bad`, `line_number: 3` (Begin=1, Update File=2, bad=3). ✓

### 5.8 No secrets / no PII beyond worker identity

Pattern scan (sk-/ghp_/xox/AKIA/private-key/password assignments) over the
evidence doc: clean. Only file paths, test output, and the worker identity
(`/root/item1_p2_worker`, :3). ✓

## 6. Mandate 6 — commit-readiness

- The four files form one coherent item: the behavior change (arm + doc
  note), its test surface (sibling file + ledger rewrite), and the TDD
  record. There is no partial state: the tree is green (§4), the red
  ledger assertion is rewritten green, and no Item-3 territory was
  touched (ledgers #1/#3/#4 and the parser-level messages are intact for
  the next item).
- No TODO/FIXME/dbg!/eprintln/todo!/unimplemented! in any item file
  (grepped). ✓
- No secrets in the evidence doc (§5.8). ✓
- The coordinator's planned commit message (spec refs, red/green
  evidence, review rounds) has everything it needs: spec §3.2/§3.4/§4 T1,
  evidence :31-56 (RED) / :78 (GREEN) / :335 + :426 (gates), this review
  round record.

## Findings

### B-N1 — Nit — RED capture line pasted with a non-nextest package/binary
separator

- **Where:** `docs/reviews/impl-item1-tdd-evidence.md`:34.
- **Evidence:** the pasted line reads
  `TRY 2 FAIL [   0.031s] (11/11) codex-apply-patch
  streaming_parser::tests::test_streaming_patch_parser_returns_errors` —
  `codex-apply-patch streaming_parser::…` (space). nextest's actual format,
  used in every GREEN capture in the same document (e.g. :76) and in this
  seat's own gate re-run, is `codex-apply-patch::all
  streaming_parser::…` (`::all` = the lib test binary).
- **Impact:** cosmetic transcription deviation on one captured line; it does
  NOT make red/green unverifiable — the panic line/column (:36), the
  left/right diff (:39-52), the summary line (:54), and the `retries = 1`
  semantics were each independently re-derived and match (§5.1).
- **Suggested resolution:** none required for commit; if the evidence file
  is ever revised, re-paste the raw line verbatim.

### B-N2 — Nit — "Final diff (item scope)" stat block predates the
parser.rs addendum

- **Where:** `docs/reviews/impl-item1-tdd-evidence.md`:358-370 vs :402-420.
- **Evidence:** the `git diff --stat` block (:360-364) lists only
  `streaming_parser.rs` (21/11) and the two untracked files; it was written
  before the coordinator folded the one-line `parser.rs` doc note into the
  item. The later section (:402-420) explicitly covers it ("`parser.rs` is
  the ONLY other file touched by this item; its diff is exactly `+1`
  line") followed by the gate re-run (:421-438).
- **Impact:** a reader stopping at the "Final diff" section would miss
  `parser.rs`; the document as a whole is complete and matches the actual
  tree (verified by this seat, §1.2). No verifiability gap.
- **Suggested resolution:** none required for commit; optionally fold the
  `parser.rs +1` line into the stat block when the evidence is archived.

### B-N3 — Nit — module-doc note points at a spec doc not yet in the repo

- **Where:** `codex-rs/apply-patch/src/streaming_parser.rs`:8 ("See
  `docs/responses-compat-apply-patch-format.md` §3.2").
- **Evidence:** the spec doc is untracked (`git status` shows `??
  docs/responses-compat-apply-patch-format.md`), and the item's commit is
  the 3 code files + evidence doc — so at commit time the in-code pointer
  dangles until the spec doc lands (breakdown Item 5: "Seam-doc updates,
  spec final pass").
- **Impact:** a committed reference to a not-yet-present file; resolves with
  the campaign's docs commits; no CI impact. The spec §3.2 implementation
  note mandates the *note itself* ("short 'lenient add-file content' note
  … discoverable on rebase"), not the file pointer.
- **Suggested resolution:** none required (coordinator call); e.g. commit
  the spec doc with or immediately after this item, or keep as-is since the
  pointer is part of the same campaign's documented plan.

## Re-verified clean

1. **Footprint:** exactly the 4 item files; every other working-tree
   modification is pre-existing seam state (incl. the `AGENTS.md` SDD
   block); nothing staged; `codex-rs/apply-patch/tests/` clean.
2. **Hunk minimality:** 4 hunks / +21−11 in `streaming_parser.rs`, +1/−0 in
   `parser.rs`; every added/removed line required by P2, breakdown Item 1,
   or the spec §3.2 implementation note; the `+` branch (:210-216) and the
   header check (:207-209) are byte-identical to HEAD (:202-208, :199).
3. **New arm semantics:** verbatim raw line + `\n`; structural check runs
   first; terminal `Err` removed as specified; `if let` guard mirrors the
   adjacent branch idiom; CRLF strip path (:151) untouched.
4. **Ledgers:** #2 rewritten in place in the existing inline `mod tests`
   (:845-852, whole-object `Vec<Hunk>` equality, `pretty_assertions`); #1
   (:836-843), #3 (:854-862), #4 (`tests/suite/tool.rs:393`) untouched;
   zero orphaned assertions of the removed AddFile error anywhere in the
   crate.
5. **Test-module conventions:** sibling file + explicit
   `#[path = "streaming_parser_p2_tests.rs"]` (:389-391); module name does
   not collide with the inline `mod tests` (:393-394); private module;
   16/16 fns are `#[test]`; `pretty_assertions::assert_eq` imported and
   used at all 20 assert sites; whole-object equality everywhere; no
   static-value tests; no negative tests for removed logic; T1.11/T1.13
   declared green locks with no prior red.
6. **Sub-case coverage:** T1.1-3, T1.6-11, T1.13 all covered (mapping
   table in §3); T1.4/T1.5/T1.12/T1.14 correctly belong to Item 3 and are
   absent.
7. **Conventions (AGENTS.md):** no new `format!` (so no inline-arg
   violations), no closures, no bool/ambiguous-`Option` params, no
   `/*param_name*/`-triggering literals, no new wildcard match arms, no new
   helpers, no `#[async_trait]`/`#[allow(async_fn_in_trait)]`;
   `streaming_parser.rs` non-test LoC = 387 (was 379) < 500; no
   TODO/FIXME/dbg!/eprintln; no `Cargo.toml`/lock changes.
8. **Gates (re-run by this seat):** `just test -p codex-apply-patch` →
   `115 tests run: 115 passed, 0 skipped`, exit 0; `just fix -p
   codex-apply-patch` → exit 0, clippy clean incl. `--tests`; tree
   unmodified by the runs (post-gate `--numstat` identical).
9. **Golden scope:** 25 fixture dirs unmodified;
   `suite::scenarios::test_apply_patch_scenarios` PASS in this seat's own
   run — canonical patches (incl. the `+` branch) parse byte-identically.
10. **Evidence authenticity:** RED = genuine new-assertion-vs-old-code
    capture (line-number re-derivation :835:9; old message byte-identical;
    `line_number: 3`; 99-test baseline; `retries = 1` explains `TRY 2`);
    green locks explicit; per-sub-case run arithmetic 99→115 monotonic;
    gate outputs present incl. doc-line re-run; final-diff stat matches the
    actual tree; all pre-check refs accurate at HEAD; 5 test bodies
    spot-checked against both the evidence and the implementation; no
    secrets.
11. **Invariants (breakdown §5):** parser-only change — no wire/API
    surface touched, OpenAI request bytes unchanged; no existence check
    added (apply layer untouched); no new public API; boundary messages
    unchanged (`streaming_parser.rs`:176, :192, :381); no Update/Delete
    leniency (arms untouched; T1.11 locks it).
12. **Commit-readiness:** one coherent item, green tree, no partial state,
    no debug leftovers, evidence doc clean of secrets.

## Counts

| Severity | Count | IDs |
|---|---|---|
| Blocking | 0 | — |
| Major | 0 | — |
| Minor | 0 | — |
| Nit | 3 | B-N1, B-N2, B-N3 |

**Seat B verdict: APPROVED** (0 Blocking, 0 Major, 0 Minor, 3 Nit). The
item may proceed to commit once the full round (both Item-1 seats) returns
0 Blocking + 0 Major.

— end of report (seat B, round 1) —
