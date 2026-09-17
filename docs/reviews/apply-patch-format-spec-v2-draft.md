# Responses-Compat — apply_patch Function-Tool Format Redesign

Bead: `apex-ayl.52` (apex_tracking ledger) · Branch: `feat/normalize-content-types-vllm`
Companion SoT: `docs/responses-compat-seam.md` · Wire research: `docs/vllm-glm-toolcall-research.md`
Evidence: `~/.codex/sessions/2026/09/{13,14,15}` rollouts · v0 archive: `docs/reviews/apply-patch-format-spec-v0-draft.md` · v1 archive: `docs/reviews/apply-patch-format-spec-v1-draft.md`

Status: SPEC — v2 (post round-2 review; resolution maps in §7). No code changes
until a fresh review round (round 3) returns zero Blocking and zero Major.

## TL;DR

For non-OpenAI providers the seam emits `apply_patch` as a JSON-schema
**function tool** with one required string argument `patch`. The current spec
text says only "the complete patch goes in the `patch` argument as plain
text" — it conveys **none** of the patch format. The format (every content
line carries a one-char prefix: `+` for Add File lines, ` `/`-`/`+` for
Update hunk lines) lives only in the Lark grammar of the freeform
custom-tool variant, which vLLM/SGLang deployments never receive.

Consequence: format correctness depends purely on model training priors.
Measured on the real deployment, 2026-09-13..15 (method and full recount
history in §1.2):

| Model | unique apply_patch calls | parse rejections | rate |
|---|---|---|---|
| glm-5.2 | 37 | 26 | **~70%** |
| qwen3.8-27b | 413 | 5 | **~1%** |

glm-5.2's rejections fall in two format classes: **raw unprefixed content
lines** (F1/F2, 19 of 26) and **missing boundary markers** (F4, 7 of 26) —
both are exactly what P1 teaches. qwen3.8-27b emits the canonical prefixed
format from priors; glm-5.2 reads "plain text" literally (raw file content)
and skips the `*** Begin Patch`/`*** End Patch` terminators. The wire is
exonerated by code and issue-tracker evidence (§1.3): 24 KB patches arrived
byte-intact; the failures are model content choices.

The fix is three-part: (P1) teach the full format in the function-tool spec
text; (P2) make the parser's Add-File state lenient about the missing `+`
prefix (defense in depth, model-agnostic); (P3) make parse errors returned to
the model teachable — including the boundary-marker errors (P3.4, v2).
Outbound request bytes for providers named `OpenAI` are unchanged; P2/P3 are
provider-agnostic parser changes that only alter outcomes for inputs upstream
rejects (exact invariant wording in §3.4).

## 1. Problem (verified at the source: session rollouts)

### 1.1 Failure modes observed (exact harness errors)

**F1 — Add File content written raw (no `+` prefix). Dominant, glm-5.2.**

```
apply_patch verification failed: invalid hunk at line 3,
'# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)' is not a valid
hunk header. Valid hunk headers: '*** Add File: {path}',
'*** Delete File: {path}', '*** Update File: {path}'
```

The model sent, verbatim (byte-for-byte match with the rollout, em-dash
included; re-verified by round-1 and round-2 seats):

```
*** Begin Patch
*** Add File: grok/plans/spec-freeze-r23-glm.md
# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)
<...raw markdown file content, no per-line prefixes...>
*** End Patch
```

Canonical format (SoT: `codex-rs/core/assets/tools/apply_patch.lark`,
`add_line: "+" /(.*)/ LF`) requires every Add-File content line to be
`+`-prefixed. The model wrote the file's plain text instead. (Two glm F1
variants add structural-looking raw first content lines — `@@` or `---` —
which the Add-File arm also rejects pre-P2; they are counted in F1, §1.2.)

**F2 — Update File written as raw content sections. glm-5.2.**

```
apply_patch verification failed: invalid hunk at line 4, Unexpected line
found in update hunk: '## PART B concordance (L2838-5833, sections 5-10)'.
Every line should start with ' ' (context line), '+' (added line), or
'-' (removed line)
```

Model used `@@` as a section separator and dumped unprefixed markdown. The
same session's retry prefixed the same section with `-` lines and
succeeded — the self-correction-on-retry claim is evidenced, not assumed.

**F3 — Empty arguments `{}`. One qwen3.8-27b session; self-corrected.**

```
apply_patch is missing the required `patch` argument
```

