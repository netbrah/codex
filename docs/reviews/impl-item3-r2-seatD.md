# Item 3 (P3 teachable errors) — Round 2 Review, SEAT D (delta lens: conventions / minimality / evidence)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` · Review date: 2026-09-16
Reviewed state: working tree at branch tip `76111f29eb` (verified: `git rev-parse HEAD` =
`76111f29eb5ac0281d93c16b25fc333806cb37a2`; Items 1+2 committed; the campaign
function-tool seam uncommitted per breakdown §0 execution baseline) + the uncommitted
Item-3 change set **as revised by the worker's round-1 fix wave** (code edit mtime
`Sep 16 07:06`, evidence-doc update mtime `Sep 16 08:03`).

Reviewer: SEAT D, independent; FRESH agent — every claim below re-verified against
source, raw logs, or a gate I ran myself. Round 1's full findings remain in the record
(`docs/reviews/impl-item3-r1-seatA.md`: F1 Major / F2 minor / F3 nit / F4 nit;
`docs/reviews/impl-item3-r1-seatB.md`: M-B1 Major / m-B2..m-B7 minor / n-B1 nit).
My scope is the delta (the fix wave) + the standing invariants; round 1's verified
mandates (byte fidelity, semantics, ledger) are not re-audited here.

## Scope

Round 1 returned REQUEST CHANGES from both seats on a shared Major — missing
`/*environment_id*/` argument comment at
`codex-rs/core/src/tools/handlers/apply_patch_tests.rs:57` (A-F1 = B-M-B1) — plus
evidence-doc minors (m-B2..m-B7, F2, F3) and two no-action nits (F4, n-B1). The worker's
fix wave (per its "Round 1 fixes (r1 → r2)" section,
`docs/reviews/impl-item3-tdd-evidence.md`:883-1105) claims to have touched exactly:

1. `apply_patch_tests.rs:57` — one line (add the comment; drop the fully-qualified
   path for fmt column limit).
2. The evidence doc — corrections for m-B2/m-B3/m-B4/m-B5/m-B6/F3 + no-action record
   for m-B7/F4 + re-run gate logs.

SEAT D verifies that delta, re-runs the four gates, and adjudicates anything new the
fix wave introduced.

## Method

- Re-read both round-1 reports in full (findings, gates, discrepancy adjudications).
- Read the evidence doc's new "Round 1 fixes (r1 → r2)" section (lines 883-1105) and
  re-verified **every** stated correction against its stated source: raw `/tmp` logs
  (grep/summary spot-checks), `git show HEAD:` output, and current-tree reads.
- Read the lint's own source (`tools/argument-comment-lint/src/lib.rs`) to pin the
  exact firing conditions (function + method calls; local or `codex_*`/
  `app_test_support`/`core_test_support` crates only; `None`/bool/numeric literals
  only — string/byte-str/c-str/char exempt; sole-arg exemption is method-calls-only
  with method name == parameter name; a `/*name*/` comment must match the parameter
  exactly). Bazel (and thus `just argument-comment-lint`) is not installed in this
  environment, so the sweep is a manual lint-semantics audit, same basis the round-1
  seats used (recorded as a method caveat).
- Repo-wide core/src sweep for uncommented opaque positional literals (single-line
  call-arg regex over 157 candidates + 59 multi-line `None,` candidates, each
  classified: std/external callee, macro, closure, tuple, struct field, or commented).
- Full read of the `invocation_from_session` function and its import chain
  (`apply_patch_tests.rs:1` `use super::*` → `apply_patch.rs:29` private import →
  `handlers/mod.rs:160-163` callee).
- Numstat invariants + md5 snapshots of all five owned files + `openai_file_mcp.rs`
  vs HEAD, taken before and re-checked after the gates.
- Gates re-run by me from `codex-rs/` in order: `just test -p codex-apply-patch`,
  `just test -p codex-core apply_patch`, `just fix -p codex-apply-patch -p codex-core`,
  `just fmt` (results in §Gates; patient runs, no PID kills, cargo lock honored).

## Mandate 1 — the Major fix at `apply_patch_tests.rs:57`

Current line (read from tree):

```
57:    let cwd = resolve_tool_environment(&step_context.environments, /*environment_id*/ None)
```

