# Responses-Compat — apply_patch Function-Tool Format: Task Breakdown (Stage 3)

Bead: `apex-ayl.52` (apex_tracking ledger) · Branch: `feat/normalize-content-types-vllm`
SoT spec: `docs/responses-compat-apply-patch-format.md` (**v5** — spec review
loop terminated at round 5 with 0 Blocking + 0 Major; five source-verified
minor/nit fixes applied as v5 and recorded in the spec §7 Round-5 map).
Companion SoT: `docs/responses-compat-seam.md` · v1 archive: `docs/reviews/apply-patch-task-breakdown-v1-draft.md` · v2 archive: `docs/reviews/apply-patch-task-breakdown-v2-draft.md`.

Status: TASK BREAKDOWN — v3 (stage-4 review loop terminated: round 2
returned 0 Blocking + 0 Major on v2; v3 records round 2 and applies the
seven source-verified terminal-round minor/nit fixes — maps in §9). Stage 5 (workflow-driven implementation) may begin.

## 0. Conventions and mapping

- **Item = one implementation unit = one commit = one multi-agent review
  loop.** The spec §4 T-items (T1..T5) are the items; the numbered sub-cases
  (T1.1, T2.2, …) are the TDD steps *inside* an item, executed one at a time
  (red → green) but committed together at item end, after the item's review
  loop returns 0 Blocking + 0 Major.
- Execution is **workflow-driven** (AGENTS.md stage 5): the coordinator never
  hand-edits product code; every code edit happens inside a workflow
  subagent. Item order below is the execution order; items are strictly
  sequential (shared test files and the red-state ledger make parallelism
  unsafe).
- Every message string, the P1 text, and every test expectation in this
  breakdown come from the **spec v5** (exact strings in §3.1/§3.2/§3.3/§3.4,
  T-plan in §4). The workflow subagent copies them verbatim — no
  paraphrasing, no "improved" wording.
- The 13 red assertions (spec §3.3 red state) are the initial red state;
  they are rewritten to the new expectations at the start of their owning
  item (go red), and green at item end. No commit lands with a red tree.
- Gates per item: see the item's "Gates" line; `just fmt` (from `codex-rs/`)
  runs after every item's final green, per repo rule.
- Line references below were re-verified at HEAD by the round-1 breakdown
  seats (full citation sweeps in both R1 reports); every citation found
  wrong or off in round 1 was corrected in this v2 revision (§9, rows 2,
  3, 9, 10, 11). They may drift as earlier items land; each item's first
  step is to re-verify its own references before editing.
- **Execution baseline (R2 N-M1):** the function-tool seam cited by
  Items 2-4 (`FunctionApplyPatchHandler`, `create_apply_patch_function_tool`,
  the spec equality test, the capability gate `spec_plan.rs`:1257-1271 /
  `provider.rs`:372) is part of this branch's **uncommitted working-tree
  diff** (present since 2026-09-13; at committed HEAD `197ea1642c` those
  symbols are absent — verified via `git show HEAD:` at R2). "At HEAD"
  in this breakdown means the working-tree state of this branch tip — the
  state both R1 and R2 seats verified — not `git show HEAD:`. §4 step 1
  pre-checks the seam's presence; if the seam diff gets committed before
  stage 5, re-derive the affected refs at the new HEAD first.

## 1. Red-state ledger (the 13 assertions; all verified at HEAD)

Rewritten to the new expectations (exact strings from spec §3.3) at the
start of the owning item; green at item end.

| # | Test (file:fn-line) | assert line | new expectation (spec ref) | owning item |
|---|---|---|---|---|
| 1 | `test_streaming_patch_parser_returns_errors` (`codex-rs/apply-patch/src/streaming_parser.rs`:814) | :828 | StartedPatch non-header first line: old text **+ appended guidance** (sentence extension; §3.3.1) | Item 3 |
| 2 | same | :838 | Add-File raw line (e.g. `bad`) → **`Ok`**, content `bad\n` (behavior removed by P2; §3.2) | Item 1 |
| 3 | same | :848 | Delete-File followed by a content line: **replacement** message, explicit (no shared prefix with old; §3.3.2) | Item 3 |
| 4 | `test_apply_patch_cli_rejects_invalid_hunk_header` (`codex-rs/apply-patch/tests/suite/tool.rs`:386) | :393 | exact-stderr updated to the new StartedPatch message (CLI path; spec T1.12) | Item 3 |
| 5 | `test_parse_patch` (`codex-rs/apply-patch/src/parser.rs`:277) | :281 | missing `*** Begin Patch`: new Begin message (parser-level string, no `invalid patch:` prefix; §3.3.4a) | Item 3 |
| 6 | same | :287 | missing `*** End Patch`: new End message (§3.3.4b) | Item 3 |
| 7-13 | `test_parse_patch_lenient` (`parser.rs`:558) | :579, :594, :609, :624, :628, :635, :639-642 (7 assertions) | lenient-path boundary Begin/End messages (§3.3.4a/b) | Item 3 |

Surviving tests that MUST stay green (verified by round-4/5 seats):
`apply_patch_cli.rs`:707 substring test (its patch is the StartedPatch case —
re-check in Item 3 that the new message keeps the asserted substring
`is not a valid hunk header` reachable on that path; if the rewrite breaks it,
Item 3 owns the update and says so in the commit), streaming tests at
`streaming_parser.rs`:819 / :784 / :797, and the 25 golden scenario fixtures
under `codex-rs/apply-patch/tests/fixtures/scenarios` (pass unmodified —
spec T1.13).

