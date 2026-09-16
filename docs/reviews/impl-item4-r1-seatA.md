# Item 4 — Integration (spec T4): Round 1 Review, SEAT A (apex-ayl.52)

Seat: A (spec/semantics + integration-correctness lens). Reviewer:
`/root/item4_review_seat_a`, 2026-09-16 EDT. Independent of seat B
(conventions/minimality/evidence lens); no coordination.

Branch: `feat/normalize-content-types-vllm`, working-tree HEAD
`2a25e69a46` (post Items 1-3: `e21f608ac4` P2, `a4d5af1f62` P1,
`939a6dc6f4` P3). Campaign function-tool seam remains uncommitted
working-tree state per breakdown §0 execution baseline. SoT: spec v5
`docs/responses-compat-apply-patch-format.md` (§4 T4, §3.1, §1.1, §3.2);
breakdown v3 `docs/responses-compat-apply-patch-task-breakdown.md`
(Item 4, §0, §5); worker evidence
`docs/reviews/impl-item4-tdd-evidence.md`.

Review scope: the two owned test files
(`codex-rs/core/tests/common/responses.rs` +11,
`codex-rs/core/tests/suite/apply_patch_cli.rs` +138, both purely
additive) and the new evidence doc. Read-only elsewhere; all product
files, the seam, and committed history verified by reading, never
edited.

## Method

- Read all named spec/breakdown/evidence sections and every cited seam
  line; re-derived the routing chain, the F1 payload shape, and the
  helper's E0308 rationale from source rather than trusting the
  evidence doc's claims.
- Independently re-measured the P1 const (length, all 7 §3.1 drift
  substrings) and parsed the worker's wire-capture log
  (`/tmp/item4-wire-capture2.log`) as JSON and compared the captured
  tool object field-by-field against spec §3.1 and the committed const.
- Re-ran the worker's binding checks against the worker's own on-disk
  logs (`/tmp/item4-gate1.log`, `/tmp/item4-gate1-run2.log`, the four
  focused logs, the three solo logs) — all still present.
- Ran the three item gates myself from `codex-rs/`
  (`just test -p codex-core apply_patch` to completion, then
  `just fix -p codex-core`, then `just fmt`) and recorded results
  below.
- Empirically probed nextest group behavior (`cargo nextest list` with
  the exact group filter expression; controlled two-test runs under
  `NEXTEST_PROFILE=local` vs `NEXTEST_PROFILE=default`) to adjudicate
  D1 and the serialization question — findings in Mandate 5 and the
  D1/D2 adjudications.

## Mandate 1 — T4.1 correctness (wire shape on the renamed provider)

**Harness routing — re-derived from source (not taken from evidence):**

- `vllm_apply_patch_harness` (apply_patch_cli.rs:2509-2515) renames
  exactly one field: `config.model_provider.name = "vLLM"`.
- `TestCodexBuilder::with_config` (test_codex.rs:347-353) pushes a
  `FnOnce(&mut Config)` onto the builder's per-instance
  `config_mutators` vec; `prepare_config` sets
  `config.model_provider` from the built-in clone
  (`..built_in_model_providers(None)["openai"].clone()`,
  test_codex.rs:840-844, name "OpenAI" per
  model-provider-info/src/lib.rs:465) and the mutators run strictly
  AFTER that assignment (test_codex.rs:878-882). So the rename hits the
  final config's `model_provider.name` — the exact field
  `is_openai()` reads (`self.name == OPENAI_PROVIDER_NAME`, exact
  case-sensitive match, model-provider-info/src/lib.rs:546-547, const
  :40). "vLLM" != "OpenAI" → `is_openai()` false.
- Capability (seam): `apply_patch_function_tool: !self.info.is_openai()`
  (model-provider/src/provider.rs:372) → true for the renamed provider.
- Registration (seam): `spec_plan.rs:1257` gate
  `environment_mode.has_environment() &&
  context.model_info.apply_patch_tool_type.is_some()`; branch
  :1260-1264 on `capabilities().apply_patch_function_tool`;
  `registry.add(FunctionApplyPatchHandler::new(include_environment_id))`
  :1267. The model-info selector is satisfied independently of the
  provider rename: harness default model `gpt-5.5`
  (test_codex.rs:858) carries `apply_patch_tool_type: "freeform"` in
  the COMMITTED models.json (verified at both `git show
  HEAD:codex-rs/models-manager/models.json` and the working tree →
  `Some(Freeform)`), so the provider name is indeed the only thing
  selecting the function path. The other claim in the mandate — that
  `gpt-5.5`'s `apply_patch_tool_type` is "the other selector" and the
  rename is what routes — is confirmed: at bare HEAD the same renamed
  provider registers only `ApplyPatchHandler` (custom-tool; HEAD
  spec_plan.rs:1256-1258 has no capability branch and HEAD
  `ProviderCapabilities` has no `apply_patch_function_tool` field —
  verified via `git show HEAD:`), so T4.1's `type == "function"`
  assertion would fail at bare HEAD (freeform `type: "custom"` served
  instead). See D5.
