# Responses-Compat — apply_patch Function-Tool Format Redesign

Bead: `apex-ayl.52` (apex_tracking ledger) · Branch: `feat/normalize-content-types-vllm`
Companion SoT: `docs/responses-compat-seam.md` · Evidence: `~/.codex/sessions/2026-09-1{3,4,5}` rollouts
Wire research: `docs/vllm-glm-toolcall-research.md` (separate seat, in flight)

Status: SPEC — round 0 (pre-review). This document is the design for the
redesign; it changes no code until it clears the multi-agent review loop.

## TL;DR

For non-OpenAI providers the seam emits `apply_patch` as a JSON-schema
**function tool** with one required string argument `patch`. The current spec
text says only "the complete patch goes in the `patch` argument as plain
text" — it conveys **none** of the patch format. The format (every content
line carries a one-char prefix: `+` for Add File lines, ` `/`-`/`+` for
Update hunk lines) lives only in the Lark grammar of the freeform
custom-tool variant, which vLLM/SGLang deployments never receive.

Consequence: format correctness depends purely on model training priors.
Measured on the real deployment, 2026-09-13..15:

| Model | apply_patch calls | parse rejections | rate |
|---|---|---|---|
| glm-5.2 | 27 | 18 | **67%** |
| qwen3.8-27b | 276 | 5 | ~2% |

qwen3.8-27b emits the canonical prefixed format from priors; glm-5.2
systematically emits **raw, unprefixed file content** (it reads "plain text"
literally) and is rejected by the canonical parser. The wire is exonerated:
24 KB patches arrived byte-intact; the failures are model content choices.

The fix is three-part: (P1) teach the full format in the function-tool spec
text; (P2) make the parser's Add-File state lenient about the missing `+`
prefix (defense in depth, model-agnostic); (P3) make parse errors
returned to the model teachable. OpenAI/Azure behavior stays byte-identical.

## 1. Problem (verified at the source: session rollouts)

### 1.1 Failure modes observed (exact harness errors)

**F1 — Add File content written raw (no `+` prefix). Dominant, glm-5.2.**

```
apply_patch verification failed: invalid hunk at line 3,
'# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)' is not a valid
hunk header. Valid hunk headers: '*** Add File: {path}',
'*** Delete File: {path}', '*** Update File: {path}'
```

The model sent, verbatim:

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

Model used `@@` as a section separator and dumped unprefixed markdown.

**F3 — Empty arguments `{}`. One qwen3.8-27b session; self-corrected.**

```
apply_patch is missing the required `patch` argument
```

Same session, next call: a fully canonical patch that succeeded. Treated as
glitch, not a systematic defect.

### 1.2 Attribution (rollout forensics, sessions 2026-09-13..15)

- `model` read from each rollout's `turn_context` payload; outcome classes:
  OK (success text), PARSE (hunk errors), VERIFY (post-parse filesystem
  verification failures, e.g. context mismatch — model-side, out of scope),
  MISSING (F3), UNSUPPORTED (pre-seam build), OTHER.
- glm-5.2: PARSE 18 / 27 calls (67%); VERIFY 6; OK 13.
- qwen3.8-27b: PARSE 5 / 276 calls (~2%); VERIFY 36; OK 253; MISSING 1;
  UNSUPPORTED 1; OTHER 1.
- Same `base_instructions` byte-identical across glm- and qwen-seat
  subagents (20,751 chars, `identical: True` in diff) — the seats differ in
  model, not in prompt.
- glm-5.2 runs over provider `cdx1_glm_proxy` (and some `llm_proxy_qwen`
  sessions); qwen3.8-27b over `llm_proxy_qwen`. Both are Responses-API
  deployments behind LiteLLM/vLLM; both receive the identical function-tool
  spec from this fork.

### 1.3 Wire exoneration

- A 24,230-char `patch` argument from glm-5.2 arrived at the harness intact:
  newlines, `+`, `***`, em-dashes, backticks all preserved; the rejection
  quoted line 3 exactly as sent. No mangling on the wire.
- The model's own post-mortem in-session: "The apply_patch tool is rejecting
  the content after the Add File header — it's treating the # line as a
  hunk header. I'll write the file via a heredoc instead" — i.e. the model
  understood the rejection and worked around it; the tool should have
  accepted (or, better, the spec should have taught the format).
