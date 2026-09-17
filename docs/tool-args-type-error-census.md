# Census: tool-call argument parse failures (string-for-number, hallucinated fields) — qwen + glm

## Status

- Date: 2026-09-17 (UTC) · Bead: `apex-xt2.2` (apex-xt2 epic, codex-combined campaign)
- Data window: ALL of `~/.codex/sessions` (2874 rollout files, 2026-08-16 .. 2026-09-17)
- Verdict: **WORTH A FIX** (details in "Verdict" below). Fix = NEW SDD campaign if greenlit;
  this document is the census + plan only — no product code changed.

## Method (read-only)

1. `~/.codex/bin/codex-triage census` for signature-class totals (catalog
   `~/.cache/codex-triage/signatures.json`).
2. Ad-hoc python census over every rollout JSONL (scripts kept at
   `/tmp/xt22-census/`, re-runnable): every `function_call_output` whose output
   STARTS with `failed to parse function arguments: <serde-error>` (real
   harness errors are short, error-only outputs; the signature also appears
   inside normal tool-output transcripts when models investigate this error
   class — those are excluded, 10 such false positives found).
3. Per instance: tool name, offending param (serde column offset mapped back
   to the nearest JSON key), expected type, model (last `turn_context.model`
   before the event), retry behavior (next same-tool call: args changed?
   succeeded?).
4. Denominator pass: total `function_call` count per model over the same window.

Pinned instance (from the operator report):
`~/.codex/sessions/2026/09/15/rollout-2026-09-15T14-06-49-01a0a640-09d4-71f3-9ded-44d14e79d64b.jsonl`
ordinals 24544/24554 — `exec_command` with `timeout_ms: "120000"` (string),
rejected with `invalid type: string "120000", expected u64`; retry sent the
same shape.

## Findings

Raw signature matches: 39 → **true parse failures: 29** (10 false positives:
signature text inside tool-output transcripts, e.g. sessions triaging this
error class itself).

### Rate per model (true hits / total function_calls)

| model | hits | total function_calls | rate |
|---|---:|---:|---:|
| glm-5.2 | 12 | 5,386 | **0.223%** (1 in ~450) |
| qwen3.8-27b | 13 | 28,497 | **0.046%** (1 in ~2,200) |
| gpt-5.6-sol | 3 | 53,173 | 0.0056% |
| gemini-3-flash-preview | 1 | 1 | n/a (single call) |
| claude-opus-4.6 | 0 | 6,485 | 0% |

glm is ~4.9× the qwen rate and ~40× the frontier rate. The class is
real but low-frequency; its cost is concentrated in the retry/abandon
behavior below.

### Subclasses (29 true hits)

| # | subclass | n | share | detail |
|---|---|---:|---:|---|
| 1 | **numeric-as-string** (`invalid type: string "N", expected u64/usize`) | 16 | 55% | all `exec_command`: `timeout_ms` ×6 (qwen), `max_output_tokens` ×6 (glm), `yield_time_ms` ×4 (glm). Values are pure digit strings: "120000", "2000", "30000", "300000". |
| 2 | **hallucinated field** (`unknown field`, `deny_unknown_fields`) | 6 | 21% | multi-agent v2 tools only: `spawn_agent.agent_name` ×2 (qwen), `wait_agent.target`/`targets` ×3 (qwen ×1, **gpt-5.6-sol ×2**), `followup_task.reasoning_effort` ×1 (**gpt-5.6-sol**). Frontier models hallucinate v2 tool fields too — this is an under-specified-spec problem, not just an open-weight problem. |
| 3 | **missing field / name confusion** | 4 | 14% | `exec_command`: model sent `command` where `cmd` is required ×2, `cd`-style confusion ×1 (all qwen); `spawn_agent` without `message` ×1 (qwen). |
| 4 | **enum drift** (`unknown variant`) | 2 | 7% | `update_plan.status`: glm used `blocked` (beads vocabulary), gemini concatenated `in_progressHeader: Analyzing symbol`. Expected: `pending`/`in_progress`/`completed`. |
| 5 | **truncated JSON** (`EOF while parsing a string`) | 1 | 3% | glm `exec_command.cmd` cut mid-string at column 211. Different species (generation truncation vs. type choice); single instance, no wire capture — track, don't fix here. |

