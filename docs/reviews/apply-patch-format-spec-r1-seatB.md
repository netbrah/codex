# Review — responses-compat apply_patch format spec (round 1, seat B, adversarial)

**Verdict: CHANGES-REQUESTED** — 0 Blocking, 6 Major, 5 Minor, 2 Nit (Blocking+Major = 6).

Spec under review: `docs/responses-compat-apply-patch-format.md` (round 0 draft).
Reviewer: independent seat B (adversarial). No other review input read.

Verification performed at the source (not from the spec):
- Grammar SoT: `codex-rs/core/assets/tools/apply_patch.lark` (read in full).
- Parser: `codex-rs/apply-patch/src/streaming_parser.rs` (all four `process_line`
  mode arms, `handle_hunk_headers_and_end_patch`, `push_delta`, `finish`),
  `parser.rs` (`parse_patch` → `parse_patch_text`, `ParseMode::Lenient` heredoc
  stripping, boundary checks), `invocation.rs` (verify phase), `lib.rs`
  (`apply_hunks_to_files`). **No second parser pass exists**: `parse_patch` is a
  batch wrapper around the same `StreamingPatchParser` (parser.rs:193-208); the
  verify phase (`try_verify_apply_patch_args`) inspects filesystem state, it does
  not re-parse. P2 is therefore neither bypassed nor contradicted by a verifier.
  The one other `StreamingPatchParser` consumer is the freeform diff consumer
  (`core/src/tools/handlers/apply_patch.rs:90`).
- Uncommitted diff: `apply_patch_spec.rs` (function tool + `create_apply_patch_function_tool`),
  `apply_patch.rs` (JSON extraction at :535, `with_environment_id_line` at :569,
  shared `run_apply_patch_text`), `spec_plan.rs` (capability-gated registration),
  `provider.rs` (`apply_patch_function_tool: !is_openai()`).
- Rollout `~/.codex/sessions/2026/09/15/rollout-2026-09-15T18-42-54-...jsonl`:
  4 `function_call name=apply_patch` payloads extracted with python3. The two
  failed F1 calls (`call_caf7ba...`, `call_c4531...`) match the spec §1.1 F1 quote
  **byte-for-byte**, including the em-dash in line 3
  (`# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)`) and the exact
  harness error text in the corresponding `function_call_output`. A third failed
  call (`call_cda093...`) is F2-shaped (raw context line in an Update hunk, error
  "Unexpected line found in update hunk"); a fourth (`call_17e7...`) is canonical
  and succeeded.
- Live parser probes: built `codex-rs/target/debug/apply_patch` (unmodified
  working tree) and ran 8 constructed counter-examples in temp dirs
  (results in §a below; probe scripts kept in /tmp/appatch-*).

## Findings

### Major

**[Major] M1 — P2 leniency + pre-existing silent Add-File overwrite = new silent-corruption path for malformed patches.**
Spec §3.2 (P2 semantics), §4 (test plan).
Evidence: an `Add File` hunk has **no existence check anywhere** — verify
(`invocation.rs:242-245` just inserts the change) and apply
(`lib.rs:505-528`: `read_optional_file_text_for_delta` tolerates an existing
file, then `write_file_with_missing_parent_retry` overwrites; the code even
records `overwritten_content`). Probe A against the unmodified binary:
`*** Add File: existing.md` + canonical `+` lines over a file containing two
lines → exit 0, file silently replaced, tool output only "A existing.md".
Today, the *malformed* variant of this patch (raw, unprefixed lines — the F1
shape) fails loudly in the AddFile arm (streaming_parser.rs:201-214), and the
rollout proves glm-5.2 recovers from that loud failure. Under P2 the same
malformed patch — `*** Add File: <existing-path>` + raw content, the natural
weak-model confusion between "create" and "edit" (the rollout shows exactly
this confusion class: F1 Add-for-new, F2 Update-attempts) — parses and
**silently overwrites the existing file**, with the model seeing only
"Success ... A <path>". P2 converts a recoverable error into silent data loss
for exactly the target model class; the spec neither considers, acknowledges,
nor tests this interaction.
Resolution: (a) add this interaction explicitly to §3.2 "Semantics and edges";
(b) add a T1 unit test pinning the outcome of a raw Add-File targeting an
existing path (overwrite, as-is, so any future guard is a conscious change);
(c) add a T4.2b integration test for the existing-file variant; (d) decide
explicitly in §3.4 whether the function-handler path should reject
Add-File-to-existing (with a teachable error) — if yes, that is a scoped
divergence from the freeform path and must be said so.

