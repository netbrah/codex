# Item 2 (P1 function-tool spec text) — Review round 1, Seat B (conventions / minimality / evidence)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` · Reviewer: SEAT B (independent; no
coordination with seat A) · Date: 2026-09-16 EDT · Item: "P1: function-tool spec text (spec T2; §3.1)".

## 1. Scope and method

**Reviewed change set** (per the execution baseline, breakdown §0 — "at HEAD" = working-tree state):

- `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` — +78 lines vs HEAD, 0 deletions.
- `codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs` — +125 lines vs HEAD, 0 deletions.

Both files also carry the uncommitted function-tool seam (present since 2026-09-13), so the whole-file
delta vs committed HEAD mixes seam code with Item 2's changes. Item 2's delta per the worker's evidence:

- spec file: consts `APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION` (:35-37) and
  `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION` (:39-72) + literal→const swaps (:83, :96);
- test file: imports (:2-5, :7), T2.3 expected-value updates (:50, :57), T2.1 drift-guard test (:84-110),
  T2.2 self-consistency test (:112-162 — the worker's evidence cites :112-152; see nit n-2).

**Method.** (1) Read spec v5 §3.1/§4 T2, breakdown Item 2/§0/§5/§9, the worker's TDD evidence, and the
companion seam doc. (2) Re-derived all mechanical claims from the tree: full `git diff HEAD` of both files,
line-by-line attribution against the worker's pre/post-edit line references, byte-identity of the const
texts against the spec §3.1 fences (independent python re-run), freeform-region comparison vs
`git show HEAD:`, 7-substring containment, extraction-rule trap analysis, type-shape checks
(`Hunk`/`UpdateFileChunk`/`ApplyPatchArgs`/`parse_patch`), and RED-capture line arithmetic. (3) Audited the
worker's raw run logs left in `/tmp` (`item2-baseline.log`, `item2-red.log`, `item2-green-t21.log`,
`item2-green-t22.log`, `item2-gate1.log`, `item2-gate2.log`, `item2-gate3.log`) against every
"captured, verbatim" block in the evidence doc. (4) Re-ran all three gates from `codex-rs/` myself
(patiently; never killed by PID): `just test -p codex-core apply_patch`, `just fix -p codex-core`,
`just fmt`; verified the tree is unchanged in scope before/after (md5 + `git status`/`git diff --stat`).

Environment at my gate runs: 14-core machine, `vm.loadavg` 11.6–35.5 over ~65 minutes, a concurrent
unrelated `cargo test --release` (xai-grok build, PID 72016, started 01:15) plus the parallel seat-A
review's own gate runs (inferred from sustained load spikes and the appearance of
`docs/reviews/impl-item2-r1-seatA.md` mid-run). This is heavier than the worker's documented window
(loadavg 16–23) and materially affects integration-test timeout counts (see §4).

## 2. Mandate 1 — Repo conventions (Rust/codex-rs AGENTS.md, applied to Item 2's diff only)

