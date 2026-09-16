# Item 3 (P3 teachable errors) — Round 1 Review, SEAT A (spec/semantics lens)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` · Review date: 2026-09-16
Reviewed state: working tree at branch tip `76111f29eb` (Items 1+2 committed; the
campaign function-tool seam uncommitted per breakdown §0 execution baseline) +
untracked Item-3 change set. Worker: `/root/item3_p3_worker` (evidence:
`docs/reviews/impl-item3-tdd-evidence.md`).

## Scope

Item 3 = "P3: teachable errors (spec §3.3)": every parse-path rejection returned
to the model becomes teachable — four message sites:

- **P3.1** streaming `StartedPatch` non-header message — sentence EXTENSION
  (old sentence kept as an exact prefix).
- **P3.2** streaming `DeleteFile` content-line message — wholesale REPLACEMENT.
- **P3.3** function-handler argument error — collapsed absent/non-string SPLIT
  into two messages; argument name re-quoted backticks → single quotes.
- **P3.4** parser boundary pre-pass `check_start_and_end_lines_strict` — both
  strings EXTENDED (old sentence remains exact prefix).

Plus T3.1 (two exact-string handler tests), T3.2 (F1-shaped raw Add-File through
the function handler), the 12 ledger-assertion rewrites (rows 1, 3–13), and the
`invocation_from_session` scaffold extraction.

**Not in scope (verified untouched):** the streaming diff-consumer parallel
messages, the `invalid patch: ` Display prefix, the Update-File message, Item 1's
P2 AddFile arm, Item 2's spec text, golden fixtures, the committed Items 1/2.

## Method

1. Re-read SoT spec v5 §3.3 (all six message strings, prefix statuses, scope
   boundary, red-state list), §3.2, §1.1 (F1), §3.4 invariants, §4 T3 plan;
   breakdown v3 Item 3 + §0 + §1 (13-row red ledger) + §5 invariants.
2. Mechanical byte-fidelity: python-sliced the six strings out of spec §3.3
   backtick spans (soft-wrap joins re-derived independently) and compared
   byte-for-byte against the six code sites; re-derived all char counts.
3. Full diff sweep of the five owned files: classified every added/removed
   string literal (scripted) to bound the Item-3 change set; verified the seam
   portions are untouched (mtime evidence + pre-check line-count arithmetic +
   T3.1 red log as the pre-split record).
4. Source-level re-derivation of every semantic claim: handler split semantics,
   `assess_patch_safety` hang root cause, `make_session_and_context*` shapes,
   `test_codex.rs` harness shape, T3.2 patch literal (Rust-escape unescape +
   P2 verbatim-append reconstruction), scaffold-extraction semantics for the two
   pre-existing callers, surviving `:707` substring test.
5. Independent log audit: every quoted run in the evidence doc re-checked
   against the raw `/tmp/item3-*.log` files (summaries, failing-fn identities,
   panic classes, T3.1/T3.2 states, substring-test states).
6. Re-ran the item gates myself from `codex-rs/` (gate results in §Gates).

## Mandate 1+2 — Byte fidelity and prefix status (mechanical, python)