- vLLM source confirmation (glm tool-call parser, empty-args conditions,
  guided-decoding behavior for `strict:false`) is being produced in
  `docs/vllm-glm-toolcall-research.md`; expected to find no wire-side
  contributor to F1/F2. If it finds one, §3.4 non-goals get revisited.

## 2. Root cause

1. **The function-tool spec is under-specified by construction.** When the
   seam converted apply_patch from custom/freeform (Lark-grammar-carried
   format) to function tool (no grammar channel on vLLM/SGLang), the format
   knowledge was dropped instead of re-expressed in text. The model's only
   format signals are: its training priors + the words "plain text" (which
   plausibly means "the file's plain text, unprefixed") + the legacy
   `{"command": ["apply_patch", "*** Begin Patch ..."]}` invocation example
   in base instructions (which shows Update-file prefixes but never
   Add-file `+` lines and uses the pre-Responses argument shape).
2. **Model priors differ.** qwen3.8-27b was exposed to canonical Codex
   apply_patch examples in training (98% canonical output); glm-5.2 was not
   (67% raw-content output). Same spec, same harness, different priors —
   which is exactly the situation a *spec text* must handle, because prompt
   is the only model-agnostic channel left once grammar constraints are gone.
3. **The parser is strict where a cheap, unambiguous leniency exists.** In
   the `AddFile` state, the only structural lines are `*** ` markers; any
   other line is content. Rejecting unprefixed content lines (and even
   empty lines) buys nothing — there is no ambiguity to protect — while it
   hard-fails a common model mistake. (Update File is different: unprefixed
   lines there are genuinely ambiguous — context vs addition — so it stays
   strict.)

## 3. Redesign

Scope: `codex-rs/core/src/tools/handlers/apply_patch_spec.rs` (spec text),
`codex-rs/apply-patch/src/streaming_parser.rs` (AddFile state + two error
messages), `codex-rs/core/src/tools/handlers/apply_patch.rs` (function
handler missing-argument message). No capability/routing changes; no
`is_openai()` branches added anywhere.

### 3.1 P1 — Teach the format in the function-tool spec (primary fix)

`create_apply_patch_function_tool` keeps its shape (one required string
argument `patch`, optional `environment_id`, `strict: false`,
`additionalProperties: false`) but gains the full format in the `patch`
argument description. New exact text (stored as a `const` next to the
function; the tool-level description becomes):

```
The `apply_patch` tool can be used to edit files (add, delete, update,
move). The complete patch goes in the `patch` argument as plain text.
```

`patch` argument description (the format block):

```
The ENTIRE patch text as plain text, starting with `*** Begin Patch` and ending with `*** End Patch`. Do not wrap it in JSON, markdown, or a shell heredoc.

FORMAT — the patch is line-oriented; every file-content line carries a one-character prefix that is part of the patch, not part of the file content:

*** Add File: <path>          creates a new file; EVERY following content line starts with '+'
*** Delete File: <path>       deletes a file; takes no content lines
*** Update File: <path>       modifies a file; see hunks below
*** Move to: <path>           optional; renames the file in the current Update hunk
@@ [context line]             starts a change chunk; optional context pins the location
<chunk lines>                 each line starts with ' ' (context/unchanged), '-' (removed), or '+' (added)
*** End of File               optional; chunk extends to end of file

Rules:
- After `*** Add File:`, every line of the new file starts with '+'. An empty file line is a bare '+' with nothing after it.
- In Update hunks, prefix every line: ' ' + line for unchanged context, '-' + line for removal, '+' + line for addition.
- Never write raw (unprefixed) file content lines anywhere in the patch.
- Multiple hunks may appear in one patch, in any order.

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

Design notes:
- The text mirrors `apply_patch.lark` 1:1 (same markers, same prefixes,
  bare `+` for empty lines, optional `@@ [context]`, optional
  `*** End of File`). The Lark file remains the machine SoT; this text is
  its prose rendering for models without grammar constraints.
- A unit test parses the embedded example back through `parse_patch` and
  asserts the expected hunks — the spec we teach must itself be valid
  (drift guard).
- Length ≈ 330 tokens, sent per request for non-OpenAI providers only.
  Accepted cost: apply_patch is the core editing tool and the freeform
  path's grammar payload is comparable in size.
- No format text is added to the OpenAI-facing freeform tool (its grammar
  already constrains decoding; adding prose would churn the OpenAI request
  and violate the byte-identical invariant).

### 3.2 P2 — Add-File leniency in the streaming parser (defense in depth)

In `StreamingPatchParser::process_line`, `StreamingParserMode::AddFile`
arm (streaming_parser.rs), after the hunk-header check fails:

- `+`-prefixed line → content after `+` (unchanged).
- Any other line (including empty, including whitespace-only, including
  lines that merely *start* with `*** ` but match no known marker) →
  **appended verbatim as content** (raw line, then `\n`).
- The final `Err(InvalidHunkError "not a valid hunk header")` in this arm
  is removed (unreachable after the two branches).

Semantics and edges:
- Canonical patches parse byte-identically to today (the `+` branch is
  untouched).
- F1 patterns now parse: `# heading` first line, raw tables, raw markdown,
  empty lines (which are rejected today — a model emitting a blank line
  inside Add-File content currently hard-fails).