| Convention | Verdict | Evidence |
|---|---|---|
| `pretty_assertions::assert_eq` import present once, at top | ✅ | `apply_patch_spec_tests.rs:6`, single import; every equality assertion in the file (T2.3 :46-65, T2.2 :133-161) resolves to it |
| Whole-object equality for structure comparisons | ✅ | T2.3 compares the entire `ResponsesApiTool` (:46-65); T2.2 compares the entire `ApplyPatchArgs` (:137-161) |
| `/*param_name*/` argument-comment convention | ✅ | All new/updated call sites self-document the bool: `/*include_environment_id*/ false` at :47, :87, :115; `/*include_environment_id*/ true` at :71; no other opaque positional literals in new code (string literals at :101, :90 are exempt) |
| No uninlined `format!` args | ✅ | No `format!` in the diff; the one interpolation uses inline `{substring}` (:107) |
| No collapsible-if / no `#[async_trait]` / no wildcard-match issues | ✅ | None introduced |
| No new public API | ✅ | Both consts are private (`const`, no `pub`) at :37/:42; `mod.rs:2` is `pub(crate) mod apply_patch_spec;` with no re-export of the consts; the only new `pub use` in `mod.rs` (`FunctionApplyPatchHandler`) is seam state, not Item 2 |
| No new std-adjacent helpers | ✅ | None |
| Test placement | ✅ | New tests went into the EXISTING `#[path = "apply_patch_spec_tests.rs"]` sibling module declared at `apply_patch_spec.rs:108-110` (pre-edit :70-71 per worker's pre-check); the "separate sibling file for NEW test modules" rule does not force a new file here |
| Module size | ✅ | Spec file 110 LoC, test file 162 LoC — far under the 500/800 targets |
| `assert!` (non-equality) in T2.1 | ✅ (no finding) | T2.1 has no equality assertions, so `assert_eq` does not apply; plain `assert!(…contains…)` matches pervasive in-repo precedent (e.g. the same file's HEAD test at :36-41; dozens across `core/tests/suite/`) |

**Findings from this mandate: none.** One comment-accuracy nit is recorded in §7 (n-3).

## 3. Mandate 2 — Minimality / footprint / attribution

**Stat (verified):** `git diff HEAD --stat` scoped to the two files = `+78/+125`, **0 deletions** — exactly
the campaign baseline. No other file in the worktree was left modified by Item 2's work: every other
modified file in `git status` is known pre-existing campaign/seam state (codex-api content_type_compat,
apply_patch.rs/apply_patch_tests.rs/mod.rs/spec_plan.rs, model-provider, models-manager, AGENTS.md), all
untouched by this item; `core/tests/suite/openai_file_mcp.rs` was **clean vs HEAD** at my start
(0-line diff — the worker's `git checkout --` restoration from discrepancy #5 verified).

**Line-by-line attribution (spec file, +78):**

- 4 lines — seam imports (`std::collections::BTreeMap`, `codex_tools::{JsonSchema, ResponsesApiTool}` +
  blank) — needed by the seam's `create_apply_patch_function_tool`, not by Item 2's consts.
- 1 line — whitespace-only blank at `apply_patch_spec.rs:22` inside `create_apply_patch_freeform_tool`
  (clippy/fmt artifact of the let_and_return collapse; see below).
- 38 lines — Item 2: the two consts incl. doc comments (:35-72).
- 33 lines — seam: `create_apply_patch_function_tool` doc + body (:74-106).
- 2 lines — blanks surrounding the new region (:73, :107).

**Line-by-line attribution (test file, +125):**

- 5 lines — Item 2: `codex_apply_patch::{ApplyPatchArgs, Hunk, UpdateFileChunk, parse_patch}` (:2-5) and
  `std::path::PathBuf` (:7) — needed only by T2.2.
- 23 lines — seam: `create_apply_patch_function_tool_matches_expected_spec` (:44-66), containing Item 2's
  T2.3 change (the two expected values now reference the consts at :50/:57).
- 15 lines — seam: `create_apply_patch_function_tool_includes_environment_id_when_requested` (:68-82),
  unchanged by Item 2.
- 27 lines — Item 2: T2.1 drift-guard (:84-110). 51 lines — Item 2: T2.2 round-trip (:112-162).
- 4 lines — inter-test blanks.

The worker's own contribution counts (≈39 spec / ≈85 test) reconcile with this exactly. The HEAD freeform
tests (:9-42 worktree) are byte-identical to HEAD except one added trailing separator blank before the
seam's first test; the HEAD freeform **function** region differs from HEAD by exactly one whitespace-only
blank (see m-2).

**Freeform region vs HEAD (discrepancy #4 core question).** I diffed
`create_apply_patch_freeform_tool` body `git show HEAD:` vs worktree byte-for-byte: **not** byte-identical —
the worktree has one extra blank line between `};` and `ToolSpec::Freeform(...)` (worktree :22; absent at
HEAD :17-18). Everything else in the function (name, description string, lark grammar usage, env-id
`replace()` text) is byte-identical. Consequences:

- The emitted wire bytes for `OpenAI`-named providers are unchanged (no string changed) — §5 invariant holds.
- T2.4 stands: both freeform tests pass unmodified in every run (worker's raw logs and mine).
- The let_and_return collapse itself is **acceptable and low-risk**: semantics identical (clippy
  `let_and_return` is a pure style lint); re-introducing `let freeform = …; freeform` would be re-fixed on
  every subsequent `just fix -p codex-core` (the per-item gate), so keeping the direct return is the stable
  fixed point — confirmed: my own gate-2 run applied **no** fix to `apply_patch_spec.rs`. No spec/breakdown/
  seam doc references the `freeform` binding (grep of `docs/`: only the evidence doc's own description and
  review reports). Risk of the item commit silently dropping the seam's let-hunk: **LOW** — the committed
  freeform function will read as `HEAD + one blank line`, which is clippy/fmt-stable (my gates 2/3 prove no
  further churn). The inaccuracy in the evidence's "matches HEAD exactly" claim is recorded as m-2.
- **No other file was left modified by the worker's clippy run**: my gate-2 re-run fixed exactly one file —
  `core/tests/suite/openai_file_mcp.rs (1 fix)` — the predicted unused-import re-trigger (discrepancy #5),
  removing only `use wiremock::matchers::body_json;` (:47). I recorded it and restored the file to HEAD
  with `git checkout --` (mirroring the worker's handling), so the tree is left exactly as the worker left
  it. Expect the same re-trigger in every future `just fix -p codex-core` (Items 3-6); concur with the
  worker's suggestion to fold the one-line removal into the seam commit.

## 4. Mandate 3 — Evidence authenticity (deep audit)

The worker left raw run logs in `/tmp` (`item2-baseline.log`, `item2-red.log`, `item2-green-t21.log`,
`item2-green-t22.log`, `item2-gate1.log`, `item2-gate2.log`, `item2-gate3.log`). Every "captured, verbatim"
block in the evidence doc matches those logs line-for-line (durations included) — the evidence is genuine.

**(a) RED capture — genuine first-fail, deterministic.** `/tmp/item2-red.log`: `TRY 1 FAIL [0.274s]` and
`TRY 2 FAIL [0.608s]`, both panicking at `apply_patch_spec_tests.rs:100:9` with
`patch argument description missing: first line is `*** Begin Patch`` — the first of the 7 substrings, i.e.
the loop's first iteration. Plausibility vs the old text: the old seam description (seam doc §3.2: "same
guidance as the freeform spec, plus an explicit 'put the ENTIRE patch text in `patch`; do not wrap in JSON
or markdown'", ~160 chars) contains **none** of the 7 substrings; the new text contains **all 7**
(re-verified in python). The panic line is exactly right for the RED-time tree: in the post-edit file the
`assert!` sits at :105:9; the RED-time tree had 5 fewer import lines (the T2.2-only imports :2-5/:7 did not
exist yet — consistent with the breakdown's step order and the worker's pre-edit refs, test :40 → post :45,
a shift of exactly 5), placing the assert at :100:9. ✅

**(b) Count arithmetic — all consistent.** Module unit tests 4→5→6 (raw green logs: 5/5 and 6/6 pass;
"4357 skipped" at 5-run = total 4362 = baseline total 4361 + T2.1, confirming nextest's run+skipped
arithmetic). Full filter 110→111→112 (raw baseline/RED/gate-1 summaries). Final gate "112 run: 98 passed
(14 flaky), 14 timed out" — 98+14=112 ✅. nextest config reconciles: `retries = 1`,
`slow-timeout = { period = "30s", terminate-after = 2 }`, and `core_apply_patch_cli_integration`
`max-threads = 1` (the flaky class is the serialized full-turn group). The pre-Item-2 baseline claim of
"identical 14 TMTs" is documented in the evidence (Baseline failure note, with the raw run summary) and I
verified it against the raw baseline log — **stronger than documented: not only the count (14) but the
identities match exactly** (`comm` on the deduped TMT name sets: 14 shared, 0 baseline-only, 0 gate1-only).
One inaccuracy found: see m-1 (the TMTs did **not** recover on retry).

**(c) Byte-identity — independently re-derived.** I sliced both §3.1 fences from the spec and the two Rust
literals from the worktree file with a fresh python script and compared:

- tool description: 126 chars / 126 bytes / 21 words — byte-identical ✅
- `patch` argument: 2,198 chars / 2,204 bytes / 380 words — byte-identical ✅
- total 2,324 chars / 401 words ✅ (matches spec §3.1 "Length (v2, measured)" at :376-377)
- neither literal contains `\` or `"` → plain multi-line `&str` (not a raw string), no escapes; this is why
  source text == value. The const text survived `just fmt` unchanged — my gate-3 run produced zero output
  and the file's md5 is identical pre/post (rustfmt does not reflow multi-line string bodies).
- One inaccuracy in the evidence's supporting footnote: it says "2,204 bytes = 2,198 + **2** extra bytes for
  the **two** multi-byte chars `—` and `→`" — actually there are **three** multi-byte chars (two em-dashes
  at offsets 215/361 + one arrow at 1292) = **+6** bytes; the headline 2,198/2,204 numbers are correct
  (see n-1).

**(d) T2.4 freeform evidence — present.** Both freeform tests appear green in every worker run (baseline,
RED, both GREEN captures, gate 1 — verified in the raw logs: `create_apply_patch_freeform_tool_matches_
expected_spec` and `create_apply_patch_freeform_tool_includes_environment_id_when_requested`), and in my
independent gate-1 run both pass as well. No edits to the freeform spec or its tests. ✅

## 5. Mandate 4 — Gates (re-run by me from `codex-rs/`)

Tree-state guard: md5 of both in-scope files taken before and after all three gates — **byte-identical
throughout**; `git status`/`git diff --stat` confirm the two files remain `+78/+125, 0 deletions` and no
new in-scope modification appeared (a transient `openai_file_mcp.rs` modification from the parallel seat-A
gate runs was observed and is clean in the final state; see §3).

**Gate 1 — `just test -p codex-core apply_patch`** (~65 min at loadavg 11.6–35.5; worker's was 8 min at
16–23): `Summary [3879.585s] 112 tests run: 66 passed (7 flaky), 12 failed, 34 timed out, 4251 skipped`,
exit 100 (non-zero, environmental).

- **All six `apply_patch_spec` unit tests PASS in my independent run** (T2.1 :22/112, T2.2 :18/112, T2.3
  :23/112, function env-id :21/112, both freeform :17/:19/112) — Item 2 is green.
- **0 logic/assertion failures attributable to the item.** All 12 terminal failures panic at
  `core/tests/common/lib.rs:388` with `timeout waiting for event: Elapsed(())` — the identical
  load-flake signature of the baseline's 4 pre-existing failures (unmodified tree). The 34 terminal
  TMTs (TMT on both tries, ~60s each) are the same slowest full-turn integration class; the count grew
  14→34 monotonically with load (my window was 2–3× heavier, including two overlapping nextest runs and
  the xai-grok release build).
- One distinct failure mode appeared in my run only:
  `code_mode_can_apply_patch_via_nested_tool` failed TRY 1 in 2.28s on `assert_eq!(items.len(), 2)`
  (left=1, right=2, `code_mode.rs:5413`) then TMT'd on TRY 2. **Not item-attributable:** (1) mechanism
  exclusion — the test runs the default `OpenAI`-named test provider → freeform path;
  `create_apply_patch_function_tool` is never constructed, and the freeform spec/wire bytes are unchanged
  (verified); Item 2 is text-only on the non-OpenAI path. (2) The same test TMT'd on **both** tries in the
  pre-Item-2 baseline log and in the worker's gate-1 log — under load it cannot even complete a turn.
  (3) In my isolated re-run it TMT'd on both tries again (120.5s, same lib.rs:388 signature). It is a
  pre-existing load-flaky test, now at the edge of failing even isolated on this loaded machine.
- Verdict per mandate: **green = 0 deterministic failures** — holds. No NEW deterministic (logic) failures;
  everything non-green in my run carries the documented load signature or is mechanism-excluded. The
  non-zero exit is the same pre-existing environmental condition the worker flagged (discrepancy #2); a
  quiet-machine/CI re-run would give a clean zero-exit record (worker's own recommendation, endorsed).

**Gate 2 — `just fix -p codex-core`**: exit 0 (4m29s). Exactly one auto-fix: `core/tests/suite/openai_file_mcp.rs
(1 fix)` — the predicted unused-import re-trigger (removes `use wiremock::matchers::body_json;`). No fix
touched either in-scope file (`apply_patch_spec.rs` is clippy-clean; the let_and_return collapse is stable).
Recorded; file restored to HEAD afterwards (worker precedent; I did not fix it myself).

**Gate 3 — `just fmt`**: exit 0, zero output, in-scope files byte-identical (multi-line const literal
confirmed fmt-inert).

## 6. Mandate 5 — Test quality

- **T2.1 asserts exactly the 7 substrings and nothing else** (:96-104): no char-count assert, no extra
  pinning — per breakdown Item 2 step 2 ("do not add extra assertions beyond the 7 substrings"). ✅
- **T2.3 still compares the WHOLE `ResponsesApiTool` object** (:46-65) — name, description, strict,
  defer_loading, full parameters (BTreeMap, required, additionalProperties), output_schema — not just the
  description field. ✅
- **T2.2 builds the expected `ApplyPatchArgs` with full structure** — hunks (`AddFile{path, contents}`;
  `UpdateFile{path, move_path, chunks:[UpdateFileChunk{change_context, old_lines, new_lines,
  context_line_indices, is_end_of_file}]}`), `patch`, `workdir`, `environment_id` — whole-object equality
  via `pretty_assertions::assert_eq`, not field-by-field. Field shapes verified against
  `parser.rs:67-83/:116-128` and `lib.rs:152-157`. The expected values match spec §4 T2.2 exactly
  (`# TODO\n\n1. ship the fix\n`; one chunk, context `fn main`, 1 removal / 1 addition / 1 context,
  `context_line_indices [(1,1)]`). The pinned extraction rule is implemented correctly: split at the first
  line EXACTLY `Example:` (the naive trap is real and verified — the description's line 0 contains the
  `*** Begin Patch` phrase; the actual marker line is line 20; `Example:` occurs exactly once as an exact
  line), remainder must end with `*** End Patch` (asserted), remainder parsed via `parse_patch`
  (`parser.rs:146`, `&str` signature matches). ✅
- No negative tests for removed logic; no tests for statically defined values (T2.1/T2.3 assert on the
  constructed tool output/wiring and T2.2 is a behavioral round-trip — none is a const-equals-const
  tautology). ✅
- Test placement correct (existing `#[path]` sibling module). Naming follows the file's
  `create_apply_patch_*` convention. ✅

**Findings from this mandate: none** (the T2.2 comment-accuracy nit n-3 is recorded in §7).

## 7. Findings list (severity: Blocking / Major / minor / nit)

- **m-1 (minor) — evidence doc misstates the TMT retry outcome (Gate 1 note).**
  `docs/reviews/impl-item2-tdd-evidence.md` (Gates §1) claims the 14 "timed out" entries "are first-try
  slow-timeout terminations … that PASS on the configured retry" and "all 112 tests end green". The
  worker's own raw log (`/tmp/item2-gate1.log`) shows all 14 with `TRY 2 TMT` — i.e. terminal timeouts on
  BOTH tries; they did not end green, and the non-zero exit comes from those 14 terminal timeouts. (Nextest
  summary buckets are disjoint: 98 passed + 14 timed out = 112.) The attribution conclusion is unaffected
  and, if anything, better than documented: the 14 TMT identities match the pre-Item-2 baseline exactly
  (verified name-for-name, not just count). The raw logs are in `/tmp` and let any reviewer confirm in
  seconds. Severity minor: an inaccuracy in the evidence record about a verifiable fact, no item-scope or
  verdict impact.

- **m-2 (minor) — evidence discrepancy #4 "matches HEAD exactly" is off by one whitespace line.**
  `apply_patch_spec.rs:22` (blank line inside `create_apply_patch_freeform_tool`, between `};` and
  `ToolSpec::Freeform(...)`) does not exist in `git show HEAD:` (HEAD :17-18) — the freeform region is
  `HEAD + one whitespace-only blank`, not byte-identical. No functional impact (no string changed;
  wire bytes identical; T2.4 green; clippy/fmt-stable — my gates 2/3 produced no further change). The item
  commit will carry this whitespace hunk and silently drop the seam's `let freeform` hunk: risk LOW,
  acceptable per mandate 2 (clippy-mandated collapse, re-fix loop otherwise). The evidence claim should be
  corrected in the record to "matches HEAD except one whitespace-only blank".

- **m-3 (minor) — discrepancy #3 adjudication: T2.3 references the consts instead of duplicating the
  literals.** The re-specified T2.3 (breakdown §9 round-1 row 1) says "update the two expected literals …
  to the spec §3.1 exact text"; the worker made the expected values
  `APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION.to_string()` / `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION
  .to_string()` (test :50/:57). Effect: the expected object still carries the exact spec text and the
  whole-object comparison stands, but T2.3 now compares builder output against the same consts the builder
  uses — it no longer independently pins the 2,198-char text to the spec; text pinning rests on T2.1 (7
  substrings) + T2.2 (example block) + the mechanical byte-identity check. Weighing: the SoT spec §3.1
  itself mandates const storage ("stored as a `const` next to the function"); the file's existing pattern
  does the same for `APPLY_PATCH_LARK_GRAMMAR` (:22); duplicating a 2,198-char literal would create a
  second source of truth that every text edit must update twice; the worker flagged the deviation
  proactively for exactly this loop. **Adjudication: acceptable (minor, not Major)** — residual drift risk
  (const text changing outside the 7 substrings and the Example block without tripping T2.3) is real but
  bounded, mitigated, and consistent with the campaign's own single-source-of-truth precedent. If the
  coordinator prefers strict independent pinning, it is a two-line swap to literal duplication — cheap
  either way; my lens does not require it.

- **n-1 (nit) — evidence byte-arithmetic footnote.** Pre-check: "2,204 bytes = 2,198 + 2 extra bytes for
  the two multi-byte chars `—` and `→`" — there are three multi-byte chars (two em-dashes + one arrow) =
  +6 bytes. Headline numbers (2,198 chars / 2,204 bytes / 380 words) are correct and re-verified.

- **n-2 (nit) — evidence line-cite for T2.2.** "…:112-152 new T2.2 self-consistency round-trip test" — the
  test actually spans :112-162 (10 lines short; :152 is mid-`UpdateFileChunk`). Content verified correct.
  (The same short cite propagated into the coordinator task contract.)

- **n-3 (nit) — T2.2 test comment misstates the pinned rule's rationale.**
  `apply_patch_spec_tests.rs:124-126`: "a naive substring search is wrong because the first sentence
  contains the same phrase" — the first sentence contains no `Example:`; per spec §4 T2.2 the hazard is
  that the first sentence contains the `*** Begin Patch` phrase (verified: description line 0 vs marker
  line 20), which is why the split key is the exact line `Example:`. The implementation is correct per the
  pinned rule; only the comment's explanation is garbled.

- **Discrepancy #1 (baseline integration flakes) — adjudicated: sufficient to exclude the item.** From the
  minimality/evidence side: the baseline run (unmodified tree) is documented in the evidence with its raw
  summary, and the raw log (`/tmp/item2-baseline.log`) confirms it: 4 failures, all
  `timeout waiting for event: Elapsed(())` at `core/tests/common/lib.rs:388`, same 14 TMTs (by identity),
  documented load (14 cores, loadavg 16–23, concurrent xai-grok build) and the repo's own
  `.config/nextest.toml` contention note. Flake identity shifts run-to-run while the new unit test fails
  deterministically — the evidence's characterization is accurate on every point I re-checked. The one
  test that failed 4× in a row (`apply_patch_exec_command_failure_propagates_error_and_skips_diff`) is a
  real pre-existing flake worth a quiet-machine/CI look (worker's note, endorsed) — but it is pre-Item-2
  and out of file ownership; it even flake-passed in my heavier-load run.

- **Discrepancy #2 (TMT exit code) — adjudicated: sufficient to exclude the item, with the m-1 correction.**
  The non-zero exit is fully explained by terminal slow-timeout kills; the identical 14 TMT identities in
  the unmodified-tree baseline make an item-caused regression implausible (text-only change on a
  capability-gated non-OpenAI path; the affected tests run the OpenAI/freeform path with byte-identical
  wire bytes). My own heavier-load re-run (34 TMT, 12 load-signature failures, 1 mechanism-excluded
  assertion flake) is monotonic with load and contains zero item-attributable failures. Coordinator
  action suggested (worker's, endorsed): one quiet-machine or CI gate-1 run for a clean zero-exit record
  before commit.

- **Discrepancy #5 — verified and closed.** `openai_file_mcp.rs` was clean vs HEAD at my start; my gate-2
  re-triggered exactly the one-line unused-import removal (recorded in §5); I restored it to HEAD, leaving
  the tree as the worker left it. Future `just fix -p codex-core` runs (Items 3-6) will re-trigger it;
  fold the removal into the seam commit (worker's suggestion, endorsed).

## 8. Invariants (§5) — item 2

- **Bounded size:** 2,324 chars total ≈ 550–615 tokens (spec §3.1 BPE estimate) — ≪ the 10K-token cap and
  under the 1K P0-review threshold. ✅
- **No new public API surface:** consts private, no re-exports; no `Cargo.toml`/lock changes. ✅
- **No unrelated refactors:** the only candidate is the clippy `let_and_return` collapse (+ whitespace
  blank) in the freeform function — adjudicated acceptable in §3/m-2 (clippy-mandated, semantics identical,
  fmt/clippy-stable, no doc references the binding). ✅
- **Outbound request bytes for `OpenAI`-named providers unchanged:** freeform name/description/grammar/env
  bytes byte-identical to HEAD; both freeform tests green. ✅
- **Non-OpenAI only, no gate change:** function path remains capability-gated (`spec_plan.rs:1257-1271` +
  `provider.rs:372`), untouched by this item. ✅

## 9. Verdict

Item 2's change set is minimal, convention-clean, and byte-faithful to spec §3.1 (independently
re-verified). The TDD evidence is authentic (every "verbatim" capture matches the worker's raw `/tmp`
logs; RED is a genuine deterministic first-fail with internally consistent line arithmetic; all counts
reconcile, including identity-exact TMT parity with the unmodified-tree baseline). All three gates
re-run by me: unit tests green in an independent run, 0 deterministic/item-attributable failures, item
clippy-clean and fmt-clean, tree unchanged in scope. The findings are record-accuracy issues in the
evidence doc (m-1, m-2, n-1, n-2, n-3) and one defensible, proactively-flagged deviation (m-3) — none
affects the item's correctness or scope.

**SEAT B round 1: APPROVED — 0 Blocking, 0 Major, 3 minor, 3 nit**
