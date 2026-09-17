# vLLM GLM-5.2 tool-call argument research (cdx1_glm_proxy incident)

Research target: vllm-project/vllm `main` @ `0fefffc93466` (2026-09-15), plus release
tag `v0.22.0` for comparison. Failing model: **glm-5.2** served via the Responses API;
sibling provider `llm_proxy_qwen` (qwen3_coder parser) unaffected.

**Verdict (one line):** vLLM does not corrupt the 24 KB patch (string args pass through
byte-for-byte); the observed `arguments: {}` is fully explained by vLLM's GLM parser
turning "no / unclosed `<arg_key>/<arg_value>` pairs" into `{}` — most likely a
model-side missing opening `<arg_value>` tag (open vLLM bug
vllm-project/vllm#49248, ~5–17% of calls under concurrency, reproduces at
temperature 0) or a zero-arg model output; max_tokens truncation yields empty or
*invalid* (partial) arguments, not `{}`. Both observed failure classes are model-side;
the wire was not at fault, and a client-side fix can only detect/retry, not recover.

**Version/parsing-path notes.**
- On the Python OpenAI server (`vllm serve`, what LiteLLM hits) the tool parser is
  always the explicit `--tool-call-parser` flag; GLM-5.x deployments run
  `--tool-call-parser glm47 --reasoning-parser glm45` (the exact configs in #49248 and
  #55541). Model-name auto-detection exists only in the new Rust chat service, where
  `"glm-5"` (case-insensitive substring) maps to the `glm47` parser
  (`rust/src/chat/src/parser/tool/mod.rs:112`); an unrecognized name simply gets no
  parser on the Python server.
- `path:line` citations are `main @0fefffc9` unless prefixed `v0.22.0`.

## Q1. Which vLLM code parses GLM-5.x tool calls

- Registered parser names: `glm45` and `glm47` both → `Glm47MoeModelToolParser`
  (`vllm/tool_parsers/__init__.py:61-68`). v0.22.0: `glm45` → `Glm4MoeModelToolParser`,
  `glm47` → `Glm47MoeModelToolParser` (v0.22.0 `vllm/tool_parsers/__init__.py:53-60`).
- GLM-5.x is explicitly known: `register_pattern("glm-5", GLM47)`, `"glm-4.7"` →
  GLM47, `"glm-4.6"`/`"glm-4.5"` → GLM45 (`rust/src/chat/src/parser/tool/mod.rs:112-115`;
  tests pin `zai-org/GLM-5.2-FP8` → glm47, `rust/src/chat/src/parser/reasoning/tests.rs:124-126`).
- Python serving path: thin adapter `vllm/tool_parsers/glm47_moe_tool_parser.py:9-11`
  wraps the engine parser `Glm47MoeParser` (`vllm/parser/glm47_moe.py:176`) through
  `ParserEngineToolAdapter` (`vllm/parser/engine/adapters.py:153`).
- Template boundaries: vLLM does **not** own the GLM template — the model's HF
  `chat_template.jinja` instructs it to emit, per call:

  ```text
  <tool_call>
  func_name
  <arg_key>key</arg_key>
  <arg_value>value</arg_value>
  </tool_call>
  ```

  Parser side: `_ARG_RE` (`vllm/parser/glm47_moe.py:40-44`) requires **both**
  the opening and closing value tag of each pair; a value whose opening tag is
  missing is silently dropped (Q2(a); vllm-project/vllm#49248). Tag constants:
  `vllm/parser/glm47_moe.py:33-39`.

### Q2. When does vLLM emit empty/malformed arguments

**(a) Model emits no args / unclosed pairs.** The model emits `</arg_key>`
followed directly by the value with no opening `<arg_value>` tag (observed in #49248
raw logprobs), or a name-only call with no pairs: `_glm47_arg_converter` finds
no complete pair → `json.dumps({})` = `{}`
(`vllm/parser/glm47_moe.py:46-60`). Non-streaming: same full-text `_ARG_RE`
scan. This is the most likely origin of F3-style `{}` calls on a glm-5.2
seat; #49248 measures it at ~5–17% of calls under 6-way concurrency
(0 sequential, temperature 0). Fix #49249 (optional opening tag) is open, not
in main. (The single observed F3 in our rollouts was on a qwen3.8-27b
session — a different parser path — so it remains classified as glitch; this
mechanism is the standing risk for glm seats.)

**(b) Truncation by max_tokens.**
- main, streaming: engine `finish()` *closes* an unterminated call by emitting
  `TOOL_CALL_END` (`vllm/parser/engine/streaming_parser_engine.py:287`, `:312-322`); the
  non-partial GLM converter then drops the unclosed value → final `{}`; and because
  `{}` is not a prefix of the partial JSON already streamed, the correction delta is
  suppressed (`vllm/parser/engine/parser_engine.py:1033-1049`) — the client is left
  holding the unterminated partial JSON it accumulated from deltas.
- main, non-streaming: body with unclosed value → converter misses the pair → `{}`.
- v0.22.0 streaming: incomplete call streams `{"patch": "…` with an *open* JSON quote
  and no closing brace (v0.22.0 `glm4_moe_tool_parser.py`, `_build_args_json_so_far`:
  `is_complete=False` → no closing `"`/`}`); v0.22.0 non-streaming: unterminated
argument value → incomplete pair → `{{}}` (the v0.22.0 regexes impose the same
pair-completeness requirement; the streaming path instead leaves the client
holding an unterminated JSON prefix).

**(c) Template/regex mismatch.** Any deviation from the pair grammar: Python path
silently drops the pair (`_ARG_RE`/`func_arg_regex` require both tags); the Rust
parser (Rust chat endpoint) is strict and errors instead — "tool parser parsing
failed" / "incomplete GLM MoE tool call"
(`rust/src/parser/src/tool/glm_xml/mod.rs:130-138`, `:236-266`). Unknown/invalid tool
name → call dropped from both `tool_calls` and content
(`vllm/parser/engine/parser_engine.py:408-415`, `:1110-1115`).

**(d) Unparseable → fallback to text.** v0.22.0: an exception in
`extract_tool_calls` returns the whole raw `model_output` as `content` (v0.22.0
`glm4_moe_tool_parser.py`, `except` block at end of `extract_tool_calls`). main:
engine-based parsers deliberately drop incomplete tool-call markup from content so
non-streaming matches streaming (comment, `vllm/parser/abstract_parser.py:543-549`).

### Q3. Schema-guided decoding for strict=false tools

No constrained decoding in our config: `get_model_structural_tag` returns `None` when
`tool_choice == "auto"` and no tool has `strict=true`
(`vllm/tool_parsers/structural_tag_registry.py:120`). With all tools `strict=false`
(our `apply_patch`), vLLM main leaves GLM sampling unconstrained; the tool schemas are
consumed only for:
- post-hoc type coercion `_fix_arg_types` → `coerce_to_schema_type`
  (`vllm/parser/engine/parser_engine.py:381`; `vllm/tool_parsers/utils.py:1080`) — a
  `string`-typed `patch` value passes through unchanged;
- streaming "streamable string keys" bookkeeping (`parser_engine.py:366-379`).

It *could* be enforced server-side on main: `structural_tag_model = "glm_4_7"`
(`vllm/tool_parsers/glm47_moe_tool_parser.py:11`) + `VLLM_ENFORCE_STRICT_TOOL_CALLING`
(default **True**, `vllm/envs.py:240`, `:1751-1753`) injects an xgrammar structural tag
(embedding the parameters schemas) via `Parser.adjust_request` → `_apply_structural_tag`
(`vllm/parser/abstract_parser.py:564-600`) when any tool is `strict=true` *or*
tool_choice is required/named. But GLM-5.x + constrained decoding is a known-broken
cluster: xgrammar FSM crash / infinite hang with GLM-5.2-NVFP4 on v0.26.0 (#49981);
forced named tool_choice runs to max_tokens with truncated arguments on GLM-5.3-Flash
(#55541); JSON repetition until context limit with `required`+streaming on v0.24.0
(#47504, fix #47512 open). v0.22.0's GLM parser explicitly *skipped* guided decoding
for required/named ("Guided decoding would force JSON output, conflicting with the XML
format", v0.22.0 `glm4_moe_tool_parser.py`, `adjust_request`). So: older releases = no
schema enforcement for GLM at all; newer = opt-in via strict/required, currently unsafe
for GLM-5.x.

### Q4. Known issues (large args / newlines / `*` / `+` / unicode)

- No vLLM issue reports byte corruption of large GLM arguments; the intact 24 KB
  pass-through matches the code (string values are not stripped: v0.22.0
  `_build_args_json_so_far` — "Don't strip string values — whitespace is significant";
  main `_glm47_arg_converter` keeps the raw value).
- #32829 (closed): GLM-4.7 long string args were *buffered* until complete (multi-second
  latency, no content) — fixed by the incremental streaming parser (v0.22.0
  `glm4_moe_tool_parser.py` docstring). Latency, not corruption.
- #49248 (open): GLM-5.2 + `glm47` parser silently returns `{}` when the model omits
  the opening `<arg_value>` tag; ~5–17% of calls at 6-way concurrency, 0 sequential,
  reproducible at temperature 0 (raw logprobs show `</arg_key>` followed directly by the
  intact value). Fix PR #49249 (make opening tag optional) still **open**; not in main.
- #47504 (open): GLM required+streaming → arguments repeat until context length
  (v0.24.0). #55541 (open): GLM-5.3-Flash forced tool_choice → 133–144-char truncated
  invalid arguments at max_tokens. #49981 (open): GLM-5.2-NVFP4 required → xgrammar FSM
  crash/hang (v0.26.0).
- Newlines / `*` / `+` / unicode inside values: no parser-level handling (raw
  pass-through); the only transform is JSON escaping at serialization (`json.dumps`,
  `ensure_ascii=False`) — lossless and reversible.

### Q5. SGLang comparison (brief)

Parsers: `python/sglang/srt/function_call/glm47_moe_detector.py` (GLM-4.7/5.x) and
`glm4_moe_detector.py` (GLM-4.5/4.6), sgl-project/sglang `main`.
`func_arg_regex` (glm47_moe_detector.py:317-318) also requires the opening
`<arg_value>` tag → the missing-`<arg_value>` model quirk drops the argument there too
(→ `{}`), same as vLLM.
Streaming: char-level state machine (INIT/BETWEEN/IN_KEY/WAITING_VALUE/IN_VALUE,
:43-46, :437-539). In WAITING_VALUE, content arriving before `<arg_value>` sits in a
tag buffer and is never emitted as value content → argument silently lost (no
recovery; no #49248-style tolerance).
Truncation: a call is finalized only when its closing tag is seen (state
machine in `glm47_moe_detector.py`); a call truncated before that emits no
arguments — the same empty-under-truncation class as vLLM.

## Relevance to our fix

**A client-side format fix CAN cover:**
- Detect `arguments == {}` (or missing required `patch`) on a GLM function_call: do
  not echo the broken call back into history (it poisons retries, per #49248), and
  retry the turn once — the missing-`<arg_value>` quirk is stochastic (~5–17% under
  concurrency), so a fresh turn usually succeeds.
- Detect `finish_reason == "length"` + truncated/invalid arguments → retry with a
  larger `max_tokens` or instruct smaller chunks; on GLM-5.x, prefer `tool_choice:
  "auto"` over forced/named choice, which routes through constrained decoding with
  the open #47504/#55541/#49981 failure cluster.
- The malformed apply_patch format (24 KB patch with raw lines, no `+` prefix): that
  is model content, validated/repaired client-side; no server component is involved.

**A client-side fix CANNOT cover:**
- Once vLLM emits `{}`, the argument bytes are gone at the wire — no client logic can
  recover the 24 KB patch; only retrying (new generation) recovers it.
- The root model quirk (missing opening `<arg_value>` under concurrency) is fixed
  only server-side by vllm-project/vllm#49249 (still open, not in main) — worth
  requesting on the glm-5.2 deployment, plus a logprobs audit to confirm the raw
  tokens in our own failing sessions.
- Server-side schema enforcement (strict=true / xgrammar structural tags) exists on
  recent vLLM but is unsafe for GLM-5.x under forced choice today; it is a
  deployment-side lever, not a client one.

## Sources

- vllm-project/vllm `main` @ `0fefffc93466` (cloned 2026-09-15):
  `vllm/parser/glm47_moe.py`, `vllm/tool_parsers/{__init__,glm47_moe_tool_parser,
  abstract_tool_parser,structural_tag_registry}.py`,
  `vllm/parser/engine/{parser_engine,streaming_parser_engine,adapters}.py`,
  `vllm/parser/abstract_parser.py`, `vllm/tool_parsers/utils.py`, `vllm/envs.py`,
  `rust/src/parser/src/tool/glm_xml/mod.rs`, `rust/src/chat/src/parser/{tool,reasoning}/`.
- vllm-project/vllm tag `v0.22.0`: `vllm/tool_parsers/glm4_moe_tool_parser.py`,
  `vllm/tool_parsers/glm47_moe_tool_parser.py`, `vllm/tool_parsers/__init__.py`.
- Issues: vllm-project/vllm #32829 (closed), #49248 (open, fix #49249 open),
  #47504 (open, fix #47512 open), #49981 (open), #55541 (open).
- sgl-project/sglang `main`: `python/sglang/srt/function_call/glm47_moe_detector.py`,
  `python/sglang/srt/function_call/glm4_moe_detector.py`.