- Structural lines inside Add-File (`*** End Patch`, next `*** Add File:`,
  `*** Delete File:`, `*** Update File:`) keep their structural meaning —
  they are handled before the content branches.
- A content line that *looks like* a valid marker (e.g. a file genuinely
  containing `*** Delete File: x` as its first content line) is
  indistinguishable from a marker and remains interpreted as structural.
  This ambiguity is pre-existing and inherent to the format (a canonical
  patch cannot represent such a file either); the leniency does not widen
  it. Documented in the module doc comment alongside the Lark grammar.
- Update-File and Delete-File arms stay strict (see §3.4).

Implementation note: keep the change inside the existing `AddFile` arm
(~8 lines net); do not restructure the state machine. Update the module
doc comment at the top of parser.rs/streaming_parser.rs with a short
"lenient add-file content" note so the divergence is discoverable on
rebase (tracked in the seam doc divergence table, §6).

### 3.3 P3 — Teachable errors

Three messages, minimal edits:

1. `StartedPatch` arm, non-header first line: append
   ` After '*** Begin Patch', the next line must be a hunk header (e.g. '*** Add File: <path>' with every content line prefixed by '+').`
2. `DeleteFile` arm, content line: replace the generic "not a valid hunk
   header" text with
   `'Delete File' hunks take no content lines; the next line must be another hunk header or '*** End Patch'`.
3. Function handler, missing `patch` (apply_patch.rs):
   `apply_patch is missing the required 'patch' argument; pass the full patch text starting with '*** Begin Patch' in 'patch'`.

The Update-File "Every line should start with ' ' (context line), '+'
(added line), or '-' (removed line)" message is already teachable and
stays unchanged.

### 3.4 Non-goals and considered-rejected alternatives

- **No Update-File leniency.** Unprefixed lines inside an Update hunk are
  genuinely ambiguous (context? addition? removal?). Guessing risks silent
  file corruption — worse than a teachable error. F2 models get P1 (format
  taught) + the existing clear error + observed self-correction on retry.
- **No `strict: true` on the function tool.** Server-side schema
  enforcement (guided decoding) varies across vLLM versions/configs and
  would couple tool reliability to deployment flags; it also cannot
  express the per-line format rules anyway (they live inside one string).
- **No base_instructions change** (the legacy `{"command": [...]}`
  apply_patch example). It is upstream-global prompt text shared with the
  OpenAI path (byte-identical invariant) and with the freeform tool where
  it is accurate. P1's argument description explicitly says "the complete
  patch goes in the `patch` argument", which outranks the legacy example
  for the function-tool path. Revisit only if F3-class empty-arg calls
  recur after P1 lands.
- **No heredoc-wrapper leniency extension.** Lenient mode already strips
  `<<'EOF'...EOF` wrappers (gpt-4.1 precedent); no evidence glm needs more
  of that.
- **No per-model parser modes.** The parser sees no model identity;
  leniency is global and additive (canonical input unchanged), matching
  the existing always-on Lenient mode.

## 4. Test plan (TDD, one item at a time, red → green)

Items map 1:1 to implementation tasks (see breakdown doc once reviewed).

