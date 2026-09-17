# Responses-Compat — apply_patch Function-Tool Format Redesign

Bead: `apex-ayl.52` (apex_tracking ledger) · Branch: `feat/normalize-content-types-vllm`
Companion SoT: `docs/responses-compat-seam.md` · Wire research: `docs/vllm-glm-toolcall-research.md`
Evidence: `~/.codex/sessions/2026/09-1{3,4,5}` rollouts · v0 archive: `docs/reviews/apply-patch-format-spec-v0-draft.md`

Status: SPEC — v1 (post round-1 review; resolution map in §7). No code changes
until a fresh review round returns zero Blocking and zero Major.

## TL;DR

For non-OpenAI providers the seam emits `apply_patch` as a JSON-schema
**function tool** with one required string argument `patch`. The current spec
text says only "the complete patch goes in the `patch` argument as plain
text" — it conveys **none** of the patch format. The format (every content
line carries a one-char prefix: `+` for Add File lines, ` `/`-`/`+` for
Update hunk lines) lives only in the Lark grammar of the freeform
custom-tool variant, which vLLM/SGLang deployments never receive.

Consequence: format correctness depends purely on model training priors.
Measured on the real deployment, 2026-09-13..15 (method in §1.2):

| Model | unique apply_patch calls | parse rejections | rate |
|---|---|---|---|
| glm-5.2 | 36 | 19 | **~50%** |
| qwen3.8-27b | 413 | 5 | **~1%** |

qwen3.8-27b emits the canonical prefixed format from priors; glm-5.2
systematically emits **raw, unprefixed file content** (it reads "plain text"
literally) and is rejected by the canonical parser. The wire is exonerated by
code and issue-tracker evidence (§1.3): 24 KB patches arrived byte-intact;
the failures are model content choices.

The fix is three-part: (P1) teach the full format in the function-tool spec
text; (P2) make the parser's Add-File state lenient about the missing `+`
prefix (defense in depth, model-agnostic); (P3) make parse errors
returned to the model teachable. Outbound request bytes for providers named
`OpenAI` are unchanged; P2/P3 are provider-agnostic parser changes that only
alter outcomes for inputs upstream rejects (exact invariant wording in §3.4).

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
included; both review seats re-verified):

```
*** Begin Patch
*** Add File: grok/plans/spec-freeze-r23-glm.md
# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)
<...raw markdown file content, no per-line prefixes...>
*** End Patch
```

Canonical format (SoT: `codex-rs/core/assets/tools/apply_patch.lark`,
`add_line: "+" /(.*)/ LF`) requires every Add-File content line to be
`+`-prefixed. The model wrote the file's plain text instead.

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

### 1.2 Attribution (rollout forensics, sessions 2026-09-13..15)

Method: every rollout under `~/.codex/sessions/2026/09-{13,14,15}`; unique
`call_id` for `function_call name=apply_patch`; model read from each
session's first `turn_context` payload; outcome from the matching
`function_call_output` (OK = success text; PARSE = hunk/parse errors;
VERIFY = post-parse filesystem verification failures — model-side context
mismatches, out of scope; MISSING = F3; UNSUPPORTED = pre-seam build).

- glm-5.2: **19 / 36 calls PARSE (~50%)**; OK 10; VERIFY 7.
- qwen3.8-27b: **5 / 413 calls PARSE (~1%)**; OK 374; VERIFY 31; MISSING
  1; UNSUPPORTED 1; OTHER 1.
- Independent recount by review seat A (same method, `docs/reviews/…seatA.md`):
  glm 18/36, qwen 4/405 — the ±1 difference is an attribution boundary
  (sessions with ambiguous `turn_context` ordering); the conclusion is
  identical. v0's table (27/276) used an undocumented per-file filter and is
  retracted in favor of this method.
