# Item 4 — Integration (spec T4): TDD evidence (apex-ayl.52)

Worker: `/root/item4_integration_worker` (stage 5, item 4 of 6).
Branch: `feat/normalize-content-types-vllm`, working-tree HEAD `2a25e69a46`
(post Items 1-3: `e21f608ac4` P2, `a4d5af1f62` P1, `939a6dc6f4` P3; the
campaign function-tool seam remains uncommitted working-tree state per
breakdown §0 execution baseline).
SoT: spec v5 `docs/responses-compat-apply-patch-format.md` (§4 T4, §3.1,
§1.1, §3.4); breakdown v3
`docs/responses-compat-apply-patch-task-breakdown.md` (Item 4, §0, §5).

## TDD-mode note (green-at-write-time regression locks)

Per breakdown Item 4, all three tests are **regression locks with no prior
red phase by design** — Items 1-3 already landed the behavior they lock,
so there is no failing-test-first step; the "red" that would prove the lock
bites is a prior item regressing. This document therefore records, in lieu
of red captures: (a) the E0308 sibling-bind rationale re-derived at the
working-tree state, (b) each test's first run (the gate run is the first
run), and (c) the T4.1 wire-body capture (the captured `/v1/responses`
request excerpt showing the function tool shape).

## Pre-check (line refs re-derived at working-tree state, protocol step 1)

Breakdown Item 4 line refs were verified by reading the current tree
before any edit (one cite corrected at r1 review — see the
`apply_patch_responses` note below). Both owned files were **clean vs HEAD** pre-change
(`git diff HEAD --stat` over the two files: empty).

**`codex-rs/core/tests/common/responses.rs` (1642 lines pre-change):**
- `ResponseMock` struct :39; `single_request()` :50-56 (panics unless
  exactly 1 request); `requests()` :58-60. Not `#[must_use]`, so
  discarding the mount helper's return value in T4.2/T4.2b warns nothing.
- `ResponsesRequest::body_json()` :188.
- Generic constructor `ev_function_call(call_id, name, arguments: &str)`
  :933-943 — takes `arguments: &str` (E0308 anchor; see below).
- Closest siblings for the new helper:
  `ev_apply_patch_custom_tool_call` doc :1005-1007 + fn :1008-1021
  (the ONLY `ev_*` apply_patch sibling with `///` docs — the new helper
  follows suit),
  `ev_exec_command_call` :1020-1023,
  `ev_exec_command_call_with_args` :1025-1028,
  `ev_apply_patch_exec_command_call_via_heredoc` :1030-1035
  (serialize-then-call shape mirrored, including the
  `expect("serialize apply_patch arguments")` message and the blank line
  before the final call).
- `mount_sse_sequence(server, Vec<String>) -> ResponseMock` :1437 (doc :1434).
- File's doc style: sparse — most `pub fn ev_*` carry no `///` docs, but
  every doc'd helper in the family uses the
  `Convenience: SSE event for …` phrasing (:736, :747, :784, :1005); the
  new helper's doc matches that phrasing and is modeled on its
  custom-tool-call counterpart.

**`codex-rs/core/tests/suite/apply_patch_cli.rs` (2487 lines pre-change):**
- `pub async fn mount_apply_patch` :228-244 — builds
  `apply_patch_responses(…, ev_apply_patch_custom_tool_call)`; `pub`
  because `apply_patch_serialization.rs:11` imports it. Reuse is
  **forbidden** by the breakdown: it always emits `custom_tool_call`,
  rejected on the renamed provider.
