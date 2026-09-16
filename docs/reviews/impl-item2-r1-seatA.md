# Item 2 (P1: function-tool spec text) — Independent Review, Round 1, SEAT A

- **Bead:** apex-ayl.52 · **Branch:** `feat/normalize-content-types-vllm` · **Working-tree HEAD:** `49d584e82e`
- **Reviewer:** SEAT A (spec/semantics lens), independent of seat B (conventions lens)
- **SoT:** `docs/responses-compat-apply-patch-format.md` (LOCKED v5) §3.1 + §4 T2; breakdown `docs/responses-compat-apply-patch-task-breakdown.md` §0/§5 + Item 2
- **Evidence under review:** `docs/reviews/impl-item2-tdd-evidence.md`

## Scope

Item 2's change set in the working tree (uncommitted; "At HEAD" = working-tree state per breakdown §0):

- `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` — two new file-local private consts (`APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION` :37, `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION` :42-72) + literal→const swaps at :83/:96 inside `create_apply_patch_function_tool`.
- `codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs` — imports :1-7, T2.3 expected values :50/:57, T2.1 drift-guard test :84-110, T2.2 self-consistency test :112-162.

Out of scope: Item 1 (P2 parser leniency, committed as `e21f608ac4`), all other uncommitted seam files (AGENTS.md, codex-api/*, apply_patch.rs, apply_patch_tests.rs, mod.rs, spec_plan.rs, model-provider/*, models-manager/*).

## Method

- `git diff HEAD` on both ownership files; `git show HEAD:` comparison for the freeform region.
- Byte-fidelity: python sliced the two fenced code blocks out of spec §3.1 and the two Rust const literals out of the source; byte-for-byte comparison (after confirming zero Rust escapes needed — no `\` or `"` in either text), char/word/byte counts, multi-byte inventory.
- T2.2 rule-sensitivity: python probes on the exact SoT bytes (naive `*** Begin Patch` marker search vs pinned exact-`Example:`-line rule) + code inspection of the parser boundary check the naive form would trip.
- Semantics: each taught rule cross-checked against `codex-rs/apply-patch` sources (`streaming_parser.rs` AddFile/UpdateFile arms, `invocation.rs` one-hunk-per-file enforcement, `parser.rs` boundary pre-pass + struct shapes) and against the function handler's actual parse entry point.
- Gate files (`spec_plan.rs`, `provider.rs`) verified untouched via mtime timeline triangulation (seam doc, item-1 evidence, item-2 evidence) and absence from the worker's final diff.
- Gates re-run from `codex-rs/` (patient, no PID kills): gate 1 `just test -p codex-core apply_patch`, gate 2 `just fix -p codex-core`, gate 3 `just fmt`.

## Findings

### Mandate 1 — Byte fidelity (mechanical)

Method: python sliced each fenced block out of spec §3.1 (first ``` after the label
heading up to the closing ```) and each Rust const literal out of the source
(from `= "` to the closing `";`), compared bytes, and counted. No Rust unescape was
possible to misbehave: neither text contains `\` or `"` (verified in both the spec
blocks and the source literals), so the plain `&str` literals carry the text verbatim.

| item | spec §3.1 block | Rust const | verdict |
| --- | --- | --- | --- |
| tool description | 126 chars / 126 bytes / 21 words | 126 chars / 126 bytes / 21 words | **byte-identical** |
| `patch` argument description | 2,198 chars / 2,204 bytes / 380 words | 2,198 chars / 2,204 bytes / 380 words | **byte-identical** |

- Definitive `bytes(spec_block) == bytes(rust_literal)` → `True` for both consts
  (`apply_patch_spec.rs:37`, `:42-72`).
- Counts match spec §3.1 "Length (v2, measured)" exactly: 126 + 2,198 = 2,324 chars;
  21 + 380 = 401 words. Bounded well under the 10K-token cap (spec: ≈550–615 tokens
  for this backtick-heavy text). §5 invariant holds.
- Multi-byte inventory identical on both sides: `—` ×2, `→` ×1 (3 bytes each → +6
  bytes; 2,198 + 6 = 2,204). The evidence's "2 extra bytes for the two multi-byte
  chars" shorthand is imprecise (it is 3 chars / 6 bytes) but the recorded numbers
  (2,198 chars / 2,204 bytes) are correct — see Discrepancy adjudications.
- Arg block first line starts `The ENTIRE patch as a single string: …` and last line
  is exactly `*** End Patch` on both sides (the spec's "ending in the `Example:`
  block" requirement).

**Verdict: PASS. Zero byte divergence.**

### Mandate 2 — Spec §3.1 clause-by-clause on the implementation

- **Builder shape unchanged** (`apply_patch_spec.rs:79-106`): one required string
  argument `patch` (`required: Some(vec!["patch"])`, `JsonSchema::string`); optional
  `environment_id` inserted only when `include_environment_id` (:86-93);
  `strict: false` (:97); `additionalProperties: false` via
  `JsonSchema::object(properties, Some(vec!["patch"]), Some(false.into()))`
  (:99-103, third arg confirmed as `additional_properties` in
  `tools/src/json_schema/types.rs:151-164`). `name: "apply_patch"`,
  `defer_loading: None`, `output_schema: None`. Matches §3.1 para 1 exactly.
- **Text lands in exactly the two prescribed places**: tool description at :96
  (`APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION.to_string()`), `patch` argument description
  at :83 (`APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION.to_string()`). No other
  string in the function changed.
- **Consts are file-local private, no new public API**: both are `const …: &str`
  without `pub`, file-scoped (`:37`, `:42`). The only `pub` item in the file,
  `create_apply_patch_function_tool`, is pre-existing seam state (uncommitted since
  the seam; not added by Item 2 — it is in the `git diff HEAD` as `+` only because
  HEAD predates the seam entirely; the pre-edit working tree already had it, per the
  evidence pre-check at `apply_patch_spec.rs:40` pre-edit / `:79` post-edit, which
  matches the current function location :79 exactly).
- **Freeform tool unchanged (T2.4)**: `git diff HEAD` on the freeform function shows
  exactly one added blank line after the `definition` let-binding; all non-blank
  lines are identical to HEAD (verified by programmatic diff: "identical ignoring
  blank lines: True"). The *emitted* freeform `ToolSpec` (name, description, lark
  grammar definition incl. the `environment_id` replace-branch) is therefore
  byte-identical to HEAD. Both freeform tests pass in every gate run below. The
  single blank line is a whitespace byproduct of the mandatory gates (worker
  discrepancy #4 — `just fix` let_and_return collapse of the seam's
  `let freeform = …; freeform` plus `just fmt`); it changes no emitted bytes. See
  findings list (nit N1).
- **No capability-gate change**: `spec_plan.rs:1257-1271` still gates on
  `provider.capabilities().apply_patch_function_tool` (function form for
  non-grammar-capable deployments, e.g. vLLM; freeform `ApplyPatchHandler`
  otherwise), and `provider.rs:372` sets
  `apply_patch_function_tool: !self.info.is_openai()`. Both files' diffs vs HEAD are
  the pre-existing seam (capability field + wiring + one seam test; registration
  site in spec_plan). Untouched-by-Item-2 evidence: (a) worker evidence pre-check
  ("Capability gate confirmed present and left untouched … No gate change in this
  item") and final-diff section (only the two ownership files + evidence doc in
  Item 2's delta); (b) mtime timeline — `spec_plan.rs` 2026-09-13 15:37:23 (seam
  era), `provider.rs` 2026-09-16 00:01:38 (item-1 window: between
  `streaming_parser.rs` 2026-09-15 23:54:55 and item-1 evidence doc 00:08:51;
  strictly before Item 2's session, whose files were last written 01:10:27 and
  whose evidence doc landed 01:11:30); (c) `docs/responses-compat-seam.md:147`
  lists the capability seam's ownership files as `provider.rs, apply_patch_spec.rs,
  apply_patch.rs, spec_plan.rs` — i.e. these are seam files, and Item 2's claim to
  have touched only its two is corroborated, not contradicted, by the timestamps.

**Verdict: PASS on all §3.1 clauses.**

### Mandate 3 — T2.1 drift guard

`create_apply_patch_function_tool_teaches_patch_format_in_argument_description`
(`apply_patch_spec_tests.rs:84-110`):

- **Exactly the 7 prescribed substrings, no more, no fewer** (:96-104), in the same
  order as spec §3.1's drift-guard note (v2 list): `first line is `*** Begin
  Patch``, `real newline characters`, `bare '+'`, `at most one hunk per patch`,
  `starts with '+'`, `multiple `@@` chunks`, `must change at least one line`.
  Cross-checked char-for-char against the spec list (spec §3.1 "v1, seat A m3"
  bullet and §4 T2.1). **No char-count assertions** — per the breakdown T2.1 step
  ("do not add extra assertions beyond the 7 substrings"); mechanical count
  verification was recorded in the worker's pre-check instead (and re-verified by
  this review, Mandate 1).
- **Asserted against the LIVE argument description**: the test calls
  `create_apply_patch_function_tool(/*include_environment_id*/ false)` (:87),
  extracts `parameters.properties["patch"].description` (:91-95) — not a
  copy-pasted string. A change to the text (or to the builder) trips the guard.
- All 7 substrings verified present in the spec SoT text (mechanical check), and
  the literal strings in the test are byte-identical to the spec-listed substrings
  (including the backticks and `+` quoting).

**Verdict: PASS.**

### Mandate 4 — T2.2 self-consistency (pinned extraction rule)

`create_apply_patch_function_tool_example_round_trips_through_parser`
(`apply_patch_spec_tests.rs:112-162`):

- **Pinned rule implemented exactly**: `description.lines()` →
  `.position(|line| *line == "Example:")` (:127-131) — first line that is EXACTLY
  `Example:` (full-line equality, not substring); `example` is the remainder
  (lines after it, to end of description, :132). This is the pinned rule from spec
  §4 T2.2 / §3.1 (breakdown §5 line 830: "split at first line exactly `Example:`").
- **Rule sensitivity probed (mechanically, on the exact SoT bytes)**:
  - The dangerous naive form — extracting the example by finding the `*** Begin
    Patch` marker substring — hits **line 0** (the first sentence: "the first line
    is `*** Begin Patch`…"), not the example's line 20. The naive 31-line extraction
    starts mid-sentence (`*** Begin Patch`, the last line is `*** End Patch`, and
    everything bet…`) so its first line ≠ `*** Begin Patch`; the parser's boundary
    pre-pass (`parser.rs:257` `check_start_and_end_lines_strict`, exact
    trimmed-line equality) would reject it deterministically. So a
    marker-substring-based extraction fails, and the exact-`Example:`-line rule is
    load-bearing, not decorative.
  - `Example:` as a substring occurs exactly once in the whole description (line
    index 19) and is the only line equal to `Example:` — the pinned rule's
    `position()` finds the intended line; no earlier false anchor exists.
  - The pinned rule's remainder: first line exactly `*** Begin Patch`, 11 lines,
    last line exactly `*** End Patch` (test asserts this at :133-136, matching the
    pinned requirement "the remainder must end with the line `*** End Patch`").