**[Major] M2 — P1 text retains the F1 trigger phrase "plain text" and a sentence the model can read as a contradiction: "Do not wrap it in JSON".**
Spec §3.1 (new argument description).
Evidence: the spec's own root-cause §2 attributes F1 partly to the model
reading "plain text" literally ("plausibly means 'the file's plain text,
unprefixed'"), yet the new text keeps the phrase twice ("The ENTIRE patch text
as plain text..."; tool description "...as plain text."). Worse, "Do not wrap
it in JSON" is directly at odds with reality: the `patch` argument is a JSON
string inside a JSON arguments object — `FunctionApplyPatchHandler` does
`serde_json::from_str::<Value>(arguments).get("patch").and_then(as_str)`
(apply_patch.rs:535-540). A weak model honoring "no JSON" plausibly (1) emits
literal backslash-n sequences instead of real newlines (mirroring the legacy
base-instructions example, see M6), or (2) emits a JSON object/number for the
value — which surfaces as the *misleading* "missing the required `patch`
argument" error. The deployment receiving this text is precisely the one whose
model has already misread the old wording once.
Resolution: drop "plain text" (say "as text"); replace the JSON sentence with
a precise value contract, e.g. "The argument value is the raw patch text
itself: lines separated by real newlines (never the two characters
backslash + n), with no JSON object, extra quotes, or heredoc wrapper around
it."

**[Major] M3 — P1 teaches "Multiple hunks may appear in one patch, in any order" without the one-hunk-per-file constraint or the multi-@@ pattern.**
Spec §3.1 (Rules bullet 4).
Evidence: `try_verify_apply_patch_args` rejects repeated paths with
`"multiple operations target {path}"` (invocation.rs:235-240). Two
`*** Update File:` hunks for the same file — the natural strategy for editing
two distant regions — is the most likely way a weak model exploits the
"any order" permission, and it fails with an error that does not say what to
do instead. The grammar supports the correct pattern
(`update_hunk: ... change?` with `change: (change_context | change_line)+`,
i.e. multiple `@@` chunks inside one `*** Update File:` hunk); the P1 text
never shows it.
Resolution: replace the bullet with "One hunk per file. To change several
places in one file, put multiple `@@` chunks inside the single
`*** Update File:` hunk." Optionally extend the T2.2 example (or add a
T2.2b) with a second `@@` chunk so the drift-guard pins the pattern.

