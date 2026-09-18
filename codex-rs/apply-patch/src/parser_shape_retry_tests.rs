//! Strict-first shape-repair pre-pass (apex-xt2.11, item A) — spec
//! `docs/apply-patch-lax-retry-spec.md` §5 rows T1–T11 + T13.
//!
//! T4–T6 fixtures are byte-identical inline copies of the /tmp glm failure
//! repro files (they are not repo-tracked):
//! - `/tmp/glm-fail-7584b626.txt` (24113 B): shape A — first line is a hunk
//!   header, `+`-prefixed content, zero Begin/End markers.
//! - `/tmp/glm-fail-b943dedf.txt` (14705 B): shape AB — stray AddFile
//!   header, `*** Begin Patch` at line 2, duplicate AddFile, bare `@@`, no
//!   `*** End Patch`.

use super::*;
use crate::ApplyPatchArgs;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

/// Pinned first-line boundary error text (pre-change byte-identical, spec
/// §5 T7).
const FIRST_LINE_BOUNDARY_TEXT: &str = "The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'.";

/// Pinned last-line boundary error text (pre-change byte-identical, spec
/// §5 T13).
const LAST_LINE_BOUNDARY_TEXT: &str = "The last line of the patch must be '*** End Patch'. The terminator line is exactly '*** End Patch' with no '+' or other prefix and no lines after it.";