- `fn apply_patch_responses(call_id, patch, assistant_msg,
  apply_patch_call: fn(&str, &str) -> serde_json::Value)` :266-283 — the
  fn-pointer parameter accepts the new `ev_apply_patch_function_call`
  (same signature) with zero changes to the shared helper. (Cite
  corrected at r1 review: the pre-check initially transcribed the
  breakdown v2's wrong `:257-271` — r1 seat B m2.)
- `harness.read_file_text` harness method (defined
  `test_codex.rs:1209-1217`), usage pattern at :323
  (`assert_apply_patch_crlf_update`) and :444 (
  `apply_patch_cli_multiple_operations_integration` — the closest full
  behavioral pattern: seed → mount → `harness.submit` → `read_file_text`
  → `assert_eq!`).
- `use pretty_assertions::assert_eq;` :10 (deep-equality convention
  already in file); `use serde_json::json;` :78; no `Value` import
  (assertions use the `body["tools"]` index/`.pointer()` style of
  `code_mode.rs:5101`, needing no new type import).
- `apply_patch_harness_with(configure)` :91-98 (boxes harness
  construction) — used by the new renamed-provider harness helper.

**`codex-rs/core/tests/common/test_codex.rs` (harness plumbing):**
- `TestCodexBuilder::with_config(FnOnce(&mut Config))` :347-353 — the
  rename mechanism; mutators run after `prepare_config` sets
  `config.model_provider`, so setting
  `config.model_provider.name = "vLLM"` lands in the final config.
- Built-in provider clone: `prepare_config` :836-844 —
  `ModelProviderInfo { base_url: Some(base_url),
  supports_websockets: false, ..built_in_model_providers(/*openai_base_url*/
  None)["openai"].clone() }` :840-844; the cloned provider's `name` is
  `"OpenAI"` (built-in definition at
  `model-provider-info/src/lib.rs:465`), so the rename must target
  **`config.model_provider.name`** — that is the field `is_openai()`
  reads.
- `test_codex()` :1363+ — `auth: CodexAuth::from_api_key("dummy")`
  :1369: API-key auth sends a plain `Bearer dummy` header for any
  provider name (no OpenAI-specific auth path), so the rename needs no
  auth change (spec §4 T4.1 note, v1 seat B d-answer).
- `submit_turn(prompt)` :959-962 = `submit_turn_with_permission_profile`
  with `PermissionProfile::Disabled` :977-985 = `AskForApproval::Never` —
  permissive by default, matching the existing apply_patch_cli test
  pattern (`harness.submit(…)` writes files without approval prompts).
- `read_file_text(rel)` :1209-1217; `write_file(rel, contents)` :1178.

**Seam/committed mechanics (read-only; what the tests lock):**
- `codex-rs/core/src/tools/handlers/apply_patch.rs`:
  `FunctionApplyPatchHandler` struct :480-482; `handle_call` :508+ (parses
  `{"patch": …}` and delegates to the shared `run_apply_patch_text`
  :400-468 — the same parse/verify/apply path as the custom-tool form);
  `matches_kind` :607-609 accepts **only** `ToolPayload::Function`.
- `codex-rs/core/src/tools/registry.rs`: kind-mismatch rejection
  :548-557, message `tool {tool_name} invoked with incompatible payload`
  :550 (this is why reusing `mount_apply_patch` — a `custom_tool_call`
  item — on the renamed provider cannot pass).
- `codex-rs/core/src/tools/spec_plan.rs` (seam line refs at working-tree
  state): registration gate `if environment_mode.has_environment() &&
  context.model_info.apply_patch_tool_type.is_some()` :1257; capability
  branch `context.turn_context.provider.capabilities()
  .apply_patch_function_tool` :1260-1264; `registry.add(
  FunctionApplyPatchHandler::new(include_environment_id))` :1267.
- `codex-rs/model-provider/src/provider.rs` (seam):
  `apply_patch_function_tool: !self.info.is_openai()` :372 — the
  function tool is served to every non-`"OpenAI"`-named provider.
- `codex-rs/model-provider-info/src/lib.rs`:
  `OPENAI_PROVIDER_NAME: &str = "OpenAI"` :40; `is_openai()` is the exact
  case-sensitive name match `self.name == OPENAI_PROVIDER_NAME`
  :546-547 — so any other name (here `"vLLM"`) takes the function path.
- `codex-rs/models-manager/models.json` (committed): the harness default
  model `gpt-5.5` (`test_codex.rs:857`) carries
  `apply_patch_tool_type: "freeform"`, so `apply_patch_tool_type.is_some()`
  is true and the provider name is the ONLY thing selecting the handler.
- T3.2 committed test modeled for the F1 payload:
  `apply_patch_tests.rs:509-558`
  (`function_apply_patch_applies_f1_shaped_raw_add_file`) — the T4.2 patch
  is byte-for-byte the spec §1.1 F1 payload used there (H1 first content
  line, em-dash line, blank line, table lines, no per-line `+` prefixes).

## E0308 rationale for the sibling local bind (re-derived)

The one-liner alternative does not compile at the working-tree state:

```rust
// does NOT compile — E0308:
pub fn ev_apply_patch_function_call(call_id: &str, patch: &str) -> Value {
    ev_function_call(call_id, "apply_patch",
        serde_json::to_string(&serde_json::json!({ "patch": patch })).unwrap())
}
```

`ev_function_call`'s third parameter is `arguments: &str`
(`responses.rs:933`), while `serde_json::to_string(…)` returns an owned
`String`; `String` does not coerce to `&str` at a function argument
position (E0308 mismatched types: expected `&str`, found `String`).
The sibling helpers solve it identically — `ev_exec_command_call_with_args`
(:1025-1028) and `ev_apply_patch_exec_command_call_via_heredoc`
(:1030-1035) both bind the serialized JSON locally
(`let arguments = serde_json::to_string(&args).expect(…);`) and pass
`&arguments`. The new helper copies that shape verbatim (breakdown Item 4
step 2). Compile-verification of the helper comes with the first gate
build (a type error would fail the `codex-core` test-target compile
before any test runs); the build in the Gates section is that
verification.

## Changes (purely additive; no product code)

**`codex-rs/core/tests/common/responses.rs`** (+11, 0 deletions):
new `pub fn ev_apply_patch_function_call` at :1041-1046 (doc :1037-1040),
inserted between `ev_apply_patch_exec_command_call_via_heredoc`
(:1030-1035) and `sse_failed` (:1048) — next to the sibling pattern
helpers, per breakdown step 2. `pub` inside the test-support module
exactly like its siblings (this file's 37 `pub fn`s are all `pub`; the
module is the crate's public test-support surface, e.g.
`apply_patch_cli.rs:8` imports the siblings by name).

**`codex-rs/core/tests/suite/apply_patch_cli.rs`** (+128, 0 deletions):
- :8 `use core_test_support::responses::ev_apply_patch_function_call;`
  (sorted between the :7 heredoc sibling and :9
  `ev_exec_command_call`).
- :63 `use core_test_support::responses::ResponseMock;` (suite-import
  convention, cf. `review.rs:43`, `compact_resume_fork.rs:30`).
- :248-259 `async fn mount_apply_patch_function_call(…) -> ResponseMock`
  — mirrors `mount_apply_patch` :230-246 through the shared
  `apply_patch_responses` fn-pointer helper :281-298, passing
  `ev_apply_patch_function_call`. Private (used only in this file),
  unlike `pub` `mount_apply_patch` (imported by
  `apply_patch_serialization.rs:11`). Returns the `ResponseMock` so T4.1
  can inspect captured requests; T4.2/T4.2b discard it (the type is not
  `#[must_use]`).
- :2505-2515 section comment + `async fn vllm_apply_patch_harness()` —
  `apply_patch_harness_with(|builder| builder.with_config(|config|
  config.model_provider.name = "vLLM".to_string()))`. Named "vLLM"
  because the campaign target is a vLLM-served non-OpenAI deployment
  (glm-5.2, spec §1.2); any non-`"OpenAI"` name takes the same path
  (`is_openai` exact match, `model-provider-info/src/lib.rs:546-547`).
- :2517-2559 **T4.1**
  `apply_patch_function_tool_served_to_non_openai_provider`: renames the
  provider, mounts a function-call turn, and asserts on the FIRST
  captured `/v1/responses` request body (`mock.requests().first()`: the
  turn posts twice — tool-call request, then the
  `function_call_output` follow-up — so `single_request()` would panic):
  `tools[]` carries exactly an `apply_patch` tool with
  `"type": "function"` (asserted via `assert_eq!(apply_patch_tool["type"],
  json!("function"))`), and
  `parameters.properties.patch.description` contains ALL 7 spec §3.1
  drift substrings (the same list T2.1 drift-guards at
  `apply_patch_spec_tests.rs:96-104`). Green at write time = regression
  lock over Items 1-2 through the full request-construction stack.
- :2561-2587 **T4.2**
  `apply_patch_function_tool_applies_f1_shaped_raw_add_file`: mocked
  model emits `function_call apply_patch` carrying the spec §1.1 F1
  payload verbatim (raw markdown Add-File, H1 first content line,
  no per-line `+` prefixes) for a NEW file `grok/plans/spec-freeze-r23-
  glm.md` → after the turn, `read_file_text` returns the byte-exact
  expected contents. Green at write time = regression lock over Item 1's
  P2 through the full stack (wire → registry → `FunctionApplyPatchHandler`
  → shared `run_apply_patch_text` → sandboxed fs).
- :2589-2615 **T4.2b**
  `apply_patch_function_tool_raw_add_file_overwrites_existing_file`:
  same F1 shape targeting an EXISTING file (seeded via
  `harness.write_file` with different contents) → final contents exactly
  the patch's. Pins the no-existence-check/overwrite decision end-to-end
  (spec §3.2 decision; T1.9 pins it at parser level).

Repo conventions honored: `pretty_assertions::assert_eq` (file import
:10) for all comparisons; deep/whole-object equality where sensible
(the `["type"]` vs `json!("function")` check); no new public API outside
the test-support module (the `ev_*` helper is `pub` inside
`core_test_support::responses` exactly like its siblings; the mount
helper and harness helper are private); `/*param_name*/` comments not
needed (no opaque positional literals in the new code); inline format!
args (no format! added); no collapsible-if; no refactors (
`apply_patch_responses` reused unmodified).

## TDD note on the mount helper (breakdown step 3 options)

Breakdown step 3 allowed "an `apply_patch_responses` function-call
variant, or a parallel `mount_sse_sequence` call". Chosen: the
`apply_patch_responses` path — `apply_patch_responses` already takes the
event constructor as a `fn(&str, &str) -> serde_json::Value` parameter
(:281-298), and `ev_apply_patch_function_call` matches that signature
exactly, so `mount_apply_patch_function_call` is a 3-line mirror of
`mount_apply_patch` with ZERO changes to the shared helper or to the
existing custom-tool/ heredoc callers (`mount_apply_patch_model_output`
:261-279, `mount_apply_patch` :230-246 untouched). A parallel
`mount_sse_sequence` call would have duplicated the two-body
created/call/completed → message/completed sequence.

## Gates (from `codex-rs/`)

Machine context (binding for interpreting this section): the host is
shared with other campaign workers; during the gate window a sibling
worker ran `cargo test --release` (6 xai-grok crates,
`CARGO_BUILD_JOBS=4`) from
`/Users/palanisd/Projects/upstream/wt/grok-build-responses`
(10:48-10:57 EDT, per its `/tmp/reqvalid-47a-gate.log` end marker),
and loadavg tracked 15-35 at the start of the item and 6.7-9.5 while
the runs below were in flight. The task's green criterion anticipates
exactly this: 0 DETERMINISTIC failures, with the known load-noise class
= `timeout waiting for event: Elapsed(())` at
`core/tests/common/lib.rs:388` + nextest TMT; binding check
`grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u`
must be empty.

### Gate 1, run 1: `just test -p codex-core apply_patch`

(first run; log `/tmp/item4-gate1.log`; 118 tests in scope = 115
post-Item-3 + 3 new; exit 100, all non-passes in the load-noise class)

```text
   TRY 2 TMT [  60.470s] ( 99/118) codex-core::all suite::apply_patch_cli::apply_patch_function_tool_applies_f1_shaped_raw_add_file
   TRY 2 TMT [  60.063s] (100/118) codex-core::all suite::apply_patch_cli::apply_patch_function_tool_raw_add_file_overwrites_existing_file
   TRY 2 TMT [  60.224s] (101/118) codex-core::all suite::apply_patch_cli::apply_patch_function_tool_served_to_non_openai_provider
   TRY 2 TMT [  61.246s] ( 86/118) codex-core::all suite::apply_patch_cli::apply_patch_cli_rejects_empty_patch
   TRY 2 TMT [  60.657s] ( 87/118) codex-core::all suite::apply_patch_cli::apply_patch_cli_rejects_invalid_hunk_header
  TRY 2 FAIL [  40.945s] ( 93/118) codex-core::all suite::apply_patch_cli::apply_patch_cli_verification_failure_has_no_side_effects
error: test run failed
error: recipe `test` failed on line 88 with exit code 100
```

Binding check (run 1):

```text
$ grep -E "thread .+ panicked" /tmp/item4-gate1.log | grep -v 'lib.rs:388' | sort -u
(empty)
$ grep -E "thread .+ panicked" /tmp/item4-gate1.log | grep -oE "lib.rs:[0-9]+" | sort | uniq -c
  79 lib.rs:388
```

**All 79 panic lines are the known load-noise class; 0 deterministic
failures.** The pre-existing apply_patch tests TMT/FAIL in the same
class (identities recorded in the discrepancy notes), confirming the
non-passes are machine saturation, not this diff: e.g. the campaign's
own `apply_patch_cli_multiple_operations_integration` (pre-existing)
panics at `lib.rs:388` on TRY 1 and fails on TRY 2 for the same reason.

**Test-name change discovered by run 1 (recorded decision; rationale
corrected at r1 review — seat A F3/F5):** my three tests were initially
named `apply_patch_function_tool_*`. The nextest group
`core_apply_patch_cli_integration` (`max-threads = 1`; its override
comment concerns Windows runner process-startup stalls — the
"Higher concurrency causes integration test timeouts under resource
contention" comment belongs to the `app_server_integration_local`
group) filters on
`package(codex-core) & kind(test) & test(apply_patch_cli)`. The initial
names WOULD have matched that filter via the module path — `test()`
matches the full test identifier, which contains
`suite::apply_patch_cli::` (proven by `cargo nextest list` under the
exact filter, r1 seat A) — and the group override is in fact inactive
under `NEXTEST_PROFILE=local` (`[profile.local] inherits = "default"`
does not carry the `[[profile.default.overrides]]` assignment, nextest
0.9.137). Run 1's TMTs were therefore host-load contention (sibling
`cargo test --release`), not group exclusion. The names were still
renamed to `apply_patch_cli_function_tool_*` because that is the file's
dominant behavioral-test naming convention. Rename diff: 3 lines (the
fn names), no logic change.