**Spec slice (re-derived by me, independent of the worker's slice).** The six
message strings were extracted from `docs/responses-compat-apply-patch-format.md`
§3.3 backtick spans. Note: the spec soft-wraps spans 1–3 and 3a/3b at word
boundaries where the newline *replaces* the wrapping space (joining without a
space under-counts by exactly the wrap count; I verified every wrap point is at
a word boundary and no joined string contains a double space). All six joined
strings are all-ASCII (chars == bytes) with lengths:

| # | spec site | len | code site (verified) | result |
|---|---|---|---|---|
| 1 | §3.3.1 suffix (leading space) | 193 | `codex-rs/apply-patch/src/streaming_parser.rs:201` (format! literal, suffix after the old sentence) | BYTE-IDENTICAL |
| 2 | §3.3.2 replacement | 103 | `codex-rs/apply-patch/src/streaming_parser.rs:228` | BYTE-IDENTICAL |
| 3a | §3.3.3a absent | 121 | `codex-rs/core/src/tools/handlers/apply_patch.rs:536` | BYTE-IDENTICAL |
| 3b | §3.3.3b non-string | 108 | `codex-rs/core/src/tools/handlers/apply_patch.rs:543` | BYTE-IDENTICAL |
| 4a | §3.3.4a Begin | 182 | `codex-rs/apply-patch/src/parser.rs:269` | BYTE-IDENTICAL |
| 4b | §3.3.4b End | 149 | `codex-rs/apply-patch/src/parser.rs:272` | BYTE-IDENTICAL |

All six char counts independently re-derived: **193/103/121/108/182/149 —
identical to the worker's claims.** Any single-byte divergence would have been
reported as BLOCKING; none found.

**Prefix status per spec (verified against the actual diff hunks):**
- **Site 1 (P3.1) — extension.** The rendered message is the old sentence
  `'{trimmed}' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'`
  (136 bytes, kept byte-identical) + the 193-byte suffix. `startswith` verified:
  the old sentence is an **EXACT PREFIX** of the new rendered message. The
  diff hunk (streaming_parser.rs @@ -198,7) changes only the format! literal;
  the `{{path}}` escapes and `{trimmed}` interpolation are untouched. The
  surviving substring `is not a valid hunk header` is contained (verified).
- **Site 2 (P3.2) — replacement.** The diff hunk (streaming_parser.rs
  @@ -225,9 +225,8) replaces the `format!("'{trimmed}' is not a valid hunk
  header…")` with the 103-byte constant `.to_string()` — shares no prefix with
  the old text (new text starts `'Delete File'`), matching the spec's
  "wholesale replacement" status. The message quotes no user input, so dropping
  `format!` is correct; `trimmed` remains consumed by
  `handle_hunk_headers_and_end_patch(trimmed)` (no unused-variable change).
- **Site 3 (P3.3) — re-quote + split.** Old message (backticked `patch`, single
  collapsed error) is recorded in the T3.1 red log verbatim
  (`apply_patch is missing the required \`patch\` argument`). New: two
  messages, argument name in SINGLE quotes in both (byte-verified against
  §3.3.3a/3b, which use single quotes).
- **Site 4 (P3.4) — extension.** `startswith` verified for both:
  `The first line of the patch must be '*** Begin Patch'` is an exact prefix of
  the 182-byte Begin message; `The last line of the patch must be '*** End
  Patch'` is an exact prefix of the 149-byte End message. Diff hunk
  (parser.rs @@ -266,10) changes only the two `String::from(...)` literals.

## Mandate 3 — Scope boundary (verified)

- **Streaming diff-consumer parallel messages UNCHANGED** (spec §3.3 explicit
  scope boundary): `finish()` End message at `streaming_parser.rs:176`,
  `NotStarted` Begin message at `:192`, `EndedPatch`-content End message at
  `:380` — none of the four streaming diff hunks (production @@ -198 / @@ -225;
  tests @@ -836 / @@ -855) touches these lines. Item 1's P2 test file
  (`streaming_parser_p2_tests.rs:297`) still asserts the old short End string
  against `finish()` and is untouched (empty diff) — consistent.
- **`invalid patch: ` Display prefix UNCHANGED**: `parser.rs:57`
  (`ParseError::InvalidPatchError` Display) is outside all four parser hunks.
- **Update-File "Every line should start with…" message UNCHANGED**: all four
  occurrences (`streaming_parser.rs:83/:276/:370` + test asserts `:905/:927`)
  untouched.
- **Item 1's P2 AddFile arm UNCHANGED by Item 3**: the AddFile arm
  (`streaming_parser.rs:207-222`, verbatim-append leniency) is outside all
  Item-3 hunks; file mtime `03:39` (step 4, P3.2) vs the arm's Item-1 commit
  `e21f608ac4`.
- **Diff sweep of both crates' error strings (scripted):** every added/removed
  string literal in the five owned files classified. In `apply-patch/` the only
  changes are the six message sites + their 12 ledger test expectations. In
  `core/.../apply_patch.rs`, besides the P3.3 pair, every other added string
  has an identical removed counterpart (pure moves into the seam's
  `run_apply_patch_text` extraction: the `apply_patch verification failed:
  {parse_error}` wraps ×2, `apply_patch is unavailable in this session`, the
  `tracing::trace!` shell-parse string, `…invalid patch input`, `…non-apply_
  patch input`) or is seam/test code (new handler's own messages, T3.1/T3.2
  expect/panic strings). **No message other than the four P3 sites changed
  content.**
- **Seam untouched by Item 3**: `handlers/mod.rs` and `spec_plan.rs` mtimes
  `Sep 13 15:37` (predate the Item-3 work window `Sep 16 03:07–05:16`);
  `apply_patch_spec.rs` (Item 2) mtime `Sep 16 01:10` (before Item 3 start).
  Line-count arithmetic closes: `apply_patch.rs` was 843 lines at Item-3 start
  (evidence pre-check) and the P3.3 split is a net +4 lines → 847 = current.
  `apply_patch_tests.rs` 445 → 555 = +110 (scaffold extraction + T3.1 ×2 +
  T3.2 + imports), matching the diff.

## Mandate 4 — Handler split semantics (P3.3)

`codex-rs/core/src/tools/handlers/apply_patch.rs:534-545` (`handle_call`):

```
let patch_value = value.get("patch").ok_or_else(|| { /* §3.3.3a */ })?;
let patch_input = patch_value
    .as_str()
    .ok_or_else(|| { /* §3.3.3b */ })?
    .to_string();
```

- **Absent vs non-string distinguished exactly per spec**: `value.get("patch")`
  is `Option<&Value>` — key absent from the JSON object → `None` → §3.3.3a.
  Present-but-non-string (object, number, null, etc.) → `Some(v)` then
  `v.as_str() == None` → §3.3.3b. (A non-object JSON value such as `[1]` or
  `"x"` has no `patch` field → §3.3.3a, same as the old collapsed behavior —
  no behavior change beyond the messages.)
- **Happy path behavior-identical**: `patch_value.as_str() → &str →
  .to_string()` yields the identical `patch_input` as the old
  `.and_then(Value::as_str)… .to_string()`.
- **No control-flow change beyond the two error branches**: the surrounding
  `handle_call` (payload extraction, JSON-parse error, `environment_id`
  handling via `with_environment_id_line`, delegation to `run_apply_patch_text`)
  is untouched by Item 3 (all seam).
- **Single quotes in both new strings** — byte-verified (site 3 status:
  backticks → single quotes).

## Mandate 5 — T3.1 tests (verified)

Both tests in `codex-rs/core/src/tools/handlers/apply_patch_tests.rs`:

- `function_apply_patch_rejects_missing_patch_argument_with_teachable_error`
  (:470): payload `ToolPayload::Function { arguments: r#"{}"# }` →
  `assert_eq!(err, FunctionCallError::RespondToModel("apply_patch is missing
  the required 'patch' argument; pass the full patch text starting with '***
  Begin Patch' in 'patch'"))` — **whole-string equality** (the entire
  `FunctionCallError` is compared; `pretty_assertions::assert_eq` imported at
  :14), exact §3.3.3a string (byte-verified).
- `function_apply_patch_rejects_non_string_patch_argument_with_teachable_error`
  (:490): payload `json!({ "patch": { "raw": "not a string" } })` — a
  **genuinely non-string JSON object** → whole-string equality against the
  exact §3.3.3b string.
- The red log (raw `/tmp/item3-step5-module2.log`, independently inspected)
  shows both tests failing pre-split with the old backticked message for BOTH
  shapes (19 tests: 17 passed, 2 failed), then 19/19 green post-split
  (`/tmp/item3-step5-green.log`).

## Mandate 6 — T3.2 semantics (the material discrepancy; verified at source)

**(a) Hang root cause — re-derived at the source.** `make_session_and_context`
(`codex-rs/core/src/session/tests.rs:5882`) builds from `build_test_config`
(`:4421`, `ConfigBuilder::without_managed_config_for_tests()` with no
permission overrides). The worker's captured DIAG (raw `/tmp/item3-t32-diag2.log`,
both tries identical): `approval_policy = OnRequest`, `permission_profile =
Managed { file_system: Restricted { Root: Read }, network: Restricted }`.
Reading `assess_patch_safety` (`codex-rs/core/src/safety.rs:29`): for a WRITE
patch (the F1 Add-File creates `grok/plans/spec-freeze-r23-glm.md` under the
session cwd), `is_write_patch_constrained_to_writable_paths(…)` is false (the
sandbox policy is Root-Read-only), so the `AutoApprove` branch is unreachable;
`rejects_sandbox_approval` is false for `OnRequest` (only `Never`/
`Granular(!sandbox_approval)` reject); control falls through to
`SafetyCheck::AskUser`. The handler then awaits an approval answer that a unit
test never provides → deterministic hang. This matches both captured hang logs
(60s TMT on both tries, no `lib.rs:388` signature — i.e. not the load-noise
class). The breakdown's assumption ("existing `invocation_for_payload`
scaffolding as-is") is therefore **false at the source** — the assumption is
confirmed refuted, and the discrepancy is material and correctly recorded.