**(a) Parameter name match — PASS.** The callee
(`codex-rs/core/src/tools/handlers/mod.rs:160-163`, private `fn`) is:

```rust
fn resolve_tool_environment<'a>(
    environments: &'a TurnEnvironmentSnapshot,
    environment_id: Option<&str>,
) -> Result<Option<&'a TurnEnvironment>, FunctionCallError> {
```

The comment `/*environment_id*/` matches the signature parameter
`environment_id` exactly (case, spelling, no suffix). Per the lint source
(`tools/argument-comment-lint/src/lib.rs:216-228`), a present-but-mismatched comment
fires `ARGUMENT_COMMENT_MISMATCH`; an exact match passes.

**(b) Exemption rules — PASS (comment was required).** The lint's sole self-documenting
exemption is `args.len() == 1 && method_name == Some(expected_name)` — it applies only
to a METHOD's sole non-self argument whose parameter shares the method name. This call
is a plain function call with TWO positional arguments, so no exemption applies;
`None` (an `Option` parameter, `is_anonymous_literal_like` via the `OptionNone` lang
item) required the comment, and it is present. The string/char exemption is irrelevant
(`None` is not a str/char). AGENTS.md's rule and the lint source agree.

**(c) Repo-wide core/src sweep — PASS (round-1 claim re-verified).** Bazel is not
installed in this environment, so I could not run `just argument-comment-lint`; I ran a
manual audit using the lint's exact firing conditions (lint source read in full; UI
test `tools/argument-comment-lint/ui/uncommented_literal.stderr` codifies the pattern).

- Single-line call-arg candidates in `codex-rs/core/src`: 157 hits of
  `<callee>(…(None|true|false))`. Every `None` one classified:
  `futures::future::ready(None)` ×3 (external crate), `Mutex::new`/`RwLock::new`/
  `AtomicBool::new`/`LazyLock::new` (std), `ArcSwapOption::from(None)` (arc_swap,
  external), `Ok(None)` (std), and one closure call `checkpoint(None)`
  (`session/tests.rs:2944` — a `let`-bound closure; the lint requires
  `DefKind::Fn | AssocFn`, so closures are not linted). Bool candidates: std/external
  callees only (`kill_on_drop` tokio, `send_replace` std mpsc, `unwrap_or`/`then_some`
  std, `read/write/create` std `OpenOptions` — sole-arg name-match exempt anyway,
  `with_ansi`/`with_level` external tracing builders, `read_only` external
  `rmcp::model::ToolAnnotations` builder (test scope)…). Zero uncommented opaque
  positional literals remain on
  single-line local/workspace-crate calls.
- Multi-line `None,` candidates: 59 lines. All classified as struct fields (prev line
  ends `:`), macro args (`assert_eq!(`), arrays/`vec![`, tuple literals (e.g.
  `session/context_window.rs:65`, `agent/control/spawn.rs:557`,
  `session/turn_tests.rs:107`), or already commented — `mcp.rs:121` carries its
  comment on the preceding line (`/*originator*/`). Zero uncommented positional
  `None` call arguments.
- Non-last single-line `None` args: 31 hits — every one already carries the convention
  comment (`/*trace*/ None`, `/*review_output*/ None`, …) or is a macro call
  (`test_case(None, …)`).
- **Owned files (all five, full-file scan, added and pre-existing lines):** the only
  opaque positional literals are `Ok(None)`/`Ok(true)`/`Ok(false)` (std),
  `assert_eq!(…, None)` (macro), the `(None, None)` tuple at `parser.rs:218`,
  `Some(true)` (not literal-like), `None =>` match patterns, and the now-commented
  :57. Clean.

**Round-1 claim re-verified:** after the fix, `:57` is no longer an instance and NO
other uncommented positional `None` function-call argument exists in
`codex-rs/core/src`. No new Major (or any severity) from this sweep.

**(d) The unqualified-name change — PASS on all four sub-checks.**

- (i) **Resolves.** `apply_patch_tests.rs:1` is `use super::*;`; the test module is a
  child of the `apply_patch` module (declared `#[cfg(test)] #[path =
  "apply_patch_tests.rs"] mod tests;` at `apply_patch.rs:845-847`); the parent module
  carries the private import `use crate::tools::handlers::resolve_tool_environment;`
  at **`apply_patch.rs:29`** (the evidence doc cites `:32` — see New finding D-1). A
  child module sees its parent's private items, and the glob re-exports them; the same
  mechanism already resolves `StepContext`, `ToolInvocation`, `PathUri`,
  `make_session_and_context`, etc. in this file. Empirical proof: my Gate 2 run
  compiles the core test target (exit status and compile success recorded in §Gates).