- Same `base_instructions` byte-identical across glm- and qwen-seat
  subagents (20,751 chars, diff `identical: True`) — the seats differ in
  model, not in prompt.

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
   not (~50% raw-content output). Same spec, same harness, different priors
   — which is exactly the situation a *spec text* must handle, because
   prompt is the only model-agnostic channel left once grammar constraints
   are gone.
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
`codex-rs/apply-patch/src/streaming_parser.rs` (AddFile state + error
messages), `codex-rs/core/src/tools/handlers/apply_patch.rs` (function
handler argument errors). No capability/routing changes; no new
`is_openai()` branches. Size: ~300–450 lines incl. tests, landed as
separate commits per part (P2, P1, P3) plus the seam-doc update; the
800-line rule applies per logical change (the branch's pre-existing seam
diff is a separate change).

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
- After `*** Add File:`, every line of the new file starts with '+'. An empty file line is a bare '+' with nothing after it. A file line that itself starts with '+' gets the prefix on top (file line `+42` → patch line `++42`).
- In Update hunks, prefix every line: ' ' + line for unchanged context, '-' + line for removal, '+' + line for addition.
- Never write raw (unprefixed) file content lines anywhere in the patch.
- Each file may appear in at most one hunk per patch; do not target the same file twice (the tool rejects patches that do).
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

Design notes (v1 revisions marked):
- The FORMAT block is the **canonical** form the model must emit. It
  describes what `StreamingPatchParser` accepts; post-P2 the parser also
  *tolerates* a missing `+` on Add-File lines (safety net, §3.2) — the
  taught form never relies on that. (v1: replaces the inaccurate "mirrors
  the Lark 1:1" claim; the .lark remains the machine SoT for the freeform
  path, and the parser accepts a subset in Update hunks, e.g. it requires
  `@@` before chunk lines.)
- (v1, seat B M2/M6) The v0 wording is dropped wholesale: "plain text"
  (the literal-reading F1 trigger) and "Do not wrap it in JSON" (false —
  the argument IS a JSON string value) are gone; the backslash-n rule
  neutralizes the legacy base-instructions example (`default.md:132`) for
  the function-tool path without touching that upstream-global text.
- (v1, seat B M3) One-hunk-per-file is now taught (enforced at
  `invocation.rs:233-236`, "multiple operations target {path}").
- (v1, seat A m3) Drift guard (T2.1) asserts these exact literal
  substrings: `first line is `*** Begin Patch``, `real newline characters`,
  `bare '+'`, `at most one hunk per patch`, `starts with '+'`.
- Self-consistency test (T2.2) parses the embedded example back through
  `parse_patch` and asserts the exact hunks (extraction rule pinned in T2).
- Length ≈ 320 tokens, sent per request for non-OpenAI providers only.
  Accepted cost: apply_patch is the core editing tool; the freeform path's
  grammar payload is comparable in size.
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

Semantics and edges (v1: list completed per review):
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
- **`*** End Patch` as a final content line (seat B m2):** a file whose
  last line is exactly `*** End Patch` is truncated at that line
  (structural check runs before content branches) — pre-existing and
  inherent (a canonical patch cannot represent such a file either); pinned
  by test (T1.9b).
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

Three message sites, minimal edits (quote style kept consistent with
existing messages, single quotes around user text):

1. `StartedPatch` arm, non-header first line: **keep the original sentence
   verbatim** (existing tests match it as a substring) and append:
   ` After '*** Begin Patch', the next line must be a hunk header (e.g. '*** Add File: <path>' with every content line prefixed by '+'), or '*** Environment ID: <id>' in multi-environment sessions.`
2. `DeleteFile` arm, content line: replace the generic "not a valid hunk
   header" text with
   `'Delete File' hunks take no content lines; the next line must be another hunk header or '*** End Patch'`.
