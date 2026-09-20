# Tool-Args Ratchet Spec (apex-xt2.7)

Status: FINAL v8.2 — rounds 1-10 complete (R1 12M/0B; R2 2B/4M; R3 1B/1M; R4 0B/1M; R5 1B/1M; R6 0B/1M; R7 0B/1M; R8 0B/0M; R9 0B/1M; R10 0B/0M — CONVERGED, both seats). Three optional cosmetic items deferred as implementer notes (round-10 log) — none test-observable, none affecting the operative rules.
Campaign: apex-xt2 (epic) / apex-xt2.7 (this spec)
Date: 2026-09-17
Author: coordinator seat (upstream/codex fork, branch `feat/normalize-content-types-vllm`)

References:
- Census (evidence base, committed): `docs/tool-args-type-error-census.md`
- Divergence audit: `FORK-MANIFEST.md` (repo root)
- Beads: `apex-xt2` epic; closed inputs `apex-xt2.1` (triage workhorse), `apex-xt2.2` (census + verdict)
- Prior research: `docs/vllm-glm-toolcall-research.md`

## 1. Background

The census over `~/.codex/sessions` (2026-09, all models) found **29 true tool-arguments parse
failures** out of ~34k function calls:

| Model | Rate | n/N |
|---|---|---|
| glm-5.2 (vLLM, on-prem) | 0.223% | 12/5386 |
| qwen (vLLM, on-prem) | 0.046% | 13/28497 |
| gpt-5.6-sol | 0.006% | 3/53173 (same classes) |

Failure subclasses (29 instances):

| Subclass | Count | Examples |
|---|---|---|
| A1 numeric-as-string | 16/29 | `timeout_ms: "120000"` (qwen x6), `max_output_tokens` string (glm x6), `yield_time_ms` string (glm x4) |
| A2 hallucinated v2-tool fields | 6/29 | spawn_agent `agent_name` (qwen x2); wait_agent `target`/`targets` x3 (incl. 2x gpt-5.6-sol); followup_task `reasoning_effort` (gpt-sol x1) |
| A3 missing-field / name confusion | 4/29 | `command` instead of required `cmd` |
| A4 update_plan enum drift | 2/29 | out of scope (see Non-goals) |
| A5 truncated JSON | 1/29 | out of scope (see Non-goals) |

Retry behavior after a failure: **52% abandoned the tool, 38% self-healed on retry, 10% stuck**.
The failures cost real work (abandoned steps) and, worse, silent drops on structs without
`deny_unknown_fields` cost correctness.

Both failure clusters trace to the same root cause family as the apply_patch format priors
(apex-ayl.52): open-weights models served via vLLM under-specify tool-call JSON — stringly-typed
numbers and parameters borrowed from adjacent/legacy tool schemas.

## 2. Goals and Non-goals

### Goals

G1. **Ratchet A — strict numeric coercion.** Model-facing tool-args fields typed `u64`/`usize`/
   `i32`/`i64` accept a JSON string **only when it is exactly an integer literal** (optional
   leading `-` for signed types, digits only, no `+`). JSON numbers keep today's exact
   accept/reject classification (error text pinned by snapshot, §3.2). Directly retires
   subclass A1 (16/29 = 55% of all failures).
G2. **Ratchet B — teachable parse errors.** Every model-facing tool-call parse failure inside
   `codex-core` names the **tool** and, when locatable, the **offending JSON parameter**, plus
   the underlying serde error (which already carries expected type + line/column for
   string-parse paths). Truncated-JSON failures get an explicit "resend the complete JSON
   object" hint. Scope and exclusions are pinned in §4.1 (the funnel inventory) and §4.5
   (per-local-funnel rulings): out-of-core copies (`tui`, `ext/goal`) and the apply_patch path
   are named follow-ups, not part of this campaign.
G3. **Ratchet C — v2 spec-level negative field guidance.** Small description additions to the
   multi-agent v2 tool specs that preempt the observed hallucinations. Schema shapes are already
   strict (`additionalProperties: false` + `deny_unknown_fields`) and stay unchanged.
G4. **Ratchet D — fail-loud unknown fields (the D2 class).** Model-facing tool-args structs
   gain `deny_unknown_fields` so unknown parameters become teachable errors instead of silent
   drops (§3.6). This is the A3/`command`-vs-`cmd` half of the census, generalized.

### Non-goals (explicit, this campaign)

- No generic whole-tree JSON coercion (e.g. globally normalizing all args).
- No provider- or model-gated behavior. No config/toml surface. The ratchets apply to every
  model, including frontier ones (census shows gpt-5.6-sol hits the same classes).