- (ii) **No collision.** `resolve_tool_environment` appears exactly once in the test
  file (the :57 call); no local, no shadow.
- (iii) **Convention-consistent — a WIN, not drift.** The test file contains NO
  fully-qualified `crate::tools::handlers::X` call sites anywhere (its only
  `crate::` usages are `use` imports plus the `ToolCallSource::Direct` struct path at
  :71). Sibling helpers from the same `handlers/mod.rs` family are called
  unqualified in this file (`apply_patch_file_update_mode(&turn)` at :87 and :96),
  and the parent module itself calls `resolve_tool_environment` unqualified
  (`apply_patch.rs:423`). The round-1 fully-qualified form was the file's outlier;
  the worker's change aligns the call site with the file's established convention.
  (The doc's supporting example `file_system_sandbox_policy_context_for_cwd` is not a
  test-file call — see D-1.)
- (iv) **Column limit.** The :57 line is 91 cols (measured) ≤ 100. The qualified form
  with the comment is 115 cols (I counted; the doc says 114 — off by one, D-1), i.e.
  rustfmt would reflow it, so dropping the path is the only way to keep the comment
  on one line without changing the numstat. Rationale holds.

## Mandate 2 — numstat invariant + tree state

`git diff --numstat` (re-run by me) on the five owned files — EXACT match:

| file | expected | observed |
|---|---|---|
| `codex-rs/apply-patch/src/parser.rs` | +8/−6 | 8/6 ✓ |
| `codex-rs/apply-patch/src/streaming_parser.rs` | +5/−6 | 5/6 ✓ |
| `codex-rs/apply-patch/tests/suite/tool.rs` | +1/−1 | 1/1 ✓ |
| `codex-rs/core/src/tools/handlers/apply_patch.rs` | +265/−60 | 265/60 ✓ |
| `codex-rs/core/src/tools/handlers/apply_patch_tests.rs` | +164/−15 | 164/15 ✓ |

No deviation → no unannounced change in the owned set. (The :57 fix is an in-place
one-line replacement inside an already-added block, so an unchanged numstat is the
expected signature of a minimal fix wave — confirmed, and reinforced by the
line-reference integrity check in Mandate 4.)

`git status` (tracked, modified) = 17 files: the 5 owned + **11 codex-rs seam files**
(`codex-api/endpoint/{content_type_compat.rs, content_type_compat_tests.rs,
responses.rs}`, `core/src/tools/handlers/mod.rs` (+1/0), `core/src/tools/spec_plan.rs`,
`model-provider/src/{amazon_bedrock/mod.rs, provider.rs}`,
`models-manager/{models.json, src/manager_tests.rs, src/model_info.rs,
src/model_info_tests.rs}`) + `AGENTS.md` (+116/0, uncommitted SDD working-principles
block, mtime `Sep 13 15:37` — predates the entire Item-3 window and the fix wave;
present in round 1's pre-gate state). The total change set is exactly
**17 files, +1252/−108** — byte-identical in numstat to round 1's post-gate state
(seat A, Gate 3 note: "17 files, +1252/−108"), i.e. the fix wave added zero net
lines anywhere. No new files, no restorations missed.
`codex-rs/core/tests/suite/openai_file_mcp.rs`: empty `git diff` — **clean vs HEAD**
✓. (The evidence doc's "Round-1 final state" paragraph counts "5 owned + 11 seam,
nothing else" = 16; the 17th tracked file is the pre-existing AGENTS.md edit — see
New finding D-2.)
## Mandate 3 — evidence-doc minor resolutions (each re-derived from its stated source)

### m-B3 — +265/−60 — RESOLVED ✓

Pre-check now reads "`apply_patch.rs` (843 lines; carries the uncommitted seam,
**+265/−60** vs HEAD)" (line 81). Recounted: `git diff --numstat` → `265 60` ✓.
Arithmetic closes: HEAD file is 642 lines; 642 + 265 − 60 = 843 = pre-edit count, and
the P3.3 split's net +4 lands at the current 847 ✓. The wrong `+321` now appears
**only** inside the fix entry itself (line 925, describing the correction); the
Final-diff table (line 855) and Round-1 final-state table (line 1099) both carry
`265  60`. Worker's "single occurrence" claim verified.

