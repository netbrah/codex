# Review: apply_patch format redesign spec — round 2, seat C (resolution-map audit + new-problem hunt)

Target: `docs/responses-compat-apply-patch-format.md` (v1, post round-1)
Reviewer: independent design-review seat C, round 2 · 2026-09-15
Mandate: (1) verify every row of the §7 resolution map against v1 and the
code; (2) hunt for new problems introduced by the v0→v1 revisions.

**Verdict: CHANGES-REQUESTED — 0 Blocking, 1 Major, 2 Minor, 3 Nit.**

Method (everything re-verified at the source, not inherited from round 1):
- Full read of v1, the v0 draft, both round-1 reports,
  `docs/vllm-glm-toolcall-research.md`, `docs/responses-compat-seam.md`.
- Code: `streaming_parser.rs` (all four `process_line` arms,
  `handle_hunk_headers_and_end_patch`, `push_delta`, `finish`,
  `ensure_update_hunk_is_not_empty`, error test at :813), `parser.rs`
  (doc-comment grammar listing, `parse_patch`/`parse_patch_text`),
  `seek_sequence.rs`, `invocation.rs` (multiple-operations + verify phase),
  `lib.rs` (Add-File apply), `apply_patch_spec.rs` (uncommitted function
  tool), `apply_patch_spec_tests.rs:40`, `apply_patch.rs` (diff consumer
  :88-125, `run_apply_patch_text` :400, `parse_patch` :411, JSON extraction
  :529-542), `provider.rs` (capability :372, test :707),
  `model-provider-info/src/lib.rs:546` (`is_openai`), `default.md:132`,
  `core/tests/common/test_codex.rs` (:347, :844, :1209, :1376),
  `core/tests/suite/apply_patch_cli.rs` (:228, :668, :707),
  `apply-patch/tests/suite/tool.rs` (:349, :386), scenario fixtures.
- Live probes against the unmodified working-tree binary
  (`codex-rs/target/debug/apply_patch`), in throwaway temp dirs:
  1. Update hunk whose first chunk lines carry **no** `@@` → **applies
     cleanly** (exit 0, file modified).
  2. `@@ a` followed by `@@ b` (first chunk empty) → error "Unexpected line
     found in update hunk: '@@ b'".
  3. Empty `*** Update File:` hunk → error "Update file hunk for path
     'a.txt' is empty".
  4. The exact T2.2 embedded example → parses and applies; Add-File
     contents byte-identical to `# TODO\n\n1. ship the fix\n`; Update chunk
     applies one removal, one addition, one context.
  5. Context lines are written to the file as the patch's own bytes
     (git-apply semantics); no space loss when context indentation matches
     (an earlier apparent loss was this reviewer's probe miscounting its
     own printf, not an applier defect).
- Byte-level analysis of the v1 P1 text: `Example:` is a standalone line;
  the description's last line is exactly `*** End Patch`; all seven FORMAT-
  block description columns start at exactly column 31 (perfect alignment);
  all five drift-guard substrings are literal substrings of the final text;
  no "plain text" / "wrap it in JSON" survives in the new text (grep clean).
- `~/bin/wiretap.py` read in full: passive logging MITM
  (`do_POST` = log + forward); **no** request/response injection
  capability.


## Mandate 1 — Resolution-map audit (v0 → v1, §7)

23 rows. Status: **VERIFIED** = claimed fix present and correct;
**PARTIAL** = present but does not fully resolve the finding; **MISSING** =
not present. 21 VERIFIED, 2 PARTIAL, 0 MISSING.

