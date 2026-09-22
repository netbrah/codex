# Spec — xt2.13 item A: widen the input-text test helpers to post-normalization span types

Status: DRAFT v3.1 (r3-converged: 0B/0M both seats; r1 findings in §8, r2 in §9,
r3 minor/nit errata applied; coordinator-verified N1 provider census in §1)
Scope: test-support only — the `core_test_support` helper, one app-server test file, one
core-suite local helper. **No product code.**

## 1. Purpose

Class N1 from the xt2.13 RUN 2 catalogue (bead `apex-xt2.13` note 2026-09-20): a subset of
the 214 RUN-2 failed tests fails because the test-side assertion helpers match only the
pre-normalization `input_text` span type. This item fixes those helpers so normalized
(non-OpenAI) requests are observable, then lets an empirical re-run split the residue.

N1 membership (MEASURED — per-test panic-site classification of RUN 2; authoritative data
`/tmp/r2_classification.json` + `/tmp/r2_panic_locs.json`, seat-B analysis; catalogue in the
bead note):

- `codex-app-server --test all` (110 failed): **N1 = 13 names**; N1+N2 (partial) = 17 (all
  `multi_agent_v2_developer_instructions`); N2/other = 80.
- `codex-core --test all` (59 failed, 1 duplicate-listing quirk): **N1 = 25 names**;
  KF/LDR/ENV/N2/other = 33 (58 unique names;
  `guardian_review…custom_provider_uses_responses` is listed twice).
- `codex-core` pending_input: **3 failures** driven by the LOCAL narrow helper in
  `core/tests/suite/pending_input.rs:256` (added to this item's scope, §3.3).
  These 3 names are a SUBSET of the 25 above (not additive).

**Clearable subset — 33 of the 38 unique N1 names** (13 app + 25 core). Coordinator
census: each core N1 name's RUN-2 `session_init` provider line was read from the run
log; for the app names none of the 110 blocks contains a `session_init` line — their
provider is the `MockResponsesConfig` default `"Mock provider for test"`
(config.rs:24), non-OpenAI, with no per-test override.
Five names are misclassified N1 and are EXCLUDED from the clearable set:

- `suite::v2::mcp_resource::orchestrator_skill_can_read_referenced_resource_without_an_executor`
  — panics at `mcp_resource.rs:380` on a tool-visibility assert
  (`tool_by_name("skills", "list").is_some()`); no span helper involved → gate-5
  bucket `N2-namespace-tools`.
- 4 core names ran under the PLAIN `"OpenAI"` provider in RUN-2 (normalization OFF ⇒
  this change is a provable no-op for their bodies) → KF candidates, adjudicated in
  gate 5:
  - `compact_remote::remote_compact_v2_rewrites_multiple_trailing_function_call_outputs::automatic`
    (panic `compact_remote.rs:1335`; already in the bead KF registry)
  - `guardian_context_budget::review_preserves_user_instructions_until_request_budgeting::complete_instructions_fit`
    (panic `guardian_context_budget.rs:165`, `delta_context.contains(&followup)`)
  - `personality::config_personality_none_sends_no_personality` (panic
    `personality.rs:88`, `{{ personality }}` placeholder in the top-level
    `instructions` string, not a span helper)
  - `subagent_notifications::spawned_full_history_v2_child_uses_model_precedence_without_dropping_context::full_fork_restores_explicit_policy_after_proactive_transition`
    (panic in `subagent_notifications.rs`, content assert)

Clearable = **12 app-server + 21 core-suite = 33 names**.

Everything outside the N1 list is out of scope here and is bucketed for item A2 (N2:
tool-output-shape / agent-message-conversion / raw-item-shape per-file rewrites) and the
KF/LDR/ENV catalogue (§7).

## 2. Root cause (verified mechanism set)

Gate: `normalize_content_types` is applied from `model-provider/src/provider.rs:399` as
`!is_openai()`; `is_openai()` = provider name == `"OpenAI"`
(`model-provider-info/src/lib.rs:583-585`).

Mechanisms (all in `codex-api/src/endpoint/content_type_compat.rs`, each documented at the
function level):

1. `normalize_content_types` (:33) — per `input[]` item: relabels `input_text`/`output_text`
   parts → `text`; relabels an `agent_message`'s `encrypted_content` part → `text` (it carries
   the plain inter-agent payload); drops all other `encrypted_content` parts.
