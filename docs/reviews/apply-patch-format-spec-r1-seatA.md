# Review: apply_patch function-tool format redesign (spec round 1, seat A)

Target: `docs/responses-compat-apply-patch-format.md` (round 0 draft)
Reviewer: independent design-review seat A · 2026-09-15
Method: every load-bearing claim re-verified at the source — Lark grammar
(`codex-rs/core/assets/tools/apply_patch.lark`), streaming parser
(`codex-rs/apply-patch/src/streaming_parser.rs`,
`codex-rs/apply-patch/src/parser.rs`), the uncommitted branch diff
(`apply_patch_spec.rs`, `apply_patch.rs`, spec_plan.rs, provider.rs), the
rollout evidence under `~/.codex/sessions/2026/09/1{3,4,5}/`, and
`docs/responses-compat-seam.md`. An independent recount of all 09-13..15
rollouts (dedup by `call_id`) and a throwaway `/tmp` grammar-logic check of
the embedded example were performed. No repo code was modified.

**Verdict: CHANGES-REQUESTED — 0 Blocking, 2 Major, 6 Minor, 4 Nit.**

The design is sound and the core engineering claims check out:

- P2 is correct against the actual `AddFile` arm for every edge probed
  (empty line, whitespace-only line, CRLF, non-matching `*** ` lines,
  `*** End Patch` with surrounding whitespace, second Add-File header,
  env-id lines, unclosed patch). `parse_patch` (the execution path for BOTH
  handlers, `apply_patch.rs:411`) delegates to `StreamingPatchParser`, so
  P2 does reach the function-tool path. "Verbatim raw line + \n" is the
  right choice (matches the `+` branch, which also preserves leading
  spaces; trimming would corrupt indentation).
- F1, F2, F3 all verified verbatim in the cited rollouts (see appendix);
  P1+P2 cover F1, P1+existing-error cover F2 (self-correction observed in
  the rollout), P3.3 covers F3.
- The embedded example is valid under the Lark grammar and parses to
  exactly the hunks T2.2 asserts (checked with a `/tmp` script).
- No new `is_openai()` branches; no freeform-tool or OpenAI request-wire
  changes; change size (~300–450 lines incl. tests) is inside the 800/500
  limits; test scaffolding for T3 exists (`invocation_for_payload`,
  `make_session_and_context` with a real local env).

The two Majors are evidence-rigor and invariant-wording issues, not design
flaws; both are cheap fixes that must land before implementation starts.

## Findings

| # | Severity | Finding (short) | Spec § |
|---|---|---|---|
| M1 | Major | Attribution table internally inconsistent; 67% not reproducible as call-rate | TL;DR, §1.2 |
| M2 | Major | "Byte-identical" claim contradicted by P2/P3.1/P3.2 shared-path changes | TL;DR, §3.2, §3.3, §3.4 |
| m1 | Minor | T2.2 extraction rule unspecified; only line-based extraction works | §3.1, T2.2 |
| m2 | Minor | Existing full-snapshot spec test invalidated by P1, not mentioned in T2 | T2 |
| m3 | Minor | T2.1 drift-guard quote "starts with `+`" not a literal substring | T2.1 |
| m4 | Minor | "Mirrors the Lark 1:1" is off on `@@` optionality (and `add_line+`) | §3.1 |
| m5 | Minor | P2 edge list omits whitespace-only / env-id / unclosed-patch edges; T1 gaps | §3.2, T1 |
| m6 | Minor | P2 silently swallows *malformed* structural lines (new silent-partial-apply class) | §3.2, T1.4 |
| n1 | Nit | P3.1 guidance omits valid `*** Environment ID:` first line (freeform multi-env) | §3.3.1 |
| n2 | Nit | parser.rs doc-comment grammar listing already drifts from the .lark | §3.2 impl note |
| n3 | Nit | P3.3 quote-style flip (backticks → single quotes) | §3.3.3 |
| n4 | Nit | PR-splitting assumption unstated (branch diff already >800 lines) | §3 scope |

### M1 [Major] — Attribution table is internally inconsistent; the 67% headline is not reproducible as a call-rate