### Gate 1, run 2 (post-rename): `just test -p codex-core apply_patch`

(log `/tmp/item4-gate1-run2.log`; still in flight while focused
verification runs — see below; partial state at 78/118 when the
focused runs started: my three tests TMT'd on both tries at 76-78/118,
contending with the focused run executing the SAME serialized tests —
self-contention; final tally recorded in Discrepancy notes D2.)

### Focused first-run verification: `just test -p codex-core apply_patch_cli_function_tool`

(runs ONLY the 3 new tests, serialized via the same group)

Focused run 1 (log `/tmp/item4-gate1-focused.log`, ~11:12-11:16 EDT,
sibling worker's gate had just ended):

```text
        PASS [   0.302s] (3/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_served_to_non_openai_provider
     Summary [ 230.522s] 3 tests run: 1 passed, 1 failed, 1 timed out, 4366 skipped
   TRY 2 TMT [  60.467s] (1/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file
  TRY 2 FAIL [  48.883s] (2/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file
```

T4.1 GREEN (the sole panic across the run is the
`lib.rs:388` class in T4.2b's TRY 2 — binding-check clean). T4.2/T4.2b
TMT'd while this run and gate run 2 executed concurrently.

Focused run 2 (log `/tmp/item4-gate1-focused2.log`):

```text
  TRY 2 PASS [   6.037s] (3/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_served_to_non_openai_provider
     Summary [ 306.975s] 3 tests run: 1 passed (1 flaky), 2 timed out, 4366 skipped
   TRY 2 TMT [  60.402s] (1/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file
   TRY 2 TMT [  60.319s] (2/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file
```

T4.1 GREEN again (flaky: TRY 1 TMT → TRY 2 PASS in 6.037s). T4.2/T4.2b
TMT'd again under the concurrent run 2.

**Interpretation (load, not logic):** T4.1 — the same harness shape as
T4.2/T4.2b (identical builder, mount, submit flow; only the patch
payload and the final assertion differ) — passes in 0.302s on a
headroom moment and 6.037s under medium contention. T4.2/T4.2b's only
failure mode in every attempt so far is the `lib.rs:388` event-wait
timeout / nextest 60s TMT — the same class that kills the pre-existing
`apply_patch_cli_*` tests in the same window (e.g. run 2:
`apply_patch_cli_add_overwrites_existing_file` TRY 2 TMT,
`apply_patch_cli_rejects_invalid_hunk_header` TRY 2 TMT). No assertion
has ever failed in any of my three tests; no panic line outside
`lib.rs:388` exists in any log. A contention-free final focused run is
recorded below (D2).

Focused run 3 (log `/tmp/item4-gate1-focused3.log`, run 2 stopped to
remove self-contention; load 5.4-7.3):

```text
        PASS [   0.358s] (1/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file
        PASS [  26.310s] (2/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file
   TRY 2 TMT [  60.398s] (3/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_served_to_non_openai_provider
     Summary [ 147.729s] 3 tests run: 2 passed, 1 timed out, 4366 skipped
```

Focused run 4 (log `/tmp/item4-gate1-focused4.log`; load 3.2 at start):

```text
        PASS [   0.295s] (1/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file
   TRY 2 TMT [  60.218s] (2/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file
   TRY 2 TMT [  60.121s] (3/3) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_served_to_non_openai_provider
     Summary [ 240.774s] 3 tests run: 1 passed, 2 timed out, 4366 skipped
```

Runs 3-4 show every test GREEN at least once (T4.1 ×2, T4.2 ×3,
T4.2b ×1) with only `lib.rs:388`-class TMTs elsewhere — but no single
3/3 run yet: the host has recurring 1-2 minute load spikes from other
workers, and a 3-test run spans ~2-4 minutes. To close with an
unambiguous per-test gate record, each test was then run SOLO (one
nextest run per test; the test needs only its own ~30s window):

```text
$ just test -p codex-core apply_patch_cli_function_tool_served_to_non_openai_provider   # T4.1 (log /tmp/item4-focused-t41.log)
        PASS [   0.298s] (1/1) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_served_to_non_openai_provider
     Summary [   0.314s] 1 test run: 1 passed, 4368 skipped

$ just test -p codex-core apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file  # T4.2 (log /tmp/item4-focused-t42.log)
        PASS [   0.268s] (1/1) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file
     Summary [   0.284s] 1 test run: 1 passed, 4368 skipped

$ just test -p codex-core apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file  # T4.2b (log /tmp/item4-focused-t42b.log)
        PASS [   0.257s] (1/1) codex-core::all suite::apply_patch_cli::apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file
     Summary [   0.275s] 1 test run: 1 passed, 4368 skipped
```

**ALL THREE NEW TESTS GREEN (per-test 1/1 summaries).** Binding check
over every focused log: empty (all panic lines, where present, are
`lib.rs:388`).

### T4.1 wire-body capture (captured `/v1/responses` request excerpt)

Method: a temporary single-line
`eprintln!("T41-WIRE-CAPTURE {apply_patch_tool}");` was injected into
T4.1 immediately before the `type` assertion, the test was run solo
with `-- --nocapture` (log `/tmp/item4-wire-capture2.log`), the captured
value was extracted, and the line was removed (final diff re-verified
purely additive — see Discrepancy D3). The object below is the EXACT
`apply_patch` tool value the test observed in the FIRST captured
`/v1/responses` request body's `tools[]` array (only the 2198-char
description is elided in the middle; head and tail shown verbatim):

```json
{
  "description": "The `apply_patch` tool can be used to edit files (add, delete, update, move). The complete patch goes in the `patch` argument.",
  "name": "apply_patch",
  "parameters": {
    "additionalProperties": false,
    "properties": {
      "patch": {
        "description": "The ENTIRE patch as a single string: the first line is `*** Begin Patch`, the last line is `*** End Patch`, and everything between is the patch body. Lines insi … [1878 elided middle chars; full text = spec §3.1 P1 const, 2198 chars, all 7 drift substrings asserted present] … ch\n*** Add File: notes/todo.md\n+# TODO\n+\n+1. ship the fix\n*** Update File: src/main.rs\n@@ fn main\n-    old_call();\n+    new_call();\n     shared();\n*** End Patch",
        "type": "string"
      }
    },
    "required": [
      "patch"
    ],
    "type": "object"
  },
  "strict": false,
  "type": "function"
}
```

Verification notes on the capture:
- `"type": "function"` — the function-tool form (NOT the freeform
  `{"type":"custom", …, "format": {"type":"grammar",…}}` form an
  OpenAI-named provider receives).
- `properties` contains only `patch` (single-environment session → no
  `environment_id`, matching
  `create_apply_patch_function_tool(/*include_environment_id*/ false)`).
- `required: ["patch"]`, `additionalProperties: false`,
  `strict: false` — the committed P1 shape (spec §3.1).
- Description length 2198 chars — matches the spec §3.1 measurement
  ("patch argument description 2,198 chars") exactly; all 7 §3.1 drift
  substrings verified present in the captured text (independent python
  check over the extracted JSON; the test itself asserts all 7 on
  every run).

### Gate 2: `just fix -p codex-core`

```text
cargo clippy --fix --tests --allow-dirty -p codex-core
    Checking core_test_support v0.0.0 (/Users/palanisd/Projects/upstream/codex/codex-rs/core/tests/common)
    Checking codex-core v0.0.0 (/Users/palanisd/Projects/upstream/codex/codex-rs/core)
       Fixed core/tests/suite/openai_file_mcp.rs (1 fix)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 52.33s
exit 0
```

**EXIT 0.** No fixes applied to either owned file.
**KNOWN HAZARD TRIGGERED (as the task brief predicted):** clippy
re-applied a pre-existing unused-import auto-fix in OUT-OF-SCOPE
`codex-rs/core/tests/suite/openai_file_mcp.rs`. Restored via
`git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs` and
verified clean vs HEAD (`git diff HEAD --stat` over that file: empty).
Recorded as Discrepancy D4.

### Gate 3: `just fmt`

```text
exit 0   (scripts/format.py, no output on success; ~9.5s)
```

`just fmt` (rustfmt) reformatted the new code in `apply_patch_cli.rs`:
moved `use core_test_support::responses::ResponseMock;` to its sorted
position in the second import group (:55, before
`ev_assistant_message`), and normalized a few line wraps
(e.g. the T4.1 patch literal to one line, `harness.submit(…)` chain
wrap). No semantic change; `responses.rs` untouched by fmt. The diff
remains purely additive: +11/+138, 0 deletions (post-fmt numstat
below). Per repo convention, tests were not re-run after `fix`/`fmt`;
the fmt-only delta over the tested source is whitespace/import-order
(verified by inspection of the post-fmt hunks).

## Final state

**Line refs at the final (post-fmt) tree state:**
- `codex-rs/core/tests/common/responses.rs` (1653 lines):
  `ev_apply_patch_function_call` :1041-1046 (doc :1037-1040) — unchanged
  by fmt.
- `codex-rs/core/tests/suite/apply_patch_cli.rs` (2625 lines):
  :8 `use …ev_apply_patch_function_call`; :55 `use …ResponseMock`
  (fmt-sorted); `mount_apply_patch_function_call` :248-259; section
  comment + `vllm_apply_patch_harness` :2504-2515; T4.1
  `apply_patch_cli_function_tool_served_to_non_openai_provider`
  :2517-2560; T4.2 `apply_patch_cli_function_tool_applies_f1_shaped_raw_
  add_file` :2562-2592; T4.2b
  `apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file`
  :2594-2625.

**Final numstat (owned files, vs HEAD; purely additive):**

```text
11   0  codex-rs/core/tests/common/responses.rs
138  0  codex-rs/core/tests/suite/apply_patch_cli.rs
```

No product-code diff (seam files and AGENTS.md untouched by this item;
`git status` over `codex-rs/` shows exactly the pre-existing seam
modifications plus the two owned test files).

## Discrepancy notes

**D1 — Test names changed mid-item (rationale corrected at r1 review,
seat A F3/F5).** Initial names `apply_patch_function_tool_*` WOULD have
matched the nextest group filter
`'package(codex-core) & kind(test) & test(apply_patch_cli)'`
(`codex-rs/.config/nextest.toml` → `core_apply_patch_cli_integration`,
`max-threads = 1`) via the module path — `test()` matches the full test
identifier, which contains `suite::apply_patch_cli::` (proven by
`cargo nextest list` under the exact filter). Run 1's TMTs were
therefore host-load contention, not group exclusion (and the group
override is inactive under `NEXTEST_PROFILE=local` anyway — profile
inheritance does not carry the override, nextest 0.9.137). Renamed to
`apply_patch_cli_function_tool_*` to match the file's dominant
behavioral-test naming convention; the rename is 3 lines (fn names
only). The breakdown specified no test names; the spec T4 plan
specifies no names either.

**D2 — Full-scope gate could not complete a clean 118/118 pass on this
host (machine load, not the diff).** Run 1: exit 100, 0 deterministic
failures, all 79 panic lines `lib.rs:388` (binding check clean) —
~30 pre-existing tests TMT'd/failed in the same class. Run 2 (post-
rename): stopped by the worker at 87/118 (48 passes, binding check
clean over all captured panics, 122 TRY-level TMT/FAIL lines, all
`lib.rs:388`) to remove self-contention from the focused verification
(run 2 and the focused runs were executing the SAME serialized tests
simultaneously). The pre-existing apply_patch tests' non-passes in
both full runs are the same load class the item-3 evidence recorded at
baseline ("17 flaky, 15 timed out"; load 15-35 here vs the item-3
window). The three NEW tests' green proof is the per-test 1/1 focused
runs (all `1 test run: 1 passed`), which is strictly stronger
evidence for them than a mixed full run under saturation would be.
Full-run non-pass identities (run 1, all `lib.rs:388`-class; recorded
per the task's "record all non-pass identities"):
`suite::apply_patch_serialization::{apply_patch_custom_tool_call_
reports_failure_output, apply_patch_custom_tool_call_updates_existing_
file, apply_patch_custom_tool_call_creates_file}`;
`suite::approvals::{approval_matrix_covers_group::apply_patch,
approving_apply_patch_for_session_skips_future_prompts_for_same_file}`;
`suite::hooks::{permission_request_hook_allows_apply_patch_with_write_
alias, pre_tool_use_blocks_apply_patch_before_execution,
post_tool_use_records_additional_context_for_apply_patch,
pre_tool_use_rewrites_apply_patch_before_execution,
post_tool_use_records_apply_patch_context_with_edit_alias,
pre_tool_use_blocks_apply_patch_with_write_alias}`;
`suite::code_mode::code_mode_can_apply_patch_via_nested_tool`;
`suite::apply_patch_cli::{apply_patch_aggregates_diff_across_multiple_
tool_calls, apply_patch_cli_add_overwrites_existing_file (run 2),
apply_patch_cli_multiple_operations_integration, apply_patch_cli_
preserves_distinct_updated_paths, apply_patch_cli_rejects_move_path_
traversal_outside_workspace, apply_patch_cli_updates_file_appends_
trailing_newline, apply_patch_cli_verification_failure_has_no_side_
effects, apply_patch_custom_tool_streaming_emits_updated_changes,
apply_patch_cli_rejects_empty_patch (TMT), apply_patch_cli_rejects_
invalid_hunk_header (TMT), apply_patch_preserves_crlf_with_preserve_
line_endings_feature, apply_patch_shell_heredoc_normalizes_crlf_
without_preserve_line_endings_feature (TMT), apply_patch_shell_heredoc_
preserves_crlf_with_preserve_line_endings_feature (TMT),
apply_patch_emits_turn_diff_event_with_unified_diff (TMT),
apply_patch_exec_command_failure_propagates_error_and_skips_diff
(TMT), apply_patch_exec_command_heredoc_with_cd_emits_turn_diff (TMT),
apply_patch_exec_command_heredoc_with_cd_updates_relative_workdir
(TMT), apply_patch_turn_diff_paths_stay_repo_relative_when_session_cwd_
is_nested (TMT), apply_patch_turn_diff_skips_git_root_when_feature_is_
enabled::mobile_work_uses_cwd_when_enabled (TMT),
apply_patch_change_context_disambiguates_target (TMT),
apply_patch_cli_multiple_chunks, apply_patch_cli_does_not_widen_
permissions_for_workspace_directory_target (TMT, run 2),
apply_patch_cli_can_use_exec_command_output_as_patch_input (TMT, run
2), apply_patch_clears_aggregated_diff_after_inexact_delta (FAIL, run
2), apply_patch_normalizes_crlf_without_preserve_line_endings_feature
(flaky→PASS run 2)}`; plus the non-`apply_patch_cli` suites in scope:
`suite::unified_exec::unified_exec_intercepts_apply_patch_exec_command
(TMT×2 both runs)`, `suite::tool_harness::{apply_patch_tool_executes_
and_emits_patch_events, apply_patch_reports_parse_diagnostics} (TMT×2
both runs)`, `suite::shell_snapshot::unified_exec_snapshot_still_
intercepts_apply_patch (TMT×2 both runs)`, `suite::request_
permissions::denied_child_permissions_require_fresh_approval::{
apply_patch_session, apply_patch_turn} (TMT×2 both runs)`,
`suite::request_permissions_tool::approved_folder_write_request_
permissions_unblocks_later_apply_patch::{with_strict_auto_review,
without_strict_auto_review} (TMT×2 both runs)`,
`suite::prompt_caching::gpt_5_tools_without_apply_patch_append_apply_
patch_instructions (TMT×2 both runs)`. Every identity's only panic
line(s) are `lib.rs:388` `timeout waiting for event: Elapsed(())`.

**D3 — Temporary eprintln for the wire capture (injected, then
removed).** One line
`eprintln!("T41-WIRE-CAPTURE {apply_patch_tool}");` was added into
T4.1 to capture the real request-body tool object (evidence §"T4.1
wire-body capture"), the solo run was taken with `-- --nocapture`
(PASS 0.292s, `/tmp/item4-wire-capture2.log`), and the line was
deleted; post-removal `git diff` re-verified purely additive (139
insertions/0 deletions pre-fmt; 149/0 post-fmt). Side effect:
`just fix -p codex-core` (gate 2) compiled the test target while that
line was present; clippy reported NO fix in `apply_patch_cli.rs`
(only the openai_file_mcp.rs fix, D4), so the fix gate's verdict over
the owned files is unaffected; per repo convention no test re-run was
done after `fix`/`fmt`.

**D4 — `just fix` re-triggered the known out-of-scope auto-fix.**
`Fixed core/tests/suite/openai_file_mcp.rs (1 fix)` — exactly the
hazard named in the task brief. Restored with `git checkout --
codex-rs/core/tests/suite/openai_file_mcp.rs`; verified clean vs HEAD
afterwards (empty `git diff HEAD --stat`).

**D5 — The tests are green on the working tree (HEAD + uncommitted
seam), and would fail on bare HEAD.** The function-tool capability
(`provider.rs:372 apply_patch_function_tool: !is_openai()`), the
registration (`spec_plan.rs:1257-1269` +
`handlers/mod.rs pub use`) are seam (uncommitted) lines;
`FunctionApplyPatchHandler` itself is committed. (r1 seat B nit: an
earlier version also listed the `model_info.rs` default as a dependency
— it is not load-bearing here: it is the fallback for unknown slugs,
and the harness model `gpt-5.5` resolves from committed `models.json`.)
This is by design of the campaign's execution baseline (breakdown §0):
the coordinator commits seam + this item together, so the committed
state will be green. Bare-HEAD T4.1 would fail on the
`"type": "function"` assertion (freeform tool served instead);
T4.2/T4.2b would fail on the registry kind-mismatch
("tool apply_patch invoked with incompatible payload",
`registry.rs:550`) since only the custom-tool handler is registered
at bare HEAD.

**D6 — Test-count note.** The scoped gate run 1 executed 118 tests
(`just test -p codex-core apply_patch`), vs the 112 recorded in the
item-3 baseline; the delta closes as 118 = 112 (item-3's baseline,
which already contained item-2's spec tests —
`tools::handlers::apply_patch_spec::tests::create_apply_patch_function_
tool_*`, all PASS in run 1 — and 4 uncommitted seam-era tests in
`apply_patch_tests.rs`) + 3 item-3-own tests (T3.1 ×2 + T3.2) + 3
item-4 tests. (Arithmetic corrected at r1 review — seat A F4 / seat B
m1: the earlier version cited the item-2 spec tests as part of the
delta, but they were already in the 112. Note: `git log -S` attributes
7 in-scope tests to item 3's commit because the 4 seam-era tests get
their first COMMIT there, though they predate it in the working tree.)

## Worker attestation

- Owned files edited: `codex-rs/core/tests/common/responses.rs`,
  `codex-rs/core/tests/suite/apply_patch_cli.rs`,
  `docs/reviews/impl-item4-tdd-evidence.md` (new). Nothing else.
- No `git add`/`commit` executed by this worker; tree left
  uncommitted for the coordinator's review loop.
- No product-code changes; no seam/AGENTS.md touch; the
  `openai_file_mcp.rs` clippy side-effect was reverted and verified.