**T1 — Parser Add-File leniency** (`codex-rs/apply-patch/`, tests in the
existing suite layout, new module file per repo convention if a new test
module is introduced):
1. F1 replay: Add-File with raw markdown content (`# H1` first line,
   tables, blank lines) parses; resulting `contents` is byte-identical to
   the canonical `+`-prefixed version of the same file.
2. Empty line inside Add-File content → empty line in `contents`
   (newly accepted; was an error).
3. Mixed `+`-prefixed and raw lines → correct concatenation.
4. Raw content line beginning `*** ` but matching no marker → content
   (lenient); `*** End Patch` and each real header line inside Add-File →
   still structural (End terminates, headers start next hunk).
5. Delete-File followed by a content line → new explicit message (§3.3.2).
6. StartedPatch followed by non-header line → original error + appended
   guidance (§3.3.1).
7. CRLF line endings in a raw Add-File → same content as LF version
   (`\r` already stripped by `push_delta`; regression guard).
8. Update-File with raw unprefixed lines → still rejected, message
   unchanged (guards §3.4 first bullet).
9. Canonical patches (existing golden tests) → byte-identical results
   (existing tests must pass unmodified).

**T2 — Function-tool spec** (`apply_patch_spec.rs` + `apply_patch_spec_tests.rs`):
1. Argument description contains the format contract: assert presence of
   the key sentences ("starts with `+`", "bare '+'", "Never write raw",
   the marker lines) — cheap drift guard, not a snapshot of the whole
   block.
2. Self-consistency: extract the example block from the argument
   description, `parse_patch` it, assert the exact hunks (Add File
   `notes/todo.md` contents `# TODO\n\n1. ship the fix\n`; Update File
   `src/main.rs` one chunk, context `fn main`, one removal, one addition,
   one context line).
3. Freeform tool spec unchanged (existing tests cover; no new surface).

**T3 — Function handler** (`apply_patch.rs` + `apply_patch_tests.rs`):
1. Missing `patch` → new teachable message text (§3.3.3).
2. Handler-level: a raw Add-File patch (F1 shape) executes successfully
   against the sandboxed test filesystem (uses existing handler test
   scaffolding).

**T4 — Integration** (`codex-rs/core/tests/suite/`, per repo rule that
agent-logic changes get integration tests):
1. Non-OpenAI provider (test provider with a distinct name): outbound
   `/v1/responses` request carries apply_patch as a **function** tool
   whose `patch` parameter description contains the format contract
   (assert on the captured request body; `core_test_support::responses`
   helpers).
2. Behavioral: mocked model emits `function_call apply_patch` with an F1-
   shaped raw Add-File patch → file exists on disk with exact expected
   contents.

**T5 — Gates** (after each green item, then at the end):
- `just fmt` (codex-rs).
- Targeted suites per the repo Justfile test recipe: `codex-apply-patch`,
  `codex-core` (apply_patch scope), `codex-model-provider` (capability
  gate regression — untouched but cheap), plus `just fix -p` for touched
  crates. No workspace-wide `--all-features` run.

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
5. Regression (qwen3.8-27b): one session doing an Add-File (code file)
   and one Update-File hunk via apply_patch. Pass = both apply cleanly.
6. Cutover: only on operator greenlight; previous binary stays as
   rollback. Post-cutover: observe the next day's rollouts for PARSE-rate
   on glm-5.2 (success metric: <10%, ideally near qwen's ~2%).

## 6. Rollout, rebase, and seam-doc updates

- `docs/responses-compat-seam.md` updates (same PR set):
  - §3 design section: add P1/P2/P3 as the format-remediation layer of the
    function-tool capability; note the 2026-09-15 glm-5.2 evidence.
  - Divergence table: add the apply-patch Add-File leniency as a tracked
    upstream divergence (global parser change, re-apply on parser
    restructure during rebase).
  - Invariants: add "the function-tool `patch` argument description must
    contain the format contract; verified by the self-consistency test".
- Beads: `apex-ayl.52` closed with evidence pointers (this doc, test
  results, rollout before/after PARSE rates).
- No `Cargo.toml`/lock changes expected; if any, `just bazel-lock-update`.

## 7. Review log

- Round 0 (this draft): pre-review, written 2026-09-15 by coordinator.