- No `update_plan` enum-drift fix (2/29; different mechanism — spec drift).
- No truncated-JSON repair (1/29; the model must resend; B's hint covers the teachable side).
- No changes to `base_instructions` or model-facing system prompts.
- No apply_patch changes (separate campaign, apex-ayl.52 family).
- No coercion of `SearchToolCallParams.limit` (protocol crate, `from_value` parse path in
  `core/src/tools/router.rs:270`, TS/wire surface) — documented ruling §3.1, follow-up
  candidate.
- No changes to `tui/src/dynamic_tools.rs:1150` or `ext/goal/src/tool.rs:415` parse copies —
  different crates, different error types, not on the tool-call wire path; named follow-ups.

### Design principles (operator rulings, 2026-09-17)

P1. **Model-agnostic ratchets.** The on-prem model mix will change (qwen is the floor; glm may
   drop). No ratchet may special-case a model, provider, or serving stack.
P2. **Fail loud over silent.** A parse failure the model can read and repair beats a silent
   misplacement or a silently-dropped parameter.
P3. **Sweep all safe cases** (repo Principle 1): where a fix shape is defined, apply it to every
   struct/field in the same class, not just the ones the census named.
P4. **Resist growing codex-core** (repo AGENTS.md): new shared machinery lands in an existing
   or new non-core crate where natural.

## 3. Ratchet A — strict numeric coercion

### 3.1 Verified target set (post-merge tree, HEAD 7bcd344fa7)

All paths below are `codex-rs/`-relative. Line numbers verified 2026-09-17 on the post-merge
tree. (Round-1 correction: tool handlers live under `core/src/tools/` — earlier notes omitted
the `tools/` segment; round-2 correction: v1 `targets` is at `multi_agents/wait.rs:279`.)

| Struct | Location | Coerced fields (current type) |
|---|---|---|
| `ExecCommandArgs` | `core/src/tools/handlers/unified_exec.rs:28` | `yield_time_ms: u64` (:37), `timeout_ms: Option<u64>` (:39), `max_output_tokens: Option<usize>` (:41) |
| `WriteStdinArgs` | `core/src/tools/handlers/unified_exec/write_stdin.rs:23` | `session_id: i32` (:25), `yield_time_ms: u64` (:29), `max_output_tokens: Option<usize>` (:31) |
| `SleepArgs` | `core/src/tools/handlers/sleep.rs:34` | `duration_ms: u64` (:35) |
| `ExecWaitArgs` (code-mode `wait`) | `core/src/tools/code_mode/wait_handler.rs:26` | `yield_time_ms: u64` (:29), `max_tokens: Option<usize>` (:31) |
| `WaitArgs` (multi-agent **v2**) | `core/src/tools/handlers/multi_agents_v2/wait.rs:128` | `timeout_ms: Option<i64>` (:129) |
| `WaitArgs` (multi-agent **v1**) | `core/src/tools/handlers/multi_agents/wait.rs:277` | `timeout_ms: Option<i64>` (:280) — D3: include |
| `BarrierArgs` (test_sync) | `core/src/tools/handlers/test_sync.rs:36` | `participants: usize` (:38), `timeout_ms: u64` (:40) — added in v2 (round-1 Major A3) |
| `TestSyncArgs` (test_sync) | `core/src/tools/handlers/test_sync.rs:44` | `sleep_before_ms: Option<u64>` (:46), `sleep_after_ms: Option<u64>` (:48) — added in v2 |

**Sweep completeness ruling (round-1 Major A3, seat A exhaustive enumeration):** all other
model-facing tool-args structs under `core/src/tools/` were checked and are numeric-free: v1
SpawnAgent/CloseAgent/ResumeAgent/SendInput Args, v2 SpawnAgent/InterruptAgent/ListAgents Args,
SendMessage/FollowupTask Args, ViewImageArgs, WaitForEnvironmentArgs, ExecCommandEnvironmentArgs,
ListResource/ReadResourceArgs, RequestPermissionsEnvironmentArgs, RecommendedPluginInstallArgs,
RequestUserInput Tool/Async Args, SendMessageToUserAsyncArgs, UpdatePlanArgs/PlanItemArg.
`SpawnAgentArgs` has no numeric fields (v2 `fork_turns` is intentionally a string — multi_agents_v2/spawn.rs:271; v1 has `fork_context: bool`, multi_agents/spawn.rs:224-231 — round-8 F3 cite fix).

**Excluded in-class field (ruling, round-1 Minor A5):** `SearchToolCallParams.limit:
Option<usize>` (`protocol/src/models.rs:2073-2077`) — lives in `codex-protocol` (TS/wire
surface with `JsonSchema`/`TS` derives), parsed via `serde_json::from_value` in
`core/src/tools/router.rs:270` (not the `from_str` funnel), and is a client-side tool_search
path, not the model tool-call hot path. Coercing it would touch the protocol crate and its wire
contract for a path with zero census hits. **Follow-up candidate**, not this campaign.

The v1 `wait_agent` schema exposes `targets: Vec<String>` (`multi_agents/wait.rs:279`) while the
v2 schema exposes **only** `timeout_ms` (`multi_agents_spec.rs:864-876`). This v1/v2 asymmetry
is the plausible source of the `target`/`targets` hallucinations in A2 — the v2 descriptions
must disambiguate (Ratchet C, §5.2).

### 3.2 Helper design (v2 restatement — visitor-based; round-1 Major A2)

Round-1 finding: a `deserialize_with` helper receives the field's deserializer and may call
**one** entry point on it; it cannot peak the token type and then delegate numbers to
`T::deserialize` (no public re-feed API for a buffered token —
`serde::__private::de::ContentRefDeserializer` is private). The v1 "delegate to
`T::deserialize`" design was not implementable. Restated:

**Mechanism:** for each concrete type T ∈ {`u64`, `usize`, `i32`, `i64`}, a pair of helpers
generated by one macro:

```rust
// codex-rs/tools/src/strict_int.rs (new module)
// macro strict_int_type!(u64)  =>  pub fn strict_u64<D: Deserializer>(D) -> Result<u64, D::Error>
//                                   + pub fn strict_u64_opt<D: Deserializer>(D) -> Result<Option<u64>, D::Error>
// (same for usize, i32, i64 — 8 functions, 1 macro)

fn strict_T(deserializer: D) -> Result<T, D::Error> {
    deserializer.deserialize_any(StrictIntVisitor::<T>::default())
}
```

**Visitor** (per concrete T), mirroring serde's primitive integer visitors so the number path
keeps today's exact accept/reject classification:

| Visit | Behavior |
|---|---|
| `visit_u64` / `visit_i64` | Same range acceptance as serde's default impl for T: u64 — `visit_u64` ok, `visit_i64` ok iff `>= 0`; usize — `u64::try_into` range check; i32 — range-checked both directions; i64 — `visit_i64` ok, `visit_u64` ok iff `<= i64::MAX`. Out-of-range → `de.custom(...)` with the same classification as today's serde error. |
| `visit_f64` | Reject — `de.custom("invalid type: floating point ...")` (float JSON is rejected for integer targets today; serde_json calls `visit_f64` for `1.0`). |
| `visit_bool`, `visit_bytes`, `visit_unit`, `visit_seq`, `visit_map`, `visit_none` | Reject with `de.custom(...)` (today's classification: wrong type). |
| `visit_str(s)` | Accept iff `s` is exactly an integer literal: `s.starts_with('+')` → reject (round-1 Major B4: Rust `FromStr` **accepts** a leading `+`, so the check is explicit — JSON never emits `+`; the contract is "optional single `-`, digits only"); otherwise `s.parse::<T>()` mapped to `de.custom("expected an integer (JSON number, or a string containing only an integer literal)")` on failure. `FromStr` then rejects `""`, `"1.5"`, `"12abc"`, `" 12"`, `"0x10"`, and `-5` on unsigned types; accepts `"0123"` (base 10 → 123) and `"-5"` on signed types. |

The `_opt` variant: `visit_unit`/`visit_none` → `Ok(None)`, everything else delegates to the
plain visitor's result wrapped in `Some`.

**Guarantee (relaxed per round-1 Major A2):** identical **accept/reject behavior** for every
input against current behavior; **number-path error text is pinned by snapshot tests** (one
representative per type in §7.1) rather than claimed byte-for-byte — the visitor reproduces
serde's classification, and any drift in exact wording is caught by the pin.

**Contract ruling (round-1 Major B4):** `"+12"` is **rejected** (explicit pre-check), not
accepted via `FromStr`. Rationale: JSON number syntax has no `+`; accepting it would be a
strict widening beyond "integer literal as a model would write it", and the v1 spec's own test
plan required rejection.

Field annotations (coexist with `serde(default)` as today — round-1 verified):

```rust
#[serde(default = "default_exec_yield_time_ms", deserialize_with = "strict_u64")]
yield_time_ms: u64,

#[serde(default, deserialize_with = "strict_u64_opt")]
timeout_ms: Option<u64>,
```

### 3.3 Placement (decision item D1 — confirmed round 1)

New module `codex-rs/tools/src/strict_int.rs`, exported from `codex-tools::lib`. `codex-tools`
already has `serde` + `serde_json` deps (`tools/Cargo.toml:28-29`) — no lockfile change. Keeps
the machinery out of codex-core (P4). `deserialize_with` is established house style:
`config/src/config_toml.rs:310`, `login/src/device_code_auth.rs:32`,
`login/src/token_data.rs:14`, `codex-api/src/sse/responses.rs:182`. BUILD.bazel: both
`codex-tools` and `codex-core` glob sources (`codex_rust_crate` default `src/**/*.rs`,
`defs.bzl:311` — round-9 F3 cite fix, the earlier :188 pointed at the def signature,
doc at :232; `core/BUILD.bazel` explicit glob) — **no Bazel changes needed**.

### 3.4 Edge cases

- Overflow (`"99999999999999999999"`) → `FromStr` err → custom error (no panic, no wrap).
- `null` on non-Option fields: unchanged (fails as today). `null` on `Option` fields:
  unchanged (`None`).
- JSON string `"null"`: rejected (not an integer literal) — fail loud, not silent `None` (P2).
- `"0123"` accepted → 123. Risk assessment (round-1 NOTE B5): base-10 `FromStr`, no octal;
  for duration/token/session-id fields no plausible alternate intent exists; worst case a
  model typo lands as a valid in-domain value instead of a teachable error — P2-tolerable.
- Number path: identical accept/reject as today; error text pinned by snapshot (§7.1).

### 3.5 Downstream raw-args consumer: rollout-trace (round-1 Major B3)

`rollout-trace/src/reducer/tool/terminal.rs` re-parses **raw model arguments** for trace
reduction: `DispatchedWriteStdinArgs` (:580) with `yield_time_ms: Option<u64>` (:584) and
`max_output_tokens: Option<usize>` (:585), parsed at :408 with plain serde; a parse failure
bails reduction (`terminal.rs:60` → `reducer/tool.rs:108` `?`).

Before Ratchet A, string-int write_stdin payloads never dispatched (core parse failed first),
so the reducer never saw them. After Ratchet A they dispatch successfully and their recorded
raw args would **break trace reduction — for exactly the A1 payloads this campaign saves**.

**Fix (in scope, decision item D5):** make those two fields tolerant in `terminal.rs` with a
small **local** mirror of the strict contract (`Option<JsonValue>` + ~10-line local parse
accepting a JSON number or an integer-literal string; failure → `None`, reduction proceeds).
Local rather than a `codex-tools` dependency: avoids a `Cargo.toml`/`Cargo.lock` change (no
`just bazel-lock-update`) and keeps the trace crate self-contained; the ~10 lines of logic
duplicated are acceptable (single consumer, small contract). `session_id` stays `JsonValue`
(already tolerant).

**Sweep evidence (round-2):** exhaustive grep of `rollout-trace/src` for typed raw-args
re-parses found exactly one other site — `reducer/tool/agents.rs:305`
(`AgentMessageInvocationArgs { message: String }`, gated to spawn/send_message/followup_task)
— which is string-only (no numeric fields) and provably unaffected by Ratchet A. All other
`from_str`/`from_value` sites parse `Value`-typed runtime payloads (codex-generated, correctly
typed). `DispatchedWriteStdinArgs` is the only affected site; D5 covers it.

### 3.6 Ratchet D — `deny_unknown_fields` sweep (the D2 class, round-1 Majors A4/B5)

Round-1 finding: D2's original three-struct scope contradicted its own rationale
("consistency with every sibling struct") and P3 — ≥15 more model-facing tool-args structs
silently drop unknown fields. Round-2 finding (Blocking N1/N2, seats C+D): the class is
bounded by a structural invariant the v1/v2 sweeps missed —

> **Invariant (round-2):** for any tool, *every typed deserialization of the raw model-args
> string* must recognize the **full tool-schema field set**. A struct parsed from the full raw
> args is a *projection*: putting `deny_unknown_fields` on a projection rejects every
> schema-legal field it does not declare.

Verified violation: `ExecCommandEnvironmentArgs` (unified_exec.rs:53, fields
`environment_id`+`workdir` only) is parsed from the **full** exec_command args at
`unified_exec/exec_command.rs:186` — *before* the main `ExecCommandArgs` parse (:236/:241).
The exec_command schema exposes `workdir` (shell_spec.rs:41) and `environment_id`
(shell_spec.rs:82-84; round-4: the `:170` citation dropped — that is the
request_permissions schema's `environment_id`, not exec_command's), and existing integration
tests send `workdir` in exec_command
args and assert success (`core/tests/suite/unified_exec.rs:845, :908`;
`core/tests/suite/approvals.rs:2950, :2957`; `core/tests/suite/request_permissions.rs:777`).
`deny_unknown_fields` on the shadow struct kills every legitimate exec_command call with
`unknown field 'cmd'`; on `ExecCommandArgs` (which lacks `workdir`/`environment_id`) it kills
every `workdir`-bearing call. Same shape on the request_permissions side
(`RequestPermissionsEnvironmentArgs` parsed at request_permissions.rs:70 from full args whose
schema requires `permissions`).

**Round-3 refinement (Blocking NEW-1, seat E) — the v3 "single-struct" shape is refuted.**
The two parse passes differ in **guard state**, not just struct: the second parse runs under
`AbsolutePathBufGuard::new(base_path)` (`mod.rs:149-158`), a thread-local that changes
*value-level* deserialization — a relative `AbsolutePathBuf` with no base hard-errors
(`utils/absolute-path/src/lib.rs:343-348`). Moving full-struct deserialization to the
pre-guard first parse therefore flips **accept→reject** for relative paths in
`additional_permissions` (legacy and canonical forms), breaking
`core/tests/suite/request_permissions.rs:728`
(`relative_additional_permissions_resolve_against_tool_workdir` — sends
`workdir:"nested"` + `additional_permissions:{file_system:{write:["."]}}` and asserts
success with the nested resolution), a test §7.5 itself names as a guard. The projection
invariant above was field-set-based only; it now also covers value semantics:

> **Invariant, value clause (round-3; round-6 tightening, seat K):** no struct parsed in
> the *pre-guard pass* (the shadow parse before the main parse) may declare a field whose
> deserialization depends on the guard (`AbsolutePathBuf`-bearing fields such as
> `additional_permissions`); the pre-guard pass must remain a projection. (The main
> parse's unguarded `None` branch — foreign-executor cwd, exec_command.rs:240-241 — is
> pre-existing behavior, not constrained by this clause; the operative regression guard is
> the `:728` test.)

**Resolution (v4) — fold + keep the shadow; deny on the full struct only:**
1. Add `workdir: Option<String>` + `environment_id: Option<String>` to `ExecCommandArgs`
   (both `#[serde(default)]`, kept raw exactly as the shadow keeps them). The handler reads
   the environment values from the **shadow parse** (as today) — the folded fields exist so
   `deny_unknown_fields` recognizes the schema-legal fields, so they are annotated
   `#[allow(dead_code)]` with a doc comment stating exactly that (workspace lints
   `rust = {}`: default warn; the annotation keeps the tree warning-clean).
2. `ExecCommandEnvironmentArgs` and its parse at exec_command.rs:186 are **unchanged**
   (no attr — it stays a projection per the invariant).
3. `deny_unknown_fields` lands on `ExecCommandArgs` only.

**Error ordering: UNCHANGED (round-3 correction of the v3 claim).** A malformed non-env field
(e.g. `cmd: 123`) still passes the shadow (unknown-to-it) and fails at the second parse,
exactly as today; a malformed env field still fails at the first parse, exactly as today.
The v3 "error ordering shifts slightly" sentence is retracted.

**Resolution — request_permissions: documented exclusion (round-2).** Its stage-2 struct
`RequestPermissionsArgs` lives in `codex-protocol` (`protocol/src/request_permissions.rs:50`,
TS/`JsonSchema` wire surface, parsed via `from_value` at request_permissions.rs:89-93). A
single-struct restructure there requires a protocol-crate change → **named follow-up (D6
class)**; the shadow `RequestPermissionsEnvironmentArgs` stays tolerant (no attr) so
request_permissions' unknown-field behavior is unchanged this campaign (documented residual).
Round-3 addition (seat E): even a core-local full struct would **regress documented
behavior** — the tool description (shell_spec.rs:194) promises "Relative filesystem paths
resolve against the selected environment cwd", implemented by the Value-level rewrite
`resolve_permission_path_strings` (call site request_permissions.rs:88; helper
:135-186, singular-path variant :188-213 — round-4 citation correction), which must run
**before** the stage-2 `from_value` deserialization. The exclusion therefore rests on two
independent reasons: (a) protocol-crate wire surface, (b) pre-deserialization path rewrite.

Full D2 class (v4 — 15 rows / 16 structs, all `#[serde(deny_unknown_fields)]` added):

| # | Struct | Location |
|---|---|---|
| 1 | `ExecCommandArgs` | `handlers/unified_exec.rs:28` — **with** the §3.6 fold (gains `environment_id`/`workdir`, `#[allow(dead_code)]`-annotated; shadow parse unchanged) |
| 2 | `WriteStdinArgs` | `handlers/unified_exec/write_stdin.rs:23` |
| 3 | `ExecWaitArgs` | `code_mode/wait_handler.rs:26` |
| 4 | `WaitArgs` v1 | `handlers/multi_agents/wait.rs:277` (round-1 Major B5 — the v1/v2 drift source) |
| 5 | `SpawnAgentArgs` v1 | `handlers/multi_agents/spawn.rs:224` |
| 6 | `CloseAgentArgs` | `handlers/multi_agents/close_agent.rs:165` |
| 7 | `ResumeAgentArgs` | `handlers/multi_agents/resume_agent.rs:165` |
| 8 | `SendInputArgs` v1 | `handlers/multi_agents/send_input.rs:144` |
| 9 | `ViewImageArgs` | `handlers/view_image.rs:59` |
| 10 | `TestSyncArgs` + `BarrierArgs` | `handlers/test_sync.rs:44,36` |
| 11 | `RequestPluginInstallArgs` | **`tools/src/request_plugin_install.rs:16`** (codex-tools crate — round-2 Major N5; single model-path consumer, grep-verified, no wire surface) |
| 12 | `RecommendedPluginInstallArgs` | `handlers/request_plugin_install.rs:49` — has `alias` (`tool_id`) |
| 13 | `ListResourceArgs` | `handlers/mcp_resource.rs:63` |
| 14 | `ReadResourceArgs` | `handlers/mcp_resource.rs:100` |
| 15 | `RequestUserInputToolArgs` | `handlers/request_user_input_spec.rs:12` |

**Excluded with documented rulings (v4):**
- `ExecCommandEnvironmentArgs` (unified_exec.rs:53) — **retained** (v4 correction of the
  refuted v3 deletion — round-3 Blocking NEW-1): the shadow struct and its parse at
  exec_command.rs:186 are unchanged; it stays a tolerant projection per the invariant +
  value clause and is excluded from the D2 sweep for that reason (`deny_unknown_fields`
  lands on the full `ExecCommandArgs` only, which now declares `workdir`/`environment_id`).
- `RequestPermissionsEnvironmentArgs` (request_permissions.rs:25) — stays tolerant: its
  stage-2 partner is a protocol-crate wire struct (follow-up D6 class); adding the attr
  alone is impossible (projection, invariant) and harmless-only exclusion is the ruling.
- `RequestPermissionsArgs` (`protocol/src/request_permissions.rs:50`) — protocol-crate wire
  surface; named follow-up (D6 class).
- `SearchToolCallParams` (`protocol/src/models.rs:2073`) — D6 (coercion) + this class:
  protocol-crate wire surface; named follow-up.

Class-completeness check (round-2 seat D): a full census of the 27 tool-args structs in
`core/src/tools` found exactly 17 without the attr = the 15 rows above (row 10 bundles two)
plus the two projection/exclusion rulings; the class is complete **within core +
codex-tools** (row 11 is the codex-tools struct — outside this core census; 15 of the 16
table structs are core — round-9 NOTE 1).

**Alias ruling:** `deny_unknown_fields` coexists with `serde(alias)` (aliases count as known
fields). TDD must prove it: one test per alias-bearing struct (**#12 only** — round-4
correction: #13 `ListResourceArgs` (mcp_resource.rs:63) has no alias; a repo-wide sweep of
the D2 class found exactly one alias, `RecommendedPluginInstallArgs.plugin_id` → `tool_id`
at request_plugin_install.rs:50) asserting the alias form still parses AND an unknown
field is rejected. If a struct breaks a legitimate usage in
testing, that **one** struct is reverted with a documented note (per-struct rollback, not
campaign rollback). (Round-2 verified: serde's derive visitor treats aliases as known field
names — the ruling is sound; the tests prove it in-tree.)

**Reliance check (round-1, clean):** `unified_exec_tests.rs` payloads use known fields only;
`core/src/memory_usage.rs:46-48` does its own `serde_json::from_str::<ExecCommandArgs>(arguments)
.ok()` (best-effort `cmd` extraction for memory estimation — see §3.7); the exec_command hook
rewrite path (`unified_exec/exec_command.rs:535-554`) only replaces `cmd` and cannot introduce
unknown fields; rollout-trace uses its own `DispatchedWriteStdinArgs` (unaffected by the core
attr). **Round-2 addition:** the existing full-payload integration tests
(`suite/unified_exec.rs:845, :908`; `suite/approvals.rs:2950, :2957`;
`suite/request_permissions.rs:777`, a payload line of the same test as `:728` — round-4
note) are the standing regression guard for the restructure — they send schema-legal
`workdir`/full payloads and assert success, and they must keep passing after row 1 lands.
**Round-4 addition (seat G; round-5 range correction):** `pre_tool_use_payload`
(exec_command.rs:522-533, parse at :527) is a third best-effort `from_str::<ExecCommandArgs>(arguments).ok()` consumer OUTSIDE
the base-path guard — fold impact is neutral-to-positive (legitimate calls parse as before;
unknown-field payloads degrade to no hook payload, consistent with the call itself failing
under D). It nominally touches the value-clause class in the benign `.ok()`-degradation
sense; disclosed here and in §3.7 so a future reviewer does not mistake it for a NEW-1
regression.

### 3.7 Best-effort `.ok()` `ExecCommandArgs` consumers (round-1 NOTE B2; round-5 extension)

Two consumers deserialize `ExecCommandArgs` with `.ok()` best-effort; both keep the same
degradation kind under the campaign (no change to either):
- `core/src/memory_usage.rs:46-48` (import at :4) — extracts `cmd` for memory estimation. Ratchet A is
  **automatically inherited** (a string `timeout_ms` no longer defeats `cmd` extraction —
  a plus). Under Ratchet D, struct-unknown payloads fall back to `None` — still
  best-effort-safe.
- `pre_tool_use_payload` (exec_command.rs:522-533, parse at :527) — pre-tool-use hook
  payload from raw model args, outside the base-path guard. Under D, struct-unknown
  payloads → no hook payload (consistent with the call itself failing); folded fields
  parse.

## 4. Ratchet B — teachable parse errors

### 4.1 The funnel and the exact inventory (v2; round-1 Majors A1/B1/B6)

The shared funnel:

```rust
// core/src/tools/handlers/mod.rs:86
pub(crate) fn parse_arguments<T>(arguments: &str) -> Result<T, FunctionCallError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(arguments).map_err(|err| {
        FunctionCallError::RespondToModel(format!("failed to parse function arguments: {err}"))
    })
}
```

**Verified inventory: 30 production call sites** (round-1 correction of "~25"; seat A list,
re-counted 2026-09-17 = 34 non-test mentions incl. 2 internal uses in `mod.rs` and the
wrapper — see below):

- `mod.rs:124` (rewrite_function_arguments), `mod.rs:157` (parse_arguments_with_base_path)
- `exec_command.rs:186, :236, :241, :527`, `write_stdin.rs:81`
- `sleep.rs:95`, `view_image.rs:132`, `wait_for_environment.rs:125`, `test_sync.rs:100`
- `request_permissions.rs:70, :87` (stage 1), `request_plugin_install.rs:125, :135`
- `request_user_input.rs:80` (round-1 omission — now included),
  `request_user_input_async.rs:94`, `send_message_to_user_async.rs:76`
- multi_agents v1: `spawn.rs:62`, `send_input.rs:46`, `close_agent.rs:43`, `resume_agent.rs:47`,
  `wait.rs:67`
- multi_agents v2: `spawn.rs:119`, `send_message.rs:40`, `followup_task.rs:40`,
  `interrupt_agent.rs:45`, `list_agents.rs:43`, `wait.rs:52`
- `dynamic.rs:138`

**+ 6 test sites** (`unified_exec_tests.rs:53, 75, 113, 141, 162, 221`) = 36 total.

**Plus one wrapper** that must thread the tool name: `parse_arguments_with_base_path`
(`mod.rs:149-158`, called from `exec_command.rs:236`) — round-1 Minor A4/B1.

**Not shared-funnel consumers (round-1 corrections — v1 spec misclassified these):**

- mcp_resource ×3 sub-handlers (`list_mcp_resources.rs:65`, `read_mcp_resource.rs:68`,
  `list_mcp_resource_templates.rs:65`) call a **local** `parse_arguments` at
  `mcp_resource.rs:373` (semantics: `Option<Value>`, empty-string→`None`, null→`None`), plus
  two local helpers: `parse_args` at `mcp_resource.rs:388` (called at `mcp_resource.rs:407`
  and `read_mcp_resource.rs:69`) and — round-7 seat N-5 — `parse_args_with_default` at
  `mcp_resource.rs:402` (calls `parse_args` at :407; its two call sites
  `list_mcp_resources.rs:66` / `list_mcp_resource_templates.rs:66` gain the `tool_name`
  argument mechanically).
- code_mode wait (`wait_handler.rs:110`) calls a **local byte-identical duplicate** at
  `wait_handler.rs:40` — deleted in this campaign, switched to the shared funnel.
- plan.rs (`parse_update_plan_arguments`, `handlers/plan.rs:108-111`) — pre-campaign
  **local inline parse** (absent from the 30-site list); the section 4.5 update_plan
  ruling converts it to the shared funnel at item-3 implementation, making it the 32nd
  production shared-funnel site (post-landing note, item 3: seat A-m-1).

### 4.2 New signature and message shapes

```rust
pub(crate) fn parse_arguments<T>(tool_name: &str, arguments: &str) -> Result<T, FunctionCallError>
```

Every call site passes the tool's public name. Implementation note (round-3 corrected set,
round-7 M-1 annotation): three funnel consumers define a `TOOL_NAME` constant —
`sleep.rs:27`, `send_message_to_user_async.rs:21`,
`request_user_input_async.rs:22` — use the constant there; `current_time.rs:25` defines a
`TOOL_NAME` constant but is **not** a funnel consumer (no `parse_arguments` call; no change
needed in that file); elsewhere pass the same literal as the
handler's `ToolName::plain("...")` (e.g. `multi_agents_v2/wait.rs:24`,
`unified_exec/exec_command.rs:114`, `unified_exec/write_stdin.rs:38`,
`multi_agents_v2/followup_task.rs:13`). The re-export path
`crate::tools::handlers::parse_arguments` is unchanged.