- Handler side: `FunctionApplyPatchHandler::matches_kind` accepts only
  `ToolPayload::Function` (apply_patch.rs:607-609); the served spec
  comes from committed P1 `create_apply_patch_function_tool`.

**Wire assertions in the test (apply_patch_cli.rs:2517-2560):**

- Asserted against the LIVE captured request body:
  `mock.requests().first().body_json()` — `ResponseMock::requests()`
  (responses.rs:58-60) clones the wiremock-captured request vec; the
  test inspects the first `/v1/responses` POST (the turn posts twice —
  tool call, then `function_call_output` follow-up — so
  `single_request()` would panic; using `.first()` is correct and
  matches the evidence doc). Not a re-implemented expectation.
- Tool name "apply_patch": selected via
  `tools.iter().find(|tool| tool["name"] == "apply_patch")` — the name
  is verified by construction of the lookup (expect-panics if absent).
- `type: "function"`: `assert_eq!(apply_patch_tool["type"],
  json!("function"))` — the routing discriminator (freeform would be
  `"custom"`). Confirmed present.
- `patch` parameter description: read via
  `.pointer("/parameters/properties/patch/description")` + `as_str`
  (expect-panics if absent/non-string), then asserts containment of ALL
  7 §3.1 drift substrings, verbatim list at
  apply_patch_cli.rs:2538-2546, identical to the T2.1 drift-guard list
  (apply_patch_spec_tests.rs:97-103). All 7 present; I also
  independently verified all 7 against the committed const and against
  the captured wire text (see below).
- `strict: false` and `required: ["patch"]` are NOT asserted by the
  test. The spec T4.1 requirement (§4) is "function tool whose `patch`
  parameter description contains the §3.1 substrings" — satisfied.
  The full shape including `strict: false`,
  `required: ["patch"]`, `additionalProperties: false` is pinned by the
  committed full-snapshot spec test
  `create_apply_patch_function_tool_matches_expected_spec`
  (apply_patch_spec_tests.rs:46-70, deep equality on the whole
  `ToolSpec`), which passed in my gate run, and is observed in the
  worker's live wire capture (below). Classified as a minor gap
  relative to this mandate's fuller expectation list — see Findings.

**Worker's wire-capture excerpt — independent verification:**

Parsed `/tmp/item4-wire-capture2.log` (worker's T41-WIRE-CAPTURE line,
still on disk) as JSON:
`description` (tool-level) 126 chars; `name` "apply_patch";
`parameters.properties.patch.type` "string";
`parameters.properties.patch.description` 2,198 chars — byte-identical
to the committed `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION`
const (measured 2,198 from apply_patch_spec.rs); all 7 §3.1 substrings
present in the captured text; `required: ["patch"]`;
`additionalProperties: false`; `strict: false`; `type: "function"`.
The excerpt in the evidence doc matches the raw capture (middle of the
description elided as documented). Verified, not merely quoted.

## Mandate 2 — T4.2 / T4.2b correctness (F1 behavior end-to-end)

**F1 shape vs spec §1.1:** spec §1.1 F1 = raw markdown Add-File — the
model's content lines carry no per-line `+` prefix and the first
content line is the H1 (the rollout's exact first lines:
`*** Add File: grok/plans/spec-freeze-r23-glm.md` +
`# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)`,
em-dash included).

- T4.2 patch (apply_patch_cli.rs:2573-2580) uses exactly that file
  path and H1 line, plus raw content lines (text line, blank line,
  two markdown table lines) and no `+` prefixes on any content line.
  It is byte-for-byte the payload of the committed T3.2 test
  `function_apply_patch_applies_f1_shaped_raw_add_file_patch`
  (apply_patch_tests.rs:509-558) — "F1-shaped" is the contract and it
  holds.
- T4.2b patch (apply_patch_cli.rs:2607-2612) is the same shape
  (Begin / Add File / raw H1 + raw content / End) targeting an
  existing file.
- T4.2 targets a NEW file: the harness cwd is a fresh per-test
  TempDir (test_codex.rs `prepare_config`); T4.2 performs no
  `write_file` seed. T4.2b seeds `overwrite-me.md` with DIFFERENT
  known contents (`harness.write_file`, apply_patch_cli.rs:2598-2603,
  harness method test_codex.rs:1178) and asserts the final contents
  are exactly the patch's — pinning the no-existence-check/overwrite
  decision (spec §3.2, T1.9 outcome) end-to-end.
- Both assert byte-exact FULL contents via
  `harness.read_file_text` (test_codex.rs:1209-1217) +
  `pretty_assertions::assert_eq` (whole-string equality, not
  substring), expected literals at apply_patch_cli.rs:2585-2590 and
  :2619-2622.

