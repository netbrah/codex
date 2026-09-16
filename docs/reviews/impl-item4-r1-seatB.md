# Item 4 — Integration (spec T4): Round-1 review, SEAT B (conventions / minimality / evidence)

Reviewer: SEAT B (conventions, minimality, evidence-authenticity lens). Independent of
SEAT A (spec/semantics/integration lens); no coordination.
Item: 4 of the apply_patch compatibility campaign (bead apex-ayl.52),
`feat/normalize-content-types-vllm`, working-tree HEAD `2a25e69a46`.
Artifacts under review:
- `codex-rs/core/tests/common/responses.rs` (+11/−0 vs HEAD)
- `codex-rs/core/tests/suite/apply_patch_cli.rs` (+138/−0 vs HEAD)
- `docs/reviews/impl-item4-tdd-evidence.md` (new; D1–D6 discrepancy notes audited)

## Scope and method

Review-only: no product-code edits, no `git add`/`commit`, no writes outside
`docs/reviews/`. Method:

1. Read breakdown Item 4 + §0 + §5, root AGENTS.md Rust/integration-test
   conventions, the evidence doc in full, and the new code in diff and final
   form, side-by-side with every sibling pattern cited.
2. Re-derived every evidence claim against the tree and the raw `/tmp` logs
   (which are present): wire capture re-parsed from
   `/tmp/item4-wire-capture2.log` (python JSON parse), solo-run captures
   diffed against `/tmp/item4-focused-t41|t42|t42b.log`, D2 binding grep
   re-run on `/tmp/item4-gate1.log`, D4 file state checked via `git diff
   HEAD`, D5 seam lines checked via `git diff HEAD` on the seam files,
   nextest group semantics verified against `codex-rs/.config/nextest.toml`.
3. Re-ran the gates myself from `codex-rs/` (patient; no PID kills):
   `just test -p codex-core apply_patch`, `just fix -p codex-core`,
   `just fmt`; scope-file numstat + `git status` before and after.
4. Enumerated the full in-scope test set with
   `cargo nextest list -p codex-core -- apply_patch` (118 tests) to verify
   the gate's count and the D6 note.

Execution baseline per breakdown §0: "At HEAD" = working-tree state; the
tree carries the uncommitted seam (AGENTS.md + 11 codex-rs files) which is
out of scope for this item.

## Mandate 1 — Conventions

### `ev_apply_patch_function_call` (responses.rs:1041-1046, doc :1037-1040)

