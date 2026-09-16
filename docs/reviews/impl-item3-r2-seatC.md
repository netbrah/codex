# Item 3 (P3 teachable errors) — Round 2 Review, SEAT C (spec/semantics delta lens)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` · Review date: 2026-09-16
Reviewer: SEAT C (fresh agent; round 1 was seats A/B — reports
`docs/reviews/impl-item3-r1-seatA.md`, `docs/reviews/impl-item3-r1-seatB.md`;
evidence under audit: `docs/reviews/impl-item3-tdd-evidence.md`, 1105 lines,
incl. the new "## Round 1 fixes (r1 → r2)" section at :883)

Round 2 exists because round 1 returned REQUEST CHANGES from both seats with a
single shared Major (bare positional `None` at
`codex-rs/core/src/tools/handlers/apply_patch_tests.rs:57` missing the
`/*environment_id*/` argument comment — A:F1 / B:M-B1) plus evidence-doc
minors (A:F2/F3/F4; B:m-B2…m-B7). The worker applied the fixes (code fix at
07:06, doc corrections through 08:03). This review re-verifies the delta and
the invariants below; round 1's full findings remain in the record and are not
re-adjudicated here except where the delta could disturb them.

## Scope

Review-only. Five owned files:

| file | numstat (add/del vs HEAD) |
---|---|
 `codex-rs/apply-patch/src/parser.rs` | 8/6 |
 `codex-rs/apply-patch/src/streaming_parser.rs` | 5/6 |
 `codex-rs/apply-patch/tests/suite/tool.rs` | 1/1 |
 `codex-rs/core/src/tools/handlers/apply_patch.rs` | 265/60 |
 `codex-rs/core/src/tools/handlers/apply_patch_tests.rs` | 164/15 |

No product code edited; no `git add`/`commit`; the only file written by this
seat is this report.

## Method

1. Re-read both round-1 seat reports in full (findings lists, mandates,
   adjudications) and the evidence doc's new r1→r2 section (:883-1105) plus
   every section the fix wave touched (pre-check, step 2, step 3, step 5,
   step 6, step 7, final-diff).
2. Source re-verification of the Major fix at `apply_patch_tests.rs:57`:
   callee signature (`handlers/mod.rs:160-163`), name resolution (import
   chain + sibling-helper convention), column count, and byte-identity of the
   rest of the function (line-anchor cross-check against both r1 reports +
   file mtimes).
3. Numstat invariant re-counted from a fresh `git diff --numstat`.
4. Evidence-doc corrections verified against their stated sources: raw
   `/tmp/item3-*.log` and `/tmp/r1-*.log` files (all present) where the doc
   cites them; source code otherwise.
5. Independent python byte-fidelity re-slice of spec §3.3 message strings
   (soft-wrap joins re-derived) vs. all four handler/parser code sites and
   the test expectations.
6. Worker's r1-wave verification logs (`/tmp/r1-check.log`,
   `/tmp/r1-fmtcheck.log`, `/tmp/r1-gate-ap.log`, `/tmp/r1-gate-core.log`,
   `/tmp/r1-fix.log`, `/tmp/r1-fmt.log`) audited: quoted summaries compared
   field-by-field; identity sets (FAIL/TMT/FLAKY, 20 owned unit tests)
   reconciled programmatically against the raw logs.
7. Gates re-run by this seat from `codex-rs/` in order (see §Gates):
   `just test -p codex-apply-patch`, `just test -p codex-core apply_patch`,
   `just fix -p codex-apply-patch -p codex-core`, `just fmt`. Machine under
   heavy load (loadavg 15-35) the whole review; cargo lock serialized
   nothing (single seat running gates).

## Mandate 1 — the Major fix at `apply_patch_tests.rs:57`

Current line 57 (verified at source):

```rust
    let cwd = resolve_tool_environment(&step_context.environments, /*environment_id*/ None)
```

**(a) Comment/parameter name match — PASS.** `handlers/mod.rs:160-163`:
`fn resolve_tool_environment<'a>(environments: &'a TurnEnvironmentSnapshot,`
`environment_id: Option<&str>) -> Result<Option<&'a TurnEnvironment>,`
`FunctionCallError>`. The second parameter is named `environment_id`; the
comment is `/*environment_id*/` — exact match (including case/underscores).