- **Spec ref:** TL;DR evidence table; §1.2.
- **Evidence:**
  - §1.2 lists for glm-5.2 "PARSE 18 / 27 calls (67%); VERIFY 6; OK 13" —
    the categories sum to 37, not 27. For qwen: 5+36+253+1+1+1 = 297 ≠ 276.
    A category sum can exceed the total only if the unit is *sessions*
    (one session can be in several classes), but the TL;DR table labels the
    column "apply_patch calls" and §1.2 says "calls". As written, the table
    is self-contradictory under either reading.
  - Independent recount of every rollout under
    `~/.codex/sessions/2026/09/1{3,4,5}/` (unique `call_id`, model from
    `turn_context`): **glm-5.2: 36 calls → PARSE 18 (50%), VERIFY 6, OK
    12**; **qwen3.8-27b: 405 calls → PARSE 4 (1%), OK 366, VERIFY 32,
    MISSING 1**. Session-level: 18 glm sessions / 33 qwen sessions with
    apply_patch activity. The PARSE/VERIFY numerators (18, 6) match the
    spec exactly, but the denominators (27 / 276) are not reproducible as
    call-level (36 / 405) or session-level (18 / 33) totals for the full
    rollout set — they imply an undocumented session/provider filter
    (cf. §1.2's "some `llm_proxy_qwen` sessions" aside, though
    `turn_context` carries no provider field I could read).
- **Impact:** the spec's justification metric (67% vs ~2%) is its headline;
  the true call-level rate is ≈50% vs ≈1% — same conclusion, different
  number. This repo's own principles (2: verify at the source; 4: no
  substitute numbers) require the evidence to be reproducible.
- **Resolution:** state the unit (calls vs sessions) and the session
  filter (or commit the forensic script), recompute, and present both rates
  in the TL;DR table. No design change implied.

### M2 [Major] — "OpenAI/Azure behavior stays byte-identical" is contradicted by P2 (and P3.1/P3.2), which touch the shared parse/error path

- **Spec ref:** TL;DR ("OpenAI/Azure behavior stays byte-identical"),
  §3 Scope, §3.2, §3.3, §3.4 (last bullet: leniency is "global and
  additive"), §6 (seam-doc updates).
- **Evidence:**
  - Both handlers run through the same `run_apply_patch_text`
    (`apply_patch.rs:400`) → `codex_apply_patch::parse_patch`
    (`apply_patch.rs:411`) → `parse_patch_text` → `StreamingPatchParser`
    (`parser.rs:193-205`). P2 rewrites the `AddFile` arm
    (`streaming_parser.rs:198-215`), and P3.1/P3.2 rewrite the
    `StartedPatch` (`streaming_parser.rs:191-196`) and `DeleteFile`
    (`streaming_parser.rs:220-225`) error texts — all shared with the
    OpenAI freeform (custom-tool) path.
  - After P2, a freeform model on the OpenAI path that emits unprefixed
    Add-File content gets the file **written**; upstream returns an
    `InvalidHunkError`. Canonical input is unchanged, but "behavior is
    byte-identical to upstream" (seam doc invariant,
    `docs/responses-compat-seam.md:218-221`) is literally violated for
    malformed input. The invariant's parenthetical test ("all capability
    flags false on `is_openai()`") is satisfied — no new capability or
    `is_openai()` branch — but the sentence-level claim is not.
- **Impact:** the spec's central safety claim will propagate (via §6) into
  the seam doc's invariant list as an overstatement; a future rebase audit
  will find the parser divergence and the byte-identical claim pointing at
  each other.
- **Resolution:** reword to "outbound **requests** to OpenAI/Azure remain
  byte-identical (no new capability, no `is_openai()` branch; P1 touches
  only the function-tool spec)" and explicitly accept P2/P3 as a tracked
  behavioral divergence: "shared parser is additively more lenient than
  upstream for malformed Add-File input; canonical input and all
  request-wire bytes unchanged". §6 already plans the divergence-table
  entry — extend it to cover the P3 message changes too, and amend the
  invariant wording in the same edit.

### m1 [Minor] — T2.2 does not specify how the example block is extracted; only a line-based rule works