**(b) Custom session = established harness shape, not a weakening.**
`codex-rs/core/tests/common/test_codex.rs:1006-1007` is exactly
`AskForApproval::Never, PermissionProfile::Disabled` (inside
`submit_turn_with_service_tier` → `submit_turn_with_permission_profile_context`);
`AskForApproval::Never` is the harness norm across the submit helpers
(:984, :995, :1057). T3.2's session is built via
`make_session_and_context_with_auth_and_config_and_rx` (`session/tests.rs:8019`,
`pub(crate)`, signature matches the call) with
`Permissions::from_approval_and_profile(Constrained::allow_any(AskForApproval::
Never), Constrained::allow_only(PermissionProfile::Disabled))` (the constructor
at `config/mod.rs:352` behaves as the name states). With `Never` + `Disabled`,
`assess_patch_safety` takes the `AutoApprove` branch (write is constrained to
writable paths under the unrestricted profile / `Disabled` match), so the write
completes — confirmed by the green runs. T3.1's two tests KEEP the default
read-only session: their payloads error during argument extraction, before any
environment/safety code runs, so nothing is weakened there.

**(c) F1 patch shape + byte-exact assertion — verified.** I unescaped the Rust
line-continuation literal mechanically; the rendered patch is exactly:

```
*** Begin Patch
*** Add File: grok/plans/spec-freeze-r23-glm.md
# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)
Freeze holds until the round-23 review lands.
<blank line>
| field | value |
| --- | --- |
*** End Patch
```