- **Matches the breakdown's prescribed body.** Breakdown Item 4 step 2 gives
  the exact body; the implementation is identical except for one blank line
  before the final `ev_function_call` call — which is present in the
  sibling `ev_apply_patch_exec_command_call_via_heredoc`
  (responses.rs:1030-1035) and is exactly what the breakdown's R1
  resolution row 5 (L-M5) prescribes ("mirroring
  `ev_apply_patch_exec_command_call_via_heredoc`"). The evidence pre-check
  explicitly notes the mirrored blank line. Not a finding.
- **`pub` visibility consistent**: every `ev_*` sibling in the pattern
  family is `pub` (`ev_function_call` :933, `ev_custom_tool_call` :975,
  `ev_apply_patch_custom_tool_call` :1008, `ev_exec_command_call` :1020,
  `ev_exec_command_call_with_args` :1025,
  `ev_apply_patch_exec_command_call_via_heredoc` :1030);
  `apply_patch_cli.rs:8` imports the siblings by name. ✓
- **Doc-comment style**: siblings are mostly undocumented, but every doc'd
  helper in the family uses the `Convenience: SSE event for …` phrasing —
  verified at the exact lines the evidence cites (responses.rs:736
  `ev_completed`, :747 `ev_response_created`, :784 `ev_assistant_message`,
  :1005-1007 `ev_apply_patch_custom_tool_call`). The new helper's 4-line
  doc uses the same phrasing and mirrors its custom-tool counterpart. ✓
- **Error message**: `.expect("serialize apply_patch arguments")` is the
  exact message of the heredoc sibling (responses.rs:1032);
  `ev_exec_command_call_with_args` uses the analogous
  `expect("serialize exec command arguments")`. Style-consistent. ✓

### `apply_patch_cli.rs` — names, helpers, assertions

- **Test names / nextest group (D1 rationale verified).** The file has 49
  `#[tokio::test]` tests: 46 pre-existing, of which 26 use the
  `apply_patch_cli_` prefix and 20 do not (e.g.
  `apply_patch_preserves_crlf_with_preserve_line_endings_feature`,
  `intercepted_apply_patch_verification_uses_local_sandbox`). So it is NOT
  true that all existing names use the prefix — but D1 only claims it is
  the "dominant behavioral-test naming convention", which is defensible
  (26/46, and all the full-stack behavioral tests). The load-bearing part
  checks out exactly: `codex-rs/.config/nextest.toml` defines
  `[[profile.default.overrides]]` with
  `filter = 'package(codex-core) & kind(test) & test(apply_patch_cli)'`
  → `test-group = 'core_apply_patch_cli_integration'`
  (`[test-groups.core_apply_patch_cli_integration] max-threads = 1`).
  Nextest's `test(...)` selector is a substring match on the test name, so
  the initial `apply_patch_function_tool_*` names would NOT have been in
  the serialized group, and the renamed `apply_patch_cli_function_tool_*`
  names are. The rename was load-bearing for serialization. ✓
  One inaccuracy: the evidence quotes the group's comment as "Higher
  concurrency causes integration test timeouts under resource contention"
  — that comment belongs to `[test-groups.app_server_integration_local]`;
  the actual comment above the core_apply_patch override is about Windows
  runner process-startup stalls. Mechanism and group name are correct;
  only the quoted comment is misattributed (nit).
- **`vllm_apply_patch_harness`**: referenced exactly 3× (the three new
  tests) — not a one-use helper. It wraps the file's existing
  `apply_patch_harness_with` (apply_patch_cli.rs:91-98) with the one config
  delta, in the same shape as the inline `apply_patch_harness_with(|builder|
  …)` calls used by the file's tests (e.g. :445). ✓
- **Assertion style**: the file imports `pretty_assertions::assert_eq`
  (apply_patch_cli.rs:10) and uses both `assert_eq!` (whole-content
  comparisons, e.g. :467-472) and `assert!(cond, "message")` (e.g. :676
  `assert!(!saw_turn_diff, "pure rename should not emit a turn diff")`,
  :749 `assert!(out.contains("Failed to find expected lines in"))`). The
  new tests use the same mix: `assert_eq!` for the `["type"]`
  whole-object-value check and byte-exact file contents, and
  `assert!(description.contains(substring), "patch argument description
  missing: {substring}")` for the 7 drift substrings — the same loop shape
  and message style as the T2.1 drift guard
  (`apply_patch_spec_tests.rs:96-104`). No contradiction with the file's
  established pattern. ✓
- **`wait_for_event` vs `wait_for_event_with_timeout`**: the new tests use
  no explicit event wait at all — they follow the file's dominant
  behavioral pattern (`harness.submit(…).await?` → `read_file_text` →
  `assert_eq!`, exactly `apply_patch_cli_multiple_operations_integration`
  :442-478 and the CRLF helper :300-346). The repo preference for
  `wait_for_event` applies when a test waits on an event; where the file
  does wait (e.g. :666, :1007, :1027) it uses `wait_for_event`, never
  `wait_for_event_with_timeout`. No convention violated. ✓
- **Mount mechanism**: `mount_apply_patch_function_call`
  (apply_patch_cli.rs:248-259) mirrors `mount_apply_patch` (:230-246 in
  the final tree; :228-244 at HEAD) through the shared
  `apply_patch_responses` fn-pointer helper (:281-298,
  `fn(&str, &str) -> serde_json::Value` parameter), passing
  `ev_apply_patch_function_call` — zero changes to the shared helper or to
  `mount_apply_patch`/`mount_apply_patch_model_output` callers. It builds
  the same two-body sequence (`created`+call+`completed` →
  `assistant_message`+`completed`) via `mount_sse_sequence`, which returns
  `ResponseMock` (responses.rs:1437) — the new helper returns it (existing
  `mount_apply_patch` discards it). Breakdown step 3 explicitly pre-cleared
  this option ("an `apply_patch_responses` function-call variant … or a
  parallel `mount_sse_sequence` call"). Reusing `mount_apply_patch` was
  forbidden (custom_tool_call would be rejected on the renamed provider);
  not reused. ✓ Visibility: private `async fn`, correct — only
  `mount_apply_patch` is `pub` (imported by
  `apply_patch_serialization.rs:11`, verified).
- **`/*param_name*/` comments**: no opaque positional literals in the new
  code. `skip_if_no_network!(Ok(()))` matches the ~40 existing identical
  callsites in the file (:305, :409, :443, …) which carry no comment. ✓
- **format!/collapsible-if/SSE construction**: no `format!` added (hence no
  inline-args question); no `if` statements at all (no collapsible-if
  question); the SSE payloads are built exclusively via `ev_*` helpers
  (`ev_apply_patch_function_call` → `ev_function_call`;
  `ev_response_created`, `ev_completed`, `ev_assistant_message` inside
  `apply_patch_responses`) — no hand-rolled JSON in the new code. ✓

**Mandate 1 result: no findings.**

## Mandate 2 — Minimality

### (a) Numstat recount — verified

`git diff HEAD --numstat` (re-run at review start and again after my gate
runs; see Scope-file integrity):

```text
11   0  codex-rs/core/tests/common/responses.rs
138  0  codex-rs/core/tests/suite/apply_patch_cli.rs
```

Exactly +11/−0 and +138/−0, purely additive, matching the worker's
reported numstat and the evidence's "Final numstat" section.

**`ResponseMock` import check**: `git show
HEAD:codex-rs/core/tests/suite/apply_patch_cli.rs | grep ResponseMock` →
no hits (exit 1): the import did NOT exist at HEAD, so
`use core_test_support::responses::ResponseMock;` (apply_patch_cli.rs:55)
is a genuinely NEW line in the diff, not a move of a pre-existing line.
The evidence's "fmt moved the import to its sorted position" describes the
worker's pre-fmt placement being re-sorted within the added lines; the
final position is correctly sorted (after `assert_regex_match`, before
`ev_assistant_message` — ASCII `R` < `e`), consistent with the 0-deletion
numstat. Likewise `use …ev_apply_patch_function_call;` (:8) is new and
sorted between the heredoc sibling and `ev_exec_command_call`. No
pre-existing line was deleted anywhere. ✓

### (b) apply_patch_cli.rs — every added line accounted (138 = 2 + 13 + 123)

- :8 — import `ev_apply_patch_function_call` (1)
- :55 — import `ResponseMock` (1)
- :248-259 — `mount_apply_patch_function_call` incl. trailing blank (13)
- :2503-2625 — final hunk (123), which breaks down exactly as:
  - 1 blank after the last pre-existing test
  - 4-line section comment (":2504-2507")
  - 8-line `vllm_apply_patch_harness` (":2508-2515")
  - 1 blank + 44-line T4.1 `apply_patch_cli_function_tool_served_to_non_openai_provider`
  - 1 blank + 31-line T4.2 `apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file`
  - 1 blank + 32-line T4.2b `apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file`

1 + 4 + 8 + 1 + 44 + 1 + 31 + 1 + 32 = 123 ✓. **Nothing unaccounted**: the
only additions are the 2 imports, the mount helper, the harness helper,
and the 3 tests — exactly the breakdown's Item 4 scope ("2 test helpers +
3 integration tests"). No product code, no edits to pre-existing helpers
(`apply_patch_responses`, `mount_apply_patch`,
`mount_apply_patch_model_output` untouched — verified in the diff).

### (c) responses.rs — every added line accounted (+11)

4 doc lines + fn signature + `let args` + `let arguments` + 1 blank +
`ev_function_call` call + closing brace + 1 trailing blank = 11 ✓.
Inserted between `ev_apply_patch_exec_command_call_via_heredoc`
(:1030-1035) and `sse_failed` — next to the sibling pattern helpers per
breakdown step 2.

### (d) No temporary-capture leftovers — verified

`grep -n "eprintln\|DIAG\|print!\|dbg!\|T41-WIRE" responses.rs
apply_patch_cli.rs` → exactly 3 hits, all PRE-EXISTING and outside the
diff: responses.rs:1311 and :1340 (pre-existing eprintlns in unrelated
helpers), apply_patch_cli.rs:1078 (pre-existing Windows-skip message).
The diff adds zero lines at any of those sites. The D3 temporary
`eprintln!("T41-WIRE-CAPTURE …")` was indeed removed. ✓

### (e) No edits outside the two owned files — verified

`git status --short` at review start: 14 modified files = AGENTS.md + the
11 seam files (content_type_compat.rs, content_type_compat_tests.rs,
endpoint/responses.rs, handlers/mod.rs, spec_plan.rs, amazon_bedrock/mod.rs,
provider.rs, models.json, manager_tests.rs, model_info.rs,
model_info_tests.rs) + the 2 owned files. All untracked files are docs
(spec/breakdown drafts, spec/breakdown review reports, seam doc, vllm
research, `docs/superpowers/`, `engine-redesign-charter.md`, and the new
item-4 evidence) — none under `codex-rs/`. Nothing in `codex-rs/` besides
the two owned files is modified. ✓

**Mandate 2 result: no findings.**

## Mandate 3 — Evidence authenticity

### (a) T4.1 wire excerpt — verified against the raw capture

`/tmp/item4-wire-capture2.log` is present and contains the
`T41-WIRE-CAPTURE {…}` line. I parsed the JSON out of the log and checked
every claim:

- `"type": "function"`, `"name": "apply_patch"`, `"strict": false`,
  `"required": ["patch"]`, `"additionalProperties": false` ✓
  (function-tool form, not the freeform custom/grammar form)
- `properties` contains exactly `patch` (no `environment_id` —
  single-environment session, matching
  `create_apply_patch_function_tool(/*include_environment_id*/ false)`) ✓
- description length = **2198 chars** — matches the spec §3.1 measurement
  exactly (tool-level description also 126 chars, per spec) ✓
- All 7 §3.1 drift substrings present (re-derived the list from spec
  §3.1's drift-guard bullet — 7 substrings in v2 — and checked each):
  `first line is `*** Begin Patch``, `real newline characters`,
  `bare '+'`, `at most one hunk per patch`, `starts with '+'`,
  `multiple `@@` chunks`, `must change at least one line` ✓
- The captured description is **byte-identical** to the committed P1 const
  `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION`
  (`codex-rs/core/src/tools/handlers/apply_patch_spec.rs:42`, committed in
  item 2 `a4d5af1f62`; file clean vs HEAD). This is the strongest
  authenticity signal in the doc: the excerpt cannot be fabricated without
  matching the committed constant.
- The head/tail shown in the evidence's elided excerpt match the raw log
  verbatim.

### (b) Per-test first-run captures — verified

`/tmp/item4-focused-t41.log`, `-t42.log`, `-t42b.log` all present. Diffed
against the evidence's quoted lines: T4.1 `PASS [0.298s] (1/1)`, summary
`0.314s … 1 passed, 4368 skipped`; T4.2 `PASS [0.268s] (1/1)`,
`0.284s`; T4.2b `PASS [0.257s] (1/1)`, `0.275s`. All three match
exactly. ✓

### (c) D1 rename rationale — mechanism verified; one misquoted comment

`codex-rs/.config/nextest.toml`:
`[[profile.default.overrides]]` with
`filter = 'package(codex-core) & kind(test) & test(apply_patch_cli)'` →
`test-group = 'core_apply_patch_cli_integration'`, and
`[test-groups.core_apply_patch_cli_integration] max-threads = 1`. Nextest
`test(...)` is a substring match, so the pre-rename
`apply_patch_function_tool_*` names were outside the serialized group and
the post-rename names are inside it — the rename was load-bearing, as
D1 states. The group name, filter, and max-threads all check out.
Inaccuracy: D1's Gate-1 section quotes the group's comment as "Higher
concurrency causes integration test timeouts under resource contention";
that comment belongs to `[test-groups.app_server_integration_local]`. The
comment actually above the core_apply_patch override concerns Windows
runner process-startup stalls. Mechanism correct, comment misquoted (nit).

### (d) D2 gate-1 identities (79/79 lib.rs:388) — re-derived

Re-ran the binding grep on the raw `/tmp/item4-gate1.log`:

```text
$ grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u
(empty)
$ grep -E "thread .+ panicked" LOG | grep -oE "lib.rs:[0-9]+" | sort | uniq -c
  79 lib.rs:388
```

79/79, zero non-388 panics — exactly as recorded. The raw log's summary
line (`118 tests run: 73 passed (3 flaky), 8 failed, 37 timed out,
4251 skipped`, exit 100) is consistent with the evidence's
"0 deterministic failures, all non-passes in the load-noise class"
characterization. ✓

### (e) D4 openai_file_mcp.rs — verified

`git diff HEAD --stat -- codex-rs/core/tests/suite/openai_file_mcp.rs` →
empty: the file is clean vs HEAD now, and the evidence records the
`git checkout --` restore. (The compile-time unused-import warning at
:47 visible in the solo-run logs is the same known hazard — the import is
unused at HEAD too, which is why `just fix` keeps re-triggering; out of
scope for this item and correctly left untouched.) ✓

### (f) D5 (green on working tree, would fail on bare HEAD) — technically
accurate; classified ACCEPTABLE per breakdown §0

Verified each dependency:

- `codex-rs/model-provider/src/provider.rs:372`
  `apply_patch_function_tool: !self.info.is_openai(),` — present ONLY in
  the uncommitted +32-line seam diff (line 25 of the diff) ✓ uncommitted
- `codex-rs/core/src/tools/spec_plan.rs` — the capability branch
  (`if context.turn_context.provider.capabilities().apply_patch_function_tool`)
  and `registry.add(FunctionApplyPatchHandler::new(include_environment_id))`
  are uncommitted `+` diff lines; working-tree gate at :1257, capability
  chain :1259-1263, registration :1267 (evidence cites :1257-1270 /
  :1260-1264 / :1267 — tail and chain range off by 1-2 lines; nits)
- `codex-rs/core/src/tools/handlers/mod.rs` — `pub use
  apply_patch::FunctionApplyPatchHandler;` is an uncommitted `+` line ✓
- `FunctionApplyPatchHandler` itself IS committed (5 refs in
  `git show HEAD:…/handlers/apply_patch.rs`) ✓ as D5 states

So on bare HEAD: no capability → no function-handler registration → T4.1
would see the freeform tool (its `"type": "function"` assertion fails) and
T4.2/T4.2b would hit the registry kind-mismatch. The bare-HEAD-fails claim
is technically correct. This is exactly the execution baseline declared in
breakdown §0 ("At HEAD" = working-tree state; the seam is uncommitted
branch state, committed together with the item by the coordinator), so it
is **acceptable per §0, not a finding**.

One inaccuracy inside D5: it lists the "model-info default
(`model_info.rs: apply_patch_tool_type: Some(Freeform)`) " as a seam line
the tests depend on. That line IS uncommitted (verified in the
model_info.rs diff), but it is the fallback for unknown slugs — the
harness model `gpt-5.5` resolves from `models.json`, where
`apply_patch_tool_type: "freeform"` is COMMITTED (verified in both HEAD
and working-tree models.json). The tests therefore do not actually depend
on that seam line; it is belt-and-braces. The two load-bearing seam lines
(provider.rs:372, spec_plan gate) are correctly identified. (nit)

### (g) E0308 rationale — re-derived, correct

`ev_function_call(call_id: &str, name: &str, arguments: &str) -> Value`
(responses.rs:933) takes `arguments: &str`;
`serde_json::to_string(…)` returns an owned `String`, which does not
coerce to `&str` at argument position → the one-liner fails with E0308.
Both cited siblings (`ev_exec_command_call_with_args` :1025-1028,
  `ev_apply_patch_exec_command_call_via_heredoc` :1030-1035) use the
  local-bind shape the new helper copies. ✓

### (h) Final numstat section — matches my recount

Evidence's "Final numstat" (11/0, 138/0) = my independent recount. ✓

### Extra: D6 test-count note — 118 verified; the note's arithmetic is loose

The 118 in-scope count is REAL and current: `cargo nextest list
-p codex-core -- apply_patch` enumerates exactly 118 tests (40 lib + 78
integration), and the raw gate log's summary says 118 run / 4251 skipped
(total 4369, self-consistent: 118 + 4251 = 4369 = 1 + 4368 as in the solo
runs). All 3 new tests are in the list; the item-3 tests
(`function_apply_patch_*`, `with_environment_id_line_*`) and item-2's
T2.1/T2.2 all appear in the raw log as PASS in run 1 (verified lines
1/118 … 32/118), as D6 claims.

However, the note's implied arithmetic does not close: it frames the
118-vs-112 delta as "Items 1-3's own added tests … plus the 3 new item-4
tests", i.e. Items 1-3 contributing 3. Verified against git:

- Item-3's commit (`939a6dc6f4`) alone introduces **7** in-scope
  codex-core tests (`git log -S` on each: all 7 first appear in
  `939a6dc6f4`; all 7 are in the current 118; none was removed by items
  3/4 — item-3's commit deletes no test functions).
- Item-2's commit (`a4d5af1f62`) commits all 4
  `create_apply_patch_function_tool_*` tests, but item-2's own evidence
  shows its filter scope grew only 110 → 112 during item-2 ("baseline
  110 → RED 111 → final 112", "all four pre-existing unit tests PASS"),
  i.e. 2 of those 4 (`matches_expected_spec`,
  `includes_environment_id_when_requested`) already existed in the
  working tree as part of the pre-campaign seam (breakdown §0: the seam
  includes "the spec equality test").
- Item-3's own evidence records its baseline as 112 and its step-5 full
  run as "114 tests = 112 baseline + 2 new" (only the two T3.1 tests had
  landed by that run; the other 5 were added later in the item).

Reconciling every measured value (110 → 112 → 114 → 118; totals
4363 → 4365 → 4369 with a constant 4251 skipped) requires that the
historical 112 baseline included ~2-4 in-scope working-tree (seam-era)
tests that no longer exist in the codex-core binaries and that no commit
removed. The gate's own claims (118 executed, 0 deterministic failures,
binding-clean) are unaffected and verified; the note's explanatory
arithmetic is what's off.

### Pre-check cite drift (nits, batched)

- `use serde_json::json;` is at apply_patch_cli.rs:78 (evidence: :73).
- Harness default model line is test_codex.rs:857 (evidence: :858).
- `mount_sse_sequence` is at responses.rs:1437, doc at :1434 (evidence:
  :1426, which is an unrelated line inside another function).
- D5 pre-check names the T3.2 test
  `function_apply_patch_applies_f1_shaped_raw_add_file`; the committed
  name ends in `_…_patch`.
- `apply_patch_harness_with` is at :91-98 (evidence: :93-100).

All within normal citation drift; none changes a substantive claim.

**Mandate 3 result: no Blocking/Major. One minor (D6 arithmetic) + a batch
of nits (misquoted nextest comment, D5 model-info over-attribution,
cite drift).**

## Gates (re-run by SEAT B)

Run from `codex-rs/`, patient (no PID kills); machine load during the
window: loadavg 5-13 (the documented 15-35 saturation class; sibling
workers active on the shared host).

### Gate 1: `just test -p codex-core apply_patch`

Log: `/tmp/item4-gate1-seatB.log`.

```text
     Summary [5259.370s] 118 tests run: 55 passed (4 flaky), 6 failed, 57 timed out, 4251 skipped
error: recipe `test` failed on line 88 with exit code 100
```

- **0 DETERMINISTIC failures.** Binding check:

  ```text
  $ grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u
  (empty)
  $ grep -E "thread .+ panicked" LOG | grep -oE "lib.rs:[0-9]+" | sort | uniq -c
    116 lib.rs:388
  ```

  116/116 panic lines are the known load-noise class
  (`timeout waiting for event: Elapsed(())` at
  `core/tests/common/lib.rs:388` + nextest TMT).
- 118 tests in scope — matches my independent
  `cargo nextest list -p codex-core -- apply_patch` enumeration exactly.
- All 40 lib unit tests in scope PASS (zero lib non-passes), including
  all 11 item-2/item-3 campaign tests (`create_apply_patch_function_tool_*`
  19-23/118, `function_apply_patch_*` / `with_environment_id_line_*`
  1-29/118).
- All 3 NEW tests TMT'd in the saturated full run (76-78/118, both tries,
  lib.rs:388 class) — the same class that hit ~60 other tests in the same
  window (full non-pass identity list: 63 unique identities, all
  `suite::` integration tests; recorded in the gate log). Pre-existing
  `apply_patch_cli_*` neighbors TMT'd identically in-window (e.g.
  `apply_patch_cli_add_overwrites_existing_file`,
  `apply_patch_cli_rejects_invalid_hunk_header`), confirming
  machine saturation rather than diff impact.
- **New tests GREEN in my runs (solo, serialized group):**

  ```text
  T4.1  PASS [ 0.645s] (1/1) apply_patch_cli_function_tool_served_to_non_openai_provider
  T4.2  TRY 2 PASS [ 24.518s] (1/1) …applies_f1_shaped_raw_add_file  (flaky: TRY 1 TMT, 388 class)
  T4.2b PASS [ 0.705s] (1/1) …raw_add_file_overwrites_existing_file
  ```

  (Logs: `/tmp/item4-seatb-solo-t41.log`,
  `/tmp/item4-seatb-solo-t42-r1.log` [after 1 TMT retry],
  `/tmp/item4-seatb-solo-t42b-r1.log`.) This is the same per-test 1/1
  protocol the worker's D2 recorded; each solo summary is
  `1 test run: 1 passed, 4368 skipped`.

### Gate 2: `just fix -p codex-core`

Log: `/tmp/item4-seatb-gate2.log`.

```text
cargo clippy --fix --tests --allow-dirty {args}
    Checking codex-core v0.0.0 (…)
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 47.21s
GATE2-EXIT=0
```

**EXIT 0.** No fixes applied to either owned file. **The documented known
hazard re-triggered exactly as predicted** (out-of-scope
`openai_file_mcp.rs` unused-import auto-fix, −1 line). Restored via
`git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs` and
verified clean vs HEAD (empty diff). Matches D4.

### Gate 3: `just fmt`

Log: `/tmp/item4-seatb-gate3.log` (no output on success). **EXIT 0.**
Both owned files byte-identical post-fmt (md5 before/after equal) — the
worker's "fmt ran last" state is stable and idempotent.

## Scope-file integrity

| check | before gates | after gates |
|---|---|---|
| responses.rs numstat | 11/0 | 11/0 |
| apply_patch_cli.rs numstat | 138/0 | 138/0 |
| `git status` (tracked) | AGENTS.md + 11 seam files + 2 owned | identical |
| openai_file_mcp.rs vs HEAD | clean | clean (after D4-class restore) |
| untracked files | docs-only (no codex-rs) | identical |

No drift from any gate run; the two owned files remain purely additive.

## Findings

1. **minor — D6 test-count note arithmetic does not close (evidence doc).**
   `docs/reviews/impl-item4-tdd-evidence.md` §"Discrepancy notes" D6: frames
   the 118-vs-112 delta as "Items 1-3's own added tests … plus the 3 new
   item-4 tests". Verified facts: item-3's commit alone introduces 7
   in-scope codex-core tests (`git log -S` on each new test: all first
   appear in `939a6dc6f4`; all 7 present in the current 118; none removed
   since — item-3 deletes no test functions); item-2's scope grew only
   110→112 during item-2 (its own evidence), i.e. 2 of its 4 committed
   `create_apply_patch_function_tool_*` tests were seam-era working-tree
   tests already present at item-2's baseline; item-3's own step-5 run was
   "114 = 112 + 2". No combination of these measured values closes to 118
   without assuming ~2-4 in-scope tests that existed in the working tree
   during the item-2/3 window no longer exist and were removed by no
   commit. The gate's own claims are unaffected: 118 is verified real
   (independent `cargo nextest list` = 118; raw log summary = 118 run /
   4251 skipped = 4369 total, consistent with the solo runs' 1 + 4368).
   Justification: a review artifact that will be committed with the item
   should have its count note close on verifiable arithmetic; as written
   it would mislead a future auditor about how many tests items 1-3
   contributed. Not Blocking/Major because the count it explains (118) is
   correct and every test-level claim in the note (T2.1/T2.2 PASS, item-3
   tests PASS) was verified true in the raw log.

2. **minor — pre-check "line refs re-derived" claim overstated for two
   breakdown cites.** The evidence pre-check asserts "Breakdown Item 4
   line refs were verified by reading the current tree before any edit",
   but `apply_patch_responses` is cited as :257-271 (actual at HEAD:
   :266-283; final tree: :281-298) and `mount_apply_patch_model_output` as
   :260-276 (actual at HEAD: :246-264; final tree: :261-279) — both match
   the (wrong) numbers in breakdown v2, indicating transcription from the
   breakdown rather than re-derivation. Same drift appears in the
   breakdown's own §0/R1-verified cites, so the root cause predates
   item 4. Justification: the campaign's explicit anti-drift process
   ("every item re-verifies its refs at HEAD first") was not fully
   honored for these two cites; no implementation impact (the mirror was
   implemented correctly against the real code), so minor.

3. **nit — nextest group comment misquoted (D1 / Gate-1 section).** The
   evidence attributes the comment "Higher concurrency causes integration
   test timeouts under resource contention" to the
   `core_apply_patch_cli_integration` override; that comment belongs to
   `[test-groups.app_server_integration_local]`. The actual comment above
   the core_apply_patch override concerns Windows runner process-startup
   stalls. The group name, filter, and max-threads (the load-bearing
   parts) are correct.

4. **nit — D5 over-attributes a dependency.** D5 lists "model-info
   default (`model_info.rs: apply_patch_tool_type: Some(Freeform)`) " as a
   seam line the tests depend on. The line is uncommitted (verified), but
   it is the fallback for unknown slugs; the harness model `gpt-5.5`
   resolves from committed `models.json` (`apply_patch_tool_type:
   "freeform"` at HEAD and in the working tree). The load-bearing seam
   lines are correctly identified (provider.rs:372, spec_plan gate).

5. **nit — line-range drift in the evidence's own cites (batched).**
   responses.rs: new helper cited :1039-1046 (doc :1039-1042), actual
   :1041-1046 (doc :1037-1040); `sse_failed` cited :1047, actual :1048;
   `mount_sse_sequence` cited :1426, actual fn :1437 (doc :1434);
   `ev_apply_patch_custom_tool_call` cited :1005-1018, actual doc :1005-
   1007 + fn :1008-1021. apply_patch_cli.rs: `mount_apply_patch` cited
   :228-239, actual :228-244 (HEAD) / :230-246 (final); mount helper
   cited :248-258, actual :248-259; `apply_patch_harness_with` cited
   :93-100, actual :91-98; `use serde_json::json;` cited :73, actual :78;
   test_codex.rs gpt-5.5 line cited :858, actual :857; spec_plan gate
   cited :1257-1270, actual block :1257-1269. All are off-by-small
   except the two tracked in finding 2; none changes a substantive claim.

**Tally: 0 Blocking, 0 Major, 2 minor, 3 nit.**

## Verdict

The new code is convention-clean against both the root AGENTS.md
integration-test guidance and the files' established local patterns:
helper shape/visibility/docs/error-message match the siblings exactly,
the mount helper mirrors `mount_apply_patch` through the shared
fn-pointer helper with the `ResponseMock` return, the 3 tests follow the
file's dominant behavioral-test pattern (skip macro, harness, mount,
submit, `read_file_text`, `pretty_assertions::assert_eq` /
`assert!` with message), and the rename into `apply_patch_cli_*` is
load-bearing (verified nextest group membership). Minimality is exact:
+11/−0 and +138/−0 with every added line accounted (2 imports + mount
helper + section comment + harness + 3 tests; nothing else), no
leftover capture code, no out-of-scope edits. Evidence authenticity held
under deep audit: the wire capture re-parsed from the raw log is
byte-identical to the committed P1 constant (2198 chars, all 7 §3.1
substrings, correct function-tool shape), the solo-run captures match
the raw logs line-for-line, D2's 79/79 lib.rs:388 binding was
re-derived, D1's nextest mechanism verified, D4's restore verified, D5's
seam dependencies verified and classified acceptable per breakdown §0,
the E0308 rationale re-derived, and the final numstat matches. My own
gate re-runs: 0 deterministic failures (binding-clean), all 3 new tests
green in my runs (solo 1/1), `just fix` exit 0 with the documented
hazard re-triggered and restored, `just fmt` exit 0 with byte-identical
owned files.

The findings are documentation-quality issues in the evidence artifact
(D6 count-note arithmetic, an overstated re-verification claim for two
breakdown-derived line cites, a misquoted nextest comment, a
D5 over-attribution, and batched cite drift) — none affects the code,
the gate claims, or the regression-lock purpose of the 3 tests.

**SEAT B round 1: APPROVED — 0 Blocking, 0 Major, 2 minor, 3 nit**