- **Spec ref:** §3.1 (design notes: "A unit test parses the embedded
  example back through `parse_patch`"), T2.2.
- **Evidence:** the first sentence of the new argument description
  contains the markers inline ("starting with `*** Begin Patch` and
  ending with `*** End Patch`"). A substring search from the first
  occurrence of `*** Begin Patch` therefore extracts garbage
  ("` and ending with `*** End Patch" — demonstrated with a `/tmp`
  script: fails to parse). A line-based rule (first line exactly equal to
  `*** Begin Patch` through the first subsequent line exactly equal to
  `*** End Patch`) extracts exactly the 11-line example, which parses
  under the grammar to precisely the hunks T2.2 asserts (Add File
  `notes/todo.md` → `# TODO\n\n1. ship the fix\n`; Update File
  `src/main.rs`, one chunk, context `fn main`, one removal, one
  addition, one context line).
- **Resolution:** pin the extraction rule verbatim in the spec
  (line-based, whole-line equality, first occurrence).

### m2 [Minor] — T2 omits the update to the existing full-snapshot spec test that P1 invalidates

- **Spec ref:** T2 ("`apply_patch_spec.rs` + `apply_patch_spec_tests.rs`").
- **Evidence:** the branch already contains
  `create_apply_patch_function_tool_matches_expected_spec`
  (`apply_patch_spec_tests.rs:40`) — an `assert_eq!` of the entire
  `ToolSpec` including the current `patch` description. P1 changes both
  the tool-level description ("…edit files (add, delete, update, move).")
  and the `patch` argument description, so this test goes red until its
  expected value is rewritten. T2.1/T2.2 describe *new* tests and T2.3
  only covers the freeform tool; nothing says the existing snapshot is
  being updated.
- **Resolution:** add an explicit T2 item: "update
  `create_apply_patch_function_tool_matches_expected_spec` to the new
  exact text; it remains the authoritative full-text assertion alongside
  the substring drift-guard". (TDD will surface this red anyway, but the
  test plan should state it so the updated snapshot is a deliberate
  review artifact.)

### m3 [Minor] — T2.1 drift-guard quotes are not literal substrings of the proposed text

- **Spec ref:** T2.1 (assert presence of "starts with `+`", "bare '+'",
  "Never write raw", the marker lines).
- **Evidence:** the proposed FORMAT text (§3.1) reads "every line of the
  new file starts with '+'" (single quotes, not backticks) — the literal
  string `starts with `+"` does not occur. "bare '+'" and "Never write
  raw" do occur. An implementer asserting the quoted strings verbatim
  gets a false red.
- **Resolution:** list the exact substrings to assert (copy from the
  final text): e.g. `starts with '+'`, `bare '+'`, `Never write raw`,
  `*** Add File: <path>`, `*** Delete File: <path>`, `*** Update File:
  <path>`.

### m4 [Minor] — "The text mirrors apply_patch.lark 1:1" is not quite true: `@@` is optional even for the first chunk

- **Spec ref:** §3.1 design notes; FORMAT line "@@ [context line]  starts
  a change chunk; optional context pins the location".
- **Evidence:** the grammar is `change: (change_context |
  change_line)+ eof_line?` (`apply_patch.lark:14-16`) — a change may
  begin directly with a prefixed `change_line`; no `@@` line is required
  anywhere (verified: a chunk of `- a / + b` with no `@@` parses under
  the grammar logic in the `/tmp` check). The FORMAT line reads as if
  each chunk starts with `@@`. Also unmirrored: `add_hunk` requires
  `add_line+` (at least one content line); the text never says an Add
  File needs ≥1 line (the streaming parser accepts an empty one —
  pre-existing divergence, not worsened here).
- **Impact:** low — a model that always emits `@@` is always accepted, so
  the misreading fails safe. But the "1:1" claim is what a future
  reviewer will check.
- **Resolution:** reword to "(the first chunk may start directly with
  prefixed lines; each additional chunk starts with its own `@@
  [context]`)" and soften "mirrors 1:1" to "covers every marker, prefix,
  and optionality of the grammar" with the two deltas above noted.

### m5 [Minor] — P2 edge list omits edges the general rule covers; T1 has matching gaps

- **Spec ref:** §3.2 "Semantics and edges"; T1.
- **Evidence (all checked in `streaming_parser.rs`):**
  - Whitespace-only line inside Add-File: §3.2 claims "including
    whitespace-only" → content (raw `   ` preserved), but no T1 item
    pins it (T1.2 covers empty lines only).
  - `*** Environment ID: x` inside an Add-File hunk: env-id is only
    structural in `StartedPatch` mode (`streaming_parser.rs:85-101`), so
    in `AddFile` state it falls to the content branch under P2 (today:
    error). Not in the edge list. (Function-tool path: the model passes
    `environment_id` as a separate JSON argument and the harness injects
    the line after `*** Begin Patch` — `apply_patch.rs:569` — so the
    in-patch case is freeform-path-only, but the spec should state it.)
  - Patch that never closes: `finish()` / the `parse_patch_text`
    boundary check still rejects with "The last line of the patch must
    be '*** End Patch'" — unchanged by P2; a one-line T1 regression is
    cheap insurance.