2. `normalize_tool_output` (private, :94) — tool-output items: an all-text `output` array
   collapses to a single `"\n"`-joined string; a mixed array keeps shape with text parts
   relabelled in place and undecryptable `encrypted_content` parts dropped.
3. `translate_agent_messages` (:155) — converts `agent_message` items to `message`/`user`
   items.

The app-server suite's `MockResponsesConfig` provider is named `"Mock provider for test"`
(`app-server/tests/common/config.rs:23-28`) ⇒ non-OpenAI ⇒ all three mechanisms apply to
every recorded request.

The N1 defect itself: the shared helpers `ResponsesRequest::message_input_texts` /
`message_input_text_groups` (`core/tests/common/responses.rs:223` / `:235`) filter spans with
`type == "input_text"` only ⇒ empty list on every normalized body ⇒ assertion failure. The
local twin helper in `pending_input.rs:256` carries the identical defect.

Precedent in-tree: `app-server/tests/suite/v2/turn_start.rs` already works around the defect
with a local `normalized_developer_message_texts` and carries the note "apex-xt2.13: consider
widening the shared helper so this local extraction becomes unnecessary."

N2 (OUT of scope, §7): tests that assert the raw POST-normalization SHAPES (collapsed
tool-output arrays, converted agent_message items, restructured tool lists) panic at
non-helper sites — measured 80 of the 110 app-server failures plus part of the core 33.

## 3. The change (exact)

Three filter widenings plus one pin test. No product code.

### 3.1 `codex-rs/core/tests/common/responses.rs` — `message_input_texts` (:223)

Widen the span-type filter from `== Some("input_text")` to
`matches!(span_type, Some("input_text" | "text"))`. Extend the doc comment: matches
both pre-normalization (`input_text`) and post-normalization (`text`) spans, so the
helper observes requests whether or not `normalize_content_types` ran (apex-xt2.13
item A).

### 3.2 Same file — `message_input_text_groups` (:235)

Identical widening of its span filter. `has_message_with_input_texts` (:250-258) is
fixed transitively (it delegates to the groups helper); no direct edit.

### 3.3 `codex-rs/core/tests/suite/pending_input.rs:256` — local twin

The local `message_input_texts(body, role)` carries the identical `input_text`-only
filter. Widen it identically (same `matches!` form). Its 3 user tests set provider
name `"OpenAI (test)"` (pending_input.rs:1429/:1514/:1631) — non-OpenAI ⇒
normalization ON — and are exactly the 3 RUN-2 pending_input failures. Migrating
its 13 call sites to the shared helper is an A2 follow-up (note it in code; do not
migrate in this item).

### 3.4 `codex-rs/app-server/tests/suite/v2/turn_start.rs` — delete workaround

Delete the local `normalized_developer_message_texts` (:2729-2752 pre-diff; :2740-2759
at spec-writing time — line drift, deletion scope verified exact by both diff seats)
including its
"apex-xt2.13: consider widening the shared helper" note (this item IS that
widening); its call sites become `request.message_input_texts("developer")`.

### 3.5 Pin test — `responses.rs` `#[cfg(test)] mod tests` (:461, next to `request_with_input`, :468)

Placement is forced: `ResponsesRequest(wiremock::Request)` has a private tuple
field (:104) and no public constructor, so a synthetic request is constructible
only inside `core_test_support`. The pin MUST be a synthetic-request test, not a
captured-wire test: normalization rewrites every span in a body (all-or-nothing
per body, `content_type_compat.rs:23-69`), so a captured request can never
contain both span types in one message — a `mount_sse_once` flow cannot exercise
the assertion. (r1 erratum corrected: `#[cfg(test)]` is REAL gating in this lib
crate; in integration-test crates it would be redundant-but-live.)