This is the spec §1.1 F1 shape: raw markdown Add-File, **H1 first content line
with the rollout's em dash byte-for-byte**, unprefixed content (no line starts
with `+`), blank line + raw table rows. Under P2's verbatim-append semantics
(content lines 2–6 of the patch, each + `\n`), the reconstructed file content
equals the test's asserted literal
`"# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)\nFreeze holds
until the round-23 review lands.\n\n| field | value |\n| --- | --- |\n"`
**byte-for-byte** (python-verified). The parse path is real:
`handle_call → run_apply_patch_text → parse_patch → check_start_and_end_lines_
strict (P3.4 site) → StreamingPatchParser (P2 leniency) → verify → apply`.
Note: the test asserts via `std::fs` against the session cwd resolved through
the SAME helper the handler uses (`resolve_tool_environment(…, None).cwd()`),
so the assertion target is exactly the file the handler writes. The test
self-cleans (file + both parent dirs); I verified no `grok/` tree remains
anywhere in the worktree. (See finding F4 for the cleanup-robustness nit.)

**(d) Scaffold extraction is semantics-preserving.** `invocation_from_session`
(`apply_patch_tests.rs:51-75`, private `async fn` inside the `#[cfg(test)]`
sibling module declared at `apply_patch.rs:845-847`) constructs the
`ToolInvocation` field-for-field identically to the old inline code (same
`StepContext::for_test`, same `CancellationToken::new()`, same tracker, same
`call_id`/`tool_name`/`source`/`payload`) and adds a returned
`PathUri` (the resolved primary-environment cwd). `invocation_for_payload`
now delegates to it after `make_session_and_context()`. The two pre-existing
callers (`pre_tool_use_payload_uses_freeform_patch_input` :102,
`post_tool_use_payload_uses_patch_input_and_tool_output` :120) changed ONLY
their binding to `let (invocation, _) = …`; their assertions are byte-identical
to HEAD (diff-verified) and both pass in the 20/20 module run and the gate
run. No behavioral change for them.

## Mandate 8 — :707 substring survival (verified)

`codex-rs/core/tests/suite/apply_patch_cli.rs:689`
(`apply_patch_cli_rejects_invalid_hunk_header`) is **UNTOUCHED** (empty diff).
The surviving assertion is at `:707` (`out.contains("is not a valid hunk
header")`); the first assertion (`:703`, `"apply_patch verification failed"`)
is satisfied by the unchanged `run_apply_patch_text` error wrap. The freeform
path reaches the same StartedPatch arm (shell-intercept → `parse_patch` →
streaming parser), so the extended P3.1 message keeps the substring reachable —
byte-verified containment in the rendered message. Test-state evidence:
PASSED deterministically in my Gate 1 run (apply-patch-side twin, `:393`
exact-stderr, 115/115); in the worker's core runs it was TMT-noise on some
tries (always `lib.rs:388`) and PASS 0.452s/0.341s in step-3/gate runs
(raw-log verified); my core gate re-confirms (see §Gates).

## Mandate 7 — Progression authenticity (verified against raw logs)