**Full stack exercised:** renamed non-OpenAI provider → function tool
served → SSE `function_call` event (see below) → registry dispatch →
`FunctionApplyPatchHandler` (matches_kind `ToolPayload::Function`,
apply_patch.rs:607-609) → `handle_call` parses `{"patch": …}`
(apply_patch.rs:511-549) → shared `run_apply_patch_text` → streaming
parser with Item-1 P2 Add-File leniency → sandboxed fs write. A
failure at any link would fail the file assertion. Confirmed.

**SSE mock event shape:** `mount_apply_patch_function_call`
(apply_patch_cli.rs:248-258) is an exact mirror of `mount_apply_patch`
(:228-239) through the shared `apply_patch_responses` helper
(:257-271): first body = `response.created` + the tool-call event +
`response.completed`; second body = assistant message +
`response.completed` — identical to the custom-tool path's two-body
shape. The only difference is the event constructor:
`ev_apply_patch_function_call` emits item `type: "function_call"`
(responses.rs:1043-1050 via `ev_function_call` :933-943) — the event
kind the function handler consumes (`ToolPayload::Function`), in
contrast to `mount_apply_patch`'s `custom_tool_call` item, which is
rejected on the renamed provider by the registry kind check
(registry.rs:549-550, "tool apply_patch invoked with incompatible
payload") since only the function handler is registered there. The
`custom_tool_call` vs `function_call` distinction is therefore exactly
right, and the breakdown's "do not reuse `mount_apply_patch`"
constraint is honored (zero changes to the shared helper or existing
callers).

## Mandate 3 — Helper correctness (`ev_apply_patch_function_call`)

- Location/shape (responses.rs:1037-1046): inserted between
  `ev_apply_patch_exec_command_call_via_heredoc` and `sse_failed`,
  next to the sibling pattern helpers, per breakdown step 2. The
  serialize-then-call shape mirrors
  `ev_apply_patch_exec_command_call_via_heredoc` (:1030-1035) verbatim
  including the `expect("serialize apply_patch arguments")` message
  and the blank line before the final call; doc comment uses the
  family's "Convenience: SSE event for …" phrasing
  (cf. `ev_apply_patch_custom_tool_call` :1005-1007).
- E0308 rationale re-derived: `ev_function_call`'s third parameter is
  `arguments: &str` (responses.rs:933);
  `serde_json::to_string(…)` returns an owned `String`, which does not
  coerce to `&str` at an argument position — the one-line form does not
  compile (E0308). The local bind + `&arguments` is the siblings'
  established solution. Confirmed correct.
- Argument JSON is exactly `{"patch": <patch>}`
  (responses.rs:1043 `serde_json::json!({ "patch": patch })`) — the
  shape `FunctionApplyPatchHandler::handle_call` expects
  (`value.get("patch")` then `as_str()`, apply_patch.rs:537-549;
  absent → P3.3a message, non-string → P3.3b message, both committed
  in Item 3 and green in my run).
- `pub` inside the test-support module matches all 37 sibling `pub
  fn ev_*`s; the crate is the test-support surface, not product API
  (breakdown step 2 prescribes exactly this signature; §5's "no new
  public API surface" invariant targets product API).

## Mandate 5 — Isolation and serialization

**Provider-rename isolation:** `with_config` mutators live on the
per-builder `config_mutators` vec (test_codex.rs:329, :351) and are
applied inside that builder's `prepare` (test_codex.rs:878-882).
`vllm_apply_patch_harness` builds a FRESH `test_codex()` builder per
call (via `apply_patch_harness_with`, apply_patch_cli.rs:89-97:
`let builder = configure(test_codex())`), so the "vLLM" rename is
instance-local; no shared/global state is touched. The pattern is
established suite-wide (e.g. compact.rs:1313, compact_resume_fork.rs:626,
guardian_review.rs:201, responses_lite.rs:392/470/522,
pending_input.rs:1429, client.rs:2514 — all rename
`config.model_provider.name` via the same mechanism inside their own
builders). T4.1's harness is not the first renamed-provider test in the
suite, but it is the first to rename to a name chosen for this
campaign; the mechanism is identical. No leakage possible: each of the
three tests constructs its own harness, and the three tests'
`TempDir`s, mock servers (wiremock on ephemeral ports), and configs are
disjoint.

**Nextest serialization group:** config verified at
codex-rs/.config/nextest.toml: `[test-groups.core_apply_patch_cli_integration]
max-threads = 1` (:25-26) with override filter
`package(codex-core) & kind(test) & test(apply_patch_cli)`
(:66-68). All three new tests' fully-qualified names
(`suite::apply_patch_cli::apply_patch_cli_function_tool_*`) contain
`apply_patch_cli` — in the function name AND in the module path — so
all three fall in the group. Verified empirically:
`cargo nextest list -p codex-core -E '<exact group filter>'` enumerates
all three (and, notably, existing module tests whose function names
lack the prefix, e.g.
`suite::apply_patch_cli::apply_patch_aggregates_diff_across_multiple_tool_calls` —
proving `test()` matches the full path as a substring, not just the
function name). Consequence for D1: the ORIGINAL names
(`apply_patch_function_tool_*`) would ALSO have matched the group via
the module path; the rename was not required for group membership (see
D1 adjudication and the local-profile finding below).

**Local-profile finding (context for Mandate 4; not an Item 4 defect):**
the group's `max-threads = 1` is NOT effective under
`NEXTEST_PROFILE=local` (what `just test` uses). Evidence:
(a) worker run 1 (`/tmp/item4-gate1.log`) and run 2
(`/tmp/item4-gate1-run2.log`): all 37 TMT'd tests' TRY-2 terminations
land within a ~0.5 s window at the 60 s slow-timeout boundary — i.e.
they all started at run start and ran concurrently; under max-threads=1
matched tests would terminate minutes apart; (b) my own full gate run
replicates the identical simultaneous-TMT pattern for matched tests
(e.g. `suite::apply_patch_cli::apply_patch_aggregates_diff_across_multiple_tool_calls`
TMT TRY 1 at 61.659 s while sibling
`suite::apply_patch_cli::apply_patch_aggregates_diff_preserves_success_after_failure`
TMT TRY 1 at 60.237 s — two group-matched tests running concurrently);
(c) controlled experiment: the two matched tests
`apply_patch_cli_rejects_empty_patch` +
`apply_patch_cli_end_of_file_anchor` under `NEXTEST_PROFILE=default`
took 241.3 s wall for 2×(60+60) s of test time — serialized, i.e. the
group IS active under the default profile; under `local` (with
`RUST_MIN_STACK` set) the same two tests abort/run concurrently. Root
cause: `[profile.local] inherits = "default"` does not carry the
`[[profile.default.overrides]]` `test-group` assignment into the local
profile (nextest 0.9.137). The group's stated purpose (nextest.toml
comment: "sensitive to Windows runner process-startup stalls when many
cases launch at once") is CI-facing, and CI does not set
NEXTEST_PROFILE=local — so CI serialization is intact; LOCAL `just
test` runs of this suite get full-pool parallelism, which is the
dominant cause of the load-TMT noise this campaign's gates document.
Pre-existing infrastructure behavior; Item 4 neither introduced nor
could fix it (it owns no config files). Recorded for the record; see
Mandate 4 for its effect on gate-sufficiency.

## Mandate 7 — No product code

`git status` + `git diff --numstat` at review start (11:50 EDT) and
re-checked after the gates: exactly 14 modified files = the pre-existing
campaign seam (AGENTS.md +116; codex-api content_type_compat.rs +70,
content_type_compat_tests.rs +133, endpoint/responses.rs +2; core
handlers/mod.rs +1, spec_plan.rs +13/-1; model-provider provider.rs +32,
amazon_bedrock/mod.rs +3; models-manager models.json +374/-18,
manager_tests.rs +50, model_info.rs +2/-1, model_info_tests.rs +13) +
Item 4's two owned files (tests/common/responses.rs +11/-0,
tests/suite/apply_patch_cli.rs +138/-0, both purely additive — verified
hunk-by-hunk, zero deletions) + untracked docs (spec, seam doc, review
reports, worker evidence, research doc). No product file was modified
by Item 4 beyond the two owned test files; the `openai_file_mcp.rs`
clippy side-effect (known hazard) was restored by the worker and the
file was clean vs HEAD at review start (its committed unused-import
warning reappears in my build output — see Gate 2). Any post-gate
drift is recorded under Mandate 6.

## Discrepancy adjudications (D1–D6)

**D1 (test names changed mid-item) — adjudicated: rename acceptable,
stated rationale incorrect.** The initial names
`apply_patch_function_tool_*` WOULD have matched the nextest group
filter: `test()` matches the fully-qualified test path as a substring,
and the module path `suite::apply_patch_cli::` already contains
`apply_patch_cli`. Proven empirically: `cargo nextest list -p
codex-core -E 'package(codex-core) & kind(test) & test(apply_patch_cli)'`
enumerates existing module tests whose function names lack the prefix
(e.g. `apply_patch_aggregates_diff_across_multiple_tool_calls`), so the
original names were in the group exactly as the final names are — group
membership did not change with the rename. (And per the local-profile
finding in Mandate 5, the group is inactive under local `just test`
either way; it is active under the default/CI profile for both name
forms.) The rename nonetheless follows the file's dominant behavioral
naming convention and is three lines — harmless and mildly
conforming. No finding against the committed state; the rationale
recorded in the evidence is corrected here (nit).

**D2 (full-scope gate could not complete clean 118/118) —
adjudicated: load, not the diff.** Independently re-verified from the
worker's on-disk logs: run 1 completed (118 tests; 73 passed (3 flaky),
8 failed, 37 timed out) with the binding check EMPTY over all 79 panic
lines (every one `core/tests/common/lib.rs:388` `timeout waiting for
event: Elapsed(())`); run 2 was stopped at 88/118 (48 passed (2
flaky), 3 failed, 37 TMT) — the "87/118" in the evidence is off by one
test vs the log's own Summary line, immaterial. The 3 new tests' only
failure mode in every attempt across all runs is the lib.rs:388-class
TMT; no assertion failure, no panic outside lib.rs:388, in any log.
The pre-existing tests' non-passes are the same class the item-3
baseline recorded (17 flaky / 15 TMT on 112 tests). Confirmed load
noise. See Mandate 4 for the sufficiency adjudication.

