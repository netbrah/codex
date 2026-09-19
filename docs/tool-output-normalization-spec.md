# TOOLOUT-NORM-1 — Tool-output content normalization seam gap (SDD)

**SDD status:** FINAL — review loop converged: R1 (M 1B/1M/1m/3n, N 0B/0M/1m/3n) + R2 (M2 0B/1M/4m/3n, N2 0B/0M/2m/2n) applied; R3 full 0B/0M round (M3 0B/0M/1m/2n, N3 0B/0M/1m/1n); post-R3 precision fixes applied; TDD cleared
**Bead:** `apex-xt2.10` (central ledger `$HOME/Projects/bitbucket/apex_tracking`)
**Repo/branch:** `/Users/palanisd/Projects/upstream/codex` @ `feat/normalize-content-types-vllm`
**Severity:** P0 (operator-declared critical; hard 400 on a core workflow — MCP tool output round-trip on on-prem)

---

## 1. Problem (incident)

On-prem v2 worker session (subagent `Erdos`, `/root/xwpreplan_glm`, model **glm-5.2**, provider `llm_proxy_qwen`) called MCP tool `mcp__codegraph.codegraph_explore`. The next model request 400'd at the LiteLLM proxy:

```
litellm.BadRequestError: messages.8.ChatCompletionMessageUserParam.content.list[...].1
  .ChatCompletionMessageContentVideoPart.type
  Input should be 'video_url' [literal_error, input_value='input_text', input_type=str]
  (union members tried: TextPart 'text' / ImagePart 'image_url' / VideoPart / AudioPart / ToolReferenceBlock)
Received Model Group=glm-5.2
```

The thread then closed; the parent TUI reported "Agent thread … is closed. Replaying saved transcript."

**Attribution correction:** the operator recalled a qwen worker; the triage card (`~/.codex/bin/codex-triage show 01a0b07a`) proves **glm-5.2** (`models: glm-5.2`, `agent_path: /root/xwpreplan_glm`, LiteLLM `Model Group=glm-5.2`). The defect is model-agnostic — a qwen session calling the same MCP tool 400s identically (same wire path).

## 2. Root cause (verified first-hand)

1. **MCP result shape.** Rollout `~/.codex/sessions/2026/09/17/rollout-2026-09-17T13-47-15-01a0b07a-….jsonl`, ordinal 36: `function_call_output` for the codegraph call has `output` = **array of 2 content parts**, both `{"type":"input_text","text":…}`. the longer part (index 1, 11,933 chars; type `input_text`, text ending `Synthesize once you've used 2.`) — its `input_text` **type string** is the byte-match to the LiteLLM-rejected `input_value` (part[0] is the 33-char `Wall time…` fragment). (Responses-API-legal shape: `function_call_output.output` accepts a string or an array of content parts.)
2. **The shim.** LiteLLM serves glm-5.2 as a chat model (vLLM); its Responses→ChatCompletions converter folds `function_call_output` into a **user** message, passing the content parts through unchanged. `ChatCompletionMessageUserParam.content` is a pydantic union accepting `text|image_url|video_url|audio_url|tool_reference` — `input_text` is not a member → 400. (Same shim class the fork seam documents for SGLang/vLLM.)
3. **The seam gap (harness-side, fixable).** `normalize_content_types` (`codex-rs/codex-api/src/endpoint/content_type_compat.rs:23`) — the branch's existing normalization for exactly this shim class — walks `input[]` items and rewrites `input_text`/`output_text` → `text` **inside `item.content` arrays only**. `function_call_output` / `custom_tool_call_output` items carry their payload in **`item.output`**, which the walk skips (`item.get_mut("content")` → `None` → `continue`). The unnormalized parts sail through.
4. **Gating (already correct).** `responses.rs:87`: when `normalize_content_types` is set, the serialized body is normalized, then `translate_agent_messages` runs. For `ConfiguredModelProvider` the flag is `!provider.is_openai()` (`codex-rs/model-provider/src/provider.rs:399`) — the `llm_proxy*` providers are covered; no gating change needed. (Precision, R1: `AmazonBedrockModelProvider` overrides `normalize_content_types: false` in `capabilities()` (`codex-rs/model-provider/src/amazon_bedrock/mod.rs:218`) despite being non-OpenAI — no impact on the llm_proxy path.)

**Verdict:** not "nothing we can do" (LiteLLM converter behavior is fixed) — there is a clean harness-side ratchet inside the existing seam. Retiring the glm worker profile would NOT remove the class (qwen sessions with MCP tools are equally exposed). Operator conditional ("retire glm worker only if no easy fix") ⇒ **implement the fix; glm worker stays.**