| Row | Status | Note |
|---|---|---|
| A-M1 evidence table | VERIFIED (nit N5) | TL;DR table (36/19/~50%, 413/5/~1%) is now internally consistent: §1.2 buckets sum to the denominators (19+10+7=36; 5+374+31+1+1+1=413). Method documented (rollout dirs, unique `call_id`, model from first `turn_context`, six outcome classes). ±1 vs seat A's recount (18/36, 4/405) disclosed. Nit: the disclosure phrasing is imprecise (N5). |
| A-M2 / B-M5 / B-n2 invariant | VERIFIED | §3.4 exact-invariant wording present and correct: request bytes for `OpenAI`-named providers unchanged (verified: function tool only emits when `!is_openai()`, `provider.rs:372`); P2/P3 named provider-agnostic, affecting only inputs upstream rejects, with full surface (function path + `ApplyPatchArgumentDiffConsumer` — verified at `apply_patch.rs:88-125` — + `apply_patch` CLI). Azure claim corrected: `is_openai()` is `name == "OpenAI"` with no Azure branch (verified `model-provider-info/src/lib.rs:546-548`, `OPENAI_PROVIDER_NAME` at :40); "not verified against any Azure deployment" stated; seam §6 correction planned. |
| B-M1 overwrite interaction | VERIFIED | §3.2 analysis present and code-accurate: no existence check anywhere (verified `invocation.rs:242-245` insert-only verify; `lib.rs:505-528` overwrite with `overwritten_content` recorded); reachability-vs-outcome distinction is correct (canonical Add-File-over-existing overwrites today — pinned upstream by `apply_patch_cli_add_overwrites_existing_file`, verified at `tool.rs:349` and `core/tests/suite/apply_patch_cli.rs:668`). Explicit decision "no existence check on any path" with rationale (a)(b)(c) in §3.2, cross-referenced from §3.4 — satisfies M1(d)'s "decide explicitly". Pinned by T1.9 + T4.2b, both present and unambiguous. |
| B-M2 wording | VERIFIED | "plain text" and "Do not wrap it in JSON" absent from both new descriptions (grep clean over the P1 block); opening sentence rewritten; backslash-n rule present; fence/heredoc wrapper prohibition present. (The "no JSON object" clause from the round-1 suggestion is not in P1, but the new P3.3b teachable error covers a non-string `patch` value — acceptable.) |
| B-M3 hunk constraint | **PARTIAL** (N3) | The constraint is taught (Rules bullet 4: "Each file may appear in at most one hunk per patch; do not target the same file twice (the tool rejects patches that do)") — verified against the enforcement at `invocation.rs:235-241` ("multiple operations target {}"). Drift-guard substring `at most one hunk per patch` is a literal substring. But the finding's core ask — teaching what to do instead (multiple `@@` chunks inside the single Update hunk for several regions) — is absent from the P1 text and example; the FORMAT block only implies it ("starts a change chunk"). |
| B-M4 rebase gate | VERIFIED (nit N2) | §6 adds `just test -p codex-apply-patch` to the seam §6 gate (verified: seam gate currently lacks it; `just test` recipe exists in Justfile), adds `streaming_parser.rs` to seam §6 conflict sites (verified absent today), and §5.4 carries the raw-format smoke. The deterministic canary (unit suite incl. T1 F1-replay) is in the gate. Nit: the smoke's mechanism is not feasible as written (N2). |
| B-M6 base instructions | VERIFIED | Backslash-n rule is in P1 (function-tool-only text → invariant-safe: function spec never ships to OpenAI-named providers); §3.4 bullet replaces v0's "outranks" assumption with the concrete in-scope fix; revisit trigger broadened to "any recurrence of F1/F2/F3-class failures". Legacy example verified at `default.md:132` (pre-Responses `{"command": [...]}` shape, literal `\n`, Update-only prefixes). |
| A-m1 / B-m3 extraction rule | VERIFIED | T2.2 pins: split at first line exactly `Example:` (verified standalone line in the description), remainder to end must end with line `*** End Patch` (verified: it is the description's last line), parse via `parse_patch`. Verified `parse_patch_text` handles a no-trailing-newline remainder (`patch.trim().lines()`). Probe 4: the example parses and yields exactly the asserted hunks. The naive-substring caveat is accurate. |
| A-m2 snapshot test | VERIFIED | T2.3 explicitly updates `create_apply_patch_function_tool_matches_expected_spec` (verified present at `apply_patch_spec_tests.rs:40` as a full `ToolSpec` `assert_eq!`). |
| A-m3 drift guard | VERIFIED | All five listed substrings are literal substrings of the final P1 text (checked programmatically): `first line is `*** Begin Patch``, `real newline characters`, `bare '+'`, `at most one hunk per patch`, `starts with '+'`. No false-red risk for an implementer copying them. |
| A-m4 "1:1 mirror" | **PARTIAL** (N1) | The "mirrors the Lark 1:1" claim is replaced — but the replacement's supporting example is factually wrong (N1, Major): "the parser accepts a subset in Update hunks, e.g. it requires `@@` before chunk lines" is contradicted by the code and by probe 1. |
| A-m5 edge list | VERIFIED | Whitespace-only / env-id / unclosed / padded-`End Patch` / CRLF edges added to §3.2 and pinned in T1.7/T1.10; each checked against the code (`push_delta` CRLF strip at :139-144; `trimmed` marker matching in `handle_hunk_headers_and_end_patch`; `finish()` boundary error). |
| A-m6 silent-partial-apply | VERIFIED | Named in §3.2 with the `*** Ad File: x` example; pinned by T1.8; mitigations (P1 "Never write raw" + P3 on the strict states) stated. |
| A-n1 env-id in P3.1 | VERIFIED | Appended guidance ends with `or '*** Environment ID: <id>' in multi-environment sessions.` — valid on the function path (the line parses in `StartedPatch` mode; the JSON `environment_id` arg is a separate, coexisting channel). |
| A-n2 doc-comment drift | VERIFIED | Impl note references the pre-existing drift — confirmed real: `parser.rs` doc comment says `add_line: "+" /(.+)/ LF` while the `.lark` has `/(.*)/`. One clarifying line promised, no full reconciliation (appropriate scope). |
| A-n3 quote style | VERIFIED | All P3 messages use single quotes around user text; the missing-arg message switches from backticks (current code) to single quotes, consistent with the other harness messages. |
| A-n4 PR splitting | VERIFIED | §3 scope states separate commits per part (P2, P1, P3) + seam-doc update, 800-line rule per logical change, branch's pre-existing seam diff named as separate. |
| B-m1 lossy `+` case | VERIFIED | Named in §3.2 as the only lossy case; probe semantics match code (the `+` branch strips one leading `+` from the raw line — `++44 20 7946 0958` → `+44 20 7946 0958`); P1 Rules bullet 1 teaches the `++` encoding; T1.9 pins it. |
| B-m2 End-Patch truncation | VERIFIED | Named in §3.2; mechanism verified (`handle_hunk_headers_and_end_patch` runs before content branches in the AddFile arm, so a final `*** End Patch` content line terminates the file — inherent, canonical form cannot represent such a file); pinned by T1.9b. |
| B-m4 test-plan scope | VERIFIED | The four rewritten assertions are enumerated and correctly numbered against `test_streaming_patch_parser_returns_errors` (verified :813-:900: the changed assertions in order are StartedPatch, AddFile, DeleteFile) plus the exact-stderr CLI test (verified `tool.rs:386`); T1.13 scoped to golden/canonical (scenario fixtures verified under `tests/fixtures/scenarios`, incl. `003_multiple_chunks`). |
| B-m5 non-string patch | VERIFIED | Separate message in §3.3.3b; T3.1 covers both branches. |
| B-n1 24,230 chars | VERIFIED | §1.3 now distinguishes arguments-JSON length (24,230) from patch-string length (23,972). |
| (vLLM research addendum) | VERIFIED | §1.3 citations checked against `docs/vllm-glm-toolcall-research.md`: `glm47` parser + `--tool-call-parser glm47`, raw string pass-through, #49248 (~5-17% under 6-way concurrency, 0 sequential, temp 0), #49249 open/not in main, SGLang same requirement, unsafe forced-choice cluster (#47504/#55541/#49981, all open), main @ 0fefffc9. §3.4 strict bullet cites the same issues; watch metric (§5.6) and deployment-side follow-up present. |


## Mandate 2 — New findings (introduced or exposed by the revisions)

### N1 [Major] — §3.1 design note asserts a parser requirement that does
not exist: "the parser … requires `@@` before chunk lines" is false

- **Where:** §3.1, first design note (the v1 A-m4 replacement text):
  "(v1: replaces the inaccurate 'mirrors the Lark 1:1' claim; the .lark
  remains the machine SoT for the freeform path, and the parser accepts a
  subset in Update hunks, **e.g. it requires `@@` before chunk lines**.)"