Harness prefix fact (spec §1.1/§3.3): the `invalid patch: ` prefix in
`function_call_output` text comes from `ParseError::InvalidPatchError`'s
Display (`parser.rs`:57); P3.4 edits the parser-level strings at
`parser.rs`:268/:271 only.

## 2. Item overview (execution order)

| Item | Spec §4 ref | What lands (commit scope) | Files touched | red-state share |
|---|---|---|---|---|
| 1 | T1 (P2 sub-cases: T1.1-3, T1.6-11, T1.13) | P2 Add-File leniency in the streaming parser + its edge tests | `codex-rs/apply-patch/src/streaming_parser.rs` (AddFile arm :198-214), new sibling test file, existing inline test (assert :838) | 1/13 |
| 2 | T2 (T2.1-4) | P1: full format taught in the function-tool `patch` argument description (non-OpenAI path) + drift guard + self-consistency + expected-literal update | `codex-rs/core/src/tools/handlers/apply_patch_spec.rs`, `apply_patch_spec_tests.rs` | 0/13 (new T2.1 test is red first) |
| 3 | T3 + T1.4, T1.5, T1.12, T1.14 (P3.1-P3.4) | Teachable parse-error messages: StartedPatch append, DeleteFile replacement, handler absent/non-string `patch`, boundary pre-pass | `streaming_parser.rs` (:193, :222), `parser.rs` (:268/:271), `codex-rs/core/src/tools/handlers/apply_patch.rs` (:534-541), `apply_patch_tests.rs`, `codex-rs/apply-patch/tests/suite/tool.rs` (:393), parser tests | 12/13 |
| 4 | T4 (T4.1, T4.2, T4.2b) | Integration: provider-rename test asserting the function tool + §3.1 substrings on the wire; F1-shaped raw Add-File end-to-end (new file + overwrite) | `codex-rs/core/tests/common/responses.rs` (new `ev_apply_patch_function_call`), `codex-rs/core/tests/suite/apply_patch_cli.rs` (new `mount_apply_patch_function_call` + 3 tests) | 0/13 (regression locks) |
| 5 | T5 + spec §6 | Seam-doc updates, spec final pass, full gates | `docs/responses-compat-seam.md`, spec (status line + §1.2 Conclusion line) | — |
| 6 | spec §5 (post-commit, operational) | Real-environment verification (release build, wiretap, exact F1 replay, raw-format probe, qwen regression), watch metrics, bead close | deployment artifacts only; no product code | — |

## 3. Item details

### Item 1 — P2: Add-File leniency (spec T1 sub-cases; §3.2)

**Scope:** make the streaming parser's `AddFile` arm accept raw/empty/
whitespace content lines verbatim. Keep the change inside the existing
AddFile arm (`streaming_parser.rs`:198-214: header check :199, `+` branch
:202-208, trailing `Err(InvalidHunkError …)` :209-214) — ~8 lines net.
Update-File stays strict (spec §3.4 decision). No existence check on any
path (spec §3.4).

**TDD steps (one at a time):**
1. Re-verify ledger #2 (`:838`) and the arm's line refs at HEAD.
2. Rewrite assertion :838 to expect `Ok` with content `bad\n` for a raw
   `bad` line inside an Add-File (spec T1.6) → run → **RED**.
3. Implement the lenient arm per spec §3.2 (raw line → verbatim content;
   `+`-prefixed line → prefix-stripped, lossy `++42` → `+42`; `*** End
   Patch` and the real header lines inside Add-File stay structural;
   whitespace-padded `*** End Patch` structural; raw (unprefixed)
   `*** End Patch` as last content line truncates; canonical `+*** End
   Patch` preserved verbatim as content).
4. Assertion :838 → **GREEN**. Run item gates (below).
5. New sub-cases, each: write test → run (RED if new behavior, green lock
   otherwise) → minimal impl fix if needed → green → gates:
   - T1.1 F1 replay: raw markdown Add-File (`# H1` first line, tables,
     blank lines) parses; `contents` byte-identical to the canonical
     `+`-prefixed version of the same file.
   - T1.2 empty line inside Add-File → empty line in `contents`.
   - T1.3 mixed `+`-prefixed and raw lines → correct concatenation.
   - T1.7 raw line beginning `*** ` but matching no marker → content;
     `*** End Patch` and each real header inside Add-File → still
     structural; whitespace-padded `*** End Patch` → structural.
   - T1.8 typo'd structural line inside Add-File (`*** Ad File: x`) →
     swallowed as content, rest of patch applies (pins the §3.2
     silent-partial-apply class).
   - T1.9 `++42 …` raw line → leading `+` stripped (pins the named lossy
     case); raw Add-File to an **existing** path → overwrite as-is (pins
     the §3.2 decision).
   - T1.9b last content line `*** End Patch`: canonical `+*** End Patch`
     preserved verbatim as content; raw unprefixed line truncates (pins
     the lenient-form-only loss).
   - T1.10 whitespace-only line → whitespace content; `*** Environment
     ID:` inside Add-File → content; unclosed patch → `finish()`/boundary
     error unchanged; CRLF raw Add-File → LF-equivalent.
   - T1.11 Update-File with raw unprefixed lines → still rejected, message
     unchanged (guards the no-Update-leniency decision).
   - T1.13 golden scope: the 25 scenario fixtures under
     `codex-rs/apply-patch/tests/fixtures/scenarios` + canonical-success
     parser tests pass **unmodified**.
6. New P2 tests live in a new sibling test file per repo convention:
   `#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"]
   mod streaming_parser_p2_tests;` — the module must NOT be named
   `tests` (the file already declares an inline `#[cfg(test)] mod tests {`
   at `streaming_parser.rs`:382-383; a second `mod tests` in the same
   file is a duplicate-name compile error). The rewritten assertion :838
   stays in the existing inline module.