## 3. Fix design (one file + its test file)

Extend `normalize_content_types` so each `input[]` item is also checked for an `output` field:

- **All-text collapse (primary ratchet):** if `output` is a non-empty array and EVERY part is a text part (`type ∈ {input_text, output_text, text}` AND has a string `text` field) → replace the array with a single string: the parts' texts, **empty/whitespace-only segments filtered first** (mirrors `function_call_output_content_items_to_text`, `codex-rs/protocol/src/models.rs:2117–2138`; trim guard `:2123`, join `:2136`), **joined with `"\n"`** (empty string if no segments remain). A plain string is the most conservative legal `output` shape: legal on the Responses API and immune to any part-type handling in any shim (no union member to mismatch).
- **Mixed in-place (fallback):** if any part is non-text (e.g. `input_image` from MCP image content, or `encrypted_content`, below) → keep the array; rewrite `input_text`/`output_text` part types to `text` in place (shared helper, same rule as the existing `content` path); **drop `encrypted_content` parts** (mirror of the content path's `content.retain`, `content_type_compat.rs:59–63`); leave remaining non-text parts (`input_image`, `input_audio` — same code branch, `input_image` exercised by T3) untouched. If the array is empty after the drop → replace with `""` (string).
- **`encrypted_content` in tool `output` (R1 blocking fix):** the v1 claim "not produced by codex" was FALSE. `convert_mcp_content_to_items` (`codex-rs/protocol/src/models.rs`, `McpContent::Text` arm) produces `EncryptedContent` when an MCP text block carries `_meta["codex/encryptedContent"]==true`, and `as_function_call_output_payload` early-returns the whole `ContentItems` array including it. It is a supported, tested MCP feature (harness test server `encrypted_output` tool, `codex-rs/rmcp-client/src/bin/test_stdio_server.rs`; core ingests it, `codex-rs/core/src/tools/handlers/mcp.rs`). Not expected in on-prem text-only flows — but such an array takes the mixed fallback and the `encrypted_content` part would ride the shim union (not a member) → same-class 400 on the same worker profile. The drop mirrors the content path (ciphertext is undecryptable to the model; the typed in-process item keeps it) and is the consistent choice for a P0.
- **Known residual (stated precisely, R1 major fix):** `input_image`/`input_audio` part labels are NOT members of the LiteLLM shim user-content union (`text|image_url|video_url|audio_url|tool_reference`) → an MCP tool that returns IMAGE content still 400s identically behind the shim. Reachable in this codebase (MCP image content → `InputImage` via `convert_mcp_content_to_items`; the tool-output path is `McpToolOutput::response_payload` → `as_function_call_output_payload`, `codex-rs/core/src/tools/context.rs:174–175` + `codex-rs/protocol/src/models.rs:2288`; harness test server `image` tool. Note: `codex-rs/core/src/tools/handlers/mcp.rs:340–378` is the unrelated code-mode/node-repl evidence-capture path, NOT this one). The pre-existing `content`-path seam has the same gap (it only relabels text types). Out of scope for this P0 (incident is text-only; no image evidence); a follow-up bead is filed at campaign close. A second ratchet (`input_image`→`image_url` relabel) requires checking the shim's `image_url` member shape (string vs object) before design.
- **String / missing / other non-array `output` values (incl. `null`):** untouched (`null` is unreachable from codex serialization — `FunctionCallOutputPayload` serializes untagged string-or-array, `models.rs:2239–2249` — T5 pins the behavior anyway).
- **Empty array:** `output: []` stays as-is (reachable: `as_function_call_output_payload` falls through to `ContentItems` when MCP returns no content and no `structured_content`, `codex-rs/protocol/src/models.rs:2320`; there are no `input_text` parts that could mismatch — pinned by T10).
- **Code shape:** factor the `input_text`/`output_text`→`text` relabel block (7 lines, `content_type_compat.rs:34–40`) out of the existing content loop into a small private helper (now referenced from both the `content` path and the `output` path — satisfies the no-single-reference-helper rule); the content path's `encrypted_content` relabel/drop behavior stays as-is.
- **Scope:** `codex-rs/codex-api/src/endpoint/content_type_compat.rs` only (+ its `#[path]` test file). No protocol, no core, no config, no new flags. OpenAI-provider requests never enter this code (gate at :87).

## 4. Test plan (TDD; file `content_type_compat_tests.rs`, existing naming/style)

| id | input (serialized body fragment) | expected |
|---|---|---|
| T1 | `function_call_output` with `output: [input_text "A", input_text "   ", input_text "B"]` | `output == "A\nB"` (string; the whitespace-only segment is filtered before the join — pins the §3 filter); nothing else changed |
| T2 | `output: [input_text "A", output_text "B", text "C"]` | `output == "A\nB\nC"` (mixed text-type labels all collapse) |
| T3 | `output: [input_text "A", {type:"input_image",…}]` | array kept; part 0 type → `text`; image part byte-identical |
| T4 | `output: "already a string"` | unchanged |
| T5 | item without `output` / `output: null`; message-`content` cases (existing tests) | unchanged — full existing test file stays green (regression guard on the refactor) |
| T6 | **incident repro, end-to-end:** full body = user message (multi-part content) + assistant `function_call` + `function_call_output` with the 2-part codegraph-shaped output. Fixture texts are pinned; the implementer MUST verify that **every string literal in the fixture body** (output texts, user-message texts, tool arguments) contains the literal substring `input_text` **zero times** (the real codegraph text does contain it — the padding branch is expected: keep the `Synthesize once you've used 2.` tail, pad with neutral filler; record the choice in the commit body) | **primary:** deep-equals `output == "part1\npart2"`; **secondary:** whole serialized body contains `input_text` zero times; **tertiary:** both part texts present, order kept |
| T7 | `custom_tool_call_output` with `output: [input_text ×2]` | collapsed like T1 (same code path) |

| T8 | `output: [encrypted_content "cipher"]` (all-encrypted array) | `output == ""` (encrypted part dropped; empty-after-drop array → string) |
| T9 | `output: [input_text "A", encrypted_content "cipher"]` (mixed text+encrypted) | array kept: `[{"type":"text","text":"A"}]` — text relabelled, encrypted dropped, array non-empty (the realistic R1-B case) |
| T10 | `output: []` (empty array) | unchanged (`[]` stays `[]`) |
| T11 | `output: [input_text " ", input_text ""]` (all-blank text array) | `output == ""` (collapse path: filter leaves no segments) |

Note (R1): `output_text`/`text` labels cannot arise from codex's own serialization — `FunctionCallOutputContentItem` produces only `input_text` for text content (`codex-rs/protocol/src/models.rs:2084–2104`); T2 is defensive against future text variants.

Assertions: deep-equals on the transformed `Value` are the PRIMARY assertions (repo rule); T6 adds the whole-serialized-body substring check as its secondary assertion.

## 5. Gate & build plan

Build queue (charlie mike; no concurrent builds during gate runs; `CARGO_BUILD_JOBS=2`):

1. Wait for the apex-xt2.8 gate re-run to finish (re-run 3b: `/tmp/xt28-rerun3b-result.txt` / `/tmp/xt28-rerun3b.log`. Re-run 3 itself was a LAUNCHER FAILURE — filters passed as whole `binary testname` lines matched 0 tests, exit 4 — fixed in 3b with `awk '{print $2}'`.)
2. Then apex-xt2.7 item-1 (strict_int) GO → its scoped gates.
3. Then apex-xt2.9 (onprem catalog/proactive) TDD items as its review loop converges.
4. **Then this campaign**: TDD red (new tests fail vs current seam) → implement → green → `just fmt` → `just fix -p codex-api` → `just test -p codex-api`. No core/protocol changes ⇒ no full-suite obligation; full `just test` only if the operator asks.
5. Push pipeline: commit `codex-api: normalize function_call_output output arrays in content-type seam (apex-xt2.10)` (provenance in body: incident thread 01a0b07a, LiteLLM 400, model-agnostic class); pathspec-strict; push BOTH remotes (never origin); SCS ff-sync; bead closed with gate evidence.

## 6. Live acceptance (verify at the source — Principle 2)

The defect manifests at the LiteLLM proxy, so post-build acceptance runs through it: rotate the new binary, then in an on-prem v2 session have a worker (glm seat or qwen) make an MCP call that returns multi-part output (`codegraph_explore` is the incident tool) and confirm: no 400, the tool text reaches the model, the turn completes. Record the session id as bead evidence.

## 7. Risks

- **Join semantics:** collapsing `[A, B]` → `"A\nB"` is what the model would read from the two-part array (same text, natural separator); any consumer expecting array-ness of `output` on a non-OpenAI shim gets the MORE compatible shape. OpenAI path untouched (gate).
- **Shim converter variance:** different proxies may fold tool output differently (tool-role vs user-fold); the string-collapse is robust to both.
- **Reasoning-item leak class (untested sibling, R1 note):** `summary` (`summary_text`, `codex-rs/protocol/src/models.rs:1983–1985`) and the top-level `encrypted_content` string on reasoning items (`models.rs:1055`) are untouched by the seam — it walks `content` (and, post-fix, `output`) arrays, and reasoning items carry neither `output` nor content parts matching the relabel/drop conditions (reasoning `content` parts are `reasoning_text`; `output` is absent from the `Reasoning` variant). `leaves_reasoning_content_untouched` (`content_type_compat_tests.rs:48–63`) codifies the `summary`/`content` half; the top-level `encrypted_content` string is unpinned by that test. No incident evidence; if a 400 ever cites a reasoning item, audit this class first.
- **Upstream pull:** `content_type_compat.rs` is a fork-only file (manifest: MUST SURVIVE, merge risk none) — the extension rides along with the file.

## 8. Acceptance criteria (bead mirror)

1. Seam handles `output` arrays per §3 (collapse + in-place fallback + `encrypted_content` drop + empty-after-drop → `""` + empty-array passthrough + string/missing/other untouched); helper factored; no other file touched.
2. T1–T11 green; pre-existing seam tests green unchanged.
3. Gates per §5 green.
4. Live acceptance per §6 passes (real proxy, on-prem session, MCP round-trip).
5. Pushed both remotes + SCS ff-sync; bead `apex-xt2.10` closed with evidence.

## 9. Review log

| round | date (UTC) | seat(s) | findings (B/M/m/n) | resolution |
|---|---|---|---|---|
| R1 | 2026-09-17 | seat M (counterfactual) + seat N (concordance), fresh, READ-ONLY | M: 1B/1M/1m/3n · N: 0B/0M/1m/3n | all applied — per-finding rows below |
| R2 | 2026-09-17 | seat M2 (counterfactual) + seat N2 (concordance), fresh, READ-ONLY | M2: 0B/1M/4m/3n · N2: 0B/0M/2m/2n | all applied in v2.1 — per-finding rows below |
| R3 | 2026-09-17 | seat M3 (counterfactual) + seat N3 (concordance), fresh, READ-ONLY | M3: 0B/0M/1m/2n · N3: 0B/0M/1m/1n | **CONVERGED** — full 0B/0M round; m/n applied as post-R3 precision fixes (below) |

**Post-R3 precision fixes (no design change; logged, verified by coordinator):** (1) §3 string/missing bullet now covers "other non-array output values" incl. `null` (T5); (2) §3 mixed rule notes `input_audio` shares T3's branch; (3) §8 criterion 1 parenthetical made exhaustive (+ empty-array passthrough + string/missing/other); (4) §7 reasoning bullet reworded — reasoning items DO carry optional `content` (`reasoning_text` parts, `models.rs:1054`; the test itself includes one) but no part matches the relabel/drop conditions and no `output` — the safety conclusion stands. R3 verdicts archived at `/tmp/toolout-r3-seat{M,N}-verdict.md`.

**R1 per-finding log** (findings + resolutions, per repo SDD rules):

| # | seat | class | finding (abridged) | resolution in v2 |
|---|---|---|---|---|
| 1 | M | B | §3 claim "`encrypted_content` parts in tool `output` are not produced by codex" is FALSE: `convert_mcp_content_to_items` produces it (MCP `_meta["codex/encryptedContent"]==true`), `as_function_call_output_payload` early-returns the array including it; reachable + tested (test server `encrypted_output`) → same-class 400 via the mixed fallback | §3: factual claim corrected (new `encrypted_content` bullet); mixed fallback drops `encrypted_content` parts, mirroring the content path's retain; T8 added; §8 criterion 1 updated |
| 2 | M | M | mixed fallback leaves `input_image` parts labeled `input_image` ∉ shim union → an MCP image tool output 400s identically; the v1 residual was imprecise ("rides the union") | §3 "Known residual" bullet stated precisely (label mismatch; reachable paths named; pre-existing content-path gap named); follow-up bead at campaign close; `image_url` relabel deferred pending shim member-shape check |
| 3 | M | m | T6 "zero `input_text` substrings" ill-defined if the fixture texts themselves contain the substring | §4 T6: fixture texts pinned + implementer verifies substring-free (padding rule); deep-equals made the primary assertion |
| 4 | M | n | `!is_openai()` gate precise only for `ConfiguredModelProvider`; `AmazonBedrockModelProvider` overrides `normalize_content_types: false` | §2.4: "for `ConfiguredModelProvider`" + Bedrock note (`amazon_bedrock/mod.rs:218`) |
| 5 | M | n | `"\n"` join keeps empty/whitespace-only segments while `function_call_output_content_items_to_text` filters them | §3 collapse rule: filter first (wire string now matches `to_text` output) |
| 6 | M | n | reasoning-item leak class (summary / top-level `encrypted_content` string) unmentioned | §7 new risk bullet (untested sibling; no incident evidence) |
| 7 | N | m | encrypted-content rationale reword (same root as #1) | merged into #1 |
| 8 | N | n | §2.1 byte-match phrasing conflated the part's text with its type (the rejected `input_value` is the TYPE string) | §2.1 reworded: "its `input_text` type string is the byte-match" |
| 9 | N | n | "two-line" relabel is actually a 7-line block (`content_type_compat.rs:33–40`) | §3 code-shape: "7 lines, `content_type_compat.rs:33–40`" |
| 10 | N | n | T2's `output_text`/`text` labels cannot arise from codex serialization (only `input_text` is produced for text) | §4 T2 note (defensive test) |

**Coordinator verification (R1→v2):** every code claim in findings 1–2 re-verified first-hand at HEAD `7bcd344fa7` — `models.rs` `McpContent::Text` arm + `as_function_call_output_payload` early-return; `test_stdio_server.rs` `encrypted_output` + `image` tools; `mcp.rs` `is_encrypted`; `amazon_bedrock/mod.rs:218`; `to_text` trim-filter + `"\n"` join; `leaves_reasoning_content_untouched` (`content_type_compat_tests.rs:48`). Full R1 verdicts preserved at `/tmp/toolout-r1-seat{M,N}-verdict.md`.

**R2 per-finding log** (numbering continues from R1):

| # | seat | class | finding (abridged) | resolution in v2.1 |
|---|---|---|---|---|
| 11 | M2 | M | T1–T8 do not pin the collapse filter or the all-blank→`""` collapse edge (T8's `""` is the mixed-drop branch — a different code path); an impl omitting the filter passes the green list | T1 input now includes a whitespace-only segment (`"A","   ","B"` → `"A\nB"`); new T11 all-blank → `""`; §8 criterion 2 → T1–T11 |
| 12 | M2 | m | §8 criterion 2 stale "T1–T7" (R1 fix added T8 but not the green list) | → T1–T11 (superseded by #11's range) |
| 13 | M2 | m | known-residual bullet cited `mcp.rs:340–378` — that is the code-mode/node-repl evidence-capture path (`on_tool_result_accepted`, `mcp.rs:297`), not the tool-output path | re-cited: `McpToolOutput::response_payload` → `as_function_call_output_payload` (`context.rs:174–175`, `models.rs:2288`); mcp.rs attributed as the unrelated evidence path |
| 14 | M2 | m | line-cite drift at HEAD: to_text is `models.rs:2117–2138` (not 2126–2145); `ReasoningItemReasoningSummary` is `:1983–1985` (not 1983–1987) | both re-pinned (to_text shared with #18) |
| 15 | M2 | m | §7 reasoning bullet internally inconsistent: "walks `content` arrays only" is false post-fix; `leaves_reasoning_content_untouched` does not pin the top-level `encrypted_content` string | reworded: walks `content` (and post-fix `output`) arrays, neither present on reasoning items; test codifies the summary/content half; top-level string at `models.rs:1055` unpinned |
| 16 | M2 | n | §2.1 "Part 1" misnumbers the array — the codegraph text is part[1] (index 1, 11,933 chars); part[0] is the 33-char `Wall time…` fragment | "the longer part (index 1, 11,933 chars)" + part[0] identified |
| 17 | M2 | n | "7 lines, :33–40" self-inconsistent (33–40 inclusive = 8 lines; the relabel block is :34–40) | "7 lines, `content_type_compat.rs:34–40`" |
| 18 | N2 | n | to_text citation `:2126–2145` wrong span (fn :2117–2138; trim :2123; join :2136) | re-pinned (same as #14) |
| 19 | N2 | n | relabel line range (same as #17) | same |
| 20 | N2 | m | mixed text+encrypted_content case (the realistic R1-B case) untested | new T9: `[input_text "A", encrypted_content "cipher"]` → array kept `[{"type":"text","text":"A"}]` |
| 21 | N2 | m | `output: []` not covered by any §3 rule (reachable via `as_function_call_output_payload` fall-through, `models.rs:2320`) | new §3 "Empty array" rule (stays as-is) + T10 |

**Coordinator verification (R2→v2.1):** re-verified first-hand — `context.rs:174–175` (`response_payload` → `as_function_call_output_payload`); to_text span :2117–2138 (trim :2123, join :2136); `ReasoningItemReasoningSummary` :1983–1985; relabel block :34–40; `models.rs:2320` empty fall-through; rollout part indices (part[0] = 33-char `Wall time` fragment, part[1] = 11,933-char codegraph text). R2 verdicts archived at `/tmp/toolout-r2-seat{M,N}-verdict.md`.