Message shapes (the raw serde error is always appended verbatim — it carries expected type and
line/column for string-parse paths):

Ordering (round-9 NOTE 2): the shape-2 `Eof` classification happens **before** parameter
extraction — shape 2 never carries a `(parameter …)` suffix, and `nearest_json_key_before`
is only consulted for non-`Eof` errors.

1. **Type/structure errors with a locatable parameter:**
   `failed to parse arguments for {tool}: {serde_err} (parameter "{key}")`
2. **Truncated input** (`serde_json` error category `Eof`):
   `failed to parse arguments for {tool}: {serde_err} — arguments appear truncated; resend the complete JSON object`
3. **Everything else:** `failed to parse arguments for {tool}: {serde_err}`

Census examples under the **v2 scope** (D landed):

- `exec_command` + `{"cmd":"true","timeout_ms":"120000"}` → **no error** (Ratchet A coerces).
  Pre-campaign it was: `failed to parse function arguments: invalid type: string "120000",
  expected u64 at line 1 column 426` — unattributable; 52% of models abandoned after it.
- `wait_agent` (v2) + `{"target":"task_1"}` →
  `failed to parse arguments for wait_agent: unknown field `target`, expected `timeout_ms` (parameter "target")`
  (`deny_unknown_fields` already names the field; B adds attribution; C reduces frequency).
- `exec_command` + `{"command":"true"}` (the A3 census case) → under Ratchet D, serde rejects
  the unknown field **as it is encountered, before the missing-field check** (round-1 Minor A3),
  so: `failed to parse arguments for exec_command: unknown field `command`, expected one of `cmd`, `shell`, `login`, ... (parameter "command")`.
  (Without D it would be `missing field `cmd`` + hint — the D variant is the expected final
  behavior; tests assert the D variant.)

### 4.3 Parameter extraction (best-effort hint)

`nearest_json_key_before(json: &str, line: usize, column: usize) -> Option<String>`
(post-landing erratum, item 3: implemented with `usize` — `serde_json::Error::line()`/
`column()` return `usize`; the round-1 `u64` annotation was descriptive):

1. Convert serde's 1-based (line, column) to a byte offset. **Pinned by probe (2026-09-17)
   and re-verified against vendored serde_json 1.0.149 (round-1 NOTE A2):** `column()` is a
   1-based **byte** offset within the line (`iter.rs:53-65` — `col += 1` per byte; 1-based
   `col()` semantics at `error.rs:38-46`); for a
   type error on a string value the position is reported **after the offending string's
   closing quote** (string consumed before the visitor error, `de.rs:316/441`). Multi-line:
   map `line` to the line start by counting newlines, then add `column - 1`.
   **Asymmetry note (post-landing erratum, item 3, seat B-N1):** error positions are not
   uniformly off-by-one in vendored serde_json 1.0.149 — the `fix_position`/`position()`
   class (e.g. `["x"]`→u32 `invalid type`) can report column 0 (the byte *before* the
   consumed byte), while direct `peek_error` calls (e.g. `1x` → `expected `,` or `}`)
   report the peeked byte's own 1-based position; the backward key-scan with its
   inclusive `offset+1` boundary absorbs both.
2. Scan backward from that offset for the last `"..."` string token followed, after optional
   whitespace, by `:` at or before the offset **plus one** — the boundary is 0-based
   `offset+1`, **inclusive** (round-8 P-1 correction; round-9 F1 rewording: the earlier
   "0-based `column` exclusive" parenthetical contradicted the qualifying example it
   accompanied and is void): a key whose closing quote ends at the reported offset and
   whose `:` sits one past it (at the 1-based `column` value, 0-based `offset+1`)
   qualifies. That is
   exactly serde's unknown-field error position (the unknown-field error fixes at the colon
   — e.g. `{"target":"task_1"}` → line 1 column 9, the colon; vendor serde_json
   `de.rs:441` / `read.rs:421-429`), and it covers the §4.2 `wait_agent` pinned example.
   Return the key text if found, else `None` (shape 3). Only `:`-terminated object keys
   qualify, and only keys *before* the error position are considered.
3. Best-effort by contract: a wrong-but-adjacent hint is acceptable; a missing hint is
   preferred over a confidently wrong one.

Placement: new sibling module `core/src/tools/handlers/args_parse.rs` holding
`parse_arguments` (moved; re-exported from `mod.rs` to keep import paths) +
`nearest_json_key_before` — keeps `mod.rs` (617L) from growing (repo 500/800 LoC guidance).

### 4.4 Excluded paths (explicit, round-1 Notes A1/B1)

- `tui/src/dynamic_tools.rs:1150` — TUI-local `parse_arguments<T>(Value) -> Result<T, String>`
  (`from_value`, "Invalid tool arguments"); TUI dynamic tools only; different error type.
  **Named follow-up.**
- `ext/goal/src/tool.rs:415` — private copy in the goal extension crate. **Named follow-up.**
- `core/src/tools/handlers/apply_patch.rs:149` (streaming patch-parser error) and :529/:599
  (args→`Value`) — separate campaign (apex-ayl.52 family), per Non-goals. (Round-2 citation
  fix: the earlier `core/src/apply_patch.rs:149` pointed at a 97-line file with no parse code.)
- `mcp_resource` local fns: in scope via §4.5, but their `Option<Value>`/empty→`None`/
  null→`None` semantics are **preserved verbatim** — only attribution is added.

### 4.5 Per-local-funnel rulings (v2; resolves round-1 Majors A1/B2)

| Local funnel | Location | Ruling |
|---|---|---|
| code_mode wait duplicate | `code_mode/wait_handler.rs:40` | **Delete**; call site :110 switches to the shared funnel (keep the `inspect_err` instrumentation) |
| mcp_resource `parse_arguments` | `mcp_resource.rs:373` (3 sub-handler call sites) | **Extend**: gains `tool_name` param (`list_mcp_resources` / `read_mcp_resource` / `list_mcp_resource_templates`); `Option<Value>` semantics unchanged |
| mcp_resource `parse_args` | `mcp_resource.rs:388` (called :407, read_mcp_resource.rs:69) | **Extend**: same treatment |
| mcp_resource `parse_args_with_default` | `mcp_resource.rs:402` (calls `parse_args` at :407; call sites `list_mcp_resources.rs:66`, `list_mcp_resource_templates.rs:66`) | **Extend**: signature forced by the `parse_args` change (round-7 seat N-5) |
| `update_plan` | `handlers/plan.rs:108-111` (`parse_update_plan_arguments`) | **Extend**: tool name + shared message shapes. **Lockstep test updates:** `core/tests/suite/tool_harness.rs:309-314` and `core/tests/suite/direct_tool_metadata.rs:285-289` pin the old message (round-1 NOTE B3) |
| request_permissions stage 2 | `request_permissions.rs:91-93` (`serde_json::from_value` into `RequestPermissionsArgs` after path rewrite) | **Extend**: tool name only (from_value errors carry no line/column; no parameter hint) |
| tool_search | `core/src/tools/router.rs:270-277` | **No change** — already teachable: message is `failed to parse tool_search arguments: {err}` (tool named). Verified, documented, no action |

Resulting mixed formats are now explicit (round-1 Major B2; round-8 F2 count update): the
shared funnel, the five extended local funnels, and (unchanged) tool_search all name the
tool; the `from_value`
variant has no column hint. Out-of-core copies stay on their old format until their named
follow-ups.

## 5. Ratchet C — v2 spec-level negative field guidance

### 5.1 Current strictness (verified round 1)

`core/src/tools/handlers/multi_agents_spec.rs` already emits strict schemas for all v2 tools:
`JsonSchema::object(..., required, Some(false.into()))` i.e. `additionalProperties: false`
(mechanism verified at `tools/src/json_schema/types.rs:151-170`) —
`create_spawn_agent_tool_v2` (:100, required `[task_name, message]`),
`create_send_message_tool`, `create_followup_task_tool` (:213, required `[target, message]`),
`create_wait_agent_tool_v2` (:280, **only** `timeout_ms` property, :864-876). Combined with
`deny_unknown_fields` on the v2 arg structs, hallucinated fields already **fail** — C reduces
their frequency by teaching the absence in the descriptions the model reads.

The v1 `wait_agent` schema exposes `targets` (v1 `WaitArgs`, `multi_agents/wait.rs:277-280`;
schema builder `multi_agents_spec.rs:836-860`). The v1/v2 parameter drift is the most plausible
origin of the `target`/`targets` hallucinations (3/29, incl. 2x gpt-5.6-sol — the drift, not
the serving stack, is the cause).

**Implementation notes (round-1 NOTE B4):** the v2 spawn description is a `format!` template
(:737-757) — the negative sentence splices into the raw string, not the template args.
`multi_agents_spec.rs` is 878 LoC — already past the ~800 LoC module guidance; the edits must
not grow the file further (one-liners only, inside existing description owners). No `.snap`
file in `codex-rs` captures any of these descriptions as literal text or the old error message
(repo-wide string sweep, verified round 1) — C and B break no other insta snapshots.
(Post-landing erratum, item 6: that round-1 string sweep missed a hash-derived pin — the astra
scenario snapshot `all__suite__scenarios__astra_async_question_and_answer.snap` pins the
collaboration-namespace content hash derived from these descriptions rather than copying
them, and legitimately moved with the item-5 edit (9ffeb7db7d; namespace
`43f195d9cc99ee15`→`4fffb2a2afa835c6`, derived parent fragment `d4f3e164a92d9bf1`→
`f80a5970748ce3ae`); it was refreshed in the item-6 closure commit that carries this erratum
and the snapshot refresh — exact commit sha in the apex-xt2.7 bead closure note.)

### 5.2 Description edits (exact, additive, one line each)

1. **wait_agent v2** (multi_agents_spec.rs:282-285). Current description ends with
   `"...or a timeout summary if no activity arrives before the deadline."`
   Append: ` It takes no target: the wait is over all live agents. To address a specific agent
   use send_message or followup_task with its task name.`
2. **spawn_agent v2** (multi_agents_spec.rs:737-757). The description already teaches
   task_name addressing; append one negative after the task_name sentence:
   ` There is no agent_name parameter; the agent is addressed by task_name.`
3. **followup_task** (multi_agents_spec.rs:213-241). Append:
   ` It cannot change the target's model or reasoning settings; spawn a new agent for those.`