**Gates (after final green, then `just fmt`):** `just test -p
codex-apply-patch`; `just fix -p codex-apply-patch`.
**DoD:** sub-cases all green; ledger #2 green; ledger #1/#3 and the parser
tests still green (untouched this item); golden scenarios unmodified;
`git diff` limited to the AddFile arm + test files; item review loop
0B+0M; one commit.

### Item 2 — P1: function-tool spec text (spec T2; §3.1)

**Scope:** teach the full patch format in the `patch` argument description
of `create_apply_patch_function_tool` (`apply_patch_spec.rs`). The exact
text is spec §3.1's embedded code block (SoT — copy verbatim): tool
description 126 chars + argument description 2,198 chars = 2,324 chars,
21 + 380 = 401 words, ending in the `Example:` block whose remainder (from
the first line exactly `Example:` to the end) parses to the T2.2 hunks.
Non-OpenAI providers only (the function path is already capability-gated at
`spec_plan.rs`:1257-1271 + `provider.rs`:372 — no gate change).

**TDD steps:**
1. T2.1: new drift-guard test asserting the 7 literal substrings (spec
   §3.1 notes): `first line is `*** Begin Patch``, `real newline
   characters`, `bare '+'`, `at most one hunk per patch`, `starts with
   '+'`, `multiple `@@` chunks`, `must change at least one line` → **RED**
   (current text lacks them).
2. Implement: replace the argument description with the spec §3.1 exact
   text (and the tool description if §3.1 specifies one — it does: the
   126-char text). Verify char/word counts match (2,198 / 401) mechanically
   in the test or as a build-time assertion only if the spec's T2.1
   mandates it — it does not; do not add extra assertions beyond the 7
   substrings.
3. T2.1 → **GREEN**; gates.
4. T2.2 self-consistency: extract per the pinned rule (split the argument
   description at the first line exactly `Example:` — a naive substring
   search is wrong, the first sentence also contains the marker phrase;
   the remainder must end with the line `*** End Patch`), `parse_patch`
   the remainder, assert the exact hunks (Add File `notes/todo.md`
   contents `# TODO\n\n1. ship the fix\n`; Update File `src/main.rs`, one
   chunk, context `fn main`, one removal, one addition, one context line).
   Green at write time; a failure means the embedded example drifted from
   spec §3.1 — fix the text, not the test.
5. T2.3: the existing spec-equality test
   `create_apply_patch_function_tool_matches_expected_spec`
   (`apply_patch_spec_tests.rs`:40-61; an inline `assert_eq!` against a
   fully-literal `ResponsesApiTool` — no insta snapshot exists for this
   test) now fails with an `assert_eq` diff on the new text → update the
   two expected literals (tool description at `:45`, `patch` argument
   description at `:52`) to the spec §3.1 exact text → green (not an
   insta snapshot — no `cargo insta` step). Intentional change, not a
   regression — say so in the commit message.
6. T2.4: freeform tool spec unchanged — run the existing freeform tests,
   confirm green, no edits.

**Gates:** `just test -p codex-core apply_patch`; `just fix -p codex-core`;
`just fmt`.
**DoD:** T2.1-4 green; expected literals updated and reviewed; no change to the
freeform spec, to the request bytes for `OpenAI`-named providers, or to any
other handler; item review loop 0B+0M; one commit.

### Item 3 — P3: teachable errors (spec T3 + T1.4/T1.5/T1.12/T1.14; §3.3)

**Scope:** make every parse-path rejection returned to the model teachable.
Four sites (per-site prefix status per spec §3.3 intro — verified in
rounds 4/5):
- **P3.1** StartedPatch, `streaming_parser.rs`:193 — **sentence extension**
  of the existing message (keeps the exact prefix; appends the §3.3.1
  guidance).
- **P3.2** Delete-File, `streaming_parser.rs`:222 — **replacement** with
  the new explicit §3.3.2 message (shares no prefix with the old text).
- **P3.3** handler, `apply_patch.rs`:508-563 — the collapsed
  `get("patch").and_then(Value::as_str).ok_or_else(…)` at :534-536 with
  message :539 splits into two messages: §3.3.3a (`patch` absent) and
  §3.3.3b (present but non-string); the argument name is re-quoted with
  single quotes per the site-3 status.
- **P3.4** boundary pre-pass, `parser.rs`:256-274, messages at :268 (Begin)
  and :271 (End) — parser-level strings (the `invalid patch: ` harness
  prefix comes from `parser.rs`:57's Display and is untouched). Pre-pass
  surface only: `parse_patch_text` (`parser.rs`:193; boundary pre-pass :195-198), the function
  path, the CLI (`lib.rs`:370), and the shell-intercept calls
  (`invocation.rs`:116/:123/:170/:175). The diff consumer's parallel
  streaming boundary messages (`streaming_parser.rs`:168/:184/:374) are
  **intentionally unchanged** (spec §3.4 divergence note).

**TDD steps:**
1. Re-verify all ledger lines 1, 3-13 and the four sites' line refs at
   HEAD (Item 1 may have shifted `streaming_parser.rs` lines — re-locate).
2. Rewrite the 12 ledger assertions (rows 1, 3-13) to the new expectations,
   exact strings from spec §3.3.1/.2/.3/.4 → run → **all 12 RED**.
3. Land P3.1 (sentence extension) → ledger #1 and #4 (same message via the
   CLI) green. Check the surviving substring test
   (`apply_patch_cli.rs`:707) — if the extended message still contains
   `is not a valid hunk header` on that path, it stays untouched; record
   the check in the commit.