- **Resolution:** enumerate these three in §3.2 and add the three T1
  assertions (whitespace-only → `   \n` content; env-id line in Add-File
  → content; unclosed patch → the existing finish error).

### m6 [Minor] — P2 introduces a new silent-partial-apply class: malformed structural lines inside Add-File are swallowed as content

- **Spec ref:** §3.2 (ambiguity note covers only *valid* markers), T1.4.
- **Evidence:** today, a line like `*** Update File : x` (typo'd header),
  `*** Move to: x`, `*** End of File`, or a second `*** Begin Patch`
  inside an Add-File hunk is an error (`streaming_parser.rs:209-214`).
  Under P2 — and per T1.4's own rule "line beginning `*** ` that matches
  no marker → content" — the line becomes file content, the patch
  "succeeds", and the hunk the model intended after it is lost silently.
  The spec's ambiguity note documents the *valid*-marker case (pre-
  existing, correct) but not this new malformed-marker swallowing.
- **Impact:** requires the model to typo a structural line *inside* an
  Add-File hunk; rare, and the alternative (erroring on unrecognized
  `*** ` lines) contradicts T1.4 and the leniency goal.
- **Resolution:** accept as a documented trade-off — add a sentence to
  §3.2 ("malformed/typo'd structural lines inside an Add-File hunk are
  accepted as content; a later intended hunk may then be lost — accepted
  in exchange for accepting raw F1 patches") and pin the behavior with a
  T1 test (typo'd `*** Update File : x` → content, no error).

### Nits

- **n1** — P3.1's appended guidance ("the next line must be a hunk
  header") omits `*** Environment ID:`, the one other line valid in
  `StartedPatch` mode on the freeform multi-environment path
  (`streaming_parser.rs:85-101`). The function-tool path is unaffected
  (env id is a separate JSON argument, injected by
  `with_environment_id_line`, `apply_patch.rs:569`). Optional: append
  "(or '*** Environment ID: <id>' in multi-environment sessions)".
- **n2** — The Lark listing in the `parser.rs` module doc
  (`codex-rs/apply-patch/src/parser.rs:1-24`) already drifts from the
  `.lark` file (doc shows `start: begin_patch environment_id? hunk+` and
  `add_line: "+" /(.+)/`; the asset has no `environment_id` and
  `add_line: "+" /(.*)/`). §3.2's impl note says to update the module
  doc — worth aligning the listing (or marking it illustrative) in the
  same edit.
- **n3** — P3.3 switches the argument name from backticks to single
  quotes (current: "missing the required `patch` argument",
  `apply_patch.rs:539`; F3's quoted error uses backticks). Cosmetic;
  keep one style.
- **n4** — The uncommitted branch diff already totals 1,194 insertions
  (seam work); this spec's own change (P1–P3 + tests, ≈300–450 lines) is
  inside the 800/500-line limits on its own. State the PR-splitting
  assumption so the change-size rule is evaluated on this change alone.

## What was checked and held up (no finding)

- **P2 edge matrix vs actual code** (`streaming_parser.rs` AddFile arm,
  L198-215): empty line → content (was error, confirmed at L209-214);
  `*** End Patch` with leading/trailing spaces → still structural
  (header match runs on `trimmed`, L84-136); second Add-File header mid-
  patch → new hunk (pre-existing, unchanged); CRLF → one trailing `\r`
  stripped per line by `push_delta` (L141-144) and by `str::lines()` in
  `parse_patch_text` (L193-197) — T1.7 claim holds on both paths;
  unclosed patch → `finish()` error unchanged (L154-173).
- **P2 wiring:** the function-tool execution path is
  `FunctionApplyPatchHandler` → `run_apply_patch_text` (`apply_patch.rs:400,
  411`) → `parse_patch` (always `ParseMode::Lenient`, `parser.rs:145-150`)
  → `StreamingPatchParser` (`parser.rs:193-205`) — so P2 in
  `streaming_parser.rs` does reach the function-tool path. The
  `StreamingPatchParser` use in `ApplyPatchArgumentDiffConsumer`
  (`apply_patch.rs:91`) is the streaming-events side and ignores parse
  errors (`.ok()?`), so it is unaffected.
- **Verbatim vs trimmed:** the `+` branch (L202-208) appends everything
  after `+` from the untrimmed line; the P2 content branch appending the
  raw line is consistent with that — right choice; trimming would drop
  indentation.
- **P1 text vs grammar:** every marker/prefix statement matches
  `apply_patch.lark` except the `@@` optionality nuance (m4). Bare `+`
  = empty file line (L11, `/(.*)/` matches empty) — matches the text.
  `*** End of File` optional (L17) — matches. Multiple hunks in any
  order (`hunk+`, L5) — matches. `environment_id` is not in the `.lark`
  (handled parser-side) — correctly absent from the FORMAT block.
- **Embedded example:** parses under the grammar; equals T2.2's asserted
  hunks exactly (`/tmp/grammar_check.py`, format-logic only).
- **F1:** `rollout-2026-09-15T18-42-54-…d121.jsonl`, model glm-5.2:
  `call_caf7ba097fbe46f282cd26fc` (24,230-char argument) and
  `call_c4531ddc5d8240afacc7649b` both rejected with exactly the quoted
  error ("invalid hunk at line 3, '# SPEC-FREEZE-1 ROUND 23 — …' is not a
  valid hunk header…"); the raw first content line after `*** Add File:`
  is unprefixed `# SPEC-FREEZE-1 …`. Same session later succeeds with a
  canonical Update File (`call_17e7779d2f0a4bd0adfe4543`) — glm can
  emit canonical form. Wire intact: JSON `arguments` re-parses to a
  patch with real newlines, no mangling.
- **F2:** `rollout-2026-09-15T15-46-52-…40f6.jsonl`, model glm-5.2:
  `call_d694eb148672418d834a6184` — `*** Update File:` + bare `@@` +
  unprefixed markdown, rejected with exactly the quoted error ("invalid
  hunk at line 4, Unexpected line found in update hunk: '## PART B
  concordance (L2838-5833, sections 5-10)'…"); retry
  `call_8d7a04fb51344aa4a4cc7406` prefixes the same section with `-`
  lines and succeeds — the §3.4 "observed self-correction on retry"
  claim is real.
- **F3:** `rollout-2026-09-15T17-57-04-…1ee5.jsonl` (resumed thread of
  the 15-46-52 session), model qwen3.8-27b: `call_26324b8d66f14373a4258ff1`
  arguments `{}` → "apply_patch is missing the required `patch`
  argument"; the next call is a canonical 45 KB Add-File that applies.
  Attribution "one qwen3.8-27b session" confirmed.
- **Root-cause claims:** the function-tool spec text on the branch
  carries no format knowledge (only Begin/End markers), confirmed in
  `apply_patch_spec.rs` (uncommitted). The legacy base-instructions
  example (`codex-rs/protocol/src/prompts/base_instructions/default.md:132`)
  shows only Update-file prefixes, no Add-file `+` lines, and the
  pre-Responses `{"command": …}` shape — §2.1 accurate.
- **Non-goals (§3.4):** all five rejections are justified. "No Update-File
  leniency" is airtight: unprefixed Update-hunk lines are genuinely
  ambiguous (context vs addition vs removal), guessing risks silent
  corruption, and the F2 rollout shows the model self-corrects against
  the existing clear error. "No per-model parser modes" is consistent
  with the always-on Lenient-mode precedent (`parser.rs:38-40`).
- **Safety:** no new `is_openai()` branches (capability computed at
  `provider.rs:372` by existing seam work, with the required OpenAI/
  Azure/custom unit test); P1 touches only the function-tool spec; the
  freeform spec and its tests are untouched; T4.1's test provider is
  feasible (routing gate `spec_plan.rs:1256-1272` + fallback
  `apply_patch_tool_type` from seam divergence #4).
- **Test plan scaffolding:** `invocation_for_payload`
  (`apply_patch_tests.rs:45`) + `make_session_and_context`
  (`session/tests.rs:5882`, real local env in a tempdir) support T3.1/
  T3.2 as claimed; apply-patch golden tests exist under
  `codex-rs/apply-patch/tests/suite/` for T1.9; a new T1 test module
  fits the existing `tests/suite/` layout.
- **Size/structure:** P2 stays inside the AddFile arm (~8 net lines, no
  state-machine restructure); `apply_patch_spec.rs` grows ~45 lines
  (71 → ~115); `apply_patch.rs` already 843 lines (pre-existing, gets +1
  line); module-size rules respected.

## Counts

- Blocking: **0**
- Major: **2** (M1, M2)
- Minor: **6** (m1–m6)
- Nit: **4** (n1–n4)
