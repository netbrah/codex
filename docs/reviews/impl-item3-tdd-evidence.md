# Item 3 — P3 teachable errors: TDD evidence (apex-ayl.52)

Worker: `/root/item3_p3_worker` (stage 5, item 3 of 6).
Branch: `feat/normalize-content-types-vllm`, working-tree HEAD `76111f29eb`
(post Items 1+2; the campaign function-tool seam remains uncommitted
working-tree state per breakdown §0 execution baseline).
SoT: spec v5 `docs/responses-compat-apply-patch-format.md` (§3.3, §3.2,
§1.1, §4 T3); breakdown v3
`docs/responses-compat-apply-patch-task-breakdown.md` (Item 3, §0, §1, §5).

## Pre-check (line refs re-derived at working-tree state, protocol step 1)

Breakdown/spec line refs were verified before Item 1 landed; Item 1's
module-doc note shifted `parser.rs` by **+1** (and Item 1's AddFile arm
rewrite renumbered the `streaming_parser.rs` test region). Every ref below
was re-verified by reading the current tree before any edit.

**`codex-rs/apply-patch/src/streaming_parser.rs` (934 lines):**
- P3.1 site: `StartedPatch` arm :195-205; `Err(InvalidHunkError {…})`
  :199-205; `format!` message literal :201. (Breakdown pre-shift ref :193.)
- P3.2 site: `DeleteFile` arm :223-233; `Err(InvalidHunkError {…})`
  :227-233; `format!` message literal :229. (Pre-shift ref :222.)
- Parallel diff-consumer messages that stay UNTOUCHED (spec §3.3 scope
  boundary): `finish()` "The last line…" :176 (pre-shift :168);
  `NotStarted` "The first line…" :192 (pre-shift :184);
  `EndedPatch`-content "The last line…" :381 (pre-shift :374).
- Ledger #1 (`test_streaming_patch_parser_returns_errors`, fn :825):
  StartedPatch `'bad'` assertion — `assert_eq!` :836-843, message literal
  :839, `line_number: 2` :840. (Pre-shift ref fn :814 / assert :828.)
- Ledger #3 (same fn): DeleteFile `'bad'` assertion — `assert_eq!`
  :855-862, message literal :858, `line_number: 3` :859. (Pre-shift
  ref :848.)
- Ledger #2 (Item 1, already green): AddFile `'bad'` → `Ok` assertion
  :846-852 — left untouched by this item.
- First assertion of the fn (NotStarted "The first line…") :827-832 —
  NOT in the ledger (P3.1 extends the StartedPatch arm only); untouched.
- Surviving asserts (must stay green, untouched):
  `test_streaming_patch_parser_finish_requires_end_patch` fn :783 (End
  message assert :792-798, message :795);
  `test_streaming_patch_parser_rejects_content_after_end_patch` fn :801
  (End message assert :808, second case :813-821); remaining UpdateFile
  error assertions inside `test_streaming_patch_parser_returns_errors`
  (:865, :874, :885, :894, :903, :913, :923).

**`codex-rs/apply-patch/src/parser.rs` (683 lines):**
- `ParseError::InvalidPatchError` Display `invalid patch: {0}` :57 —
  UNTOUCHED (harness prefix).
- `check_start_and_end_lines_strict` :257-275 (pre-shift :256-274).
  P3.4 Begin message literal :269 (pre-shift :268); P3.4 End message
  literal :272 (pre-shift :271).
- `test_parse_patch` fn :278 (pre-shift :277):
  - Ledger #5 (missing Begin) `assert_eq!` :279-284, message :282
    (pre-shift :281).
  - Ledger #6 (missing End) `assert_eq!` :285-290, message :288
    (pre-shift :287).
- `test_parse_patch_lenient` fn :559 (pre-shift :558):
  - Shared `expected_error` (Begin message) defined :576-577, literal
    :577 — feeds the six Begin assertions.
  - Ledger #7 Strict plain-heredoc `assert_eq!` :580-583 (pre-shift :579).
  - Ledger #8 Strict single-quoted-heredoc `assert_eq!` :595-598
    (pre-shift :594).
  - Ledger #9 Strict double-quoted-heredoc `assert_eq!` :610-613
    (pre-shift :609).
  - Ledger #10 Strict mismatched-quotes `assert_eq!` :625-628
    (pre-shift :624).
  - Ledger #11 Lenient mismatched-quotes `assert_eq!` :629-632
    (pre-shift :628).
  - Ledger #12 Strict missing-closing `assert_eq!` :636-639
    (pre-shift :635).
  - Ledger #13 Lenient missing-closing (End message) `assert_eq!`
    :640-645, message literal :643 (pre-shift :639-642).
  - The four Lenient `Ok(ApplyPatchArgs {…})` assertions (:584-592,
    :599-607, :614-622) do not involve the boundary strings; untouched.

**`codex-rs/apply-patch/tests/suite/tool.rs` (437 lines):**
- Ledger #4: `test_apply_patch_cli_rejects_invalid_hunk_header` fn :386,
  exact-stderr assertion :393 (single line; no shift — Item 1 did not
  touch this file).

**`codex-rs/core/src/tools/handlers/apply_patch.rs` (843 lines; carries
the uncommitted seam, +265/−60 vs HEAD):**
- P3.3 site: `FunctionApplyPatchHandler::handle_call` :508-563; the
  collapsed `value.get("patch").and_then(serde_json::Value::as_str)
  .ok_or_else(…)?` :534-542; old message literal :539
  (`"apply_patch is missing the required `patch` argument"`).
- `FunctionApplyPatchHandler` struct :480-482, `#[derive(Default)]`
  :479 — `FunctionApplyPatchHandler::default()` is available to tests.
- `run_apply_patch_text` :400-468 — the shared parse/verify/apply path
  (F4 boundary errors surface here as
  `"apply_patch verification failed: {parse_error}"`).

**`codex-rs/core/src/tools/handlers/apply_patch_tests.rs` (445 lines;
carries the uncommitted seam, +39 vs HEAD):**
- `invocation_for_payload` :45-61 (scaffolding for T3.1/T3.2); existing
  callers :85 and :103.
- `use pretty_assertions::assert_eq;` :13; `use serde_json::json;` :14
  (line numbers at the 445-line pre-check state; the T3.2 import
  additions later shift them to :16/:17 in the round-1 file).

**Read-only references:**
- `codex-rs/core/tests/suite/apply_patch_cli.rs` (2487 lines):
  surviving substring test `apply_patch_cli_rejects_invalid_hunk_header`
  fn :689; substring assertion `out.contains("is not a valid hunk
  header")` :707. The freeform path reaches the same StartedPatch arm
  via `parse_patch` → streaming parser, so the P3.1 extension (old text
  kept as exact prefix) keeps the substring reachable.
- `codex-rs/core/src/session/tests.rs`: `make_session_and_context`
  :5882 (returns `(Session, TurnContext)`); the cwd accessor is
  `TurnEnvironment::cwd()` (`pub(crate)`, `turn_context.rs:200`,
  `&self.selection.cwd`) — the same path `run_apply_patch_text` applies
  against via `turn_environment.cwd()`.

**Repo-wide sweep (spec §3.3 mandate):** `grep -rn` over `codex-rs/**.rs`
for all four old message strings found assertions ONLY in the ledger
sites above plus the surviving/parallel sites listed above. The old
handler message `apply_patch is missing the required `patch` argument`
exists at exactly one site: `apply_patch.rs:539`. No other test asserts
any of the four old strings.