4. Land P3.2 (replacement) → ledger #3 green.
5. Write the T3.1 new tests (`apply_patch_tests.rs`): `patch` absent →
   exact §3.3.3a message; `patch` present but non-string (e.g. an object) →
   exact §3.3.3b message → run → **both RED** (the collapsed handler at
   `apply_patch.rs`:534-536 returns the same old :539 message for both
   shapes) → land P3.3 (split handler messages) → T3.1 **GREEN**.
6. Land P3.4 (both boundary strings) → ledger #5-13 green.
   After each sub-step: run the item test commands. Until step 6
   completes, the only permitted failures are the ledger assertions of
   sites not yet landed — exactly: 12 red after step 2 (observed as 4
   failing test fns: the streaming errors test, the CLI hunk-header test,
   `test_parse_patch`, `test_parse_patch_lenient`) → 10 after P3.1
   (step 3) → 9 after P3.2 (step 4) → 9 after P3.3 (step 5; P3.3 owns no
   ledger rows) → 0 after P3.4 (step 6) — nothing else may fail. That
   progression is the designed state; full-green item gates are required
   before the review loop and commit.
7. T3.2 new test: handler-level F1-shaped raw Add-File patch executes
   successfully against the sandboxed test filesystem (scaffolding:
   `invocation_for_payload` `apply_patch_tests.rs`:45,
   `make_session_and_context` `session/tests.rs`:5882).
8. T1.14 is satisfied by steps 2+6 (the parser.rs rewrites are the T1.14
   red state); no extra code beyond P3.4.

**Gates:** `just test -p codex-apply-patch`; `just test -p codex-core
apply_patch`; `just fix -p codex-apply-patch -p codex-core`; `just fmt`.
**DoD:** all 13 ledger assertions green; T3.1/T3.2 green; surviving tests
(`apply_patch_cli.rs`:707, `streaming_parser.rs`:819/:784/:797) green and
untouched; golden scenarios unmodified; no change to any non-error-message
behavior; item review loop 0B+0M; one commit.

### Item 4 — Integration (spec T4)

**Scope:** end-to-end coverage on the renamed non-OpenAI provider. New
test helpers only + 3 integration tests; no product code.

**TDD steps:**
1. T4.1: new test in `apply_patch_cli.rs` — the default `test_codex()`
   provider is named "OpenAI" (→ freeform path), so rename it via
   `with_config` (`test_codex.rs`:347; the built-in clone lives at
   :839-859) to a non-OpenAI name (default dummy auth works for any name);
   assert the captured `/v1/responses` request body (via `ResponseMock`
   `.single_request()`/`.requests()`, `responses.rs`:39-60) carries
   apply_patch as a **function** tool whose `patch` parameter description
   contains the §3.1 drift substrings. Green at write time (regression
   lock over Items 1-2); a failure means a prior item is incomplete —
   stop and report, do not weaken the assertion.
2. New helper `ev_apply_patch_function_call` in
   `core/tests/common/responses.rs`, next to the pattern helpers
   (`ev_exec_command_call_with_args` :1025-1028,
   `ev_apply_patch_exec_command_call_via_heredoc` :1030-1035; generic
   constructor `ev_function_call` :933-943), mirroring the sibling
   helpers' serialize-then-call shape:
   ```rust
   pub fn ev_apply_patch_function_call(call_id: &str, patch: &str) -> Value {
       let args = serde_json::json!({ "patch": patch });
       let arguments = serde_json::to_string(&args).expect("serialize apply_patch arguments");
       ev_function_call(call_id, "apply_patch", &arguments)
   }
   ```
   The sibling local bind is required: `ev_function_call` takes
   `arguments: &str`, so the one-line
   `ev_function_call(call_id, "apply_patch",
   serde_json::to_string(&serde_json::json!({…})).unwrap())` does not
   compile (E0308 — owned `String` where `&str` is expected; re-derived
   at HEAD for this revision).
3. New `mount_apply_patch_function_call` in `apply_patch_cli.rs`,
   mirroring `mount_apply_patch` (:228) but built from the new event
   constructor (`apply_patch_responses` with a function-call variant, or a
   parallel `mount_sse_sequence` call). **Do not reuse `mount_apply_patch`**
   — it always emits `custom_tool_call`, which is rejected on the renamed
   provider: `FunctionApplyPatchHandler::matches_kind` accepts only
   `ToolPayload::Function` (`apply_patch.rs`:603-605) and the registry
   rejects the kind mismatch (`registry.rs`:549-565, message :550).
4. T4.2: mocked model emits `function_call apply_patch` with an F1-shaped
   raw Add-File patch for a **new** file → file exists with exact expected
   contents (`read_file_text`, harness method used at :323). Green at write
   time (regression lock over Item 1's P2 through the full stack).
5. T4.2b: same shape targeting an **existing** file → overwrite, contents
   exactly the patch's (pins the no-existence-check / overwrite decision
   end-to-end).
6. Prefer `wait_for_event` / `mount_sse_once` per repo integration-test
   conventions.

**Gates:** `just test -p codex-core apply_patch`; `just fix -p codex-core`;
`just fmt`.
**DoD:** 3 tests green + helpers added; no product-code diff; item review
loop 0B+0M; one commit.

### Item 5 — Docs, gates, spec final pass (spec T5 + §6)