**(b) Unqualified name resolves — PASS.** The test file is the sibling module
`mod tests` declared at `apply_patch.rs:845-847` (`#[cfg(test)]
#[path = "apply_patch_tests.rs"]`), its first line is `use super::*;`
(:1). The parent module's private import `use
crate::tools::handlers::resolve_tool_environment;` sits at
**`apply_patch.rs:29`** (the r1-fix section of the evidence doc cites
:32 — off by three; see finding n-C2); private parent imports are visible to
child modules both through the glob and through module-tree lookup, so the
unqualified name resolves to exactly the item the old qualified path named.
The same convention is already used in this file for the sibling helper
`apply_patch_file_update_mode` (private `fn` at `apply_patch.rs:64`, called
unqualified at test-file :87/:96) — a live proof that unqualified
parent-item resolution works in this exact module. No other
`crate::tools::handlers::` qualified reference remains in the test file
(grep: zero matches post-fix).

**(c) ≤100 cols, fmt-clean — PASS.** Measured: line 57 is **91 columns**.
Worker's `cargo fmt -p codex-core --check` (raw `/tmp/r1-fmtcheck.log`)
exited clean (only the stable-toolchain `imports_granularity` warnings, no
file listed). This seat's own `just fmt` gate (Gate 4) re-confirms with no
reflow — line 57 byte-identical pre/post (md5
`5da58afcfa7bd5afbf300fe882e778bc` before, after Gate 4; see §Gates).

**(d) Rest of the function byte-identical to round 1 — PASS (evidence
triangulated; no r1 snapshot commit exists to diff against directly).**
- File mtime: `apply_patch_tests.rs` is the only one of the five owned files
  modified after the round-1 work window — `Sep 16 07:06:36` vs.
  `parser.rs` 04:15:32, `streaming_parser.rs` 03:39:24, `tool.rs` 03:07:17,
  `apply_patch.rs` 03:45:50 (the r1 work window the seats recorded as
  03:07-05:16).
- Numstat identical to round 1 (164/15) — a same-line in-place edit preserves
  numstat; any added/removed line elsewhere in the file would move it.
- Every line-number anchor cited by the round-1 reports matches the current
  file exactly: `invocation_from_session` :51 (fn body ends :76),
  `invocation_for_payload` :78, pre-existing callers :102/:120 with the
  `let (invocation, _)` bindings at :107/:125, seam-helper tests :431/:441/
  :450/:462, T3.1 tests :470/:490 (expectations :484/:504), T3.2 :510,
  `pretty_assertions` import :16 / `serde_json::json` :17 (current-tree
  values after the T3.2 import additions).
- The full function body (lines 51-76) reads exactly as both seats
  described in round 1 (same `StepContext::for_test`, the two chained
  `.expect("primary turn environment must resolve"/"…must exist")`,
  `.cwd().clone()`, field-for-field `ToolInvocation` construction, returned
  `PathUri`) with line 57 the only difference from the r1 description (the
  old line was the fully-qualified call with bare `None`).

## Mandate 2 — numstat invariant

PASS. Fresh `git diff --numstat` at review start (re-confirmed after the
gates):

```text
8    6    codex-rs/apply-patch/src/parser.rs
5    6    codex-rs/apply-patch/src/streaming_parser.rs
1    1    codex-rs/apply-patch/tests/suite/tool.rs
265  60   codex-rs/core/src/tools/handlers/apply_patch.rs
164  15   codex-rs/core/src/tools/handlers/apply_patch_tests.rs
```

Exact match with the round-1 values recorded by both seats. Same-line comment
adds no lines, as predicted. No unannounced change.

## Mandate 3 — evidence-doc minor resolutions (each vs. its stated source)

| doc finding | correction in doc | verified against | result |
|---|---|---|---|
| m-B3 / B#6 | pre-check apply_patch.rs seam numstat `+321/−60` → `+265/−60` | fresh `git diff --numstat codex-rs/core/src/tools/handlers/apply_patch.rs` → `265 60` | **PASS** |
| m-B2 | step-2 "verbatim, first try" thread IDs: streaming `85275019`→`85281497`, CLI `85275777`→`85282092` | raw `/tmp/item3-red-step2.log`: TRY-1 panics at :173 `(85281497)` streaming `:836:9`, :265 `(85282092)` CLI `function.rs:250:5`; parser blocks `85281015`/`85280948` unchanged and still exact; no `85275xxx` ID exists anywhere in the raw log | **PASS** — all four "first try" blocks now match raw TRY-1 IDs exactly |
| m-B4 / F2 | step-6 table step-3 row → "3 fns / 10 assertions — the CLI fn went green; the streaming fn is still red on the DeleteFile assertion" | raw `/tmp/item3-red-step3.log`: `Summary [7.778s] 115 tests run: 112 passed, 3 failed`; failing fns = the two parser fns + streaming (panic `:855:9` both tries, 85289229/85289280); CLI fn absent from failure list (green); matches the doc's step-3 narrative "Ledger #1 and #4 green (12 → 10 red)" | **PASS** |
| m-B5 | import refs `:14/:15` → `:13/:14` (pre-check) + step-5 "already imported at :14" → ":13", with note that T3.2's imports shift them to :16/:17 in the round-1 file | `git show HEAD:…apply_patch_tests.rs` import block: `use pretty_assertions::assert_eq;` :13, `use serde_json::json;` :14 (identical at the 445-line pre-check state — pre-edit additions were tail-only); current tree: :16/:17 | **PASS** |
| m-B6 | `TurnContext::cwd()` → `TurnEnvironment::cwd()` (pre-check read-only refs + step-5 rationale) | `turn_context.rs`: `impl TurnEnvironment` block starts :77; :200 = `pub(crate) fn cwd(&self) -> &PathUri` (`&self.selection.cwd`); the deprecated `TurnContext.cwd: AbsolutePathBuf` field is at :314. Both doc sites now read `TurnEnvironment::cwd()` (doc :109, :415); the step-5 compile-fix note (:436-444) already self-corrected and is unchanged | **PASS** |
| F3 | step-7 log labels re-pointed: "First attempt" → `item3-step7-module.log`; DIAG narrative → `t32-debug` / `t32-diag` / `t32-diag2` in sequence; 20/20 → `t32-module.log` | raw `/tmp` files: `item3-step7-module.log` = 20 tests, `19 passed, 1 timed out`, `Summary [120.040s]`, F1 test TMT both tries 60.011/60.009 (pre-fix hang); `item3-t32-debug.log` = 1 test TMT both tries `Summary [120.035s]`, no DIAG; `item3-t32-diag.log` = compile failure E0425 (`assess_patch_safety` not in scope) + E0308; `item3-t32-diag2.log` = 1 test TMT both tries `Summary [120.031s]`, `grep -c DIAG` = 2; `item3-t32-module.log` = `20 passed`, `Summary [0.489s]`, F1 PASS 0.455s | **PASS** — all five labels now match the files' actual contents and mtimes (04:20/04:25/04:27/04:29/04:34) |
| m-B7 / F4 (no action) | doc item 7 records: parser.rs keeps std `assert_eq!` (accepted); T3.2 cwd cleanup best-effort; no stray `grok/` | this seat ran `find . -name grok -not -path './.git/*'` → **no match**; `grep pretty_assertions codex-rs/apply-patch/src/parser.rs` → no import (pre-existing file-local pattern, as round 1 recorded) | **PASS (no action, as recorded)** |

## Mandate 4 — semantic regression spot-checks (post-fix)

**(a) The 20 owned `tools::handlers::apply_patch::tests` unit tests** — see
Gate 2 below (re-run by this seat; all 20 must PASS). Corroboration from the
worker's r1-wave core run (raw `/tmp/r1-gate-core.log`): exactly 20 distinct
owned-fn identities carry a PASS line and zero owned fn appears in any
FAIL/TMT/FLAKY/SLOW-final state.

**(b) T3.1 exact-string assertions and the T3.2 F1 test untouched by the
fix — PASS.** Read the full bodies at :470-508 (T3.1 ×2) and :510-560
(T3.2): whole-`FunctionCallError` equality against the §3.3.3a/3b strings
(`r#"{}"#` payload; `json!({ "patch": { "raw": "not a string" } })`
genuinely non-string object), the F1 patch literal
(`grok/plans/spec-freeze-r23-glm.md`, H1 first content line, em dash,
`apex-ayl.45` artifact name), the custom-session shape
(`Constrained::allow_any(AskForApproval::Never)` +
`Constrained::allow_only(PermissionProfile::Disabled)` via
`make_session_and_context_with_auth_and_config_and_rx`), and the exact
created-file content assertion with best-effort cleanup — all byte-consistent
with the round-1 descriptions; no hunk in the fix wave touched any of them
(the only post-r1 mtime is line 57's function, whose rest is byte-identical
per Mandate 1d).