4. **wait_agent v1 (D4)** (multi_agents_spec.rs:264-270, `create_wait_agent_tool_v1`;
   exact text pinned round-7, seat N-8). The current v1 description ends with
   `"...a notification message will be received containing the same completed status."`
   Append: ` It takes targets: the agent ids to wait on. The v2 wait_agent takes no
   target.`

Test impact (round-1 Major B6): `multi_agents_spec_tests.rs:354-358` pins the followup_task
description with `assert_eq!` on the **exact** string — update it to the exact new string
(keep `assert_eq!`, do not relax to `contains`). The spawn assertions at :80-98 are
`contains` and survive edit 2.

### 5.3 Why this is a ratchet and not overfitting

Each added sentence states a fact about the current schema that is true for every model
(token cost: well under ~100 tokens in the cached tools prefix — round-1 NOTE B4). Frontier
models also emitted these hallucinations in the census, so frontier behavior improves or is
neutral; open-weights models improve. No model-specific language is introduced.

## 6. Decision items (rounds 1-2 rulings recorded)

| ID | Question | Ruling (coordinator, for round-2 confirmation) |
|---|---|---|
| D1 | Coercion helper placement | **codex-tools**, `tools/src/strict_int.rs` (P4; deps already present; no lockfile/Bazel change) |
| D2 | `deny_unknown_fields` scope | **Full class, v4** — 15 rows / 16 structs per §3.6 (round-2 Blocking N1/N2 resolved, round-3-corrected: exec_command **fold** — `workdir`/`environment_id` added to `ExecCommandArgs`, shadow **retained**, §3.6 v4 — + request_permissions documented exclusion; round-2 Major N5: `RequestPluginInstallArgs` added). Alias structs get alias-preservation tests; per-struct rollback allowed on proven breakage |
| D3 | v1 `WaitArgs.timeout_ms` coercion | **Yes** — identical field class, one field (multi_agents/wait.rs:280) |
| D4 | v1 `wait_agent` description clarification | **Yes, add** the one-liner noting v1 takes `targets` and v2 takes none |
| D5 | rollout-trace raw-args fix | **In scope** — local tolerant mirror in `terminal.rs` (§3.5); no new dependency, no lockfile change; gate `just test -p codex-rollout-trace` |
| D6 | protocol-crate wire-surface structs | **Out of scope (named follow-up class)** — `SearchToolCallParams.limit` (coercion; protocol crate / from_value path / wire surface, §3.1) **and**, round-2 extension, all protocol-crate D2 candidates: `RequestPermissionsArgs` (`protocol/src/request_permissions.rs:50`). Class boundary is deliberate: this campaign touches `codex-core` + `codex-tools` + `codex-rollout-trace` only |

## 7. TDD plan (repo conventions: unit in sibling `_tests.rs`; integration under `core/tests/suite` via `test_codex`)

### 7.1 Unit — codex-tools (`tools/src/strict_int_tests.rs`)

Per type in {u64, usize, i32, i64} × value class:
- JSON number in range → ok
- JSON number out of range / float token / bool / null (non-Option) → **error text snapshot
  pinned** (one representative per type; the v1 "byte-for-byte" claim is replaced by these
  pins per round-1 Major A2)
- digit string `"120000"` → ok, coerced
- `"0123"` → ok (123)
- `"-5"` on i32/i64 → ok; `"-5"` on u64/usize → custom error
- **`"+12"` → custom error (explicit `+` rejection — round-1 Major B4 contract)**
- `"1.5"`, `"12abc"`, `" 12"`, `""`, `"null"`, `"0x10"` → custom error, §3.2 message
- overflow digit string → custom error (no panic/wrap)
- array/object root (`visit_seq`/`visit_map`) → custom error (round-2 consistency note: the
  visitor table rejects them; the matrix must too)
- `_opt` variants: `null` → None; digit string → Some; bad string → custom error

### 7.2 Unit — core (`core/src/tools/handlers/args_parse_tests.rs`)

- Message shape 1: type error with key before it → `(parameter "timeout_ms")` suffix + tool
  prefix
- Shape 1 negative: key after the error position not attributed; no key → no hint
- Shape 2: truncated `{"cmd": "ls",` → Eof hint present
- Shape 3: non-object root `"[1,2]"` → plain prefix, no hint
- `nearest_json_key_before`: nested objects (inner key wins), keys in arrays (no `:` → not a
  key), unicode content before the error (byte-offset pin per §4.3), consume-vs-peek
  asymmetry case (invalid number), empty/short inputs
- Regression: existing parse tests that use the shared funnel keep passing with the new
  signature

### 7.3 Integration — `core/tests/suite/tool_args_ratchet.rs` (**new file**, registered in
`core/tests/suite/mod.rs` explicit mod list — round-1 Minor B2; uses `core_test_support::
responses`: `mount_sse_once` + `ev_function_call` + `ResponsesRequest`)

1. **A end-to-end (primary proof for A — round-1 confirmed it genuinely executes coercion
   SSE→funnel→strict_int→real exec):** mock server; `exec_command` function_call with args
   `{"cmd":"true","timeout_ms":"120000","yield_time_ms":"10000"}` (strings, census shape).
   Assert: no parse-error `function_call_output`; the call executes;
   `mock.single_request()` round-trips.
2. **A negative:** `sleep` with `{"duration_ms":"12.5"}` → output contains the §3.2 custom
   error naming "integer". (Round-4, seat H: `sleep` is exposed namespaced as
   `clock.sleep` — sleep.rs:26 `NAMESPACE`, `ToolName::namespaced` :66-67 — so use
   `ev_function_call_with_namespace` (pattern: current_time_reminder.rs:880-884); enabling
   needs `Feature::SleepTool` + `sleep_tool_mode` (spec_plan.rs:1183-1199) — the closest
   helper `enable_current_time_reminder` (current_time_reminder.rs:134) is file-local, so
   set the features directly on the test config.)
3. **B attribution:** `wait_agent` v2 with `{"target":"task_1"}` → output names tool
   `wait_agent` and parameter `target` (regression guard: deny_unknown_fields + B suffix).
   (Round-4, seat H: registration needs `Feature::MultiAgentV2` — spec_plan.rs:692/1301,
   pattern core/tests/suite/multi_agent_mode.rs:78; `wait_agent` v2 itself is plain-named at
   the handler level — multi_agents_v2/wait.rs:24.) **Round-7 correction (seat N-1, MAJOR —
   the round-6/7 "verified" list checked only that handler-level fact and missed the
   registration wrap):** in the suite context the v2 handler is wrapped in
   `MultiAgentV2NamespaceOverride` (spec_plan.rs:1433-1437) because
   `ProviderCapabilities::default()` sets `namespace_tools: true`
   (model-provider/src/provider.rs:64; the suite's built-in `openai` provider does not
   override it) and `MultiAgentV2Config.tool_namespace` defaults to `Some("collaboration")`
   (core/src/config/mod.rs:1315); `ToolRegistry::tool` is an exact key lookup with no
   plain-name fallback (core/src/tools/registry.rs:490-494). The mock call MUST be
   `ev_function_call_with_namespace(CALL_ID, MULTI_AGENT_V2_NAMESPACE, "wait_agent", …)` —
   suite files define `const MULTI_AGENT_V2_NAMESPACE: &str = "collaboration"` (pattern:
   subagent_notifications.rs:2542-2547) — or the test must set
   `config.multi_agent_v2.tool_namespace = None`. A plain `ev_function_call(…, "wait_agent",
   …)` never routes to the handler. The message assertion is unaffected (the handler passes
   the plain literal to the funnel, §4.2), so the B output still names `wait_agent`.
4. **C spec text:** the three edited description strings appear in the registered v2 tool
   specs (extends `multi_agents_spec_tests.rs`; assert #3's `assert_eq!` at :354-358 updated
   to the exact new string).
5. **No-regression:** one existing happy-path test per coerced struct family unchanged —
   round-4 (seat H) explicit family list: `exec_command` (real JSON numbers),
   `write_stdin` (the most type-divergent set: i32 `session_id`, u64 `yield_time_ms`, usize
   `max_output_tokens`, write_stdin.rs:23-32), plus the remaining six structs per the §3.1
   census.
6. **D census case (round-1 Major B7):** `exec_command` with `{"command":"true"}` → output
   names `unknown field `command`` + the tool (D2 behavior change, previously silent drop or
   missing-field only).
7. **D alias preservation (round-1 alias ruling; round-5 re-scope, seat J):** covered by
   the §7.5 unit test in `core/src/tools/handlers/request_plugin_install_tests.rs`
   (existing per-presentation scaffold at :33-84): `RecommendedPluginInstallArgs` —
   RecommendationContext presentation, the only branch parsing the alias-bearing struct
   (branch arm request_plugin_install.rs:134-136, parse at :135 — round-9 F2) — accepts
   alias `tool_id` AND rejects an unknown field.
   (Suite-level enablement would need ToolSuggest+Apps+Plugins + non-empty
   `tool_suggest_candidates` + the mocked `/ps/plugins/suggested/codex` endpoint —
   spec_plan.rs:633-638/:1203-1217, pattern suite/request_plugin_install.rs:126-149; the
   unit route is cheaper and unambiguous.)

### 7.4 Existing tests that break — must be updated in the same items (round-1 Major B6)

| File | What changes |
|---|---|
| `handlers/multi_agents_tests.rs:1768-1770` | pins old funnel message verbatim (`"failed to parse function arguments: unknown field `interrupt`..."`) → new shape with `send_message` tool name (the pin is send_message-specific — round-7 seat N-7; no wait_agent message pin exists repo-wide) |
| `handlers/unified_exec_tests.rs:53, 75, 113, 141, 162, 221` | 6 `parse_arguments(...)` sites gain the tool-name argument |
| `handlers/multi_agents_spec_tests.rs:354-358` | followup_task `assert_eq!` → exact new string |
| `core/tests/suite/tool_harness.rs:309-314`, `core/tests/suite/direct_tool_metadata.rs:285-289` | `update_plan` message pins (lockstep with the plan.rs funnel extension) |
| `mcp_resource_tests.rs:74-85` | message assertions **unaffected** (targets the local fn), but the 3 `parse_arguments(...)` call sites (:76, :81, :85) gain the `tool_name` argument — §4.5 extends the local fn's signature (round-2 correction to the round-1 "unaffected" note) |

### 7.5 D (deny_unknown_fields) regression coverage (round-1 Major B7)

Unit (per-struct representative set — all three census-affected structs + one alias struct +
one v1 multi-agent struct): unknown field → `FunctionCallError::RespondToModel` naming the
field; known fields unchanged; alias forms accepted (§6 D2). Integration: §7.3 item 6
(item 7's alias-preservation test is the unit test in
`core/src/tools/handlers/request_plugin_install_tests.rs` — round-6 re-scope marker).