**Scope:** docs only + full gate run. No product code.
1. `docs/responses-compat-seam.md` updates (same change set, per spec §6):
   - §2: correct the stale "plus an Azure branch" description of
     `is_openai()` — it is a name match on `OPENAI_PROVIDER_NAME`
     (`model-provider-info/src/lib.rs`:546-547, constant :40);
     Azure-named providers are in seam scope (§3.4 invariant wording).
   - §3: add P1/P2/P3 as the format-remediation layer of the
     function-tool capability, with the 2026-09-13..15 evidence (glm-5.2
     26/37 ≈ 70% as of the spec §1.2 pinned snapshot; 09-15 alone 13/21 ≈
     62%; qwen 5/413 ≈ 1%).
   - Divergence table: one row for the apply-patch parser change —
     provider-agnostic; surface = function-tool path + freeform diff
     consumer + `apply_patch` CLI (P2) plus the `parse_patch` pre-pass
     surface for P3/P3.4 (function path + CLI + shell-intercept); the
     diff consumer's parallel streaming boundary messages intentionally
     unchanged; re-apply on upstream restructure of `streaming_parser.rs`
     or `parser.rs` during rebase.
   - §6 conflict sites: add `codex-rs/apply-patch/src/streaming_parser.rs`
     and `codex-rs/apply-patch/src/parser.rs`.
   - §6 verification gate: add `just test -p codex-apply-patch` and the
     raw-format probe (spec §5.4).
   - Invariants: add the spec §3.4 exact-invariant text (request bytes for
     `OpenAI`-named providers unchanged; P2/P3 provider-agnostic, affecting
     only inputs upstream rejects; P3.4 pre-pass-only surface).
2. Spec final pass (`docs/responses-compat-apply-patch-format.md`): status
   line → implemented (record the five item commit SHAs); apply the §1.2
   Conclusion correction "≈60×" → "≈58×" (round-1 nit K-N1: the pinned
   ratio is 58.04× per the spec's own R5 map; the reviewed spec was left
   untouched in the review rounds to avoid disturbing a reviewed artifact);
   leave the §7 review log and all archives otherwise untouched.
3. Full gates: `just test -p codex-apply-patch`, `just test -p codex-core
   apply_patch`, `just test -p codex-model-provider` (capability-gate
   regression), `just fix -p` over all touched crates, `just fmt`. No
   workspace-wide `--all-features` run; no full `just test` without asking
   the user. No `Cargo.toml`/lock changes expected — if any occur,
   `just bazel-lock-update` and include the lockfile in this item.

**DoD:** seam doc and spec updated; all gates green; one commit (docs +
lockfile if any).

### Item 6 — Real-environment verification + bead close (spec §5;
operational, post-commit)

Not a TDD item; executed by the coordinator with the operator in the loop.
1. Release build `codex-cli` from this branch.
2. Install under a **new** name (e.g. `codex-aplfix1`); production binary
   untouched; rollback binary preserved in `codex-bin-backups/`.
3. Wiretap (`python3 ~/bin/wiretap.py <port>` — passive logging proxy; one
   turn pinned to it): confirm a glm-5.2 session's apply_patch arrives as
   a function tool carrying the new format description, vLLM shims active.
4. **Exact-failure replay (glm-5.2):** a session creating a new markdown
   file whose first line is an H1 (`# …`) via apply_patch — the precise F1
   scenario. Pass = file on disk, byte-identical to intent.
   **Plus the deterministic raw-format probe:** the standalone
   `apply_patch` binary from the same release build, scratch dir,
   constructed raw Add-File patch in F1 shape (no `+` prefixes, `# …`
   first content line). Pass = exit 0, contents byte-identical to the raw
   content. (A real glm session cannot be driven to emit raw format —
   wiretap is passive-only, the rate is stochastic — so the binary is the
   pinned mechanism; it shares `parse_patch` with the function handler.)
5. Regression (qwen3.8-27b): one Add-File (code file) + one Update-File
   hunk via apply_patch. Pass = both apply cleanly.