Pin (name it greppably, e.g. `message_input_texts_accepts_input_text_and_text_spans`):
via `request_with_input`, build a request whose `input[]` holds one developer
`message` item whose content has exactly two parts — `{"type": "input_text", ...}`
then `{"type": "text", ...}` (in that order) — and assert on the method pair:
- `.message_input_texts("developer")` returns both texts, in input order;
- `.message_input_text_groups("developer")` puts both in the same (single)
  message group.

RED (pre-widening) the pin fails on the first assert (the `text` span is dropped);
GREEN (post-widening) it passes.

## 4. Risk

- Monotonic: the widened filter only ADDS spans to every returned list; no caller
  can lose a span it saw before.
- Assertion-side only: the full call-site census (three independent greps: coordinator
  78 files / 329 paren-sites; seat A r2 83 files / 356; seat B r1 83 files / 363
  including doc mentions — spread is mention-inclusion, not scope) reads helper output
  inside assertions only; zero request-construction usage.
- Only new failure mode: exact-equality / count / order / negative assertions
  written against normalized bodies that legitimately carry `text` spans (e.g.
  raw-injected items, `app-server/tests/suite/v2/thread_inject_items.rs`) now see
  the extra spans. Gate 4's empirical split is the detector; each such test gets
  per-test adjudication (gate 5) — never a blanket assertion rewrite.
- Non-normalized (OpenAI) bodies are unaffected: on the OpenAI path codex emits
  no `text`-typed spans in message content (part types are input_text/input_image/
  input_audio/encrypted_content; `agent_message` items sit outside the message
  role/item filter) — seat A found no counterexample.
- No product code, no wire format, no snapshot files change in this item.

## 5. Gates (all SCS, incremental, one cargo at a time)

SCS env (bead-note recipe): `CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=1.94.1-x86_64-unknown-linux-gnu CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864 CODEX_SKIP_BWRAP_BUILD=1` + `RUSTY_V8_ARCHIVE` / `RUSTY_V8_SRC_BINDING_PATH` (ptrcomp-sandbox release paths); clippy waived (no component on 1.94.1 — CI enforces).

1. RED: `cargo test -p core_test_support` — the pin fails pre-widening. First-run
   target: record any pre-existing crate-test failures as baseline.
2. GREEN: same command — the pin passes post-widening.
3. Incremental: `cargo test -p core_test_support` (pin + crate tests;
   baseline-vs-delta for the first-run baseline) + `cargo test -p codex-core
   --lib` + rustfmt check of the 3 touched files (SCS rustfmt if the component is
   present on 1.94.1, else the Mac default-rustfmt `--check --edition 2024`
   recipe; record which ran).
4. Empirical split: `cargo test -p codex-app-server --test all` + `cargo test -p
   codex-core --test all` + `cargo test -p codex-core --lib` + `cargo test -p
   codex-core --test responses_headers`. Expectation: all 33 clearable N1 names
   clear (12 app-server + 21 core-suite; the 3 pending_input names are inside the
   21). The 17 N1+N2 partial names (all
   `multi_agent_v2_developer_instructions`) are mixed per RUN-2: 4 panic at the
   non-helper site :674, 1 at the helper site :413 (could clear), 12 are deadline
   timeouts with no panic site — so the app residue is 97 or 98; record the
   measured outcome, do not chase in this item.
   Expected residue (measured-truth baseline, not a target): app-server 110 → 97–98
   (110 − 12 clearable − at most 1 partial; the `mcp_resource` name fails at its
   tool-visibility assert); core 59 rows (58
   unique) → 37 (the 21 clearable names clear, incl. the dup-listed
   `guardian_review…custom_provider_uses_responses` = 2 rows; the 4 plain-OpenAI
   names stay red); pending_input 3 → 0; responses_headers 1 (ENV, unchanged).
5. Residue adjudication (BOTH suites): a per-name disposition table, every
   surviving failure classified into a named bucket — `N2-tool-output-shape`,
   `N2-agent-message-conversion`, `N2-raw-item-shape`, `N2-namespace-tools`,
   `KF`, `LDR` (isolate re-run once), `ENV`, `other`. The 5 excluded N1 names (§1)
   are dispositioned here too. The KF registry in the bead note is updated in the
   same pass: add `unified_exec::tests::reusing_completed_process_returns_unknown_process`
   (new in RUN-2); correct the module prefixes of the two ALREADY-registered
   core-`--lib` names to `session::tests::user_shell_commands_do_not_inherit_managed_network_proxy`
   and `session::turn::tests::post_sampling_token_estimate_is_disabled_by_always_on_sinks`;
   plus the exec_process deadline names if their isolate re-runs still fail; plus KF
   entries for the 4 plain-OpenAI core-suite names if their isolate re-runs confirm
   flake (else a new item is filed).