**[Major] M4 — Rebase safety-net gap: the seam §6 verification gate never runs the codex-apply-patch suite, so P2 can be silently lost on rebase.**
Spec §6; companion `docs/responses-compat-seam.md` §4/§6.
Evidence: the seam's post-rebase gate is
`cargo check -p codex-cli` / `cargo test -p codex-models-manager --lib` /
`cargo test -p codex-core apply_patch` / `cargo test -p codex-model-provider
--lib` — no `codex-apply-patch` entry. The P2 change lives entirely in
`codex-rs/apply-patch/src/streaming_parser.rs`; its only canaries are the T1
tests in that crate plus T4.2 (core integration, not in the rebase gate). The
seam §5/§6 functional smoke (qwen creates a file) uses *canonical* output and
would pass with P2 absent. If an upstream rebase restructures the AddFile arm
(conflict resolved in upstream's favor) the fork ships with leniency silently
gone: glm-5.2 PARSE rate reverts to ~67% with no gate, no wire check, and no
rollout alarm until a human re-runs the 3-day forensics.
Resolution: spec §6 must also (a) add `cargo test -p codex-apply-patch` (or
`just test -p codex-apply-patch`) to the seam §6 verification gate, (b) add
`codex-rs/apply-patch/src/streaming_parser.rs` to the seam §6 expected
conflict-site list with "re-apply the AddFile leniency (divergence #N)", and
(c) keep T1's F1-replay test as the named canary in the divergence-table row.
**[Major] M5 — "OpenAI/Azure behavior stays byte-identical" is false as written; the seam doc's "plus an Azure branch" gate description is stale.**
Spec TL;DR, §3.1 design notes, §6 (invariant update); seam doc §2.
Evidence: `ModelProviderInfo::is_openai()` is `self.name == OPENAI_PROVIDER_NAME`
with `OPENAI_PROVIDER_NAME = "OpenAI"` (`codex-rs/model-provider-info/src/lib.rs:546`,
introduced as a plain name match in b3ddd50eee; there is no "azure" string
anywhere in that crate). A provider named "Azure" therefore gets
`apply_patch_function_tool = true` (and already got
`normalize_content_types`/`flatten_namespace_tools = true`). The uncommitted
capability test asserts exactly that (`provider.rs` new test: Azure case →
`true`), which is consistent with the code but contradicts the seam doc §2
("string match ... plus an Azure branch") and its invariant ("OpenAI/Azure
provider behavior is byte-identical to upstream (all capability flags false on
`is_openai()`)"). The spec inherits and re-asserts the false claim, and its
§6 would write that invariant into the SoT. Any Azure-named deployment would
receive the function tool + P1 text + content-type normalization, unverified
against Azure (Principle 2), while the spec's safety case says Azure is
untouched.
Resolution: correct the spec's claim to match the gate ("providers whose
`name` is not exactly `OpenAI` — including anything named Azure — are in
scope for this capability; no Azure deployment is part of this change's
verification, flagged as such") or restore an Azure branch in `is_openai()`;
fix the seam doc §2 description in the same PR set; do not enshrine the
"OpenAI/Azure byte-identical" invariant while the code disagrees with it.

**[Major] M6 — Unaddressed contradiction: the legacy base-instructions example is present in every non-OpenAI prompt and conflicts with P1 on exactly the weak models P1 targets.**
Spec §3.4 (non-goal "No base_instructions change").
Evidence: `codex-rs/protocol/src/prompts/base_instructions/default.md:132`
(and the fork copies `codex-rs/models-manager/prompt.md:132`,
`models-manager/models.json` `instructions_template`) send every seat —
including the glm/qwen function-tool seats, whose base_instructions the spec
itself established as identical across seats — the example
`{"command":["apply_patch","*** Begin Patch\n*** Update File: ..."]}`: the
pre-Responses argument shape, literal two-character `\n` sequences in the
visible text, Update prefixes only, no Add-File `+` lines. The spec's
countermeasure is the assertion that "P1's argument description ... outranks
the legacy example for the function-tool path" — an untested assumption about
precisely the models that have already demonstrated they read prompt text
literally (F1) — with a revisit trigger scoped too narrowly ("only if
F3-class empty-arg calls recur"). A glm model copying the visible example
would emit a single-line patch with literal backslash-n, failing with "The
first line of the patch must be '*** Begin Patch'" — a new failure class P1
neither prevents nor anticipates.
Resolution: (minimal, invariant-safe) add one sentence to the P1 argument
description: "Lines in the patch are separated by real newlines, never by the
two characters backslash + n." (function-tool-only; OpenAI request bytes
unchanged). Broaden the §3.4 revisit trigger to any F1/F2/F3-class recurrence,
not just empty-arg calls.

### Minor

**[Minor] m1 — Under P2, a raw Add-File line beginning with `+` silently loses its leading `+`.**
Spec §3.2 (the `+` branch is "unchanged" and is checked first), §4 T1 (no test).
Evidence: the AddFile arm evaluates `line.strip_prefix('+')` before any new
content fallback (streaming_parser.rs:201-207). Probe C on the unmodified
binary shows the canonical consequence: `++44 20 7946 0958` → file content
`+44 20 7946 0958` (prefix stripped). A model following P1 writes `++`
correctly; a model ignoring P1 — P2's entire target audience — writing a raw
line that starts with `+` (phone numbers, `+1`-style increments, `+` markdown
bullets, diffs quoted in docs) gets a silent one-character loss per affected
line, with no error and no marker in the tool output. This is the one case
where P2 is *lossy*, and the spec does not name the consequence.
Resolution: add it to §3.2 "Semantics and edges" ("a raw content line starting
with '+' is parsed as a prefixed line; its leading '+' is stripped — the one
residual lossy case"), add a T1 test, and optionally let P1 note "to add a
line that starts with '+', write '++'".

**[Minor] m2 — `*** End Patch` as the last content line of a file being added truncates the file; the spec acknowledges the class but not this case.**
Spec §3.2 (structural-lines bullet; ambiguity paragraph uses `*** Delete File: x`
as its example).
Evidence: in AddFile mode `*** End Patch` is handled before any content branch
(streaming_parser.rs:84-106), so a file whose final line is exactly
`*** End Patch` (e.g. a doc quoting the format — this repo's own spec docs) is
silently truncated; if more content follows, the patch then fails post-End.
Probe H confirms the post-End failure mode on the unmodified binary. Pre-existing
and inherent (a canonical patch cannot represent such a file either), but the
truncation outcome is not spelled out and T1.4 does not cover it.
Resolution: add `*** End Patch`-as-final-content-line to the §3.2 worked
ambiguity examples with the truncation consequence, and add a T1 test pinning
it.

**[Minor] m3 — T2.2 self-consistency extraction rule is unspecified.**
Spec §4 T2.2.
Evidence: the P1 example is an unfenced bare block that is the suffix of the
argument description (after "Example:\n"). "Extract the example block" is
ambiguous as written (first match? last match? fence-based?). The example
itself is valid: tracing it through the parser semantics (bare `+` → empty
line; `@@ fn main` → context "fn main"; one `-`, one `+`, one context line)
yields exactly the hunks T2.2 asserts (`# TODO\n\n1. ship the fix\n`).
Not a Blocking gap — the description's shape makes the rule inferable — but the
drift guard must be deterministic.
Resolution: pin the rule in one sentence: "split the description on the first
'Example:\n'; the remainder (which must end with '*** End Patch') is parsed
by `parse_patch`."

**[Minor] m4 — T1.9 "existing tests must pass unmodified" is overbroad: 4 existing assertions necessarily change under P2/P3.**
Spec §4 T1/T1.9.
Evidence: `test_streaming_patch_parser_returns_errors`
(streaming_parser.rs:813) asserts (a) StartedPatch `'bad'` message — changed by
P3.1 (appended guidance); (b) AddFile `'bad'` → `InvalidHunkError` — *behavior*
removed by P2 (becomes `Ok([AddFile{contents: "bad\n"}])`); (c) DeleteFile
`'bad'` message — changed by P3.2. Additionally
`test_apply_patch_cli_rejects_invalid_hunk_header`
(`codex-rs/apply-patch/tests/suite/tool.rs:386-394`) asserts the exact CLI
stderr of the StartedPatch message and breaks under P3.1. (The core suite test
at `apply_patch_cli.rs:707` survives — it matches the substring "is not a
valid hunk header", which P3.1 preserves.)
Resolution: have T1.4/1.5/1.6 explicitly enumerate these four assertions as the
tests they rewrite (they are the red state), and scope T1.9 to "golden/canonical
suite passes unmodified" (the scenario fixtures under
`codex-rs/apply-patch/tests/fixtures/scenarios` plus canonical-success parser
tests).

**[Minor] m5 — P3.3's missing-argument message is also the message for a present-but-non-string `patch` value.**
Spec §3.3.3.
Evidence: `value.get("patch").and_then(as_str).ok_or_else("missing the required
...")` (apply_patch.rs:535-540) cannot distinguish `{"patch": {...}}` from
absence. Under the M2 wording risk, a JSON-object value is a plausible weak-model
output, and "missing" misdirects the model.
Resolution: branch the error: present-but-non-string → "the `patch` argument
must be a string containing the full patch text starting with '*** Begin
Patch'".

### Nit

**[Nit] n1 — §1.3 "A 24,230-char `patch` argument" is the arguments-JSON length, not the patch length.**
Evidence: rollout extraction — call `call_caf7ba...`: arguments JSON 24,230
chars; the `patch` string itself 23,972 chars. The wire-integrity claim is
unaffected; tighten the wording.

**[Nit] n2 — P2 also changes behavior on the OpenAI-path freeform diff consumer; note the surface.**
Evidence: `ApplyPatchArgumentDiffConsumer` (core/src/tools/handlers/apply_patch.rs:90,
121) feeds raw tool-input deltas through the same `StreamingPatchParser`. A
raw Add-File emitted by an OpenAI model on this fork now streams as content
(diffs) instead of erroring. Request bytes are unchanged, so the seam
invariants hold, but the divergence-table row in §6 should state the full
surface ("parser change is provider-agnostic; also affects the freeform diff
consumer and the `apply_patch` CLI") so rebase readers are not surprised.

## Adversarial list answers

**a) Does P2 create a corruption path?**
- *Malformed Update whose lines fall into an Add-File context:* only possible
  if the model emitted a `*** Add File: <path>` header (that is the sole way to
  enter the AddFile arm). If `<path>` exists → silent overwrite (M1, probe A:
  unmodified binary, exit 0, file replaced, output just "A <path>"). If
  `<path>` is new → a wrong-content file is created and the intended target is
  untouched: silent *intent* failure, no data loss. If no header at all (raw
  lines straight after `*** Begin Patch`), the StartedPatch arm stays strict
  under P2 → loud error. No new corruption there.
- *`*** Add File:` for an EXISTING file:* parses and overwrites **today
  already** for canonical patches (no existence check in verify or apply —
  `invocation.rs:242-245`, `lib.rs:505-528`). P2 does not change the outcome;
  it changes *reachability* (raw patches now get there instead of erroring).
  That delta is M1.
- *Missing `*** End Patch`:* the tool path fails at the batch boundary check
  before streaming ("Invalid patch: The last line of the patch must be
  '*** End Patch'", probe E); `finish()`'s duplicate check
  (streaming_parser.rs:154-171) only guards streaming-only consumers. P2/P3
  leave both unchanged; the message already names the required line, so P3
  correctly omits it.
- *Content line exactly `*** End Patch` in a file being added:* pre-existing
  structural ambiguity (handled before content in both today's and P2's arm);
  the spec's §3.2 ambiguity paragraph covers the class but not this specific
  truncation case → m2.
- *Windows paths / backslashes in Add-File paths:* no P2-specific risk — path
  resolution (`hunk.resolve_path` → `cwd.join(to_string_lossy())`,
  PathUri-based) is untouched; backslashes in content are appended verbatim
  with no escape processing anywhere in the parser.
- *UTF-8/emoji in content:* safe — `push_delta` iterates `chars()`, content is
  a `String`; the rollout's em-dashes/tables arrived and echoed intact.
- *A `+` line the model meant as content:* leading `+` is stripped under P2
  for raw Add-File lines (the `strip_prefix('+')` branch is checked first,
  streaming_parser.rs:201-207; probe C). This is m1 — silent one-char loss per
  affected line, untested, unnamed in the spec.

**b) P1 contradictions / token bloat:**
"plain text" (retained twice — the phrase the spec's own §2 blames for F1) +
"Do not wrap it in JSON" (the value *is* a JSON string; literal-`\n` and
JSON-object misreads are the likely failures) → M2. "Multiple hunks may appear
in one patch, in any order" contradicts the one-hunk-per-file verify rule →
M3. Bloat: the block is 1,448 chars / 244 words ≈ 317–360 tokens — the spec's
"≈ 330 tokens" is accurate; the cost argument (core editing tool, comparable
to the grammar payload) holds.

**c) Does T2.2 survive how the example is embedded?**
The example is an unfenced bare block forming the suffix of the description
(after "Example:\n"). Extraction is inferable but the spec should pin the rule
(m3). The example itself is valid and parses to exactly the asserted hunks
(verified by tracing `process_line`: `+# TODO`, bare `+` → empty line,
`+1. ship the fix`; then Update `src/main.rs` with one `@@ fn main` chunk, one
removal, one addition, one context line). Not a Blocking spec gap — a
one-line precision fix.

**d) Test-plan gaps:**
- *Canonical-regression catcher:* T1.9, backed by the real golden suite that
  exists at `codex-rs/apply-patch/tests/fixtures/scenarios` (10+ scenario dirs,
  run via `tests/suite/scenarios.rs` against the built binary) plus the
  canonical-success parser tests; T5 runs `codex-apply-patch`. Caveat: four
  existing *strictness* assertions must change (m4) — T1.9 as written
  ("existing tests must pass unmodified") cannot be read as a blanket freeze.
- *T4.1 feasibility:* yes, with one caveat the spec should state.
  `test_codex()` builds its provider as a clone of the built-in `openai` entry
  (`core/tests/common/test_codex.rs:839-844`) whose `name` is "OpenAI" →
  `apply_patch_function_tool = false`, so the default builder exercises the
  FREEFORM path. The test must rename the provider via `with_config`
  (e.g. `config.model_provider.name = "Test Provider"`); default auth is
  `CodexAuth::from_api_key("dummy")` (test_codex.rs:1376), which works for any
  provider name. Precedents exist for asserting the tools array on captured
  request bodies (`client.rs:2140`, `agent_execution.rs:552`) and for real
  turns over custom-named providers (`model_provider_requirements_tests.rs`).
- *T4.2* end-to-end covers P2 for new files only; the existing-file variant is
  missing (M1(d)).

**e) Rebase/upstream risk:**
P2 modifies shared upstream `streaming_parser.rs`. The spec's §6 plans the
divergence-table row (good) but the seam §6 *verification gate* does not run
`codex-apply-patch` tests, and the post-rebase wire/functional smoke uses
canonical qwen output — so an upstream restructure of the AddFile arm that
drops the leniency passes every gate and silently reverts glm-5.2 to ~67%
PARSE (M4). Also list `streaming_parser.rs` among the seam §6 expected
conflict sites, and note the provider-agnostic surface (diff consumer, CLI) in
the divergence row (n2).

**f) Other model-facing patch-format documentation:**
- `codex-rs/protocol/src/prompts/base_instructions/default.md:132` + fork copy
  `models-manager/prompt.md:132` + `models.json` `instructions_template`:
  legacy `{"command": [...]}` example with literal `\n` and no Add-File `+`
  lines — sent to every seat, contradicting P1's argument shape and format;
  the §3.4 non-goal rests on the untested "outranks" assumption → M6.
- `core/gpt_5_1_prompt.md` / `gpt_5_2_prompt.md` contain `*** Begin Patch`
  examples, but only for GPT-catalog models on the OpenAI path — no
  non-OpenAI contradiction.
- Freeform tool description (`apply_patch_spec.rs`, "...do not wrap the patch
  in JSON") is OpenAI-only; the Lark grammar travels only with the freeform
  tool. No tool-result message teaches format beyond the P3 errors.
  `events.rs`/`lib.rs`/`invocation.rs` "Begin Patch" hits are internal/tests.
  So the only unaddressed contradicting surface is base_instructions (M6).

## Counts

- Blocking: 0
- Major: 6 (M1–M6)
- Minor: 5 (m1–m5)
- Nit: 2 (n1–n2)
- **Blocking + Major = 6** → CHANGES-REQUESTED per the review-loop rule
  (advance only after a fresh round returns zero Blocking and zero Major).

## Appendix — parser probe results (unmodified working tree, built binary)

| # | Input (Add-File arm unless noted) | Result (today) | Under P2 |
|---|---|---|---|
| A | `*** Add File: existing.md` + canonical `+` lines, file exists | **exit 0, file silently overwritten** ("A existing.md") | unchanged (M1) |
| B | raw line then `*** Update File:` inside Add-File | error at raw line (line 3) | raw line → content; header still structural; empty Update hunk rejected at End |
| C | canonical `++44 20 7946 0958` | file content `+44 20 7946 0958` (prefix stripped) | raw `+...` lines lose leading `+` (m1) |
| D | raw Add-File with blank content line | error at line 3 | accepted (T1.2) |
| E | missing `*** End Patch` | "Invalid patch: The last line of the patch must be '*** End Patch'" (boundary check) | unchanged |
| F | CRLF raw Add-File | error at line 3 | accepted, LF-equivalent (T1.7) |
| G | Delete-File followed by content line | generic "'stray content' is not a valid hunk header..." | P3.2 message (T1.5) |
| H | content after `*** End Patch` | "The last line of the patch must be '*** End Patch'" | unchanged; `*** End Patch` as final *content* line truncates (m2) |

## Evidence index

- Rollout: `~/.codex/sessions/2026/09/15/rollout-2026-09-15T18-42-54-01a0a73c-cab6-7183-a71a-f8449028d121.jsonl` — F1 quote byte-for-byte match (em-dash included); F1 args 23,972 chars (arguments JSON 24,230); F2-shaped failure at call `call_cda093...` (line 3 raw context line).
- Grammar: `codex-rs/core/assets/tools/apply_patch.lark` (add_line `+` prefix; update_hunk multi-chunk `change`).
- Parser: `streaming_parser.rs:84` (headers/End), `:139` (push_delta, CRLF strip), `:154` (finish), `:187/:198/:216` (StartedPatch/AddFile/DeleteFile arms), `:813` (test whose 3 assertions P2/P3 must update); `parser.rs:145-208` (single parse path, Lenient heredoc); `invocation.rs:235-245` (multiple-operations + verify); `lib.rs:505-528` (Add-File apply, no existence check).
- Seam: `docs/responses-compat-seam.md` §2 (stale Azure branch claim), §4 (divergence map), §6 (gate missing codex-apply-patch).
- Gate code: `model-provider-info/src/lib.rs:546` (`is_openai` = name match only); uncommitted `provider.rs` capability test (Azure → true); `spec_plan.rs` registration site; `core/tests/common/test_codex.rs:839-844,1376` (T4.1 provider/auth setup).