**Exec_command restructure regression (round-2 Blocking N1/N2):** full-payload happy paths —
`{cmd, workdir, environment_id, approval/sandbox params}` and bare `{cmd, workdir}` (both
schema-legal) must parse and execute. The existing guards `suite/unified_exec.rs:845, :908`,
`suite/approvals.rs:2950, :2957`, `suite/request_permissions.rs:777` are the standing proof
and must keep passing after row 1 lands (listed as mandatory regression evidence; no new
tests required for the happy path). **Round-3 additions:**
- `suite/request_permissions.rs:728`
  (`relative_additional_permissions_resolve_against_tool_workdir`) — **named guard for the
  relative-`additional_permissions` value class** (round-3 Blocking NEW-1): the v4 design
  (pre-guard pass stays a projection) is precisely what keeps this test green; it must keep
  passing after item 4.
- Standing green guards (round-3, seat F; round-4 citation correction):
  `unified_exec_tests.rs:358` (`exec_command_reuses_foreign_windows_grant` — full-handler
  workdir payload) and `core/src/session/tests/guardian_tests.rs:618, :747, :1213`
  (workdir-bearing full-handler exec payloads; `:618` also carries
  additional_permissions + justification).
- **Row 11 unit test (round-3, seat F Major):** in `tools/src/request_plugin_install_tests.rs`
  (existing sibling, `#[path]`-declared; `FunctionCallError` lives in codex-tools at
  `tools/src/function_call_error.rs:5`): full known payload
  `{tool_type, action_type, tool_id, suggest_reason}` parses; unknown field → `Err` naming
  the field.
- **Schema-omitted/struct-declared fields stay ACCEPTED post-D (round-5 correction of the
  rounds 3-4 "flip" characterization — seats I+J):** `deny_unknown_fields` is
  **struct-relative**, not schema-relative — it rejects only names the struct does not
  declare. `timeout_ms` (schema-omitted in the interactive spec; struct-declared at
  unified_exec.rs:39), `tty` (:35) and `yield_time_ms` (:37 — functional in one-shot,
  passed into `ExecCommandRequest` at exec_command.rs:426), and every config-conditional
  omission — `shell`/`login` (spec builder exec_command.rs:117-140 +
  shell_spec.rs:65-100/:232-278), `environment_id` (Single-mode configs: spec_plan.rs:1056
  sets `include_environment_id` only for `ToolEnvironmentMode::Multiple`),
  and `additional_permissions` when approvals are off (round-6 correction, seat K:
  `create_approval_parameters`, shell_spec.rs:232-278, always emits
  `sandbox_permissions`/`justification`/`prefix_rule` — only `additional_permissions` is
  gated, `if exec_permission_approvals_enabled`, :268-274; `ExecPermissionApprovals` is
  off by default per features/src/lib.rs:1177-1181; `sandbox_permissions`' enum values
  merely narrow when off) — all remain struct-declared after D and therefore **still
  parse** post-D. The fold is precisely what keeps `workdir`/`environment_id` parseable
  under deny: the accept→reject class D creates is schema-SENT-field → struct-UNKNOWN
  rejection (e.g. `command`, `agent_name`), never struct-declared → schema-omitted
  rejection. Pins (unit, `handlers/unified_exec_tests.rs`): (1) interactive exec_command
  with a **numeric** `timeout_ms` still parses post-D (previously silently accepted and
  **ignored** in the interactive lifetime — exec_command.rs:310-319 consumes it only in
  the OneShot arm; string form parses post-A via Ratchet A, so numeric keeps the pin a
  pure D no-change assertion); (2) one-shot with a **numeric** `tty: false`/
  `yield_time_ms` payload still parses post-D; (3) ToolSpec-level schema pins, asserted
  via `ExecCommandHandler::{new,one_shot}(options).spec()` (`new` = interactive lifetime —
  round-7 seat N-2: there is no `interactive` ctor; the private
  `one_shot_exec_command_spec` is not directly callable from the tests module — round-6
  implementer route): the one-shot spec's properties lack `tty`/`yield_time_ms` (removed
  at exec_command.rs:496-497) and the interactive base schema lacks `timeout_ms`
  (shell_spec.rs:35-102 — round-7 seat M-2: the properties map begins at :35). Import
  note (round-7 seat N-10): pins (1)/(2) need no new imports; pin (3) requires
  `use codex_tools::ToolSpec;` in the test module. Pre-D validation errors are
  D-independent and fire at the same
  stage post-D: `login:true` with login shells disabled (get_command fn at
  unified_exec.rs:99, rejection arm :105-109) and `tty:true` with `UnifiedExecTty` off
  (exec_command.rs:244-248). Observation (round-5 seat I; no action): in a 1-Ready +
  ≥1 Starting/Failed environment turn, the model context renders the Ready env's id
  (world_state/environment.rs:256-291) while the Single-mode schema omits
  `environment_id` — the id is schema-illegal but model-plausible; it parses post-D
  (folded field) exactly as it parsed pre-D (shadow selection) — behavior stable across D.
- **Optional ordering pin (round-3, seat F):** one suite test locking the
  malformed-`cmd`-in-unavailable-env interaction (parse error vs
  `"unified exec is unavailable in this session"`, exec_command.rs:193 — round-5: full
  string) — cheap, include.
**Projection invariant (round-2 NOTE N8 + round-3 value clause):** no struct parsed from the
full raw model-args string may (a) reject a field of its tool's schema, or (b) declare a
guard-dependent field outside the guard — expressed operationally by the happy-path +
`:728` guard set above plus the §7.3 item 6 negative test.

### 7.6 rollout-trace (D5)

Unit test in `rollout-trace` (sibling `_tests.rs` per convention): raw write_stdin dispatch
payload with string `yield_time_ms`/`max_output_tokens` reduces without bailing; fields come
out as the coerced values; malformed string → `None` (reduction proceeds).

### 7.7 Gates

1. `just fmt` (after all edits)
2. `just fix -p codex-core -p codex-tools -p codex-rollout-trace` (round-7 seat N-3: item 2 modifies rollout-trace — the D5 mirror — so the clippy gate must scope it too)
3. `just test -p codex-core -p codex-tools -p codex-rollout-trace`
4. Full `just test` — core is touched; per AGENTS.md, **ask the operator before running**
   (Mac load discipline; load has been 10-51 from unrelated rust builds).

**800-line guidance (round-3, seat E NEW-3 — ruled on the record):** campaign-wide changed
lines estimate ≈ 1000-1250 (new modules ~750-850; ~36 funnel-site signature changes ~50-70;
16 D2 attrs ~20; fold+deny ~40; local funnels/rollout-trace mirror ~60-80; test updates ~80-120;
wiring ~10) — over the 800 soft cap as a whole. Compliance is via the §9 staging: all six
items are individually <~500 lines (AGENTS.md's own escape clause: "explore whether it can be
split into reviewable stages"). Each item commits green; the cap is per-change, not
per-campaign.

No `Cargo.toml`/`Cargo.lock` changes expected (D5 is a local mirror — round-1 Major B3
resolved without a dependency). No Bazel changes (glob-based source lists, §3.3). If anything
nonetheless changes a lockfile, `just bazel-lock-update` becomes mandatory.

## 8. Divergence bookkeeping (post-landing; operationalized: rows for
`git diff --name-only <base>..HEAD`, no exceptions — round-1 Major B8)

**Added (fork-only rows):**
- `codex-rs/tools/src/strict_int.rs` + `codex-rs/tools/src/strict_int_tests.rs`
- `codex-rs/core/src/tools/handlers/args_parse.rs` + `args_parse_tests.rs`
- `codex-rs/core/tests/suite/tool_args_ratchet.rs` (+ mod line in `core/tests/suite/mod.rs`)

**Modified (upstream-shared rows):**
- `core/src/tools/handlers/mod.rs` (funnel moved to args_parse.rs; re-export; wrapper
  signature), `handlers/unified_exec.rs`, `handlers/unified_exec/write_stdin.rs`,
  `handlers/sleep.rs`, `handlers/test_sync.rs` (A+D2), `handlers/multi_agents_v2/wait.rs`,
  `handlers/multi_agents_spec.rs` (C), `handlers/multi_agents/wait.rs` (D3+D2),
  `handlers/multi_agents/spawn.rs` (D2), `handlers/multi_agents/close_agent.rs` (D2),
  `handlers/multi_agents/resume_agent.rs` (D2), `handlers/multi_agents/send_input.rs` (D2),
  `handlers/view_image.rs` (D2), `handlers/request_permissions.rs` (stage-1 funnel +
  stage-2 tool name only — **no D2 attr**, ruling §3.6),
  `handlers/request_plugin_install.rs` (D2 + funnel), `handlers/request_user_input_spec.rs`
  (D2), `handlers/mcp_resource.rs` + 3 sub-handler files (funnel attribution),
  `handlers/plan.rs` (funnel), `code_mode/wait_handler.rs` (local copy deleted),
  `handlers/dynamic.rs`, `handlers/unified_exec/exec_command.rs` (funnel sites + restructure),
  `handlers/multi_agents_v2/spawn.rs`, `handlers/multi_agents_v2/send_message.rs`,
  `handlers/multi_agents_v2/followup_task.rs`, `handlers/multi_agents_v2/interrupt_agent.rs`,
  `handlers/multi_agents_v2/list_agents.rs`, `handlers/wait_for_environment.rs` (funnel
  site — round-2 N4), `handlers/request_user_input.rs` (funnel site — round-2 N4),
  `handlers/request_user_input_async.rs` (funnel site — round-2 N4),
  `handlers/send_message_to_user_async.rs` (funnel site — round-2 N4),
  `tools/src/request_plugin_install.rs` (D2 — codex-tools struct, round-2 N5),
  `tools/src/lib.rs` (strict_int `mod` declaration + `pub use` re-exports — round-6 seat L;
  every tools module is explicitly declared there),
  `rollout-trace/src/reducer/tool/terminal.rs` (D5)
- test files: `handlers/unified_exec_tests.rs`, `handlers/multi_agents_tests.rs`,
  `handlers/multi_agents_spec_tests.rs`, `handlers/mcp_resource_tests.rs` (3 call sites,
  round-2 N3), `core/tests/suite/mod.rs`, `core/tests/suite/tool_harness.rs`,
  `core/tests/suite/direct_tool_metadata.rs`, `rollout-trace` sibling test file,
  `tools/src/request_plugin_install_tests.rs` (row-11 unit test, round-3 seat F),
  `core/src/tools/handlers/request_plugin_install_tests.rs` (alias-preservation unit test,
  round-5 seat J)

The manifest's existing "Pending known future divergence" row for apex-xt2.7 must grow with
`multi_agents/wait.rs` (D3) and the D2-class files when they land (round-1 Major B8).

## 9. Task breakdown (draft — cross-reviewed at SDD stage 4)

One TDD item per coherent slice, ordered by dependency; each: red → green → gates →
multi-review loop to zero Blocking/Major → commit. **Ratchet D is a separate item** (staged
after coercion so its behavior change is isolable if any struct regresses):

1. `strict_int` helpers + macro + unit tests (codex-tools) — no behavior change yet
2. Wire A into the 8 target structs (4 exec/wait/sleep + v2 WaitArgs + v1 WaitArgs + test_sync
   2) + rollout-trace D5 local mirror + unit regression + integration 1/2/5 + 7.6
3. `args_parse` module (funnel move, tool-name param, key extraction, EOF hint, base_path
   wrapper) + unit tests; mechanical sweep of all 36 sites (verified list §4.1, `rg` sweep at
   implementation — no exceptions); delete code_mode local copy (post-campaign the
   shared-funnel sweep totals 38 sites — 32 production + 6 test — the code_mode wait
   switch adds one and the section 4.5 update_plan ruling converts plan.rs' local parse
   into the 32nd production site (post-landing correction, item 3: seat A-m-1 verified
   32/38 on final bytes); the 36-site list is the pre-change inventory — round-9 NOTE 3);
   extend the five local
   funnels per §4.5 rulings (mcp ×3 with semantics preserved — their 3 test call sites
   updated, plan.rs with lockstep test pins, request_permissions stage 2 tool-name only) +
   integration 3 (wait_agent B attribution)
4. Ratchet D: exec_command fold (add `environment_id`/`workdir` to `ExecCommandArgs`,
   `#[allow(dead_code)]`-annotated; shadow parse unchanged — §3.6 v4) + `deny_unknown_fields` on
   the 15-row/16-struct class + alias unit test (item 7) + unit regression (7.5) +
   integration 6 +
   full-payload regression suite (7.5); per-struct rollback with documented note if a proven
   breakage appears
   (v4: fold = add `environment_id`/`workdir` to `ExecCommandArgs` with
   `#[serde(default)]` + `#[allow(dead_code)]` + doc comment; shadow parse **unchanged** — §3.6;
   item also carries the row-11 tools-crate test, the `:728` guard, and the optional ordering
   pin (7.5, malformed-`cmd`-in-unavailable-env) per §7.5.)
5. Ratchet C description edits (+ D4 v1 line) + `multi_agents_spec_tests.rs:354` exact-string
   update + integration 4
6. `just fix`/`fmt` sweep, scoped gates (7.7), full test with operator sign-off,
   FORK-MANIFEST rows per §8 (`git diff --name-only`, no exceptions), bead closure