**Text extraction (mechanical, no retyping):** python sliced the six
spec §3.3 message strings out of `docs/responses-compat-apply-patch-
format.md` (backtick spans), joined the two markdown soft-wraps per
string (all wrap points verified at word boundaries; no double spaces
after join; all-ASCII, `len == bytes`):
- site 1 suffix (leading space; appended after the kept sentence):
  193 chars — `" After '*** Begin Patch', the next line must be a hunk
  header (e.g. '*** Add File: <path>' with every content line prefixed
  by '+'), or '*** Environment ID: <id>' in multi-environment sessions."`
- site 2 replacement: 103 chars.
- site 3a (absent): 121 chars.
- site 3b (non-string): 108 chars.
- site 4a (Begin): 182 chars — old Begin sentence is an exact prefix.
- site 4b (End): 149 chars — old End sentence is an exact prefix.
Byte-verification of the spliced literals is recorded after each sub-step
that lands them (grep the compiled source for the exact spec bytes).

## Baseline gate state (unmodified tree, before any Item 3 edit)

1. `just test -p codex-apply-patch` (from `codex-rs/`):

```text
     Summary [   5.154s] 115 tests run: 115 passed, 0 skipped
```

   Green. (115 = 99 pre-Item-1 + 16 Item-1 P2 tests.)

2. `just test -p codex-core apply_patch` (from `codex-rs/`, ~12.6 min):

```text
     Summary [ 759.175s] 112 tests run: 97 passed (17 flaky), 15 timed out, 4251 skipped
   ...
error: test run failed
error: recipe `test` failed on line 88 with exit code 100
```

   Non-zero exit is the documented pre-existing TMT load-timeout noise
   (task contract: green criterion = 0 DETERMINISTIC failures). 17 flaky
   = FAIL TRY 1 → PASS TRY 2. 0 deterministic failures.
   TMT identities (recorded):
   `suite::apply_patch_serialization::apply_patch_custom_tool_call_reports_failure_output`,
   `suite::approvals::approval_matrix_covers_group::apply_patch`,
   `suite::hooks::permission_request_hook_allows_apply_patch_with_write_alias`,
   `suite::apply_patch_serialization::apply_patch_custom_tool_call_updates_existing_file`,
   `suite::code_mode::code_mode_can_apply_patch_via_nested_tool`,
   `suite::hooks::pre_tool_use_blocks_apply_patch_before_execution`,
   `suite::hooks::post_tool_use_records_additional_context_for_apply_patch`,
   `suite::apply_patch_serialization::apply_patch_custom_tool_call_creates_file`,
   `suite::hooks::post_tool_use_records_apply_patch_context_with_edit_alias`,
   `suite::hooks::pre_tool_use_rewrites_apply_patch_before_execution`,
   `suite::hooks::pre_tool_use_blocks_apply_patch_with_write_alias`,
   `suite::apply_patch_cli::apply_patch_aggregates_diff_across_multiple_tool_calls`,
   `suite::approvals::approving_apply_patch_for_session_skips_future_prompts_for_same_file`,
   `suite::prompt_caching::gpt_5_tools_without_apply_patch_append_apply_patch_instructions`,
   `suite::apply_patch_cli::apply_patch_cli_add_overwrites_existing_file`.
   The surviving substring test `suite::apply_patch_cli::
   apply_patch_cli_rejects_invalid_hunk_header` is in neither the flaky
   nor the TMT list — it PASSED at baseline (deterministic).

## Step 2 — rewrite the 12 ledger assertions → all 12 RED

Rewrites (exact spec §3.3 strings, python-sliced per pre-check):
- Ledger #1 `streaming_parser.rs:836-843`: StartedPatch `'bad'` message →
  old sentence + §3.3.1 suffix (extension).
- Ledger #3 `streaming_parser.rs:855-862`: DeleteFile `'bad'` message →
  §3.3.2 replacement.
- Ledger #4 `tool.rs:393`: exact-stderr → CLI prefix + extended
  StartedPatch message.
- Ledger #5 `parser.rs:279-284`: missing-Begin → §3.3.4a full text.
- Ledger #6 `parser.rs:285-290`: missing-End → §3.3.4b full text.
- Ledger #7-12 `parser.rs:576-578`: the shared `expected_error` literal
  (fed to all six Begin assertions) → §3.3.4a full text.
- Ledger #13 `parser.rs:642-645`: inline End message → §3.3.4b full text.

Test command: `just test -p codex-apply-patch`.

RED (captured verbatim; full log `/tmp/item3-red-step2.log`):

```text
     Summary [  20.783s] 115 tests run: 111 passed, 4 failed, 0 skipped
  TRY 2 FAIL [   1.494s] ( 55/115) codex-apply-patch parser::test_parse_patch
  TRY 2 FAIL [   2.302s] ( 56/115) codex-apply-patch parser::test_parse_patch_lenient
  TRY 2 FAIL [   1.791s] ( 84/115) codex-apply-patch streaming_parser::tests::test_streaming_patch_parser_returns_errors
  TRY 2 FAIL [   1.769s] (114/115) codex-apply-patch::all suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header
error: test run failed
error: recipe `test` failed on line 88 with exit code 100
```

Exactly the 4 failing test fns the design predicted. Each fn aborts at its
FIRST red assertion (Rust `assert_eq!` panics), so the 12 rewritten
assertions surface as 4 panics; the full 12-assertion red state:

Numbering: the table below numbers this item's 12 assertions 1-12;
breakdown §1 ledger rows (13 total) map as table #1 = row 1, #2 = row 3,
#3 = row 4, #4-5 = rows 5-6, #6-12 = rows 7-13 (row 2 is Item 1's,
already green).

| # | Assertion (post-rewrite loc) | Red-state observation |
|---|---|---|
| 1 | `streaming_parser.rs:836-843` StartedPatch extended msg | PANIC :836:9 — observed (first in fn) |
| 2 | `streaming_parser.rs:855-862` DeleteFile replacement msg | blocked behind #1 in same fn |
| 3 | `tool.rs:393` CLI exact stderr | PANIC (stderr diff) — observed |
| 4 | `parser.rs:279-284` missing Begin (§3.3.4a) | PANIC :279:5 — observed (first in fn) |
| 5 | `parser.rs:285-290` missing End (§3.3.4b) | blocked behind #4 in same fn |
| 6 | `parser.rs:580-583` Strict heredoc (via `expected_error`) | PANIC :582:5 — observed (first in fn) |
| 7 | `parser.rs:595-598` Strict single-quote heredoc | blocked behind #6 |
| 8 | `parser.rs:610-613` Strict double-quote heredoc | blocked behind #6 |
| 9 | `parser.rs:625-628` Strict mismatched quotes | blocked behind #6 |
| 10 | `parser.rs:629-632` Lenient mismatched quotes | blocked behind #6 |
| 11 | `parser.rs:636-639` Strict missing-closing | blocked behind #6 |
| 12 | `parser.rs:640-646` Lenient missing-closing End msg | blocked behind #6 |

The four observed panics (verbatim, first try):

```text
thread 'streaming_parser::tests::test_streaming_patch_parser_returns_errors' (85281497) panicked at apply-patch/src/streaming_parser.rs:836:9:
assertion failed: `(left == right)`

Diff < left / right > :
 Err(
     InvalidHunkError {
<        message: "'bad' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'",
>        message: "'bad' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}' After '*** Begin Patch', the next line must be a hunk header (e.g. '*** Add File: <path>' with every content line prefixed by '+'), or '*** Environment ID: <id>' in multi-environment sessions.",
         line_number: 2,
     },
 )

thread 'suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header' (85282092) panicked at …/core/src/ops/function.rs:250:5:
Unexpected stderr, failed diff original var
├── original: Invalid patch hunk on line 2: '*** Frobnicate File: foo' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}' After '*** Begin Patch', the next line must be a hunk header (e.g. '*** Add File: <path>' with every content line prefixed by '+'), or '*** Environment ID: <id>' in multi-environment sessions.
├── diff:
│   --- orig
│   +++ var
│   @@ -1 +1 @@
│   -Invalid patch hunk on line 2: … After '*** Begin Patch', the next line must be a hunk header (e.g. '*** Add File: <path>' with every content line prefixed by '+'), or '*** Environment ID: <id>' in multi-environment sessions.
│   +Invalid patch hunk on line 2: '*** Frobnicate File: foo' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'

thread 'parser::test_parse_patch' (85281015) panicked at apply-patch/src/parser.rs:279:5:
assertion `left == right` failed
  left: Err(InvalidPatchError("The first line of the patch must be '*** Begin Patch'"))
 right: Err(InvalidPatchError("The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'."))

thread 'parser::test_parse_patch_lenient' (85280948) panicked at apply-patch/src/parser.rs:582:5:
assertion `left == right` failed
  left: Err(InvalidPatchError("The first line of the patch must be '*** Begin Patch'"))
 right: Err(InvalidPatchError("The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'."))
```