6. Watch metrics (next day's rollouts, spec §1.2 method): PARSE rate per
   model (success: glm-5.2 <10%, ideally near qwen's ~1%) **and** empty
   `{}`-arg rate on glm seats (vLLM #49248 proxy; >1% triggers the §3.4
   retry re-evaluation + the deployment-side #49249 request).
7. Cutover **only on operator greenlight**; previous binary stays as
   rollback.
8. Bead close: `command bd -C
   /Users/palanisd/Projects/bitbucket/apex_tracking close apex-ayl.52`
   with evidence pointers (this breakdown, spec v5, commit SHAs, test
   results, wiretap capture, F1 replay result, raw-probe result, qwen
   regression result, before/after PARSE + empty-arg rates, #49249
   request status). Optional operator follow-up bead: vLLM #49249 on the
   glm-5.2 deployment.

## 4. Per-item workflow protocol (AGENTS.md stage 5)

For each item, in order (the coordinator orchestrates; all code edits
happen inside workflow subagents):

1. **Pre-check:** re-verify the item's line refs at HEAD; confirm the tree
   is green (`just test -p <crate>` scoped) before starting. For Items
   2-4 additionally confirm the function-tool seam is present in the
   working tree (§0 execution baseline — `FunctionApplyPatchHandler` in
   `apply_patch.rs`); if the seam diff was committed since v2, re-derive
   the item's refs at the new HEAD first.
2. **TDD loop per sub-case:** write the failing test (or rewrite the
   ledger assertion) → run → confirm **RED** (with the exact captured
   failure) → implement the minimal change → run → confirm **GREEN** →
   run the item gates. No sub-case is "done" without captured red and
   green output (sub-cases that are regression locks with no prior red say
   so explicitly in the record).
3. **Item gates + `just fmt`** after the final green.
4. **Review loop:** dispatch MULTIPLE independent review agents on the
   item's diff (brief below); on any Blocking/Major: fix (inside a
   workflow subagent) → re-run gates → FRESH review round → repeat until a
   full round returns **0 Blocking + 0 Major**. Record each round's
   findings + resolutions in `docs/reviews/impl-<item>-r<N>.md`.
5. **Commit** (one commit per item, message: `apply-patch compat: <item>
   (apex-ayl.52) — spec §refs, red/green evidence, review rounds`).
6. Update this breakdown's §8 execution log, then start the next item.
   The §8 row is declared coordinator bookkeeping (K-N3): exempt from the
   item's code-review loop, containing factual pointers only to the
   already-reviewed `docs/reviews/impl-<item>-r<N>.md` records. The row
   (red/green evidence + review-round pointer) lands in the item's own
   commit; the commit-SHA cell is filled by a follow-up bookkeeping edit
   right after the SHA exists (itself exempt under the same declaration).

**Item review brief (every seat gets it):**
- You review ONE item's diff (files listed in the item) against
  (a) spec v5 (`docs/responses-compat-apply-patch-format.md`) — exact
  strings for messages/P1 text, the §3.4 invariants, the red-state ledger
  in this breakdown; (b) captured red/green evidence; (c) the repo rules
  in AGENTS.md (clippy `collapsible_if`, inlined `format!` args, method
  refs over closures, no bool/ambiguous-`Option` params, `/*param_name*/
  comments for opaque positional literals, exhaustive matches, private
  modules, no new small single-reference helpers, module-size targets,
  integration-test conventions, snapshot policy); (d) diff minimality
  (no drive-by edits, no scope creep beyond the item's files).
- Re-derive at the source: open every cited line, re-run the item's test
  command yourself, run the item gates.
- Severity: Blocking = wrong behavior vs spec, invariant violation
  (OpenAI request bytes changed), or a red tree in the commit; Major =
  spec string drift, missing ledger assertion, unrun gate, convention
  violation that CI would reject; Minor/Nit = the rest.
- Verdict per seat: APPROVED or CHANGES-REQUESTED with counts; item
  proceeds only on a full round of 0 Blocking + 0 Major.

## 5. Invariants (checked by every item's reviewers)

- Outbound request bytes for providers named `OpenAI` are unchanged (the
  freeform tool spec, the `base_instructions` bytes, and everything else
  on the wire).
- P2/P3 are provider-agnostic parser/handler changes that affect only
  inputs upstream rejects (raw/missing-prefix content, missing boundary
  markers, absent/non-string `patch`); no behavior change for canonical
  patches (golden scenarios prove it).
- P3.4 touches the `parse_patch` pre-pass surface only (function path +
  CLI + shell-intercept); the diff consumer's parallel streaming boundary
  messages (`streaming_parser.rs`:168/:184/:374) stay unchanged.
- No existence check is added on any path: Add-File to an existing file
  overwrites as today (pinned by fixture scenario 011 and the core-suite
  test `apply_patch_cli_add_overwrites_existing_file`; the AddFile apply
  branch `lib.rs`:508-535 writes unconditionally); Update-File keeps its
  existing requirement of an existing file (fixture scenario 009);
  Delete-File keeps its existing behavior of deleting an existing file and
  failing when the file is missing (fixture scenario 007; the DeleteFile
  apply branch `lib.rs`:536-595 reads the file's content and propagates
  removal errors). No Update-File leniency; no `strict:true`; no
  client-side retry of `{}` calls; no per-model parser modes.
- No new public API surface; no `Cargo.toml`/lock changes (if unavoidable,
  `just bazel-lock-update` lands in Item 5).

## 6. Open questions (for the stage-4 cross-review)

1. Item 3 step 3: does the extended P3.1 message keep the substring
   asserted at `apply_patch_cli.rs`:707 on that path? Verify at step time;
   the fallback (update that test in Item 3's commit) is pre-declared here.
2. Item 4 step 3: `apply_patch_responses` function-call variant vs a
   parallel `mount_sse_sequence` call — the subagent picks the smaller
   change consistent with existing helpers; both are pre-cleared.
3. Item 6 sub-step 2: the new binary name (`codex-aplfix1`) — confirm the
   `codex-bin-backups/` rollback state with the operator before installing.

## 7. Risks

- `streaming_parser.rs` line refs drift after Item 1 (mitigated: every
  item re-verifies its refs at HEAD first).
- vLLM-side `{}`-arg behavior (#49248) is unchanged by this work — the
  empty-arg watch metric (Item 6.6) is the tripwire for the deployment
  follow-up, not a defect in this change set.
- Upstream rebase of `streaming_parser.rs`/`parser.rs` re-opens the P2/P3.4
  hunk sites (recorded in the seam doc divergence table, Item 5).

## 8. Execution log (filled during stage 5)

| item | red evidence | green evidence | review rounds (0B+0M at) | commit |
|---|---|---|---|---|
| 1 | impl-item1-tdd-evidence.md §Sub-case 1 (RED, try 2) | evidence §Gates 115/115 + §Sub-cases 3–12 (GREEN) | r1 seats A/B 0B+0M (A: 1n; B: 3n) | `e21f608ac4` |
| 2 | impl-item2-tdd-evidence.md §T2.1 (RED, deterministic) | evidence §Gates: 0 deterministic failures (98 pass; 14 TMT = pre-existing baseline, m-1 corrected) | r1 seats A/B 0B+0M (A: 1m/4n; B: 3m/3n; m/n fixed in evidence pre-commit) | `a4d5af1f62` |
| 3 | impl-item3-tdd-evidence.md §Steps 2–6 (12→10→9→9→0 red window) | evidence §Gates: apply-patch 115/115; core 0 deterministic failures (lib.rs:388 load class); T3.1/T3.2 green | r1 seats A/B 0B/1M (shared, 1-line comment fix) → r2 seats C/D 0B+0M (2m/3n doc-only, fixed pre-commit) | <fill post-commit> |
| 4 | — | — | — | — |
| 5 | — | — | — | — |
| 6 | — | — | — (operator-gated) | — |

## 9. Review log (stage 4)

### Round 1 (against v1)

| Seat | Report | Verdict | Counts |
|---|---|---|---|
| L (implementability & reference accuracy) | `docs/reviews/apply-patch-task-breakdown-r1-seatL.md` | CHANGES-REQUESTED | 1 Blocking, 1 Major, 3 Minor, 4 Nit |
| K (spec conformance & TDD protocol) | `docs/reviews/apply-patch-task-breakdown-r1-seatK.md` | APPROVED | 0 Blocking, 0 Major, 6 Minor, 3 Nit |

Corroborated findings (both seats independently): L-M1 ≡ K-M3 (T2.3 insta
flow — L rated Blocking, K rated Minor), L-M2 ≡ K-M1 (ledger #4 file
path), L-M3 ≡ K-M2 (fixture count), L-N3 ≡ K-N2 (`responses.rs` range).

Every R1 finding was re-verified at the source by the coordinator before
the v2 edits were applied (line refs re-counted at HEAD with `sed -n` /
`grep -n`; the L-M5 E0308 claim re-derived against `ev_function_call`'s
`arguments: &str` signature at `responses.rs`:933; the K-M6 claims
re-derived against `lib.rs`:508-595 and fixture scenarios 007/009/011;
the K-M4 red-set progression re-derived from the ledger ownership in §1).

### v1 → v2 resolution map (17 rows: 15 from R1 + 2 coordinator additions)

| # | Finding | Sev (L / K) | v2 resolution (applied 2026-09-15 EDT) |
|---|---|---|---|
| 1 | T2.3 prescribes an insta snapshot flow for a non-insta test (L-M1 ≡ K-M3) | B / m | Item 2 T2.3 rewritten: the test is an inline `assert_eq!` equality check (`apply_patch_spec_tests.rs`:40-61; no insta snapshot exists); update the two expected literals (tool description `:45`, `patch` argument description `:52`) to the spec §3.1 exact text; no `cargo insta` step. Overview row 2 and DoD rewritten from "snapshot" to "expected literal(s)" |
| 2 | Ledger #4 cites nonexistent `codex-rs/core/tests/suite/tool.rs` (L-M2 ≡ K-M1) | M / m | Corrected to `codex-rs/apply-patch/tests/suite/tool.rs`:386 (assert :393) in ledger row 4 and in the Item 3 overview files cell; `core/tests/suite/` has no `tool.rs` (re-listed at v2) |
| 3 | "24 scenario fixtures" is 25 at HEAD (L-M3 ≡ K-M2) | m / m | Both occurrences now say 25 (25 fixture dirs — numbered 001-024 with two `020_*` dirs, `README.md` alongside in the scenarios dir; the scenario runner iterates `read_dir` dynamically, so the count cannot break the suite) |
| 4 | Item 1 step 6's `mod tests;` sibling declaration collides with the file's inline `mod tests` (L-M4) | m | Module name fixed to `streaming_parser_p2_tests` with the reason stated (inline `#[cfg(test)] mod tests {` at `streaming_parser.rs`:382-383); §6 open question 1 (which asked to confirm the name) removed as resolved |
| 5 | Item 4 step 2 helper body does not type-check as written (L-M5) | m | Body rewritten as the sibling local-bind pattern mirroring `ev_apply_patch_exec_command_call_via_heredoc` (`let args = serde_json::json!({ "patch": patch });` → `let arguments = serde_json::to_string(&args).expect(…);` → `ev_function_call(call_id, "apply_patch", &arguments)`); explicit E0308 note (`ev_function_call` takes `arguments: &str`; the one-line `.unwrap()` form passes an owned `String`) |
| 6 | Item 3 step 6 "never leave the tree red at a gate-run boundary" unsatisfiable mid-red-window (K-M4) | m | Rewritten: until step 6 completes, the only permitted failures are the not-yet-landed ledger assertions — exactly 12 → 10 (after P3.1) → 9 (after P3.2) → 9 (after P3.3, which owns no ledger rows) → 0 (after P3.4), observed at test-run granularity as 4 failing test fns, nothing else; full-green gates before review/commit |
| 7 | Item 3 step 5 reads implement-before-test for T3.1 (K-M5) | m | T3.1 made red-first in step 5 (both tests written and RED against the collapsed handler's shared old :539 message → P3.3 lands → GREEN); old step 7 (T3.1 description) deleted; steps renumbered 8→7 (T3.2), 9→8 (T1.14) |
| 8 | §5 bullet "No existence check on any path (… Delete-File never reads)" false at the source (K-M6) | m | Rewritten to the spec §3.4 shape: no existence check is ADDED on any path; Add-File overwrite as today (scenario 011, `apply_patch_cli_add_overwrites_existing_file`, AddFile branch `lib.rs`:508-535 writes unconditionally); Update requires an existing file (scenario 009); Delete deletes existing and fails missing (scenario 007; DeleteFile branch `lib.rs`:536-595 reads content, propagates removal errors) |
| 9 | `apply_patch.rs`:508-560 ends before `handle_call` closes (L-N1) | n | Now `:508-563` (verified: closing brace at :563) |
| 10 | `registry.rs`:548-556 off-by-one at start, short at end (L-N2) | n | Now `:549-565` (full kind-mismatch block: `if !tool.matches_kind(…)` at :549 through closing `}` at :565; message at :550 unchanged) |
| 11 | `responses.rs`:39-58 ends on the `requests` signature line (L-N3 ≡ K-N2) | n / n | Now `:39-60` (struct :39-41, `single_request` :50-56, `requests` :58-60) |
| 12 | §8 execution-log row reaches a commit outside any review loop (K-N3) | n | §4 step 6 now declares the §8 row coordinator bookkeeping: exempt from the item code-review loop, factual pointers only to the already-reviewed `docs/reviews/impl-<item>-r<N>.md` records; the row lands in the item's own commit, the commit-SHA cell by a follow-up exempt edit |
| 13 | Spec §1.2 "≈60×" vs the R5 map's "≈58×" (58.04×) (K-N1) | n | Deferred to Item 5 step 2 (spec final pass): apply "≈60×" → "≈58×"; the reviewed spec is deliberately untouched by this breakdown revision |
| 14 | L's own supporting-table cite `lib.rs:374-384` ends before the hunk-arm message (L-N4) | n | No breakdown fix — the breakdown's own citation (`lib.rs`:370) is correct at HEAD; the finding concerns the seat report's internal supporting table only |
| 15 | Status line + review log (bookkeeping) | — | Status → "TASK BREAKDOWN — v2 (post round-1 review; resolution map in §9)"; this §9 appended |
| 16 | §0 bullet claimed "round-4/5 seats' line-ref sweeps: 0 wrong" for the breakdown's own references — stale after R1 found 3 wrong cites (coordinator-added; not raised in R1) | n | Reworded to reference the R1 re-verification and the §9 corrections (rows 2, 3, 9, 10, 11); fixed pre-emptively at v2 to avoid a round-2 finding |
| 17 | v1 archival (coordinator-added; bookkeeping) | — | v1 archived to `docs/reviews/apply-patch-task-breakdown-v1-draft.md`; header pointer added (mirrors the spec's per-version archive convention) |

### Round 2 (against v2) — verdicts 2026-09-15 EDT

| Seat | Report | Verdict | Counts |
|---|---|---|---|
| M (implementability & reference accuracy) | `docs/reviews/apply-patch-task-breakdown-r2-seatM.md` | APPROVED | 0 Blocking, 0 Major, 0 Minor, 4 Nit |
| N (spec conformance & TDD protocol) | `docs/reviews/apply-patch-task-breakdown-r2-seatN.md` | APPROVED | 0 Blocking, 0 Major, 1 Minor, 2 Nit |

Full round = **0 Blocking + 0 Major** → the stage-4 review loop
**terminated on v2**. Per the campaign convention (spec round-5
precedent: loop terminated on v4, v5 applied the recorded terminal-round
minor/nit fixes), those fixes are applied here as v3, source-verified,
mapped below.

### v2 → v3 resolution map (terminal-round minor/nit fixes; applied 2026-09-15 EDT)

| # | Finding (seat) | Severity | v3 resolution |
|---|---|---|---|
| 1 | N-M1 — the execution baseline (function-tool seam, Items 2-4 targets) is uncommitted working-tree state; "verified at HEAD" holds only in the working-state sense | m | New §0 "Execution baseline" bullet + §4 step 1 pre-check: the seam (present since 2026-09-13) is absent at committed HEAD `197ea1642c` (re-verified for v3: `git show HEAD:` has 0 `FunctionApplyPatchHandler` occurrences in `apply_patch.rs`); "at HEAD" = working-tree state of this branch tip; if the seam diff is committed before stage 5, refs are re-derived at the new HEAD first |
| 2 | M-N1 — `parser.rs`:193-199 endpoint loose (fn ends :211; pre-pass is :195-198) | n | Item 3 P3.4 bullet now cites `parse_patch_text` (`parser.rs`:193; boundary pre-pass :195-198) (verified at HEAD) |
| 3 | M-N2 — AddFile branch `lib.rs`:508-534 off-by-one (arm brace :535) | n | `:508-535` in both the §5 bullet and the v1→v2 map row 8 (verified: arm :508-535, `added.push` :534, brace :535) |
| 4 | M-N3 — a `:560-586` "sub-detail" cited as one line short of :587 | n | **No edit needed:** the breakdown cites the full DeleteFile arm `:536-595` (verified correct at HEAD); the `:560-586` sub-range quoted by the finding does not appear anywhere in the breakdown; the removal-error path verified at source (`return Err(error)` :586, closing brace :587) |
| 5 | M-N4 — row-3 "plus README.md" phrasing attaches a file to the dir set | n | Row 3 reworded: "25 fixture dirs — numbered 001-024 with two `020_*` dirs, `README.md` alongside" |
| 6 | N-N1 — v1→v2 map row 17 did not self-identify as coordinator-added | n | Row 17 now "(coordinator-added; bookkeeping)" |
| 7 | N-N2 — Item 5 overview "Files touched" cell stale after the row-13 §1.2 Conclusion edit | n | Cell now "spec (status line + §1.2 Conclusion line)" |

Bookkeeping with this v3 revision: status line → v3 (header); v2
archived to `docs/reviews/apply-patch-task-breakdown-v2-draft.md` (header
pointer added). **Stage 5 (workflow-driven implementation) may begin per
the breakdown's item order (1→6).**