## 6. Acceptance (all measurable on SCS)

- The 33 clearable N1 names are green at the post-fix commit (per-name, from the
  gate-4 logs); the 5 excluded names (§1) remain red at gate 4 — gate 5
  dispositions each (KF entry on confirmed flake, else a new item).
- The gate-5 per-name disposition table is recorded in the bead note.
- Pin green, proven by the named command `cargo test -p core_test_support`
  (test name recorded).
- The commit's file set (per `git show --stat <commit>`) is confined to exactly:
  `codex-rs/core/tests/common/responses.rs`, `codex-rs/core/tests/suite/pending_input.rs`,
  `codex-rs/app-server/tests/suite/v2/turn_start.rs`, and this spec (untracked until
  commit — hence absent from a pre-commit `git diff --stat`).
- Dual fresh seats: 0B/0M on this spec (the round reviewing v3) and 0B/0M on the
  landed diff
  (post-TDD, fresh seats again).

## 7. Out of scope

- Item A2: N2 per-file rewrites (tool-output-shape asserts, agent_message
  conversion find-predicates, raw-item-shape asserts, namespace-tool visibility
  asserts). Full per-test list: /tmp/r2_panic_locs.json + the bead catalogue.
- Item B: FCAT TUI model-catalog snapshots. NOTE: the glm catalog-row change
  (f443aced98) may ripple TUI catalog tests — the item-B re-run must include
  them.
- Item C: KF/ENV catalogue notes (ENV rows: exec-server load, TLS real-cert,
  skills walk, crossterm TTY, air-gap, bwrap `--argv0`, v8 ptrcomp).
- xt2.14: the exec-crate apply_patch shutdown deadlock (separate bead; 3rd
  repro recorded in the RUN-2 note).

## 8. r1 findings disposition

Both seats' r1 verdicts (/tmp/xt213-itema-seat-{A,B}-r1-verdict.md). Every r1
finding, and where v2 disposes it:

| Finding | v2 disposition |
|---|---|
| A-B1 scope ≠ failure population | §1 measured N1 list; §3.3 adds pending_input.rs to scope; gate-4 expectation redefined (41 names, residue recorded); §7 splits N2 into A2 |
| A-M2 no gate command runs the pin | §3.5 names the placement; gates 1/2/3 run `cargo test -p core_test_support`; §6 names the proof command |
| A-m3 cfg(test) erratum wrong | §3.5 reworded (real gating in this lib crate; redundant-but-live in integration-test crates) |
| A-m4 §4 cross-ref "gate 5" for the SCS re-run | the re-run is gate 4; refs corrected in §4/§5 |
| A-m5 gate 4 omits `--test responses_headers` | added to gate 4 |
| A-n6 duplicated paragraphs + census undercount | v2 rewrite de-duplicates; §4 census corrected to 83 files / 363 call sites |
| B-B1 ≥61/110 outside the span-filter defect | same as A-B1 (§1/gate 4/§7) |
| B-M1 pin unimplementable at assumed placement; fallback unsound; no standing gate | §3.5: placement = existing `#[cfg(test)] mod tests` (only constructible site — private tuple field :104, no pub constructor); pin is synthetic-by-design (captured bodies are all-or-nothing per body); gates 1/2/3 + §6 carry it |
| B-M2 no app-server residue adjudication; "N1 = 0" unmeasurable | gate 5 adjudicates BOTH suites with named buckets; N1 = the measured list (§1, §6) |
| B-m1 cfg(test) erratum inverted | same as A-m3 |
| B-m2 §2 documents 1 of 3 wire mechanisms | §2 documents all three (normalize_content_types, normalize_tool_output, translate_agent_messages) under the single `!is_openai()` gate |
| B-n1 duplicated paragraphs | v2 rewrite de-duplicates |
| B-n2 census undercount | §4: 83 files / 363 call sites (measured) |
| B-n3 path prefix | kept prefix-free `codex-api/…` form — matches in-tree doc style (turn_start.rs:2735) |