### m-B2 — step-2 thread IDs — RESOLVED ✓

The two corrected blocks now cite (evidence doc step-2 capture set):

- streaming fn: `thread '…test_streaming_patch_parser_returns_errors' (85281497)
  panicked at apply-patch/src/streaming_parser.rs:836:9`
- CLI fn: `thread '…test_apply_patch_cli_rejects_invalid_hunk_header' (85282092)
  panicked at …/ops/function.rs:250:5`

Re-derived from `/tmp/item3-red-step2.log` (present, 25,970 bytes, mtime Sep 16 03:08):
TRY-1 section — streaming `(85281497)` at log line 173 ✓, CLI `(85282092)` at log line
265 ✓ (parser fns `85281015`/`85280948` unchanged, also match). `grep -c 85275
/tmp/item3-red-step2.log` → **0** — the wrong IDs appear nowhere in the raw log,
exactly as the fix entry states. The "verbatim, first try" label is now fully true
against the raw log for all four blocks.

### m-B4 / F2 — step-6 table step-3 row — RESOLVED ✓

The table row now reads: "`parser::test_parse_patch`, `parser::test_parse_patch_lenient`,
`streaming_parser::tests::test_streaming_patch_parser_returns_errors` (**3 fns / 10
assertions** — the CLI fn went green; the streaming fn is still red on the DeleteFile
assertion)". Re-derived from `/tmp/item3-red-step3.log` (present, mtime Sep 16 03:09):
`Summary [7.778s] 115 tests run: 112 passed, 3 failed, 0 skipped` ✓; the three failing
fns are exactly those three (each FAILED on both tries in the per-test sections); the
streaming fn now panics at `streaming_parser.rs:855:9` — the DeleteFile (ledger #3)
assertion ✓; the CLI fn is absent from the failure list (green) ✓. Matches the
step-3 narrative ("Ledger #1 and #4 green (12 → 10 red)") and the breakdown §1
ownership (P3.1 owns rows #1+#4). The stale "same 4 fns, minus streaming (11
assertions)" text is gone from the table.

### m-B5 — import line refs — RESOLVED ✓

- Pre-check now: "`use pretty_assertions::assert_eq;` :13; `use serde_json::json;` :14
  (line numbers at the 445-line pre-check state; the T3.2 import additions later shift
  them to :16/:17 in the round-1 file)". Verified both states: `git show
  HEAD:codex-rs/core/src/tools/handlers/apply_patch_tests.rs` → `pretty_assertions`
  at **:13**, `serde_json::json` at **:14** ✓; current tree → **:16**/**:17** ✓
  (the three added T3.2 imports `CodexAuth`/`PermissionProfile`/`AskForApproval`
  slot in above them).
- Step-5 fix: "already imported at **:13**" ✓ (old `:14` text only survives inside the
  fix entry itself, line 958).

### m-B6 — `TurnEnvironment::cwd()` — RESOLVED ✓

Both places fixed; the mislabel `TurnContext::cwd()` now appears only inside the fix
entry (line 959). Verified at source (`codex-rs/core/src/session/turn_context.rs`):
`impl TurnEnvironment {` starts at **:77** ✓; `pub(crate) fn cwd(&self) -> &PathUri {
&self.selection.cwd }` is at **:200-202** ✓; `TurnContext`'s deprecated `cwd:
AbsolutePathBuf` field is at **:313-314** (`#[deprecated]` attr :313, field :314) ✓ —
exactly the boundary the doc now states. Occurrences of the corrected name: 4
(pre-check read-only refs; step-5 scaffolding rationale; step-5 compile-fix note; fix
entry).

### F3 — step-7 log labels — RESOLVED ✓

Step 7 now cites, in true chronological order (file mtimes: 04:20 → 04:25 → 04:27 →
04:29 → 04:34, all present in `/tmp`):

| doc label | file | my spot-check of the file | result |
|---|---|---|---|
| First attempt (pre-fix hang, 20-test module run) | `item3-step7-module.log` | `TRY 1 TMT [60.011s]`, `TRY 2 TMT [60.009s]`, `Summary [120.040s] 20 tests run: 19 passed, 1 timed out, 4346 skipped` | VERBATIM ✓ |
| Single-test rerun | `item3-t32-debug.log` | TMT both tries, `Summary [120.035s] 1 test run: 0 passed, 1 timed out`, 0 DIAG lines | ✓ (DIAG-absence confirmed) |
| First DIAG iteration (compile failure) | `item3-t32-diag.log` | `error[E0425]: cannot find function assess_patch_safety in this scope`, `error[E0308]: mismatched types`, "could not compile `codex-core` (lib test) due to 2 previous errors" | ✓ (matches E0425/E0308 claim) |
| DIAG-instrumented run | `item3-t32-diag2.log` | TMT both tries (60.006/60.007), `Summary [120.031s] 1 test run: 0 passed, 1 timed out`, DIAG output ×2 (byte-identical lines) | ✓ |
| Post-fix result | `item3-t32-module.log` | `PASS [0.455s] (16/20) …f1_shaped…`, `Summary [0.489s] 20 tests run: 20 passed, 4346 skipped` | VERBATIM ✓ |

All quoted content is authentic; the label/sequence errors from round 1 (A-F3) are
gone.

### m-B7 / F4 — NO ACTION (recorded) — CONFIRMED ✓

- m-B7 (parser.rs keeps std `assert_eq!`): the no-action record in the fix section
  (item 7) correctly characterizes it as the pre-existing file-local pattern,
  consistent with round 1's string-only-rewrite mandate (Mandate 2b of the item) and
  the fact that nothing CI-enforces the import. No action was the right call; the 9
  rewritten assertions still use the module's std `assert_eq!` (verified in the file).
- F4 (stray-`grok/` risk): I ran `find . -name grok -not -path './.git/*'` from the
  repo root → **no output, exit 0** — no stray `grok/` tree anywhere. No action
  required; record stands.
## Mandate 4 — worker fix-wave minimality

Claim under test: the fix wave touched exactly `apply_patch_tests.rs:57` (one line)
+ the evidence doc. Evidence, all verified by me:

- **Numstat identical to round 1** for all five owned files (Mandate 2 table) — the
  fix is an in-place replacement of one already-added line, so any additional edit in
  `apply_patch_tests.rs` would have changed the +164/−15 balance or a line count.
- **File line counts unchanged:** `apply_patch_tests.rs` = 555 lines (round-1 count),
  `apply_patch.rs` = 847 lines (round-1 count).
- **Every round-1 line reference still lands on the same symbol** (spot-set covering
  the scaffold, both callers, the four seam tests, and all three new tests):
  `invocation_from_session` :51 (seat A: :51-75) ✓; `pre_tool_use_payload_uses_freeform_
  patch_input` :102 and `post_tool_use_payload_uses_patch_input_and_tool_output` :120
  (seat A) ✓; caller bindings `let (invocation, _) = invocation_for_payload(payload).
  await;` at :107 and :125 (seat B) ✓; seam test
  `with_environment_id_line_inserts_after_begin_patch_header` :431 (seat B: :431-468)
  ✓; T3.1a :470, T3.1b :490 (seat A) ✓; T3.2 :510 (seat B: ":510+") ✓. An in-place
  edit leaves all of these intact; any other line-level change in the file would have
  shifted at least one (or the numstat).
- **Content spot-reads match round-1 descriptions byte-for-byte in the reviewed
  substance:** T3.1a payload `arguments: r#"{}"#` + whole-`FunctionCallError`
  equality against the exact §3.3.3a string; T3.1b payload
  `json!({ "patch": { "raw": "not a string" } })` (genuinely non-string object) +
  exact §3.3.3b equality; T3.2 F1 patch literal (em dash `SPEC-FREEZE-1 ROUND 23 —
  REVIEW RUN 2 of 3 (apex-ayl.45)`, unprefixed content, raw table) + custom session
  (`AskForApproval::Never` + `PermissionProfile::Disabled`) + byte-exact file-content
  assertion + best-effort `let _ =` cleanup — all identical to what seats A/B
  verified.
- **No DIAG/eprintln leftovers:** `rg 'DIAG|eprintln'` over all five owned files →
  zero matches (exit 1).
- **No new helpers:** the function inventory of the test module is unchanged
  (`invocation_from_session` is the single round-1 extraction; nothing added).
- **No comment-style drift:** the only comment the fix wave added to product code is
  the convention-required `/*environment_id*/` at :57.
- **Evidence doc:** every correction is traceable to a round-1 finding (items 1-7 of
  the fix section map one-to-one to m-B3/m-B2/m-B4+F2/m-B5/m-B6/F3/m-B7+F4), each
  verified above; the new gate section cites `/tmp/r1-*.log` files, which exist and
  match their quoted summaries (r1-gate-ap 115/115 EXIT=0; r1-gate-core
  `Summary [2642.225s] 115 tests run: 73 passed (1 slow, 3 flaky), 6 failed, 36
  timed out, 4251 skipped` with all 73 `panicked` lines at `lib.rs:388` — binding
  grep empty, 73/73; r1-fix: clippy "Fixed core/tests/suite/openai_file_mcp.rs (1
  fix)" EXIT=0; r1-fmt EXIT=0; r1-check EXIT=0 with only the pre-existing
  openai_file_mcp warning; r1-fmtcheck clean). The 6 final-FAIL identities in the
  doc match the raw log's final results section exactly, and all 20
  `tools::handlers::apply_patch::tests::*` unit tests show PASS (20/20, 0 FAIL/TMT)
  in `r1-gate-core.log`.

**Verdict on minimality: PASS.** The code delta is exactly the one documented line.

## New findings from the fix wave (Mandate 6 adjudication)

**D-1 — minor — the code-fix justification paragraph in the evidence doc carries
three small factual slips (all in the "Round 1 fixes → Code fix" section, lines
~893-912; none affects the code, the gates, or the core justification).**
  1. Cites the parent import as `apply_patch.rs:32`; the private
     `use crate::tools::handlers::resolve_tool_environment;` is at **:29** (:32 is
     the `ToolOrchestrator` import). Off-by-three line ref — same class as round-1
     m-B5.
  2. Claims the unqualified form is "the same convention the file already uses for
     `file_system_sandbox_policy_context_for_cwd` and `apply_patch_file_update_mode`"
     — but `file_system_sandbox_policy_context_for_cwd` is **never called in the
     test file** (its unqualified call sites are `apply_patch.rs:760` and
     `unified_exec/exec_command.rs:326`). The convention claim itself is correct via
     `apply_patch_file_update_mode` (:87/:96) and the parent module's own :423
     call — the supporting example is just attributed to the wrong file.
  3. "the qualified form is 114 cols" — it is **115** cols (counted); the substance
     (>100, fmt would reflow) holds either way.
  Severity rationale: same class as round-1's m-B2/m-B5 (evidence-record accuracy),
  corrected-claim is true and verifiable, zero code impact.

**D-2 — nit — "Round-1 final state" paragraph undercounts the tracked change set.**
It states "`git status` (tracked) = the 5 owned files + the 11 pre-existing seam
files, nothing else" — the 11 codex-rs seam files are exactly right, but the 17th
tracked modification is `AGENTS.md` (+116/0, pre-existing uncommitted SDD
working-principles block, mtime Sep 13 — present in round 1's pre-gate state, not
introduced by the fix wave). The paragraph's own numstat table (the substantive
claim) is exact. Nit: tally/wording, not a state discrepancy.