**D3 (temporary eprintln for wire capture) — adjudicated: verified
clean.** `grep -c T41-WIRE-CAPTURE` over the final
apply_patch_cli.rs = 0; the file's diff vs HEAD is purely additive
(+138/-0) and contains no capture instrumentation. The captured value
was independently parsed and verified (Mandate 1). The side effect on
the fix gate is neutral (clippy reported no fix in apply_patch_cli.rs
— consistent with the final file passing clippy; my Gate 2 re-confirms).

**D4 (`just fix` re-triggered the out-of-scope auto-fix) —
adjudicated: known hazard, correctly handled.** Confirmed the hazard
is real and pre-existing: the committed openai_file_mcp.rs:47 carries
an unused `wiremock::matchers::body_json` import — the warning
reappears in my own build output at review start, i.e. before any gate
of mine ran. The worker restored the file via `git checkout --` and it
was clean vs HEAD at review start. My Gate 2 will re-trigger the same
fix; restoration + verification is recorded under Mandate 6.

**D5 (green on working tree, would fail on bare HEAD) — adjudicated:
NOT a defect; designed incremental-seam pattern under §0.** The
§0 execution baseline governs: "At HEAD" = working-tree state, and the
coordinator commits seam + item together, so the committed state is
green. The tests CANNOT be written to avoid depending on the seam: the
behavior under lock — a non-OpenAI-named provider being served the
function form of apply_patch and dispatching function payloads to
`FunctionApplyPatchHandler` — IS the seam (capability
provider.rs:372, registration spec_plan.rs:1257-1270, pub use
handlers/mod.rs). At bare HEAD, T4.1 would fail on the
`type == "function"` assertion (freeform `custom` tool served — HEAD
registers only `ApplyPatchHandler` with no capability branch), and
T4.2/T4.2b would fail on the registry kind-mismatch rejection
(registry.rs:549-550) since only the custom-tool handler
(`matches_kind` Custom, apply_patch.rs:655-657) exists there. These are
regression locks over the campaign's own change set (Items 1-3 + seam),
committed as one unit; writing them any other way would defeat their
purpose. Verified mechanics at every cited line (Mandate 1). Acceptable
by design; no action.