- **`parse_patch` on the remainder** (:137-138) — the same entry point the
  production function handler uses (`apply_patch.rs:411`
  `codex_apply_patch::parse_patch(patch_input)`), so the round trip exercises the
  real parse path (lenient mode, `PARSE_IN_STRICT_MODE = false`,
  `parser.rs:54`).
- **Whole-object equality** on the parsed args via
  `pretty_assertions::assert_eq` (:137-161), exactly the hunks prescribed by
  breakdown/spec §4 T2.2:
  - `Hunk::AddFile { path: notes/todo.md, contents: "# TODO\n\n1. ship the fix\n" }`
    — matches spec (§3.1 self-consistency note: "Add `notes/todo.md` =
    `# TODO\n\n1. ship the fix\n`").
  - `Hunk::UpdateFile { path: src/main.rs, move_path: None, chunks: [one
    UpdateFileChunk] }` with `change_context: Some("fn main")`, removal
    `    old_call();`, addition `    new_call();`, context `    shared();`
    (represented as `old_lines`/`new_lines` with the context line interleaved at
    index 1 and `context_line_indices: vec![(1, 1)]` — the canonical
    `UpdateFileChunk` representation, `push_context_line` at
    `parser.rs:138`), `is_end_of_file: false`.
  - `patch: example.clone()` (the trimmed remainder verbatim), `workdir: None`,
    `environment_id: None`.

**Verdict: PASS** (test also passes in this review's gate 1 run, below).

### Mandate 5 — Semantics: taught rules vs actual parser behavior

Cross-checked every rule in the two texts against the committed `codex-apply-patch`
sources (post-Item-1) and the function handler's parse path. Classification basis:
a taught rule the parser rejects = Blocking; a parser-accepted behavior wrongly
forbidden = Major.

| taught rule (text location) | parser/handler behavior (source) | verdict |
| --- | --- | --- |
| first line `*** Begin Patch`, last line `*** End Patch` (arg text line 1) | `check_start_and_end_lines_strict` requires exact trimmed-line equality (`parser.rs:257`) | accurate |
| real newlines, never `backslash + n` (arg text line 1) | parser is line-oriented (`patch.trim().lines()`, `parser.rs:195`); a literal 2-char `\n` stays inside a line and corrupts prefix parsing | accurate |
| no markdown fences / heredoc wrapper (arg text line 1) | fence first-line fails the Begin boundary check (rejected); the legacy `<<EOF…EOF` wrapper is unwrapped only by the pre-existing lenient pre-pass (`parser.rs:229-250`) — the prohibition is conservative, never blocks a valid patch | accurate (conservative) |
| Add File: every content line starts with `+`; bare `+` = empty line; `+42` file line → `++42` patch line; empty file = no content lines (arg text Rules 1) | committed P2 AddFile arm (`streaming_parser.rs:206-222`): `+`-prefix → content after `+` + `\n` (bare `+` → empty line; `++42` → `+42`); other lines appended verbatim (P2, Item 1). `AddFile` hunk pushed with empty contents at the header (`streaming_parser.rs:117-121`), closed by the next marker → empty file accepted | accurate, lossless for the canonical form |
| Update hunks: prefix every line ` `/`-`/`+`; multiple `@@` chunks in the single Update hunk (arg text Rules 2 + FORMAT) | ` ` / `+` / `-` branches each auto-create the first chunk when `chunks.is_empty()` (`streaming_parser.rs:316-349`); `chunks: Vec<UpdateFileChunk>` supports multiple; golden `scenarios/003_multiple_chunks` pins it; a second `@@` on a line-less chunk is rejected (`streaming_parser.rs:270-281`) — the text never teaches that as valid | accurate |
| "Never write raw (unprefixed) file content lines anywhere in the patch" (arg text Rule 3) | Update hunks: raw lines rejected (pinned spec §3.2 edge 11). AddFile: raw lines accepted verbatim (P2). The "anywhere" scope is slightly stronger than the parser's Update-only rejection; the forbidden deviation is either content-identical (verbatim) or lossy (raw line starting `+` loses one char — the spec §3.2 named "Lossy case", which the `++` rule exists to prevent) | accurate for Update; protective canonical-form instruction for AddFile — see N2 |
| at most one hunk per file; tool rejects repeats (arg text Rule 4) | `invocation.rs:235-239`: `changes.contains_key(&path)` → `Err("multiple operations target {path}")` on resolved paths | accurate |
| "An Update hunk must change at least one line" (arg text Rule 4) | zero-chunk Update (incl. rename-only `*** Move to`) → `ensure_update_hunk_is_not_empty` → "Update file hunk for path '…' is empty" (`streaming_parser.rs:62-79`); pinned upstream test | accurate for the rejection it targets; see N3 re: context-only no-op corner |
| rename without edit → `*** Delete File:` + `*** Add File:` (arg text Rule 4) | rename-only Update rejected (same empty-hunk error); Delete `old` + Add `new` are two distinct resolved paths → no one-hunk-per-file conflict, executes as delete+create | accurate |
| multiple hunks for different files, any order (arg text Rule 5) | hunks accumulate in order; standard multi-file parse (golden scenarios) | accurate |
| FORMAT block lines: `*** Move to:`, `@@ [context line]`, `<chunk lines>` prefixes, `*** End of File` | real parser features: `move_path`, `change_context`, prefix branches, `is_end_of_file` (`parser.rs:116-133`, `streaming_parser.rs` UpdateFile arm) | accurate |

**OpenAI-named providers unchanged**: the capability gate
(`provider.rs:372`: `apply_patch_function_tool: !self.info.is_openai()`, consumed at
`spec_plan.rs:1260-1269`) routes OpenAI-named providers to the freeform
`ApplyPatchHandler`, whose emitted `ToolSpec` is byte-identical to HEAD (Mandate 2).
The new text ships only on the function-tool path. ✓

**Verdict: no taught rule is rejected by the parser (no Blocking); no
parser-accepted behavior is forbidden in a way that blocks a legitimate outcome
(no Major).** Two nuance notes recorded as N2/N3.
**Verdict: no taught rule is rejected by the parser (no Blocking); no
parser-accepted behavior is forbidden in a way that blocks a legitimate outcome
(no Major).** Two nuance notes recorded as N2/N3.

### Mandate 6 — Evidence authenticity (this lens)

**RED genuineness.** The captured RED (evidence "Sub-case T2.1") is internally
consistent with a genuine deterministic first-missing-substring failure:

- Deterministic 2-try failure: `TRY 1 FAIL [0.274s]` + `TRY 2 FAIL [0.608s]` with
  the identical panic — `retries = 1` (`.config/nextest.toml`) produces exactly two
  tries, and an assertion failure fails both deterministically (unlike the
  environmental TMTs, which PASS on retry).
- Failing substring matches the first missing one: panic message `patch argument
  description missing: first line is `*** Begin Patch`` is the FIRST entry of the
  test's substring array (`apply_patch_spec_tests.rs:97`); the loop's first
  `assert!` is the one that panics. Pre-implementation the seam's short argument
  description contained none of the 7 substrings, so first-in-order is
  necessarily the failure point.
- Panic line consistent: `apply_patch_spec_tests.rs:100:9`. At RED time the file
  lacked the 5 new import lines (they arrived with the T2.2 test, per the TDD
  order) and the 42-line T2.2 test — so the drift test's `assert!` (final file
  :105) sat at :100, column 9 (8-space indent + `assert!`). Arithmetic checks out
  exactly.
- RED-run summary `111 tests run: 95 passed (15 flaky), 2 failed, 14 timed out,
  4251 skipped`: 111 = baseline 110 + the one new test ✓; the second failure is a
  different `suite::` integration test (`apply_patch_cli_updates_file_appends_
  trailing_newline`, TRY 2 only) — baseline-PASS, i.e. the flake identity moved
  again, consistent with load, not with Item 2 (text-only change).

**GREEN / per-sub-case count consistency.** The table (baseline 4 → RED 5 → final
6 unit tests; full filter 110 → 111 → 112) matches the diff exactly: T2.1 adds
one test (→111 at RED), implementation+T2.3 changes no test count, T2.2 adds one
(→112 final). GREEN captures show the right tests named
(`teaches_patch_format_in_argument_description`, `matches_expected_spec`,
`example_round_trips_through_parser`, both freeform tests) and
`112 tests run: 98 passed (14 flaky), 14 timed out, 4251 skipped` at gate 1 with
0 deterministic failures. All re-verified by this review's own gate runs below.

**14 first-try TMTs pre-existing.** The pre-Item-2 baseline run on the unmodified
tree shows the identical `14 timed out` with the same signature
(`slow-timeout period=30s, terminate-after=2` first-try terminations that PASS on
the configured retry — `.config/nextest.toml`: `retries = 1`, plus the documented
"Higher concurrency causes integration test timeouts under resource contention"
note). Load context recorded (14 cores, loadavg 16–23, concurrent unrelated
build); the `apply_patch_cli` group is already serialized
(`max-threads = 1` in nextest config), so cross-test concurrency is not the
mechanism — machine load is. Causal isolation for Item 2 is airtight: the change
is two string constants on a capability-gated path that the default
OpenAI-named test provider never takes (freeform, byte-identical — Mandate 2).

**Discrepancy adjudications (worker notes #1–#5):**

1. **Baseline integration failures (4 `suite::` timeouts at tree start) —
   ACCEPT as pre-existing.** Identical signature to the TMT class
   (`timeout waiting for event` at `core/tests/common/lib.rs:388`); 3 of 4 flip to
   PASS on isolated re-run; the persistent one
   (`apply_patch_exec_command_failure_propagates_error_and_skips_diff`) is a
   `skip_if_no_network!` real-shell test that fails 4/4 under loadavg 16–23 —
   environmental, and it rides the freeform path (Mandate 2), which Item 2 does
   not touch. Not attributable to Item 2. (CI/quiet-machine confirmation is the
   right follow-up; noted, not a review-loop blocker for a text-only change.)
2. **Gate 1 non-zero exit from TMTs — ACCEPT per baseline note.** The task
   contract's green criterion is "0 DETERMINISTIC failures; TMTs that recover on
   retry are recorded, not failing, per the baseline". The baseline establishes
   the 14-TMT condition pre-change; this review's independent gate 1 re-run
   (below) supplies a fresh data point.
3. **T2.3 as const references instead of duplicated literals — acceptable;
   classified minor (m), not B/M.** (Full reasoning in the findings list, M1.)
4. **Gate-2 `let_and_return` collapse in the freeform builder — acceptable.**
   Semantically identical (clippy let_and_return on the seam's uncommitted
   `let freeform = …; freeform`); net source delta in the freeform region vs HEAD
   is one blank line with all non-blank lines identical (verified, Mandate 2);
   both freeform tests green in every run; no emitted-byte change; disclosed.
   The blank line itself is recorded as nit N1.
5. **Gate-2 out-of-scope auto-fix to `core/tests/suite/openai_file_mcp.rs`,
   restored — ACCEPT.** Verified current state: file is clean vs HEAD
   (`git diff HEAD --stat` empty). It is not a seam file (seam doc ownership
   table, `docs/responses-compat-seam.md:147`, does not list it). The flagged
   import (`use wiremock::matchers::body_json`, `openai_file_mcp.rs:47`) is
   genuinely unused at HEAD — the only `body_json(` hits are the
   `set_body_json` method — so the worker's "clippy will re-fix it on the next
   `just fix`" note is correct; folding the one-line removal into the seam commit
   (worker's suggestion) is the right disposition. No seam content lost.
   (`just fix`)" note is correct; folding the one-line removal into the seam commit
   (worker's suggestion) is the right disposition. No seam content lost.

### Breakdown §5 invariants

- **No history-rewrite-style tricks**: nothing rewrites context/history; the
  change is two const string literals + test expectations. ✓
- **Bounded text**: 2,324 chars total (Mandate 1) ≪ 10K-token cap; per-item cap
  (no item >10K tokens) holds with wide margin; the >1K-token P0 highlight gate
  is not crossed (spec estimate ≈550–615 tokens for this backtick-heavy text). ✓
- **No new public API**: consts private/file-local; no `pub` additions
  (Mandate 2). ✓
- **No unrelated refactors**: both diffs purely additive (+78/+125) apart from the
  disclosed one-blank-line whitespace in the freeform function (nit N1, gate
  byproduct). No other file touched by Item 2 (Mandate 2/6). ✓

### Gates (re-run by this reviewer, from `codex-rs/`)

All three gates re-run by this reviewer (patient; no PID kills; log files under
`/tmp/seatA-*.log` on the review host).

**Gate 1 — `just test -p codex-core apply_patch`:**

```text
     Summary [3917.805s] 112 tests run: 65 passed (13 flaky), 13 failed, 34 timed out, 4251 skipped
error: recipe `test` failed on line 88 with exit code 100
```

- 112 total = the worker's final filter count (6 `apply_patch_spec` unit tests
  included) ✓. The non-zero exit is NOT a deterministic failure; adjudication:
  - **Unit tests (Item 2's surface): all green.** Dedicated re-run
    `just test -p codex-core apply_patch_spec`:
    `6 tests run: 6 passed, 4357 skipped`, exit 0 — including both new tests
    (`teaches_patch_format_in_argument_description`,
    `example_round_trips_through_parser`) and the T2.3-updated
    `matches_expected_spec` (0.1–0.2 s each — no load sensitivity).
  - All 13 both-try "failed" tests are `suite::` integration tests
    (`apply_patch_cli` ×11, `request_permissions` ×1, `unified_exec` ×1) — none
    in `tools::handlers::apply_patch_spec::tests`.
  - **Signature check (isolated re-run):** the 13 tests were re-run in isolation
    (same `just test` wrapper/`local` profile, 13 tests selected):
    `Summary [428.166s] 13 tests run: 8 passed (2 flaky), 2 failed, 3 timed
    out`. Every single failure/TMT in BOTH runs — including the two that
    failed on both isolated tries
    (`apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::
    coding_originator_keeps_repository_root_when_disabled` 53.0 s,
    `::web_work_uses_cwd_when_enabled` 24.8 s) — panics with the IDENTICAL
    environmental signature:
    `panicked at core/tests/common/lib.rs:388:14: timeout waiting for event:
    Elapsed(())`. Zero assertion/logic failures in either run. The sibling
    subtests of the same `turn_diff_skips_git_root` family PASS in these runs
    (and the whole family passed in the worker's lower-load runs), which rules
    out a deterministic logic defect: same code, same binary (tree unchanged —
    verified via `git status`/diff-stat before and after), different load,
    different outcome.
  - **Load context:** machine at loadavg 9.7–33.6 (14 cores, 52 users) across
    gate 1 (vs 16–23 recorded during the worker's run) — squarely the
    "resource contention" condition `.config/nextest.toml` documents
    (`slow-timeout = { period = "30s", terminate-after = 2 }`, `retries = 1`,
    serialized `core_apply_patch_cli_integration` group). The worker's
    pre-Item-2 baseline on the unmodified tree already showed the same
    signature class (4 failures, identical panic line).
  - **Causal isolation:** the failing tests run the default OpenAI-named test
    provider → freeform path, whose `ToolSpec` is byte-identical to HEAD
    (Mandate 2); Item 2's two new strings exist only on the
    capability-gated function-tool path. No plausible mechanism from a
    text-only const change to event-wait timeouts in these tests.
- **Green criterion (task contract): "0 DETERMINISTIC failures; TMTs that
  recover on retry are recorded, not failing, per the baseline" — MET**: 0
  deterministic failures observed; 13 flaky recovered on retry; 34 TMTs + the
  isolated-run TMT/FAIL set are the recorded environmental class (identical
  signature to the pre-Item-2 baseline). A clean zero-exit record will require
  a quiet machine or CI (same open note as worker discrepancy #2).

**Gate 2 — `just fix -p codex-core`:**

```text
    Checking codex-core v0.0.0 (/Users/palanisd/Projects/upstream/codex/codex-rs/core)
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 39.63s
exit 0
```

- Exit 0 ✓. The one auto-fix is EXACTLY the pre-existing unused import
  (`use wiremock::matchers::body_json;`, `openai_file_mcp.rs:47`) outside Item 2's
  ownership that worker discrepancy #5 predicted would recur
  ("clippy will re-fix it on the next `just fix -p codex-core`") — verified
  before restoring. Restored to HEAD with `git checkout --` (worker's
  documented handling; file verified clean vs HEAD afterwards), so the tree is
  left exactly as the worker handed it off. The two ownership files were
  UNCHANGED by this gate (diff vs HEAD still exactly +78/+125 — identical to
  the worker's final diff stat). No Item 2 lint issues: `apply_patch_spec.rs`
  and its tests are clippy-clean as committed-in-tree.

**Gate 3 — `just fmt`:**

```text
exit 0 (no output; no files modified — git status unchanged after the run)
```

- No reflow of the const text: post-fmt, both consts re-verified **byte-identical
  to the spec §3.1 blocks** (re-ran the mechanical comparison; `True`/`True`,
  counts 126/21 and 2,198/380).

**Gate conclusion:** all three gates satisfy their green criteria; the only
non-clean signal is the pre-existing, load-driven event-wait timeout class in
`suite::` integration tests, documented above and in the worker's baseline note.

### Findings list (classified)

**Blocking: 0**

**Major: 0**

**minor (m):**

1. **M1 — T2.3 expected values reference the consts instead of duplicating the
   spec §3.1 literals** (`apply_patch_spec_tests.rs:50,:57`). Spec/semantics
   adjudication of worker discrepancy #3: referencing the consts preserves T2.3's
   *shape* pinning (the full `ResponsesApiTool` — name, `strict: false`,
   `defer_loading`, parameter schema, `required`, `additionalProperties: false`,
   `output_schema` — is still asserted byte-for-byte) and the expected values do
   carry the spec §3.1 exact text (the consts are verified byte-identical to the
   SoT, Mandate 1), so the breakdown's "update the two expected literals … to the
   spec §3.1 exact text" is satisfied in value. The pinning value that is NOT
   preserved is byte-exact *ongoing* protection of the text itself: with consts on
   both sides, the text comparison is tautological. The argument description is
   still substantially pinned by T2.1 (7 key drift substrings) + T2.2 (the full
   `Example:` block byte-exact plus parse round-trip) — which is precisely the
   ongoing guard the SoT's own drift-guard design chose for it (§3.1 "v1, seat A
   m3" bullet: the 7 substrings, no byte-count assertions). The 126-char tool
   description, however, has no independent test pin at all (T2.1/T2.2 target only
   the argument description; the SoT prescribes no drift substrings for it), so a
   future silent edit of `APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION` would not trip any
   test. Mitigating constraints: the breakdown's T2.1 step explicitly forbids
   adding assertions beyond the 7 substrings (so an in-item tool-description pin
   would be out of scope), and the file's existing convention references
   `APPLY_PATCH_LARK_GRAMMAR` rather than duplicating the grammar text. Judgment:
   acceptable as shipped — not B/M — because the SoT's text is what it pins (the
   value equals the SoT, verified), the disclosed choice matches local convention,
   and the SoT's own guard design accepts substring-level ongoing protection; the
   residual gap (tool description unpinned; ~1.7 KB of argument prose outside the
   7 substrings + Example block unpinned) is a defense-in-depth observation for a
   later campaign step (e.g. a spec-level decision on tool-description drift
   substrings), not an Item 2 defect. Cites: `apply_patch_spec_tests.rs:44-66`
   (test), `:50,:57` (const refs), `apply_patch_spec.rs:37,:42` (consts).

**nit (n):**

1. **N1 — One whitespace-only blank line added in `create_apply_patch_freeform_tool`**
   (`apply_patch_spec.rs:23`, the blank line after the `definition` let-binding).
   Source delta vs HEAD in the freeform region is this line only; all non-blank
   lines identical, so the *emitted* freeform `ToolSpec` is byte-identical to HEAD
   (T2.4 satisfied at the spec-bytes level). Provenance: mandatory gate 2
   (`just fix` let_and_return collapse of the seam's `let freeform = …; freeform`)
   + gate 3 (`just fmt`), as disclosed in evidence discrepancy #4. No behavioral
   or wire impact. Cosmetic only — acceptable; recorded so the seam commit's diff
   is explained.
2. **N2 — "Never write raw (unprefixed) file content lines anywhere in the patch"**
   (arg text Rule 3) is slightly stronger than the committed parser, which
   accepts raw AddFile lines verbatim (P2, Item 1 — `streaming_parser.rs:215-220`).
   This is the LOCKED SoT text (copied byte-exactly, so not an implementation
   defect) and the prohibition is protective: the only parser-accepted deviation
   that would be blocked is either content-identical (verbatim append) or the
   named lossy corner (raw line starting `+` loses one char — spec §3.2 "Lossy
   case"), which the text's own `++` rule exists to prevent. No legitimate goal is
   unreachable under the taught rule. Informational; no action for Item 2.
3. **N3 — "An Update hunk must change at least one line"** (arg text Rule 4): the
   parser accepts a context-only chunk (no `+`/`-`) as a no-op —
   `ensure_update_hunk_is_not_empty` (`streaming_parser.rs:62-79`) only fires for
   zero-chunk hunks or a final chunk with no lines at all. The taught rule is
   directionally correct and non-blocking: a no-op Update hunk serves no purpose
   (any file state is achievable without emitting it), and the rule's target
   (zero-chunk / change-less hunks, the actual glm rejection class) is rejected
   exactly as taught. SoT text as locked; informational.
4. **N4 — Evidence byte shorthand** (evidence pre-check: "2,204 bytes = 2,198 + 2
   extra bytes for the two multi-byte chars `—` and `→`"): the inventory is 3
   multi-byte chars (`—` ×2, `→` ×1, 3 bytes each → +6 bytes); the recorded
   numbers (2,198 chars / 2,204 bytes) are nonetheless correct. Documentation
   imprecision only.

## Verdict

Item 2's change set is byte-faithful to LOCKED spec v5 §3.1, implements §3.1
clause-by-clause (builder shape, text placement, private consts, freeform
untouched, no gate change), implements T2.1 with exactly the 7 prescribed
substrings against the live description, implements T2.2 with the pinned
exact-`Example:`-line extraction rule (proven load-bearing by the naive-marker
probe) and whole-object parser equality, teaches only rules the committed
parser accepts (no taught rule is rejected; no parser-accepted behavior is
forbidden in a way that blocks a legitimate outcome), and its TDD evidence is
genuine (deterministic RED with consistent panic line; GREEN counts consistent
with the diff; TMT baseline pre-existing). Gates re-run independently: 0
deterministic failures, `just fix` exit 0 (recurrence of the predicted
out-of-scope one-line auto-fix, restored), `just fmt` exit 0 with const text
re-verified byte-identical. All §5 invariants hold (no history-rewrite tricks;
2,324 chars total bounded text; no new public API; no unrelated refactors).

**SEAT A round 1: APPROVED — 0 Blocking, 0 Major, 1 minor, 4 nit**