## 10. Review rounds log

**Round 1 (2026-09-17) — seats A (design correctness/completeness) + B (testability/integration
risk). Verdicts: A 0 Blocking / 4 Major; B 0 Blocking / 8 Major. All 12 Majors resolved in v2:**

| # | Finding (seat) | Resolution in v2 |
|---|---|---|
| A1 | B inventory wrong (~25 → 36; request_user_input.rs:80 omitted; 4 misclassified local sites; 4 unaddressed local funnels) | §4.1 exact 36-site inventory; §4.5 per-local-funnel rulings table |
| A2 | §3.2 mechanism not implementable; "byte-for-byte" overclaimed | §3.2 restated visitor-based (concrete types via macro); guarantee relaxed to accept/reject identity + snapshot-pinned error text |
| A3 | test_sync 4 numeric fields omitted from sweep | §3.1 table adds BarrierArgs/TestSyncArgs rows; all other structs verified numeric-free |
| A4 | D2 scope inconsistent with own rationale / P3 | §3.6 full 16-struct class; alias ruling + tests; per-struct rollback rule |
| B1 | mcp_resource not a funnel consumer; §7.2 mcp note wrong | §4.1/§4.4/§4.5 corrected; §7.4 marks mcp_resource_tests unaffected |
| B2 | G2 unattainable as scoped (3 in-crate + 2 out-of-crate local paths) | G2 reworded (§2); §4.5 rulings incl. tool_search verified-teachable no-change; tui/ext/goal named follow-ups; mixed formats explicit |
| B3 | rollout-trace hidden dependent breaks under A; lockfile premise | §3.5 local tolerant mirror (no dep); D5; gate 7.7 adds `-p codex-rollout-trace`; §7.6 test |
| B4 | `"+12"` contradiction (FromStr accepts `+`) | §3.2 explicit `+` rejection ruling; §7.1 test case |
| B5 | D2 omits v1 WaitArgs | §3.6 row 4 |
| B6 | Test-impact list wrong/incomplete (3 files break, 1 doesn't) | §7.4 table (multi_agents_tests:1768, unified_exec_tests 6 sites, multi_agents_spec_tests:354; mcp unaffected); §5.2 note |
| B7 | D2 behavior change has no planned regression test | §7.3 items 6-7 + §7.5 |
| B8 | §8 manifest list incomplete | §8 exhaustive list; operationalized as `git diff --name-only` in item 6 |

Minor/Note dispositions: v1 `targets` line ref fixed (:279, §3.1 header); `{"command":"true"}`
example updated to D-variant (§4.2); `parse_arguments_with_base_path` named (§4.1/§9 item 3);
tui/dynamic_tools + apply_patch explicit exclusions (§4.4/Non-goals); consume-vs-peek
asymmetry noted (§4.3); BUILD.bazel definitively no-change (§3.3/§7.7); memory_usage disclosed
(§3.7); SearchToolCallParams.limit ruled out (§3.1/D6); suite test file named
(§7.3); "0123" risk accepted (§3.4).

**Round 2 (2026-09-17) — seats C (design) + D (testability/integration), fresh agents.
Verdicts: C 0 Blocking / 1 Major; D 2 Blocking / 3 Major. All resolved in v3 (this revision):**

| # | Finding (seat) | Resolution in v3 |
|---|---|---|
| N1 (BLOCKING, D) | D2 rows 11-12 (env-shadow structs) parsed from FULL raw args before the main parse — deny would reject every schema-legal call (`unknown field 'cmd'` / `'permissions'`) | §3.6 projection invariant + exec_command restructure (fold env fields, delete shadow — **superseded by round 3**: v4 retains the shadow, §3.6) + request_permissions documented exclusion |
| N2 (BLOCKING, D) | `ExecCommandArgs` lacks schema-legal `workdir`/`environment_id`; deny would break existing integration tests (suite/unified_exec.rs:845,908; approvals.rs:2950,2957; request_permissions.rs:777) | Same restructure; those tests are now the mandatory regression guard (§7.5) |
| N3 (MAJOR, D) | §4.5/§7.4 contradiction: mcp_resource_tests.rs 3 call sites gain tool_name | §7.4 reworded; file added to §8 test list |
| N4 (MAJOR, D) | §8 misses 4 necessarily-modified funnel files | §8 adds wait_for_environment.rs, request_user_input.rs, request_user_input_async.rs, send_message_to_user_async.rs |
| N5 (MAJOR, D) + (C Major) | D2 misses `RequestPluginInstallArgs` (codex-tools, tools/src/request_plugin_install.rs:16) | §3.6 row 11; §8 row; §7.5 representative set; protocol-crate survivors named as D6 follow-up class |
| C-1 (MAJOR) | D2 "full class" = the N5 miss | Same as N5 |
| N6 (MINOR, D) | §7.3 integration item 3 unassigned in §9 | §9 item 3 gains it |
| N7 (MINOR, D) + C-minor cluster | line-number slips (terminal.rs :408/:60, defs.bzl :311, etc.) | All corrected in place |
| C (MINOR) | apply_patch citation pointed at wrong file | §4.4 → core/src/tools/handlers/apply_patch.rs:149 (+ :529/:599) |
| C (NOTE) | agents.rs:305 sweep evidence | §3.5 sweep-evidence paragraph |
| C (NOTE) | TOOL_NAME constant claim | §4.2 implementation note (only sleep.rs defines it) |
| C (consistency) | §7.1 matrix missing seq/map root cases | §7.1 line added |
| D (NOTE, N8) | rely-check audited external consumers but not in-handler second parses | §3.6 projection invariant + §7.5 happy-path guard |

Round-2 also **verified** (no action needed): all 36 funnel sites; visitor design vs serde
1.0.228 default integer visitors; alias coexistence soundness; rollout-trace D5 coherence +
single-affected-site sweep; gate package names (codex-tools/codex-core/codex-rollout-trace);
BUILD.bazel glob claims (defs.bzl:311, core/BUILD.bazel:7); core_test_support::responses
helper names (mount_sse_once :1119, ev_function_call :933, ResponsesRequest :104,
single_request :50); no .snap captures of edited strings; 27-struct census completeness
within core+codex-tools.

**Round 3 (2026-09-17) — seats E (design) + F (testability/integration), fresh agents.
Verdicts: E 1 Blocking / 0 Major (2 minor, 3 note); F 0 Blocking / 1 Major (2 note).
All resolved in v4 (this revision):**

| # | Finding (seat) | Resolution in v4 |
|---|---|---|
| NEW-1 (BLOCKING, E) | v3 "single-struct restructure" (pre-guard first parse = full `ExecCommandArgs`) refuted: the two parses differ in **guard state** — the second runs under `AbsolutePathBufGuard` (`mod.rs:149-158`) and a relative `AbsolutePathBuf` hard-errors without a thread-local base (`utils/absolute-path/src/lib.rs:343-348`) → relative `additional_permissions` flips accept→reject, breaking `suite/request_permissions.rs:728` | §3.6 v4 design: fold `workdir`/`environment_id` into `ExecCommandArgs` (`#[serde(default)]` + `#[allow(dead_code)]` + doc comment); shadow `ExecCommandEnvironmentArgs` + its parse **unchanged**; `deny_unknown_fields` on the full struct only; v3 "error ordering shifts" claim retracted (ordering unchanged); projection invariant gains a value clause; `:728` named guard in §7.5 |
| NEW-2 (MINOR, E) | §4.2 note factually wrong: **four** files define a `TOOL_NAME` constant (`sleep.rs:27`, `send_message_to_user_async.rs:21`, `request_user_input_async.rs:22`, `current_time.rs:25`) — the round-2 correction itself was wrong | §4.2 corrected to the four-file set |
| NEW-3 (MINOR, E) | campaign-wide changed lines ≈ 1000-1250, over the 800-line guidance; compliance exists only via §9 staging | ruling recorded on the record in §7.7 (all six items individually <~500; AGENTS.md split-into-stages escape clause) |
| NEW-4 (NOTE, E) | schema-omitted/struct-declared fields "flip" to hard `unknown field` errors post-D (interactive exec rejects `timeout_ms`; one-shot rejects `tty`) — **round-5 refutation (seats I+J): `deny_unknown_fields` is struct-relative; struct-declared fields stay ACCEPTED post-D** | §7.5 pins re-scoped to acceptance + ToolSpec pins (v6) |
| NEW-5 (NOTE, E) | field-by-field schema↔struct enumeration otherwise clean — the only accept→reject class beyond intended unknowns is NEW-1's relative-path value class | verification logged (no edit) |
| NEW-6 (NOTE, E) | `memory_usage.rs:46` best-effort `.ok()` and hook paths (`with_updated_hook_input` `Value` round-trip, `cmd`-only rewrite) unaffected | verification logged (no edit) |
| F-1 (MAJOR, F) | row-11 `RequestPluginInstallArgs` deny lands with **no planned test** — §7.5 representative set pins only pre-existing behavior | unit test added: `tools/src/request_plugin_install_tests.rs` (existing `#[path]` sibling; `FunctionCallError` in codex-tools at `tools/src/function_call_error.rs:5`); §3.6/§7.5/§8/§9 item 4 |
| F-2 (NOTE, F) | §7.5 guard list under-enumerates standing green guards (`unified_exec_tests.rs:358`; `guardian_tests.rs:618/:747/:1213`) | added to §7.5 standing-proof list |
| F-3 (NOTE, F) | malformed-args + unavailable-env interaction untested (parse error vs `"unified exec is unavailable in this session"` — round-5: full string, `exec_command.rs:193`); v4 keeps today's ordering (shadow parse first) | optional suite-test ordering pin added to §7.5 (included, cheap) |

Round-3 also **verified** (no action needed): fold is construction-safe (no `ExecCommandArgs`
construction sites — parse sites only); strict_int cannot double-apply (shared funnel, no state
carried between passes); `ExecCommandEnvironmentArgs` has exactly 3 refs (def/import/parse);
request_permissions exclusion rests on **two** independent reasons — (a) protocol-crate wire
surface (`protocol/src/request_permissions.rs:50`) and (b) `resolve_permission_path_strings`
(`request_permissions.rs:88`, :183-213) must rewrite relative paths on the Value **before**
stage-2 `from_value` (:89-93) — reason (b) added to §3.6; N5 `RequestPluginInstallArgs` scope;
D6 protocol-crate survivors (`SearchToolCallParams` at `protocol/src/models.rs:2073`);
N4/N3/N6/N7 + C-minor spot-checks.

**Round 4 (2026-09-17) — seats G (design) + H (testability/integration), fresh agents.
Verdicts: G 0 Blocking / 1 Major (3 minor, 2 note); H 0 Blocking / 0 Major (3 minor, 2 note).
All resolved in v5 (this revision):**

| # | Finding (seat) | Resolution in v5 |
|---|---|---|
| G-1 (MAJOR) | §3.6 exclusion list still asserted the refuted v3 deletion ("`ExecCommandEnvironmentArgs` — deleted by the restructure") — direct contradiction with the v4 design in the same section; D2 table header still labeled v3 | exclusion bullet rewritten (shadow **retained**, tolerant projection, D2-excluded for that reason); D2 table header relabeled v4; round-2 log N1 row marked superseded |
| G-2 (MINOR) | §6 D2 decision row recorded the refuted v3 design ("single-struct restructure", "Full class, v3") as the current ruling | §6 D2 row reworded to the v4 fold + shadow retained; v3→v4 |
| G-3/H-3 (MINOR, duplicate) | flip enumeration incomplete: the one-shot spec also removes `yield_time_ms` (exec_command.rs:497; struct-declared at unified_exec.rs:37, functional at exec_command.rs:426) → "unrecorded accept→reject flip"; config-conditional omission class not on the record — **round-5 (seats I+J) refuted the flip premise itself: a struct-declared field can never be `unknown field` under deny** | §7.5 v6: corrected acceptance + ToolSpec pin set; conditional-omission class recorded as behaviorally unchanged |
| G-4/H-1 (MINOR, duplicate) | alias-test list named #13 `ListResourceArgs` (mcp_resource.rs:62) as alias-bearing — it has none; the D2 class has exactly one alias (#12 `RecommendedPluginInstallArgs.plugin_id`, request_plugin_install.rs:50) | alias ruling corrected to #12 only |
| G-5/H-4 (NOTE, duplicate) | citation precision: `guardian_tests.rs` unqualified (four same-named files; intent is `core/src/session/tests/guardian_tests.rs`); `:747`/`:1213` label over-describes; `:777` is inside the `:728` test; `shell_spec.rs:170` is the request_permissions schema's `environment_id`, not exec_command's | path qualified; label softened; `:728`/`:777` same-test note; `:170` citation dropped |
| H-2 (MINOR) | §7.3 item 2 under-specified: `sleep` is exposed namespaced as `clock.sleep` (plain dispatch → unknown-tool) and enabling needs `Feature::SleepTool` + `sleep_tool_mode` (spec_plan.rs:1183-1199; `enable_current_time_reminder` is file-local); item 3 needs `Feature::MultiAgentV2` (spec_plan.rs:692/1301; wait_agent v2 plain-named) | item 2 gains `ev_function_call_with_namespace` + feature prerequisites; item 3 gains the MultiAgentV2 prerequisite |
| H-5 (NOTE) | §7.3 item 5 named only the exec_command family | family list made explicit (exec_command + write_stdin + remaining census structs) |
| G-6 (NOTE) | §3.6 reliance check omitted `pre_tool_use_payload` (exec_command.rs:522-533 — round-5 range correction; parse at :527) — a best-effort `.ok()` full-struct parse outside the guard | disclosed in the reliance check + §3.7 (extended, v6) |

Round-4 also **verified** (no action needed): all five NEW-1 fold clauses re-derived against
the tree (shadow field types — no rename/alias mirroring needed; zero `ExecCommandArgs`
construction sites; guard-state reasoning; error ordering unchanged; `:728` guard semantics);
NEW-2 four-file TOOL_NAME set; NEW-3 §7.7 ruling; NEW-5 schema↔struct enumeration clean for
ALL served variants (incl. `allow_tty`/single-environment/no-approvals configs); NEW-6
memory_usage/hook paths; request_permissions two-reason exclusion; `[workspace.lints.rust] =
{}` default-warn → the field-level `#[allow(dead_code)]` warning-clean claim; all nine §7.5
guard sites at cited lines; row-11 tools test feasible in-crate (derives Debug+Deserialize
only; `#[path]` sibling exists; `FunctionCallError` in codex-tools); §7.3 helper lines exact;
F-3 ordering pin implementable (zero-env state pattern approvals.rs:679-680,
session/tests.rs:7567-7596); v4 regression sweep clean (every workdir/environment_id exec
payload repo-wide stays green; the exec_command.rs:299-309 destructuring carries `..`);
§7.4 breakage list complete with no NEW v4 breakage (16-struct schema↔struct re-verification);
zero .snap captures of edited strings; gate package names exact.

**Round 5 (2026-09-17) — seats I (design) + J (testability/integration), fresh agents.
Verdicts: I 1 Blocking / 0 Major (2 minor, 4 note — round-6 tally correction); J 0 Blocking / 1 Major (1 minor, 2 note).
The Blocking and the Major share one root cause (the "flip" premise) and are resolved
together in v6 (this revision):**

| # | Finding (seat) | Resolution in v6 |
|---|---|---|
| J-1 (MAJOR) + I-1 (BLOCKING, same root cause) | the §7.5 "flip pins" assert behavior that **cannot occur**: `deny_unknown_fields` is struct-relative — `timeout_ms`/`tty`/`yield_time_ms` (and, post-fold, `workdir`/`environment_id`) are declared on `ExecCommandArgs`, so they stay accepted post-D regardless of schema omission; the pins would fail forever (round-3 NEW-4 seeded the mischaracterization, round 4 amplified it). I-1's claimed `environment_id` accept→reject flip likewise cannot fire post-fold (declared field) | §7.5 rewritten: "schema-omitted/struct-declared fields stay ACCEPTED post-D" on the record, with the true accept→reject class named (schema-sent → struct-unknown, e.g. `command`/`agent_name`); pins re-scoped to (1) interactive `timeout_ms` still parses, (2) one-shot `tty`/`yield_time_ms` still parses, (3) ToolSpec-level schema pins (one-shot lacks tty/yield_time_ms at exec_command.rs:496-497; interactive lacks timeout_ms, shell_spec.rs:34-102); pre-D validation errors (`login:true`, `tty:true`) recorded as D-independent; the 1-Ready+Starting/Failed env-id visibility case recorded as a no-action observation (behavior stable across D); round-3 NEW-4 row marked refuted |
| J-2 (MINOR) | §7.3 item 7 under-specified: the `tool_id` alias is parsed only under RecommendationContext presentation (request_plugin_install.rs:134); suite-level enablement needs ToolSuggest+Apps+Plugins + `tool_suggest_candidates` + mocked `/ps/plugins/suggested/codex` endpoint (spec_plan.rs:633-638/:1203-1217) | item 7 re-scoped to a unit test in `core/src/tools/handlers/request_plugin_install_tests.rs` (existing per-presentation scaffold :34-80); file added to §8 |
| I-2 (MINOR) | the reliance-check edit claimed a §3.7 disclosure §3.7 did not contain | §3.7 extended to the full `.ok()` consumer class (memory_usage + pre_tool_use_payload), heading renamed |
| I-3 (MINOR) | `pre_tool_use_payload` range off (:517-530; fn is :522-533, parse :527 exact) | corrected in §3.6 + round-4 log row |
| I-4 (NOTE) | §3.6 cited memory_usage `from_str` at :48 (chain is :46-48) | corrected to :46-48 |
| I-5 (NOTE) | quoted error string was partial (`"unified exec is unavailable"`; actual: `"...in this session"`) | full string in §7.5 + round-3 F-3 log row |
| I-6 (NOTE) | "previously silently honored" imprecise for interactive `timeout_ms` (no effect in interactive lifetime) | "silently accepted and ignored" |
| I-7 (NOTE) | shell/login omission range shell_spec.rs:74-100 omitted the `shell` gate (:65-72) | corrected to :65-100 |
| J-3 (NOTE) | `multi_agent_mode.rs:78` ambiguous (two same-named files) | qualified to core/tests/suite/multi_agent_mode.rs:78 |
| J-4 (NOTE) | `current_time_reminder.rs:875-880` call actually spans :880-884; pre_tool_use range (with I-3) | corrected |

Round-5 also **verified** (no action needed): §3.6 exclusion bullet matches the v4 design
items 1-3; no leftover refuted-v3 wording (log hits historical, N1 marked superseded); D2
header + §6 row v4-labeled; one_shot removes tty/yield_time_ms at :496/:497; yield_time_ms
functional (exec_command.rs:426 → process_manager.rs:632-637); config gates (shell
:65-72, login :73-81, environment_id :82-90, approval params :232-278); ExecPermissionApprovals
default-off (features/src/lib.rs:1177-1181); pre-D validation sites (get_command
unified_exec.rs:99, rejection arm :105-109; tty exec_command.rs:244-248); exactly one alias in the D2 class;
pre_tool_use_payload parse at :527 with .ok() at :528; zero ExecCommandArgs construction
sites; NEW-2/NEW-5/NEW-6 re-spot-checks; request_permissions flow :70/:88/:89-93; D2 census
arithmetic (27 structs); all §7.3 prerequisites (clock namespace sleep.rs:26/:66-67,
ev_function_call_with_namespace responses.rs:945, Feature::SleepTool gate
spec_plan.rs:1183-1199, file-local enable_current_time_reminder, Feature::MultiAgentV2
spec_plan.rs:692/:1301, wait_agent v2 plain-named multi_agents_v2/wait.rs:24); §7.3 item 5
family list exact (write_stdin.rs:23-32); guard-site citations (guardian_tests
:618/:747/:1213 payloads; :777 inside the :728 test); §7.3 item 6 `command` absent from
every exec schema variant; snapshots clean (1535 .snap files); §8 package coverage exact
(codex-core/codex-tools/codex-rollout-trace only); round-4 log verdicts + rows accurate.

**Round 6 (2026-09-17) — seats K (design) + L (testability/integration), fresh agents.
Verdicts: K 0 Blocking / 0 Major (3 minor, 4 note); L 0 Blocking / 1 Major (2 minor, 5 note).
All resolved in v7 (this revision):**

| # | Finding (seat) | Resolution in v7 |
|---|---|---|
| L-1 (MAJOR) | `codex-rs/tools/src/lib.rs` missing from the §8 Modified list — §3.3 mandates exporting `strict_int` from codex-tools, and lib.rs declares every module explicitly (same class as round-2 N4) | `tools/src/lib.rs` row added to §8 (strict_int `mod` + `pub use` re-exports) |
| K-2/L-2 (MINOR, duplicate) | stale "Integration: §7.3 items 6-7" + §9 "integration 6/7" after the item-7 unit re-scope | both reworded (item 6 integration; item 7 = unit test in `core/src/tools/handlers/request_plugin_install_tests.rs`) |
| K-3 (MINOR) | §7.5 approval-omission list mislabeled 3 of 4 fields: `create_approval_parameters` (shell_spec.rs:232-278) always emits `sandbox_permissions`/`justification`/`prefix_rule`; only `additional_permissions` is gated (:268-274) | reworded to the exact gated field + the enum-narrowing note |
| K-1/L-6 (MINOR, duplicate) | round-5 verdict header "(3 minor, 3 note)" contradicted its own table (2 minor, 4 note) | header corrected (row tags are the finer record) |
| K-4/L-3/L-4 (NOTE, cluster) | off-by-one citation cluster (login gate :73-81, environment_id :82-90, spec() :117-140, get_command :99/:105-109) + 8 D2 table rows citing derive lines instead of struct lines (spawn :224, close_agent :165, resume_agent :165, send_input :144, view_image :59, request_plugin_install :49, mcp_resource :63/:100) + item-7 scaffold range (:33-84) | all corrected to struct lines / exact ranges |
| K-5/L-5 (NOTE, duplicate) | garbled §3.7 citation `memory_usage.rs:4/46-48` | normalized to `:46-48` (import at :4) |
| K-6 (NOTE) | value-clause first sentence over-broad — the main parse's unguarded `None` branch (foreign-executor cwd, exec_command.rs:240-241) parses a guard-dependent struct pre-existing | tightened to "parsed in the *pre-guard pass*" with the `None`-branch carve-out noted |
| K-7/L-7 (NOTE, duplicate) | pin (3) route: private `one_shot_exec_command_spec` is not callable from the tests module; the working route is `ExecCommandHandler::{new,one_shot}(options).spec()` (`new` = interactive; round-7 seat N-2 corrected the ctor name; pub(crate) ctors; `properties: Option<BTreeMap<String, JsonSchema>>` supports key-absence asserts) | route recorded in the pin (3) text |
| L-8 (NOTE) | "previously silently accepted" holds only for numeric `timeout_ms` (string was a parse error pre-A) | pins (1)/(2) specified to numeric payloads (pure D no-change assertion) |

Round-6 also **verified** (no action needed): core semantics re-derived — no escape hatch for
declared-field rejection (unconditional derive, no cfg-gated fields/aliases, field-level
`deserialize_with` only, single struct for both lifetimes, lifetime selects schema only);
pin set implementable from `handlers/unified_exec_tests.rs` (funnel-level parse sites +
handler `spec()` route; shell_spec builders reachable); flip sweep CLEAN (no unmarked stale
flip wording in §1-§9; NEW-1 relative-path class untouched); invariant vs §7.5 consistent;
round-5 env-id observation confirmed (spec_plan.rs:1056, tool_config.rs:88-94,
environment_selection.rs:965-972, world_state/environment.rs:256-291/:515-518; behavior
stable across D); item-7 unit test writable (scaffold :33-84, `use super::*` gives the
private struct, FunctionCallError in scope); item-2 feature enablement route concrete
(`config.features.enable(Feature::…)` mod.rs:1079 + `config.sleep_tool_mode` :1076 — first
suite use of SleepTool); §8 package coverage exact incl. the two new test files; D2 census
independently re-derived (27 structs, 17 open = 15 table rows + 2 exclusions, 10 already
deny — message_tool.rs "opens" were grep false positives); 1535 .snap files clean.

**Round 7 (seats M+N, v7):** M = 0 Blocking / 0 Major (1 minor, 1 note); N = 0 Blocking /
1 Major (2 minor, 7 note). Resolved in v8:

| ID | Finding | v8 action |
|---|---|---|
| N-1 (MAJOR) | §7.3 item 3 "no namespace needed" false in the suite context — v2 handler wrapped in `MultiAgentV2NamespaceOverride` (spec_plan.rs:1433-1437; `namespace_tools: true` default at model-provider/src/provider.rs:64, suite `openai` provider does not override; `tool_namespace` default `Some("collaboration")` config/mod.rs:1315; exact-key lookup registry.rs:490-494) ⇒ plain `ev_function_call` never routes; item 3 could not go green as worded | item-3 note rewritten: `ev_function_call_with_namespace(CALL_ID, MULTI_AGENT_V2_NAMESPACE, "wait_agent", …)` (pattern subagent_notifications.rs:2542-2547) or `config.multi_agent_v2.tool_namespace = None`; message assertion unaffected (funnel gets the plain literal, §4.2); round-6/7 handler-level "verified" note corrected (wait.rs:24 plain is true but incomplete) |
| N-2 (MINOR) | §7.5 pin (3) named a non-existent ctor `interactive` | both occurrences (pin text + K-7/L-7 log row) corrected to `ExecCommandHandler::{new,one_shot}(options).spec()` (`new` = interactive lifetime) |
| N-3 (MINOR) | §7.7 gate 2 omitted `-p codex-rollout-trace` though item 2 modifies rollout-trace (D5) and gate 3 includes it | gate 2 now `-p codex-core -p codex-tools -p codex-rollout-trace` |
| M-1 (MINOR) | §4.2 listed `current_time.rs:25` among "four files … use the constant there" though it is not a funnel consumer | annotated "defines the constant but is **not** a funnel consumer (no `parse_arguments` call; no change needed)" |
| M-2 (NOTE) | pin (3) cited `shell_spec.rs:34-102`; the properties map begins at :35 | corrected to :35-102 |
| N-4 (NOTE) | `parse_args` cited at `mcp_resource.rs:394` (×2); the fn is at :388 | both corrected to :388 |
| N-5 (NOTE) | inventory missed the third local helper `parse_args_with_default` (mcp_resource.rs:402; calls `parse_args` at :407; call sites list_mcp_resources.rs:66 / list_mcp_resource_templates.rs:66) whose signature change is mechanically forced | named in §4.1 and as a new §4.5 row |
| N-6 (NOTE) | §7.5 range `shell_spec.rs:65-100/:232-275` 3 lines short for the approval builder | aligned to :232-278 (matches the :232-278 already cited in the same section) |
| N-7 (NOTE) | §7.4 row 1 "wait_agent/send_message tool name" — the pin is send_message-specific | "wait_agent/" dropped |
| N-8 (NOTE) | D4 v1 line had no exact wording unlike C's three exact strings | §5.2 item 4 added with the exact append text (verified against `create_wait_agent_tool_v1`, multi_agents_spec.rs:264-270; v1 takes `targets` per multi_agents/wait.rs:279) |
| N-9 (NOTE) | §9 item 4 did not name the "Optional ordering pin" (7.5) | named in the item-4 parenthetical |
| N-10 (NOTE) | pin (3) needs `use codex_tools::ToolSpec;` in the test module; pins (1)/(2) need no new imports | recorded in the pin (3) text |

Round-7 also **verified** (no action needed): all seven v6→v7 resolutions clean (§8 lib.rs
row; §7.5 approval-field correction — `create_approval_parameters` always emits
sandbox_permissions/justification/prefix_rule, only additional_permissions gated; pin set +
`spec()` route; value-clause tightening + the :240-241 None-branch carve-out; D2 struct-line
cites; round-5 tally); the fold design fully consistent with the tree (handler reads
workdir/environment_id from the shadow at exec_command.rs:189/:198 ⇒ folded fields genuinely
dead ⇒ `#[allow(dead_code)]` correct; the :299-309 destructure's `..` absorbs them); flip
sweep clean; alias `#[serde(alias = "tool_id")]` at request_plugin_install.rs:50; D2 census
"17 open" independently re-derived (27 structs); 1535 `.snap` files clean; §8 file set
re-derived complete (38 production + 13 test files) with package coverage exactly
codex-core/codex-tools/codex-rollout-trace; all five §7.4 rows accurate (repo-wide old-message
grep = exactly the three listed test pins); each item transition leaves a green tree; root
`justfile`/`Justfile` byte-identical with `working-directory := "codex-rs"` ⇒ gate text
cwd-correct.

**Round 8 (seats O+P, v8):** O = 0 Blocking / 0 Major (3 minor, 2 note); P = 0 Blocking /
0 Major (2 minor, 2 note). **CONVERGED — the round returned zero Blocking + zero Major.**
Applied in v8.1 (clerical patch; each fix is a seat-proposed, tree-verified text/citation
correction — no design change):

| ID | Finding | v8.1 action |
|---|---|---|
| O-F1 (MINOR) | §9 item 2 "8 target structs (5 exec/wait/sleep + v2 WaitArgs + v1 WaitArgs + test_sync 2)" sums to 9; the §3.1 authoritative set is 8 (exec/wait/sleep group is 4: ExecCommandArgs, WriteStdinArgs, SleepArgs, ExecWaitArgs) | "5" → "4" |
| O-F2 (MINOR) | stale local-funnel counts after v8's N-5 row: the §4.5 tail "the four extended local funnels" and §9 item 3 "extend the four local funnels (mcp ×2 …)" — the extended set is now five (mcp ×3: :373/:388/:402 + update_plan + request_permissions stage 2) | "four" → "five", "mcp ×2" → "mcp ×3" (both places) |
| O-F3 (MINOR) | §3.1 cite "fork_turns … spawn.rs:271" unresolvable against the v1 file (v1 spawn.rs is 256 lines; `fork_turns` is v2-only at multi_agents_v2/spawn.rs:271; v1 has `fork_context: bool`, multi_agents/spawn.rs:224-231) | cite qualified to multi_agents_v2/spawn.rs:271 + v1 note |
| P-1 (MINOR) | §4.3 key-scan boundary off-by-one vs the §4.2/§7.3 item-3 contract: for `{"target":"task_1"}` serde fixes the unknown-field error at the colon (line 1 column 9 = 0-based offset+1), which the "at or before the offset" scan excludes ⇒ the flagship B case would miss its `(parameter "target")` hint as worded | step-2 boundary corrected to the 1-based `column` (0-based `column` exclusive), naming serde's unknown-field position and covering the §4.2 example |
| P-2 (MINOR) | §4.1 inventory "request_plugin_install.rs:125, :135" — the RecommendationContext parse is at :134 (self-contradiction with §7.3 item 7's :134 cite) | ":135" → ":134" |

Round-8 also **verified** (no action needed): all v7→v8 resolutions present and factually
correct — N-1 wrap/defaults/registry/exact-key with **both** item-3 alternatives viable
(incl. the `tool_namespace = None` normalization path via `with_default_namespace`,
protocol/src/tool_name.rs:39-44,68); N-2/N-10 ctor + import audit exact; N-3 gate scope;
M-1 consumer set exact at sleep.rs:95 / send_message_to_user_async.rs:76 /
request_user_input_async.rs:94; M-2/N-4/N-5/N-6 cites line-exact; N-7/N-8/N-9 exact incl.
byte-for-byte C/D4 description tails; all 36 funnel sites (30 production + 6 test)
line-exact; all 15 D2 struct-line cites exact; each item transition leaves a green tree;
gates runnable from repo root with exact package coverage; §7.3 item 3's B attribution is
green-capable pre-item-4 (v2 `WaitArgs` already denies, multi_agents_v2/wait.rs:127-131);
§10 round-7 table consistent with the v8 body on all rows.

Round-8 notes (no action): O-F4 boundary off-by-one cluster (registry.rs:490-494 vs
:489-493, terminal.rs:60/:62, tool.rs:108/:109, multi_agents_spec.rs:864-876/:873,
shell_spec.rs:268-274/:275, :35-102/:90, router.rs:270-277/:268-269, exec_command.rs:310-319/:318,
direct_tool_metadata.rs:285-289/:283-288 — every range contains the referenced code; only
the registry.rs span was v8-introduced); O-F5 log bookkeeping nits (N-1 row's "round-6/7"
attribution — the handler-level fact originates at round-4 H-2 / round-5 verified; the
frozen round-6 J-1 row retains the pre-M-2 ":34-102" cite as a historical record); P-NOTE
residual ±1 cites (§4.5 request_permissions :91-93 vs the :90 `from_value` call; §3.7
memory_usage.rs:46-48 vs :45-47; §3.1/D6 router.rs:270 vs :269 — cosmetic, all
contain/adjacent); P-NOTE log hygiene (same frozen J-1 row — acceptable precedent).

**Round 9 (seats Q+R, v8.1):** Q = 0 Blocking / 0 Major (2 minor); R = 0 Blocking /
1 Major (2 minor, 3 note). Not converged. Resolved in v8.2:

| ID | Finding | v8.2 action |
|---|---|---|
| R-F1 (MAJOR) / Q-F2 (MINOR) | v8.1 P-1 parenthetical "(0-based `column` exclusive)" self-contradictory: the operative rule, the qualifying example, and serde's verified behavior (unknown-field error at line 1 column 9, the colon at 0-based offset+1 — vendor serde_json de.rs:441, read.rs:421-429, de.rs:1986-2020) all pin **inclusive**; a literal exclusive reading breaks the flagship §4.2/§7.3 item-3 B case | parenthetical reworded to "the boundary is 0-based `offset+1`, **inclusive** … a key whose closing quote ends at the reported offset and whose `:` sits one past it (at the 1-based `column` value, 0-based `offset+1`) qualifies"; the voided wording noted inline |
| R-F2 (MINOR) / Q-F1 (MINOR) | v8.1 P-2 inverted a correct cite: the RecommendationContext parse is at request_plugin_install.rs:135; :134 is the match arm — the pre-v8.1 ":125, :135" inventory was right; v8.1 "fixed" §4.1 toward §7.3 item 7's branch-arm :134 cite | §4.1 restored to `:125, :135`; §7.3 item 7 disambiguated "(branch arm request_plugin_install.rs:134-136, parse at :135)" |
| R-F3 (MINOR) | §3.3 `defs.bzl:188` cites the def signature; the default glob is at defs.bzl:311 (doc :232) — stale drift, pre-v8.1 (round-2 log had :311) | `defs.bzl:188` → `defs.bzl:311` with the drift noted |
| R-NOTE1 | §3.6 census sentence: apparent 16+2=18 vs 17 open — row 11 is the codex-tools struct, outside the 27-struct core census | "(row 11 is the codex-tools struct — outside this core census; 15 of the 16 table structs are core)" added |
| R-NOTE2 | Eof-before-key-extraction ordering was implicit (shape-2 test wording would not catch an appended parameter suffix) | one-line ordering clause added under the §4.2 message shapes |
| R-NOTE3 | post-campaign shared-funnel sweep totals 37 sites (31 production + 6 test) — the code_mode wait switch adds one; 36 is the pre-change inventory | clause added to §9 item 3 |

Round-9 also **verified** (no action needed): O-F1/F2/F3 all correctly applied (8-struct
arithmetic sums to 8 vs §3.1; five Extend rows vs the §4.5 table — mcp ×3 call sites
line-exact incl. mcp_resource_tests.rs:76/:81/:85; multi_agents_v2/spawn.rs:271 =
`fork_turns: Option<String>` with v1 struct :224-231 `fork_context: bool`, v1 file 256
lines); the P-1 boundary RULE itself sound — worked through against vendored serde_json:
unknown-field at column 9 ⇒ the key qualifies under the inclusive boundary; class (b)
string type-error (`timeout_ms`, C=35, colon at 0-based 26) found under all readings;
class (c) all truncations `Category::Eof` ⇒ step 2 never reached for shape 2; no
step-1/step-2 disagreement; the D2 census 27/17/10 exact (17 = 15 core table structs + 2
exclusions; row 11 codex-tools; A\D = {SleepArgs, WaitArgs v2} — the two A-targets that
already deny, sleep.rs:33 / multi_agents_v2/wait.rs:127; A∩D = 6); 30+6 funnel sites
line-exact apart from the F2 site; §4.1 file set ⊆ §8; §7.3↔§9 mapping one-to-one and
complete exactly as round 8 claimed; gates: every §8 file maps to the three packages,
args_parse.rs + args_parse_tests.rs in codex-core (mod.rs:86 re-export), strict_int* +
row-11 in codex-tools (deps at tools/Cargo.toml:28-29), D5 mirror implementable
(terminal.rs:417-418 single use, JsonValue in scope, no dep change); header/§10 R1-R8
tallies match the log.

Round-9 notes (no action): the round-8 frozen rows retain their pre-correction cites per
the frozen-row precedent (Q's F-1 and R's F2 both apply to the v8.1 action rows, not the
frozen records).

**Round 10 (seats S+T, v8.2):** S = 0 Blocking / 0 Major (1 note); T = 0 Blocking /
0 Major (1 minor, 2 note). **CONVERGED — SPEC COMPLETE at v8.2.** Both seats worked the
step-2 boundary from text alone against vendored serde_json 1.0.149: case (a)
`{"target":"task_1"}` ⇒ offset 8, boundary offset+1 = 9 inclusive, colon at 9 qualifies
⇒ hint `target` (the §4.2 pinned message + §7.3 item 3 satisfiable); case (b)
`{"cmd":"true","timeout_ms":"120000"}` ⇒ column 35 / offset 34, `timeout_ms` (colon
0-based 26) found under all readings. All six v8.2 items verified correctly applied with
no fallout: "exclusive" survives only in the step-2 void-note + frozen rows; ":134" only
in item 7's annotated branch-arm cite + frozen rows; "defs.bzl:188" only in the
cited-fix note; "36" only in pre-change-inventory contexts; §10 round-9 table matches
the v8.2 body verbatim on all six rows; the §4.2/§4.3/§7.2 triangle partitions totally
and disjointly (serde Category single-valued: Eof ⇔ truncation, type errors are Data);
count census re-derived exact (8 / 27-17-10 / 15 rows-16 structs / 30+6=36 / 4+1+1+2=8 /
36 pre-37 post / 2+6=8); §9 item 3's NOTE-3 parenthetical preserves the list's meaning;
defs.bzl:311 = the default glob (doc :232), tools/BUILD.bazel uses the default,
core/BUILD.bazel:7 explicit glob covers the new files — "no Bazel changes needed" holds.

Deferred optional items (implementer notes — no action this campaign; each is a 1-line
clerical polish, none test-observable, none affecting the operative rules):
1. §4.3 step 2 example phrase "line 1 column 9, the colon" fuses the 1-based/0-based
   bases (S-note/T-minor, same root): under the spec's 1-based convention column 9 is the
   key's closing quote (0-based 8); the colon is the next unconsumed byte at 0-based 9.
   The operative rule (offset = column-1; boundary offset+1 inclusive) is unambiguous
   under either reading — a future pass could reword to "line 1 column 9 — 0-based
   offset 8, the key's closing quote; the colon is the next unconsumed byte".
2. §7.2 shape-2 test line could be strengthened to "Eof hint present, **no `(parameter`
   suffix**" so the round-9 NOTE-2 ordering clause is test-enforced rather than
   review-enforced (T-note).
3. §7.7 line estimate "~36 funnel-site signature changes" could read "~37" to match the
   post-campaign count (T-note; the estimate is explicitly approximate).

**Stage-4 disposition:** §9 (the task breakdown) is cross-reviewed — it was audited in
every spec round (round-8 §7.3↔§9 one-to-one mapping; round-10 full count census + item-3
walkthrough on the final v8.2 text). Proceed to stage 5 (workflow-driven TDD, one item at
a time).