Same session, next call: a fully canonical patch that succeeded. For glm
seats specifically, vLLM research (§1.3) identified a standing mechanism for
this shape (vllm-project/vllm#49248); the single observed F3 was on qwen,
so it remains classified as glitch.

**F4 — Missing boundary markers. glm-5.2; 7 of 26 PARSE (5 Begin, 2 End).**

```
apply_patch verification failed: The first line of the patch must be '*** Begin Patch'
```
or
```
apply_patch verification failed: The last line of the patch must be '*** End Patch'
```

(v1 mis-bucketed these seven glm rejections as "VERIFY"; they are parse-path
rejections — §1.2 recount history.)

Origin: the `check_start_and_end_lines_strict` pre-pass
(`codex-rs/apply-patch/src/parser.rs:256-272`, messages at :268/:271), run
by `parse_patch_text` (parser.rs:193-199) **before** the streaming parser —
so F4 calls never reach the streaming/handler message sites that P3.1-P3.3
teach, and v1's failure list omitted this class entirely. Rollout examples
(re-verified by round-2 seat D): `~/.codex/sessions/2026-09-14/rollout-2026-09-14T03-25-55-*`
— patch is just `*** Add File: …` + `*** End Patch` (no Begin marker, no
content); `…/rollout-2026-09-14T07-58-02-*` — patch ends with a
`+`-prefixed terminator (`+*** End Patch`), never a bare one;
`…/2026-09-15/rollout-2026-09-15T14-33-27-*` — first line is
`*** Add File:`, Begin missing. P1 teaches both terminators (first/last
line); P3.4 (v2) makes these two messages teachable.

### 1.2 Attribution (rollout forensics, sessions 2026-09-13..15)

Method: every rollout under `~/.codex/sessions/2026-09-{13,14,15}`; unique
`call_id` for `function_call name=apply_patch`; model read from each
session's first `turn_context` payload; outcome from the matching
`function_call_output`. Classification (strict, v2): **PARSE** = any
rejection by the parse path, including the boundary pre-pass
(`check_start_and_end_lines_strict`) and hunk/parse errors; **VERIFY** =
post-parse filesystem verification failures only (`Failed to find expected
lines`, `Failed to read file`, …); MISSING = F3; UNSUPPORTED = pre-seam
build; OTHER = other rejections.

| Model | unique calls | PARSE | OK | VERIFY | other |
|---|---|---|---|---|---|
| glm-5.2 | 37 | **26 (~70%)** | 11 | 0 | — |
| qwen3.8-27b | 413 | **5 (~1%)** | 374 | 31 | MISSING 1, UNSUPPORTED 1, OTHER 1 |

glm PARSE breakdown (26): 13 F1-class (raw Add-File content, including
`@@`/`---`-looking first content lines), 2 F2-class (raw Update content),
5 missing `*** Begin Patch` (F4), 2 missing `*** End Patch` (F4; one patch
ends with the `+`-prefixed terminator). Most recent day (09-15) alone: 13 of
21 glm calls rejected (~62%). Zero glm call in the three days failed with a
filesystem verification error.

Recount history (disclosed in full, v2):
- v0 table (27/276, "67%"): undocumented per-file filter — **retracted**.
- Round-1 seat A (`docs/reviews/…r1-seatA.md`): glm 18/36, qwen 4/405
  (±8 qwen calls = one session's attribution boundary; ±1 glm PARSE =
  borderline reclassifications).
- v1 (coordinator): glm 19/36 "PARSE" + 10 OK + 7 "VERIFY", qwen 5/413.
  Round-2 seat D's strict recount shows v1's 7 glm "VERIFY" bucket contained
  **no** filesystem verification failures: they are the 7 F4 boundary
  rejections above, which are PARSE under the method's own definitions. The
  denominator also moved 36 → 37 in the strict recount (one call the earlier
  per-file scans missed).
- Round-2 seat D strict recount (canonical, v2): glm 26/37, qwen 413.
  Under this method the qwen3.8-27b table **reproduces exactly**
  (413 = 374 + 5 + 31 + 1 + 1 + 1), which validates the recount pipeline;
  the glm row was the outlier.

Conclusion: unchanged in direction, worse in magnitude — glm-5.2's
parse-rejection rate is ~70%, an order of magnitude above qwen's ~1%.

- Same `base_instructions` byte-identical across glm- and qwen-seat
  subagents (20,751 chars, one shared SHA-256 prefix `ac8ae107a0d7`,
  diff `identical: True`) — the seats differ in model, not in prompt.

### 1.3 Wire exoneration (rollout + vLLM source, `docs/vllm-glm-toolcall-research.md`)

- The failed F1 call's arguments JSON was 24,230 chars (the `patch` string
  itself 23,972); it arrived at the harness intact — newlines, `+`, `***`,
  em-dashes, backticks all preserved; the rejection quoted line 3 exactly as
  sent. No mangling on the wire.
- vLLM source (main @ 0fefffc9) + issue tracker: GLM-5.x is parsed by the
  `glm47` tool parser (`--tool-call-parser glm47` on the deployment class);
  string argument values pass through raw (no strip, only JSON escaping);
  no vLLM issue reports byte corruption of large GLM arguments; with
  `tool_choice: auto` and `strict:false` there is **no** schema-guided
  decoding for GLM.
- F3 mechanism for glm seats: vllm-project/vllm#49248 (open) — the glm47
  parser returns `{}` when the model omits the opening `<arg_value>` tag;
  measured ~5–17% of calls under 6-way concurrency, 0 sequential,
  temperature 0; fix #49249 (optional opening tag) is open, not in main.
  SGLang has the same requirement (its GLM detector drops the argument
  likewise).