### Retry behavior (next same-tool call after the failure)

| outcome | n | share | meaning |
|---|---:|---:|---|
| no retry | 15 | 52% | model abandoned the tool step — silent workflow degradation (the work never landed via that call) |
| retry succeeded | 11 | 38% | model self-corrected from the error text |
| retry failed | 3 | 10% | same-shape re-send (the pinned instance); turn ends with the step lost |

~88% of failures either permanently lose a step or burn a round-trip. The
`unknown_field` errors self-heal well (serde lists the exact expected field
names — already teachable); the `invalid_type` errors do not name the
offending **param**, which is where the stuck retries cluster.

## Code sites (verified 2026-09-17, all on upstream-shared files)

Divergence note: `reference/upstream-openai` (= `44b9011611`, 2026-09-13) is
the exact merge-base of `feat/normalize-content-types-vllm`, and none of the
files below appear in `git diff reference/upstream-openai..HEAD`. Every fix
site is UNMODIFIED upstream code — each fix is a small additive hunk on a
shared file (low ratchet-conflict risk; to be classified in FORK-MANIFEST.md
per apex-xt2.4).

- Parse funnel: `parse_arguments<T>` in
  `codex-rs/core/src/tools/handlers/mod.rs` — single `serde_json::from_str`
  used by every tool handler; error text `failed to parse function
  arguments: {err}` (no tool/param context — the teachable-error gap).
- Numeric tool-arg fields (complete sweep of Deserialize tool-arg structs,
  non-test):

| struct | file | numeric fields |
|---|---|---|
| `ExecCommandArgs` | `core/src/tools/handlers/unified_exec.rs` | `yield_time_ms: u64`, `timeout_ms: Option<u64>`, `max_output_tokens: Option<usize>` |
| `WriteStdinArgs` | `core/src/tools/handlers/unified_exec/write_stdin.rs` | `yield_time_ms: u64`, `max_output_tokens: Option<usize>` |
| `WaitArgs` (wait_agent) | `core/src/tools/handlers/multi_agents_v2/wait.rs` | `timeout_ms: Option<i64>` |
| `SleepArgs` | `core/src/tools/handlers/sleep.rs` | `duration_ms: u64` |
| code-mode wait handler | `core/src/tools/code_mode/wait_handler.rs` | `yield_time_ms: u64`, `max_tokens: Option<usize>` |

- `SpawnAgentArgs` (`multi_agents_v2/spawn.rs`): `#[serde(deny_unknown_fields)]`,
  fields `message, task_name, agent_type, model, reasoning_effort, fork_turns,
  fork_context`. `SendMessageArgs`/`FollowupTaskArgs`
  (`multi_agents_v2/message_tool.rs`): `target, message`,
  `deny_unknown_fields`. `WaitArgs` has NO `target` field — the
  `unknown field 'target'` hits are cross-tool vocabulary bleed
  (send_message/followup_task use `target`).
- Tool descriptions (spec-level teaching site): `multi_agents_spec.rs`
  (+ per-tool specs in `multi_agents_v2/`), `shell_spec.rs`.

## Verdict: WORTH A FIX

- glm-5.2 is at 1 failure per ~450 tool calls (4.9× qwen, ~40× frontier);
  ~52% of failures are abandoned without retry — each one silently loses a
  workflow step. On an on-prem workhorse that runs long multi-agent sessions,
  that compounds across hundreds of tool steps per session.
- The dominant subclass (55%) is mechanically fixable with zero ambiguity
  (pure-digit string → integer is an unambiguous coercion); the second
  (21%) is fixable at the spec level (the v2 tool descriptions understate
  their exact field set — frontier models trip on it too).