## 9. r2 findings disposition

Round-2 verdicts: /tmp/xt213-itema-seat-{A,B}-r2-verdict.md. Where the v2
disposition of r1 findings A-B1/B-B1 (and B-M2) was only formal (the re-measured
list itself was flawed), v3 carries the corrected numbers in §1/gate 4/§6.

| Finding | v3 disposition |
|---|---|
| A-B1 N1 list not exact: 41 double-counts; 2 names un-clearable | §1: 38 unique names (3 pending_input ⊂ 25); 5 excluded (mcp_resource → `N2-namespace-tools`; personality → KF candidate), 33 clearable; gate 4/§6 restated |
| A-M2 residue math wrong (core ~31; ~97 off-by-1; "4 partial" vs §1's 17) | gate 4: exact arithmetic — app 110 → 97–98; core 59 rows (58 unique) → 37; 17 N1+N2 partial names |
| A-m3 §3.3 "6 call sites" (actually 13) | §3.3: 13 (coordinator grep-verified: 13 local-fn call sites; the 2 `request.message_input_texts` sites are the shared helper) |
| A-m4 KF add-list: 2 pre-existing entries + wrong module prefix | gate 5: only `unified_exec::tests::reusing_completed_process_returns_unknown_process` is new; the other two are already registered and their prefixes are corrected (`session::tests::` / `session::turn::tests::`) |
| A-m5 gates omit SCS `RUST_MIN_STACK` env requirement | §5 env line (full bead-note recipe incl. `RUST_MIN_STACK=67108864`, `CODEX_SKIP_BWRAP_BUILD=1`, RUSTY_V8 paths, clippy waiver) |
| A-n6 census 363 = mentions, 356 strict | §4: three-grep spread stated (78/329, 83/356, 83/363), point unchanged (assertion-side only) |
| A-n7 §2 `normalize_tool_output` :106 → :94 | §2 corrected (coordinator grep-verified) |
| B-B1 5 of 38 N1 names un-clearable (mcp_resource + 4 plain-OpenAI core) | §1: coordinator provider census (core: RUN-2 `session_init` lines; app: MockResponsesConfig default, no per-test override — all 55 names checked; exactly 4 core under plain `OpenAI`, 0 app); 33 clearable; gate 4/§6 restated |
| B-M2 "41 = 13+25+3" double-counts; core 59 → ~31 subtracts twice | §1: 38 unique; gate 4: core → 37 (21 clearable names; dup-listed name among them) |
| B-M3 "4 N1+N2 partial names" contradicts §1's 17 | gate 4: 17 (JSON-verified; the 4 was the r1-era single-site panic count); v3.1 adds the per-name breakdown (4 @ :674 non-helper, 1 @ :413 helper-site, 12 deadline timeouts) |
| B-m1 KF wrong module prefix | gate 5 (same as A-m4) |
| r3-A-m1 17 partials: "non-helper panic sites" overstates | gate 4: mixed breakdown (4 @ :674, 1 @ :413, 12 timeouts); residue 97–98 |
| r3-B-m1 census source: app blocks carry no `session_init` line | §1: core from `session_init`; app from MockResponsesConfig default (config.rs:24) |
| r3-B-n1 §6 "remain red" vs flake path | §6: "remain red at gate 4 — gate 5 dispositions each" |
| r3-A-n1 line drift (:236/:250-256/:461-477) | §3.2/§3.5 corrected (:235 / :250-258 / mod :461, fn :468) |
| r3-A-n2+n3 §3.5 loose free-function phrasing; vestigial §5.x gate refs | §3.5 method-form assertions; all gate refs now "gate N" |
| B-m2 census 83/363 not reproducible (81/344 measured) | §4: hedged three-grep spread |
| B-m3 untracked spec can't appear in `git diff --stat` | §6: acceptance reworded to the commit's file set per `git show --stat` |