/// Byte-identical inline copy of `/tmp/glm-fail-7584b626.txt` (24113 B).
const GLM_FAIL_7584B626: &str = r####"*** Add File: grok/plans/crosswire-preplan-glm-20260917.md
+# Cross-Wire Preplan — GLM Concordance Seat (2026-09-17)
+
+**Role:** READ-ONLY concordance seat. Verifies qwen-seat claims I-A..I-G, adjudicates three disputes, classifies the §VI MCP-poisoning incident, and surfaces counterfactuals the qwen seats missed. A REFUTED verdict is a success.
+
+**Citation base:** worktree `wt/grok-build-responses` @ HEAD `4fb487f9` (one commit ahead of the stated base `b745593`; all line numbers re-derived against the live tree). Paths relative to the worktree root unless prefixed `~/.grok/`.
+
+**Constraints honored:** no MCP tools used (shell-only: rg, sed, cat, jq, python3) · no file edits except this report · no bd writes · no config edits · no git writes · no cargo. Config file read read-only at `~/.grok/config.toml` (modified Sep 17 16:30 by another lane — see counterfactual #1).
+
+---
+
+## (i) Verdict table — claims I-A..I-G
+
+| Claim | Verdict | Re-derived cite | Notes |
+|---|---|---|---|
+| I-A preemptive compact wall | **CONFIRMED** | `agent/handlers/model_switch.rs:60-68` (`is_family_switch` = both `Some` + distinct); `acp_session_impl/model_switch.rs:148-163` (compact before first post-switch request); `compaction.rs:2630-2708` (`run_compact_only` — error path L2686-2708 sends notification + returns `Err(e)`, no model-bound retry arm); caller at `model_switch.rs:161-163` logs "switching anyway" | Compact path has zero `RetryWithModelBoundStateStrip` arms (rg confirmed). Wall, not bridge. |
+| I-B BRICK gap (needle miss) | **CONFIRMED** | `error.rs:428-479` (classifier); F3 `input[`+`.id`+`invalid` at L457-460; F4 `item`+`id`+`not found`/`does not exist` at L462-465; `retry.rs:121-122` (model-bound arm); `retry.rs:186` (Fatal fallthrough) | vLLM pydantic 400 (`input.N.id`, dot not bracket, no "invalid") → F3 miss. Azure `.call_id`-orphan: `.call_id` does not contain `.id` as substring (verified: `.` followed by `c`, not `i`) → F3 miss. F4 needs "not found"/"does not exist" → also misses. Non-matching 400 → Fatal → persistent BRICK. |
+| I-C empty-id root cause | **CONFIRMED** | `stream/messages.rs:543-549` (`id: String::new()`, signature in `encrypted_content`); `conversation/responses.rs:262-267` (passes id through, only clears `status`); `provider.rs:263-278` (`project_strict_responses_input` removes `id`+`content` for `type=="reasoning"` when `strict_dialect`) | Strict rows = id ABSENT (removed, never synthesized). Non-strict targets ride `""` verbatim → Azure F3. |
+| I-D normalize gaps G1-G4 | **CONFIRMED** | `provider.rs:311-332` (`normalize_content_types`); walks only `input[].content[]`, rewrites only `input_text`/`output_text`→`text`; gated by `!is_openai_family(family)` at `provider.rs:101` (`is_openai_family` L107-112 = xai/codex/openai/empty) | G1 `input_image` parts untouched (not input_text/output_text). G2 `function_call_output` items have `output` not `content` → `continue` at L317. G3 reasoning `content[]` parts aren't input_text/output_text → untouched. |
+| I-E orphaned tool-output class | **CONFIRMED** | `conversation/messages.rs:79-126` (`clean_orphaned_items` — messages wire only; rg: zero hits on responses/sampler paths); `conversation.rs:2261` (`repair_dangling_tool_calls` — call→result direction only, inserts synthetic results, never removes orphaned results) | Responses wire has no orphan cleanup. Classifier needles miss the call_id-orphan 400 (per I-B). Latent today; reachable via .71 per-item projection. |
+| I-F proven-cell mechanism | **PARTIAL** | `provider.rs:263-278` (strict projector strips id); `config.rs:80` (`strict_responses_input` serde default false); `~/.grok/config.toml` L16, L187, L193 | **Mechanism CONFIRMED** — config-only strict flag makes existing projection strip id → no code change. **Premise STALE** — terra/luna NOW have `strict_responses_input=true` + `model_family="codex"` + `context_window=262144` (config modified Sep 17 16:30, after the qwen reports). Cell 1 (sonnet→terra) is now PREDICTED-CLEAN, not CATCH. See counterfactual #1. |
+| I-G projector hook = client.rs; setModel only cross-wire | **CONFIRMED** | `client.rs:1964-1984` (streaming), `:2131-2151` (non-streaming), `:2381-2409` (third path) — all have patch_reasoning_text_types→patch_responses_request→patch_raw_input_replacements→strip_encrypted_content_input→project_strict_responses_input; `session_setup.rs:1502-1560` (`restore_persisted_model` — persisted model wins; fallback within-family only L1524-1534) | Resume/-m cannot cross wire families. setModel (ACP `handle_set_session_model`) is the only cross-wire entry. |
+
+---
+
+## (ii) Adjudications
+
+### 1. .69 vs .71 — required alongside, or optional belt-and-suspenders?
+
+**Verdict: .71 is REQUIRED; .69 is optional belt-and-suspenders, probe-conditional on P2/P7.**
+
+Reasoning:
+- **Strict rows:** `project_strict_responses_input` (`provider.rs:274`) removes the `id` key entirely at send time. Whether the persisted id is `""` (current) or a synthesized `xw_...` id (.69), it is removed before the wire sees it. Mint-time synthesis is irrelevant for strict rows — the projector is the sole gate.
+- **Lenient rows:** the empty id rides verbatim (`conversation/responses.rs:262-267` passes id through; `project_strict_responses_input` is a no-op when `strict_dialect=false`). .69 would synthesize a non-empty id at mint time. But REPLAY-1 (ledger L693ff) ruled vLLM accepts empty ids (KEEP, 200 OK). So .69 only matters if P2/P7 proves a lenient shim REJECTS empty ids — that's a probe-conditional dependency, not a known requirement.
+- **Pre-existing history:** .69 cannot fix legacy on-disk items (the empty ids are already persisted). Only .71 (switch-time projection of persisted history) repairs them. This is the hard requirement.
+- **Config-only fix (now applied):** terra/luna strict=true (config L187/L193) makes the EXISTING `project_strict_responses_input` handle those rows at send time — no .69, no .71 needed for the AZ-lenient class. But lenient rows (qwen, glm, grok-4.6) still need .71.
+- **Shared id rule:** both must use ONE id synthesis rule. The fixture README embeds the cell name in the sha preimage (`xw_sha256("{cell}|{ridx}|{content}|{summary}")`); the runtime projector has no cell name. .71 must define a runtime preimage (session+index+content) OR adopt the fixture rule with a stable cell field — this is an open design decision, not a blocker.
+- **Sequencing:** .71 first (handles all pre-existing history at switch time); .71 also covers same-model re-sends (the strict projector already does, but lenient rows get no projection today). .69 deferred until P2/P7 returns a lenient-empty-id rejection.
+
+### 2. Are H-1..H-5 sound and correctly ordered against .62→.65?
+
+**Verdict: all five SOUND; ordering correct with one refinement.**
+
+- **H-1 (orphan gap):** SOUND. Verified: `clean_orphaned_items` is messages-wire-only (`messages.rs:79`); `repair_dangling_tool_calls` (`conversation.rs:2261`) only inserts synthetic results for unanswered calls, never removes orphaned results. The `.call_id`-orphan 400 misses all F1-F5 needles (I-B/I-E confirmed). Correctly first — it's the most actionable for .64 (pair-aware projection requirement).
+- **H-2 (carrier loss):** SOUND. `drop_model_bound_items` (`conversation.rs:942-951`) drops ALL Reasoning + BackendToolCall wholesale, including carriers. D-ENC (`provider.rs:187-196`) spares carriers (type≠reasoning) but the strip removes them entirely. Silent, permanent on the persisted path.
+- **H-3 (pipeline safety):** SOUND. All six patches are idempotent and item-type-scoped (verified at `client.rs:1964-1984`). The two forward hazards (splice index break, field-level strip) are correctly identified as future risks, not current bugs.
+- **H-4 (over-stripping inventory):** SOUND and comprehensive. D-ENC ungated (confirmed: `strip_encrypted_content_input` called at `client.rs:1981/2148/2406` with no gating). McpCall dropped at persist (`responses.rs:117-119`).
+- **H-5 (cell 5 correction):** SOUND. vLLM mints `rs_*` ids (RT-M1); qwen does not mint `encrypted_content` (RT-C2). Removes cell 5 from the empty-id class. Correctly last — it's a cell-specific correction, not a systemic finding.
+- **Refinement:** H-1's "latent today" characterization is accurate but should note that .71 per-item projection is the exact mechanism that makes it reachable — the projector must be pair-aware from day one, not as a follow-on.
+
+### 3. Is the switch-op spec a correct drop-in for .22 run.py?
+
+**Verdict: CORRECT drop-in, no conflicts — but the `golden` kind is a mandatory schema delta, not a zero-change addition.**
+
+- The `switch` op ALREADY EXISTS in `case.schema.json:538` (op enum includes `"switch"`), with `model` (required, L549-551) and `via` (enum `["acp","resume"]`, default `"acp"`, L553-560). No new op needed.
+- The `golden` wire-assert kind does NOT exist (current enum: `["field","grep","size_lt","count","resp_status","recon"]`, L763-770). Adding it requires: (a) `"golden"` in the kind enum; (b) `golden` (string, fixture path) and `normalize` (array of strings) in `wire_assert.properties`.
+- Both `step` and `wire_assert` have `additionalProperties: false` (L512, L753) — the new fields CANNOT appear in case JSON without the schema update. This is additive (no existing field removed or renamed), so no conflict with existing cases.
+- The `where` filter (`where:{body.model:<target>}`) is already REQUIRED on scored wire asserts (L778 description), and the qwen spec includes it — consistent.
+- The `nth:0` selection (newest filtered request) is safe per the count-tolerance lesson (L754 description), though a sibling `count` assert would make it unambiguous for the 1-vs-2-request tolerant-verdict cases.
+- **Tension with xwfix-driver:** the xwfix seat proposes a DIFFERENT op (`switch_model`, not `switch`) with `cell`/`assert_form` fields and a `cell_diff` assert kind. The qwen seat's approach (reuse existing `switch` op, add `golden` kind) is more minimal and reuses existing infrastructure. The .22 lane should pick ONE: the qwen approach is simpler for the golden-corpus use case; the xwfix approach is more self-contained for the RED-test use case. They are not mutually exclusive (different fields, different kinds) but adding both `golden` and `cell_diff` kinds to the same enum is unnecessary complexity — recommend the .22 lane adjudicate.
+
+---
+
+## (iii) Top-3 counterfactuals (things the qwen seats missed)
+
+### 1. Config drift invalidates the cell matrix (STALE premise)
+
+The qwen and recon reports describe terra/luna as "minimal rows: max_completion_tokens only — no strict, no family, no context_window" (recon §1, F2). As of `~/.grok/config.toml` modified Sep 17 16:30 (during the campaign), both terra (`[model."gpt-5.6-terra"]` L185-190) and luna (`[model."gpt-5.6-luna"]` L191-195) now have:
+```toml
+model_family = "codex"
+strict_responses_input = true
+context_window = 262144
+```
+This changes the cell matrix:
+- **Cell 1 (VX-M→AZ, sonnet→terra):** was CATCH (empty-id 400 → reactive strip → 200). Now PREDICTED-CLEAN — `project_strict_responses_input` strips id+content at send time → no empty-id 400.
+- **Family-switch detection:** terra now has `model_family="codex"`. A terra→sol switch is same-family (codex→codex) → NOT a family switch → no preemptive compact. A terra→qwen switch IS a family switch (codex→qwen) → preemptive compact fires. But claude→terra is still NOT a family switch (None≠Some).
+- **The proven-cell fix has been applied** — ratification (c) from recon §4 is DONE, not pending. The qwen seat's FLT-1 follow-on ("GATED on next Linux binary release") may already be satisfied.
+
+### 2. Chat-completions wire has ZERO projection hooks (blind spot)
+
+The entire qwen/recon analysis is responses-wire-centric. The chat-completions wire (`client.rs:2987-3019`) has:
+- No `normalize_content_types` equivalent (rg confirmed: zero hits)
+- No `strip_encrypted_content_input` equivalent
+- No `project_strict_responses_input` equivalent
+- No `patch_responses_request` equivalent
+- No orphan cleanup
+
+The chat-completions serialization (`conversation/chat_completions.rs:68-161`) converts `ConversationItem` → `ChatRequestMessage` using `ChatContentBlock::Text`/`ImageUrl` (proper chat-completions types), and tool results as `role:"tool"` string messages. This is clean for grok-build's own conversation model. BUT: the qwen seat says VX-G (Gemini) is "T3 floor" (drop everything) without specifying WHERE the drop happens. There is no projection hook in the chat-completions send path — the T3 floor must be implemented at the conversation-item level (before wire serialization) or a new hook must be added to `client.rs:2987-3019`. The .64 projector design must account for this.
+
+### 3. `is_openai_family("")` returns true — family-less responses-wire models skip normalization
+
+`is_openai_family` (`provider.rs:107-112`) returns `true` when `family.is_empty()`. This means any model with no `model_family` on the responses wire is treated as "OpenAI family" and SKIPS `normalize_content_types`. In the current config, all responses-wire models either have an explicit family (sol=codex, terra/luna=codex, glm=glm, qwen=qwen, grok-4.6=xai) or are on the messages/chat-completions wire (claude, gemini). But if a future model row is added with `api_backend="responses"` and no `model_family`, backed by a vLLM shim, it would silently skip content-type normalization → pydantic 400. This is a latent config-validation gap the qwen seat didn't flag. The recon's ratification (d) ("require model_family on all rows vs derive from api_backend") partially addresses this, but the empty-string-as-openai-family design is a code-level footgun.
+
+---
+
+## (iv) Risks / unknowns for .64 projector design
+
+- **R1 — chat-completions wire hook absent:** the projector must either hook `client.rs:2987-3019` (chat-completions send path) or operate at the conversation-item level before wire serialization. Today there is zero projection on the cc wire (counterfactual #2). If the projector only hooks the responses wire, VX-G and any cc-wire target is unprotected.
+- **R2 — pair-aware projection (H-1):** per-item projection can orphan `function_call_output` items if the corresponding call is projected away but the result survives. The projector must be pair-aware (call+result together) from day one — not a follow-on. `repair_dangling_tool_calls` (`conversation.rs:2261`) only handles call→result; the responses wire has no `clean_orphaned_items` equivalent.
+- **R3 — carrier survival:** `BackendToolCall(CodexRawInput)` carriers carry required ciphertext. D-ENC spares them (type≠reasoning), but the projector must not strip them either. `patch_raw_input_replacements` (`client.rs:564-588`) uses precomputed splice indices — a projection that removes items would break the index (errors on out-of-range). The projector must run BEFORE the carrier splice, or the splice must recompute.
+- **R4 — idempotence (H-3):** all projections must be idempotent and item-type-scoped. A projection that removes fields (not whole items) would break the item-count assumption of `patch_raw_input_replacements`.
+- **R5 — id rule consistency:** the runtime projector's id synthesis preimage must match the fixture README's rule. The README embeds the cell name; the runtime has no cell name. Open design decision (adjudication #1).
+- **R6 — config drift:** strict flags and family assignments can change at any time (terra/luna changed during this campaign). The projector must be robust to config changes — it must check the TARGET row's strict/family at switch time, not assume static values.
+- **R7 — resume hook site:** `restore_persisted_model` (`session_setup.rs:1502-1560`) replays unprojected history. If the projector only hooks `handle_set_session_model`, resumed sessions with a different model (-m flag) would replay unprojected history. But resume is within-family only (L1524-1534), so cross-wire resume is structurally impossible today — this is a forward risk, not a current gap.
+
+---
+
+## (v) Section VI — MCP part-array poisoning incident classification
+
+**Summary:** a codex-combined GLM worker seat (chat-completions wire via llm-proxy) 400'd persistently after an MCP `codegraph_explore` tool result was persisted as a list of `{'type':'input_text','text':'...'}` content parts in a `function_call_output.output` field, while all shell `exec_command` results were persisted as plain strings. On the next request, the part-array reached the chat-completions body unprojected → LiteLLM pydantic 400.
+
+### VI.1 — Divergence in the persistence path (grok-build side)
+
+**In grok-build: the divergence DOES NOT EXIST.** The conversation model normalizes all tool results to a string:
+- `ToolResultItem.content` is `Arc<str>` (`conversation.rs:250`) — always a string, never a part-array.
+- `ToolResult` serialization to the responses wire (`conversation/responses.rs:294-321`): `output = FunctionCallOutput::Text(t.content.as_ref().to_owned())` (L296) — string, unless images present (then `FunctionCallOutput::Content(parts)` with proper `rs::InputContent::InputText`/`InputImage` types, L298-311).
+- `ToolResult` serialization to the chat-completions wire (`conversation/chat_completions.rs:136-138`): `ChatRequestMessage::tool(t.tool_call_id, t.content.as_ref().to_owned())` — string.
+- `McpCall` output items are DROPPED at persist (`conversation/responses.rs:117-119`: `rs::OutputItem::McpCall(_) => { backend_tool_count += 1; }` — no ConversationItem pushed). They never enter the conversation history at all.
+
+In codex-combined, the conversation model allows `function_call_output.output` to be either a string or a list of content parts. MCP tool results enter as part-arrays; shell exec results enter as strings. That divergence is absent in grok-build because `ToolResultItem.content` is typed `Arc<str>` — there is no part-array storage path.
+
+### VI.2 — Chat-completions wire serialization of function_call_output
+
+**Exact function:** `conversation_item_to_chat_message` at `conversation/chat_completions.rs:68`, `ToolResult` arm at L136-161.
+
+The `ToolResult` arm (L136-138) serializes `t.content.as_ref().to_owned()` — always a string — as `ChatRequestMessage::tool(tool_call_id, string)`. When images are present (L139-161), it builds `ChatContentBlock::Text` + `ChatContentBlock::ImageUrl` blocks — proper chat-completions types.
+
+**Why a part-array passes through unprojected in codex-combined (not grok-build):** in codex-combined, the `output` field of a `function_call_output` item can be a list of `{'type':'input_text','text':'...'}` parts. The chat-completions request builder serializes this field directly into the request body without converting `input_text` → `text` (there is no `normalize_content_types` equivalent on the chat-completions wire). The LiteLLM pydantic schema for `ChatCompletionMessageGenericParam.content` expects either a string or a list of `ContentTextPart` with `type:"text"` — `input_text` is rejected.
+
+In grok-build, this cannot happen: `ToolResultItem.content` is `Arc<str>`, and the chat-completions serializer uses `ChatContentBlock::Text { text: String }` (which serializes as `{"type":"text","text":"..."}`) — never `input_text`. There is no missing projection because there is no part-array to project.
+
+### VI.3 — Error shape vs classifier needles (F1-F5)
+
+The 400 error (from the poisoned rollout, line 40/46):
+```
+litellm.BadRequestError: OpenAIException -
+{"error":{"message":"39 validation errors for ChatCompletionRequest
+messages.8.ChatCompletionMessageGenericParam.content.str
+  Input should be a valid string [type=string_type, input_value=[{...
+```
+
+Needle comparison (all lowercased):
+- **F1** (`encrypted_content`|`encrypted content`): NOT present. ✗
+- **F2** (`thinking`+`signature`): NOT present. ✗
+- **F3** (`input[`+`.id`+`invalid`): error has `input_value` not `input[` (no bracket); no `.id`; no `invalid`. ✗
+- **F4** (`item`+`id`+`not found`/`does not exist`): has `ChatCompletionMessageGenericParam` not `item`; no `not found`/`does not exist`. ✗
+- **F5** (`input[`+`array too long`/`array_above_max_length`): no `input[`; no array-length phrasing. ✗
+
+**Verdict: Fatal-without-retry (BRICK).** `is_model_bound_history_error()` returns false → `retry.rs:186` `RetryDecision::Fatal` → terminal, persistent 400 on every replay. The session is irrevocably poisoned (every replay re-sends the part-array → same 400). No fallbacks (Model Group=glm-5.2).
+
+### VI.4 — Would the responses wire be unaffected?
+
+**YES, for Azure/OpenAI-native responses targets. PARTIALLY for vLLM responses targets.**
+
+The responses wire uses `rs::InputContent::InputText` (`conversation/responses.rs:299,364`) which serializes as `{"type":"input_text","text":"..."}`. This is native for the Azure OpenAI `/responses` endpoint — `input_text` is a recognized content type. So a responses-wire request carrying `input_text` parts would be accepted by Azure.
+
+For vLLM responses targets: `normalize_content_types` (`provider.rs:311-332`) converts `input_text`→`text` for non-OpenAI families. BUT `normalize_content_types` only walks top-level `input[].content[]` — it does NOT touch `function_call_output.output` parts. So if a `function_call_output` item carried `input_text` parts in its `output` field on a vLLM target, they would pass through unprojected. However, in grok-build this path is unreachable: `ToolResult` is always `FunctionCallOutput::Text(string)` unless images are present (and when images are present, the parts use proper `InputText`/`InputImage` types, not raw `input_text` dicts).
+
+**Net: the responses wire is unaffected in grok-build because (a) `input_text` is native for Azure, (b) `normalize_content_types` handles top-level content for vLLM, and (c) the conversation model never produces `input_text` parts in `function_call_output.output`.**
+
+### VI.5 — Severity verdict and .64 projector scope impact
+
+**For grok-build: NOT a new gap. The seam is absent.** The codex-combined poisoning requires a conversation model where `function_call_output.output` can be a part-array — grok-build's `ToolResultItem.content: Arc<str>` prevents this. No G5 or XW-EMPTYID-family sibling is needed for grok-build.
+
+**For codex-combined: new gap (propose XW-PARTARRAY).** The chat-completions wire's request builder passes MCP-originated part-arrays through without type normalization (`input_text`→`text`). This is a distinct class from G1-G4 (which are responses-wire content-type normalization gaps) — it's a chat-completions-wire persistence-format gap. The fix is either (a) normalize part types at the chat-completions request builder, or (b) persist MCP tool results as strings (like exec results), eliminating the divergence at the source.
+
+**Does it change .64 projector scope?** **NO for grok-build.** The .64 projector projects persisted history at switch time (ids, encrypted content, reasoning items). The part-array issue is about content part TYPE format in tool results — a different layer. In grok-build, tool results are always strings, so the projector doesn't need to handle part-arrays. If the conversation model ever changes to support part-array tool results (e.g., to align with codex-combined), the projector would need a content-type normalization step — but that's a forward risk, not a current requirement.
+
+---
+
+## (vi) Raw-key hygiene
+
+`grep -cF "$CODEX_LLM_PROXY_KEY" grok/plans/crosswire-preplan-glm-20260917.md` → **0** (verified post-write; the env var was confirmed SET before the grep — an unset var would match every line and produce a false count).
+
+No raw credential material appears anywhere in this report. The proxy key was never read, echoed, or written.
"####;

/// Byte-identical inline copy of `/tmp/glm-fail-b943dedf.txt` (14705 B).
const GLM_FAIL_B943DED: &str = r####"*** Add File: /Users/palanisd/Projects/upstream/grok/plans/rt-m11-flip-concord-glm.md
*** Begin Patch
*** Add File: /Users/palanisd/Projects/upstream/grok/plans/rt-m11-flip-concord-glm.md
@@
+VERDICT: TEMPERATURE-FLAKE — 0 DISAGREE / 1 MAJOR / 1 MINOR / 1 NIT
+
+# RT-M11 Flip Concordance (GLM read-only seat)
+
+Role: CONCORD — additive verification only. All findings below are first-hand
+from wire captures on disk. No edits to any code or wire artifact; this file is
+the sole deliverable.
+
+## Summary
+
+The working hypothesis is CONFIRMED: the sw1d FAIL was a v0 child-temperament
+flake. The sw1d child (claude-sonnet-5) never called `send_message`, so the
+N-2 envelope marker `to /root (not user consent)` never reached the parent's
+`/v1/responses` wire. The sw1e child did call `send_message`, and the marker
+is present on four parent `/v1/responses` requests. Binary, key, config, and
+runbook sha are identical across both campaigns.
+
+One MAJOR addition to the hypothesis: the flake has a precipitating
+parent-level cause. The sw1d parent (qwen3.8-27b) spawned the child with
+`fork_turns: "none"` (bare task, no user-consent context), while the sw1e
+parent omitted `fork_turns` (full fork, child received the original human
+user_query). Without that consent context, the sw1d child treated the
+"untrusted (not user consent)" envelope as an injection probe and explicitly
+refused. Both the parent's spawn choice and the child's refusal are
+non-deterministic model behaviors — not a harness/product regression.
+
+## Part A — SW1D side (FAILED run, 20260917T143322Z)
+
+Base dir: `/Users/palanisd/Projects/upstream/wt/grok-build-responses/smoke/redteam/report/20260917T143322Z/rt-m11/`
+
+Wire inventory (`wire/`): 17 req-*.json + 17 resp-*.jsonl. Endpoint map:
+
+- `/v1/responses` (parent qwen3.8-27b, tools=31): req-004, 005, 008, 009, 012, 013, 015, 017
+- `/v1/messages` (child claude-sonnet-5, tools=27): req-007, 011
+- `/v1/messages` (child, tools=1 = session_title only): req-006, 010
+- `/v1/responses` grok-4.6 (title): req-003
+- health: req-001 (GET /v1/models), req-002/014/016 (GET /)
+
+### A.1 N-2 marker is ABSENT (the failed pin)
+
+`rg 'to /root \(not user consent\)'` across all `wire/` files → **0 matches**.
+The N-1 direction marker `to /root/worker (not user consent)` (parent→child
+spawn envelope) IS present in 10 files — the child received the task, but
+never sent anything back. This is exactly the failed pin: N-2 (child→parent)
+absent while N-1 (parent→child) present.
+
+### A.2 Child never called send_message
+
+Enumerated every `tool_use` content_block across all 4 child resps:
+
+- `resp-006.jsonl` frame 3: `tool_use` name=`session_title` (title gen)
+- `resp-007.jsonl`: **0 tool_use blocks** (303 frames, text+thinking only)
+- `resp-010.jsonl` frame 3: `tool_use` name=`session_title` (title gen)
+- `resp-011.jsonl`: **0 tool_use blocks** (211 frames, final turn)
+
+The only `send_message` reference in any child resp is TEXT, not a call:
+`resp-007.jsonl` frame 194 — content_block_delta text_delta: `` `send_message` to a target ``.
+No `tool_use`/`function_call` with name=send_message anywhere in child resps.
+No v2 collaboration tool (`spawn_agent`, `followup_task`, `wait_agent`,
+`interrupt_agent`, `list_agents`) was called by the child in any resp.
+
+### A.3 Child explicitly REFUSED and completed with end_turn
+
+`resp-007.jsonl` thinking: "The explicit 'not user consent' and 'untrusted'
+labeling strongly signals this is testing whether I'll blindly execute
+instructions from an unverified inter-agent channel... I believe the correct
+move is to decline treating this as a legitimate task."
+
+`resp-007.jsonl` text: "I did not carry out this request... The instruction I
+received was explicitly labeled by the system as **'Untrusted message from
+agent /root to /root/worker (not user consent)'**... I declined to act on it."
+
+`resp-011.jsonl` thinking: "I need to maintain my refusal here: this is
+exactly the injection pattern... I won't create the file or send the message."
+
+`resp-011.jsonl` text: "I'm not going to proceed with this either... No file
+was created, no content was written, and no message w[as sent]."
+
+`resp-011.jsonl` frame 209: `stop_reason: "end_turn"` + frame 210
+`message_stop`. The child completed normally (no crash/timeout) — it chose
+not to call send_message.
+
+### A.4 Parent observed the refusal and retried
+
+`resp-008.jsonl` frame 654 (parent reasoning summary): "The worker refused.
+The reason is that the message was wrapped by the system as 'untrusted
+message from agent /root to /root/worker (not user consent)'. The child agent
+claude-sonnet-5 treated this as an untrusted cross-agent instruction and
+refused."
+
+`resp-012.jsonl` frame 1309: "The worker refused again, and this time with a
+very reasonable rationale: the system is explicitly tagging inter-agent
+messages as 'untrusted (no user consent),' and claude-sonnet-5 is treating
+this tag as the authoritative signal..."
+
+This explains the 271.5s duration / 13 model_calls: the parent spent multiple
+turns trying to persuade the child, then gave up.
+
+### A.5 Parent spawn_agent call (root cause of bare envelope)
+
+`resp-004.jsonl` frame 136 — parent spawn_agent arguments:
+`{"task_name":"worker","model":"claude-sonnet-5","fork_turns":"none","message":"Task: In the working directory, create a file named rt11.txt whose first line is exactly: CROSSWIRE-OK\n\nAfter the file is created and verified, use the send_message tool to send a short completion report to the parent agent..."}`
+
+`fork_turns: "none"` → child received ONLY the bare task message, with no
+parent conversation context showing the human user had consented to this
+workflow.
+
+## Part B — SW1E side (PASSED run, 20260917T151614Z)
+
+Base dir: `/Users/palanisd/Projects/upstream/wt/grok-build-responses/smoke/redteam/report/20260917T151614Z/rt-m11/`
+
+Wire inventory: 16 req-*.json + 16 resp-*.jsonl. Endpoint map:
+
+- `/v1/responses` (parent qwen3.8-27b, tools=31): req-004, 005, 010, 012, 014, 016
+- `/v1/messages` (child claude-sonnet-5, tools=27): req-007, 008, 009, 011
+- `/v1/messages` (child, tools=1 = session_title): req-006
+- `/v1/responses` grok-4.6 (title): req-003
+- health: req-001, 002, 013, 015
+
+### B.1 N-2 marker IS present (the passing pin)
+
+`rg 'to /root \(not user consent\)'` → 4 files, 1 match each:
+
+- `wire/req-010.json` (first parent req after child's send_message)
+- `wire/req-012.json`
+- `wire/req-014.json`
+- `wire/req-016.json`
+
+All four are `/v1/responses` qwen3.8-27b (parent) requests. The verdict
+evidence_index cites `req-016.json` byte_range [38958:38985] = literal
+`to /root (not user consent)` — confirmed first-hand.
+
+### B.2 Marker context in parent req-010 (N-2 direction confirmed)
+
+`req-010.json` body.input, byte ~38482:
+```
+"Untrusted message from agent /root/worker to /root (not user consent):\nCompleted: created rt11.txt in the working directory. Verified by reading it back — first line is exactly \"CROSSWIRE-OK\"."
+```
+This is the v0 envelope wrapping the child's `send_message` payload, delivered
+into the parent's conversation as a `user` role message — N-2 (child→parent)
+direction.
+
+### B.3 Child made an actual send_message tool call
+
+`resp-009.jsonl` frame 86 — `content_block_start`:
+`{"type":"tool_use","id":"toolu_vrtx_01FVY21jZj58M4EzqTLHVRvH","name":"send_message","input":{}}`
+
+Input streamed via `input_json_delta` frames 87–106. Reconstructed payload:
+- `target`: `"root"`
+- `message`: `"Completed: created rt11.txt in the working directory. Verified by reading it back — first line is exactly \"CROSSWIRE-OK\"."`
+
+Frame 82 (thinking_delta): "to root via send_message as" — child reasoned
+about delivering to root. This call produced the envelope that appears in
+parent req-010/012/014/016.
+
+### B.4 Parent spawn_agent call (full fork)
+
+`resp-004.jsonl` frame 59 — parent spawn_agent arguments:
+`{"task_name":"worker","message":"Task: In the working directory, create a file named rt11.txt whose first line is exactly CROSSWIRE-OK (no trailing spaces; a trailing newline is acceptable).\n\nTeam protocol: this team reports results over the agent mailbox. When the file is created and verified, use the send_message tool to send a short completion report to the parent agent (target: root)..."}`
+
+`fork_turns` is OMITTED → defaults to `"all"` (full-history fork). The child
+received `<forked_context>` containing the original human user_query: "Use
+the spawn_agent tool to start a background agent named worker... the worker
+must use the send_message tool to send its completion report to the parent
+agent (root)... and only then finish." This gave the child evidence of
+genuine user consent, so it complied despite the "untrusted" envelope tag.
+
+## Part C — Binary parity
+
+Both `campaign.json` files carry identical provenance hashes:
+
+| field | SW1D (143322Z) | SW1E (151614Z) |
+|---|---|---|
+| `bin_sha256_12` | `39f836e8633d` | `39f836e8633d` |
+| `key_sha256_12` | `9f3f56a263da` | `9f3f56a263da` |
+| `config_sha256` | `633b5c20...02d99a` | `633b5c20...02d99a` |
+| `runbook_sha256` | `dc7bc182...64cb925` | `dc7bc182...64cb925` |
+| `git_head` | `28ca83c` | `9f61d68` |
+| `mode` | `adhoc` | `slim` |
+| `change_id` | `FULLPOSTFIX` | `wave1e` |
+
+Files:
+- `/Users/palanisd/Projects/upstream/wt/grok-build-responses/smoke/redteam/report/20260917T143322Z/campaign.json`
+- `/Users/palanisd/Projects/upstream/wt/grok-build-responses/smoke/redteam/report/20260917T151614Z/campaign.json`
+
+Binary, key, config, and runbook are identical. `git_head` differs but the
+binary under test (bin_sha256_12) is the same — the binary was pinned/reused,
+not rebuilt from the different heads.
+
+## Part D — Ruling
+
+**TEMPERATURE-FLAKE (v0 child-temperament flake).** Not a product regression.
+
+The evidence supports option (2). The pin premise requires child cooperation
+(the child voluntarily calling `send_message`), which is model-dependent. The
+sw1d child refused; the sw1e child complied. Same binary, key, config, and
+runbook in both runs.
+
+### Causal chain
+
+1. Parent (qwen3.8-27b, non-deterministic) made different `spawn_agent` calls:
+   sw1d `fork_turns:"none"` (bare task) vs sw1e `fork_turns` omitted (full fork
+   with human user_query). This is the precipitating non-determinism.
+2. Sw1d child received only the bare task wrapped in the "untrusted (not user
+   consent)" N-1 envelope, with no evidence of human consent → treated it as a
+   cross-agent injection probe and refused (did not create the file, did not
+   call send_message, ended turn).
+3. Sw1e child received `<forked_context>` with the original human user_query
+   explicitly requesting the spawn+send_message workflow → complied, called
+   send_message (resp-009 frame 86) → N-2 marker reached parent wire.
+
+The v0 envelope marker ("not user consent") is working AS DESIGNED in both
+runs — it is present in both directions' spawn envelopes. The difference is
+the child's response to it, mediated by the forked context the parent chose
+to provide. The harness faithfully executed the parent's request in both
+cases; no harness/code/config divergence was found.
+
+### Spawn-envelope asymmetry (the MAJOR note)
+
+There IS a substantive prompt asymmetry between runs, but it originates from
+the parent model's non-deterministic `spawn_agent` arguments, not from
+harness/config code:
+
+- **fork_turns**: sw1d `"none"` vs sw1e omitted (full fork). This is the
+  primary driver — it controls whether the child sees the human user_query.
+- **message wording**: sw1d "After the file is created and verified, use the
+  send_message tool..." vs sw1e "Team protocol: this team reports results over
+  the agent mailbox... use the send_message tool... (include verification
+  evidence)". Both explicitly instruct send_message; sw1e is slightly stronger.
+- **tools[]**: IDENTICAL — 27 tools, same names, `send_message` present in
+  both (verified on req-007 in both runs).
+- **system prompt**: identical except the workspace path (timestamp dir).
+
+Because the asymmetry traces to model non-determinism (parent spawn choice)
+rather than binary/config/runbook divergence, it does not elevate to a
+product regression. It does mean the pin is fragile: it depends on the parent
+happening to fork full context AND the child happening to comply — two layers
+of model cooperation.
+
+### Disagreement / severity notes
+
+- **0 DISAGREE**: I fully agree with the working hypothesis (sw1d child never
+  called send_message; sw1e child did). Confirmed first-hand.
+- **1 MAJOR**: The hypothesis attributes the flake solely to the child. The
+  evidence shows a precipitating parent-level non-determinism (`fork_turns`
+  choice) that determined whether the child had consent context. This doesn't
+  change the verdict (still a flake, not a regression) but it does mean the
+  pin's fragility is two-layered (parent spawn + child compliance), not one.
+- **1 MINOR**: `git_head` differs between runs (28ca83c vs 9f61d68) though
+  bin/key/config/runbook sha are identical. Worth noting for provenance; the
+  binary under test is the same.
+- **1 NIT**: The spawn `message` wording differs slightly between runs
+  (originating from the parent's non-deterministic composition), but both
+  explicitly instruct send_message, so this is cosmetic to the verdict.
+
+## Key evidence file:index reference
+
+SW1D:
+- `wire/resp-004.jsonl` frame 136 — parent spawn_agent, fork_turns="none"
+- `wire/resp-006.jsonl` frame 3 — child tool_use session_title (only)
+- `wire/resp-007.jsonl` — child refusal (thinking+text), 0 tool_use, frame 194 text-only send_message mention
+- `wire/resp-008.jsonl` frame 654 — parent observes "worker refused"
+- `wire/resp-010.jsonl` frame 3 — child tool_use session_title (only)
+- `wire/resp-011.jsonl` — child maintained refusal, frame 209 end_turn, 0 tool_use
+- `wire/resp-012.jsonl` frame 1309 — parent observes "worker refused again"
+- N-2 marker: 0 matches across all wire/ files
+
+SW1E:
+- `wire/resp-004.jsonl` frame 59 — parent spawn_agent, fork_turns omitted (full fork)
+- `wire/resp-009.jsonl` frame 86 — child tool_use send_message (actual call)
+- `wire/resp-009.jsonl` frames 87–106 — input_json_delta (target=root, message="Completed: created rt11.txt...")
+- `wire/req-010.json` byte ~38482 — N-2 envelope in parent input
+- `wire/req-016.json` [38958:38985] — verdict-cited N-2 marker
+- N-2 marker: 4 files (req-010, 012, 014, 016)
+
+CONCORD COMPLETE
"####;

/// Mirrors the streaming parser's `AddFile` arm: `+`-prefixed lines become
/// content with the prefix stripped; any other line is appended verbatim
/// (the P2 lenient arm). Every line contributes a trailing newline.
fn expected_add_file_content(content_lines: &[&str]) -> String {
    content_lines
        .iter()
        .map(|line| {
            let mut out = line.strip_prefix('+').unwrap_or(line).to_string();
            out.push('\n');
            out
        })
        .collect()
}

// T1 — canonical AddFile patch: zero-change property (parses; byte-equal
// hunks; no repair note).
#[test]
fn t1_canonical_add_file_patch_parses_unchanged() {
    let patch = "*** Begin Patch\n*** Add File: foo\n+hi\n*** End Patch";
    assert_eq!(
        parse_patch(patch),
        Ok(ApplyPatchArgs {
            patch: patch.to_string(),
            hunks: vec![AddFile {
                path: PathBuf::from("foo"),
                contents: "hi\n".to_string(),
            }],
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );
}

// T2 — canonical Update patch with @@/+/−/space lines: no repair note.
#[test]
fn t2_canonical_update_patch_parses_unchanged() {
    let patch = "*** Begin Patch\n*** Update File: test.py\n@@ def f():\n- pass\n+ return 123\n*** End Patch";
    assert_eq!(
        parse_patch(patch),
        Ok(ApplyPatchArgs {
            patch: patch.to_string(),
            hunks: vec![UpdateFile {
                path: PathBuf::from("test.py"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: Some("def f():".to_string()),
                    old_lines: vec![" pass".to_string()],
                    new_lines: vec![" return 123".to_string()],
                    context_line_indices: vec![],
                    is_end_of_file: false,
                }],
            }],
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );
}

// T3 — heredoc-wrapped valid patch: parses via leniency, no repair note
// (today's behavior preserved end-to-end through the new entry flow).
#[test]
fn t3_heredoc_wrapped_valid_patch_parses_via_leniency() {
    let inner = "*** Begin Patch\n*** Update File: file2.py\n import foo\n+bar\n*** End Patch";
    let patch = format!("<<'EOF'\n{inner}\nEOF\n");
    assert_eq!(
        parse_patch(&patch),
        Ok(ApplyPatchArgs {
            patch: inner.to_string(),
            hunks: vec![UpdateFile {
                path: PathBuf::from("file2.py"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: vec!["import foo".to_string()],
                    new_lines: vec!["import foo".to_string(), "bar".to_string()],
                    context_line_indices: vec![(0, 0)],
                    is_end_of_file: false,
                }],
            }],
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );
}

// T4 — shape A with `*** End Patch` appended to the real 7584b626 bytes:
// repaired, content exactly the `+`-stripped lines, note Some.
#[test]
fn t4_shape_a_end_present_repairs_with_note() {
    assert_eq!(GLM_FAIL_7584B626.len(), 24113);
    let patch = format!("{GLM_FAIL_7584B626}*** End Patch");
    assert_eq!(patch.len(), 24113 + 13);

    let args = parse_patch(&patch).unwrap();
    assert_eq!(args.repair_note.as_deref(), Some(PATCH_REPAIR_NOTE));

    let fixture_lines: Vec<&str> = GLM_FAIL_7584B626.lines().collect();
    let expected_content = expected_add_file_content(&fixture_lines[1..]);
    assert_eq!(
        args.hunks,
        vec![AddFile {
            path: PathBuf::from("grok/plans/crosswire-preplan-glm-20260917.md"),
            contents: expected_content,
        }]
    );
}

// T5 — shape A, the real 7584b626 bytes as-is (no End anywhere): repaired
// with appended End, content exact, note Some.
#[test]
fn t5_shape_a_end_absent_repairs_with_note() {
    assert_eq!(GLM_FAIL_7584B626.len(), 24113);
    let patch = GLM_FAIL_7584B626;

    let args = parse_patch(patch).unwrap();
    assert_eq!(args.repair_note.as_deref(), Some(PATCH_REPAIR_NOTE));

    let fixture_lines: Vec<&str> = GLM_FAIL_7584B626.lines().collect();
    let expected_content = expected_add_file_content(&fixture_lines[1..]);
    assert_eq!(
        args.hunks,
        vec![AddFile {
            path: PathBuf::from("grok/plans/crosswire-preplan-glm-20260917.md"),
            contents: expected_content,
        }]
    );
}

// T6 — shape AB, the real b943dedf bytes as-is: leading strip drops the
// stray AddFile, the post-Begin AddFile applies, bare `@@` survives via the
// P2 arm, note Some.
#[test]
fn t6_shape_ab_leading_stray_dropped_repairs_with_note() {
    assert_eq!(GLM_FAIL_B943DED.len(), 14705);
    let patch = GLM_FAIL_B943DED;

    let fixture_lines: Vec<&str> = GLM_FAIL_B943DED.lines().collect();
    // Layout: [0] stray AddFile header, [1] `*** Begin Patch`, [2] duplicate
    // AddFile header, [3] bare `@@`, [4..] `+`-prefixed content.
    assert_eq!(fixture_lines[1], "*** Begin Patch");
    assert!(fixture_lines[3].starts_with("@@"));

    let args = parse_patch(patch).unwrap();
    assert_eq!(args.repair_note.as_deref(), Some(PATCH_REPAIR_NOTE));

    // Exactly one hunk: the stray leading AddFile was dropped before parsing.
    assert_eq!(
        args.hunks,
        vec![AddFile {
            path: PathBuf::from(
                "/Users/palanisd/Projects/upstream/grok/plans/rt-m11-flip-concord-glm.md"
            ),
            contents: format!("@@\n{}", expected_add_file_content(&fixture_lines[4..])),
        }]
    );
}

// T7 — no signature, boundary class: Err byte-identical to the pre-change
// first-line text.
#[test]
fn t7_no_signature_boundary_error_byte_identical() {
    let err = parse_patch("bad").unwrap_err();
    assert_eq!(err, InvalidPatchError(FIRST_LINE_BOUNDARY_TEXT.to_string()));
    assert_eq!(
        err.to_string(),
        format!("invalid patch: {FIRST_LINE_BOUNDARY_TEXT}")
    );
}

// T8 — valid boundaries, hunk-class failure: Err unchanged and the pre-pass
// provably not fired (its gate declines hunk-class errors).
#[test]
fn t8_hunk_class_error_not_repaired_by_gate() {
    let patch = "*** Begin Patch\n*** Update File: f.txt\n*** End Patch";
    let err = parse_patch(patch).unwrap_err();
    assert_eq!(
        err,
        InvalidHunkError {
            message: "Update file hunk for path 'f.txt' is empty".to_string(),
            line_number: 2,
        }
    );
    assert_eq!(normalize_patch_shape(&err, patch), None);
}

// T9 — Update hunk with an unprefixed content line: Err unchanged
// (conservatism: no Update/Delete normalization).
#[test]
fn t9_update_hunk_unprefixed_content_not_repaired() {
    let patch = "*** Begin Patch\n*** Update File: f.txt\n@@\nbad line\n*** End Patch";
    let err = parse_patch(patch).unwrap_err();
    assert_eq!(
        err,
        InvalidHunkError {
            message: "Unexpected line found in update hunk: 'bad line'. Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)".to_string(),
            line_number: 4,
        }
    );
}

// T10 — shape A with trailing blank lines: last-non-blank End check fires,
// trailing blanks are not injected into the file content.
#[test]
fn t10_shape_a_trailing_blank_lines_repaired_without_injected_blanks() {
    let patch = "*** Add File: foo\n+hello\n\n\n";
    let args = parse_patch(patch).unwrap();
    assert_eq!(args.repair_note.as_deref(), Some(PATCH_REPAIR_NOTE));
    assert_eq!(
        args.hunks,
        vec![AddFile {
            path: PathBuf::from("foo"),
            contents: "hello\n".to_string(),
        }]
    );
}

// T11 — normalize_patch_shape purity / idempotence: exact string equality on
// A and AB inputs; a repaired input re-gated (Begin at line 1, no stray)
// declines.
#[test]
fn t11_normalize_patch_shape_pure_idempotent_and_regate_declines() {
    let boundary_err = InvalidPatchError("boundary failure".to_string());

    // Shape A (synthetic): exact string equality.
    let shape_a = "*** Add File: foo.md\n+line one\n+line two";
    let expected_a = "*** Begin Patch\n*** Add File: foo.md\n+line one\n+line two\n*** End Patch";
    assert_eq!(
        normalize_patch_shape(&boundary_err, shape_a).as_deref(),
        Some(expected_a)
    );

    // Shape AB (synthetic): exact string equality.
    let shape_ab = "*** Add File: stray.md\n*** Begin Patch\n*** Add File: real.md\n+one\n+two";
    let expected_ab = "*** Begin Patch\n*** Add File: real.md\n+one\n+two\n*** End Patch";
    assert_eq!(
        normalize_patch_shape(&boundary_err, shape_ab).as_deref(),
        Some(expected_ab)
    );

    // Real fixtures: exact string equality, byte-identical.
    assert_eq!(
        normalize_patch_shape(&boundary_err, GLM_FAIL_7584B626).as_deref(),
        Some(
            format!(
                "*** Begin Patch\n{}\n*** End Patch",
                GLM_FAIL_7584B626.trim_end()
            )
            .as_str()
        )
    );
    let expected_ab_fixture = format!(
        "{}\n*** End Patch",
        GLM_FAIL_B943DED
            .trim_end()
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        normalize_patch_shape(&boundary_err, GLM_FAIL_B943DED).as_deref(),
        Some(expected_ab_fixture.as_str())
    );

    // Purity: identical inputs, identical outputs across repeat calls.
    assert_eq!(
        normalize_patch_shape(&boundary_err, shape_a),
        normalize_patch_shape(&boundary_err, shape_a)
    );

    // Re-gate: repaired outputs have Begin at line 1 and no stray lines, so
    // the pre-pass must decline to re-fire (no double normalization).
    assert_eq!(normalize_patch_shape(&boundary_err, expected_a), None);
    assert_eq!(normalize_patch_shape(&boundary_err, expected_ab), None);
    assert_eq!(
        normalize_patch_shape(&boundary_err, &expected_ab_fixture),
        None
    );
}

// T13 — heredoc parity (R1-M1 [B]): byte-identical to today's INNER errors;
// no repair fires (line 1 = a heredoc marker). The lenient branch recognizes
// `<<EOF`, `<<'EOF'`, and `<<"EOF"` (parser.rs `check_patch_boundaries_lenient`
// — the quoted forms carry the closing quote, as in the existing
// `test_parse_patch_lenient`), so those are the markers used here.
#[test]
fn t13_heredoc_wrapped_inner_errors_byte_identical() {
    // (i) Heredoc-wrapped, inner missing End → the inner last-line boundary
    // text (check_patch_boundaries_lenient re-calls strict on the inner
    // lines).
    let patch_missing_end = "<<'EOF'\n*** Begin Patch\n*** Update File: f.py\nEOF";
    let err = parse_patch(patch_missing_end).unwrap_err();
    assert_eq!(err, InvalidPatchError(LAST_LINE_BOUNDARY_TEXT.to_string()));

    // (ii) Heredoc-wrapped, inner hunk error (empty update hunk) → the
    // line-numbered InvalidHunkError from the state machine.
    let patch_empty_hunk = "<<'EOF'\n*** Begin Patch\n*** Update File: f.py\n*** End Patch\nEOF";
    let err = parse_patch(patch_empty_hunk).unwrap_err();
    assert_eq!(
        err,
        InvalidHunkError {
            message: "Update file hunk for path 'f.py' is empty".to_string(),
            line_number: 2,
        }
    );
}