**Adjudicated — no finding: the unqualified `resolve_tool_environment` name.**
Round-1's file state had exactly ONE fully-qualified `crate::tools::handlers::X` call
in the test file (the :57 line) and zero others; the file's established convention is
unqualified calls via `use super::*` (sibling family helper
`apply_patch_file_update_mode` at :87/:96; the parent module's own `:423` call).
The worker's change is a convention-consistency WIN forced by rustfmt's 100-col limit
(qualified + comment = 115 cols), not a drift. No action.

**Carried, no action (round-1 nits, re-confirmed):** F4/n-B1 (T3.2 writes the F1
artifact under the session temp cwd with best-effort cleanup; content is the
spec-mandated F1 shape carrying another bead's id `apex-ayl.45`) — no `grok/` tree
exists now (Mandate 3 re-run), test-only, no product impact.

## Mandate 5 — gates (re-run by me from `codex-rs/`, 2026-09-16)

Machine was under heavy load (gate-2 test phase ran 85.6 min vs the worker's
44 min comparable). Patient runs; no PID kills; cargo lock serialized the gates.

**Gate 1 — `just test -p codex-apply-patch`: GREEN.**
`Summary [41.893s] 115 tests run: 115 passed (14 slow), 0 skipped`, EXIT=0
(log `/tmp/seatD-gate1.log`). Includes all 12 Item-3 ledger rewrites, the golden
scenarios, the apply-patch-side twin of the :707 test
(`suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header`, exact-stderr
against the extended P3.1 message), and the surviving streaming boundary tests
(`streaming_parser.rs` :783/:801-region asserts per the breakdown DoD).

**Gate 2 — `just test -p codex-core apply_patch`: PASS per green criterion (0
deterministic failures).**
`Summary [5134.185s] 115 tests run: 51 passed, 2 failed, 62 timed out, 4251
skipped`, recipe EXIT=100 (nonzero on any failed/TMT — the load noise).
Binding check (re-run by me on `/tmp/seatD-gate2.log`):

```text
grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u   → (empty)
grep -cE "thread .+ panicked" LOG                                    → 112
grep -oE "panicked at [^:]+:[0-9]+:[0-9]+" LOG | sort -u             → exactly one site:
   core/tests/common/lib.rs:388:14   (112/112)
```

**Zero deterministic failures.** All 112 panics (both FAILs + all 62 TMTs) are the
known load-noise class `timeout waiting for event: Elapsed(())` at
`core/tests/common/lib.rs:388:14`; I read both FAILs' stderr blocks individually —
both are the harness event-wait, not an assertion.

- Final FAIL (2, both `suite::apply_patch_cli`, both lib.rs:388 class — the
  process-spawning, most load-sensitive tests, exactly the class that failed in
  every round-1 run under load): `apply_patch_shell_accepts_lenient_heredoc_wrapped_
  patch`, `apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::
  web_work_uses_cwd_when_enabled`.
- Final TMT (62): `apply_patch_cli` (40, incl. the surviving :707 test
  `apply_patch_cli_rejects_invalid_hunk_header` — both its tries died in the
  harness event-wait, not the assertion; its assertion path is exercised
  deterministically by the apply-patch crate's exact-stderr twin, 115/115 in my
  Gate 1), `apply_patch_serialization` (3), `approvals` (2), `code_mode` (1),
  `hooks` (6), `prompt_caching` (1), `request_permissions` (2),
  `request_permissions_tool` (2), `shell_snapshot` (1), `tool_harness` (2),
  `unified_exec` (2). Full identity list in the gate log
  (`/tmp/seatD-gate2.log` final results section); noise population overlaps the
  round-1 runs (same process-spawning suites), consistent with machine load, not
  regression — and no owned test appears anywhere in the failure/TMT sets.
- **The 20 item-3-owned unit tests (`tools::handlers::apply_patch::tests::*`) all
  PASS in my run — 20 PASS / 0 FAIL / 0 TMT** (recounted from the log), including
  the three new tests (`function_apply_patch_rejects_missing_patch_argument_with_
  teachable_error`, `function_apply_patch_rejects_non_string_patch_argument_with_
  teachable_error`, `function_apply_patch_applies_f1_shaped_raw_add_file_patch`).
  This run also compiles the core test target — empirical proof of the Mandate 1(d)
  name resolution for the unqualified `resolve_tool_environment` at :57.

**Gate 3 — `just fix -p codex-apply-patch -p codex-core`: EXIT=0 (2m33s; log
`/tmp/seatD-gate3.log`).** Clippy fixed exactly one file — `Fixed
core/tests/suite/openai_file_mcp.rs (1 fix)` — the documented, pre-existing,
out-of-scope unused `wiremock::matchers::body_json` import (line 47) that
`clippy --fix` re-triggers on every run (identical in both round-1 runs and the
worker's r1 run). Restored via `git checkout -- codex-rs/core/tests/suite/
openai_file_mcp.rs`; verified clean vs HEAD (empty diff) and the tree back to
exactly 17 tracked files, +1252/−108 — no other file touched by the gate. (Gate 3's
lint set is clippy; the Bazel-backed `argument_comment_lint` is not part of this
gate and Bazel is not installed in this environment — see Mandate 1(c) for the
manual sweep that substitutes for it.)

**Gate 4 — `just fmt`: EXIT=0 (silent; log `/tmp/seatD-gate4.log`).** Post-fmt
verification: `apply_patch_tests.rs:57` byte-identical — 91 cols,
`/*environment_id*/` comment retained (fmt did not strip or reflow it); md5 of all
five owned files + `openai_file_mcp.rs` identical before/after all four gates. Per
repo rule, no tests were re-run after fix/fmt.

## Standing invariants (re-confirmed in the delta lens)

- **No new public API / no `pub` in the fix wave:** the fix is inside a
  `#[cfg(test)]` private fn; nothing exported changed.
- **No `Cargo.toml`/lock changes:** none in `git status`.
- **OpenAI request bytes unchanged:** the fix wave touches no wire/spec code
  (test-only line + doc).
- **Bounded messages / no non-error-message behavior change:** unchanged from
  round 1 (numstat-identical owned files; the single edited line is a test call
  site whose runtime behavior is byte-identical — the qualified and unqualified
  names resolve to the same fn, same args).
- **Tree state:** HEAD `76111f29eb`; 17 tracked modified files (5 owned + 11
  codex-rs seam + pre-existing AGENTS.md), +1252/−108, no new files,
  `openai_file_mcp.rs` clean vs HEAD — identical to round 1's post-gate state.

## Findings (round 2)

| # | Severity | Where | Summary |
|---|----------|-------|---------|
| D-1 | minor | `docs/reviews/impl-item3-tdd-evidence.md`, "Round 1 fixes → Code fix" paragraph (~:893-912) | Three small factual slips in the fix justification: (1) import cited at `apply_patch.rs:32` — it is at `:29`; (2) `file_system_sandbox_policy_context_for_cwd` cited as a same-file convention example — it is never called in the test file (the real same-file examples are `apply_patch_file_update_mode` :87/:96 and the parent's own `:423` call); (3) "qualified form is 114 cols" — it is 115. All three are supporting details; the core claims (name resolves via `use super::*` + private parent import; convention-consistent; fmt forces the unqualified form) are each independently verified TRUE |
| D-2 | nit | `docs/reviews/impl-item3-tdd-evidence.md`, "Round-1 final state" paragraph (~:1093-1105) | Tally says "5 owned + 11 pre-existing seam files, nothing else" (16); the 17th tracked modification is the pre-existing `AGENTS.md` (+116/0, mtime Sep 13, present in round 1's pre-gate state — not introduced by the fix wave). The paragraph's numstat table is exact; only the tally/wording is off |

Round-1 finding disposition (all re-verified above): F1/M-B1 (Major) — FIXED,
Mandate 1 PASS. m-B2 — fixed, Mandate 3 PASS. m-B3 — fixed, Mandate 3 PASS.
m-B4/F2 — fixed, Mandate 3 PASS. m-B5 — fixed, Mandate 3 PASS. m-B6 — fixed,
Mandate 3 PASS. F3 — fixed, Mandate 3 PASS. m-B7/F4 (no-action) — record
confirmed, `grok` find empty. n-B1 (no-action) — carried, re-confirmed
test-only.

## Verdict

The shared round-1 Major is correctly fixed: the `/*environment_id*/` comment
matches the callee parameter exactly, the exemption rules do not apply, the
repo-wide core/src sweep re-confirms the round-1 claim (no other uncommented
opaque positional literal in item-touched code; none at all in local/workspace
function-call positions across `codex-rs/core/src`), and the accompanying
unqualified-name change is a convention-consistency win, not drift. The fix wave
is byte-minimal (one line in one owned file; all round-1 line refs and numstat
intact; no DIAG leftovers; no new helpers). All eight round-1 evidence-doc
minors are resolved against their stated sources, verified verbatim against the
raw `/tmp` logs and `git show HEAD:`. All four gates pass on my own runs (Gate 2
with 0 deterministic failures, all 112 panics in the known load-noise class, 20/20
owned unit tests green). The two new findings are minor/nit documentation
accuracy items in the fix section of the evidence doc — no code impact, no gate
impact, no convention or minimality violation.

SEAT D round 2: APPROVED — 0 Blocking, 0 Major, 1 minor, 1 nit
