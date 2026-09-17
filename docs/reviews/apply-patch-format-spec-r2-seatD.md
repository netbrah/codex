# Review — apply-patch function-tool format spec (v1), Round 2, Seat D

Target: `docs/responses-compat-apply-patch-format.md` (v1, 619 lines)
Reviewer: seat D (independent; no earlier reviews read). Date: 2026-09-15.

## Verdict

**CHANGES-REQUESTED** — 0 Blocking, 2 Major, 3 Minor, 3 Nit.

The design (P1/P2/P3) is sound and nearly every load-bearing claim
verifies at the source: the F1 evidence is byte-exact, the qwen evidence
line reproduces exactly, the parser/handler/routing/invariant claims all
hold, and the vLLM citations check out on GitHub. The two Majors are:
(M1) the glm evidence table is not reproducible under the spec's own
documented method — the "VERIFY 7" bucket contains no filesystem
verification failures at all — and (M2) T4.2 names a test scaffold that
cannot drive the function-tool path it is supposed to test.

## Findings

### M1 — Major — glm evidence table contradicts the spec's own method; "VERIFY 7" does not exist as defined

- **Spec ref:** TL;DR table; §1.2 (attribution); §2.2 ("~50% raw-content
  output"); §6 ("19/36 vs 5/413"); §7 resolution map (A-M1 row).
- **Finding:** Recounting every rollout under `~/.codex/sessions/2026/09-15`
  per the method §1.2 documents (first `turn_context` model per file,
  unique `call_id`, outcome from the matching `function_call_output`)
  gives **glm-5.2: 37 calls = 26 PARSE + 11 OK + 0 VERIFY**, not
  "19 / 36 PARSE; OK 10; VERIFY 7". The qwen line reproduces **exactly**
  (413 = 374 OK + 5 PARSE + 31 VERIFY + 1 MISSING + 1 UNSUPPORTED + 1
  OTHER), which validates the recount pipeline — the glm row is the
  outlier. The 7 glm calls the spec buckets as "VERIFY — post-parse
  filesystem verification failures — model-side context mismatches, out
  of scope" are not filesystem failures at all: zero glm call in the
  three days failed with `Failed to find expected lines` / `Failed to
  read file`. They are 7 **structural parse errors** — 5× `The first line
  of the patch must be '*** Begin Patch'` + 2× `The last line of the
  patch must be '*** End Patch'` — which (a) violate the spec's own
  VERIFY definition and (b) are in-scope format failures that P1
  directly addresses (P1 teaches both boundary markers).
- **Evidence:**
  - Rollouts, e.g. `~/.codex/sessions/2026/09/14/rollout-2026-09-14T03-25-55-*`
    (call `call_17d7edbb…`: patch is just `*** Add File: …` + `*** End
    Patch` — no Begin marker, no content), `…/2026-09-14T07-58-02-*`
    (call `call_10f109a3…`: ends with `+*** End Patch`, never a bare
    terminator), `…/2026-09-15T14-33-27-*` (call `call_bf4d74bc…`:
    first line is `*** Add File:`, Begin marker missing).
  - Mechanism: these messages come from
    `check_start_and_end_lines_strict` in
    `codex-rs/apply-patch/src/parser.rs:274-283` (the pre-pass in
    `parse_patch_text`, parser.rs:193-210), not from the streaming
    parser — which is why P3's three sites (all streaming-parser /
    handler messages) never teach a model how to fix them, and why
    §1.1's F1/F2/F3 list does not name this failure class at all.
  - The spec's own method section defines PARSE as "hunk/parse errors"
    and VERIFY as "post-parse filesystem verification failures"; under
    those definitions glm = 26 PARSE / 0 VERIFY. The 19/7 split only
    works if "hunk/parse errors" means hunk-level messages and
    "VERIFY" is a catch-all for other `verification failed` prefixes —
    i.e., the table and its definitions disagree.
  - Consequence: headline glm rate is ~70% (26/37), not ~50% (19/36).
    09-15 subset: 13 PARSE / 8 OK of 21 glm calls.
  - Aggravating: the §7 resolution map asserts this exact table comes
    from a "retried recount with documented method" (A-M1 row) — the
    documented method does not yield the table.