- Cost: small, localized hunks on 5–7 files; no wire-format change;
  OpenAI/Azure behavior unchanged (coercion only widens acceptance).
- Counter-argument considered: rate is <0.25% and 38% self-heal — "wait and
  see" is defensible for the enum-drift (7%) and truncation (3%) subclasses,
  but NOT for the two dominant ones, which are deterministic and cheap.

## Fix plan (NEW SDD campaign if greenlit — spec → multi-review → TDD)

**Design principle (operator ruling, 2026-09-17): model-agnostic ratchets, no
overfitting.** The on-prem model mix may change (qwen is the floor, glm may
drop); every fix below must help regardless of which model runs. No
provider/model gating, no per-model prompt changes, no config/toml changes.
Clarification: the failing `max_output_tokens`/`timeout_ms`/`yield_time_ms`
are PER-CALL TOOL ARGUMENTS of `exec_command`/`write_stdin` (caps on the
tool-result surface), NOT model-level config params — nothing in toml is
involved.

### A. Strict numeric coercion (fixes subclass 1, 55%)
- Add a lenient deserializer (`core/src/tools/handlers/` or a small shared
  helper): accepts a JSON number OR a string of 1+ ASCII digits (nothing
  else — `"1.5"`, `"12abc"`, `""` still error); apply via
  `#[serde(deserialize_with = ...)]` to the numeric fields in the five
  structs listed under Code sites (11 fields total).
- Unambiguous by construction: a pure-digit string in an integer field has
  exactly one reading. No generic/whole-tree coercion (string fields that
  legitimately hold digit strings — e.g. ids — must stay strings).
- Wire impact: none (server-side deserialization only). OpenAI/Azure
  behavior unchanged in practice (they don't send this shape).

### B. Teachable parse errors (fixes the 10% stuck-retry cluster)
- `parse_arguments<T>` currently returns raw serde text with line/column but
  no tool or param context. Add the offending param name (column offset →
  nearest enclosing key, same mapping this census used) and the tool name to
  the `RespondToModel` message, e.g.
  `failed to parse function arguments for exec_command: field
  max_output_tokens: expected usize, got string "2000"`.
- `unknown_field`/`missing_field` already name the exact fields (serde +
  `deny_unknown_fields`) — leave as is; that is why 38% self-heal.

### C. Spec-level field teaching for multi-agent v2 tools (fixes subclass 2,
21%)
- In the v2 tool descriptions (`multi_agents_spec.rs` + per-tool specs):
  state the EXACT field set per tool and the common confusions:
  - `wait_agent` takes ONLY `timeout_ms` — there is no `target` (that field
    belongs to `send_message`/`followup_task`); it waits on the caller's own
    mailbox.
  - `spawn_agent` requires `message` + `task_name`; there is no `agent_name`
    (the spawned agent is addressed by its `task_name`).
  - `followup_task` takes ONLY `target` + `message` (no `reasoning_effort`).
- Same philosophy as the apply_patch P1 (apex-ayl.52): the function-tool
  spec must teach its own vocabulary; model priors are not the contract.

### Out of scope (tracked, not fixed here)
- Enum-drift `update_plan.status` (7%): description already lists the three
  variants; revisit only if the rate grows.
- Truncated-JSON generation (3%, 1 instance): different species; needs wire
  evidence before any fix.
- Generic argument validation layer / schema-driven coercion: rejected —
  more mechanism than the 11 named fields need.

### TDD sketch (for the campaign spec)
- Unit: lenient deserializer accepts `120000` and `"120000"`, rejects
  `"1.5"`/`"12abc"`/`""`/`null`; per-struct round-trips.
- Unit: error message contains tool + param for an invalid_type failure.
- Integration (`test_codex` + `core_test_support::responses`): mount SSE with
  a `function_call` carrying `timeout_ms: "120000"` → assert the call
  executes (no `failed to parse` output); and a `spawn_agent` call with
  `agent_name` → assert the error lists the exact expected fields.
- Gate: `just test -p codex-core` (handlers live there) + `just fmt`/`fix`.