3. Function handler argument errors (apply_patch.rs):
   - `patch` absent from the JSON object:
     `apply_patch is missing the required 'patch' argument; pass the full patch text starting with '*** Begin Patch' in 'patch'`.
   - (v1, seat B m5) `patch` present but not a string:
     `apply_patch 'patch' argument must be a string containing the full patch text starting with '*** Begin Patch'`.

The Update-File "Every line should start with ' ' (context line), '+'
(added line), or '-' (removed line)" message is already teachable and
stays unchanged.

Existing tests that **necessarily change** under P2/P3 (the red state,
per seat B m4): in `test_streaming_patch_parser_returns_errors`
(streaming_parser.rs:813) the three assertions for the StartedPatch 'bad'
message (P3.1 append), AddFile 'bad' (behavior removed by P2 — becomes
`Ok([AddFile{contents: "bad\n"}])`), and DeleteFile 'bad' message (P3.2
replacement); and `test_apply_patch_cli_rejects_invalid_hunk_header`
(`codex-rs/apply-patch/tests/suite/tool.rs:386`) which asserts the exact
CLI stderr of the StartedPatch message. The core-suite test
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
  (v1, broadened): any recurrence of F1/F2/F3-class failures after this
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

**Invariant (v1 exact wording, replaces v0's "byte-identical"):**
Outbound **request bytes for providers named `OpenAI` are unchanged** by
this work (P1 touches only the function-tool spec, which never ships to
them). P2 and P3 are **provider-agnostic parser changes**: for every input
upstream accepts, behavior is byte-identical; for inputs upstream
*rejects*, P2 turns some Add-File rejections into accepted content and P3
turns some rejections into teachable errors. Full surface of that
divergence: the function-tool execution path, the freeform diff consumer
(`ApplyPatchArgumentDiffConsumer` streams the same parser — raw Add-File
content now streams as diffs instead of erroring), and the `apply_patch`
CLI. The capability gate itself is unchanged: `is_openai()` is
`name == "OpenAI"` with **no Azure branch** (`model-provider-info/src/lib.rs:546`)
— so any Azure-*named* provider is in seam scope (receives the function
tool + P1 text); the seam doc §2 "plus an Azure branch" description is
stale and is corrected in the same PR set (v1, seat B M5 / seat A M2).
This change is **not verified against any Azure deployment** (none in use);
Principle-2 verification covers the vLLM/LiteLLM deployments only.

## 4. Test plan (TDD, one item at a time, red → green)

Items map 1:1 to implementation tasks. T1.4/T1.5/T1.6's rewrites of the
four existing assertions listed in §3.3 are the initial red state.

**T1 — Parser Add-File leniency** (`codex-rs/apply-patch/`; new test module
in the existing `tests/suite/` layout or adjacent `*_tests.rs` per repo
convention):
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
9b. Added file whose last content line is `*** End Patch` → truncated at
   that line (pins the §3.2 inherent-ambiguity case).
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

**T2 — Function-tool spec** (`apply_patch_spec.rs` + `apply_patch_spec_tests.rs`):
1. Drift guard: assert the exact literal substrings listed in §3.1
   (`first line is `*** Begin Patch``, `real newline characters`, `bare
   '+'`, `at most one hunk per patch`, `starts with '+'`).
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
   expected contents (uses `mount_apply_patch` + `harness.submit` +
   `read_file_text` from `apply_patch_cli.rs`).
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
   **Plus a raw-format probe (v1, seat B M4):** drive one deliberate
   unprefixed-Add-File call (e.g. via the same harness) and confirm P2's
   leniency executes — so a future rebase that silently drops P2 fails
   this smoke, not just the (previously un-gated) unit suite.
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
  capability, with the 2026-09-15 glm-5.2 evidence (19/36 vs 5/413).
- Divergence table: one row for the apply-patch parser change —
  **provider-agnostic**, surface = function-tool path + freeform diff
  consumer + `apply_patch` CLI (v1, seat B n2), re-apply on upstream
  restructure of `streaming_parser.rs` during rebase.
- §6 conflict sites: add `codex-rs/apply-patch/src/streaming_parser.rs`
  (v1, seat B M4).
- §6 verification gate: **add `just test -p codex-apply-patch`** and the
  raw-format glm smoke (this doc §5.4) — v0's gate would have passed a
  rebase that silently reverted P2 (v1, seat B M4).
- Invariants: add the §3.4 exact-invariant text (request-bytes for
  `OpenAI`-named providers unchanged; P2/P3 = provider-agnostic changes
  affecting only inputs upstream rejects).

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
| A-M1 evidence table | Retried recount with documented method; glm 19/36 (~50%), qwen 5/413 (~1%); v0 numbers retracted; ±1 vs seat A's recount disclosed | TL;DR, §1.2 |
| A-M2 / B-M5 / B-n2 invariant | "Byte-identical" replaced with exact request-bytes invariant; P2/P3 named as provider-agnostic changes affecting only inputs upstream rejects; full surface (function path, diff consumer, CLI); Azure claim corrected, no Azure verification | §3.4, §6 |
| B-M1 overwrite interaction | Full analysis added (no existence check pre-existing; P2 changes reachability); **decision: no existence check on any path** with rationale; pinned by T1.9 + T4.2b | §3.2, §3.4, T1/T4 |
| B-M2 wording | "plain text" and "Do not wrap it in JSON" removed; opening sentence rewritten | §3.1 |
| B-M3 hunk constraint | "at most one hunk per file" rule taught; drift guard substring | §3.1, T2.1 |
| B-M4 rebase gate | `just test -p codex-apply-patch` + raw-format smoke added to seam §6 gate; `streaming_parser.rs` added to conflict sites | §5.4, §6 |
| B-M6 base instructions | Backslash-n rule added to P1 (function-tool-only, invariant-safe); revisit trigger broadened to any F1/F2/F3 recurrence | §3.1, §3.4 |
| A-m1 / B-m3 extraction rule | Pinned: split at first line exactly `Example:`; remainder must end with `*** End Patch` | T2.2 |
| A-m2 snapshot test | Existing `create_apply_patch_function_tool_matches_expected_spec` explicitly updated | T2.3 |
| A-m3 drift guard | Exact literal substrings listed | §3.1, T2.1 |
| A-m4 "1:1 mirror" | Claim replaced with accurate parser-subset wording | §3.1 notes |
| A-m5 edge list | Whitespace-only / env-id / unclosed / padded-End edges added with tests | §3.2, T1.7/1.10 |
| A-m6 silent-partial-apply | Named class + test pin | §3.2, T1.8 |
| A-n1 env-id in P3.1 | Guidance now mentions `*** Environment ID:` first line | §3.3.1 |
| A-n2 doc-comment drift | One clarifying line added; no full reconciliation | §3.2 impl note |
| A-n3 quote style | Consistent single-quote style | §3.3 |
| A-n4 PR splitting | Separate commits per part stated; 800-line rule per logical change | §3 scope |
| B-m1 lossy `+` case | Named (only lossy case) + `++` taught + test pin | §3.1 rules, §3.2, T1.9 |
| B-m2 End-Patch truncation | Named + test pin | §3.2, T1.9b |
| B-m4 test-plan scope | Four rewritten assertions enumerated as the red state; T1.13 scoped to golden/canonical | §3.3, T1.4-6/12/13 |
| B-m5 non-string patch | Separate error message added | §3.3.3, T3.1 |
| B-n1 24,230 chars | Clarified (arguments-JSON vs patch-string length) | §1.3 |
| (vLLM research addendum) | §1.3 expanded (wire exoneration by code+issues; F3 mechanism #49248; strict/forced-choice constraints); §3.4 strict bullet cited; deployment-side follow-up + watch metric | §1.3, §3.4, §5.6 |

- **Round 2**: pending (fresh independent seats on v1, verifying this map).