- **Resolution:** Re-tabulate §1.2/TL;DR per the documented method
  (glm 26/37 ≈ 70% PARSE, OK 11, VERIFY 0; keep the ±1 total disclosure
  as an attribution-boundary note). Name the structural class in §1.1
  (e.g. F4 — missing boundary markers) and state explicitly whether P3
  covers the boundary messages in `check_start_and_end_lines_strict`
  (recommend: at minimum name it; P1 already teaches both markers, and a
  one-line append to those two messages would be cheap and in the spirit
  of P3). Update §2.2 and the §6 evidence pointer.

### M2 — Major — T4.2/T4.2b scaffold mismatch: `mount_apply_patch` cannot drive the function-tool path

- **Spec ref:** §4, T4.2 and T4.2b ("Behavioral: mocked model emits
  `function_call apply_patch` … (uses `mount_apply_patch` + `harness.submit`
  + `read_file_text` from `apply_patch_cli.rs`)").
- **Finding:** `mount_apply_patch`
  (`codex-rs/core/tests/suite/apply_patch_cli.rs:228-240`) always builds
  the SSE mock from `ev_apply_patch_custom_tool_call`
  (`codex-rs/core/tests/common/responses.rs:1008-1018`), which emits a
  `custom_tool_call` item. In a session whose provider is renamed to a
  non-OpenAI name (T4.1's setup), only `FunctionApplyPatchHandler` is
  registered (spec_plan.rs diff), and its `matches_kind` accepts only
  `ToolPayload::Function` (apply_patch.rs:603-605). The registry rejects
  the kind-mismatched custom payload — "tool apply_patch invoked with
  incompatible payload" (`codex-rs/core/src/tools/registry.rs:549`,
  message at :821). So T4.2 as written cannot pass: the P2 behavioral
  pin (raw Add-File patch executes on the function path) is not
  executable with the named scaffold.
- **Evidence:** `apply_patch_cli.rs:228` (helper body uses
  `ev_apply_patch_custom_tool_call` only; the only alternative in the
  file, `ev_apply_patch_exec_command_call_via_heredoc`, routes through
  `exec_command`, not `apply_patch`); `responses.rs:933-942`
  (`ev_function_call` generic constructor exists and is the pattern used
  at responses.rs:1030-1035); `registry.rs:549`.
- **Resolution:** Add a function-call variant (e.g.
  `mount_apply_patch_function_call` using
  `ev_function_call(call_id, "apply_patch", json!({"patch": …}).to_string())`)
  and update T4.2/2b to name it. `read_file_text` (harness method, used
  e.g. at apply_patch_cli.rs:323) and `harness.submit` are fine as-is.

### m1 — Minor — design note: "parser … requires `@@` before chunk lines" is factually wrong

- **Spec ref:** §3.1 design notes (v1 A-m4 replacement: "the parser
  accepts a subset in Update hunks, e.g. it requires `@@` before chunk
  lines").
- **Finding:** The parser does **not** require `@@` before chunk lines.
  The `UpdateFile` arm auto-creates a chunk when the first prefixed line
  arrives (`if chunks.is_empty() { chunks.push(UpdateFileChunk::default()); }`
  in each of the ` ' `/`+`/`-` branches, streaming_parser.rs:316-360).
  Empirically: an Update hunk with **no `@@` at all**
  (`-    old_call();` / `+    new_call();` directly under
  `*** Update File:`) parses and applies with the unmodified binary.
  The Lark grammar also does not require a leading `@@` (`change:
  (change_context | change_line)+` admits a `change_line` first). The
  actual parser-strict-vs-grammar direction is the reverse one: the
  parser rejects zero-chunk Update hunks the grammar admits (`change?`
  — e.g. move-only updates; pinned upstream by the "Update file hunk for
  path … is empty" assertions in
  `test_streaming_patch_parser_returns_errors`, streaming_parser.rs
  ~866-880).
- **Evidence:** probe run — `apply_patch` binary on
  `*** Begin Patch\n*** Update File: main.rs\n-    old_call();\n+    new_call();\n*** End Patch`
  → `Success. … M main.rs`; streaming_parser.rs UpdateFile arm;
  apply_patch.lark `change:` rule.
- **Resolution:** Replace the parenthetical with the accurate statement
  (parser ⊋ grammar for `@@`-less chunks; parser ⊊ grammar for
  move-only/zero-chunk updates). The design decision (teach canonical)
  is unaffected.

### m2 — Minor — End-Patch truncation justification is false for canonical form

- **Spec ref:** §3.2, "**`*** End Patch` as a final content line (seat B
  m2)** … (a canonical patch cannot represent such a file either)".
- **Finding:** A canonical patch **can** represent a file whose last line
  is exactly `*** End Patch`: prefix it (`+*** End Patch`) — the
  structural check runs on the *trimmed, un-prefixed* line, so the `+`
  branch takes it as content. Empirically: the unmodified binary applied
  `*** Add File: tricky.txt` / `+line1` / `+*** End Patch` / `*** End
  Patch` and wrote `tricky.txt` = `line1\n*** End Patch\n`. The
  truncation corner is a lenient-form-only loss, which weakens the
  "inherent" justification (the taught form is complete; only the P2
  safety net is lossy here).
- **Evidence:** probe run above; streaming_parser.rs AddFile arm (header
  check on `trimmed`, `+` branch on `line`).
- **Resolution:** Keep T1.9b and the decision; correct the justification
  to "canonical form represents the file via `+*** End Patch`; only the
  unprefixed (lenient) form truncates".

### m3 — Minor — P1 length estimate undercounted

- **Spec ref:** §3.1 design notes ("Length ≈ 320 tokens, sent per
  request for non-OpenAI providers only").
- **Finding:** The proposed text is 2,017 chars (tool description 126 +
  `patch` argument description 1,891) ≈ 450–580 tokens by standard BPE
  counting — roughly 1.5× the stated figure. Still far under the 1K-token
  highlight gate, so no design impact; but the "accepted cost" argument
  should state the real number.
- **Evidence:** measured from the §3.1 code block in the spec.
- **Resolution:** Restate as ≈ 500 tokens (or measure against the actual
  tokenizer used for the provider and cite it).

### n1 — Nit — empty-file creation unaddressed in the FORMAT block

- **Spec ref:** §3.1 FORMAT block + Rules.
- **Finding:** The parser (pre- and post-P2) accepts an Add-File hunk
  with zero content lines (`*** Add File: x` immediately followed by the
  next marker → empty file; verified with the unmodified binary), while
  the grammar's `add_line+` does not. P1's "EVERY following content line
  starts with '+'" is vacuously satisfied, so nothing conflicts — but a
  model asked to create an empty file has no explicit instruction.
- **Resolution:** Optional one-liner in Rules ("To create an empty file,
  write no content lines after the header").

### n2 — Nit — rename-only update is unrepresentable, untaught, and its error is untaught

- **Spec ref:** §3.1 FORMAT block; §3.3 (three sites).
- **Finding:** `*** Update File: a` + `*** Move to: b` with no chunk is
  valid per the grammar (`change?`) but rejected by the parser ("Update
  file hunk for path 'a' is empty"); P1 never mentions rename-only
  updates, and that message is not among P3's three teachable sites.
  Pre-existing, rare; noted for completeness so it is a known corner.
- **Resolution:** None required; optionally add the message to a future
  P3 pass or note it in §3.2's edge list.

### n3 — Nit — line-reference drift

- **Spec ref:** §3.1 design notes ("enforced at `invocation.rs:233-236`").
- **Finding:** The one-hunk-per-file check is at
  `codex-rs/apply-patch/src/invocation.rs:235-241` (message at :237);
  233-234 are the loop start and path resolution. All other cited
  locations verified accurate: `model-provider-info/src/lib.rs:546`,
  `apply_patch_spec_tests.rs:40`, `streaming_parser.rs:813` (test fn
  starts :814), `apply-patch/tests/suite/tool.rs:386`,
  `core/tests/suite/apply_patch_cli.rs:707`, `default.md:132`.
- **Resolution:** Fix the range to 235-241.

## Counts

Blocking: **0** · Major: **2** (M1, M2) · Minor: **3** (m1-m3) · Nit: **3** (n1-n3)

Per the spec's own convergence rule, round 2 does not clear while 2
Major findings stand.

## What I verified (appendix)

1. **F1 rollout evidence** —
   `~/.codex/sessions/2026/09/15/rollout-2026-09-15T18-42-54-01a0a73c-*`:
   arguments JSON len 24,230; `patch` string len 23,972 (spec §1.3
   exact); header lines byte-identical to the §1.1 F1 quote including
   U+2014; harness error string matches the quote exactly. The retry
   (call `call_c4531ddc…`) failed identically; the model then fell back
   to a heredoc `exec_command`, matching §1.3's post-mortem quote.
2. **Recount (09-13..15, spec's documented method)** — qwen3.8-27b:
   413 = 374 OK + 5 PARSE + 31 VERIFY + 1 MISSING + 1 UNSUPPORTED + 1
   OTHER — **exact match** to §1.2. glm-5.2: 37 = 26 PARSE + 11 OK + 0
   VERIFY — contradicts §1.2 (see M1). 09-15 subset: 13/21 PARSE.
   No call_id has multiple outputs; no cross-file call_id collisions.
   glm PARSE breakdown: 13 F1-class (raw Add-File content), 2 F2-class
   (raw Update content), 3 `@@`-as-first-content-line, 1 `---`
   first-content-line, 5 missing `*** Begin Patch`, 2 missing
   `*** End Patch` (incl. `+*** End Patch` as final line).
3. **F2 self-correction** — session `…/2026-09-15T15-46-52-*`: first
   call has unprefixed `## PART B concordance (…)` (exact F2 error);
   retry prefixes the same section with `-` lines and succeeds.
4. **F3** — qwen session `…/2026-09-15T17-57-04-*`: call 1 args
   exactly `{}` → "apply_patch is missing the required `patch`
   argument"; call 2 (canonical Add-File) succeeds.
5. **base_instructions parity** — `session_meta.base_instructions.text`:
   glm-5.2 and qwen3.8-27b seats share one SHA-256 (prefix
   `ac8ae107a0d7`), 20,751 chars — matches §1.2.
6. **Grammar vs P1** — every FORMAT-block marker/prefix/optionality
   statement checked against `codex-rs/core/assets/tools/apply_patch.lark`
   (Add/Delete/Update/Move/`@@`/chunk lines/`*** End of File`/
   Begin/End); the `add_line: "+" /(.*)/ LF` quote is verbatim.
   Exceptions: m1 (design note) and the silent corners n1/n2.
7. **T2.2 example executed through the unmodified parser** (built
   `apply_patch` binary): `notes/todo.md` written as exactly
   `# TODO\n\n1. ship the fix\n` (24 bytes); `src/main.rs` chunk = 1
   chunk, `change_context` = `fn main`, 1 removal, 1 addition, 1
   context line — matches T2.2's asserted hunks. All five T2.1
   drift-guard substrings present in the §3.1 text.
8. **Parser baseline behavior (pre-P2 binary probes):** raw Add-File
   content → `invalid hunk at line 3, … not a valid hunk header`
   (F1 reproduction); `++44 20 7946 0958` → content `+44 20 7946 0958`
   (lossy case, spec §3.2 exact); empty Add-File accepted (0-byte
   file); `+*** End Patch` content line accepted (m2); `@@`-less update
   chunk accepted (m1).
9. **§3.2 edge list** — verified against
   `codex-rs/apply-patch/src/streaming_parser.rs`: AddFile arm
   structure (header check → `+` branch → Err); CRLF strip in
   `push_delta`; trimmed marker matching; `*** Environment ID:`
   honored only in `StartedPatch`; End-Patch structural before content
   branches; UpdateFile/DeleteFile arms strict.
10. **Red-state assertions** — `test_streaming_patch_parser_returns_errors`
    (streaming_parser.rs:814) contains exactly the three claimed
    assertions (StartedPatch 'bad' :824-832, AddFile 'bad' :834-843,
    DeleteFile 'bad' :845-854); `test_apply_patch_cli_rejects_invalid_hunk_header`
    (tool.rs:386) asserts the exact CLI stderr of the StartedPatch
    message; `core/tests/suite/apply_patch_cli.rs:707` uses substring
    matches that survive P3.1. No other test asserts any of the
    changed message strings (searched for "missing the required" and
    "not a valid hunk header" across codex-rs).
11. **Handler (P3.3 sites)** — `FunctionApplyPatchHandler::handle_call`
    (apply_patch.rs:508-560): `value.get("patch").and_then(Value::as_str)`
    collapses absent and non-string `patch` into one error — the
    §3.3.3 indistinguishability claim holds. Freeform handler,
    `run_apply_patch_text`, and `intercept_apply_patch` share the
    "apply_patch verification failed: {parse_error}" prefix;
    `ParseError` Display formats match the quoted errors
    (parser.rs:57-60).
12. **Invariants** — `is_openai()` at
    `codex-rs/model-provider-info/src/lib.rs:546` = `name == "OpenAI"`,
    no Azure branch; capability at
    `codex-rs/model-provider/src/provider.rs:372`
    (`apply_patch_function_tool: !self.info.is_openai()`; Bedrock
    hardcoded false in `amazon_bedrock/mod.rs`); routing in the working
    tree `codex-rs/core/src/tools/spec_plan.rs` diff (function handler
    iff capability); built-in "openai" provider name is "OpenAI"
    (lib.rs:465) so `test_codex()` defaults to the freeform path and
    `with_config` renaming runs after provider assignment
    (test_codex.rs:858-881) — T4.1 feasible. P1 text therefore ships
    only to non-OpenAI-named providers. The seam doc's stale "plus an
    Azure branch" text exists (`docs/responses-compat-seam.md:54`), so
    §6's correction is grounded.
13. **Test scaffolding** — `invocation_for_payload`
    (apply_patch_tests.rs:45) + `make_session_and_context`
    (session/tests.rs:5882); end-to-end handler invocation pattern
    exists (unified_exec_tests.rs:208) → T3 feasible; fixtures under
    `codex-rs/apply-patch/tests/fixtures/scenarios/` (20 dirs, incl.
    `011_add_overwrites_existing_file`); `test_apply_patch_cli_add_overwrites_existing_file`
    (tool.rs:349); snapshot tests at apply_patch_spec_tests.rs:5 (freeform)
    and :40 (function) — T2.3/T2.4 grounded; `read_file_text` harness
    method used at apply_patch_cli.rs:323. `invocation.rs:242-245`
    Add-File has no existence check; `lib.rs:504-535` overwrites —
    §3.2's overwrite analysis holds. **Exception: M2** (`mount_apply_patch`
    is custom-path only).
14. **vLLM citations (GitHub, vllm-project/vllm main)** —
    `vllm/parser/glm47_moe.py`: tag constants and `_ARG_RE`
    (`<arg_key>…</arg_key>\s*<arg_value>…</arg_value>`, re.DOTALL)
    require the opening `<arg_value>` tag; `_glm47_arg_converter`
    copies `match.group("value")` raw (no strip; JSON escaping only).
    Issue #49248 (open): missing opening tag → `{}`, ~5–17% at 6-way
    concurrency, 0 sequential, temp 0, GLM-5.2 — matches spec §1.3.
    PR #49249 (optional opening tag): open, `merged: false` — matches
    "fix … open, not in main".