- Forced/named `tool_choice` on GLM-5.x is currently unsafe (open
  vLLM #47504 / #55541 / #49981 cluster) — we run `auto`; this is a
  deployment-side constraint, recorded for the §3.4 non-goal.
- The model's own in-session post-mortem: "The apply_patch tool is
  rejecting the content after the Add File header — it's treating the # line
  as a hunk header. I'll write the file via a heredoc instead" — the model
  understood the rejection and worked around it; the tool should have
  accepted (or, better, the spec should have taught the format).

## 2. Root cause

1. **The function-tool spec is under-specified by construction.** When the
   seam converted apply_patch from custom/freeform (Lark-grammar-carried
   format) to function tool (no grammar channel on vLLM/SGLang), the format
   knowledge was dropped instead of re-expressed in text. The model's only
   format signals are: its training priors + the words "plain text" (which
   plausibly means "the file's plain text, unprefixed") + the legacy
   `{"command": ["apply_patch", "*** Begin Patch ..."]}` invocation example
   in base instructions (`codex-rs/protocol/src/prompts/base_instructions/
   default.md:132`, present in every seat's prompt), which shows
   Update-file prefixes only, never Add-file `+` lines, and uses the
   pre-Responses argument shape with literal backslash-n sequences.
2. **Model priors differ.** qwen3.8-27b was exposed to canonical Codex
   apply_patch examples in training (~99% canonical output); glm-5.2 was
   not (26 of 37 calls rejected at parse, §1.2 — raw content and missing
   boundary markers). Same spec, same harness, different priors — which is
   exactly the situation a *spec text* must handle, because prompt is the
   only model-agnostic channel left once grammar constraints are gone.
3. **The parser is strict where a cheap, unambiguous leniency exists.** In
   the `AddFile` state, the only structural lines are `*** ` markers; any
   other line is content. Rejecting unprefixed content lines (and even
   empty lines) buys nothing — there is no ambiguity to protect — while it
   hard-fails a common model mistake. (Update File is different: unprefixed
   lines there are genuinely ambiguous — context vs addition — so it stays
   strict.) See also §3.2 for the pre-existing Add-File-overwrite
   interaction this leniency makes reachable.

## 3. Redesign

Scope: `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` (spec text),
`codex-rs/apply-patch/src/streaming_parser.rs` (AddFile state),
`codex-rs/apply-patch/src/parser.rs` (boundary pre-pass messages, P3.4),
`codex-rs/core/src/tools/handlers/apply_patch.rs` (function handler
argument errors). No capability/routing changes; no new `is_openai()`
branches. Size: ~300–450 lines incl. tests, landed as separate commits per
part (P2, P1, P3) plus the seam-doc update; the 800-line rule applies per
logical change (the branch's pre-existing seam diff is a separate change).

### 3.1 P1 — Teach the format in the function-tool spec (primary fix)

`create_apply_patch_function_tool` keeps its shape (one required string
argument `patch`, optional `environment_id`, `strict: false`,
`additionalProperties: false`) but the argument description now carries the
full format. New exact text (stored as a `const` next to the function).

Tool-level description:

```
The `apply_patch` tool can be used to edit files (add, delete, update, move). The complete patch goes in the `patch` argument.
```

`patch` argument description:

```
The ENTIRE patch as a single string: the first line is `*** Begin Patch`, the last line is `*** End Patch`, and everything between is the patch body. Lines inside the string are separated by real newline characters — never by the two characters backslash + n. Do not add markdown fences, code-block markers, or a shell heredoc wrapper around the patch.

FORMAT — the patch is line-oriented; every file-content line carries a one-character prefix that is part of the patch, not part of the file content:

*** Add File: <path>          creates a new file; EVERY following content line starts with '+'
*** Delete File: <path>       deletes a file; takes no content lines
*** Update File: <path>       modifies a file; see hunks below
*** Move to: <path>           optional; renames the file in the current Update hunk
@@ [context line]             starts a change chunk; optional context pins the location
<chunk lines>                 each line starts with ' ' (context/unchanged), '-' (removed), or '+' (added)
*** End of File               optional; chunk extends to end of file

Rules:
- After `*** Add File:`, every line of the new file starts with '+'. An empty file line is a bare '+' with nothing after it. A file line that itself starts with '+' gets the prefix on top (file line `+42` → patch line `++42`). To create an empty file, write no content lines after the header.
- In Update hunks, prefix every line: ' ' + line for unchanged context, '-' + line for removal, '+' + line for addition. To change several places in one file, put multiple `@@` chunks inside the single `*** Update File:` hunk.
- Never write raw (unprefixed) file content lines anywhere in the patch.
- Each file may appear in at most one hunk per patch; do not target the same file twice (the tool rejects patches that do). An Update hunk must change at least one line; to rename a file without editing it, use `*** Delete File:` followed by `*** Add File:`.
- Multiple hunks for different files may appear in one patch, in any order.

Example:
*** Begin Patch
*** Add File: notes/todo.md
+# TODO
+
+1. ship the fix
*** Update File: src/main.rs
@@ fn main
-    old_call();
+    new_call();
     shared();
*** End Patch
```

Design notes (v2 revisions marked):
- The FORMAT block is the **canonical** form the model must emit; both the
  freeform grammar (`.lark`) and the streaming parser accept it.
  (v2, per round-2 seat C N1 / seat D m1, replacing v1's false "requires
  `@@` before chunk lines": the parser does **not** require `@@` — each of
  the ` `/`+`/`-` branches auto-creates the first chunk from its first
  prefixed line (streaming_parser.rs:316-360; probed on the unmodified
  binary; pinned upstream by the "Update hunk without an explicit @@ header"
  assertion in `test_parse_patch`, parser.rs:402). The real
  parser-vs-grammar deltas run the other way: the parser **rejects** a
  zero-chunk (move-only) Update hunk that the grammar's `change?` admits
  ("Update file hunk for path … is empty"; pinned upstream in
  `test_streaming_patch_parser_returns_errors`), and **rejects** a second
  `@@` while the current chunk has no lines; it **accepts** a new `@@`
  after `*** End of File` where the grammar ends the change at
  `eof_line?`. Probed: seat C probes 1-3, seat D appendix item 8.)
- (v1, seat B M2/M6) The v0 wording is dropped wholesale: "plain text"
  (the literal-reading F1 trigger) and "Do not wrap it in JSON" (false —
  the argument IS a JSON string value) are gone; the backslash-n rule
  neutralizes the legacy base-instructions example (`default.md:132`) for
  the function-tool path without touching that upstream-global text.
- (v1, seat B M3; completed in v2 per seat C N3) One-hunk-per-file is
  taught (enforced at `invocation.rs:235-241`, message at :237), and the
  replacement pattern is now taught too: multiple `@@` chunks inside the
  single `*** Update File:` hunk (supported and golden-tested by
  `scenarios/003_multiple_chunks`).
- (v2, seat D n1) Empty-file creation is taught: the parser accepts a
  zero-content Add-File (next line is another marker → empty file) pre- and
  post-P2 (probed on the unmodified binary); the grammar's `add_line+`
  would not — another parser-lenient corner, now explicit in the text.
- (v2, seat D n2) Rename-only updates are unrepresentable (the parser
  rejects zero-chunk hunks — grammar's `change?` admits) and untaught in
  v1; the text now directs Delete+Add for content-unchanged renames.
- (v1, seat A m3) Drift guard (T2.1) asserts these exact literal
  substrings (7 in v2): `first line is `*** Begin Patch``,
  `real newline characters`, `bare '+'`, `at most one hunk per patch`,
  `starts with '+'`, `multiple `@@` chunks`, `must change at least one
  line`.
- Self-consistency test (T2.2) parses the embedded example back through
  `parse_patch` and asserts the exact hunks (extraction rule pinned in T2).
  Both round-2 seats ran the v1 example through the unmodified binary and
  got exactly the asserted hunks (Add `notes/todo.md` =
  `# TODO\n\n1. ship the fix\n`; Update `src/main.rs` = one chunk, context
  `fn main`, 1 removal / 1 addition / 1 context); the Example is unchanged
  in v2, so the assertions stand.
- Length (v2, measured): tool description 126 chars + `patch` argument
  description 2,198 chars = 2,324 chars / 380 words ≈ 550–615 tokens by
  standard BPE counting for this backtick-heavy text (v1's "≈ 320 tokens"
  was an undercount — round-2 seats C N4 / D m3). Sent per request for
  non-OpenAI providers only. Accepted cost: apply_patch is the core editing
  tool; the freeform path's grammar payload is comparable in size. Well
  under the 1K-token highlight gate.
- No format text is added to the OpenAI-facing freeform tool (its grammar
  already constrains decoding; adding prose would churn the OpenAI request).

### 3.2 P2 — Add-File leniency in the streaming parser (defense in depth)

In `StreamingPatchParser::process_line`, `StreamingParserMode::AddFile`
arm (streaming_parser.rs), after the hunk-header check fails:

- `+`-prefixed line → content after `+` (unchanged).
- Any other line (including empty, whitespace-only, and lines that merely
  *start* with `*** ` but match no known marker) → **appended verbatim as
  content** (raw line, then `\n`). Verbatim is correct: the `+` branch also
  preserves everything after the prefix (including leading spaces);
  trimming would corrupt indentation.
- The final `Err(InvalidHunkError "not a valid hunk header")` in this arm
  is removed (unreachable after the two branches).

Semantics and edges (list completed per round 1; v2 additions marked):
- Canonical patches parse byte-identically to today (the `+` branch is
  untouched).
- F1 patterns now parse: `# heading` first line, raw tables, raw markdown.
- Empty line inside Add-File content → empty line in `contents` (newly
  accepted; was an error). Whitespace-only line → that whitespace as
  content. CRLF → one trailing `\r` stripped per line by `push_delta`
  before `process_line` (unchanged path).
- **Lossy case (named, per seat B m1):** a raw content line that *starts*
  with `+` is parsed as a prefixed line — its leading `+` is stripped
  (probe: patch line `++44 20 7946 0958` → content `+44 20 7946 0958`,
  identical to today's canonical behavior). P1 teaches the `++` encoding;
  a model that ignores P1 and writes a raw `+1`-style line loses one char
  per affected line, silently. Accepted: it is the parser's existing
  canonical semantics for `+` lines, P2 merely extends reachability, and
  no non-lossy alternative exists (any disambiguation heuristic would be
  worse).
- **Silent-partial-apply class (named, per seat A m6):** a typo'd
  structural line inside Add-File content (e.g. `*** Ad File: x`) is
  swallowed as content instead of erroring — the patch then continues and
  may apply partially. Pre-existing in the other direction (today: whole
  patch fails); post-P2: the rest of the patch applies. Accepted and
  pinned by test (T1.8); the P1 "Never write raw" rule plus P3 errors on
  the remaining strict states are the mitigations.
- **`*** End Patch` as a final content line (seat B m2; justification
  corrected in v2 per seat D m2):** an *unprefixed* final content line
  exactly `*** End Patch` is truncated at that line (structural check runs
  before content branches) — a lenient-form-only loss. The canonical form
  **can** represent such a file: `+*** End Patch` is content, because the
  structural check runs on the trimmed line and the `+` branch takes the
  raw line (probed on the unmodified binary: file contents `line1\n*** End
  Patch\n`). The taught form is therefore complete; only the P2 safety net
  is lossy here. Pinned by test (T1.9b, both variants).
- **Zero-content Add-File (v2, seat D n1):** `*** Add File: x` immediately
  followed by another marker → empty file. Accepted pre- and post-P2
  (probed on the unmodified binary); the grammar's `add_line+` would not
  admit it. P1 now teaches it explicitly.
- **Zero-chunk (move-only) Update hunk (v2, seat D n2):** `*** Update
  File: a` + `*** Move to: b` with no chunk lines → rejected, "Update file
  hunk for path 'a' is empty". Pre-existing, unchanged by P2; the grammar's
  `change?` admits it (parser ⊊ grammar — §3.1 design note). P1 directs
  Delete+Add for content-unchanged renames. The rejection message is not
  among P3's sites (known corner; candidate for a future P3 pass).
- **Add-File to an existing file (seat B M1 — the key interaction):**
  `Add File` has no existence check anywhere (verify:
  `invocation.rs:242-245`; apply: `lib.rs:505-528` overwrites, output just
  `A <path>`). A *canonical* Add-File over an existing file overwrites
  **today already** (pinned upstream by
  `apply_patch_cli_add_overwrites_existing_file`). P2 does not change the
  outcome; it changes **reachability**: the raw/malformed F1-shaped
  variants (glm's create-vs-edit confusion, evidenced in F1/F2) now parse
  and overwrite instead of failing loudly. **Decision (v1): no existence
  check is added on any path.** Rationale: (a) an asymmetric guard
  (reject raw-Add-File-to-existing, allow canonical) is an unexplainable
  divergence between two paths of the same tool; (b) "replace whole file
  via Add-File" is a legitimate model workflow the pinned upstream test
  codifies; (c) the path-confusion risk is identical pre/post for canonical
  patches. The interaction is documented here, pinned by T1 (overwrite
  outcome, as-is) and T4.2b (integration, existing-file variant).
- Structural lines inside Add-File (`*** End Patch`, next `*** Add File:`,
  `*** Delete File:`, `*** Update File:`, `*** Environment ID:` only in
  `StartedPatch`) keep their structural meaning — handled before content
  branches. A content line that *looks like* a valid marker remains
  structural (pre-existing inherent ambiguity, unchanged).
- `*** Environment ID:` line inside Add-File content → content (the
  env-id is honored only in `StartedPatch`, pre-existing). Unclosed patch →
  `finish()`/boundary error unchanged. `*** End Patch` with surrounding
  whitespace → still structural (trimmed match, pre-existing).
- Update-File and Delete-File arms stay strict (§3.4).

Implementation note: keep the change inside the existing `AddFile` arm
(~8 lines net); do not restructure the state machine. Update the module
doc comment with a short "lenient add-file content" note so the divergence
is discoverable on rebase (tracked in the seam doc divergence table, §6).
Note: the grammar listing in `parser.rs`'s doc comment already drifts from
the `.lark` (pre-existing, seat A n2) — this change adds one clarifying
line there as well, but does not attempt a full reconciliation.

### 3.3 P3 — Teachable errors

Four message sites (v2: the boundary pre-pass added), minimal edits (quote
style kept consistent with existing messages, single quotes around user
text). Full new message strings shown; in each case the pre-existing text
remains an exact prefix substring of the new string.

1. `StartedPatch` arm, non-header first line: **keep the original sentence
   verbatim** (existing tests match it as a substring) and append:
   ` After '*** Begin Patch', the next line must be a hunk header (e.g.
   '*** Add File: <path>' with every content line prefixed by '+'), or
   '*** Environment ID: <id>' in multi-environment sessions.`
2. `DeleteFile` arm, content line: replace the generic "not a valid hunk
   header" text with
   `'Delete File' hunks take no content lines; the next line must be
   another hunk header or '*** End Patch'`.
3. Function handler argument errors (apply_patch.rs:508-560 —
   `value.get("patch").and_then(Value::as_str)` collapses absent and
   non-string into one path today):
   - `patch` absent from the JSON object:
     `apply_patch is missing the required 'patch' argument; pass the full
     patch text starting with '*** Begin Patch' in 'patch'`.
   - (v1, seat B m5) `patch` present but not a string:
     `apply_patch 'patch' argument must be a string containing the full
     patch text starting with '*** Begin Patch'`.
4. **(v2, round-2 seat D M1 recommendation) Boundary pre-pass messages** —
   `check_start_and_end_lines_strict` (parser.rs:256-272), the single owner
   of both strings; reached by every `parse_patch` caller: the function
   handler (apply_patch.rs:411), the `apply_patch` CLI (lib.rs:370), and
   the shell-intercept path (invocation.rs:116/123/170/175). This is the
   class F4 hits; it runs before the streaming parser, so none of sites
   1-3 can teach it.
   - Begin site (parser.rs:268) → full new text:
     `The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'.`
   - End site (parser.rs:271) → full new text:
     `The last line of the patch must be '*** End Patch'. The terminator line is exactly '*** End Patch' with no '+' or other prefix and no lines after it.`
   - **Scope boundary (explicit):** the streaming parser carries parallel
     messages with the same text (`streaming_parser.rs:168` finish,
     `:184` NotStarted, `:374` EndedPatch-content). P3.4 does **not** touch
     them: they are reachable only via the direct-streaming diff consumer
     (freeform path, where the grammar constrains the same class) and were
     not observed in the failure data. The duplication is pre-existing and
     is recorded in the seam divergence table (§6).

The Update-File "Every line should start with ' ' (context line), '+'
(added line), or '-' (removed line)" message is already teachable and
stays unchanged.

Existing tests that **necessarily change** under P2/P3 (the red state, per
seat B m4, extended in v2): in `test_streaming_patch_parser_returns_errors`
(streaming_parser.rs:814) the three assertions for the StartedPatch 'bad'
message (P3.1 append), AddFile 'bad' (behavior removed by P2 — becomes
`Ok([AddFile{contents: "bad\n"}])`), and DeleteFile 'bad' message (P3.2
replacement); `test_apply_patch_cli_rejects_invalid_hunk_header`
(`codex-rs/apply-patch/tests/suite/tool.rs:386`) which asserts the exact
CLI stderr of the StartedPatch message; and (v2, P3.4) `test_parse_patch`
(parser.rs:277 — 2 exact-string assertions) and `test_parse_patch_lenient`
(parser.rs:558 — 5 exact-string assertions across its Strict-mode and
missing-closing-heredoc cases). A repo-wide search for the two boundary
strings found assertions only in those two parser tests plus the
streaming-parser tests, which stay green (their messages are untouched).
The implementer must run the `codex-apply-patch` suite at red and record
every failing assertion in the red log. The core-suite test
(`core/tests/suite/apply_patch_cli.rs:707`) survives (substring match).

### 3.4 Non-goals, decisions, and the exact invariant

- **No Update-File leniency.** Unprefixed lines inside an Update hunk are
  genuinely ambiguous (context? addition? removal?). Guessing risks silent
  file corruption — worse than a teachable error. F2 models get P1 (format
  taught) + the existing clear error + evidenced self-correction on retry.
- **No existence check for Add-File on any path** — see §3.2 for the full
  interaction analysis and decision (v1, seat B M1).
- **No `strict: true` on the function tool.** vLLM research confirms: with
  `tool_choice: auto` + `strict:false` there is no schema-guided decoding
  for GLM anyway; recent vLLM versions *do* add constrained decoding for
  strict/required GLM tools, but that path is currently unsafe
  (vllm-project/vllm #47504 required+streaming argument repetition,
  #55541 forced tool_choice truncation, #49981 xgrammar FSM crash — all
  open). Enabling it would couple tool reliability to deployment vLLM
  version/config, and it cannot express the per-line format rules
  regardless (they live inside one string).
- **base_instructions: minimal in-scope fix, rest deferred.** The legacy
  `{"command": [...]}` example with literal backslash-n sequences
  (`default.md:132`) ships to every seat and contradicts the function-tool
  argument shape. Changing that upstream-global text would churn the
  OpenAI-path request, so the in-scope fix is the backslash-n rule inside
  P1 (function-tool-only text, §3.1). (v1, seat B M6: v0's bare "outranks"
  assumption is replaced by that concrete sentence.) **Revisit trigger
  (v1, broadened): any recurrence of F1/F2/F3/F4-class failures after this
  lands** — not just empty-arg calls.
- **No client-side auto-retry of empty-argument (`{}`) calls.** Frequency
  1/413 observed; self-correcting. For glm seats the standing mechanism is
  deployment-side (vLLM #49248; fix #49249 open — **follow-up: request it
  on the glm-5.2 deployment, operator decision**, tracked on the bead).
  Watch metric in §5; revisit (retry-once in the handler) only if the
  empty-arg rate on glm seats exceeds ~1%.
- **No heredoc-wrapper leniency extension.** Lenient mode already strips
  `<<'EOF'...EOF` wrappers (gpt-4.1 precedent); no evidence glm needs more.
- **No per-model parser modes.** The parser sees no model identity;
  leniency is global and additive (canonical input unchanged), matching
  the existing always-on Lenient mode.

**Invariant (v1 exact wording; v2 surface refinement):**
Outbound **request bytes for providers named `OpenAI` are unchanged** by
this work (P1 touches only the function-tool spec, which never ships to
them). P2 and P3 are **provider-agnostic parser changes**: for every input
upstream accepts, behavior is byte-identical; for inputs upstream
*rejects*, P2 turns some Add-File rejections into accepted content and P3
turns some rejections into teachable errors. Full surface of that
divergence: the function-tool execution path, the freeform diff consumer
(`ApplyPatchArgumentDiffConsumer` streams the same parser — raw Add-File
content now streams as diffs instead of erroring), and the `apply_patch`
CLI. v2 refinement: P3.4 applies to the `parse_patch` pre-pass surface
only (function path + CLI + shell-intercept via invocation.rs:116/123/
170/175) — it does **not** touch the diff consumer's parallel streaming
boundary messages. The capability gate itself is unchanged: `is_openai()`
is `name == "OpenAI"` with **no Azure branch**
(`model-provider-info/src/lib.rs:546`) — so any Azure-*named* provider is
in seam scope (receives the function tool + P1 text); the seam doc §2
"plus an Azure branch" description is stale and is corrected in the same
PR set (v1, seat B M5 / seat A M2). This change is **not verified against
any Azure deployment** (none in use); Principle-2 verification covers the
vLLM/LiteLLM deployments only.

## 4. Test plan (TDD, one item at a time, red → green)

Items map 1:1 to implementation tasks. The rewrites of the existing
assertions listed in §3.3 are the initial red state.

**T1 — Parser Add-File leniency + boundary messages**
(`codex-rs/apply-patch/`; new test module in the existing `tests/suite/`
layout or adjacent `*_tests.rs` per repo convention):
1. F1 replay: Add-File with raw markdown content (`# H1` first line,
   tables, blank lines) parses; `contents` byte-identical to the canonical
   `+`-prefixed version of the same file.
2. Empty line inside Add-File content → empty line in `contents`.
3. Mixed `+`-prefixed and raw lines → correct concatenation.
4. StartedPatch non-header first line → original error text + appended
   guidance (§3.3.1) [rewrites assertion 1 of
   `test_streaming_patch_parser_returns_errors`].
5. Delete-File followed by a content line → new explicit message (§3.3.2)
   [rewrites assertion 3 of the same test].
6. Add-File raw line (e.g. `bad`) → `Ok` content `bad\n` [rewrites
   assertion 2: behavior removed by P2].
7. Raw content line beginning `*** ` but matching no marker → content;
   `*** End Patch` and each real header line inside Add-File → still
   structural; `*** End Patch` with surrounding whitespace → structural.
8. Typo'd structural line inside Add-File (`*** Ad File: x`) → swallowed
   as content, rest of patch applies (pins the §3.2 silent-partial-apply
   class).
9. Raw line that starts with `+` (`++42 …`) → leading `+` stripped (pins
   the named lossy case); raw Add-File to an **existing** path → overwrite,
   as-is (pins the §3.2 decision).
9b. Added file whose last content line is `*** End Patch` → (v2, both
   variants per seat D m2) the **canonical** line `+*** End Patch` is
   preserved verbatim as content; the **raw** (unprefixed) line is
   truncated at that line (pins the §3.2 lenient-form-only loss).
10. Whitespace-only line in Add-File → whitespace content; `***
    Environment ID:` inside Add-File → content; unclosed patch →
    `finish()`/boundary error unchanged; CRLF raw Add-File → LF-equivalent.
11. Update-File with raw unprefixed lines → still rejected, message
    unchanged (guards the no-Update-leniency decision).
12. `test_apply_patch_cli_rejects_invalid_hunk_header`
    (`tests/suite/tool.rs:386`) updated to the new StartedPatch message
    [the fourth rewritten assertion].
13. **Golden/canonical scope:** the scenario fixtures under
    `codex-rs/apply-patch/tests/fixtures/scenarios` and canonical-success
    parser tests pass **unmodified** (v1 scoping replaces v0's overbroad
    "all existing tests unmodified").
14. **(v2, P3.4)** Boundary pre-pass: `parse_patch_text(Strict)` on a
    patch missing `*** Begin Patch` → new Begin message (§3.3.4a); missing
    `*** End Patch` → new End message (§3.3.4b) [rewrites
    `test_parse_patch` (parser.rs:277) and `test_parse_patch_lenient`
    (parser.rs:558) exact-string assertions].

**T2 — Function-tool spec** (`apply_patch_spec.rs` +
`apply_patch_spec_tests.rs`):
1. Drift guard: assert the 7 exact literal substrings listed in §3.1
   (v1's 5 plus `multiple `@@` chunks`, `must change at least one line`).
2. Self-consistency: **extraction rule (pinned, v1):** split the argument
   description at the first line exactly `Example:`; the remainder (to end
   of description) must end with the line `*** End Patch`; `parse_patch`
   that remainder and assert the exact hunks (Add File `notes/todo.md`
   contents `# TODO\n\n1. ship the fix\n`; Update File `src/main.rs` one
   chunk, context `fn main`, one removal, one addition, one context line).
   (A naive substring search is wrong — the first sentence also contains
   "first line is `*** Begin Patch`".)
3. (v1, seat A m2) The existing full-snapshot test
   `create_apply_patch_function_tool_matches_expected_spec`
   (apply_patch_spec_tests.rs:40) is **updated to the new exact text** —
   an intentional change, not a regression.
4. Freeform tool spec unchanged (existing tests cover; no new surface).

**T3 — Function handler** (`apply_patch.rs` + `apply_patch_tests.rs`):
1. `patch` absent → message §3.3.3a; `patch` present but non-string (e.g.
   an object) → message §3.3.3b (v1).
2. Handler-level: an F1-shaped raw Add-File patch executes successfully
   against the sandboxed test filesystem (existing scaffolding:
   `invocation_for_payload`, `make_session_and_context`).

**T4 — Integration** (`codex-rs/core/tests/suite/`, per repo rule that
agent-logic changes get integration tests):
1. Non-OpenAI provider: the default `test_codex()` provider is named
   "OpenAI" (→ freeform path), so the test renames it via `with_config`
   (`config.model_provider.name = "Test Provider"`; default dummy auth
   works for any name — v1, seat B d-answer with setup pointers). Assert
   the captured `/v1/responses` request body carries apply_patch as a
   **function** tool whose `patch` parameter description contains the §3.1
   substrings.
2. Behavioral: mocked model emits `function_call apply_patch` with an
   F1-shaped raw Add-File patch for a **new** file → file exists with exact
   expected contents. **Scaffold (v2, round-2 seat D M2):** the existing
   `mount_apply_patch` (apply_patch_cli.rs:228) is **not usable** here — it
   always builds `ev_apply_patch_custom_tool_call` (a `custom_tool_call`
   item), and on the renamed non-OpenAI provider only
   `FunctionApplyPatchHandler` is registered, whose `matches_kind` accepts
   only `ToolPayload::Function` (apply_patch.rs:603-605); the registry
   rejects the kind-mismatched payload ("tool apply_patch invoked with
   incompatible payload", registry.rs:548-556). Add instead:
   - `ev_apply_patch_function_call(call_id, patch)` in
     `core/tests/common/responses.rs`:
     `ev_function_call(call_id, "apply_patch",
     serde_json::to_string(&json!({ "patch": patch })).unwrap())` — same
     pattern as `ev_exec_command_call_with_args` /
     `ev_apply_patch_exec_command_call_via_heredoc` (responses.rs:1030-1038,
     generic constructor at :933-942);
   - `mount_apply_patch_function_call` in `apply_patch_cli.rs`, mirroring
     `mount_apply_patch` but built from the new event constructor
     (`apply_patch_responses` with a function-call variant, or a parallel
     `mount_sse_sequence` call).
   `read_file_text` (harness method, apply_patch_cli.rs:323) and
   `harness.submit` are fine as-is.
2b. Existing-file variant: same shape targeting an **existing** file →
   overwrite, contents exactly the patch's (v1, seat B M1).

**T5 — Gates** (after each green item, then at the end):
- `just fmt` (codex-rs).
- `just test -p codex-apply-patch` (full suite incl. golden scenarios),
  `just test -p codex-core apply_patch` (scope filter), `just test -p
  codex-model-provider` (capability gate regression).
- `just fix -p codex-apply-patch -p codex-core` for touched crates.
  No workspace-wide `--all-features` run.

## 5. Real-environment verification (Principle 2 — never a substitute)

The problem occurs on the live vLLM/LiteLLM deployment; verification runs
there, following `docs/responses-compat-seam.md` §6:

1. Build: release `codex-cli` from this branch.
2. Install under a **new** name (e.g. `codex-aplfix1`); production binary
   untouched; rollback binary preserved in `codex-bin-backups/`.
3. Wiretap (`python3 ~/bin/wiretap.py <port>` + one-turn session pinned to
   it): confirm for a glm-5.2 session that apply_patch arrives as a
   function tool carrying the new format description, with the vLLM shims
   active.
4. **Exact-failure replay (glm-5.2):** a session that must create a new
   markdown file whose first line is an H1 (`# ...`) via apply_patch —
   the precise F1 scenario. Pass = file on disk, byte-identical to intent.
   **Plus a raw-format probe (v2, per round-2 seat C N2 / seat B M4,
   deterministic mechanism pinned):** run the **standalone `apply_patch`
   binary from the same release build** against a scratch dir with a
   constructed raw Add-File patch in the F1 shape (no `+` prefixes, `# …`
   first content line); pass = exit 0 and the file's contents
   byte-identical to the raw content. A real glm session cannot be
   *driven* to emit raw format — `~/bin/wiretap.py` is a passive logging
   proxy (log + forward; no request/response injection, verified by
   reading it) and the raw-format rate is stochastic, not drivable — so
   the probe uses the binary directly, which shares the exact parser the
   function handler calls (`parse_patch`). This makes a future rebase that
   silently drops P2 fail this smoke, not just the unit suite.
5. Regression (qwen3.8-27b): one session doing an Add-File (code file)
   and one Update-File hunk via apply_patch. Pass = both apply cleanly.
6. **Watch metrics (v1):** next day's rollouts — PARSE rate per model
   (success: glm-5.2 <10%, ideally near qwen's ~1%) **and** empty-argument
   `{}` rate on glm seats (proxy indicator for vLLM #49248; threshold:
   >1% triggers the §3.4 retry re-evaluation and the deployment-side
   #49249 request).
7. Cutover: only on operator greenlight; previous binary stays as
   rollback.

## 6. Rollout, rebase, and seam-doc updates

`docs/responses-compat-seam.md` updates (same PR set):
- §2: correct the stale "plus an Azure branch" description of `is_openai()`
  (name match only; Azure-named providers are in seam scope, §3.4
  invariant wording adopted).
- §3: add P1/P2/P3 as the format-remediation layer of the function-tool
  capability, with the 2026-09-15 glm-5.2 evidence (26/37 ≈ 70% vs qwen
  5/413 ≈ 1%, §1.2).
- Divergence table: one row for the apply-patch parser change —
  **provider-agnostic**, surface = function-tool path + freeform diff
  consumer + `apply_patch` CLI (P2; v1, seat B n2) **plus** the
  `parse_patch` pre-pass surface for P3/P3.4 (function path + CLI +
  shell-intercept; the diff consumer's parallel streaming boundary
  messages are intentionally unchanged — v2), re-apply on upstream
  restructure of `streaming_parser.rs` or `parser.rs` during rebase.
- §6 conflict sites: add `codex-rs/apply-patch/src/streaming_parser.rs`
  (v1, seat B M4) and `codex-rs/apply-patch/src/parser.rs` (v2, P3.4).
- §6 verification gate: **add `just test -p codex-apply-patch`** and the
  raw-format probe (this doc §5.4) — v0's gate would have passed a rebase
  that silently reverted P2 (v1, seat B M4).
- Invariants: add the §3.4 exact-invariant text (request-bytes for
  `OpenAI`-named providers unchanged; P2/P3 = provider-agnostic changes
  affecting only inputs upstream rejects; P3.4 pre-pass-only surface).

Beads: `apex-ayl.52` closed with evidence pointers (this doc, test
results, rollout before/after PARSE + empty-arg rates, deployment-side
#49249 request status). Follow-up bead (optional, operator call): vLLM
#49249 / optional-opening-tag patch on the glm-5.2 deployment.

No `Cargo.toml`/lock changes expected; if any, `just bazel-lock-update`.

## 7. Review log

- **Round 0** (2026-09-15): draft written by coordinator; v0 archived at
  `docs/reviews/apply-patch-format-spec-v0-draft.md`.
- **Round 1** (2026-09-15): two independent seats, both
  **CHANGES-REQUESTED, 0 Blocking**:
  - Seat A (`docs/reviews/apply-patch-format-spec-r1-seatA.md`): 2 Major
    (M1 evidence table internally inconsistent / 67% not reproducible; M2
    "byte-identical" contradicted by P2/P3 shared-path changes), 6 Minor,
    4 Nit. Verified P2 against the actual AddFile arm for every edge;
    confirmed parse_patch wiring to the function path; re-counted the
    rollouts independently (18/36 glm, 4/405 qwen).
  - Seat B (`docs/reviews/apply-patch-format-spec-r1-seatB.md`,
    adversarial): 6 Major (M1 P2 + silent Add-File overwrite = new
    corruption reachability; M2 "plain text"/"don't wrap in JSON" wording;
    M3 one-hunk-per-file constraint untaught; M4 rebase gate gap; M5
    "OpenAI/Azure byte-identical" false — no Azure branch in
    `is_openai()`; M6 legacy base-instructions example unaddressed), 5
    Minor, 2 Nit. Built the unmodified `apply_patch` binary and probed 8
    constructed counter-examples (appendix table in that report).

### Round-1 resolution map (v0 → v1)

| Finding | Resolution | v1 location |
|---|---|---|
| A-M1 evidence table | Retried recount with documented method; glm 19/36 (~50%), qwen 5/413 (~1%); v0 numbers retracted; ±1 vs seat A's recount disclosed. **(v2 note: the v1 table was itself later found mis-bucketed — see R2 row D-M1.)** | TL;DR, §1.2 |
| A-M2 / B-M5 / B-n2 invariant | "Byte-identical" replaced with exact request-bytes invariant; P2/P3 named as provider-agnostic changes affecting only inputs upstream rejects; full surface (function path, diff consumer, CLI); Azure claim corrected, no Azure verification | §3.4, §6 |
| B-M1 overwrite interaction | Full analysis added (no existence check pre-existing; P2 changes reachability); **decision: no existence check on any path** with rationale; pinned by T1.9 + T4.2b | §3.2, §3.4, T1/T4 |
| B-M2 wording | "plain text" and "Do not wrap it in JSON" removed; opening sentence rewritten | §3.1 |
| B-M3 hunk constraint | "at most one hunk per file" rule taught; drift guard substring. **(v2 note: partial — the replacement pattern was untaught; completed by R2 row C-N3.)** | §3.1, T2.1 |
| B-M4 rebase gate | `just test -p codex-apply-patch` + raw-format smoke added to seam §6 gate; `streaming_parser.rs` added to conflict sites | §5.4, §6 |
| B-M6 base instructions | Backslash-n rule added to P1 (function-tool-only, invariant-safe); revisit trigger broadened to any F1/F2/F3 recurrence (v2: +F4) | §3.1, §3.4 |
| A-m1 / B-m3 extraction rule | Pinned: split at first line exactly `Example:`; remainder must end with `*** End Patch` | T2.2 |
| A-m2 snapshot test | Existing `create_apply_patch_function_tool_matches_expected_spec` explicitly updated | T2.3 |
| A-m3 drift guard | Exact literal substrings listed (v2: 7, was 5) | §3.1, T2.1 |
| A-m4 "1:1 mirror" | Claim replaced with parser-subset wording. **(v2 note: partial — the replacement wording was itself empirically false; corrected by R2 row C-N1/D-m1.)** | §3.1 notes |
| A-m5 edge list | Whitespace-only / env-id / unclosed / padded-End edges added with tests | §3.2, T1.7/1.10 |
| A-m6 silent-partial-apply | Named class + test pin | §3.2, T1.8 |
| A-n1 env-id in P3.1 | Guidance now mentions `*** Environment ID:` first line | §3.3.1 |
| A-n2 doc-comment drift | One clarifying line added; no full reconciliation | §3.2 impl note |
| A-n3 quote style | Consistent single-quote style | §3.3 |
| A-n4 PR splitting | Separate commits per part stated; 800-line rule per logical change | §3 scope |
| B-m1 lossy `+` case | Named (only lossy case) + `++` taught + test pin | §3.1 rules, §3.2, T1.9 |
| B-m2 End-Patch truncation | Named + test pin. **(v2 note: justification corrected — see R2 row D-m2.)** | §3.2, T1.9b |
| B-m4 test-plan scope | Four rewritten assertions enumerated as the red state; T1.13 scoped to golden/canonical (v2: red state extended by P3.4 tests, T1.14) | §3.3, T1.4-6/12/13/14 |
| B-m5 non-string patch | Separate error message added | §3.3.3, T3.1 |
| B-n1 24,230 chars | Clarified (arguments-JSON vs patch-string length) | §1.3 |
| (vLLM research addendum) | §1.3 expanded (wire exoneration by code+issues; F3 mechanism #49248; strict/forced-choice constraints); §3.4 strict bullet cited; deployment-side follow-up + watch metric | §1.3, §3.4, §5.6 |

- **Round 2** (2026-09-15): two fresh independent seats on v1, both
  **CHANGES-REQUESTED, 0 Blocking**:
  - Seat C (`docs/reviews/apply-patch-format-spec-r2-seatC.md`) — mandate:
    resolution-map audit + new-problem hunt: 23 map rows → 21 VERIFIED, 2
    PARTIAL (B-M3, A-m4), 0 MISSING. 1 Major (N1: the v1 A-m4 replacement
    sentence "parser … requires `@@` before chunk lines" is empirically
    false — probe on the unmodified binary), 2 Minor (N2: §5.4 raw-format
    probe not drivable via passive wiretap; N3: multi-`@@` replacement
    pattern untaught), 3 Nit (N4: token undercount; N5: imprecise
    ±1 disclosure; N6: `invocation.rs` line range off).
  - Seat D (`docs/reviews/apply-patch-format-spec-r2-seatD.md`) —
    independent, full source re-verification (14-point appendix: F1
    byte-exact, recount, F2 self-correction, F3, prompt-parity SHA,
    grammar check, T2.2 example executed, parser probes, red state,
    handler, invariants, scaffold, vLLM GitHub citations). 2 Major (M1:
    glm evidence table contradicts the spec's own documented method —
    strict recount gives glm 26/37 ≈ 70% PARSE / 11 OK / 0 VERIFY; the v1
    "VERIFY 7" bucket contained no filesystem failures; the 7 are F4
    boundary rejections. The qwen line reproduces **exactly**, validating
    the pipeline. M2: T4.2's `mount_apply_patch` emits `custom_tool_call`,
    rejected by `FunctionApplyPatchHandler` — the named scaffold cannot
    pass), 3 Minor (m1: the false `@@` note, corroborating C N1; m2:
    End-Patch truncation is a lenient-form-only loss — canonical
    `+*** End Patch` represents the file; m3: length undercount), 3 Nit
    (n1: empty-file creation untaught; n2: rename-only updates
    unrepresentable and untaught; n3: `invocation.rs:233-236` →
    :235-241).

### Round-2 resolution map (v1 → v2)

| Finding | Resolution | v2 location |
|---|---|---|
| D-M1 glm table | Re-tabulated per the strict documented method: glm 26/37 ≈ 70% PARSE (19 content-class + 7 boundary-class F4), OK 11, VERIFY 0; full recount history disclosed (v0 27/276 retracted; seat A 18/36; v1 19/36 mis-bucketed; denominator 36→37); qwen 413 exact reproduction named as method validation | TL;DR, §1.2 |
| D-M1 (class) F4 | Named in §1.1 with exact messages, pre-pass origin (parser.rs:256-272, run at :193-199 before the streaming parser), rollout examples, and per-day subset | §1.1 F4 |
| D-M1 (recommend) | P3.4 added: one-sentence append to both pre-pass messages; streaming parallel messages explicitly out of scope (pre-existing duplication → seam divergence table); red state enumerated (`test_parse_patch`, `test_parse_patch_lenient`) | §3.3.4, §4 T1.14, §6 |
| D-M2 T4.2 scaffold | New helper pair named: `ev_apply_patch_function_call` (responses.rs, pattern :933-942/:1030-1038) + `mount_apply_patch_function_call` (apply_patch_cli.rs); `mount_apply_patch` documented as unusable for the function path (custom_tool_call vs `matches_kind` Function-only, registry.rs:548-556) | §4 T4.2/2b |
| C-N1 / D-m1 false `@@` note | Replaced with the verified statement: parser auto-creates the first chunk from a prefixed line, no `@@` required (streaming_parser.rs:316-360, probed, upstream-pinned at parser.rs:402); real deltas the other way (zero-chunk hunk rejected, second `@@` on empty chunk rejected, new `@@` after `*** End of File` accepted) | §3.1 notes |
| C-N2 §5.4 probe | Deterministic mechanism pinned: standalone release `apply_patch` binary on a scratch dir with the F1-shaped raw patch; wiretap passivity documented as the reason a session cannot be driven | §5.4 |
| C-N3 multi-`@@` rule | P1 Rule 2 now teaches multiple `@@` chunks in the single Update hunk; new drift-guard substring; Example unchanged (T2.2 assertions stand, re-verified by both seats) | §3.1, T2.1 |
| C-N4 / D-m3 length | Re-measured on the final v2 text: 2,324 chars / 380 words ≈ 550–615 tokens; 1K gate noted | §3.1 notes |
| C-N5 disclosure | Recount history written out in full (v0/seatA/v1/canonical, per-model deltas, denominator move, qwen exact-reproduction validation) | §1.2 |
| C-N6 / D-n3 line refs | `invocation.rs:235-241` (message at :237); other cited line refs re-verified and corrected (boundary messages at parser.rs:268/:271) | §3.1 notes, §1.1, §3.3.4 |
| D-m2 truncation justification | Corrected: canonical form represents the file via `+*** End Patch` (probed); truncation is lenient-form-only; T1.9b pins both variants | §3.2, T1.9b |
| D-n1 empty file | P1 Rule 1 teaches "no content lines after the header"; edge item added (parser-accepts, grammar `add_line+` would not; pre- and post-P2) | §3.1, §3.2 |
| D-n2 rename-only | P1 Rule 4 teaches Delete+Add for content-unchanged renames; edge item added (zero-chunk rejection pre-existing; message noted as future-P3 candidate) | §3.1, §3.2 |
| (scope, v2) | §3 scope list gains `apply-patch/src/parser.rs` (P3.4); §6 conflict sites gain `parser.rs`; §3.4 invariant gains the P3.4 pre-pass-only surface refinement; revisit trigger gains F4 | §3, §6, §3.4 |

- **Round 3**: pending — two fresh independent seats on v2, verifying this
  map. Spec may not be declared review-complete until a full round returns
  zero Blocking and zero Major.