**(c) Spec §3.3 byte fidelity unchanged — PASS (re-derived independently,
4 of 6 strings in this seat; the numstat invariant covers the other two).**
Python re-slice of the spec backtick spans with soft-wrap joins re-derived
(first space after the wrap is the wrap space; no double spaces; all-ASCII):

| spec site | spec len | code site(s) | result |
|---|---|---|---|
| §3.3.3a (absent) | 121 | `apply_patch.rs:536`; T3.1 `apply_patch_tests.rs:484` | BYTE-IDENTICAL both |
| §3.3.3b (non-string) | 108 | `apply_patch.rs:543`; T3.1 `apply_patch_tests.rs:504` | BYTE-IDENTICAL both |
| §3.3.4a (Begin) | 182 | `parser.rs:269`; test literals `:282`, `:578` | BYTE-IDENTICAL all |
| §3.3.4b (End) | 149 | `parser.rs:272`; test literal `:645` | BYTE-IDENTICAL all |

`parser.rs` (mtime Sep 04:15, untouched by the fix wave) and
`apply_patch.rs` (mtime Sep 03:45, untouched) are both the two files the fix
did not touch; the fix wave's numstat neutrality (Mandate 2) independently
bounds the other sites.

## Mandate 6 — new-findings adjudication (the delta itself)