**D6 (test-count note 112 → 118) — adjudicated: count correct,
attribution imprecise.** 118 verified independently
(`cargo nextest list -p codex-core apply_patch` = 118; my gate run
"Starting 118 tests"). The delta reconciles exactly as: 112 (item-3
baseline, measured on the working tree that already contained 4
uncommitted seam tests in apply_patch_tests.rs — the item-3 commit
itself notes the seam in those files was "inseparable within the same
files") + 3 item-3-own new in-scope tests
(`function_apply_patch_rejects_missing_patch_argument_with_teachable_error`,
`function_apply_patch_rejects_non_string_patch_argument_with_teachable_error`,
`function_apply_patch_applies_f1_shaped_raw_add_file_patch`; the other
4 tests added by the item-3 commit — `with_environment_id_line_*` ×2,
`function_apply_patch_patch_text_*` ×2 — were the pre-existing seam
tests already inside the 112) + 3 Item-4 tests = 118. The evidence's
example attribution ("Items 1-3's own added tests … e.g.
`create_apply_patch_function_tool_*`") is imprecise: those item-2 spec
tests were already in the 112 baseline. Nit.

## Mandate 4 — Gate-sufficiency adjudication (worker's partial full gate)

**Record (all independently re-verified from on-disk logs):**

- Run 1 (full 118, pre-rename names): COMPLETED. Exit 100; 73 passed
  (3 flaky), 8 failed, 37 timed out. Binding check over the full log:
  EMPTY (79/79 panic lines at `core/tests/common/lib.rs:388`). All 3
  new tests TMT'd on both tries, lib.rs:388 class only. All 13
  non-pass identities are pre-existing tests (list in Gate 1 below).
- Run 2 (full 118, post-rename names): STOPPED by the worker at
  88/118 (log Summary: 48 passed (2 flaky), 3 failed, 37 TMT) to remove
  self-contention with the focused verification runs. Binding check
  clean over all captured panics; the 3 renamed tests TMT'd (load
  class) among the first 78 completions.
- Focused 3-test runs (3 and 4): every test green at least once
  (T4.1 ×2, T4.2 ×3, T4.2b ×1); the non-passes in those runs are
  lib.rs:388-class TMTs on the OTHER tests.
- Solo per-test 1/1 runs: T4.1 0.298 s, T4.2 0.268 s, T4.2b 0.257 s —
  all PASS, binding-clean. (The solo condition is strictly stronger
  than the CI serialization condition for the individual test: no
  same-group peers, no pool contention.)
- Load context: loadavg 15-35 at item start, 6.7-9.5 during the runs
  (evidence), with a sibling worker running `cargo test --release`
  concurrently; my review window shows sustained loadavg ~5-13 with
  seat B running the same gate concurrently.

**Adjudication:** The documented green criterion for this item is
"0 DETERMINISTIC failures" (binding grep empty; non-pass identities
recorded; the 3 new tests PASS) — not "118/118 pass", which is
unachievable under the documented load (the item-3 baseline on the
same host: 17 flaky / 15 TMT on 112 tests). Against that criterion,
the worker's record is complete in substance:

1. A COMPLETED full-scope run (run 1) exists, binding-clean, covering
   all 118 tests including all apply_patch-adjacent tests.
2. The only change between run 1 and the committed state is the
   3-line test RENAME — behavior-neutral, and (per Mandate 5/D1)
   group-membership-neutral. It cannot alter any other test's outcome,
   so the 30 tests not re-run in run 2 are covered by run 1 by
   construction.
3. Positive proof for each new test exists under the strongest
   available condition (solo 1/1, sub-0.3 s each).

A completed post-rename full run would add only more load-class TMTs on
pre-existing tests (the signal it could add — deterministic failures
among the 3 new or adjacent tests — is already ruled out by run 1 +
binding-clean + solo greens). The marginal value under documented load
is ~zero; the cost is another 30-90 min of a shared host.

**Classification: the early stop is `minor`** (a documented, reasoned
deviation from the item's gate recipe, compensated by stronger
per-test evidence; the committed state's gate standing is finalized by
my reviewer's COMPLETED full run, Mandate 6 — if that run is
binding-clean and the 3 new tests PASS in it, the gate is satisfied and
this finding carries no residual risk). It is not Major: no assertion
or binding evidence is missing, and the stop is fully disclosed in the
evidence doc with the reason (self-contention), which is legitimate
under the documented load conditions.

## Mandate 6 — Gates (re-run by this reviewer, from `codex-rs/`)

**Gate 1 — `just test -p codex-core apply_patch` (run to COMPLETION by this
reviewer; started 12:18 EDT, `NEXTEST_PROFILE=local`,
`RUST_MIN_STACK=8388608`; loadavg ~5-13 over the window with seat B
running the same gate concurrently):**

```text
    Starting 118 tests across 3 binaries (4251 tests skipped)
     Summary [4762.650s] 118 tests run: 58 passed (3 flaky), 4 failed, 56 timed out, 4251 skipped
error: test run failed        (exit 100 — all non-passes in the load class)
```

- 79.4 min wall — inside the documented 30-90 min band. 118 tests —
  matches D6.
- **Binding check (green criterion): EMPTY.**
  `grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u`
  → 0 lines; 109/109 panic lines in the log are
  `core/tests/common/lib.rs:388` `timeout waiting for event:
  Elapsed(())`. **0 deterministic failures.**
- The 3 new tests in the full run: T4.2
  `apply_patch_cli_function_tool_applies_f1_shaped_raw_add_file`
  FLAKY 2/2 → **PASS** (TRY 1 failed at 52.5 s on the lib.rs:388
  event-wait timeout — the test's internal ~50 s timeout fired before
  nextest's 60 s kill; TRY 2 PASS 0.407 s). T4.1
  `apply_patch_cli_function_tool_served_to_non_openai_provider` and
  T4.2b `apply_patch_cli_function_tool_raw_add_file_overwrites_existing_file`
  TMT'd on both tries (60 s-class, lib.rs:388 only) under the
  concurrent-reviewer load.
- **Solo re-runs (reviewer's, the documented per-test pattern; the
  load had cleared): ALL THREE PASS 1/1** —
  T4.1 `PASS [0.612s] (1/1)`; T4.2 `PASS [0.297s] (1/1)`;
  T4.2b `PASS [0.381s] (1/1)` (14:07 EDT; logs in session output).
  This completes the green criterion: 0 deterministic failures +
  all 3 new tests PASS in this reviewer's run.
- Final non-pass identities (60 = 4 failed + 56 TMT; 3 more flaky→pass;
  every one pre-existing except T4.1/T4.2b whose solo PASSes are
  recorded above; every panic line in the log is lib.rs:388):
  `suite::apply_patch_cli::{apply_patch_aggregates_diff_across_multiple_tool_calls,
  apply_patch_aggregates_diff_preserves_success_after_failure,
  apply_patch_change_context_disambiguates_target,
  apply_patch_clears_aggregated_diff_after_inexact_delta,
  apply_patch_cli_add_overwrites_existing_file,
  apply_patch_cli_can_use_exec_command_output_as_patch_input,
  apply_patch_cli_delete_directory_reports_verification_error,
  apply_patch_cli_delete_missing_file_reports_error,
  apply_patch_cli_does_not_widen_permissions_for_workspace_directory_target,
  apply_patch_cli_end_of_file_anchor,
  apply_patch_cli_insert_only_hunk_modifies_file,
  apply_patch_cli_missing_second_chunk_context_rejected,
  apply_patch_cli_move_overwrites_existing_destination (FAIL),
  apply_patch_cli_move_without_content_change_has_no_turn_diff,
  apply_patch_cli_moves_file_to_new_directory,
  apply_patch_cli_multiple_chunks,
  apply_patch_cli_multiple_operations_integration,
  apply_patch_cli_preserves_distinct_updated_paths,
  apply_patch_cli_rejects_duplicate_resolved_paths,
  apply_patch_cli_rejects_empty_patch,
  apply_patch_cli_rejects_invalid_hunk_header,
  apply_patch_cli_rejects_move_path_traversal_outside_workspace (FAIL),
  apply_patch_cli_rejects_path_traversal_outside_workspace,
  apply_patch_cli_reports_missing_context,
  apply_patch_cli_reports_missing_target_file,
  apply_patch_cli_updates_file_appends_trailing_newline (FAIL),
  apply_patch_cli_verification_failure_has_no_side_effects,
  apply_patch_custom_tool_streaming_emits_updated_changes,
  apply_patch_exec_command_heredoc_with_cd_emits_turn_diff,
  apply_patch_exec_command_heredoc_with_cd_updates_relative_workdir,
  apply_patch_normalizes_crlf_without_preserve_line_endings_feature,
  apply_patch_preserves_crlf_with_preserve_line_endings_feature,
  apply_patch_shell_accepts_lenient_heredoc_wrapped_patch,
  apply_patch_shell_heredoc_normalizes_crlf_without_preserve_line_endings_feature,
  apply_patch_shell_heredoc_preserves_crlf_with_preserve_line_endings_feature (FAIL),
  apply_patch_turn_diff_paths_stay_repo_relative_when_session_cwd_is_nested,
  apply_patch_turn_diff_skips_git_root_when_feature_is_enabled::coding_originator_keeps_repository_root_when_disabled};
  `suite::apply_patch_serialization::{apply_patch_custom_tool_call_creates_file,
  apply_patch_custom_tool_call_reports_failure_output,
  apply_patch_custom_tool_call_updates_existing_file}`;
  `suite::approvals::{approval_matrix_covers_group::apply_patch,
  approving_apply_patch_for_session_skips_future_prompts_for_same_file}`;
  `suite::code_mode::code_mode_can_apply_patch_via_nested_tool`;
  `suite::hooks::{permission_request_hook_allows_apply_patch_with_write_alias,
  post_tool_use_records_additional_context_for_apply_patch,
  post_tool_use_records_apply_patch_context_with_edit_alias,
  pre_tool_use_blocks_apply_patch_before_execution,
  pre_tool_use_blocks_apply_patch_with_write_alias,
  pre_tool_use_rewrites_apply_patch_before_execution}`;
  `suite::prompt_caching::gpt_5_tools_without_apply_patch_append_apply_patch_instructions`;
  `suite::request_permissions::denied_child_permissions_require_fresh_approval::{apply_patch_session, apply_patch_turn}`;
  `suite::request_permissions_tool::approved_folder_write_request_permissions_unblocks_later_apply_patch::{with_strict_auto_review, without_strict_auto_review}`;
  `suite::shell_snapshot::unified_exec_snapshot_still_intercepts_apply_patch`;
  `suite::tool_harness::{apply_patch_reports_parse_diagnostics,
  apply_patch_tool_executes_and_emits_patch_events}`;
  `suite::unified_exec::unified_exec_intercepts_apply_patch_exec_command`.
  The 4 non-TMT FAILs (39-59 s failure times) are the same
  event-wait-timeout class: the captured panic for one of them is at
  lib.rs:388 and the global binding check proves no other panic site
  occurred anywhere in the run; all four also non-passed in the
  worker's runs (run 1/run 2) in the same class.

**Gate 2 — `just fix -p codex-core`:** EXIT 0 in 23.9 s; NO fixes
applied and NO files modified in this reviewer's run (the documented
`openai_file_mcp.rs` unused-import hazard did NOT re-trigger; the file
was clean vs HEAD before and after — `git status`/`git diff HEAD`
empty for it; both owned files' SHA-256 unchanged:
`d1eea158…` responses.rs, `4d0f17e7…` apply_patch_cli.rs).
Corroborating check: plain `cargo clippy --tests -p codex-core` exits
0 and still reports the committed baseline warnings (openai_file_mcp.rs
unused import — the known hazard source; an unrelated "empty line after
doc comment" in the seam's codex-api content_type_compat.rs — out of
Item 4 scope, noted for the seam owner). No restore was necessary in
this review; the worker's D4 restore is verified by the clean state.

**Gate 3 — `just fmt`:** EXIT 0 (~9.7 s, no output on success).
Post-fmt verification: `shasum -a 256 -c` over the two owned files →
both OK (byte-identical; the tree was already fmt-clean).
`git status` after all gates: exactly the 14 pre-existing modified
files + untracked docs — zero drift caused by this review's gates.

## Findings

**F1 (minor) — T4.1 does not assert `strict: false` and
`required: ["patch"]` on the captured wire body.**
apply_patch_cli.rs:2530-2549 asserts the tool name, `type: "function"`,
and the 7 §3.1 description substrings; the `strict`/`required` shape is
not part of the assertion. The SoT (spec §4 T4.1; breakdown Item 4
step 1) requires only "function tool whose `patch` parameter
description contains the §3.1 substrings", and the full wire shape
including `strict: false` / `required: ["patch"]` /
`additionalProperties: false` is pinned by the committed full-snapshot
spec test `create_apply_patch_function_tool_matches_expected_spec`
(apply_patch_spec_tests.rs:46-70, green in this review's run) and
observed byte-exact in the worker's live wire capture (verified:
`strict: false`, `required: ["patch"]` present). The mandate's
expectation list goes slightly beyond the SoT here; adding the two
one-liner assertions would make the integration lock self-sufficient
for the whole P1 shape, but their absence is a scope choice, not a
correctness gap — no behavior is under-locked in any way the spec
requires. Not blocking/major: the spec text, drift substrings, routing,
and shape are all locked across the item-2 spec tests + this
integration test together.

**F2 (minor) — Worker stopped the post-rename full gate run 2 before
completion (88/118).** Documented in the evidence (D2) with a
legitimate reason (self-contention with the focused runs) and
compensated by (a) the completed pre-rename full run 1 (binding-clean,
all 118), (b) the rename being behavior- and group-membership-neutral
(3 lines, verified in D1 adjudication), and (c) per-test 1/1 solo
greens for all three new tests. This reviewer's completed full run
(Mandate 6) is binding-clean with 0 deterministic failures and all 3
new tests PASS (T4.2 flaky→pass in-run; T4.1/T4.2b solo 1/1), so the
item's gate standing is finalized and the deviation carries no residual
risk. Classified minor as a procedural deviation from the item's gate
recipe, not a correctness defect.

**F3 (nit) — D1's stated rationale for the rename is incorrect.** The
initial names `apply_patch_function_tool_*` would have matched the
`core_apply_patch_cli_integration` filter via the module path
(`suite::apply_patch_cli::` contains `apply_patch_cli`; `test()`
matches the full path — proven by `cargo nextest list` enumerating
unprefixed module tests under the exact group filter). The rename was
not required for serialization; it is nonetheless harmless and matches
the file's dominant naming convention. Rationale corrected in the D1
adjudication above.

**F4 (nit) — D6's delta attribution is imprecise.** 118 = 112 (item-3
baseline, which already contained the 4 uncommitted seam tests in
apply_patch_tests.rs and item-2's spec tests) + 3 item-3-own tests +
3 Item-4 tests. The evidence's example cites item-2 spec tests
(`create_apply_patch_function_tool_*`) as part of the delta; those were
already in the 112. The count itself is correct and independently
verified.

**F5 (informational, not a finding against Item 4) — the
`core_apply_patch_cli_integration` max-threads=1 group is inactive
under `NEXTEST_PROFILE=local`** (what `just test` uses):
`[profile.local] inherits = "default"` does not carry the
`[[profile.default.overrides]]` test-group assignment (nextest
0.9.137); proven by simultaneous TMT terminations for group-matched
tests in all full runs (worker run 1/run 2 and this review's run)
versus a 241 s serialized wall time for two matched tests under
`NEXTEST_PROFILE=default`. CI (default profile) serializes; local runs
do not. This is the dominant cause of the documented load-TMT noise in
this campaign's local gates and explains why full-scope local runs of
this suite TMT heavily under any shared-host load. Pre-existing
infrastructure behavior; Item 4 owns no config files and neither
introduced nor could fix it. Candidate for a follow-up (e.g. add the
group override to the local profile, or accept local parallelism) —
outside Item 4 scope, no action required here.

## Verdict

All seven mandates verified; all six discrepancies adjudicated; all
three gates re-run to completion (or their documented equivalent) with
0 deterministic failures and all 3 new tests green. The diff is
purely additive test-only code that correctly locks the campaign's
wire shape and F1 end-to-end behavior on the renamed non-OpenAI
provider, with no product-code changes and no isolation or
serialization gaps (in CI terms).

SEAT A round 1: APPROVED — 0 Blocking, 0 Major, 2 minor, 2 nit
(+1 informational note, F5)
