# apex-xt2.14 — FunctionApplyPatchHandler: accept raw-text custom_tool_call payloads

Status: FINAL v1.4 (spec rounds 1-4 closed 0B 0M; breakdown round 1 closed
0B 0M; ready for stage-5 TDD execution)
Item: apex-xt2.14 (P1 BUG)
Date: 2026-09-22
Author: coordinator (RCA first-hand; see §3)
Related: docs/apply-patch-lax-retry-spec.md (seam design, §2.4 call-site map),
docs/responses-compat-seam.md

## 1. Problem statement

On any non-OpenAI provider (the entire on-prem fleet: vLLM-served glm/qwen via
LiteLLM, and any custom-named provider such as the "test" provider used by
upstream test suites), the model's (or test mock's) raw-text `custom_tool_call`
for `apply_patch` is rejected by the fork-registered `FunctionApplyPatchHandler`
with `FunctionCallError::Fatal("tool apply_patch invoked with incompatible
payload")`.

Consequences, by build profile:

- **Debug builds (all test runs):** the turn task panics and the codex process
  hangs forever (never exits, no error surfaced). This is the apex-xt2.14
  failure of upstream test
  `suite::apply_patch::shutdown_flushes_completed_turn_and_file_diff`
  (repro 2/2 on SCS x86_64 and 2/2 on Mac aarch64; TMT @60s).
- **Release builds (live codex-combined):** the tool call is silently dropped;
  the turn "completes" without the patch being applied. Silent data-loss class:
  the user believes their edit landed.

Upstream never hits this: upstream registers `ApplyPatchHandler` (accepts
`ToolPayload::Custom`) for all providers. The fork registers
`FunctionApplyPatchHandler` for non-OpenAI providers (seam commit
`799ec98053`, capability `apply_patch_function_tool = !is_openai()`), and that
handler only accepts `ToolPayload::Function`.

## 2. Why the upstream test trips it