**Shadowing risk from the unqualified name: none.**
- No local definition or binding named `resolve_tool_environment` exists in
  the test module (grep: the name occurs once in the file — the call site).
- No other call site in the file uses the name, so nothing else resolves
  differently.
- The unqualified form resolves to the identical item the old qualified path
  named (single import in the parent, no second candidate through the glob —
  the glob re-exports the parent's namespace, which contains exactly that
  binding).
- Empirically: the worker's `cargo check -p codex-core --tests` (raw
  `/tmp/r1-check.log`) exited 0 with only the pre-existing
  `openai_file_mcp.rs:47` warning, and this seat's Gate 2 compiles and runs
  the module (20/20 PASS).

**Does the delta introduce any other new finding?** The delta is one line in
one file; the residual observations are documentation-only (this seat's
findings n-C2, n-C3, n-C4 in §Findings, all in the new r1→r2 section of the
evidence doc — none affects code, the ledger, or any invariant).

## Mandate 5 — gates (re-run by this seat, from `codex-rs/`)

Gate runs serialized in one background script
(`/tmp/seatC-gates.sh`), in the mandated order; machine load heavy the whole
window (loadavg 15-35 early, easing to ~11 by gate 2's end); no command was
interrupted.

**Gate 1 — `just test -p codex-apply-patch` → PASS (EXIT=0).**
`Summary [4.563s] 115 tests run: 115 passed, 0 skipped` (log
`/tmp/seatC-gate-ap.log`). 115/115 as expected; covers the 4 rewritten test
fns and all 13 ledger rows on the apply-patch side.

**Gate 2 — `just test -p codex-core apply_patch` → PASS per the binding
criterion (0 DETERMINISTIC failures).**
`Summary [5322.120s] 115 tests run: 50 passed (1 flaky), 2 failed, 63
timed out, 4251 skipped`, exit 100 (log `/tmp/seatC-gate-core.log`). 50+2+63
= 115 ✓; the run took 89 min — machine load materially worse than the
worker's r1-wave run (2642s) and round 1's (2117s).

Binding check (this seat, on the raw log):

```text
grep -E "thread .+ panicked" /tmp/seatC-gate-core.log | grep -v 'lib.rs:388' | sort -u
# → (empty)
```

All 117 panics in the run sit at exactly one distinct site,
`core/tests/common/lib.rs:388:14` (`timeout waiting for event:
Elapsed(())`) — the documented load-noise class. **Zero deterministic
failures.** Recorded non-pass identities (all lib.rs:388 class):

- Final FAIL (2, both `suite::apply_patch_cli`, process-spawning):
  `apply_patch_preserves_crlf_with_preserve_line_endings_feature`,
  `apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::web_work_uses_cwd_when_enabled`
  (panic sites verified per-identity at lib.rs:388).
- FLAKY (1, pass on retry):
  `apply_patch_turn_diff_paths_stay_repo_relative_when_session_cwd_is_nested`
  (TRY 1 lib.rs:388 panic → TRY 2 PASS).
- Final TMT (63): all process-spawning suite tests — 41 in
  `suite::apply_patch_cli` (incl. the surviving :707 substring twin
  `apply_patch_cli_rejects_invalid_hunk_header`, whose assertion path is
  exercised deterministically by the apply-patch crate's exact-stderr twin in
  Gate 1, 115/115), 3 `apply_patch_serialization`, 2 `approvals`, 1
  `code_mode`, 7 `hooks`, 1 `prompt_caching`
  (`gpt_5_tools_without_apply_patch_append_apply_patch_instructions`), 2
  `request_permissions`, 2 `request_permissions_tool`, 1 `shell_snapshot`, 2
  `tool_harness`, 1 `unified_exec` (full identities in
  `/tmp/seatC-gate-core.log`).
- **The 20 owned `tools::handlers::apply_patch::tests::*` unit tests: all
  20 PASS, none in any FAIL/TMT/FLAKY/SLOW-final state** (reconciled per
  identity from the raw log).

**Gate 3 — `just fix -p codex-apply-patch -p codex-core` → PASS (EXIT=0,
2m 52s).** Clippy auto-fixed exactly one file: `Fixed
core/tests/suite/openai_file_mcp.rs (1 fix)` — the documented, pre-existing,
OUT-OF-SCOPE unused `wiremock::matchers::body_json` import at :47 that
`cargo clippy --fix` re-triggers on every run (third independent
re-observation: worker r1 wave, seat A r1, this seat). Recorded per
mandate: pre-restore numstat `0 1` (the import line removed), remaining
`body_json` hits in the file are only the unrelated `set_body_json` method
names (:150/:166); restored via
`git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs`; post-restore
`git status` for the file is empty (clean). Zero clippy changes to any
owned file (post-gate numstat below unchanged).

**Gate 4 — `just fmt` → PASS (EXIT=0, silent — tree already fmt-clean).**
The `fmt` recipe runs `scripts/format.py` (Rust + justfile + Starlark +
Python); independent `cargo fmt --all --check` also exits 0 with no file
listed (only the stable-toolchain `imports_granularity` warnings).
**`apply_patch_tests.rs:57` is byte-identical pre/post fmt** (md5
`5da58afcfa7bd5afbf300fe882e778bc` captured before Gate 4 and re-measured
after) — no reflow, comment retained. No tests re-run after fix/fmt, per
repo rule.

**Post-gate scope integrity:** `git diff --numstat` on the five owned files
is unchanged from the pre-gate invariant (8/6, 5/6, 1/1, 265/60, 164/15);
`git status` (tracked) back to exactly the 17-file pre-review set (5 owned +
12 pre-existing branch files incl. AGENTS.md); nothing committed.

## Findings (severity, round 2 — delta lens)

| # | Severity | Where | Summary |
|---|----------|-------|---------|
| F-C1 | **minor** | `docs/reviews/impl-item3-tdd-evidence.md` "Round-1 final state" (:1097-1105) | `git status (tracked) = the 5 owned files + the 11 pre-existing seam files, nothing else` — off by one: the working tree carries **12** non-owned tracked modifications (5+12=17, re-counted by this seat; the same document's Final-diff section :871-878 enumerates all 12: AGENTS.md; codex-api content_type_compat{,_tests}.rs + responses.rs; core handlers/mod.rs + spec_plan.rs; model-provider amazon_bedrock/mod.rs + provider.rs; models-manager models.json + manager_tests.rs + model_info{,_tests}.rs). Bookkeeping only — no bearing on the code under review. |
| n-C2 | nit | evidence doc "Code fix" subsection (:885-922) | cites the parent import at `apply_patch.rs:32` — it is at **:29**; and cites `file_system_sandbox_policy_context_for_cwd` as a test-file sibling using the unqualified convention — that helper is not referenced in the test file (the live example is `apply_patch_file_update_mode`, unqualified at :87/:96). The described resolution mechanism (glob re-export of the parent's private import; module-tree visibility) is correct and the code is correct — line-ref/prose citation only. |
| n-C3 | nit | evidence doc r1 Gate-2 TMT list (:1029-1050) | the 15 `apply_patch_cli`-group names omit the leading `apply_patch_` prefix (e.g. doc `cli_multiple_chunks` = raw `apply_patch_cli_multiple_chunks`). Consistent with the original round-1 Gate-2 section's established shorthand (which round-1 seats verified without flagging); 36/36 count matches and every name is 1:1 reconstructable (verified programmatically against the raw log). |

No Blocking findings. No Major findings. The round-1 shared Major
(F1/M-B1) is fixed and verified (Mandate 1); all round-1 minors are
resolved or correctly recorded as no-action (Mandate 3); the semantic
state, ledger, and invariants are undisturbed (Mandates 2, 4, 6); all four
gates pass on their binding criteria (Mandate 5).

## Round-1 findings disposition (delta re-check)

| r1 finding | Disposition in round 2 |
|---|---|
| A:F1 / B:M-B1 (Major) — `/*environment_id*/` comment | **FIXED & VERIFIED** (Mandate 1 a-d; gates) |
| A:F2 / B:m-B4 — step-3 table row | **FIXED & VERIFIED** (Mandate 3; raw step-3 log) |
| A:F3 — step-7 log labels | **FIXED & VERIFIED** (Mandate 3; raw /tmp files) |
| A:F4 — T3.2 cwd cleanup best-effort | **NO ACTION, correctly recorded** (doc item 7; `find` re-run: no stray `grok/`) |
| B:m-B2 — step-2 thread IDs | **FIXED & VERIFIED** (Mandate 3; raw step-2 log) |
| B:m-B3 — `+321/−60` typo | **FIXED & VERIFIED** (Mandate 3; fresh numstat) |
| B:m-B5 — import refs off-by-one | **FIXED & VERIFIED** (Mandate 3; HEAD + current tree) |
| B:m-B6 — TurnContext/TurnEnvironment mislabel | **FIXED & VERIFIED** (Mandate 3; turn_context.rs:77/:200/:314) |
| B:m-B7 — parser.rs std assert_eq! | **NO ACTION, correctly recorded** (file-local pre-existing pattern; doc item 7) |
| B:n-B1 — F1 artifact name in T3.2 literal | unchanged (spec-mandated F1 shape; no action per r1) |

## Verdict

The revised artifact is clean. The single round-1 Major is fixed at source
with the correct parameter name, a correctly resolving unqualified name, a
91-column fmt-stable line, and byte-identity of the rest of the function;
the five-file numstat invariant holds exactly; every evidence-doc minor is
resolved against its stated source (raw logs re-audited where present);
spec §3.3 byte fidelity is undisturbed; the 20 owned unit tests pass in
this seat's own gate run; and all four gates pass on their binding criteria
(115/115; 0 deterministic core failures with 117/117 panics in the
lib.rs:388 class; clean fix/fmt with the known out-of-scope auto-fix
restored; fmt no-reflow). This seat's only new observations are
documentation-only (1 minor, 2 nits) in the new r1→r2 section of the
evidence doc — none affects code, the ledger, or any invariant.

SEAT C round 2: APPROVED — 0 Blocking, 0 Major, 1 minor, 2 nit