No other test failed (111 passed) — the only red state is the ledger.

## Step 3 — land P3.1 (sentence extension) → 10 red

Change: `streaming_parser.rs` StartedPatch arm :201 — appended the
§3.3.1 suffix to the `format!` literal (no braces in the suffix; the
`{trimmed}` interpolation and `{{path}}` escapes untouched; the original
sentence stays byte-identical as a prefix).

Byte-verification (post-edit): the source literal now reads
`'*** Update File: {{path}}' After '*** Begin Patch', the next line must
be a hunk header (e.g. '*** Add File: <path>' with every content line
prefixed by '+'), or '*** Environment ID: <id>' in multi-environment
sessions."` — i.e. old rendered sentence + exact spec suffix
(python re-slice of spec §3.3 item 1 compared byte-for-byte against the
spliced suffix in the source: identical, 193 bytes incl. leading space).

Test command: `just test -p codex-apply-patch`.

Result (captured; full log `/tmp/item3-red-step3.log`):

```text
     Summary [   7.778s] 115 tests run: 112 passed, 3 failed, 0 skipped
  TRY 2 FAIL [   0.139s] ( 51/115) codex-apply-patch parser::test_parse_patch
  TRY 2 FAIL [   0.205s] ( 56/115) codex-apply-patch parser::test_parse_patch_lenient
  TRY 2 FAIL [   0.155s] ( 78/115) codex-apply-patch streaming_parser::tests::test_streaming_patch_parser_returns_errors
```