`suite::apply_patch::shutdown_flushes_completed_turn_and_file_diff`
(codex-rs/exec/tests/suite/apply_patch.rs:151, added upstream in
88f1cd9664 #34831) runs codex-exec against a wiremock server with:

- `model_provider="test"` + a custom `model_providers.test` entry
  (name-anchored, non-OpenAI) — i.e. the fork's
  `apply_patch_function_tool` capability is ACTIVE;
- request 0 SSE = `ev_apply_patch_custom_tool_call` (raw freeform patch text)
  + `ev_completed`;
- expects the patch to be applied (flushed.md written) and analytics to flush
  on shutdown.

Upstream registers `ApplyPatchHandler` (matches `ToolPayload::Custom`) for ALL
providers, so the test passes there. The fork's capability gate swaps in
`FunctionApplyPatchHandler` for the "test" provider -> mismatch -> defect.

## 3. Evidence (all first-hand, 2026-09-22)

1. Mock A/B triad (harness /tmp/xt214/, HEAD binary):
   | run | provider | model shape | result |
   |---|---|---|---|
   | A | "test" (non-OpenAI -> FunctionApplyPatchHandler) | custom_tool_call raw text | HANG (rc=124, flushed.md absent, 1 req) |
   | B | built-in "openai" (-> ApplyPatchHandler) | custom_tool_call raw text | PASS (file written, 2 reqs) |
   | C | "test" (-> FunctionApplyPatchHandler) | function_call JSON {patch} | PASS (file written, 2 reqs) |
2. Trace-logged hang repro (RUST_LOG trace, HEAD binary, port 65430):
   - dispatch resolves in 1ms: `codex.tool_result ... success=false
     output=tool apply_patch invoked with incompatible payload`
   - then: `thread 'tokio-rt-worker' panicked at core/src/util.rs:95:9:
     in-flight tool future failed during drain: Fatal error: tool apply_patch
     invoked with incompatible payload`
   - process then parks forever (watchdog SIGKILL required). Log:
     /tmp/xt214/trace_head.log
3. Culprit attribution by commit inspection: `799ec98053` (ayl.52 seam
   baseline) is the commit that introduced BOTH the `apply_patch_function_tool`
   capability (`!is_openai()`) and the `FunctionApplyPatchHandler`
   registration in spec_plan.rs (verified by diff inspection of the commit:
   pre-gate code was a plain `registry.add(ApplyPatchHandler::new(...))`).
   Its parent predates the function handler entirely, where a Custom payload
   is handled by ApplyPatchHandler and the defect is impossible.
4. Probe correction (recorded 2026-09-22, breakdown review seat A, verified
   by shasum): the first "5789dd8500 probe" (port 65440, /tmp/xt214/work_test_1790057808)
   did NOT test 5789dd8500 — the worktree build omitted CARGO_TARGET_DIR, so
   run_ab2.sh's hardcoded EXE resolved to the main-tree HEAD binary (the
   shared target binary's shasum/mtime were verified untouched). That run was
   a second HEAD repro, not a bisect data point; its log lines are valid HEAD
   evidence only. CORRECTED PROBE (2026-09-22 07:29 UTC): worktree build with
   explicit CARGO_TARGET_DIR=/tmp/xt214-bisect/target5789; binary sha256
   022a856187725a95e341d2ca0a1ec5b01481bf2eca78ab5a71d1a57ee59fb264 recorded
   before the run; hardened harness (per-run reqdir, orphan-port guard,
   pkill teardown, wire-shape assert). Result at 5789dd8500: rc=124 hang,
   flushed.md absent, 1 mock request (client stalled after the first
   response), rollout tail ends at the ingested custom_tool_call, and the log
   (/tmp/xt214/out_probe5789.log) shows the identical mechanism and panic
   site as HEAD (`util.rs:95:9` drain panic). The defect reproduces at the
   5789dd8500 baseline, consistent with culprit 799ec98053 (its ancestor).
   Full record: /tmp/xt214/bisect_log.md. Worktree removed after the probe.
   Attribution does not depend on the probe: the mechanism is proven at HEAD
   (item 2) and the minimal culprit commit is established by diff inspection
   (item 3).

## 4. Root cause chain

1. `spec_plan.rs` (add_core_utility_tools, ~:1224): capability
   `apply_patch_function_tool` (= `!is_openai()` per model-provider-info)
   registers `FunctionApplyPatchHandler` instead of `ApplyPatchHandler` for
   every non-OpenAI provider.
2. `FunctionApplyPatchHandler::matches_kind` accepts ONLY
   `ToolPayload::Function`; a raw `custom_tool_call` arrives as
   `ToolPayload::Custom { input }`.
3. `ToolRegistry::dispatch_any_with_terminal_outcome` (~:579) returns
   `FunctionCallError::Fatal("tool apply_patch invoked with incompatible
   payload")`.
4. `ToolCallRuntime::handle_tool_call` maps the inner
   `FunctionCallError::Fatal` to the in-flight future's `Err(CodexErr::Fatal)`
   (Display: "Fatal error: tool apply_patch invoked with incompatible
   payload"); the future in `in_flight` (FuturesOrdered) resolves Err.
5. `drain_in_flight` (session/turn.rs:~2377) calls `error_or_panic` on Err.
   `error_or_panic` (core/src/util.rs:95) PANICS under `cfg!(debug_assertions)`.
6. The panic unwinds the turn task spawned in `Session::start_task`
   (tasks/mod.rs:~359) BEFORE `sess.on_task_finished(...)` and BEFORE
   `done_clone.notify_waiters()`. No `TurnComplete` event, no `done`
   notification, no rollout flush. The exec client (and anything awaiting
   turn completion / shutdown) blocks forever. All tokio workers parked,
   CPU 0. This is the observed "hang".
7. Release builds: `error_or_panic` only logs; the turn completes with the
   patch silently unapplied (no tool output recorded for the call).

Note: the hang mechanism (steps 5-6) is upstream code with an intentional
"fail loud in debug" policy; the fork-introduced element is the ONLY trigger
on this path (steps 1-3). Upstream can never reach a Fatal here because its
only apply_patch handler accepts the custom shape.

Scope boundary (r1 review A-m2): this fix closes the Fatal class for
handlers registered via the `apply_patch_function_tool` capability
(non-openai-named providers). The mirror gap is PRE-EXISTING and OUT OF
SCOPE: a provider NAMED "openai" (e.g. an on-prem vLLM/LiteLLM deployment
that keeps the `openai` name) still registers the Custom-only
`ApplyPatchHandler`, where a function-shaped apply_patch call still Fatals
(debug hang / release silent drop); Bedrock hardcodes the capability off
(model-provider/src/amazon_bedrock/mod.rs:219). Same defect class, opposite
shape — recorded as a follow-up consideration, revisit if observed in the
wire.

## 5. Fix design

Single PRODUCTION file: `codex-rs/core/src/tools/handlers/apply_patch.rs`
(test-only additions in the sibling `apply_patch_tests.rs`; production/test
boundary explicit for pathspec-strict staging).
Make `FunctionApplyPatchHandler` payload-agnostic so whatever wire shape a
deployment's model emits is handled by the registered handler.

5.1 `matches_kind`: accept both payload kinds
    `ToolPayload::Function { .. } | ToolPayload::Custom { .. }`.
    (`ApplyPatchHandler` is unchanged and remains Custom-only.)

5.2 `handle_call`: extract the patch text from either payload, then run the
    EXISTING shared path `run_apply_patch_text(...)`:
    - `ToolPayload::Function { arguments }`: unchanged (JSON parse, `patch`
      string, teachable errors for missing/non-string patch, then
      `with_environment_id_line` when an `environment_id` argument is
      present).
    - `ToolPayload::Custom { input }`: use `input` verbatim as the patch
      text. (The freeform `*** Environment ID:` line, if present, is already
      handled inside the shared parse path — no injection needed.)

    Content policy (decided, r1 review m3): the Custom `input` is used
    VERBATIM ONLY. Considered and rejected: a narrow JSON sniff (parse
    `input` as a JSON object with a string `patch` field and use that) — no
    evidence of that wire shape in the fleet (observed non-conforming shapes
    are raw freeform text: triad run A + the upstream test), and a wrong
    guess degrades to a recoverable `RespondToModel` verification error,
    never a Fatal or silent drop. Revisit if a JSON-in-custom shape is
    observed in the wire. The §6 extractor test pins the verbatim behavior.

    Divergence pre-note (r3 B3-m1): the restructured guard's
    `return Err(FunctionCallError::RespondToModel(..))` lines blame to
    upstream e9702411ab (#17027), but they exist ONLY inside the fork-only
    `FunctionApplyPatchHandler::handle_call` (no upstream counterpart — the
    #17027-era guard lived in `ApplyPatchHandler::handle_call`; in the
    current tree the surviving e9702411ab lines are
    L46/L261/L712/L530/L532/L533 (L531 is fork-owned) and this plan touches
    only L530/L532/L533); the change creates no
    upstream-derived hunk.

5.3 `create_diff_consumer`: return
    `Some(Box::<ApplyPatchArgumentDiffConsumer>::default())` (same as
    ApplyPatchHandler; the consumer is payload-agnostic and is only used for
    custom_tool_call streams per the turn loop, so function-shaped calls are
    unaffected).

5.4 Hook payloads: extend the FORK-OWNED `function_apply_patch_patch_text`
    (apply_patch.rs:600, blame apex-ayl.52) into the exhaustive extractor and
    rename it `apply_patch_handler_patch_text` (fork-owned,
    FunctionApplyPatchHandler only), mapping:
    - Function -> JSON `patch` string (as today)
    - Custom -> `input` verbatim
    - ToolSearch -> None (exhaustiveness arm; unreachable — matches_kind
      gates the registry dispatch)
    `apply_patch_payload_command` (apply_patch.rs:284, upstream-derived,
    blame #18391, byte-identical at upstream 205f3671e1) stays
    BYTE-IDENTICAL: it remains Custom-only and is used only by
    `ApplyPatchHandler` (r2 B-M1: generalizing it would change
    ApplyPatchHandler's hook methods for Function payloads — unreachable in
    production but a real upstream behavior delta = merge debt; the
    fork-owned extractor achieves the identical outcome with zero
    upstream-derived hunks).
    The extended extractor is used by `FunctionApplyPatchHandler::
    pre_tool_use_payload` and `::post_tool_use_payload`. The renamed
    extractor's doc comment (currently "Extracts the patch text from a
    function-form apply_patch payload for hook reporting",
    apply_patch.rs:598-599) must be updated to reflect both arms. Separately,
    `with_updated_hook_input` gains a Custom arm rewriting
    `ToolPayload::Custom { input: patch }` (patch derived from
    `updated_hook_command(&updated_input)`, mirroring ApplyPatchHandler).
    Hook wire shape stays identical for both payload kinds
    (`{"command": <patch text>}`).

5.5 Doc comments on the handler updated to state it accepts both payload
    kinds and why (wire-shape tolerance for the compat seam).

Out of scope / rejected alternatives:

- A1 (rejected): change upstream `drain_in_flight` / `error_or_panic` so a
  Fatal never panics. Upstream's debug-loud design is intentional; the fork
  must not carry an upstream-behavior change as merge debt. This fix removes
  the only fork-introduced trigger on the path. Revisit only if a second
  fork-introduced Fatal trigger appears.
- A2 (rejected): register a combined/both handler set. Duplicates handler
  state, complicates the spec plan gate, and 5.1-5.4 are strictly smaller.
- A3 (rejected): relax `matches_kind` only (without 5.2). Would still return
  the payload-guard error from `handle_call` for Custom — a recoverable
  `RespondToModel` (no hang, no silent drop) — but the patch is never
  applied. Not a fix.
- A4 (rejected): modify the upstream test to use a Function-shaped call.
  The test encodes the desired product behavior (custom_tool_call must work
  for any provider); weakening it is forbidden and would mask the live
  release-mode silent-drop.

Ratchet compliance: no provider/model gating added; no capability change;
the change is handler-internal payload-shape tolerance, model-agnostic, and
benefits every model served by a `FunctionApplyPatchHandler`-registered
provider (see the §4 scope boundary for the pre-existing mirror gap).

## 6. Test plan (TDD, one item at a time)

Item 1 — RED: core-level tests in `core/src/tools/handlers/apply_patch_tests.rs`.
  Session shape is load-bearing (r1 M1, both seats): the file-write test MUST
  use the F1 harness pattern — `make_session_and_context_with_auth_and_config_and_rx`
  with `AskForApproval::Never` + `PermissionProfile::Disabled` and
  `invocation_from_session` (precedent: the sibling test
  `function_apply_patch_applies_f1_shaped_raw_add_file_patch`, which documents
  that the default test session is read-only + approval-on-request and would
  stall a write on an unanswered approval prompt — the exact symptom of the
  bug under test). `invocation_for_payload` (default session) is only for
  no-write tests (hook payloads).
  - `function_apply_patch_handler_accepts_custom_payload` (F1 session shape):
    FunctionApplyPatchHandler + `ToolPayload::Custom { input: sample patch }`
    -> handle_call succeeds, target file written with expected content
    (mirror of triad run A at unit level). RED today: the payload guard
    returns `Err(FunctionCallError::RespondToModel("apply_patch function
    handler received unsupported payload"))` — the registry-level
    `Fatal("...invoked with incompatible payload")` only fires through
    `dispatch_any_with_terminal_outcome` and is pinned by the matches_kind
    test below + the exec-suite repro.
  - `function_apply_patch_matches_kind_accepts_function_and_custom`:
    matches_kind true for Function, Custom, false for ToolSearch.
    Expected RED today (Custom arm).
  - `function_apply_patch_pre_and_post_hook_payloads_cover_both_payload_kinds`:
    pre_tool_use_payload / post_tool_use_payload produce
    `{"command": <patch>}` for BOTH payload kinds. Expected RED today (Custom
    returns None). (No-write: `invocation_for_payload` fine.)
  - `function_apply_patch_with_updated_hook_input_rewrites_custom_payload`:
    with_updated_hook_input rewrites a Custom payload to the hook-updated
    patch text (mirror of the existing Function arm). RED today: Ok with the
    Custom input UNCHANGED (the `payload => payload` fallthrough,
    apply_patch.rs:637 — a silent no-op).
  - `function_apply_patch_handler_diff_consumer_is_some`:
    `FunctionApplyPatchHandler::default().create_diff_consumer().is_some()`
    (pins the §5.3 flip). RED today: `create_diff_consumer()` returns None
    (apply_patch.rs:616-618).
  - Update AND rename BOTH extractor tests for the §5.4 rename
    (`function_apply_patch_patch_text` -> `apply_patch_handler_patch_text`):
    - `function_apply_patch_patch_text_extracts_patch_argument` ->
      `apply_patch_handler_patch_text_extracts_patch_argument` (Function arm
      unchanged);
    - `function_apply_patch_patch_text_rejects_non_function_payloads` ->
      `apply_patch_handler_patch_text_extracts_custom_input_verbatim` (the
      old name becomes a lie after the fix), asserting the verbatim-only
      policy of §5.2 (Custom -> Some(input)).
    No call-site changes for `apply_patch_payload_command` (unchanged).
  - (r1 follow-up, SEAT B F2) `apply_patch_handler_patch_text_extracts_custom_input_json_shaped_verbatim`:
    Custom input is exactly `{"patch":"*** Begin Patch\n*** End Patch\n"}`
    (raw string literal; the JSON-escaped `\n` stays literal) — the precise
    wire shape the §5.2-rejected "JSON sniff" design would parse. Asserts the
    extractor returns the raw JSON string verbatim (Some(input)), never the
    decoded patch: RED today (None), GREEN only under true verbatim-only.
    Added per T1 review round 1 (verdict
    /tmp/xt214-item-t1-seat-B-r1-verdict.md).

Item 2 — GREEN: implement §5.1-5.5; item 1 tests pass.

Item 3 — regression anchors (must stay green):
  - existing `function_apply_patch_rejects_missing_patch_argument_with_teachable_error`,
    `function_apply_patch_rejects_non_string_patch_argument_with_teachable_error`
    (Function shape unchanged);
  - existing diff-consumer tests (ApplyPatchArgumentDiffConsumer) unchanged;
  - upstream exec-suite test `shutdown_flushes_completed_turn_and_file_diff`
    (the apex-xt2.14 repro) passes — run explicitly via
    `just test -p codex-exec` scope `suite::apply_patch::shutdown`.
    SKIP WARNING (r1 B-M2): the test begins with `skip_if_no_network!` — in a
    sandboxed shell (CODEX_SANDBOX_NETWORK_DISABLED=1) it is SKIPPED (prints
    'Skipping test because it cannot execute when network is disabled in a Codex sandbox.'. A
    skipped run is NOT green for Item 3; the §6 Item 4 mock harness is the
    authoritative E2E evidence in that case.

Item 4 — wire-shape acceptance (mock harness, /tmp/xt214/; runs
  OUT-OF-SANDBOX on Mac or SCS — where the §3 evidence was taken — and is the
  authoritative E2E evidence whenever the exec-suite repro is skipped):
  - run A (MODE=custom, provider "test") -> rc=0, flushed.md written, 2 mock
    requests (previously rc=124 hang);
  - run C (MODE=function, provider "test") -> rc=0 (regression);
  - run B (built-in openai) -> rc=0 (regression).

## 7. Gates

1. `just test -p codex-core` (apply_patch_tests + core suite)
2. `just test -p codex-exec` (suite::apply_patch incl. the repro test)
3. `just fmt` (codex-rs)
4. `just fix -p codex-core`
5. Mock harness acceptance per §6 Item 4 (debug binary; release behavior
   follows from the same code path and is re-verified at the SCS release
   rebuild per the campaign chain).
6. Full suite `just test` (repo AGENTS.md: required after core changes;
   operator ask before the full-suite run). Mind gate 2's skip semantics
   per §6 Item 3.

## 8. Review log

Round 1 (2026-09-22) — dual fresh seats, verdicts
  /tmp/xt214-item-spec-seat-{A,B}-r1-verdict.md:
  - Seat A (RCA + fix design): B=0 M=1 m=3 n=2. M1 test session shape ->
    Item 1 rewrite (F1 harness pattern mandated); m1 RED-note error class ->
    fixed in Item 1; m2 scope overstatement -> §4 scope boundary + ratchet
    wording; m3 Custom content policy -> §5.2 verbatim-only decision; n1
    extractor naming -> §5.4 generalize `apply_patch_payload_command`,
    rename test (Item 1); n2 §5.1 parenthetical -> fixed.
  - Seat B (tests + divergence + ratchet): B=0 M=2 m=4 n=4. M1 same session
    shape (converged with A; fixed); M2 E2E skip semantics -> Item 3 skip
    warning + Item 4 out-of-sandbox authority + gate 6; m1 RED note
    (converged; fixed); m2 with_updated_hook_input test -> added to Item 1;
    m3 diff-consumer flip assertion -> added to Item 1; m4 full-suite gate
    -> added as gate 6; n1 A3 class wording -> fixed; n2 production/test
    boundary -> §5 header; n3 rename + call-site update -> Item 1; n4 §4
    step-4 wording -> fixed.
Round 2 (2026-09-22) — dual fresh seats, verdicts
  /tmp/xt214-item-spec-seat-{A,B}-r2-verdict.md:
  - Seat A: 0B 0M 0m 5n. All r1 B/M verified RESOLVED; fix design holds
    end-to-end (matches_kind closes the sole payload-Fatal source,
    registry.rs:592, on every dispatch path; verbatim-only degrades
    worst-case to a teachable RespondToModel). Nits: two were the
    coordinator's batch-1 write failure (status line, §5 production/test
    header) — applied post-round-2 and verified (lines 3, 124); remainder
    doc-hygiene per A's verdict.
  - Seat B: 0B 1M 1m 4n. B2-M1 (MAJOR): §5.4 had generalized the
    UPSTREAM-derived apply_patch_payload_command (blame #18391), breaking
    the r1-audited "ApplyPatchHandler unchanged" invariant with an
    unreachable upstream behavior delta (merge debt). RESOLVED via B's
    option (a): §5.4 now extends the FORK-OWNED function_apply_patch_patch_text
    (renamed apply_patch_handler_patch_text); apply_patch_payload_command
    stays byte-identical; Item 1 test plan updated. B2-m1 (§5 header):
    stale at review time — already applied post-Seat-A-r2 (verified line
    124). B2-n1 ("Used by" misattributed with_updated_hook_input): fixed in
    §5.4. B2-n2 (second extractor test name staleness): fixed in Item 1
    (both tests renamed). B2-n3 (skip-message quote): corrected to the
    exact string from core/tests/common/lib.rs:602. B verified: r1 M1/M2
    resolved, gate list complete, no test-name collisions, matches_kind
    test exhaustive, ratchet/scope wording accurate.
Round 3 (2026-09-22) — dual fresh seats, verdicts
  /tmp/xt214-item-spec-seat-{A,B}-r3-verdict.md. CONVERGENCE ROUND: both
  seats 0B 0M; Seat B issued the explicit READY call.
  - Seat A: 0B 0M 0m 2n. v1.2 §5.4 rework verified end-to-end at source
    (blame: function_apply_patch_patch_text = fork 939a6dc6f4;
    apply_patch_payload_command = upstream #18391, byte-identical to
    205f3671e1); new probe found and EXCLUDED a second "incompatible
    payload" Fatal producer (codex-rs/tools/src/tool_call.rs:164) — the
    matches_kind gate remains the sole producer. Nits applied in v1.3:
    A3-n1 extractor doc-comment mandate (§5.4); A3-n2 rename parenthetical
    (§5.4).
  - Seat B: 0B 0M 1m 3n. Divergence invariant HOLDS at hunk level (ownership
    table; 8/8 new/renamed identifiers collision-free; skip-string
    byte-exact). Findings applied in v1.3: B3-m1 line-blame pre-note
    (§5.2); B3-n1 extractor doc comment (§5.4, same as A3-n1); B3-n2
    "two-arm" exhaustiveness (§5.4, ToolSearch arm named); B3-n3 missing
    RED notes (Item 1, both added).
Round 4 (2026-09-22) — dual fresh confirmation seats, verdicts
  /tmp/xt214-item-spec-seat-{A,B}-r4-verdict.md. Both 0B 0M 0m 1n, both
  READY. All six v1.3 additions verified present and factually exact at
  source (guard blame e9702411ab, 3-variant ToolPayload exhaustiveness,
  0 name collisions, doc-comment verbatim, :637 fallthrough, :616-618
  None, Item 1 consistency, §8 log match). The single shared nit (B4-n1 =
  R4-n1): the §5.2 pre-note parenthetical was #17027-era provenance, not a
  current-tree statement — reworded in v1.3.1 with the surviving-line
  inventory. CONVERGED: two consecutive clean 0B/0M rounds (r3, r4); spec
  stage closed.

## 9. Task breakdown (stage 4)

Ownership model: ONE worker per task; the coordinator verifies first-hand,
runs gates, and commits (pathspec-strict). No task touches files outside its
ownership list. You are not alone in the codebase — do not revert others'
edits.

### T1 — RED: add the 5 existing-symbol tests + update 1 extractor test
(r1 follow-up delta: 6th new test per SEAT B F2, §6 bullet; extractor-test
 rename pulled forward into T1 — see §11)
Ownership: `codex-rs/core/src/tools/handlers/apply_patch_tests.rs` ONLY.
1. Implement §6 Item 1 with this compilation constraint: in T1 NO test may
   reference `apply_patch_handler_patch_text` (the renamed extractor does not
   exist yet). Concretely:
   - add `function_apply_patch_handler_accepts_custom_payload` (F1 harness
     session pattern per Item 1 — write-asserting). Use the F1-proven patch
     shape (trailing newline after `*** End Patch`, cf. the F1 test's patch
     literal at apply_patch_tests.rs:512-518) rather than `sample_patch()`
     (which lacks it and is unproven through the parser). The F1 harness
     session cwd is the PROCESS cwd (crate dir under nextest), so the test
     MUST clean up its written file after asserting (fs::remove_file) or it
     will dirty `git status` for the exit check.
   - add `function_apply_patch_matches_kind_accepts_function_and_custom`;
   - add `function_apply_patch_pre_and_post_hook_payloads_cover_both_payload_kinds`;
   - add `function_apply_patch_with_updated_hook_input_rewrites_custom_payload`
     (RED note: Ok, Custom input UNCHANGED — :637 fallthrough);
   - add `function_apply_patch_handler_diff_consumer_is_some` (RED note:
     None — :616-618);
   - UPDATE in place (keeping the current function name
     `function_apply_patch_patch_text`) the test
     `function_apply_patch_patch_text_rejects_non_function_payloads`:
     flip its Custom assertion to `Some(input)` per §5.2 verbatim-only
     policy. (This makes it RED today; it is renamed in T2 alongside the
     implementation rename.)
2. Run the scoped tests with the sanctioned runner (repo AGENTS.md: never
   bare `cargo test` — the `test` recipe sets RUST_MIN_STACK; the nextest
   filter is positional and must match the MODULE PATH
   `tools::handlers::apply_patch::tests` — the file name `apply_patch_tests`
   is not part of any test name):
   `just test -p codex-core -- tools::handlers::apply_patch::tests`
   (add `-v --nocapture` when a test's stdout must be visible).
   CARGO_BUILD_JOBS=2, machine under load. Save the scoped-suite output under
   /tmp/xt214/ (e.g. /tmp/xt214/t1-red.log) and RECORD evidence from it
   for each test: it fails (or, for the update, fails on the Custom
   assertion) with the spec's documented RED class (guard
   `RespondToModel("apply_patch function handler received unsupported
   payload")` for the write test; Custom-arm false for matches_kind; None
   for hook payloads; Ok-unchanged for with_updated_hook_input; None for
   diff consumer; None for the extractor Custom case).
Exit: 6 new tests + 1 updated test all RED for the documented reasons;
implementation file untouched — verify the tracked-change set is only the
test file (`git diff --name-only` / `git status --porcelain | rg '^[MAD] '`;
the tree carries pre-existing untracked noise, so raw `git status` is not the
check).

### T2 — GREEN: implement §5.1-5.5 + finish the extractor rename
Ownership: `codex-rs/core/src/tools/handlers/apply_patch.rs` (production) +
`codex-rs/core/src/tools/handlers/apply_patch_tests.rs` (rename only).
1. Implement §5.1 (matches_kind both kinds), §5.2 (handle_call payload
   extraction, Custom verbatim; keep the Function-arm teachable errors and
   `with_environment_id_line` flow intact), §5.3 (create_diff_consumer
   `Some(Box::<ApplyPatchArgumentDiffConsumer>::default())`), §5.4 (extend
   `function_apply_patch_patch_text` -> rename `apply_patch_handler_patch_text`,
   exhaustive 3-arm, doc comment updated; `apply_patch_payload_command`
   BYTE-IDENTICAL — verify with `git diff` that it is untouched), §5.5
   (handler doc comments), and the `with_updated_hook_input` Custom arm.
2. Finish the extractor rename (r1 follow-up delta: the updated test
   `apply_patch_handler_patch_text_extracts_custom_input_verbatim` was
   ALREADY renamed in T1 — do not look for the old name). Remaining in T2:
   rename the sibling `function_apply_patch_patch_text_extracts_patch_argument`
   -> `apply_patch_handler_patch_text_extracts_patch_argument`, apply the
   production rename (item 1), and update BOTH extractor tests' call sites.
   Forbidden-identifier checks must be call-site scoped: the test fn names
   now contain the new production name.
3. Run the scoped tests (`just test -p codex-core
   -- tools::handlers::apply_patch::tests`) -> ALL GREEN (6 new + 2 renamed
   extractor tests + the pre-existing function-shape teachable-error tests +
   diff-consumer tests stay green).
Exit: scoped suite green; `git diff` on apply_patch.rs confined to the
fork-owned regions (the §5.2 divergence pre-note names the exact
upstream-blamed lines touched: L530/L532/L533 — L531 is fork-owned);
MECHANIZED hunk check: `git diff -U0 --
codex-rs/core/src/tools/handlers/apply_patch.rs | rg '^@@'` must show NO hunk
intersecting L283-290 (`apply_patch_payload_command` — byte-identical) and no
hunk outside the L~480-660 fork handler region; and the tracked-change set
(`git status --porcelain | rg '^[MAD] '`) shows only the two owned files.

### T3 — gates (verification runs, no code edits)
1. Rebuild + run `just test -p codex-core` (full crate).
2. Rebuild + run `just test -p codex-exec`; the repro test
   `suite::apply_patch::shutdown_flushes_completed_turn_and_file_diff` MUST
   ACTUALLY EXECUTE (this environment does not set
   CODEX_SANDBOX_NETWORK_DISABLED). Verification method (a skip exits 0 and
   nextest HIDES the captured stdout of passing/skipped tests, so the full
   `just test -p codex-exec` run cannot distinguish executed from skipped):
   run the single test with `just test -p codex-exec -- -v --nocapture
   'shutdown_flushes_completed_turn_and_file_diff'` and assert the visible
   output shows execution (no 'Skipping test because it cannot execute when
   network is disabled in a Codex sandbox.' line; the test body's activity is
   present). If it is skipped, gate 2 is NOT green.
3. `just fmt` (codex-rs), then `just fix -p codex-core`. STALE-BINARY
   GUARD: record `shasum` of target/debug/codex-exec immediately before step
   4; if `just fix` modified apply_patch.rs (check `git status` on that file),
   re-run steps 1-2 first — the harness must always exercise the exact binary
   T4 commits.
4. Mock harness (out-of-sandbox, main-tree debug codex-exec; its shasum must
   match step 3's recorded value). First harden the /tmp/xt214 harness
   scripts (coordinator-owned /tmp tooling, not repo files). Do NOT assume a
   pid-file off-by-one (empirically `$!` usually captures the real server
   process); the failure observed in the RCA runs was a SILENT teardown
   (`kill ... 2>/dev/null`) leaving an orphan mock that holds pipes and can
   answer a later run — make teardown `pgrep -f 'mock_server.py <port>'`
   based and verify no orphan remains after each run. Clear request-dir
   residue before the acceptance runs (move stale `req_test/`/`req_builtin/`
   + `mock_*.pid` aside with timestamped mv; the scripts recreate dirs).
   Then, on distinct ports 65450+, with per-run wire-shape assertions:
   run A `MODE=custom bash /tmp/xt214/run_ab2.sh test <port>` -> rc=0,
   flushed.md present, 2 requests, AND req1.json contains `custom_tool_call`
   (previously rc=124 hang);
   run C `MODE=function bash /tmp/xt214/run_ab2.sh test <port>` -> rc=0 AND
   req1.json contains `function_call` (guards against run C silently hitting
   an orphaned custom-mode mock);
   run B `bash /tmp/xt214/run_builtin.sh <port>` -> rc=0.
5. Full suite `just test` (Mac, CARGO_BUILD_JOBS=2, load-check first;
   operator standing campaign approval per the campaign's gate practice —
   record the approval basis in the bead note).
Exit: all gates green with recorded evidence (logs under /tmp/xt214/).

### T4 — commit, push, sync, bead (coordinator)
1. Pathspec-strict commit: `codex-rs/core/src/tools/handlers/apply_patch.rs`,
   `codex-rs/core/src/tools/handlers/apply_patch_tests.rs`,
   `docs/xt214-custom-payload-fix-spec.md`. Commit msg via `-F` file
   (subject: `apply-patch compat: FunctionApplyPatchHandler accepts raw
   custom_tool_call payloads (apex-xt2.14)`).
2. Push BOTH remotes (fork APEX/codex + netbrah/codex; NEVER origin);
   `git ls-remote` verify both at the full SHA.
3. SCS ff-sync (`ssh 10.234.218.97`: `git fetch` + `git merge --ff-only` in
   /x/eng/ai_engineering/APEX/codex, preserving `M codex-rs/.cargo/config.toml`
   + pre-existing untracked junk).
4. Bead apex-xt2.14: note with gate evidence; leave OPEN (closure happens
   after the campaign-chain release rebuild re-verifies; if the chain is
   deferred, the note records "fix landed, release re-verify pending").
   Pathspec discipline: gate 3's `just fix`/`just fmt` can modify files
   outside the pathspec list — report any such edit in the bead note (leave
   uncommitted or amend into the commit deliberately) so the pathspec-strict
   commit is not silently incomplete (expected no-op on a clippy-clean
   crate).

## 10. Breakdown review log

Round 1 (2026-09-22) — dual fresh seats, verdicts
/tmp/xt214-item-breakdown-seat-{A,B}-r1-verdict.md. Both 0B 0M; both READY
for stage 5. Findings applied in v1.4:
- Seat A (4m 5n): M1 gate-2 skip-verification method (nextest hides skip
  stdout; single-test `-v --nocapture` run mandated, T3.2); M2 stale-binary
  window between `just fix` and the mock harness (shasum guard + rebuild
  rule, T3.3); M3 run-C wire-shape assertion vs orphaned custom-mode mock
  (per-run req1.json shape asserts + distinct ports, T3.4); M4 forbidden
  `cargo test` example + zero-match filter (sanctioned `just test` runner +
  module-path filter `tools::handlers::apply_patch::tests`, T1.2/T2.3);
  N1 off-by-one premise retracted — real risk is silent-teardown orphans
  (T3.4 reworded); N2 §5.2 surviving-line range tightened (L531 fork-owned);
  N3 write-test cwd is process cwd -> cleanup mandated (T1); N4 patch shape
  must mirror F1's proven literal (T1); N5 just fix/fmt may touch files
  outside the pathspec (T4.4 reporting duty).
- Seat B (3m 4n): m1 same runner/filter fix (converged with A-M4; T1.2/
  T2.3); m2 request-count residue sensitivity (residue-clear step, T3.4);
  m3 T2 exit missing file-scope check (tracked-change set added, T2 exit);
  n1 T1 exit tracked-change check (applied); n2 T1 evidence location
  /tmp/xt214/ (applied); n3 T4 bead wording (applied); n4 mechanized
  hunk/byte-identity check (T2 exit).
BREAKDOWN CLOSED at 0B 0M.

## 11. Status

Spec design: FINAL (rounds 1-4, 0B 0M). Breakdown: FINAL (round 1, 0B 0M).
§3 item 4 corrected 5789dd8500 probe: COMPLETE (hang reproduced; see bisect_log.md).

Stage 5 (TDD execution):
- T1 (RED): DONE. Review round 1 dual seats 0B 0M (A: 1MINOR 2NIT,
  B: 2MINOR 1NIT; verdicts /tmp/xt214-item-t1-seat-{A,B}-r1-verdict.md).
  r1 follow-up applied: F2 JSON-shaped verbatim discriminator test added
  (s6 delta recorded above; counts updated in s9 T1/T2 exits) + evidence
  record (rename pull-forward deviation, SUPERSEDED continuation note).
  Final T1 state: 27 scoped tests, 7 RED (all documented reasons, both
  retries), tracked change = apply_patch_tests.rs only (+162/-3).
  Round 2 (delta-scoped) CLOSED 0B 0M 0MINOR (A 3NIT, B 2NIT — all NITs
  remediated in this doc + t1-red.log:426 span fix); verdicts
  /tmp/xt214-item-t1-seat-{A,B}-r2-verdict.md. T1 COMPLETE.
- T2 (GREEN): DONE. 27/27 scoped GREEN (t2-green.log); divergence invariants
  hold (no hunk before L482; apply_patch_payload_command L283-290
  byte-identical to HEAD); old name zero refs. Review round 1 dual seats
  0B 0M (A: 0MINOR 2NIT, B: 1MINOR 1NIT; verdicts
  /tmp/xt214-item-t2-seat-{A,B}-r1-verdict.md). MINOR-1 (record-level, no
  code fix): the T1->T2 test-file delta is 4 rename-related lines, two of
  them T1-added call-site lines (numstat-neutral); HEAD-numstat shows +2/+2
  (164/5 minus T1's 162/3). The T4 commit message and bead note cite the
  true 4-line rename delta. B-NIT-1 (pre-existing warnings) + A-NITs: no
  action. T2 review loop converged at r1 (no artifact change to re-review).
- T3 (gates): GATE-PLANE MIGRATION to SCS (operator standing instruction:
  'we move to linux when resources are scarce'; Mac load-check FAILED).
  Mac gate 1 (just test -p codex-core, session 11699) frozen 60+min:
  `sample` shows the cargo main thread 100% in stat() inside
  fingerprint::Fingerprint::check_filesystem — sustained disk I/O storm
  (13k small-IOPS; operator's grok-build release test + iCloud/TM
  churn on the 19.5k-file target). NOT a hang; left running as bonus
  evidence. SCS plane: worktree at 3e0e40021 + scp overlay of the two
  changed files (git diff --stat confirms); load ~3. SCS deviations
  recorded: no cargo-nextest (offline registry lacks it; network 503
  blocks rustup component adds) -> gate runners use `cargo test`
  (libtest) instead of nextest; no clippy on SCS -> gate 4 (clippy
  --fix) deferred to the Mac once the I/O storm clears. SCS gate 1
  (cargo test -p codex-core --no-fail-fast, CARGO_BUILD_JOBS=4,
  offline recipe) in flight: /x/eng/ai_engineering/APEX/scs-gate1-core.log.
  Stale-binary guard: pre-gate Mac codex-exec sha eece971839... (mtime
  01:46:46, pre-fix HEAD build); SCS codex-exec rebuilt from the
  overlay tree before the mock harness gate.

  GATE 1 (cargo test -p codex-core, SCS): 4578 passed / 35 failed (with fix).
  Control run (same subset at HEAD, overlay reverted): 31/34 fail identically.
  Set-delta adjudication (all first-hand, SCS logs scs-gate1-*.log,
  scs-control-failset.log, scs-control-delta4.log, scs-overlay-delta1.log):
  - suite::network_approval::...plain_http and
    responses_stream_includes_turn_metadata_header_for_git_workspace_e2e:
    FAIL in both contexts (small-batch HEAD and overlay) -> pre-existing.
  - session::turn::tests::post_sampling_token_estimate_...: failed only in the
    full-parallel run; PASSES 3/3 in isolation WITH the fix (and at HEAD)
    -> parallel-context flake, not fix-attributable.
  - suite::guardian_subagent_authorization::...legacy_oversized_answer:
    full-suite-only failure; passes in isolation both contexts -> flake class.
  VERDICT: ZERO fix-attributable failures; all 35 are pre-existing
  branch/environment state on SCS (network-dependent suite tests, fork
  snapshot drift, context flakes). No apply_patch-path test fails.
  GATE 2 (exec repro, SCS): suite::apply_patch::shutdown_flushes_completed_
  turn_and_file_diff EXECUTES (no skip line) and PASSES (8.47s) with the
  fix (scs-gate2-exec-repro.log).
  GATE 3 (fmt, SCS): rustfmt 1.94.1 --check on both changed files: clean.
  GATE 5 (mock harness, SCS /x/eng/ai_engineering/APEX/xt214-harness,
  codex-exec sha e7ecb4502516e1d8 built from the overlay tree):
  - custom + "test" provider (THE pre-fix hang case): rc=0, flushed.md
    written, 2 requests, wire-shape OK (custom_tool_call echoed), rollout
    ends custom_tool_call_output | task_complete.
  - function + "test": rc=0 PASS (control). builtin "openai": rc=0 PASS.
  - All teardowns clean, no orphans.
  GATE 4 (clippy --fix -p codex-core): deferred to Mac (no clippy on SCS;
  rustup 503).
  GATE 6 (full suite): SATISFIED BY COMPOSITION (recorded deviation; a
  literal full workspace re-run is infeasible on either plane: SCS NFS 97%
  full vs 179G warm target; Mac under sustained load). Basis: (a) the
  branch closed apex-xt2.13 at this exact HEAD 3e0e400218 with a green
  full-suite gate and was pushed to both remotes; (b) this diff touches
  exactly two codex-core files, so only codex-core test behavior can
  change — covered by GATE 1 (full crate, both planes, adjudicated);
  (c) the full codex-exec suite (changed crate's main consumer binary)
  re-runs after gate 1 as the additional anchor. If gate 1 surfaces any
  failure, the gate-1 adjudication protocol (HEAD control / isolation
  re-runs) applies to it.
- T4 (commit/push/sync/bead): pending.