Ledger ownership (breakdown §1): P3.1 → rows #1 (streaming StartedPatch) + #4
(CLI exact-stderr, same message); P3.2 → row #3 (streaming DeleteFile);
P3.4 → rows #5–13 (parser boundary: 2 in `test_parse_patch`, 6 Begin + 1 End
in `test_parse_patch_lenient`). 2 + 1 + 9 = 12 item-3 rows (row #2 is Item
1's, already green). Arithmetic: 12 → (−2, P3.1) → 10 → (−1, P3.2) → 9 →
(−0, P3.3 owns no rows) → 9 → (−9, P3.4) → 0. **Verified.**

Raw-log audit (all files present in `/tmp`; every quoted figure re-checked):

| step | log | observed | matches design? |
|---|---|---|---|
| 2 (12 asserts rewritten) | `item3-red-step2.log` | 115 run: 111 passed, **4 failed** — `parser::test_parse_patch`, `parser::test_parse_patch_lenient`, `streaming_parser::tests::test_streaming_patch_parser_returns_errors`, `suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header` | YES (4 fns / 12 assertions) |
| 3 (P3.1) | `item3-red-step3.log` | 115 run: 112 passed, **3 failed** (CLI fn gone; streaming fn still red at the DeleteFile assert) | YES (10 red asserts) |
| 4 (P3.2) | `item3-red-step4.log` | 115 run: 113 passed, **2 failed** (the two parser fns) | YES (9 red asserts) |
| 5 (T3.1 RED → P3.3) | `item3-step5-red.log` (compile), `item3-step5-module2.log`, `item3-step5-green.log` | first compile fails with exactly E0599 `turn.cwd()` + E0277×2 `expect_err` (as recorded); module run 19 tests: **17 passed, 2 failed** = the two T3.1 tests, panics show the OLD backticked message for both shapes (the pre-split record); post-split 19/19 | YES (red state holds at 9 — P3.3 owns no ledger rows; the apply-patch crate was untouched between step 4 and step 6: parser.rs mtime 04:15, streaming 03:39, tool.rs 03:07) |
| 6 (P3.4) | `item3-step6-green.log` | 115 run: **115 passed** | YES (0 red) |

Red-log completeness: the evidence doc's 12-row table enumerates **every**
rewritten assertion with its red-state observation (4 observed panics recorded
verbatim; 8 "blocked behind" entries correctly explained by Rust
first-panic-aborts-per-fn semantics — a blocked assertion cannot produce its
own panic output, and each blocked row is mapped to the owning fn's first
panic). The four verbatim panics match the raw logs (message diffs, line:col
panics :836:9/:279:5/:582:5, stderr diff at function.rs:250). No red state
outside the ledger in any run (111/112/113 passed = all non-ledger green).

**Discrepancy (my finding F2):** the step-6 summary table's step-3 row reads
"same 4 fns, minus streaming (11 assertions)" — contradicted by the same
document's step-3 narrative ("Ledger #1 and #4 green (12 → 10 red)") and by
`item3-red-step3.log` (3 failing fns: the CLI fn is green too). Correct state:
3 fns / 10 red assertions. The headline progression 12 → 10 → 9 → 9 → 0 is
correct everywhere it matters.

## Invariants (breakdown §5 + task invariants)

- **OpenAI-named-provider request bytes unchanged**: Item 3 touches no
  wire/spec code; `apply_patch_spec.rs` unmodified since 01:10 (pre-Item-3).
  The P3.3 messages can only surface on the capability-gated non-OpenAI
  function path; P3.1/3.2/3.4 are provider-agnostic parser error-text changes
  that fire only for inputs upstream already rejects (StartedPatch non-header,
  DeleteFile content, missing Begin/End). No success-path change.
- **No new public API**: `invocation_from_session` and all T3 tests live in
  the `#[cfg(test)]` sibling module (`apply_patch.rs:845-847`); helper is a
  private `async fn`. No `pub` additions in the Item-3 change set.
- **Bounded messages**: largest rendered message = 136+193 = 329 bytes (P3.1)
  ≪ 10K tokens; all six ≤ 329 bytes.
- **No change to non-error-message behavior**: full diff sweep (Mandate 3) —
  every non-message change is test scaffolding or new tests; production diff
  is exactly the six literals (four sites) plus the two-error split structure.
- **Golden scenarios unmodified**: `git status` shows no fixture changes;
  `suite::scenarios::test_apply_patch_scenarios` PASS in my Gate 1 run.
- **No `Cargo.toml`/lock changes**: none in `git status`.
- **13-row ledger fully green at item end**: Gate 1 (mine) 115/115 includes
  all 12 Item-3 rewrites + Item 1's row #2.

## Discrepancy adjudications (worker's five, in my lens)

**D1 — T3.2 session shape (material). ADJUDICATED: real; worker's handling
correct; no action required.** The breakdown's scaffolding assumption is
refuted at the source (Mandate 6a: read-only `Managed{Root:Read}` + `OnRequest`
→ `assess_patch_safety = AskUser` → unanswered-approval deadlock, deterministic
in two independent raw logs). The custom session (`AskForApproval::Never` +
`PermissionProfile::Disabled`) is the integration harness's established shape
(`test_codex.rs:1006-1007`, verified verbatim), so it is not a weakening; the
test stays test-only, in the owned file, minimal (one extraction + one session
build). T3.1 needed no such change and kept the default session (payloads
error before env/safety code) — correctly reasoned.

**D2 — `invocation_for_payload` signature change (mechanical). ADJUDICATED:
legitimate; semantics preserved.** Return type extended to
`(ToolInvocation, PathUri)`; the two pre-existing callers changed only their
binding (`let (invocation, _) = …`), assertions byte-identical to HEAD, both
green in every run. The extracted helper is test-private.

**D3 — `openai_file_mcp.rs` pre-existing unused import + `just fix`
re-trigger. ADJUDICATED: real, handled correctly.** The
`wiremock::matchers::body_json` import is unused at HEAD (`:47`; the only other
`body_json` hits are the unrelated `set_body_json` method name), so the warning
pre-dates the campaign. `just fix` auto-removes it on every run (worker's
`/tmp/item3-fix.log`: "Fixed core/tests/suite/openai_file_mcp.rs (1 fix)");
worker restored via `git checkout` — verified: file == HEAD now (empty diff),
and my own Gate 3 run re-triggers and I restore identically (see §Gates).
Known recurring hazard for every remaining item's `just fix`.

**D4 — Load noise worse than the expected window. ADJUDICATED: real; the
binding green criterion (0 DETERMINISTIC failures) is correct.** All three
worker core logs (`item3-step3-core.log` 1658s, `item3-step5-full.log` 1602s,
`item3-gate-core.log` 2117s) were audited: every panic is
`core/tests/common/lib.rs:388:14: timeout waiting for event: Elapsed(())`
— `grep -E "thread .+ panicked" | grep -v 'lib.rs:388' | sort -u` is empty in
all three; noisy-test populations overlap across runs (same ~30-test set),
consistent with machine load rather than regression. My independent core gate
run (see §Gates) is the final confirmation.

**D5 — Spec §3.3 line-ref drift. ADJUDICATED: real, expected, handled by the
protocol.** Spec cites `parser.rs:256-274` / `:268`/`:271`; the current tree
is `:257-275` / `:269`/`:272` (Item 1's module-doc line shifted parser.rs by
+1 — the evidence pre-check records this). The spec's
`apply_patch.rs:508-560` handler range predates the seam's final size
(`handle_call` now `:508-573`). The spec is LOCKED v5 (SoT), so no spec edit
belongs in Item 3; the breakdown §0 convention ("each item's first step is to
re-verify its own references") covers this, and the pre-check re-verified
every ref at the working tree. No action required.

## My additional findings (see §Findings for severity)

- **F1 (Major, conventions CI enforces):** `apply_patch_tests.rs:57` —
  `resolve_tool_environment(&step_context.environments, None)` passes a bare
  `None` positionally with no `/*environment_id*/` comment. This is the ONLY
  uncommented positional `None` in all of `codex-rs/core/src` (repo-wide grep);
  every other direct-call bare literal in committed code is commented (e.g.
  `compact_remote_v2.rs:834-877` test code). The strict
  `argument_comment_lint` (dylint, `uncommented-anonymous-literal-argument`,
  deny via the lint binary's `STRICT_LINTS`) explicitly lints inline
  `#[cfg(test)]` call sites (`tools/argument-comment-lint/list-bazel-targets.sh`
  comment: targets are added "so inline `#[cfg(test)]` call sites are linted")
  and its UI test (`ui/uncommented_literal.stderr`) codifies exactly this
  pattern (direct call, `None` for an `Option` param → warning, suggested fix
  `/*base_url*/ None`). The callee is a same-crate fn with a meaningful
  parameter name and two arguments (no self-documenting exemption), so the
  lint fires. Item 2's own review (r1-seatB row "/*param_name*/
  argument-comment convention") enforced the same rule on test code. Fix is
  one line: `/*environment_id*/ None`. I did not run the Bazel-backed lint
  itself (cold Bazel on a loaded machine); the classification rests on the
  lint's mechanical source, its UI test, the target-listing script, and the
  repo-wide clean-state precedent.
- **F2 (minor):** evidence-doc step-6 summary table, step-3 row arithmetic
  (11 assertions / "same 4 fns minus streaming") contradicts the doc's own
  step-3 narrative and the raw log (3 fns / 10 red). Bookkeeping error only;
  the authoritative progression is correct.
- **F3 (nit):** evidence-doc step 7 mislabels two log files: the quoted 20/20
  "Result" block actually lives in `/tmp/item3-t32-module.log`, and the
  pre-fix module-scope hang is `/tmp/item3-step7-module.log` (the doc cites
  `item3-t32-diag2.log`, which is the DIAG-instrumented single-test hang). All
  quoted content is authentic and present in `/tmp`; only the file labels are
  swapped/wrong.
- **F4 (nit):** T3.2 writes into the session cwd, which for
  `make_session_and_context_with_auth_and_config_and_rx` (via
  `build_test_config`, no cwd override) is the test process cwd; cleanup is
  best-effort (`let _ =` removes), so a failing run could leave a stray
  `grok/` tree under the process cwd. No artifact exists now (verified);
  test-hygiene only, no product impact. The breakdown's "sandboxed test
  filesystem" phrasing is thus approximate (local fs +
  `PermissionProfile::Disabled`), but the substantive requirement (F1 patch
  executes through the handler; byte-exact content asserted) is met.

## Gates (re-run by me from `codex-rs/`, 2026-09-16)

**Gate 1 — `just test -p codex-apply-patch`: GREEN.**
`Summary: 115 tests run: 115 passed, 0 skipped` (my run; test phase 4.9s).
Includes all 12 Item-3 ledger rewrites, the golden scenarios
(`suite::scenarios::test_apply_patch_scenarios` PASS), and the apply-patch-side
twin of the :707 test (`suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header`
PASS — exact-stderr against the extended P3.1 message).

**Gate 2 — `just test -p codex-core apply_patch`: PASS per green criterion
(0 deterministic failures).** My run: `Summary [3066.528s] 115 tests run:
69 passed (2 flaky), 7 failed, 39 timed out, 4251 skipped`; recipe EXIT=100
(the recipe exits nonzero on any failed/TMT, i.e. the load noise, not on
product failures). Green criterion applied per the task contract: 0
DETERMINISTIC failures — every failure/TMT must be the known load-noise class
`timeout waiting for event: Elapsed(())` at `core/tests/common/lib.rs:388`.
Verified: ALL 79 `panicked at` lines in the log sit at `lib.rs:388` (0 panic
sites anywhere else), so no deterministic failure occurred. Log:
`/tmp/seatA-gate-core.log`.

All Item-3 tests PASS deterministically in my run (durations from my log):
`function_apply_patch_patch_text_extracts_patch_argument` (0.084s),
`function_apply_patch_patch_text_rejects_non_function_payloads` (0.114s),
`with_environment_id_line_inserts_after_begin_patch_header` (0.103s),
`with_environment_id_line_leaves_malformed_patch_unchanged` (0.112s),
`function_apply_patch_rejects_missing_patch_argument_with_teachable_error`
(0.633s), `function_apply_patch_rejects_non_string_patch_argument_with_teachable_error`
(0.721s), `function_apply_patch_applies_f1_shaped_raw_add_file_patch`
(0.825s), `post_tool_use_payload_uses_patch_input_and_tool_output` (0.729s),
`pre_tool_use_payload_uses_freeform_patch_input` (0.740s), and the :707 core
twin `suite::apply_patch_cli::apply_patch_cli_rejects_invalid_hunk_header`
(0.381s).

The 7 FAIL identities (all `suite::apply_patch_cli::*` — the
process-spawning, most load-sensitive tests; each failed via the
lib.rs:388 TMT-class panic):
- `apply_patch_change_context_disambiguates_target`
- `apply_patch_cli_rejects_duplicate_resolved_paths`
- `apply_patch_cli_rejects_empty_patch`
- `apply_patch_cli_updates_file_appends_trailing_newline`
- `apply_patch_cli_verification_failure_has_no_side_effects`
- `apply_patch_cli_emits_turn_diff_event_with_unified_diff`
- `apply_patch_cli_shell_heredoc_normalizes_crlf_without_preserve_line_endings_feature`

The 39 TMT identities (all process-spawning suite tests; `codex-core::all`
prefix omitted):
- `suite::apply_patch_serialization::apply_patch_custom_tool_call_creates_file`
- `suite::code_mode::code_mode_can_apply_patch_via_nested_tool`
- `suite::apply_patch_serialization::apply_patch_custom_tool_call_updates_existing_file`
- `suite::apply_patch_serialization::apply_patch_custom_tool_call_reports_failure_output`
- `suite::hooks::permission_request_hook_allows_apply_patch_with_write_alias`
- `suite::approvals::approval_matrix_covers_group::apply_patch`
- `suite::hooks::post_tool_use_records_apply_patch_context_with_edit_alias`
- `suite::hooks::post_tool_use_records_additional_context_for_apply_patch`
- `suite::hooks::pre_tool_use_blocks_apply_patch_with_write_alias`
- `suite::apply_patch_cli::apply_patch_aggregates_diff_across_multiple_tool_calls`
- `suite::approvals::approving_apply_patch_for_session_skips_future_prompts_for_same_file`
- `suite::hooks::pre_tool_use_blocks_apply_patch_before_execution`
- `suite::prompt_caching::gpt_5_tools_without_apply_patch_append_apply_patch_instructions`
- `suite::hooks::pre_tool_use_rewrites_apply_patch_before_execution`
- `suite::request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::without_strict_auto_review`
- `suite::request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_session`
- `suite::request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_turn`
- `suite::unified_exec::unified_exec_intercepts_apply_patch_exec_command`
- `suite::shell_snapshot::unified_exec_snapshot_still_intercepts_apply_patch`
- `suite::tool_harness::apply_patch_tool_executes_and_emits_patch_events`
- `suite::tool_harness::apply_patch_reports_parse_diagnostics`
- `suite::apply_patch_cli::apply_patch_aggregates_diff_preserves_success_after_failure`
- `suite::request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::with_strict_auto_review`
- `suite::apply_patch_cli::apply_patch_cli_rejects_move_path_traversal_outside_workspace`
- `suite::apply_patch_cli::apply_patch_cli_rejects_path_traversal_outside_workspace`
- `suite::apply_patch_cli::apply_patch_cli_reports_missing_context`
- `suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff`
- `suite::apply_patch_cli::apply_patch_exec_command_heredoc_with_cd_emits_turn_diff`
- `suite::apply_patch_cli::apply_patch_exec_command_heredoc_with_cd_updates_relative_workdir`
- `suite::apply_patch_cli::apply_patch_normalizes_crlf_without_preserve_line_endings_feature`
- `suite::apply_patch_cli::apply_patch_preserves_crlf_with_preserve_line_endings_feature`
- `suite::apply_patch_cli::apply_patch_shell_accepts_lenient_heredoc_wrapped_patch`
- `suite::apply_patch_cli::apply_patch_shell_heredoc_preserves_crlf_with_preserve_line_endings_feature`
- `suite::apply_patch_cli::apply_patch_turn_diff_paths_stay_repo_relative_when_session_cwd_is_nested`
- `suite::apply_patch_cli::apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::coding_originator_keeps_repository_root_when_disabled`
- `suite::apply_patch_cli::apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::mobile_work_uses_cwd_when_enabled`
- `suite::apply_patch_cli::apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::web_work_uses_cwd_when_enabled`
- `suite::apply_patch_cli::escalated_patch_rejects_symlink_swapped_after_approval_request`
- `suite::apply_patch_cli::intercepted_apply_patch_verification_uses_local_sandbox`

**Gate 3 — `just fix -p codex-apply-patch -p codex-core`: EXIT=0 (1m 01s;
my run, log `/tmp/seatA-item3-fix.log`).** Clippy fixed exactly one file:
`Fixed core/tests/suite/openai_file_mcp.rs (1 fix)` — the pre-existing,
OUT-OF-SCOPE unused `wiremock::matchers::body_json` import (line 47) that
`clippy --fix` re-triggers on every run (identical behavior in the worker's
own 05:14 run and in my item-2 gate run). I restored the file with
`git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs` and verified
the tree returned to exactly the pre-gate change set (`git diff --stat`
byte-identical: 17 files, +1252/−108; import back at line 47). No in-scope
file was modified by clippy. Note: this gate's lint set is clippy; F1 is the
separate Bazel-backed `argument_comment_lint`, which this gate does not
execute, so Gate 3 passing does not clear F1. (My first launch of this gate
lost its background shell session before producing output; the run above is
the valid, fully logged one.)

**Gate 4 — `just fmt`: EXIT=0 (silent — tree already fmt-clean; log
`/tmp/seatA-item3-fmt.log`).** Post-gate `git status`/`git diff --stat`
confirm no file was touched (17-file change set unchanged). No tests were
re-run after fix/fmt, per repo rule.

## Findings (severity, round 1)

| # | Severity | Where | Summary |
|---|----------|-------|---------|
| F1 | **Major** | `core/src/tools/handlers/apply_patch_tests.rs:57` | bare positional `None` missing `/*environment_id*/` comment — CI-enforced strict `argument_comment_lint` (dylint) rejects this; one-line fix |
| F2 | minor | `docs/reviews/impl-item3-tdd-evidence.md` step-6 table | step-3 row arithmetic wrong ("same 4 fns, minus streaming (11 assertions)" vs actual "minus CLI (10 assertions)"); bookkeeping only, progression itself correct |
| F3 | nit | `docs/reviews/impl-item3-tdd-evidence.md` step 7 | two log-file labels swapped/wrong; quoted content authentic |
| F4 | nit | T3.2 test (F1 patch) | writes into the test process cwd with best-effort `let _ =` cleanup; no stray artifact exists now (verified) |

No Blocking findings. All mandates (1–8) verified PASS, invariants hold, the
13-row ledger is green, byte fidelity to spec §3.3 is confirmed at all six
sites, and discrepancy adjudications D1–D5 all land as "real, handled
correctly, no action required" (§Discrepancy adjudications).

F1 is trivially fixable — one line,
`resolve_tool_environment(&step_context.environments, /*environment_id*/ None)`
— and it is a convention CI will reject (strict dylint lint, checked on all
three CI platforms), so per the item protocol — "item proceeds only on a
full round of 0 Blocking + 0 Major" — this round is not a pass.

## Verdict

SEAT A round 1: REQUEST CHANGES — 0 Blocking, 1 Major, 1 minor, 2 nit