- **Evidence (code + live probe on the unmodified binary):**
  - `streaming_parser.rs` UpdateFile arm: context/add/remove lines
    (`line.strip_prefix(' ')` / `+` / `-`) each do
    `if chunks.is_empty() { chunks.push(UpdateFileChunk::default()); }`
    before appending — i.e. the first chunk starts **implicitly** from its
    first prefixed line, no `@@` required.
  - Probe 1 (shipped `apply_patch` binary): `*** Update File: a.txt`
    followed directly by `- second line` / `+ new line` → exit 0, file
    updated. The claimed requirement does not exist.
  - The actual parser-vs-grammar deltas run in **both** directions:
    parser-strict: empty Update hunk rejected
    (`ensure_update_hunk_is_not_empty`, "Update file hunk for path '…' is
    empty" — probe 3) although the grammar allows it (`change_move?
    change?`); second `@@` while the current chunk has no lines rejected
    (probe 2) although the grammar's `change: (change_context |
    change_line)+` allows consecutive contexts. parser-lenient: implicit
    first chunk (probe 1); a new `@@` chunk after `*** End of File`
    accepted although the grammar ends the change at `eof_line?`.
- **Why Major:** this sentence is the v1 replacement for a round-1-flagged
  inaccuracy (A-m4), so the spec now presents an empirically false,
  confidently worded claim about parser behavior in the exact spot a
  reviewer/implementer checks. It is a *new* problem introduced by the
  revision (v0 never made this claim). Concretely, an implementer who
  trusts the note would write a test asserting `@@`-less first chunks are
  rejected — that test is red against the real parser. No model-facing
  impact: the FORMAT block teaches the stricter, always-accepted form
  (every chunk shown with `@@`), so the teaching is safe; the golden
  scenario `003_multiple_chunks` pins the multi-`@@` pattern.
- **Suggested resolution:** reword the note to the verified facts, e.g.:
  "(the FORMAT block teaches the canonical, always-accepted form; the
  streaming parser is not identical to the .lark — it accepts an implicit
  first Update chunk starting with a prefixed line and no `@@`, but
  rejects an Update hunk with no content lines and a second `@@` while the
  current chunk has no lines; both the freeform grammar and the parser
  accept the taught form)."

### N2 [Minor] — §5.4 raw-format probe is not feasible as written via
wiretap + real session

- **Where:** §5.4: "drive one deliberate unprefixed-Add-File call (e.g. via
  the same harness)".
- **Evidence:** `~/bin/wiretap.py` is a passive logging proxy
  (`do_POST` logs `tool_names(body)` and forwards; no request modification,
  no canned-response injection — verified by reading the file). A real
  glm-5.2 one-turn session therefore cannot be made *deliberately* to emit
  an unprefixed Add-File: the model decides the content, and its raw-format
  rate (~50% measured) is stochastic, not drivable. As written, the probe
  either becomes "run a session and hope glm goes raw" (not a gate) or
  needs an injection mode the harness does not have.
- **Impact:** low — the rebase canary that actually carries the B-M4 risk
  is the now-gated deterministic suite (`just test -p codex-apply-patch`,
  incl. the T1 F1-replay test), which is in the seam §6 gate. The smoke is
  belt-and-braces and its mechanism is under-specified.
- **Suggested resolution:** pin a deterministic real-environment mechanism,
  e.g.: "run the `apply_patch` standalone binary from the same release
  build against a scratch dir with a constructed raw Add-File patch (the
  F1 shape); pass = file created with the raw content, exit 0" — or, if a
  session-level probe is wanted, add a canned-response injection mode to
  wiretap first (test-harness mode) and say so.

### N3 [Minor] — B-M3 resolution is partial: the constraint is taught, the
replacement pattern is not

- **Where:** §3.1 Rules bullet 4 + FORMAT block + Example; §7 row B-M3.
- **Evidence:** round-1 M3's resolution asked to replace the bullet with
  "One hunk per file. **To change several places in one file, put multiple
  `@@` chunks inside the single `*** Update File:` hunk.**" v1 teaches the
  constraint ("at most one hunk per patch; do not target the same file
  twice (the tool rejects patches that do)") but never teaches the
  alternative. The FORMAT block only implies it ("`@@ [context line]`
  starts a change chunk"); the Example shows a single `@@`. For exactly the
  weak-model class this spec targets (glm reads prompts literally, per
  F1), a two-distant-region edit in one file — M3's "most likely
  exploitation" — now gets a pre-warning that the naive form is rejected,
  but no instruction on the correct form; if it retries naively it hits
  the unteachable "multiple operations target {path}" error. The pattern is
  supported and golden-tested (`scenarios/003_multiple_chunks`), so the
  teaching gap is cheap to close.
- **Suggested resolution:** add one sentence to Rules (or the FORMAT `@@`
  line): "To change several places in one file, put multiple `@@` chunks
  inside the single `*** Update File:` hunk." Optionally extend the
  Example with a second `@@` chunk (then keep T2.2's hunk assertions in
  sync).

### N4 [Nit] — "Length ≈ 320 tokens" underestimates the P1 text

- **Where:** §3.1 design note ("Length ≈ 320 tokens, sent per request for
  non-OpenAI providers only").
- **Evidence:** measured from the v1 text: 1,891 chars / 325 words. Any
  reasonable count of this marker/backtick-heavy text lands well above 320
  (≈ 420-540). The cost conclusion ("accepted cost") is unaffected — it is
  still small and non-OpenAI-only — but the number as written is not
  supportable.
- **Suggested resolution:** re-measure with the actual tokenizer or soften
  to "≈ 400-550 tokens (1.9 KB), non-OpenAI providers only".

### N5 [Nit] — §1.2's "±1 difference" disclosure is imprecise about what
differs

- **Where:** §1.2, third bullet ("the ±1 difference is an attribution
  boundary (sessions with ambiguous `turn_context` ordering)").
- **Evidence:** the headline PARSE numerators do differ by exactly 1 per
  model (19 vs 18 glm; 5 vs 4 qwen) — that part is accurate. But the full
  reconciliation is: glm 36 calls split 19/10/7 (v1) vs 18/12/6 (seat A) —
  two borderline calls reclassified (OK bucket shifts by 2); qwen
  denominators differ by 8 (413 vs 405), i.e. ~one session's calls
  re-attributed. The parenthetical explains the qwen denominator side
  only; the glm shift is within-session outcome reclassification, not
  session attribution. Internally the v1 table is consistent (buckets sum
  to the denominators), so this is phrasing precision, not a numbers
  problem.
- **Suggested resolution:** reword to "v1's recount differs from seat A's
  by one PARSE call per model; for glm two borderline calls are
  reclassified across OK/VERIFY/PARSE (denominator unchanged at 36), for
  qwen one additional session (8 calls) is attributed to qwen (413 vs
  405); the conclusion is identical."

### N6 [Nit] — §3.1 line-range citation is a few lines off

- **Where:** §3.1 design note: "(enforced at `invocation.rs:233-236`,
  'multiple operations target {path}')".
- **Evidence:** the `if changes.contains_key(&path)` check spans
  `invocation.rs:235-241` (message at :237). Lines 233-234 are the `for`
  and `let path` lines. (Round-1 seat B had cited 235-240.) Cosmetic.
- **Suggested resolution:** update to `invocation.rs:235-241`.

Note (checked, clean): no quote-escaping issue in the P3 messages — the
appended P3.1 guidance and the P3.2/P3.3 strings contain no `{` or `}`
characters, so they are safe both inside the existing `format!` (with its
`{{path}}` escapes) and as static strings. The P3.2 message drops the
quoted offending line that the generic and Update-File messages include;
acceptable (the message is fully teachable and the model just emitted the
patch), so not filed.


## Mandate 2 checklist — items checked and found clean

- **P1 FORMAT-block alignment:** all seven description columns start at
  exactly column 31 (measured byte-for-byte) — no ragged alignment to fix.
- **P1 "Lines … real newline characters" sentence:** no misreadable
  construction; "the two characters backslash + n" is unambiguous and
  neutralizes the `default.md:132` legacy example for the function path.
- **P1 Example block:** `Example:` is a standalone line; the description
  ends with exactly `*** End Patch`; the example parses via
  `parse_patch` (no trailing newline needed — `parse_patch_text` trims and
  splits with `lines()`) and yields exactly the hunks T2.2 asserts
  (probe 4). Hunk-per-file rule (Rules bullet 4) does not contradict the
  FORMAT block; the only gap is the untaught multi-`@@` alternative (N3).
- **§3.2 "no existence check" decision:** rationale (a)(b)(c) holds
  against the code; T1.9 and T4.2b pins are unambiguous (no implementer
  misreading found). Pre-existing overwrite behavior verified at
  `lib.rs:505-528`.
- **§3.3 message 1:** P3.1 keeps the original sentence verbatim and
  appends; core-suite substring match (`apply_patch_cli.rs:707`) survives;
  streaming-test full-string assertions are in the enumerated red state.
  No format!/quote-escaping hazards (see note above).
- **T2.2 extraction rule:** works end-to-end (see clean items above); the
  "naive substring search is wrong" caveat is accurate.
- **T4.1 provider rename:** `with_config` exists (`test_codex.rs:347`);
  default test provider is a clone of the built-in `openai` entry
  (`test_codex.rs:844`, name "OpenAI" via `OPENAI_PROVIDER_NAME`) →
  freeform path by default; `is_openai()` name match drives
  `apply_patch_function_tool` (`provider.rs:372`); dummy auth
  (`CodexAuth::from_api_key("dummy")`, `test_codex.rs:1376`) works for any
  name; captured-request tools assertions have precedent (round-1 seat B:
  `client.rs:2140`, `agent_execution.rs:552`).
- **§6 seam-doc references:** all referenced sections exist — seam §2
  (contains the stale "plus an Azure branch" sentence, as claimed), §3
  (design section to extend), §4 (divergence map table), §6 (expected
  conflict sites + verification gate + invariants list).
- **§1.3 / §3.4 wire citations:** all accurate against
  `docs/vllm-glm-toolcall-research.md` (see audit table row).
- **Other v1 line-number/behavior claims:** `streaming_parser.rs:813`
  (test), `apply_patch_spec_tests.rs:40`, `tool.rs:386`,
  `apply_patch_cli.rs:668/707`, `session/tests.rs:5882`,
  `apply_patch_tests.rs:45`, `mount_apply_patch` (`apply_patch_cli.rs:228`),
  `read_file_text` (`test_codex.rs:1209`) — all present as cited. The CRLF,
  verbatim-append, trimmed-marker, env-id-in-AddFile, and
  structural-before-content claims in §3.2 all match the code.

## Verdict

**CHANGES-REQUESTED** — 0 Blocking, 1 Major, 2 Minor, 3 Nit.

The v1 revision is substantively strong: 21 of 23 resolution-map rows are
fully verified, and every load-bearing claim I re-checked against the code
held except one. The single Major (N1) is a one-sentence factual error
introduced by the A-m4 fix itself; N2/N3 are small wording/teaching gaps.
None of the findings touches the design (P1/P2/P3 shapes, the invariant,
the no-existence-check decision, the test plan structure) — all are cheap
spec-text fixes. A round-3 pass after those fixes should be able to return
zero Blocking/Major.

## Counts

- Resolution-map rows: 23 → VERIFIED 21, PARTIAL 2 (B-M3, A-m4), MISSING 0.
- New findings: **1 Major (N1), 2 Minor (N2, N3), 3 Nit (N4, N5, N6)**;
  0 Blocking.
- Live parser probes on the unmodified binary: 5 (1 dispositive for N1,
  2 characterizing real parser/grammar deltas, 2 validating the T2.2
  example end-to-end).