Ledger #1 and #4 green (12 → 10 red). The streaming errors test now
passes the StartedPatch assertion and aborts one assertion later, at the
DeleteFile assertion (ledger #3, still red):

```text
thread 'streaming_parser::tests::test_streaming_patch_parser_returns_errors' (85289229) panicked at apply-patch/src/streaming_parser.rs:855:9:
assertion failed: `(left == right)`

Diff < left / right > :
 Err(
     InvalidHunkError {
<        message: "'bad' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'",
>        message: "'Delete File' hunks take no content lines; the next line must be another hunk header or '*** End Patch'",
         line_number: 3,
     },
 )
```

`suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header` PASSED
(its exact-stderr now matches the extended message through the CLI).
No new failures beyond the un-landed ledger sites.

### Step 3 surviving-substring check (spec/breakdown mandate)

Command: `just test -p codex-core apply_patch` (from `codex-rs/`, ~27.6 min
under heavy machine load):

```text
     Summary [1658.581s] 112 tests run: 80 passed (1 slow, 8 flaky), 13 failed, 19 timed out, 4251 skipped
error: recipe `test` failed on line 88 with exit code 100
```

Diagnosis (per task contract: green criterion = 0 DETERMINISTIC failures):
- Every panic in the entire run (58 of them) is the pre-existing TMT
  signature `timeout waiting for event: Elapsed(())` at
  `core/tests/common/lib.rs:388`. `grep "panicked" | grep -v lib.rs:388`
  → **0 non-TMT panics** → 0 deterministic failures. The 13 "failed"
  (both tries TMT) + 19 "timed out" are all load noise; this run's 1658s
  vs the baseline's 759s confirms degraded machine load during the run.
- **The surviving substring test
  `suite::apply_patch_cli::apply_patch_cli_rejects_invalid_hunk_header`
  (apply_patch_cli.rs:689, assert :707) is FLAKY 2/2: TRY 1 failed with
  the TMT event-wait timeout (47.270s, lib.rs:388 — noise, not an
  assertion failure), TRY 2 PASSED in 0.452s.** The P3.1 extension keeps
  `is not a valid hunk header` reachable on that path, so the substring
  assertion holds and the test is UNTOUCHED (no edit made).
- TMT identities recorded (full list, deduped):
  apply_patch_cli::apply_patch_aggregates_diff_across_multiple_tool_calls,
  apply_patch_cli::apply_patch_aggregates_diff_preserves_success_after_failure,
  apply_patch_cli::apply_patch_cli_delete_directory_reports_verification_error,
  apply_patch_cli::apply_patch_cli_delete_missing_file_reports_error,
  apply_patch_cli::apply_patch_cli_does_not_widen_permissions_for_workspace_directory_target,
  apply_patch_cli::apply_patch_cli_move_overwrites_existing_destination,
  apply_patch_cli::apply_patch_cli_multiple_chunks,
  apply_patch_cli::apply_patch_cli_multiple_operations_integration,
  apply_patch_cli::apply_patch_cli_preserves_distinct_updated_paths,
  apply_patch_cli::apply_patch_custom_tool_streaming_emits_updated_changes,
  apply_patch_cli::apply_patch_shell_heredoc_preserves_crlf_with_preserve_line_endings_feature,
  apply_patch_serialization::apply_patch_custom_tool_call_creates_file,
  apply_patch_serialization::apply_patch_custom_tool_call_reports_failure_output,
  apply_patch_serialization::apply_patch_custom_tool_call_updates_existing_file,
  approvals::approval_matrix_covers_group::apply_patch,
  approvals::approving_apply_patch_for_session_skips_future_prompts_for_same_file,
  code_mode::code_mode_can_apply_patch_via_nested_tool,
  hooks::permission_request_hook_allows_apply_patch_with_write_alias,
  hooks::post_tool_use_records_additional_context_for_apply_patch,
  hooks::post_tool_use_records_apply_patch_context_with_edit_alias,
  hooks::pre_tool_use_blocks_apply_patch_before_execution,
  hooks::pre_tool_use_blocks_apply_patch_with_write_alias,
  hooks::pre_tool_use_rewrites_apply_patch_before_execution,
  prompt_caching::gpt_5_tools_without_apply_patch_append_apply_patch_instructions,
  request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_session,
  request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_turn,
  request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::with_strict_auto_review,
  request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::without_strict_auto_review,
  shell_snapshot::unified_exec_snapshot_still_intercepts_apply_patch,
  tool_harness::apply_patch_reports_parse_diagnostics,
  tool_harness::apply_patch_tool_executes_and_emits_patch_events,
  unified_exec::unified_exec_intercepts_apply_patch_exec_command.

## Step 4 — land P3.2 (replacement) → 9 red

Change: `streaming_parser.rs` DeleteFile arm :227-231 — the `format!`
"not a valid hunk header" message replaced with the §3.3.2 constant
(wholesale replacement; shares no prefix with the old text; `format!`
dropped — the message quotes no user input). `trimmed` is still consumed
by `handle_hunk_headers_and_end_patch(trimmed)`, so no unused-variable
change.

Byte-verification: source literal now equals the python-sliced spec
§3.3 item-2 string byte-for-byte (103 ASCII bytes, no escapes needed).

Test command: `just test -p codex-apply-patch`.

Result (captured; full log `/tmp/item3-red-step4.log`):

```text
     Summary [   5.412s] 115 tests run: 113 passed, 2 failed, 0 skipped
  TRY 2 FAIL [   0.125s] ( 44/115) codex-apply-patch parser::test_parse_patch
  TRY 2 FAIL [   0.164s] ( 51/115) codex-apply-patch parser::test_parse_patch_lenient
error: test run failed
error: recipe `test` failed on line 88 with exit code 100
```

Ledger #3 green (12 → 10 → 9 red). The streaming errors test now passes
in full (StartedPatch, AddFile-Ok, DeleteFile assertions plus all
surviving UpdateFile assertions). The only red fns are the two parser
boundary tests (ledger #5-13), as designed.

## Step 5 — T3.1 RED first, then P3.3, then T3.1 GREEN

Scaffolding change (recorded per task contract — strictly required):
`invocation_for_payload` (apply_patch_tests.rs:45) now returns
`(ToolInvocation, PathUri)` — the second element is
`turn.cwd().clone()` captured before the turn is moved into an `Arc`.
Reason: T3.2 must assert the file the handler actually created, and the
handler applies against `turn_environment.cwd()` — the session's cwd,
which the test can only learn from the returned `TurnContext`
(`Session::cwd()` is `pub(super)` and invisible from `tools::handlers`;
`TurnEnvironment::cwd()` is `pub(crate)`, `turn_context.rs:200`). The two
pre-existing callers (:85, :103) were updated mechanically to
`let (invocation, _) = …`.

New tests appended to apply_patch_tests.rs (whole-exact-string
assertions, `pretty_assertions::assert_eq` already imported at :13):
- `function_apply_patch_rejects_missing_patch_argument_with_teachable_error`
  — `ToolPayload::Function { arguments: r#"{}"# }` → exact §3.3.3a
  message.
- `function_apply_patch_rejects_non_string_patch_argument_with_teachable_error`
  — `arguments = json!({ "patch": { "raw": "not a string" } })` (patch
  present but a JSON object) → exact §3.3.3b message.

Byte-verification: both expected literals byte-identical to the
python-sliced spec §3.3 item-3 strings (121 / 108 ASCII bytes);
argument name single-quoted per the site-3 re-quote status (old message
used backticks).

### T3.1 RED

Compile fix during authoring (test code only, recorded):
- `turn.cwd()` does not exist — `TurnContext` has only the DEPRECATED
  `pub(crate) cwd: AbsolutePathBuf` field (`turn_context.rs:314`,
  "use the selected turn environment cwd instead"); the non-deprecated
  accessor is `TurnEnvironment::cwd()` (`turn_context.rs:200`,
  `&self.selection.cwd`). The helper therefore resolves the primary
  environment through the SAME helper the handler uses:
  `crate::tools::handlers::resolve_tool_environment(
  &step_context.environments, None)` (`handlers/mod.rs:160`) and takes
  `.cwd()`.
- `Result::expect_err` requires the Ok type (`Box<dyn ToolOutput>`) to be
  `Debug`; it is not. Replaced with `match … { Err(err) => err,
  Ok(_) => panic!(…) }` (mirrors the `let Err(…) = … else { panic!(…) }`
  idiom already used in `unified_exec_tests.rs`).

First compile attempt failed with exactly those two errors (E0599 on
`turn.cwd()`, E0277 ×2 on `expect_err`); no product-code compile issues.

Test command (module scope, same binary as the item command):
`just test -p codex-core tools::handlers::apply_patch::tests` — 19 tests
(the whole test module, including the two updated `invocation_for_payload`
callers):

```text
  TRY 2 FAIL [   0.236s] (18/19) codex-core tools::handlers::apply_patch::tests::function_apply_patch_rejects_non_string_patch_argument_with_teachable_error
  TRY 2 FAIL [   0.237s] (19/19) codex-core tools::handlers::apply_patch::tests::function_apply_patch_rejects_missing_patch_argument_with_teachable_error
```

Both T3.1 tests RED (full log `/tmp/item3-step5-module2.log`); the other
17 module tests (incl. both updated callers) PASS — the scaffolding
change is behavior-neutral. Verbatim panics (first try):

```text
thread 'tools::handlers::apply_patch::tests::function_apply_patch_rejects_missing_patch_argument_with_teachable_error' (85560018) panicked at core/src/tools/handlers/apply_patch_tests.rs:468:5:
assertion failed: `(left == right)`

Diff < left / right > :
 RespondToModel(
<    "apply_patch is missing the required `patch` argument",
>    "apply_patch is missing the required 'patch' argument; pass the full patch text starting with '*** Begin Patch' in 'patch'",
 )

thread 'tools::handlers::apply_patch::tests::function_apply_patch_rejects_non_string_patch_argument_with_teachable_error' (85560002) panicked at core/src/tools/handlers/apply_patch_tests.rs:488:5:
assertion failed: `(left == right)`

Diff < left / right > :
 RespondToModel(
<    "apply_patch is missing the required `patch` argument",
>    "apply_patch 'patch' argument must be a string containing the full patch text starting with '*** Begin Patch'",
 )
```

Exactly as designed: the collapsed handler at
`apply_patch.rs:534-542` (pre-split) returns the SAME old message for
both shapes, so both new tests fail against their distinct new
expectations.

### P3.3 landed (handler split)

Change: `apply_patch.rs` `handle_call` — the collapsed
`value.get("patch").and_then(serde_json::Value::as_str).ok_or_else(…)?`
:534-542 replaced with two steps:
1. `value.get("patch").ok_or_else(… §3.3.3a …)?` → `&Value`
2. `patch_value.as_str().ok_or_else(… §3.3.3b …)?` → `&str`, then
   `.to_string()` (as before).

Byte-verification: both literals byte-identical to the python-sliced
spec §3.3.3a/3b strings; no behavior change beyond the two error
messages (the successful path produces the identical
`patch_input: String`).

### T3.1 GREEN

Test command: `just test -p codex-core tools::handlers::apply_patch::tests`:

```text
     Summary [   0.593s] 19 tests run: 19 passed, 4346 skipped
```

Both T3.1 tests PASS with the split messages (full log
`/tmp/item3-step5-green.log`). The item-wide command
`just test -p codex-core apply_patch` was re-run for the record
(result in the next section; ledger count stays 9 — P3.3 owns no ledger
rows; the apply-patch suite state was re-confirmed at 2 red fns / 9 red
assertions in step 5 before the T3.1 tests were added, and the
apply-patch crate was untouched between that run and step 6).

### Step 5 full item command (record)

Command: `just test -p codex-core apply_patch` (from `codex-rs/`,
~26.7 min under heavy machine load; now 114 tests = 112 baseline + 2 new):

```text
     Summary [1602.386s] 114 tests run: 81 passed (4 flaky), 14 failed, 19 timed out, 4251 skipped
error: test run failed
error: recipe `test` failed on line 88 with exit code 100
```

- Both T3.1 tests PASS deterministically:
  `PASS [3.289s] (13/114) …function_apply_patch_rejects_missing_patch_argument_with_teachable_error`,
  `PASS [3.620s] (22/114) …function_apply_patch_rejects_non_string_patch_argument_with_teachable_error`.
- **0 non-TMT panics** (`grep "panicked" | grep -v lib.rs:388` → 0) → 0
  deterministic failures; the 14 failed-after-retry + 19 timed out are
  all the documented TMT event-wait timeout at
  `core/tests/common/lib.rs:388`. TMT identities (failed-after-retry
  set): apply_patch_cli::apply_patch_aggregates_diff_preserves_success_after_failure,
  apply_patch_cli::apply_patch_cli_multiple_chunks,
  apply_patch_cli::apply_patch_cli_rejects_invalid_hunk_header (TMT on
  BOTH tries this run — load noise; it PASSED deterministically in the
  step-3 run, and its assertion target is untouched by this step),
  apply_patch_cli::apply_patch_cli_rejects_move_path_traversal_outside_workspace,
  apply_patch_cli::apply_patch_cli_rejects_path_traversal_outside_workspace,
  apply_patch_cli::apply_patch_emits_turn_diff_event_with_unified_diff,
  request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_session,
  request_permissions::denied_child_permissions_require_fresh_approval::apply_patch_turn,
  request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::with_strict_auto_review,
  request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::without_strict_auto_review,
  shell_snapshot::unified_exec_snapshot_still_intercepts_apply_patch,
  tool_harness::apply_patch_reports_parse_diagnostics,
  tool_harness::apply_patch_tool_executes_and_emits_patch_events,
  unified_exec::unified_exec_intercepts_apply_patch_exec_command.

Red-ledger state after step 5: 9 red (unchanged by P3.3, which owns no
ledger rows): `parser::test_parse_patch` (2) + `parser::test_parse_patch_lenient`
(7), both still failing on the §3.3.4a/b boundary messages.

## Step 6 — land P3.4 (both boundary strings) → 0 red

Change: `parser.rs` `check_start_and_end_lines_strict` — both boundary
error strings extended to the §3.3.4a/§3.3.4b constants (the single owner
of these strings; reached by every `parse_patch` caller per spec scope
note). Production change only — the 9 parser test expectations were
already rewritten to the new strings in step 2, so this is the only hunk
this step lands:

```diff
@@ check_start_and_end_lines_strict
         (Some(first), _) if first != BEGIN_PATCH_MARKER => Err(InvalidPatchError(String::from(
-            "The first line of the patch must be '*** Begin Patch'",
+            "The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'.",
        ))),
         _ => Err(InvalidPatchError(String::from(
-            "The last line of the patch must be '*** End Patch'",
+            "The last line of the patch must be '*** End Patch'. The terminator line is exactly '*** End Patch' with no '+' or other prefix and no lines after it.",
        ))),
```

Byte-verification (re-derived at step 6, python against the spec §3.3.4
backtick text): 4a = 182 chars, 4b = 149 chars, all-ASCII; both keep the
old sentence as an exact prefix (spec prefix status for sites 1 and 4).

Test command: `just test -p codex-apply-patch`.

Result (captured; full log `/tmp/item3-step6-green.log`):

```text
     Summary [   5.224s] 115 tests run: 115 passed, 0 skipped
EXIT=0
```

Red-ledger state after step 6: 0 red. Full progression across the five
landing steps: **12 → 10 → 9 → 9 → 0**.

| step | change landed | red fns (failing fns, not assertions) |
| --- | --- | --- |
| 2 | 12 ledger assertions rewritten to spec strings | `parser::test_parse_patch`, `parser::test_parse_patch_lenient`, `streaming_parser::tests::test_streaming_patch_parser_returns_errors`, `suite::tool::test_apply_patch_cli_rejects_invalid_hunk_header` (4 fns / 12 assertions) |
| 3 | P3.1 StartedPatch suffix (streaming :201) | `parser::test_parse_patch`, `parser::test_parse_patch_lenient`, `streaming_parser::tests::test_streaming_patch_parser_returns_errors` (3 fns / 10 assertions — the CLI fn went green; the streaming fn is still red on the DeleteFile assertion) |
| 4 | P3.2 DeleteFile replacement (streaming) | `parser::test_parse_patch`, `parser::test_parse_patch_lenient` (9 assertions) |
| 5 | P3.3 handler split (apply_patch.rs) | unchanged (9 assertions) — P3.3 owns no ledger rows |
| 6 | P3.4 both boundary strings (parser.rs) | none (0) |

## Step 7 — T3.2: F1 raw Add-File through the function handler (hang → custom session → GREEN)

Test: `function_apply_patch_applies_f1_shaped_raw_add_file_patch` in
`apply_patch_tests.rs` — the spec §1.1 F1 shape (raw markdown Add-File, H1
first content line, no per-line `+`) applied through
`FunctionApplyPatchHandler::handle` against LOCAL_FS, asserting the exact
created file content.

### First attempt — deterministic hang (TMT both tries)

The test first ran against the existing `invocation_for_payload`
scaffolding in a full module-scope run — the other 19 tests passed and
the F1 test timed out on both tries (not load noise — see the
determinism note below; log `/tmp/item3-step7-module.log`):

```text
   TRY 1 TMT [  60.011s] codex-core tools::handlers::apply_patch::tests::function_apply_patch_applies_f1_shaped_raw_add_file_patch
   TRY 2 TMT [  60.009s] codex-core tools::handlers::apply_patch::tests::function_apply_patch_applies_f1_shaped_raw_add_file_patch
     Summary [ 120.040s] 20 tests run: 19 passed, 1 timed out, 4346 skipped
```

A single-test rerun reproduced it in isolation
(`/tmp/item3-t32-debug.log`; TMT both tries, `Summary [ 120.035s] 1 test
run: 0 passed, 1 timed out`). A temporary DIAG block (a bare
`eprintln!`, since unit-test binaries have no tracing subscriber —
`RUST_LOG` is useless there) then identified the cause. The first DIAG
iteration (`/tmp/item3-t32-diag.log`) failed to compile — E0425
`assess_patch_safety` not in scope and E0308 `&PathUri` vs `PathUri` at
the `verify_apply_patch_args_with_mode` call — both fixed in the
diagnostic block before the DIAG run (`/tmp/item3-t32-diag2.log`; TMT
both tries, `Summary [ 120.031s] 1 test run: 0 passed, 1 timed out`),
which printed identically on both tries:

```text
DIAG approval_policy = OnRequest permission_profile = Managed { file_system: Restricted { entries: [FileSystemSandboxEntry { path: Special { value: Root }, access: Read, missing_path_behavior: None }], glob_scan_max_depth: None }, network: Restricted } fs_policy = FileSystemSandboxPolicy { kind: Restricted, glob_scan_max_depth: None, entries: [FileSystemSandboxEntry { path: Special { value: Root }, access: Read, missing_path_behavior: None }] } sandbox_available = true safety = AskUser
```

Root cause: `make_session_and_context` builds a read-only session
(`Managed { fs: Restricted { Root: Read } }`) under
`AskForApproval::OnRequest`. The F1 patch writes
`grok/plans/spec-freeze-r23-glm.md` under the temp cwd, so
`assess_patch_safety` evaluates `AskUser`; the approval request has no
answerer in a unit test and `handler.handle` waits forever. Determinism
evidence: identical DIAG output on both tries, no `lib.rs:388`
`timeout waiting for event: Elapsed(())` signature (that signature is the
load-noise TMT marker), and the test never reached the file assertion.

### Discrepancy (recorded for the review record)

The task breakdown assumed T3.2 could reuse the existing
`invocation_for_payload` scaffolding as-is. That assumption is false at
the source: the scaffolding's session shape is read-only + on-request
approval and cannot complete any write. T3.1's two tests are unaffected
because their payloads error before environment/safety code runs.

### Fix (test-only; owned file `apply_patch_tests.rs`)

1. Extracted `invocation_from_session(payload, session: Arc<Session>,
   turn: Arc<TurnContext>) -> (ToolInvocation, PathUri)` from
   `invocation_for_payload`, which now delegates after
   `make_session_and_context()`. The two pre-existing callers
   (`pre_tool_use_payload_uses_freeform_patch_input`,
   `post_tool_use_payload_uses_patch_input_and_tool_output`) are
   unchanged.
2. T3.2 builds its own session via
   `make_session_and_context_with_auth_and_config_and_rx`
   (`session/tests.rs`, `pub(crate)`) with
   `Permissions::from_approval_and_profile(Constrained::allow_any(AskForApproval::Never),
   Constrained::allow_only(PermissionProfile::Disabled))` — the same shape
   the integration harness uses (`core/tests/common/test_codex.rs:1006-1007`
   submit helpers), imported as `crate::config::Constrained` /
   `crate::config::Permissions`.
3. Removed the DIAG block.

Assertion notes: the handler result is matched (not `expect`ed) because
the Ok value is `Box<dyn ToolOutput>` (not `Debug`); the F1 patch literal
uses Rust line-continuations (the blank content line is the source line
containing only `\n`), and the em dash round-trips through serde_json in
the JSON arguments. T3.1's tests keep the default read-only session.

### Result (module scope; log `/tmp/item3-t32-module.log`)

```text
        PASS [   0.455s] (16/20) codex-core tools::handlers::apply_patch::tests::function_apply_patch_applies_f1_shaped_raw_add_file_patch
     Summary [   0.489s] 20 tests run: 20 passed, 4346 skipped
EXIT=0
```

Compile note: the run emitted exactly one warning — the pre-existing
unused `wiremock::matchers::body_json` import at
`core/tests/suite/openai_file_mcp.rs:47` (present at HEAD, out of scope).

## Discrepancies (breakdown assumptions vs. source)

1. **T3.2 session shape (material).** The breakdown assumed the existing
   `invocation_for_payload` scaffolding sufficed for T3.2. It does not:
   the scaffolding session is read-only + `OnRequest`, so a write patch
   hits `safety = AskUser` and the handler deadlocks on an unanswered
   approval (deterministic hang, both tries; full evidence in step 7).
   Fix: extracted `invocation_from_session` and gave T3.2 the
   integration-harness session shape (`AskForApproval::Never` +
   `PermissionProfile::Disabled`). T3.1 needed no change (its payloads
   error before env/safety code).
2. **`invocation_for_payload` signature change (mechanical).** The
   scaffolding helper at HEAD returned `ToolInvocation`; it now returns
   `(ToolInvocation, PathUri)` so T3.2 can assert against the created
   file under the session cwd. Its two pre-existing callers were
   updated mechanically to `let (invocation, _) = ...`; their behavior
   is unchanged (both pass in the 20/20 module run).
3. **Pre-existing warning, out of scope.** `core/tests/suite/openai_file_mcp.rs:47`
   has an unused `wiremock::matchers::body_json` import at HEAD; every
   core test build emits it. Not touched (not an owned file). Note for
   the gate record: `just fix` may auto-remove it (`cargo fix` applies
   unused-import suggestions); if so it is restored by
   `git checkout --` and recorded in the gate section.
4. **Machine-load TMT noise (binding protocol).** Core runs take
   13–28 min with 15–32 `timeout waiting for event: Elapsed(())`
   timeouts at `core/tests/common/lib.rs:388` under current machine
   load; the green criterion is **0 deterministic failures**, verified
   per run by
   `grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u`
   being empty. The step-5 full run had
   `apply_patch_cli::apply_patch_cli_rejects_invalid_hunk_header` TMT
   on both tries; it passed deterministically in the step-3 run (and its
   assertion target is Item 1's, untouched by Item 3). The step-7 T3.2
   hang was explicitly NOT this class of noise (no lib.rs:388 signature;
   see step 7).
5. **Spec §3.3 line refs shifted.** The spec cites apply_patch.rs:508-560
   for the collapsed argument-error path and parser.rs:256-274 for
   `check_start_and_end_lines_strict`; both re-derived at the working
   tree in the pre-check section (handler split now at
   apply_patch.rs:534/541; boundary fn at parser.rs:257-276). All four
   message sites verified byte-for-byte against the spec v5 text
   (193/103/121/108/182/149 char counts in the pre-check).

## Gates (post-implementation, run in order from `codex-rs/`)

### Gate 1 — `just test -p codex-apply-patch` → GREEN

```text
     Summary [  98.893s] 115 tests run: 115 passed, 0 skipped
EXIT=0
```

Log: `/tmp/item3-gate-ap.log`. All 12 ledger assertions green (step 6).

### Gate 2 — `just test -p codex-core apply_patch` → 0 deterministic failures

Summary (log `/tmp/item3-gate-core.log`; run took 2117s ≈ 35 min under
extreme machine load — beyond the expected 13–28 min window):

```text
     Summary [2117.020s] 115 tests run: 77 passed (3 flaky), 5 failed, 33 timed out, 4251 skipped
```

Green criterion (binding, see discrepancy 4): every panic in the log
verified against the TMT signature. Command:

```text
grep -E "thread .+ panicked" /tmp/item3-gate-core.log | grep -v 'lib.rs:388' | sort -u
# → (empty)
```

All 64 panics in the run are
`core/tests/common/lib.rs:388:14: timeout waiting for event: Elapsed(())`
— the load-noise class. **Zero deterministic failures.**

Noisy-test identities this run (all lib.rs:388 class):

- Final FAIL (5): `apply_patch_clears_aggregated_diff_after_inexact_delta`,
  `apply_patch_cli_move_without_content_change_has_no_turn_diff`,
  `apply_patch_cli_multiple_operations_integration`,
  `apply_patch_preserves_crlf_with_preserve_line_endings_feature`,
  `apply_patch_shell_heredoc_normalizes_crlf_without_preserve_line_endings_feature`
  (all `suite::apply_patch_cli`).
- Final TMT (33): `apply_patch_cli`: aggregates_diff_across_multiple_tool_calls,
  aggregates_diff_preserves_success_after_failure, change_context_disambiguates_target,
  cli_can_use_exec_command_output_as_patch_input, cli_delete_directory_reports_verification_error,
  cli_delete_missing_file_reports_error, cli_does_not_widen_permissions_for_workspace_directory_target,
  cli_does_not_write_through_symlink_escape_outside_workspace, cli_end_of_file_anchor,
  cli_moves_file_to_new_directory, normalizes_crlf_without_preserve_line_endings_feature,
  shell_heredoc_preserves_crlf_with_preserve_line_endings_feature; `apply_patch_serialization`:
  custom_tool_call_creates_file, custom_tool_call_reports_failure_output,
  custom_tool_call_updates_existing_file; `approvals`: approval_matrix_covers_group::apply_patch,
  approving_apply_patch_for_session_skips_future_prompts_for_same_file; `code_mode`:
  code_mode_can_apply_patch_via_nested_tool; `hooks`: permission_request_hook_allows_apply_patch_with_write_alias,
  post_tool_use_records_additional_context_for_apply_patch,
  post_tool_use_records_apply_patch_context_with_edit_alias,
  pre_tool_use_blocks_apply_patch_before_execution,
  pre_tool_use_blocks_apply_patch_with_write_alias,
  pre_tool_use_rewrites_apply_patch_before_execution; `prompt_caching`:
  gpt_5_tools_without_apply_patch_append_apply_patch_instructions;
  `request_permissions`: denied_child_permissions_require_fresh_approval::{apply_patch_session, apply_patch_turn};
  `request_permissions_tool`: approved_folder_write_request_permissions_unblocks_later_apply_patch::{with_strict_auto_review, without_strict_auto_review};
  `shell_snapshot`: unified_exec_snapshot_still_intercepts_apply_patch;
  `tool_harness`: apply_patch_reports_parse_diagnostics,
  apply_patch_tool_executes_and_emits_patch_events; `unified_exec`:
  unified_exec_intercepts_apply_patch_exec_command.
- Flaky (passed on retry; 3): `apply_patch_cli_insert_only_hunk_modifies_file`,
  `apply_patch_cli_multiple_chunks`,
  `apply_patch_turn_diff_paths_stay_repo_relative_when_session_cwd_is_nested`.

Overlap note: the 33 TMTs here match the noise identities from the
step-3/step-5 core runs (same lib.rs:388 class, same test population),
which confirms machine-load origin rather than a regression introduced by
Item 3. The item-3-owned core test population in this run
(`tools::handlers::apply_patch::tests::*`, 20 tests) is a unit-test
binary: all 20 PASS within this gate run itself (0 TMT/FAIL/FLAKY among
them), and also in the module-scope run (step 7); none appears in the
noisy sets above.

### Gate 3 — `just fix -p codex-apply-patch -p codex-core`

```text
cargo clippy --fix --tests --allow-dirty {args}
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 33s
EXIT=0
```

Log: `/tmp/item3-fix.log`. Clippy made **zero** changes to the owned
files. It did re-trigger the known hazard (discrepancy 3):
`cargo fix` auto-removed the pre-existing unused
`wiremock::matchers::body_json` import in the out-of-scope file
`codex-rs/core/tests/suite/openai_file_mcp.rs:47`. Restored immediately
with `git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs`;
`git status` confirms the file is back to its HEAD/seam state and no
other out-of-scope file was touched.

### Gate 4 — `just fmt`

```text
EXIT=0
```

Log: `/tmp/item3-fmt.log`. Follow-up `cargo fmt -p codex-apply-patch
-p codex-core --check` → exit 0 (tree is fmt-clean; only the pre-existing
`imports_granularity` nightly-only warnings from the tooling itself).
Per repo protocol, no tests were re-run after fix/fmt.

## Final diff (owned files, uncommitted)

`git diff --numstat` scoped to the five tracked owned files
(insertions / deletions):

```text
8    6    codex-rs/apply-patch/src/parser.rs
5    6    codex-rs/apply-patch/src/streaming_parser.rs
1    1    codex-rs/apply-patch/tests/suite/tool.rs
265  60   codex-rs/core/src/tools/handlers/apply_patch.rs
164  15   codex-rs/core/src/tools/handlers/apply_patch_tests.rs
```

- `apply_patch.rs` carries the pre-existing uncommitted seam (the large
  environment/permissions plumbing predates Item 3); the Item-3-specific
  part of that file is the P3.3 two-error split at :534/:541.
- `apply_patch_tests.rs` carries the seam's test additions plus Item 3's
  T3.1/T3.2 tests and the `invocation_from_session` scaffold refactor.
- Untracked: this evidence doc, `docs/reviews/impl-item3-tdd-evidence.md`
  (1105 lines).
- `git status` (tracked) matches exactly: the five owned files above plus
  the pre-existing seam files (AGENTS.md; codex-api
  content_type_compat{,_tests}.rs + responses.rs; core handlers/mod.rs +
  spec_plan.rs; model-provider amazon_bedrock/mod.rs + provider.rs;
  models-manager models.json + manager_tests.rs + model_info{,_tests}.rs).
  No unexpected modifications; nothing committed (coordinator commits).

## Verdict

All five ledger landing steps green in order (12 → 10 → 9 → 9 → 0 red
assertions), T3.1 RED→GREEN, T3.2 GREEN after the custom-session fix,
and all four gates pass (115/115 apply-patch; 0 deterministic core
failures; clean fix/fmt with the known out-of-scope auto-fix restored).
Discrepancies recorded above: 1 material (T3.2 session shape), 4
mechanical/noise (scaffold signature, pre-existing warning + fix
re-trigger, load-noise TMT class, spec line-ref shift).

## Round 1 fixes (r1 → r2)

### Code fix (the Major — both seats F1/M-B1, identical finding)

`apply_patch_tests.rs:57` — the positional `None` passed to
`resolve_tool_environment` lacked the argument comment required by the
`argument_comment_lint` convention (CI enforces it on `#[cfg(test)]`
call sites too). Verified at source: the callee signature
(`handlers/mod.rs:160-163`) names the parameter `environment_id`:

```rust
fn resolve_tool_environment<'a>(
    environments: &'a TurnEnvironmentSnapshot,
    environment_id: Option<&str>,
) -> Result<Option<&'a TurnEnvironment>, FunctionCallError> {
```

Exact one-line change:

```diff
-    let cwd = crate::tools::handlers::resolve_tool_environment(&step_context.environments, None)
+    let cwd = resolve_tool_environment(&step_context.environments, /*environment_id*/ None)
```

The unqualified name (in place of the fully-qualified path) is required
to keep the line within rustfmt's 100-col limit **with** the 18-char
comment: the qualified form is 115 cols (r2 seat D correction) and `just fmt` would reflow it,
changing the file's numstat. The name resolves through
`use super::*` — the parent module's private
`use crate::tools::handlers::resolve_tool_environment;`
(apply_patch.rs:29) is re-exported into the test module by the glob —
the same convention the file already uses for
`apply_patch_file_update_mode` (called unqualified at :87/:96)
(r2 seats C/D corrections).

Verified: `cargo check -p codex-core --tests` → exit 0 (only the
pre-existing openai_file_mcp warning); `cargo fmt -p codex-core
--check` → exit 0 (line now 91 cols, comment retained); file numstat
UNCHANGED at +164/−15 (confirmed again after the round-1 gates, below).

### Evidence-doc corrections (each re-derived from raw /tmp logs + current tree)

1. **(B m-B3)** `+321/−60` → `+265/−60` for the apply_patch.rs seam
   numstat (pre-check section). Source: re-ran
   `git diff --numstat codex-rs/core/src/tools/handlers/apply_patch.rs`
   → `265  60`. Single occurrence; the Final-diff table already carried
   the correct value.
2. **(B m-B2)** Two step-2 "verbatim" panic blocks carried thread IDs
   from an earlier diagnostic attempt. Re-derived from
   `/tmp/item3-red-step2.log` (TRY 1 / TRY 2 markers):
   - streaming fn: `85275019` → `85281497` (raw TRY 1 ID; the wrong ID
     appears nowhere in the raw log).
   - CLI fn: `85275777` → `85282092` (raw TRY 1 ID; same).
   The other two blocks (`85281015`, `85280948`) already matched the
   raw log's first-try IDs and were left as-is. The "verbatim, first
   try" label is now fully true against the raw log.
3. **(B m-B4 / A F2)** Step-6 summary table, step-3 row — was "same 4
   fns, minus streaming (11 assertions)". Re-derived from
   `/tmp/item3-red-step3.log`: `Summary 115 run: 112 passed, 3
   failed`; the 3 failing fns are `parser::test_parse_patch`,
   `parser::test_parse_patch_lenient`,
   `streaming_parser::tests::test_streaming_patch_parser_returns_errors`
   (streaming now panics at :855 — the DeleteFile assertion — while the
   CLI fn PASSED). Corrected row: "3 fns / 10 assertions — the CLI fn
   went green; the streaming fn is still red on the DeleteFile
   assertion". Matches the step-3 section body ("Ledger #1 and #4
   green (12 → 10 red)").
4. **(B m-B5)** Import refs off-by-one in two places. Source: HEAD's
   import block (`git show HEAD:…apply_patch_tests.rs` →
   `pretty_assertions` :13, `serde_json::json` :14) is identical to the
   pre-check (445-line) and step-5 file states (no imports were added
   before the T3.2 work).
   - Pre-check: `:14; … :15` → `:13; … :14`, with a note that the T3.2
     import additions shift them to :16/:17 in the round-1 file
     (current-tree values, re-verified).
   - Step 5: "already imported at :14" → ":13".
5. **(B m-B6)** `TurnContext::cwd()` → `TurnEnvironment::cwd()` in two
   places (pre-check read-only refs; step-5 scaffolding rationale).
   Source: `turn_context.rs:200` is `pub(crate) fn cwd(&self) ->
   &PathUri` inside `impl TurnEnvironment` (block starts :77);
   `TurnContext` owns only the DEPRECATED `cwd` field
   (`turn_context.rs:314`). The T3.2 setup takes `.cwd()` on
   `resolve_tool_environment`'s `&TurnEnvironment` return — the doc now
   names the correct type.
6. **(A F3)** Step-7 log labels. Verified against the /tmp files'
   contents:
   - `/tmp/item3-step7-module.log` (04:20): full 20-test module run,
     `19 passed, 1 timed out` (F1 test TMT both tries, 60.011/60.009,
     `Summary 120.040s`) — THE pre-fix hang.
   - `/tmp/item3-t32-debug.log` (04:25): single-test rerun, TMT both
     tries, `Summary 120.035s`, no DIAG output.
   - `/tmp/item3-t32-diag.log` (04:27): first DIAG iteration, compile
     failure (E0425/E0308).
   - `/tmp/item3-t32-diag2.log` (04:29): DIAG-instrumented single-test
     run, TMT both tries, `Summary 120.031s`, DIAG output ×2.
   - `/tmp/item3-t32-module.log` (04:34): post-fix 20/20 GREEN.
   The "First attempt" subsection now cites `step7-module.log` with
   that run's own verbatim TMT/Summary lines, and the DIAG narrative
   cites `t32-debug` / `t32-diag` / `t32-diag2` in true sequence. The
   20/20 block already cited `t32-module.log` (confirmed correct).
7. **(B m-B7 / A F4 — NO ACTION, recorded)** The rewritten parser.rs
   assertions keep `std assert_eq!` (pre-existing file-local pattern in
   parser.rs — accepted as-is, not "fixed"); T3.2's post-assert cwd
   cleanup is best-effort (`let _ =` remove_file/remove_dir) and could
   in principle leave a stray `grok/` tree under the temp cwd on a
   failure path. Verified at round-1 time: repo-wide
   `find . -name grok -not -path ./.git/*` → **no match — no stray
   `grok/` exists in the tree**.

### Round-1 gates (re-run in order from `codex-rs/` after the code fix)

**Gate 1 — `just test -p codex-apply-patch`** (log `/tmp/r1-gate-ap.log`):

```text
     Summary [   4.639s] 115 tests run: 115 passed, 0 skipped
EXIT=0
```

**Gate 2 — `just test -p codex-core apply_patch`** (log
`/tmp/r1-gate-core.log`; 2642s ≈ 44 min — machine load worse than the
round-1 run):

```text
     Summary [2642.225s] 115 tests run: 73 passed (1 slow, 3 flaky), 6 failed, 36 timed out, 4251 skipped
```

Binding check (0 deterministic failures):

```text
grep -E "thread .+ panicked" /tmp/r1-gate-core.log | grep -v 'lib.rs:388' | sort -u
# → (empty) — all 73 panics are core/tests/common/lib.rs:388:14
#   `timeout waiting for event: Elapsed(())`
```

**Zero deterministic failures.** Noisy-test identities this run:

- Final FAIL (6, all `suite::apply_patch_cli`, all lib.rs:388 class):
  `apply_patch_cli_move_overwrites_existing_destination`,
  `apply_patch_cli_move_without_content_change_has_no_turn_diff`,
  `apply_patch_cli_rejects_invalid_hunk_header`,
  `apply_patch_cli_updates_file_appends_trailing_newline`,
  `apply_patch_emits_turn_diff_event_with_unified_diff`,
  `apply_patch_shell_heredoc_normalizes_crlf_without_preserve_line_endings_feature`.
  Note: `apply_patch_cli_rejects_invalid_hunk_header` is the surviving
  :707 substring test; both its tries died in the harness event-wait
  (lib.rs:388), not on the assertion — its assertion path is exercised
  deterministically by the apply-patch crate's exact-stderr twin (115/115
  in gate 1) and it passed in the step-3 run.
- Final TMT (36): `apply_patch_cli`: aggregates_diff_across_multiple_tool_calls,
  aggregates_diff_preserves_success_after_failure, cli_multiple_chunks,
  cli_multiple_operations_integration, cli_preserves_distinct_updated_paths,
  cli_preserves_existing_hard_link_outside_workspace,
  cli_rejects_empty_patch, cli_rejects_path_traversal_outside_workspace,
  cli_reports_missing_context, cli_reports_missing_target_file,
  cli_verification_failure_has_no_side_effects,
  exec_command_heredoc_with_cd_emits_turn_diff,
  normalizes_crlf_without_preserve_line_endings_feature,
  preserves_crlf_with_preserve_line_endings_feature,
  shell_accepts_lenient_heredoc_wrapped_patch; `apply_patch_serialization`:
  custom_tool_call_creates_file, custom_tool_call_reports_failure_output,
  custom_tool_call_updates_existing_file; `approvals`:
  approval_matrix_covers_group::apply_patch,
  approving_apply_patch_for_session_skips_future_prompts_for_same_file;
  `code_mode`: code_mode_can_apply_patch_via_nested_tool; `hooks`:
  permission_request_hook_allows_apply_patch_with_write_alias,
  post_tool_use_records_additional_context_for_apply_patch,
  post_tool_use_records_apply_patch_context_with_edit_alias,
  pre_tool_use_blocks_apply_patch_before_execution,
  pre_tool_use_blocks_apply_patch_with_write_alias,
  pre_tool_use_rewrites_apply_patch_before_execution; `prompt_caching`:
  gpt_5_tools_without_apply_patch_append_apply_patch_instructions;
  `request_permissions`:
  denied_child_permissions_require_fresh_approval::{apply_patch_session, apply_patch_turn};
  `request_permissions_tool`:
  approved_folder_write_request_permissions_unblocks_later_apply_patch::{with_strict_auto_review, without_strict_auto_review};
  `shell_snapshot`: unified_exec_snapshot_still_intercepts_apply_patch;
  `tool_harness`: apply_patch_reports_parse_diagnostics,
  apply_patch_tool_executes_and_emits_patch_events; `unified_exec`:
  unified_exec_intercepts_apply_patch_exec_command.
- Flaky (passed on retry; 3): `apply_patch_change_context_disambiguates_target`,
  `apply_patch_cli_rejects_duplicate_resolved_paths`,
  `apply_patch_custom_tool_streaming_emits_updated_changes`.
- The 20 item-3-owned unit tests
  (`tools::handlers::apply_patch::tests::*`) all PASS within this run
  again (0 TMT/FAIL/FLAKY among them).

**Gate 3 — `just fix -p codex-apply-patch -p codex-core`** (log
`/tmp/r1-fix.log`):

```text
cargo clippy --fix --tests --allow-dirty {args}
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 52.78s
EXIT=0
```

Zero clippy changes to owned files; the known out-of-scope auto-fix
re-triggered again (pre-existing unused `body_json` import,
`openai_file_mcp.rs:47`). Restored via
`git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs`;
`git status`/`git diff --numstat` confirm the file back to its HEAD/seam
state and no other out-of-scope file touched.

**Gate 4 — `just fmt`** (log `/tmp/r1-fmt.log`): exit 0. Verified
`apply_patch_tests.rs:57` retained the `/*environment_id*/` comment
(91 cols, byte-identical to the pre-fmt line) — fmt did not strip or
reflow it. Per protocol, no tests re-run after fix/fmt.

### Round-1 final state

`git diff --numstat` across the 5 owned files — **unchanged from round 1**:

```text
8    6    codex-rs/apply-patch/src/parser.rs
5    6    codex-rs/apply-patch/src/streaming_parser.rs
1    1    codex-rs/apply-patch/tests/suite/tool.rs
265  60   codex-rs/core/src/tools/handlers/apply_patch.rs
164  15   codex-rs/core/src/tools/handlers/apply_patch_tests.rs
```

HEAD still `76111f29eb` on `feat/normalize-content-types-vllm`; tree
uncommitted; `git status` (tracked) = the 5 owned files + the 12
pre-existing non-owned files (11 codex-rs seam + AGENTS.md), nothing
else (r2 seat C F-C1 correction).
