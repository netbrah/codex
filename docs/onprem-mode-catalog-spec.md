# ONPREM-MODE-1 — On-prem model catalogue + Proactive delegation default (SDD)

**SDD status:** DRAFT v2.6 — v2.4 = R1+R2+R3+R4 seat rounds applied (R1: M 0B/1M/5m/4n · N 0B/1M/6m/5n; R2: M2 0B/1M/3m/4n · N2 0B/1M/5m/4n; R3: M3 0B/2M/2m/3n · N3 0B/0M/4m/2n; R4: M4 0B/0M/3m/4n · N4 0B/0M/0m/2n — round 4 = full 0B/0M, SDD CONVERGED; v2.4 = post-convergence polish of the 9 R4 minors/nits, no design change; v2.5 = post-gate coordinator adjudications from first-hand SCS wire evidence (§7.5 v2.5 notes: row-9 dump-verified rewording, rows 6/7 harness blind spot + option-B ruling, astra snapshot disposition), no design change — verdicts at `/tmp/onprem-r2-seatM-verdict.md`, `/tmp/onprem-r2-seatN-verdict.md`, `/tmp/onprem-r3-seatM-verdict.md`, `/tmp/onprem-r3-seatN-verdict.md`, `/tmp/onprem-r4-seatM-verdict.md`, `/tmp/onprem-r4-seatN-verdict.md`; v2.6 = item-3 FORK-MANIFEST re-derivation + spec fix rounds: R1 + R2 dual seats (R1: A 0B/2M/2m/3n · B 0B/0M/2m/1n; R2: A 0B/0M/1m/1n · B 0B/0M/0m/2n — both 0B/0M, CONVERGED), all 14 findings (2M/5m/6n) fixed; R2's 4 as coordinator micro-round, no third seat round (recorded deviation) — verdicts at `/tmp/xt29-item3-seatA-verdict.md`, `/tmp/xt29-item3-seatB-verdict.md`, `/tmp/xt29-item3-seatA-r2-verdict.md`, `/tmp/xt29-item3-seatB-r2-verdict.md`)
**Bead:** `apex-xt2.9` (central ledger `$HOME/Projects/bitbucket/apex_tracking`)
**Repo/branch:** `/Users/palanisd/Projects/upstream/codex` @ `feat/normalize-content-types-vllm` (post-merge; HEAD at spec time `7bcd344fa7`)
**Binary affected:** `codex-combined` / `codex-combined-v2` (fork build only; npm `codex` unaffected)

**Coordinator note (dirty tree at spec time):** uncommitted in-tree edits to `codex-rs/models-manager/src/manager_tests.rs` (unrelated `render_model_instructions` swap, pre-existing) and `codex-rs/tools/src/lib.rs` (in-flight apex-xt2.7). Pathspec-strict staging: item-1's commit must NOT swallow the in-flight hunks; T1a/T1b's RED phase runs against the modified working-tree test file. **Staging for `manager_tests.rs` MUST be hunk-level (`git add -p`)**: file-level `git add <path>` would swallow the dirty `render_model_instructions` hunk into the item-1 commit (both R2 seats).

---

## 1. Problem

Three operator-visible symptoms in the on-prem harness (`codex-combined-v2 -p qwen --yolo --enable multi_agent_v2`):

1. **Metadata warning on every launch:** ``⚠ Model metadata for `qwen3.8-27b` not found. Defaulting to fallback metadata; this can degrade performance and cause issues.`` (backticks around the slug per the real format string, `turn_context.rs:1155`)
2. **`--enable multi_agent_v2` required on every on-prem launch** (qwen, glm, grok alike) for multi-agent v2 to work.
3. **Explicit delegation policy at the on-prem default effort (xhigh):** the operator wants the **Proactive** delegation policy (or an option for it) for on-prem models; the built-in trigger for Proactive is the `Ultra` reasoning-effort tier, which on-prem deployments do not use.

Operator ruling (2026-09-17, verbatim intent):
- Add on-prem model data to the bundled catalogue (glm/grok precedent) — kills symptom 1.
- **No new flag in `config.toml`** (shared with the npm `codex` binary); instead **this binary defaults to Proactive**.
- Document everything; model-agnostic — no overfitting to glm/qwen.

## 2. Evidence (verified against source, spec-time HEAD)

### 2.1 Multi-agent version resolution chain

`Config::multi_agent_version_for_model` (`codex-rs/core/src/config/mod.rs:1557`):

```
multi_agent_version_override()            // :1537 — CLI/feature: --enable multi_agent_v2 => V2;
    .or(model_multi_agent_version)          //     agents_enabled=false => Disabled; else None
    .unwrap_or_else(from_features)          // catalog ModelInfo.multi_agent_version (via .or above);
                                           // from_features (:1547) re-checks override, then
                                           // Feature::Collab => V1, else Disabled
```

Catalog field: `ModelInfo.multi_agent_version: Option<MultiAgentVersion>` (`codex-rs/protocol/src/openai_models.rs:503`), sourced from the bundled `codex-rs/models-manager/models.json`. Fallback `ModelInfo` for unknown slugs sets it `None` (`codex-rs/models-manager/src/model_info.rs:147`).

### 2.2 Current catalogue state (post-merge HEAD)

| slug | multi_agent_version |
|---|---|
| gpt-6-astra, gpt-5.6-sol, gpt-5.6-terra, gpt-daybreak-blue-latest, gpt-daybreak-red-latest | `"v2"` |
| gpt-5.6-luna, codex-auto-review | `"v1"` |
| gpt-5.5, gpt-5.4 | `null` |
| **gpt-5.4-mini, gpt-5.2** (re-added by the fork in commit `799ec98053`; retired upstream in `eb7bd64ef9`) | `null` |
| **glm-5.2, grok-4.6** (fork-added, commit `799ec98053`) | `null` |
| **qwen3.8-27b** | **ABSENT from catalogue entirely** |

### 2.3 Key finding: qwen3.8-27b was never in the catalogue

- `git log --all -S 'qwen3.8-27b' -- codex-rs/models-manager/models.json` → **zero commits**. The slug never existed in this file's history (verified at `5789dd8500` = current production binary, and at HEAD).
- The FORK-MANIFEST line `models.json | +392L: glm-5.2 / qwen3.8-27b catalog entries` was therefore **stale/aspirational** — only glm-5.2 + grok-4.6 were real. This spec's docs item corrects the manifest.
- Consequence: `qwen3.8-27b` resolves via `model_info_from_slug` fallback (`model_info.rs:100`): `used_fallback_model_metadata: true` (warning check at `core/src/session/turn_context.rs:1151` — single-line `if` — format at `:1155–1156`), `visibility: None` (absent from the `/model` switcher), `supported_reasoning_levels: []` (no level validation — why profile `xhigh` works today), `multi_agent_version: None`.
- **glm-5.2 and grok-4.6 ARE in the catalogue (no warning) but have `multi_agent_version: null`** → they fall through to the feature fallback → they also require `--enable multi_agent_v2` today. Symptom 2 covers all three on-prem models.

### 2.4 Delegation mode selection

`effective_multi_agent_mode` (`codex-rs/core/src/session/multi_agents.rs`):

```
hint = config [features.multi_agent_v2].multi_agent_mode_hint_text
      OR catalogue MultiAgentMessages.mode.hint_text
if hint            => MultiAgentMode::Custom(hint)
else if effort == Ultra => Proactive   (catalogue mode.proactive text, else bundled;
                                         → Custom(text) if catalogue text override resolves —
                                         ResolvedMessage::Catalog, multi_agents.rs:109–111)
else                => ExplicitRequestOnly (catalogue mode.explicit text, else bundled;
                                             → Custom(text) if catalogue text override resolves —
                                             ResolvedMessage::Catalog, multi_agents.rs:109–111)
```

`effective_reasoning_effort` = config `model_reasoning_effort` OR catalogue `default_reasoning_level` (`core/src/session/step_settings.rs:87`). On-prem profiles pin non-Ultra efforts (qwen: xhigh; glm/grok: high) → Explicit today. `MultiAgentMode` variants: `ExplicitRequestOnly | Proactive | Custom(String)` (`codex-rs/protocol/src/config_types.rs`); rendered per turn by `MultiAgentModeInstructions` (`core/src/context/multi_agent_mode_instructions.rs`) as a developer fragment.

### 2.5 Profile mechanics (context)

- `-p <name>` = upstream profile-v2: loads `$CODEX_HOME/<name>.config.toml` as a config layer (`CONFIG_PROFILE_V2_SUFFIX = ".config.toml"`, `config/mod.rs:265`, `resolve_profile_v2_config_path` :1970). Not fork-specific.
- Operator profiles: `~/.codex/qwen.config.toml` (model `qwen3.8-27b`, effort `xhigh`, provider `llm_proxy_qwen` = LiteLLM proxy `/v1` untagged, `[agents.glm]` worker seat → `roles-qwen/glm.toml`), `glm.config.toml`, `grok.config.toml`. Base `~/.codex/config.toml` is shared with the npm `codex` binary (default model gpt-5.6-sol) — hence the no-new-flag ruling.

## 3. Design

Three items, one campaign, all small:

- **Item 1 — Catalogue data fix** (kills symptoms 1 and 2): add the `qwen3.8-27b` entry to the bundled `models.json`; set `multi_agent_version: "v2"` on all three on-prem entries. Pure data; no model-specific code; survives upstream pulls as one reviewed hunk (manifest already flags `models.json` as watch).
- **Item 2 — Fork-local Proactive default** (fixes symptom 3, per operator ruling): one documented fork-only seam function + a one-line selector change in `effective_multi_agent_mode`. No new config field, no new CLI flag, no protocol/TS surface, no new effort tier. The existing `multi_agent_mode_hint_text` (config or catalogue) remains the override/escape hatch and keeps full precedence.
- **Item 3 — Docs + manifest + verification tests**: correct the stale FORK-MANIFEST line, register the new fork seam, write the campaign doc with the *add-a-new-on-prem-model recipe* (the operator's "models change and I forget what we have to do" pain), and land the profile-layer/CLI merge verification test the operator required.

**Rejected alternatives (recorded for reviewers):**

- *A — paste Proactive text into `multi_agent_mode_hint_text` (zero code):* works today but forces manual policy-text duplication that drifts from upstream text evolution; operator wants a durable default, not a pasted string.
- *B — new config knob `delegation_mode: auto|explicit|proactive`:* originally recommended, **superseded by operator ruling** (no new flag in shared config). Would also have added schema/TS surface for one preference.
- *C — catalogue `default_mode` field:* protocol + TS + catalogue schema change; puts a deployment *preference* in the *capability* layer; the fork-default seam achieves the same outcome with strictly less surface.
- *D — set `model_reasoning_effort = "ultra"` in profiles:* Ultra is a real tier but wire-side (LiteLLM may reject unknown effort values on vLLM), TUI special-cases it (`effort_ignition.rs`, `app/model_defaults.rs`), and it couples effort to delegation policy — the exact overfit the operator flagged.

**Scope note:** the Proactive default applies to *all* sessions started by this binary (any model, any profile, including a bare `codex-combined-v2` on the base config's gpt-5.6-sol). This is intentional per the ruling (this binary is the on-prem workhorse; sessions are not cross-model resumable anyway). Reversal path if ever wanted: single function in `fork_defaults.rs` (or per-session `multi_agent_mode_hint_text`). Documented in the campaign doc and manifest.

## 4. Item 1 — Catalogue data (`codex-rs/models-manager/models.json`)

### 4.1 New entry: `qwen3.8-27b`

Shape: **byte-identical to the existing `glm-5.2` entry** (same field set, same order, appended after it or at the models-array end per file convention) with exactly these divergences:

| field | qwen3.8-27b value | rationale |
|---|---|---|
| `slug` | `"qwen3.8-27b"` | exact string the profile/model wire uses; must match `qwen.config.toml` |
| `display_name` | `"Qwen3.8-27B"` | `/model` switcher label |
| `description` | on-prem floor-model description (one line, e.g. "Qwen 3.8 27B — on-prem agentic coding model (vLLM via LLM proxy).") | switcher text |
| `default_reasoning_level` | `"xhigh"` | operator's on-prem default ("xhigh … default on our on-prem deployment"); profile pins it anyway |
| `supported_reasoning_levels` | `low`, `medium`, `high` (inherit glm descriptions) + **`xhigh`** with its own required `description` string | entries are `ReasoningEffortPreset { effort, description }` — `description` is REQUIRED (`protocol/src/openai_models.rs:196–201`); glm template is low/medium/high; `xhigh` added so the profile's effort is a first-class supported level (picker-honest) AND keeps spawn-agent effort validation green (`core/src/agent/child_config.rs:345–365`). **`ultra`/`max` deliberately excluded** — decouples effort tier from delegation policy (see §3-D) and changes the TUI picker (documented in §10) |
| `multi_agent_version` | `"v2"` | the fix: v2 active without `--enable` |
| `priority` | `52` | explicit picker slot (glm-5.2 is 50; sort is stable-by-priority only, `models-manager/src/manager.rs:154`) |
| `base_instructions` | **copy the exact `base_instructions` string from the `glm-5.2` entry** | consistency across the on-prem trio; the legacy top-level key is promoted to `model_messages.instructions_template` at deserialize (`openai_models.rs:767–806`; `From<ModelInfo> for ModelPreset` at :809) and its absence would ERROR — byte-identical copy is mandatory |
| all other fields | unchanged from glm-5.2 (`context_window`/`max_context_window` already `262144` in the template — no-op, `prefer_websockets: true`, `apply_patch_tool_type: "freeform"`, `web_search_tool_type: "text"`, image input modalities, `truncation_policy` bytes/10000, `supports_parallel_tool_calls: true`, `visibility: "list"`, `shell_type: "unified_exec"`, include_* usage instructions `true`, `supports_reasoning_sum*` `true`, …) | sibling shape; freeform is required by this branch's apply_patch seam |

`visibility: "list"` (inherited from the template) puts qwen in the `/model` switcher, matching glm/grok.

### 4.2 Existing on-prem entries

- `glm-5.2`: `"multi_agent_version": null` → `"v2"` (one-line value change).
- `grok-4.6`: `"multi_agent_version": null` → `"v2"` (one-line value change).

No other models.json changes. No GPT-catalogue entries touched.

### 4.3 Why this kills the symptoms

- `get_model_info("qwen3.8-27b")` now returns a catalogue `ModelInfo` with `used_fallback_model_metadata == false` → warning at `turn_context.rs:1151` no longer fires.
- `multi_agent_version_for_model(Some(V2))` with no CLI/feature override resolves `V2` → `--enable multi_agent_v2` drops from launch commands for all three on-prem models.

**Post-fix launch shape (acceptance):** `codex-combined-v2 -p qwen --yolo` (no `--enable`) ⇒ v2 active, no metadata warning, Proactive mode (Item 2).

## 5. Item 2 — Fork-local Proactive default

### 5.1 New fork-only module `codex-rs/core/src/fork_defaults.rs`

Location is PINNED: crate root (`mod fork_defaults;` in `codex-rs/core/src/lib.rs`, matching the flat-module style). No test module in this file (see §7.2 — the behavior test T2a is the seam ratchet; a static-value assertion would violate the repo's "no tests for statically defined values" rule and was dropped in R1).

```rust
//! Fork-local defaults for the codex-combined on-prem build (FORK-MANIFEST tracked).
//!
//! This file is a fork seam: it exists in this repository only and MUST NOT be
//! merged upstream. Operator ruling 2026-09-17: this binary is the on-prem
//! workhorse, and proactive multi-agent delegation is its default policy
//! (no new config flag; see docs/onprem-mode-catalog-spec.md).

/// Proactive delegation is the default mode in this build when no
/// `multi_agent_mode_hint_text` override is configured (any reasoning effort).
/// Upstream builds keep the effort-derived selection (Ultra => Proactive,
/// otherwise ExplicitRequestOnly).
pub(crate) fn proactive_delegation_default() -> bool {
    true
}
```


### 5.2 Selector change `core/src/session/multi_agents.rs` — `effective_multi_agent_mode`

Current (verbatim):

```rust
            let (message, builtin) =
                if settings.effective_reasoning_effort() == Some(ReasoningEffort::Ultra) {
                    (multi_agent_messages.proactive, MultiAgentMode::Proactive)
                } else {
```

Becomes (the ONLY product-code change in Item 2):

```rust
            // Fork default (codex-combined on-prem build): Proactive unless a
            // hint-text override is configured; upstream stays effort-derived.
            let (message, builtin) =
                if settings.effective_reasoning_effort() == Some(ReasoningEffort::Ultra)
                    || crate::fork_defaults::proactive_delegation_default()
                {
                    (multi_agent_messages.proactive, MultiAgentMode::Proactive)
                } else {
```

Precedence preserved: `hint_text` (config or catalogue) is evaluated **before** this branch and still wins (`MultiAgentMode::Custom`). Everything downstream (catalogue text overrides `mode.proactive`/`mode.explicit`, source gating, `MultiAgentModeInstructions` rendering) is untouched.

### 5.3 Behavior matrix (acceptance)

| v2? | hint text | effort | session source | mode |
|---|---|---|---|---|
| yes | none | xhigh (or any non-Ultra) | root/ThreadSpawn | **Proactive** (new) |
| yes | none | Ultra | root/ThreadSpawn | Proactive (unchanged) |
| yes | config or catalogue | any | root/ThreadSpawn | Custom(hint) (unchanged) |
| no (v1/disabled) | — | any | any | None (unchanged) |
| yes | any | any | Internal / non-ThreadSpawn SubAgent | None (unchanged) |

Session-source gate (verbatim, `multi_agents.rs:116–125`): mode emitted for `ThreadSpawn | Cli | VSCode | Exec | Mcp | Custom(_) | Unknown`; `None` for `Internal(_)` and other `SubAgent` sources.

No wire-format change (no new request fields; reasoning effort untouched). No rollout/resume schema change (mode is recomputed per turn from config + model info).

## 6. Item 3 — Docs, manifest, verification test

### 6.1 FORK-MANIFEST.md edits

1. **Fix the stale line** in "Fork modifications to shared files": `models.json | +392L: glm-5.2 / qwen3.8-27b catalog entries` → rewrite: glm-5.2 + grok-4.6 entries added by `799ec98053`; **qwen3.8-27b added by this campaign (apex-xt2.9)**; all three on-prem entries at `multi_agent_version: "v2"`. Keep the watch note (upstream owns this file).
2. **Register fork-only file:** add `codex-rs/core/src/fork_defaults.rs` to "Fork-only files → Code — MUST SURVIVE" (purpose: on-prem Proactive delegation default seam, apex-xt2.9). **Update counts (R4-corrected, M4 minor):** "50 added"→"52 added", "Code (4)"→"Code (5)", "Docs (46)"→"Docs (47)" (`FORK-MANIFEST.md:35,37,46`) — first-hand `git diff f3da3861c5..HEAD --name-status` (R4-M4): 52 A = 47 docs + 5 non-docs; excluding the manifest file itself → 51 fork-only files today = 4 code + 47 docs; after `fork_defaults.rs` lands → 52 = 5 code + 47 docs. The manifest's Docs section misses the 14 `docs/reviews/impl-item{1,2,3,4}-*.md` audit files (its wildcard covers only `impl-item5-*.md`) — extend the wildcard to `docs/reviews/impl-item*.md` (or list the 14 files).
3. **Register shared-file modifications** in the "we modified, upstream UNTOUCHED" table (count 17→22, `:61`; and bump the section heading `## Fork modifications to shared files (27)` at `FORK-MANIFEST.md:57` to (32) — derivation: 17 files first-hand-verified in the manifest table + `multi_agents.rs` + the four test files; supersedes the provisional 21/31 from the R2 handoff, which counted three test files before `step_settings.rs` was added to the inventory): `codex-rs/core/src/session/multi_agents.rs` (one-line fork-default disjunct in `effective_multi_agent_mode`, MUST SURVIVE; re-derive at ratchet if upstream reworks the selector) **and the four test files adjusted in Item 2** (§7.5: `core/tests/suite/multi_agent_mode.rs`, `core/tests/suite/subagent_notifications.rs`, `core/tests/suite/step_settings.rs`, `app-server/tests/suite/v2/turn_start.rs`) — each MUST SURVIVE / re-derive-at-ratchet (expectations encode fork behavior).
4. **Feature-level lines:** correct item 4 (`FORK-MANIFEST.md:93`, "glm-5.2 / qwen3.8-27b entries") to include grok-4.6 + the new `v2` status + this campaign's qwen addition; correct item 5 (`:95–98`, "multi_agent_v2 — UPSTREAM code … we only consume it. Keep it upstream-shaped") — FALSE after Item 2 lands the first fork edit to this surface; rewrite to name the fork seam + test-expectation edits.
5. Bump "Last verified" after the campaign commits (same convention as the 2026-09-17 ratchet block).

### 6.2 Campaign doc

This file (`docs/onprem-mode-catalog-spec.md`) serves as the campaign record. Section 9 adds the **add-a-new-on-prem-model recipe** (three steps, copy-paste) so the recurring "models change and I forget what we have to do" cost is eliminated.

### 6.3 Profile-layer × CLI merge verification (operator-required)

Even though this campaign adds **no** config flag, the spec must verify the layering the operator asked about: a `[features.multi_agent_v2]` table in a **profile layer** (`<profile>.config.toml`) merges cleanly with the CLI `--enable multi_agent_v2` — the feature stays enabled AND the profile-layer config fields (e.g. `multi_agent_mode_hint_text`) survive. Test lives with the existing `ConfigBuilder`/`config_tests.rs` patterns (T4 in §7.3). This also future-proofs the escape-hatch path.

## 7. Test plan

TDD per item: failing test first (RED), implement, GREEN, then gates. Integration/unit placement follows existing patterns; behaviors are pinned below, fixture choice is the implementer's (lightest existing harness first; note the choice in the commit body).

### 7.1 Item 1 tests (crate: `codex-models-manager`, file: `src/manager_tests.rs` or adjacent per existing convention)

- **T1a** `get_model_info("qwen3.8-27b", …)` → `used_fallback_model_metadata == false`, `multi_agent_version == Some(V2)`, `visibility == List`, `resolved_context_window() == Some(262144)`. (Deep-equals on the whole `ModelInfo` is preferred per repo rules; at minimum assert these discriminating fields plus equality of `apply_patch_tool_type`/`truncation_policy` against expected values.)
- **T1b** `get_model_info("glm-5.2")` / `("grok-4.6")` → `multi_agent_version == Some(V2)` (regression guard on the two value flips).
- **T1c** (core, config level): with NO feature enabled and NO CLI override, `Config::multi_agent_version_for_model(Some(MultiAgentVersion::V2))` → `V2` (proves `--enable` is no longer required when the catalogue says v2); and `multi_agent_version_for_model(None)` → `Disabled` (fallback chain unchanged for truly unknown models).

### 7.2 Item 2 tests (crate: `codex-core`)

Fixture: **no existing test drives the selection function directly** (R1-verified: the world-state unit tests construct `MultiAgentModeState` from enum values and test rendering only; full-pipeline coverage lives in `core/tests/suite/multi_agent_mode.rs` + the four `astra_*` scenario snapshots, §7.5). `effective_multi_agent_mode` is `pub(crate)` and pure over its inputs → **add a unit test** driving it via the existing seam constructors — `StepContext::for_test` (`core/src/session/tests.rs:251`) + `Session::make_turn_context` (`core/src/session/turn_context.rs:814`) (no minimal `StepContext` struct literal exists: 10 required `pub(crate)` fields — R2-verified). The fixture model MUST carry **bundled** (not catalogue-overridden) multi-agent messages so the expected result is the builtin enum + bundled text (a catalogue `mode.proactive`/`hint_text` override would resolve to `Custom` via `ResolvedMessage::Catalog`, `multi_agents.rs:108–110`).

Behaviors (the §5.3 matrix, one test per row minimum):
- **T2a** v2 + no hint + effort `xhigh` → `Some(MultiAgentMode::Proactive)` **and** the rendered `MultiAgentModeInstructions` body equals the bundled **proactive** text (proves the right *text* rides with the mode, not just the enum). Doubles as the fork-seam ratchet (fails if the default ever flips).
- **T2b** v2 + no hint + effort `Ultra` → Proactive (upstream path intact).
- **T2c** v2 + config `multi_agent_mode_hint_text = "H"` → `Custom("H")` (precedence preserved).
- **T2d** v2 + catalogue `mode.hint_text = "C"` → `Custom("C")`.
- **T2e** v1 → `None`; v2 + `SessionSource::Internal` → `None`; v2 + non-ThreadSpawn SubAgent → `None` (gating intact).
(T2f from v1 — static-value assertion on the seam constant — **dropped** in R1: violates "Do not add tests for values that are statically defined" and is redundant with T2a.)

### 7.3 Item 3 test (crate: `codex-core`, `config_tests.rs` or profile-layer test file)

- **T4** Profile-layer `[features.multi_agent_v2] { multi_agent_mode_hint_text = "…" }` + CLI `--enable multi_agent_v2` (via `ConfigBuilder` cli_overrides/harness path, following existing feature-enable test patterns) → loaded `Config`: feature `MultiAgentV2` enabled **and** `multi_agent_v2.multi_agent_mode_hint_text` preserved from the profile layer. (If the CLI `--enable` plumbing lands as a `features` cli-override, mirror however the existing `--enable` tests assert it — the invariant is *both survive*.)

### 7.4 End-to-end acceptance (post-merge, live binary — operator environment)

After the release build rotates to `codex-combined-v2`:
- `codex-combined-v2 -p qwen --yolo` (NO `--enable`): launches with v2 (spawn_agent available), **no** `⚠ Model metadata` warning, developer context carries the **Proactive** policy text; `/model` lists qwen3.8-27b alongside glm-5.2 / grok-4.6.
- Same for `-p glm` and `-p grok`.
- This is the item-6-style acceptance evidence for the cutover (Mac = live acceptance environment; SCS key tagging rules do not apply to on-prem profiles).

### 7.5 Item 2 pre-verified test-impact inventory (R1, first-hand)

The fork default flips non-Ultra, v2, no-hint selections from Explicit to Proactive. Verified breakage set (exhaustive grep for both policy texts across `codex-rs`):

**Hard-assertion integration tests (10 expectation rows spanning 9 test functions across 2 crates — `codex-core` rows 1–5/8–10, `codex-app-server` rows 6–7; rows 1–2 share `catalog_proactive_mode_is_ultra_only`; rows 8–9 fully parametrized — test-CODE edits required; v2.3: row/function/crate wording corrected per R3 (M3+N3); v2.2: extended from the R1 7-function inventory after R2 (seats M2+N2, independently) found the 3 functions missed below):**

| # | location | expectation today | under fork default |
|---|---|---|---|
| 1 | `core/tests/suite/multi_agent_mode.rs:232` test case "non ultra ignores proactive override" (High + catalogue proactive) | emits catalogue EXPLICIT text | emits catalogue PROACTIVE text |
| 2 | `:235` "empty proactive leaves non ultra unchanged" (High + empty proactive) | Explicit branch taken directly (catalogue `proactive = Some("")` never reaches the Proactive branch) → catalogue EXPLICIT text | no mode text emitted at all |
| 3 | `:284` model-switch case (High) | both catalogue explicit texts | proactive text(s) |
| 4 | `:483` `live_mode_change_appends_mode_without_reappending_usage_hint` | rollout records `["proactive","explicitRequestOnly"]`; second developer-text tuple `(ROOT_USAGE_HINT_TEXT, PROACTIVE_TEXT, NO_SPAWN_TEXT) == (1,1,1)` (:538) | `recorded_modes == [json!("proactive")]` — NO second mode record: the Proactive→Proactive High leg leaves the `multi_agent_mode` section unchanged (mode + `usage_hint_hash` equal), so `create_merge_patch` returns `None` for equal values (`core/src/context/world_state/mod.rs:497–499`) and `merge_patch_from` `None` for an empty patch (`:313–331`); the (:538) tuple flips `(1,1,1) → (1,1,0)` (turn-1 proactive block retained, no new block, no NO_SPAWN text) |
| 5 | `core/tests/suite/subagent_notifications.rs:1456` (High "explicit" turn ≈:1705; asserts ≈:1741, counts `(1,1,0,1)` at :1837) | explicit turn expectations | parent’s final world-state mode is Proactive (the High leg recomputes Proactive, no new block) and the child’s single mode block carries it — `:1837` tuple flips `(1,1,0,1) → (1,0,1,1)`; the ≈:1741 parent-turn assert (explicit-turn request contains `FULL_HISTORY_EXPLICIT_POLICY`) flips to a proactive-policy check |
| 6 | `app-server/tests/suite/v2/turn_start.rs:2730` `turn_start_ignores_deprecated_multi_agent_mode` | explicit text present (:2784) AND proactive absent (:2787–:2791, contains-check :2790) | both invert |
| 7 | `:2797` `thread_start_ignores_deprecated_multi_agent_mode` | reported `multi_agent_mode == ExplicitRequestOnly` (:2829) + explicit text (:2857) | reported field **UNCHANGED** — hardcoded deprecated constant in every production path (`thread_processor.rs:1620/:4134/:5327`, `thread_lifecycle.rs:760`, `thread_summary.rs:193`; protocol docs it `@deprecated Always explicitRequestOnly`, `v2/thread.rs:218`) so the `:2829` assert stays green; flip ONLY the text asserts (:2854–:2864) — explicit text absent, proactive text present |
| 8 | `core/tests/suite/step_settings.rs:1250` `active_model_switch_updates_multi_agent_policy_from_captured_effort` (parametrized: `Some(Ultra)` + `None` test_cases; v2 turn; catalogue `multi_agent = None`) | `assert!(!requests[0].body_contains_text(proactive_text))` at :1327 — requests[0] (MODEL_A non-Ultra leg) must NOT carry proactive text | flips: requests[0] carries the BUNDLED proactive text in both test_cases (non-Ultra leg is Proactive under the fork default) |
| 9 | `core/tests/suite/step_settings.rs:1006` `active_model_switch_updates_core_context_from_captured_settings` (parametrized: 6 `TokenBudgetScenario` variants; catalogue `multi_agent` with `explicit: Some("Delegation policy for {slug}.")`, `proactive: None` — set at :1078–:1088, load-bearing `proactive: None` at :1085) | asserts `Delegation policy for {MODEL_A}.` (requests[0], :1162) and `Delegation policy for {MODEL_B}.` (switch developer texts, :1194) | **[v2.5, dump-verified 2026-09-19, `xt29-row9dump.log` first-hand]:** the model switch re-emits the FULL `<multi_agent_mode>` block (usage_hint_hash change; append-only latest-wins). In requests[1] (the switch request) the developer messages carry the bundled proactive text exactly **twice** (dev[5] + dev[11]), BOTH inside `<multi_agent_mode>`, ZERO inside `<model_switch>` (dev[6] carries the switch instructions only; the other section texts ride the settings re-bundle). Final assert (SCS-green, `xt29-row9verify.log` 6/6): `proactive_messages.len() == 2` + each occurrence inside `<multi_agent_mode>` and NOT inside `<model_switch>`; plus the row-8 flip (non-Ultra MODEL_A leg carries the bundled proactive text). Catalogue `proactive: None` → bundled fallback confirmed. |
| 10 | `core/tests/suite/multi_agent_mode.rs:564` `leaving_ultra_after_cold_resume_emits_explicit_mode` | assert `(open_tag, NO_SPAWN_TEXT, PROACTIVE_TEXT) == (2, 1, 1)` at :609–616 (the post-resume High leg emits the explicit-mode block) | coordinator-derived: no mode-change block on the High leg (Proactive→Proactive) → expected `(1, 0, 1)`; re-derive at TDD |

**Insta snapshots (4 files — flip verified by inspection):** `core/tests/suite/snapshots/all__suite__scenarios__astra_{disabled_executor_skills,kickoff_remote_compaction_windows,settings_release_check_tool_shapes,plugin_refresh}.snap` — contain the bundled Explicit text (1/2/1/2 occurrences) from full-pipeline astra (v2, non-Ultra) turns; flip to Proactive text. **[v2.5: the original "intentional `insta` acceptance" is SUPERSEDED — see v2.5 note 2 (astra snapshot disposition) below.]**

**Verified NOT at risk:** world-state unit snapshot tests (`core/src/context/world_state/multi_agent_mode_tests.rs` + its `.snap`) — build state from enum values, test rendering only; `search_tool.rs:930` + `spawn_agent_description.rs:259` — assert the STATIC `spawn_agent` tool description (the explicit rule at `multi_agents_spec.rs:702` is unconditional in the template; the proactive mode message overrides it at runtime by design); `multi_agents_spec.rs` itself (static template source); `core/src/agent/control_tests.rs` — `"Proactive multi-agent delegation is active."` — fixture strings at :1842/:2190 in history-forking tests (`spawn_agent_can_fork_parent_thread_history_with_sanitized_items` :1750, `spawn_agent_fork_strips_parent_usage_hints_from_compacted_history` :2107) + strip-assertion at :2318 (`assert!(!history_contains_text(…, “Proactive multi-agent delegation is active.”))` — stays green: the fork strip at `core/src/agent/control/spawn.rs:111` (`retain_forked_developer_message`) is unconditional and mode-independent), NOT mode-selection — unaffected (R2 both seats; R3 M3); `prompts/src/model_messages/multi_agent.rs` — the bundled-text definitions themselves (:46 `EXPLICIT_REQUEST_ONLY_MULTI_AGENT_MODE_TEXT`, :47 `PROACTIVE_MULTI_AGENT_MODE_TEXT`); the text source, not a test — cannot break (R3 N3).

**Remediation policy (up front):** adjust the 10 expectation rows (9 test functions) to fork behavior with a one-line comment per hunk pointing at this spec §7.5; accept the 4 snapshot updates **[v2.5: SUPERSEDED — excluded from the item-2 commit; one-pass rebaseline of all 6 pending snapshots queued to apex-xt2.13; see v2.5 note 2]**; all four test files registered in FORK-MANIFEST (§6.1.3); commit body lists the full inventory. These edits are the fork's intentional behavior, not regressions.

**v2.5 notes (post-gate coordinator adjudications, 2026-09-19 — first-hand SCS wire evidence; no seat round: these resolve gate findings, not design):**

1. **Rows 6/7 (6a HARD STOP, resolved = option B).** The SCS wire dump (`xt29-appdbg6a3.log`, run 3) proves the proactive text IS model-visible on the wire in the v2 flow: input_len=5, in[2] raw = the full two-paragraph `<multi_agent_mode>` developer message — yet `message_input_texts("developer")` returned `[]` (dev_count=0). Root cause = FORK-LOCAL wire normalization: for non-OpenAI providers `normalize_content_types: !info.is_openai()` (`provider.rs:399`; the app-server test's `MockResponsesConfig` is a non-OpenAI provider) routes the body through `normalize_content_type_strings` + `translate_agent_message_items` (`codex-api/endpoint/responses.rs:87`, `content_type_compat.rs`), which re-labels `input_text` spans as `"text"`; the upstream-identical helper `message_input_texts` (`core/tests/common/responses.rs:223`) matches `input_text` spans ONLY → positive asserts fail on the empty list, negative asserts pass vacuously. Parent was red on both tests too (static proof: helper + normalization identical on parent; item 2 touches neither) — the earlier "flip the text asserts" expectation change chased a phantom. **Ruling: option (B)** — test-local extraction in the two turn_start tests only (turn_start.rs is already in the item-2 change set): collect developer-message text spans of BOTH `"text"` and `"input_text"` types; negative asserts run against the same correctly-extracted set (no longer vacuous — strictly stronger); test comment cites this normalization layer + apex-xt2.13. **Option (A)** (widen the shared helper ~1 line) is DEFERRED to apex-xt2.13: it also flips the 19 no-panic "model visible" tests and the 17 `no_history`/`cold_resume` predicate failures (same class — `translate_agent_message_items` converts `agent_message` wire items to plain `message`/`user` items, breaking their `inputs_of_type("agent_message")` find-predicate), and it edits an upstream-identical shared file (ratchet surface). The product flow is CORRECT — the rows 6/7 expectations are TRUE per the wire.
2. **Astra snapshot disposition (deviation from the original "accept the 4" remediation, with rationale).** Flip verified FIRST-HAND by byte-diff (SCS parent worktree vs wip `.snap.new`, 2026-09-19): kickoff 2 hunks (L126, L310) + plugin_refresh 2 hunks (L113, L227), each −1 explicit line → +3 proactive lines (two-paragraph text + blank), nothing else; wip occurrence counts (disabled=1, kickoff=2, settings=1, plugin=2) match this table's baseline (1/2/1/2) exactly, explicit=0 in all 4. The parent astra run established kickoff + plugin_refresh were ALREADY red pre-item-2 (ENV drift: SCS skills-catalog state differs from the committed `.snap` baseline — 5 → ~43 skills, already drifted once); disabled + settings passed on parent (their `.snap.new` = baseline + flip only). The checker script (`xt29_astra_diff_check.py`) rc=1 = FALSE NEGATIVE: its 1:1 flip-line model ignores the 1→3 line expansion, and its base diff is confounded by the ENV drift. **Disposition: the 4 astra `.snap.new` (plus the 2 parent-only `all__suite__compact__*` `.snap.new`) are EXCLUDED from the item-2 commit** — accepting kickoff/plugin_refresh would bake SCS-env catalog state into committed snapshots (recurring red + re-accept churn; ratchet-conflict magnet for upstream pulls). The astra gate PASSES BY INSPECTION (byte-diff + counts above). One-pass snapshot rebaseline of all 6 pending snapshots in a pinned env state is queued to apex-xt2.13 (test-expectation fixes / clean full-suite baseline before the release build).
3. **Gate data on file (SCS; first-hand tails re-verified by coordinator):** scoped 96-row gate = 92 pass / 4 fail (the 4 = pre-existing fork-red, xt2.13 class; `xt29-scoped-final.log`); gate-1 rerun (pre-row-9-fix) 86/10; parent baseline 79/17 ×2 (one invalid wip-scoped-clean 79/17 discarded, do-not-cite); gate-3 (app-server) 1298/79 = 2 (6a) + 17 (6b predicate class) + 60 (6c pre-existing, incl. the 19 model-visible class); gate-4 core full 1936/69 + responses_headers 4/1 (the 1 = SCS git-mirror URL ENV, not code); clippy = env waiver accepted (no clippy component on air-gapped SCS; repo deny-lints enforced by CI; `just fmt` green). 88/4 → 92/4 arithmetic correction accepted (coordinator prediction was 92/4).

## 8. Gate & build plan (load discipline: charlie mike)

Sequencing (hard rule: no concurrent builds during gate runs; `CARGO_BUILD_JOBS=2` everywhere on the Mac):

1. **Wait** for the apex-xt2.8 re-run-3 quiet window to finish (auto-launcher `/tmp/xt28-rerun3-autolaunch.log`; result in `/tmp/xt28-rerun3-result.txt`) — it re-establishes the post-merge baseline the campaign builds on.
2. **Then** the staged apex-xt2.7 item-1 TDD (strict_int) gets its GO (already queued; single crate `codex-tools`, short build).
3. **Then this campaign**, item by item (TDD red→green, one item at a time, review loop to 0B/0M between items):
   - Item 1: `just test -p codex-models-manager` (fast crate) + T1c in `just test -p codex-core --lib` scoped run.
   - Item 2: new unit test (RED pre-impl) → impl → GREEN; then the pre-verified §7.5 set: `just test -p codex-core` (the 8 core expectation-row adjustments spanning 7 test functions — `multi_agent_mode.rs` ×5 rows / 4 fns, `subagent_notifications.rs` ×1, `step_settings.rs` ×2 — + the 4 intentional `astra_*` snapshot acceptances) **AND `just test -p codex-app-server`** (the 2 `turn_start` adjustments) — the app-server crate gate was missing from v1 (R1).
   - Item 3: docs-only + T4 scoped run.
   - Gates per repo rules: `just fmt` (auto, no approval), `just fix -p codex-core`, `just fix -p codex-app-server`, and `just fix -p codex-models-manager` (no re-run tests after fix/fmt).
   - Full workspace `just test` is NOT run without asking the operator (changes touch core; the re-run-3 pack is the merge baseline; operator decides at push time).
4. **Push pipeline** (after gates green): commit(s) `<area>: <description> (apex-xt2.9)`, pathspec-strict; push BOTH remotes (`git push https://github.com/netbrah/codex.git feat/normalize-content-types-vllm` + `git push fork feat/normalize-content-types-vllm`; NEVER origin); FORK-MANIFEST "Last verified" bump; SCS ff-sync (`ssh scs3 'git -C /x/eng/ai_engineering/APEX/codex fetch && git -C /x/eng/ai_engineering/APEX/codex merge --ff-only feat/normalize-content-types-vllm'`); bead `apex-xt2.9` closed with evidence.

## 9. Recipe — adding a new on-prem model (operator maintenance path)

When an on-prem model appears/disappears (vLLM host changes, new model version), exactly three steps:

1. **Catalogue entry** in `codex-rs/models-manager/models.json`: copy the `glm-5.2` block, change `slug` (must match the profile's `model` string exactly), `display_name`, `description`, `context_window`/`max_context_window` (the model's real context), `default_reasoning_level` + `supported_reasoning_levels` (the deployment's default effort), keep `multi_agent_version: "v2"` and `apply_patch_tool_type: "freeform"`, keep `base_instructions` byte-identical to the sibling entries. Rebuild.
2. **Profile file** `~/.codex/<name>.config.toml`: `model = "<slug>"`, `model_provider = "llm_proxy_<name>"` (untagged `/v1` base URL unless the proxy needs tags), `model_reasoning_effort = "xhigh"` (or the deployment default), `approval_policy = "never"`, `web_search = "disabled"` — plus a `[model_providers.llm_proxy_<name>]` block (copy the sibling profile's block: `name`, `base_url`, `env_key`, `supports_websockets = false`); custom providers resolve only from config, never from the built-in catalog.
3. **Launch:** `codex-combined-v2 -p <name> --yolo` — **no** `--enable multi_agent_v2` needed (catalogue says v2), **no** metadata warning (catalogue has the entry), Proactive delegation by default (fork seam).

If a model's metadata must stay out of the shared binary (never expected for on-prem — this binary IS the on-prem harness), the alternative is a per-profile `model_catalog_json` override (`$CODEX_HOME/<file>.json`, full `ModelsResponse` shape) — not needed for the current trio.

## 10. Risks & audit

- **`models.json` is upstream-owned:** next pull ratchet may conflict on our entries (manifest watch note already exists; strategy: re-apply the three on-prem entries as one reviewed hunk). The qwen entry's `base_instructions` blob is the largest diff chunk — keep it byte-identical to the sibling entries so re-application is mechanical.
- **Selector line in `multi_agents.rs`:** if upstream reworks `effective_multi_agent_mode` (new tiers, new selection inputs), the fork disjunct must be re-derived at ratchet (manifest entry says MUST SURVIVE + re-derive).
- **Test-impact drift (Item 2) — pre-verified in §7.5:** 10 expectation rows spanning 9 test functions (core rows ×8, app-server rows ×2) + 4 `all__suite__scenarios__astra_*` insta snapshots flip (flip verified by inspection — [v2.5: snapshot acceptance deferred to the apex-xt2.13 one-pass rebaseline, §7.5 v2.5 note 2]); all intentional, all registered in FORK-MANIFEST, all listed in the item-2 commit body. World-state unit tests and the `spawn_agent` description tests are verified NOT at risk.
- **One-shot mode-change on resume:** resuming a pre-campaign on-prem session (rollout recorded `explicit`) recomputes Proactive on the next turn and emits a one-shot explicit→proactive mode-change developer message (world-state diff, cf. the `Known(&explicit) -> Known(&proactive)` snapshot case). Operator-visible once per resumed session; documented, accepted.
- **TUI picker delta (Item 1):** qwen's effort popup changes from fallback behavior (empty `supported_reasoning_levels` → the `[]` arm) to exactly the four catalog levels low/medium/high/xhigh (`ModelPreset.supported_reasoning_efforts` via `From<ModelInfo>`, `openai_models.rs:809`; consumed at `tui/src/chatwidget/model_popups.rs:243`). TDD verifies and records the exact before/after popup state in the commit body.
- **Proactive for non-on-prem sessions in this binary** (e.g. bare launch on gpt-5.6-sol): accepted per operator ruling; escape hatch = `multi_agent_mode_hint_text`; reversal = one function.
- **No wire/resume SCHEMA change:** no new request fields, no rollout schema change, no protocol/TS export change (the seam is `pub(crate)` in `codex-core` only); the only resume-visible effect is the one-shot mode-change above.
- **RPC clients cannot observe the new default:** `thread/start|resume|fork` (and settings) responses keep reporting the documented constant `multiAgentMode = explicitRequestOnly` in this build — hardcoded in every production path (`v2/thread.rs:218/:319/:468/:662`) — so IDE/app-server clients of `codex-combined-v2` see `explicitRequestOnly` while the session runs Proactive; the true policy is observable only in the model’s developer context (R3 M3).
- **Debris noted (out of scope):** `~/.codex/glm-model-catalog.json` (single glm-5.2 entry, **no** `multi_agent_version` key — resolves `None`; coordinator R4-verified first-hand) is referenced by no profile — leftover from an earlier iteration; operator may delete. Not touched by this campaign.

## 11. Acceptance criteria (bead mirror)

1. `models.json`: qwen3.8-27b entry per §4.1; `multi_agent_version: "v2"` on qwen3.8-27b, glm-5.2, grok-4.6. T1a–T1c green.
2. `fork_defaults.rs` + one-line selector change per §5; T2a–T2e green; no other product-code changes in Item 2.
3. Zero new config/CLI/protocol surface (verify: `git diff` shows no additions to `features/feature_configs.rs`, `MultiAgentV2Config`, `MultiAgentV2ConfigToml`, app-server protocol, or CLI flag parsing).
4. T4 (profile-layer × CLI merge) green.
5. FORK-MANIFEST updated per §6.1; this doc committed as campaign record.
6. Gates per §8 green (fmt, fix, scoped project tests); full-suite decision with operator.
7. Pushed to both remotes + SCS ff-sync; bead closed with evidence (commit SHAs, gate logs, acceptance notes).

## 12. Review log

| round | date (UTC) | seat(s) | findings (B/M/m/n) | resolution |
|---|---|---|---|---|
| R1 | 2026-09-17 | seat M (counterfactual) + seat N (concordance), fresh, parallel, READ-ONLY | M: 0B/1M/5m/4n · N: 0B/1M/6m/5n | all 22 findings applied in v2 — per-finding rows below |
| R2 | 2026-09-17 | fresh dual seats (M2 counterfactual, N2 concordance), parallel, READ-ONLY | M2: 0B/1M/3m/4n · N2: 0B/1M/5m/4n | all 13 findings applied in v2.2 — per-finding rows 23–35 below |
| R3 | 2026-09-17 | M3 0B/2M/2m/3n · N3 0B/0M/4m/2n (fresh dual seats, READ-ONLY) | all 13 applied in v2.3 (rows 36–48) | both M3 Majors coordinator-adjudicated from source (rows 4, 7); N3 row-4 "verified clean" re-derivation adjudicated WRONG (see R3 paragraph) |
| item-3 R1+R2 | 2026-09-19 | dual seats A + B per round, parallel, READ-ONLY (FORK-MANIFEST re-derivation @ `10d14858c7` + this spec) | R1: A 0B/2M/2m/3n · B 0B/0M/2m/1n (merged 10); R2: A 0B/0M/1m/1n · B 0B/0M/0m/2n — both 0B/0M, CONVERGED | all 14 findings (2M/5m/6n) fixed in v2.6; R2's 4 as coordinator micro-round, no third seat round (recorded deviation); verdicts at `/tmp/xt29-item3-seatA-verdict.md`, `/tmp/xt29-item3-seatB-verdict.md`, `/tmp/xt29-item3-seatA-r2-verdict.md`, `/tmp/xt29-item3-seatB-r2-verdict.md` |

**R1 per-finding log** (findings + resolutions, per repo SDD rules; full verdicts preserved at `/tmp/onprem-r1-seatM-verdict.md` and `/tmp/onprem-r1-seatN-verdict.md`, recovered from seat rollouts `01a0b0c5-69c8…` / `01a0b0c5-b60f…`):

| # | seat | class | finding (abridged) | resolution in v2 |
|---|---|---|---|---|
| 1 | M | M | §7.2/§8/§10: the fork default breaks **7 hard-assert integration cases** — core: `multi_agent_mode.rs:232` / `:235` / `:284` / `:483`, `subagent_notifications.rs:1456`; app-server: `turn_start.rs:2730` / `:2797` — none named by v1; spec cited nonexistent `multi_agents_tests.rs` and pointed at the world-state unit tests (NOT affected); §8 missed the app-server crate gate | §7.5 full inventory + remediation policy (expectation edits w/ one-line comment per hunk, snapshot accepts, manifest registration, commit-body inventory); §8 item 2 adds `just test -p codex-app-server`; §6.1.3 registers the 3 test files; §10 risk line points at §7.5. Coordinator re-verified `multi_agent_mode.rs:232/:235` directly and `turn_start.rs:2730` exists. **[v2.2: the "7 hard-assert" inventory was NOT exhaustive — R2 (both seats, independently) found 3 more functions; corrected total = 10 functions + 4 snapshots, rows 23–35]** |
| 2 | N | M | §10 names the wrong at-risk test (world-state unit test constructs `MultiAgentModeState` directly — rendering only, NOT affected) and misses the **4 suite snapshots** that DO flip: `astra_{disabled_executor_skills,kickoff_remote_compaction_windows,settings_release_check_tool_shapes,plugin_refresh}.snap`; §8 contradicted §10 | §7.5 names the 4 snapshots (coordinator grep-verified at HEAD: bundled Explicit text 1/2/1/2 occurrences, zero Proactive → they flip); §10 rewritten; §8 item 2 includes the 4 intentional `insta` accepts |
| 3 | M | m | §5.1 inline `#[cfg(test)]` module violates the test-module rule; T2f asserts a statically-defined value | §5.1: no test module in `fork_defaults.rs`; T2f dropped (T2a is the seam ratchet) — same class as #9, #11 |
| 4 | M | m | §6.1 under-registered: manifest :93 (item 4) omits grok-4.6 + the new `v2` status; :97–99 (item 5 "UPSTREAM code … we only consume it") false after item 2 lands the first fork edit to this surface | §6.1.4 corrects both feature-level lines |
| 5 | M | m | dirty working tree at spec time (`manager_tests.rs`, `tools/src/lib.rs` in-flight) unstaged for the campaign | header coordinator note: pathspec-strict staging; T1a/T1b RED runs against the modified working-tree test file |
| 6 | M | m | §10 missed the one-shot explicit→proactive mode-change developer message when resuming a pre-campaign session | §10 risk bullet (operator-visible once per resumed session; documented, accepted) |
| 7 | M | m | §4.1/§10: TUI effort-picker delta for qwen undocumented (4 catalog levels replace the `[]` fallback arm) | §10 risk bullet: exact before/after popup state recorded in the item-1 commit body |
| 8 | N | m | §2.1 inline comments misattribute which line consumes the catalogue (`multi_agent_version_override` vs `.or` vs `from_features`) | §2.1 comments realigned |
| 9 | N | m | §7.2 "known consumers" inaccurate: NO existing test drives the selection function directly | §7.2: R1-verified → add a unit test (minimal `StepContext` fixture, bundled multi-agent messages) |
| 10 | N | m | §6.1 manifest counts + feature-level seam line missing (51 added / Code (5) / 18 modified; :93 item 4; new seam item) | §6.1.2 / §6.1.4 |
| 11 | N | m | §5.1 inline test module (AGENTS.md `#[path]` sibling-file convention) | §5.1 — same as #3 |
| 12 | N | m | T2a fixture could resolve `ResolvedMessage::Catalog` → `Custom` (`multi_agents.rs:108–110`) instead of Proactive | §7.2: fixture model MUST carry bundled (not catalogue-overridden) multi-agent messages |
| 13 | N | m | T2f borderline violates "no tests for static values" | dropped — same as #3 |
| 14 | M | n | §2.3 cites `model_info_from_slug` at `model_info.rs:97` (fn at :100) | §2.3 cite fixed (shared with #18) |
| 15 | M | n | §4.1 lists 262144 as a divergence (no-op — glm template already 262144); `priority: 50` ties glm-5.2 | §4.1: context row moved to the "no-op" column; `priority: 52` (explicit slot) |
| 16 | M | n | `supported_reasoning_levels` entries are `ReasoningEffortPreset { effort, description }` with REQUIRED description; v1 listed effort names only | §4.1 row: xhigh carries its own required `description` (others inherit glm's) |
| 17 | M | n | §5.1/§5.2 placement ambiguity (crate root vs `session/`) broke the `crate::fork_defaults::` path verbatim | §5.1 location PINNED: crate root |
| 18 | N | n | `model_info.rs:97` cite (same as #14) | fixed |
| 19 | N | n | §2.3 warning cite `turn_context.rs:1153` (check at :1151, format at :1155) | §2.3 range cite `:1150–1153` + format `:1155` |
| 20 | N | n | §12 one-row-per-round less auditable than the xt2.7 per-finding precedent | this section: per-finding rows |
| 21 | N | n | §5.3 "root/ThreadSpawn" understates the session-source gate | §5.3: gate stated verbatim under the matrix (`ThreadSpawn | Cli | VSCode | Exec | Mcp | Custom | Unknown`; `None` for `Internal`/other-SubAgent) |
| 22 | N | n | §1 symptom 1 missing backticks around the slug (real format string uses `` `{}` ``) | §1 fixed (double-backtick code span) |
| 23 | M2+N2 | M | **§7.5 inventory not exhaustive — 3 missed breakage functions** (both seats found `step_settings.rs` independently; coordinator added the third): `step_settings.rs:1250` `active_model_switch_updates_multi_agent_policy_from_captured_effort` (both test_cases; `!body_contains_text(proactive_text)` at :1327 flips), `step_settings.rs:1006` `active_model_switch_updates_core_context_from_captured_settings` (all 6 `TokenBudgetScenario` variants; delegation-text asserts at :1162/:1194 flip to bundled proactive), `multi_agent_mode.rs:564` `leaving_ultra_after_cold_resume_emits_explicit_mode` (tuple assert at :610–617 flips; coordinator-derived expected `(1,0,1)`) | §7.5 v2.2: 10-function table (rows 8–10 added); §7.5 header, §8, §10, remediation count updated; §12 row 1 marked superseded (also closes N2’s 4th nit: the row-1 “7 hard-assert” completeness claim) |
| 24 | N2 | m | §6.1 manifest line pins wrong: "50 added" :35 (spec :33), "Code (4)" :37 (spec :35), "17 we modified" heading :61 (spec :53), item 5 :95 (spec :97–99) | §6.1.2 `(:35,:37)`; §6.1.3 `:61`; §6.1.4 `:95–98` |
| 25 | M2+N2 | m | §6.1 staging guidance insufficient: file-level `git add` of `manager_tests.rs` swallows the dirty `render_model_instructions` hunk | header coordinator note: hunk-level `git add -p` mandated (both R2 seats) |
| 26 | M2 | m | §7.2 "minimal `StepContext`" fixture: no minimal struct literal exists (10 required `pub(crate)` fields) | §7.2 names the real seam: `StepContext::for_test` (`core/src/session/tests.rs:250`) + `Session::make_turn_context` (`turn_context.rs:814`) |
| 27 | M2 | n | §7.5 row 2 "expectation today" mechanism imprecise | row 2: "Explicit branch taken directly (catalogue `proactive = Some(\"\")` never reaches the Proactive branch)" |
| 28 | coordinator | m | manifest "17 we modified" count + heading math: v2 said 17→18; R2 handoff provisional 17→21/(27)→(31) counted three test files | §6.1.3 v2.2: 17→22 + heading (27)→(32) — 17 files first-hand-verified in the manifest table + `multi_agents.rs` + the four test files (v2.2's 4th = `step_settings.rs`) |
| 29 | N2 | n | §5.3 gate range :115–122 incomplete (gate is :116–125 incl. `Unknown => Some(...)` arm) | §5.3: `:116–125` |
| 30 | M2+N2 | n | §2.4 pseudocode omits Catalog→Custom override resolution (`multi_agents.rs:109–111`) | §2.4: both arms annotated `(→ Custom(text) if catalogue text override resolves)` |
| 31 | M2+N2 | n | §4.1 `openai_models.rs:753–805` range starts mid-function (fn at :767) | §4.1: `:767–806`; §10 companion pin `:810`→`:809` (`From<ModelInfo> for ModelPreset`) |
| 32 | N2 | m | `control_tests.rs` (proactive text at :1842/:2190/:2318) absent from inventory even though non-breaking | §7.5 "Verified NOT at risk" += `control_tests.rs` fixture line (history-forking fixture data, not mode-selection) |
| 33 | N2 | m | §7.5 row 5 counts tuple at :1816 wrong (actual :1837) | row 5: `:1837` |
| 34 | N2 | n | §2.3 warning-check range loose (`:1150–1153`; `if` is single-line at :1151) | §2.3: check at `:1151`, format `:1155–1156` |
| 35 | coordinator | n | §11 item 2 cites dropped test T2f | §11: "T2a–T2f" → "T2a–T2e" (coordinator-found in the v2.2 pass) |
| 36 | M3 | M | row 7: reported `multi_agent_mode` field is a hardcoded deprecated constant (`thread_processor.rs:1620/:4134/:5327`, `thread_lifecycle.rs:760`; `v2/thread.rs:218`) — the `:2829` assert stays green; an implementer following the v2.2 row would have chased a protocol change | row 7 rewritten: reported field UNCHANGED; flip ONLY the text asserts |
| 37 | M3 | M | row 4: `["proactive","proactive"]` unreachable — `create_merge_patch` None for equal values (`:497–499`), `merge_patch_from` None for empty patch (`:313–331`) → no second record | row 4: `recorded_modes == [json!("proactive")]` + tuple `(1,1,1)→(1,1,0)` |
| 38 | M3 | m | row 5: missing the exact post-fork tuple + the `:1741` flip | row 5: `(1,1,0,1)→(1,0,1,1)` + `:1741` proactive-policy check |
| 39 | M3 | m | §10/§6.2: RPC clients cannot observe the new default (`multiAgentMode` keeps reporting the documented constant) | §10 RPC-visibility bullet added |
| 40 | M3 | n | row 9 pin `:1078–1084` excludes load-bearing `:1085` | row 9 pin → `:1078–1088` |
| 41 | M3 | n | `control_tests.rs:2318` is a strip-assertion, not fixture data | "Verified NOT at risk" entry reworded (fixtures :1842/:2190 + strip-assert :2318, mode-independent) |
| 42 | M3 | n | "10 test functions" conflates cases with functions (9 unique) | §7.5 header/§8/§10 wording → "10 expectation rows spanning 9 test functions" |
| 43 | N3 | m | §7.5 header: 10 rows = 9 unique functions, 2 crates (not 3), "2 parametrized" true only of rows 8–9 | header corrected: rows 1–2 share `catalog_proactive_mode_is_ultra_only`; rows 8–9 fully parametrized |
| 44 | N3 | m | remediation line stale "all three test files" (§6.1.3 registers four) | remediation line → "all four test files" |
| 45 | N3 | m | §8 gate list missing `just fix -p codex-app-server` | added to §8 gates |
| 46 | N3 | m | "Verified NOT at risk" omits bundled-text definition sites | added: `prompts/src/model_messages/multi_agent.rs` :46/:47 (text source, cannot break) |
| 47 | N3 | n | five 1–3-line pin drifts | corrected: `:2787–:2791`, `:1078–:1085`, `:609–:616`, `:251`, `:345–:365` |
| 48 | N3 | n | §12 R2-log row 32 seat misattributed (M2→N2); row 23 folds N2's row-1 nit without attribution | row 32 relabeled N2; row 23 attribution note added |
| R4 | 2026-09-17 | M4 0B/0M/3m/4n · N4 0B/0M/0m/2n (fresh dual seats, READ-ONLY, on the v2.3 artifact) | all 9 applied in v2.4 (rows 49–57) | **full round 0B/0M = SDD CONVERGENCE; no R5** |
| 49 | M4 | m | row 7 site list missing fifth production site `thread_summary.rs:193` (field doc `v2/thread.rs:319`) | row 7 site list += `thread_summary.rs:193` |
| 50 | M4 | m | §6.1.2 manifest math: 51 = 5+46 wrong on both numbers (52 A = 47 docs + 5 non-docs, first-hand `git diff f3da3861c5..HEAD --name-status`) | §6.1.2 → 52 added / Code (5) / Docs (47) + wildcard extend to `docs/reviews/impl-item*.md` |
| 51 | M4 | m | §12 R3 round row still "pending" + R3 per-finding rows missing | R3 row rewritten; rows 36–48 added (M4's own "11 findings / 36–46" ref is a miscount — the archived R3 verdicts enumerate 13) |
| 52 | M4 | n | row 7 text-assert pin `:2857–:2871` overshoots into the next test fn | pin → `:2854–:2864` |
| 53 | M4 | n | `merge_patch_from` pin `:313–332` (fn ends `:331`) | → `:313–331` (row 4 cell + §12 R3 paragraph) |
| 54 | M4 | n | §10 debris bullet "`mav: null`" — the key is ABSENT, not null | → "no `multi_agent_version` key (resolves `None`)" (coordinator re-verified first-hand: `python3`/json read) |
| 55 | M4 | n | remediation "10 expectations (functions)" conflates rows/functions | → "10 expectation rows (9 test functions)" |
| 56 | N4 | n | row 7 pin `:2857–:2871` loose (same class as row 52; both seats concur) | pin → `:2854–:2864` (single application covers both seats) |
| 57 | N4 | n | §2.4 "profiles pin xhigh" overstates (only qwen pins xhigh; glm/grok pin high) | → "pin non-Ultra efforts (qwen: xhigh; glm/grok: high)" 

**Coordinator R2 verification (first-hand at HEAD `7bcd344fa7`):** the 3 R2-found functions re-read in full — `step_settings.rs:1250` (assert at :1327 `!body_contains_text(proactive_text)`; both test_cases; bundled text because catalogue `multi_agent = None`), `step_settings.rs:1006` (catalogue `explicit: Some("Delegation policy for {slug}.")` set at :1078–1084; asserts at :1162/:1194; 6 `TokenBudgetScenario` variants), `multi_agent_mode.rs:564` (tuple assert at :610–617; derived `(1,0,1)` under fork default — no mode-change block on the Proactive→Proactive High leg). All v2.2 line pins (`:767–806`, `:809`, `:116–125`, `:109–111`, `:1151`/`:1155–1156`, `:1837`, `:250`, `:814`, FORK-MANIFEST `:35`/`:37`/`:57`/`:61`/`:95–98`) verified against the live tree. Verdicts archived: `/tmp/onprem-r2-seatM-verdict.md`, `/tmp/onprem-r2-seatN-verdict.md`.

**Adjudication note (Majors #1 vs #2, re-verified in R2):** the two Majors contradicted on snapshot impact — M claimed NO insta snapshot flips; N claimed the suite snapshots flip but named them without the `all__suite__scenarios__` prefix. The coordinator adjudicated by direct grep at HEAD `7bcd344fa7`: all 4 `core/tests/suite/snapshots/all__suite__scenarios__astra_*.snap` contain the bundled Explicit policy text (1/2/1/2 occurrences) and zero Proactive text → **they flip**; re-confirmed by both R2 seats (the snapshot half of the R1 adjudication stands). N's substance was right, M's "no snapshots flip" was wrong. **The R1 claim that M's 7 hard-assert inventory "was complete" is RETRACTED:** R2 (seats M2 and N2, independently) found the inventory missed `step_settings.rs` (×2 functions) and `multi_agent_mode.rs:564` — no false positives in the 7 (all re-verified first-hand), but NOT complete. Corrected total: **10 hard-assertion test functions (core ×8, app-server ×2) + 4 snapshots** (v2.2 §7.5).
**Coordinator R3 verification (first-hand at HEAD `7bcd344fa7`):** R3 verdicts: M3 0B/2M/2m/3n, N3 0B/0M/4m/2n (archived `/tmp/onprem-r3-seatM-verdict.md`, `/tmp/onprem-r3-seatN-verdict.md`). Both M3 Majors adjudicated by the coordinator from source before applying: (1) **row 4** — the `live_mode_change_appends_mode_without_reappending_usage_hint` test today takes Ultra→Proactive (turn 1) then High→explicit (turn 2), hence the two rollout records `[proactive, explicitRequestOnly]`; under the fork default the High leg recomputes Proactive, the `multi_agent_mode` section (mode + `usage_hint_hash`) is unchanged, `create_merge_patch` (`core/src/context/world_state/mod.rs:497–499`) returns `None` for equal values and `merge_patch_from` (`:313–331`) `None` for an empty patch → **no second record**; the (:538) tuple flips `(1,1,1)→(1,1,0)`. N3’s "verified clean" re-derivation of row 4 as `["proactive","proactive"]` is adjudicated WRONG — N3’s own row-10 note ("Proactive→Proactive emits no block") is the identical mechanism. (2) **row 7** — `ThreadStartResponse.multi_agent_mode` is a documented hardcoded `@deprecated Always explicitRequestOnly` constant (`v2/thread.rs:218`; production paths `thread_processor.rs:1620/:4134/:5327`, `thread_lifecycle.rs:760`); the `:2829` assert stays green under the fork default; only the text asserts (:2857–:2871) flip — no protocol/wire change (out of scope per §11 item 3). All R3 minors/nits applied in v2.3: rows 4/5/6/7/9/10 rewritten; §7.5 header + §8 + §10 row/function/crate wording; "four test files" remediation clause; `just fix -p codex-app-server` added to §8 gates; bundled-text definition sites + `control_tests.rs:2318` strip-assert rewording added to "Verified NOT at risk"; five pin corrections (`:2787–:2791`, `:1078–:1088`/:1085, `:609–616`, `:251`, `:345–:365`); §12 row-32 seat relabel M2→N2 + row-23 attribution note; §10 RPC-visibility bullet added. R4 dual round (fresh seats) on the v2.3 artifact: **CONVERGED — M4 0B/0M/3m/4n, N4 0B/0M/0m/2n (full round 0B/0M; all 9 applied in v2.4, rows 49–57).**

**Coordinator R4 verification (first-hand; R4 seats verified at HEAD `7bcd344fa7`; v2.4 applied at HEAD `9ac22bb79c` — the two intervening commits `2fb993185d` (manager_tests merge fix) and `9ac22bb79c` (FORK-MANIFEST Last verified) touch none of this spec's claims):** (1) **Convergence:** R4 is a full 0B/0M round across both seats → SDD review loop closed, no R5 required (repo rule: re-review to a full 0B/0M round). (2) **Adjudication carry-forward:** N4 independently re-confirmed both R3 adjudications — row 4 (plus a second confirmation: `render_diff` early-returns `None` for equal mode + `usage_hint_hash`, `core/src/context/world_state/multi_agent_mode.rs:67–73`) and row 7 (plus the fifth-site completeness note `thread_summary.rs:193`, applied as row 49). (3) **Debris adjudication (new, coordinator first-hand):** N4 recorded `~/.codex/glm-model-catalog.json` as "`multi_agent_version: null` ✓" while M4 said the key is ABSENT; a `python3`/json read of the file shows the single glm-5.2 entry has NO `multi_agent_version` key (absent → deserializes `None`) → M4's wording applied (row 54). (4) **Manifest math adjudication:** M4's live-tree re-derivation (52 A = 47 docs + 5 non-docs; 51 fork-only = 4 code + 47 docs today) supersedes N4's "verified clean" on the v2.3-stated counts — N4 verified the v2.3 math as written; M4 re-derived against the live tree (row 50). (5) **M4 self-miscount noted:** M4's §12 minor cited "11 findings (36–46)"; the archived R3 verdicts enumerate 13 (M3 2M+2m+3n + N3 4m+2n) → rows 36–48 added accordingly (row 51). (6) **v2.4 is post-convergence polish** — the 9 R4 minors/nits + the version-label bump; no design change, no test-plan change — applied without a new review round per the SDD rule (the loop runs to a full 0B/0M round; that round is R4 itself). **TDD item 1: GO.**
