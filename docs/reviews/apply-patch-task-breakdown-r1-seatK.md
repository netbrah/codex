# Task Breakdown Cross-Review — Round 1, Seat K

Artifact: `docs/responses-compat-apply-patch-task-breakdown.md` (460 lines, stage 3)
SoT: `docs/responses-compat-apply-patch-format.md` (v5, 1026 lines) · Branch: `feat/normalize-content-types-vllm`
Repo: /Users/palanisd/Projects/upstream/codex · HEAD: 197ea1642c (working tree: spec + breakdown untracked; product code at HEAD)

## Verdict

**APPROVED — 0 Blocking, 0 Major, 6 Minor, 3 Nit.**

The breakdown conforms to spec v5 item-for-item (every T1.1-T5 sub-case appears
exactly once in the right item with the same expectation), the red-state ledger
matches HEAD exactly (13 assertions, correct per-assertion P-site mapping,
accurate surviving list except one count and one path), the per-commit
green-tree logic holds for all six items, the invariants match spec §3.4 in
substance (one over-claimed bullet), and the §4 protocol implements the
AGENTS.md stage-5 flow with no hole that lets an unreviewed edit reach a
commit. The five v5 fixes (Mandate 0) all re-verify at source, including
byte-exact rollout evidence for I-M1 and I-N3/J-M1. The six Minors are
documentation defects in the breakdown itself (a wrong ledger path, a wrong
fixture count, a wrong snapshot-update mechanism, two protocol ambiguities,
one false invariant clause); none changes the design, the red state, or the
item order, and each is a one-line fix. Recommend the coordinator apply them
in a breakdown v5.1 pass (and record them in a short resolution note in the
breakdown) before launching the stage-5 workflow.

## Method

- Rollout snapshot (live dir is live-growing): copied
  `~/.codex/sessions/2026/09/{13,14,15}` to `/tmp/rout_snap_k1` via rsync at
  **2026-09-16T02:47:34Z** — 361 files; on-disk layout verified as
  `YYYY/MM/DD` (day dirs `13`, `14`, `15`); file triple at snapshot time
  83/162/116 (09-15 has grown past the 86 at the spec's 00:27Z freshness pin —
  live drift, not a finding per the review brief).
- All pinned claims were checked at their stated pins, not at snapshot time.
- Every line cite in the spec and breakdown was opened at HEAD (`sed -n` /
  `grep -n`); message strings were grep-swept repo-wide (`--include="*.rs"`)
  to establish the complete red/surviving assertion set; scenario fixtures
  were enumerated on disk and their patches scanned for P2/P3 outcome
  sensitivity; the P1 text block was re-measured programmatically (chars,
  words, drift substrings, T2.2 extraction rule).
- Rollout forensics were done in Python against the snapshot only
  (parse each JSONL line; join `function_call` → `function_call_output` by
  `call_id`; model from each session's first `turn_context`).
- No file was modified other than this report. No subagents (leaf reviewer).

## Mandate 0 — v5 minor/nit fix re-verification

All five fixes verified: (a) present in spec v5 at the cited location, (b)
matching the source fact.

### 0.1 I-M1 (Minor) — both missing-`*** End Patch` patches end with the `+`-prefixed terminator

- Spec text at §1.2 (line 165): "2 missing `*** End Patch` (F4; both patches
  end with the `+`-prefixed terminator — final line `+*** End Patch`, never
  bare; v5: corrected from "one", both patches re-derived byte-exact)". The
  R5 map row (line 1015) records the fix.
- Rollout evidence (from `/tmp/rout_snap_k1`):
  - `call_10f109a345a4439094df5930` — `14/rollout-2026-09-14T07-58-02-…jsonl`,
    event 2026-09-14T12:03:39.616Z, model glm-5.2 (first `turn_context`).
    Patch: 233 lines; final line repr `'+*** End Patch'` — exactly the
    `+`-prefixed terminator (not bare). Matching output:
    `apply_patch verification failed: invalid patch: The last line of the
    patch must be '*** End Patch'`.
  - `call_8ee1b0c158f849c28f6f9da4` — `14/rollout-2026-09-14T23-11-53-…jsonl`
    (session spans midnight; event 2026-09-15T03:17:57.255Z, i.e. 09-15 as the
    spec states), model glm-5.2. Patch: 160 lines; final line repr
    `'+*** End Patch'` — exactly the `+`-prefixed terminator (not bare). Same
    output string.
  - Both timestamps are before the glm pin 2026-09-15T23:34:04.678Z.
- Completeness sweep (due diligence beyond the mandate): joining
  `function_call(name=apply_patch)` → output across the whole snapshot,
  **exactly 2** apply_patch calls at/before the glm pin carry the
  missing-End error (the two named) and **exactly 5** carry the
  missing-Begin error — matching the spec's F4 "5 Begin, 2 End" glm count.
  (An unjoined string sweep initially matched ~100 extra qwen-session
  `function_call_output`s; those are shell/CLI invocations echoing the same
  sentence, not apply_patch function-tool calls — the name-join removes them.)

### 0.2 I-N3/J-M1 (Minor) — VERIFY exemplar list gains `Failed to find context` (file_update.rs:110)

- Spec text at §1.2 Method (line 143): VERIFY definition now reads "…
  `Failed to read file`, `Failed to write file`, `Failed to find context`
  (file_update.rs:110) …". R5 map row (line 1016) records the fix.
- Source: `codex-rs/apply-patch/src/file_update.rs:110` is the literal
  `"Failed to find context '{ctx_line}' in {path}"` inside
  `ApplyPatchError::ComputeReplacements` (returned from the update-hunk
  context seek at :109-111). Confirmed.
- Rollout evidence: `call_a9cf9f970e8b4405af8c5bbf` (2026-09-15T23:56:08.699Z)
  and `call_69f201bef73d4b7fba575661` (2026-09-15T23:56:49.864Z), both in
  `15/rollout-2026-09-15T19-34-01-…jsonl`, both carry output
  `apply_patch verification failed: Failed to find context '…' in
  …/docs/reviews/apply-patch-format-spec-r2-seatC.md` — a post-parse
  filesystem verification failure, i.e. the VERIFY class. Both timestamps are
  after the qwen pin 23:30:11.662Z and before the round-3 freshness copy
  window (00:23:58-00:27:48Z 09-16), consistent with the spec's "outside
  every pinned row" annotation.

### 0.3 J-N1 (Nit) — `ev_function_call` cited at responses.rs:933-943

- Source: `codex-rs/core/tests/common/responses.rs` —
  `pub fn ev_function_call(call_id: &str, name: &str, arguments: &str) ->
  Value {` opens at **:933** and closes at **:943** (verified by line
  number). The fn really spans :933-:943.
- Spec cites: §4 T4.2 (line 711) "generic constructor at :933-943" and the
  R2-map D-M2 row (line 882) "v5: constructor range end :942→:943" — both
  correct. The adjacent pattern-helper cites also re-verify:
  `ev_exec_command_call_with_args` = :1025-1028 and
  `ev_apply_patch_exec_command_call_via_heredoc` = :1030-1035.
- Breakdown's own cite (Item 4 step 2): "generic constructor
  `ev_function_call` :933-943" — matches.

### 0.4 I-N1 (Nit) — Conclusion ratio arithmetic

- Spec text at §1.2 Conclusion (lines 208-210): "≈60× qwen's ~1%
  (26/37 = 70.27% vs 5/413 = 1.21%)".
- Math check: 26/37 = 70.27% (70.2702…); 5/413 = 1.21% (1.21065…); both
  percentages exact. Ratio 26/37 ÷ 5/413 = 10738/185 = **58.04×**. So
  "≈60×" overstates the exact ratio by ~3.4%, while the same spec's R5 map
  (line 1018) says "70.27% / 1.21% ≈ 58×" and the round-5 seat-I summary
  (line 989) says "≈ 58×". The fix's direction is right (v4's "an order of
  magnitude above" implied ~10×, a 6× understatement) and the two quoted
  percentages are exact, but the "≈60×" label is less precise than the ≈58×
  used elsewhere in the same document. Logged as K-N1 (spec-side wording;
  the breakdown merely inherits the Conclusion text).

### 0.5 I-N2 (Nit) — R3-map E-N1 row opening restated

- Spec text at R3 map E-N1 row (line 923) now opens: "Snapshot pinned (v5:
  two explicit pins — qwen 2026-09-15T23:30:11.662Z with pin-instant file
  triple 83/162/72, glm 2026-09-15T23:34:04.678Z; v3 had paired the qwen pin
  with the 83/162/86 freshness-copy triple — Round-4 map, G2 row); …".
- The superseded pairing "Snapshot pinned (2026-09-15T23:30:11Z, 83/162/86
  rollout files)" is gone from the opening; the 83/162/86 triple appears
  once, only in the mid-row annotation about v3's superseded pairing. The
  only other 83/162/86 occurrence in the spec is the §1.2 Freshness bullet,
  where it is correctly paired with the 00:23:58-00:27:48Z freshness copy.
  Fix verified.

**Mandate 0 result: 5/5 fixes present in v5 at the cited locations and
matching the source fact** (one residual wording nit, K-N1, noted for
completeness — it is in the spec, not the breakdown).

## Mandate 1 — Spec ↔ breakdown conformance

### 1(a) Sub-case mapping (spec §4 T1-T5 → breakdown items)

Walked every spec sub-case against the breakdown. Result: each appears
**exactly once**, in the right item, with the same expectation. No missing,
duplicated, or altered sub-case.

| Spec sub-case | Breakdown location | Expectation match |
|---|---|---|
| T1.1 F1 replay | Item 1, step 5, bullet 1 | yes (raw markdown, `# H1` first, byte-identical to canonical `+` form) |
| T1.2 empty line | Item 1, step 5, bullet 2 | yes |
| T1.3 mixed prefixed/raw | Item 1, step 5, bullet 3 | yes |
| T1.4 StartedPatch rewrite [assert 1] | Item 3 (ledger #1; steps 2, 3) | yes |
| T1.5 DeleteFile rewrite [assert 3] | Item 3 (ledger #3; steps 2, 4) | yes |
| T1.6 AddFile raw → `Ok` `bad\n` [assert 2] | Item 1 (ledger #2; steps 2, 4) | yes |
| T1.7 `*** `-prefixed non-marker / structural lines | Item 1, step 5, bullet | yes (incl. whitespace-padded `*** End Patch`) |
| T1.8 typo'd `*** Ad File: x` swallowed | Item 1, step 5, bullet | yes |
| T1.9 `++42` lossy + existing-path overwrite | Item 1, step 5, bullet | yes |
| T1.9b final-line `*** End Patch`, both variants | Item 1, step 5, bullet | yes |
| T1.10 whitespace-only / Env-ID / unclosed / CRLF | Item 1, step 5, bullet | yes |
| T1.11 Update-File stays strict | Item 1, step 5, bullet | yes |
| T1.12 CLI test update [assert 4] | Item 3 (ledger #4; steps 2, 3) | yes (see K-M1 for the ledger path defect) |
| T1.13 golden/canonical scope | Item 1, step 5, bullet (also §1 surviving list) | yes (see K-M2 for the "24" count) |
| T1.14 boundary pre-pass [rewrites parser tests] | Item 3 (ledger #5-13; steps 2, 6; step 9 states T1.14 = steps 2+6, no extra code) | yes |
| T2.1 drift guard (7 substrings) | Item 2, steps 1, 3 | yes (all 7 substrings listed verbatim) |
| T2.2 self-consistency + pinned extraction rule | Item 2, step 4 | yes (split at first line exactly `Example:`; remainder ends `*** End Patch`; exact hunks) |
| T2.3 full-spec test updated to new text | Item 2, step 5 | yes (see K-M3 for the wrong update mechanism) |
| T2.4 freeform unchanged | Item 2, step 6 | yes |
| T3.1 absent / non-string `patch` messages | Item 3, steps 5, 7 | yes (§3.3.3a/§3.3.3b) |
| T3.2 handler-level F1 raw Add-File succeeds | Item 3, step 8 | yes (scaffolding cites verified: `invocation_for_payload` apply_patch_tests.rs:45, `make_session_and_context` session/tests.rs:5882) |
| T4.1 provider rename + wire assertion | Item 4, step 1 | yes (`with_config` test_codex.rs:347, clone :839-859; `ResponseMock` cites — see K-N2) |
| T4.2 F1 raw Add-File end-to-end (new file) + scaffold | Item 4, steps 2-4 | yes (new `ev_apply_patch_function_call` + `mount_apply_patch_function_call`; explicit do-not-reuse `mount_apply_patch` with the `matches_kind` Function-only rationale, apply_patch.rs:603-605 / registry.rs:548-556 — both verified at HEAD) |
| T4.2b existing-file overwrite | Item 4, step 5 | yes |
| T5 gates | per-item "Gates" lines + Item 5 step 3 | yes (`just fmt`, `just test -p codex-apply-patch`, `just test -p codex-core apply_patch`, `just test -p codex-model-provider`, `just fix -p` over touched crates, no `--all-features`) |

The item split (Item 1 = P2 only; Item 2 = P1; Item 3 = P3.1-P3.4 +
T1.4/T1.5/T1.12/T1.14 rewrites; Item 4 = integration; Item 5 = docs/gates;
Item 6 = real-env + close) is internally consistent with the spec's
"separate commits per part (P2, P1, P3) plus the seam-doc update" (spec §3)
and with the dependency order (P2 before P3 so the AddFile-arm rewrite and
the message-site rewrites never share a commit; P1 before T4.1 which asserts
the new wire text).

### 1(b) Per-commit green-tree logic

Red-state share per item (breakdown §2 column; verified against the ledger):
Item 1 = 1/13 (#2), Item 2 = 0/13, Item 3 = 12/13 (#1, #3, #4-13),
Item 4 = 0/13, Items 5-6 = none. Sums to 13.

State of the 13 assertions at each commit boundary (verified against the
actual test bodies at HEAD):

| After commit | Green | Untouched (still green — code not yet changed) | Tree |
|---|---|---|---|
| Item 1 (P2) | #2 (rewritten, now `Ok`) | #1, #3, #4-13 | green — the only behavior P2 changes is the AddFile arm; repo-wide greps show no other test asserts that arm's error (the "not a valid hunk header" test occurrences are exactly :828/:838/:848 in streaming_parser.rs + the two CLI/core tests), and all 25 scenario Add-File patches are canonical `+`-prefixed (P2 outcome-neutral) |
| Item 2 (P1) | (no red state) | all 13 green | green — only the function-tool spec text + its own tests change; the freeform spec and OpenAI-path bytes are untouched (Item 2 DoD + §5 invariant) |
| Item 3 (P3) | all 13 | — | green at commit; the 12-assertion red window opens at step 2 (batch rewrite) and closes at step 6 (P3.4) — **entirely inside Item 3** |
| Item 4 (integration) | — | all 13 green | green — new tests only; "green at write time" regression locks with explicit "stop and report, do not weaken" instruction |
| Item 5 (docs/gates) | — | — | green — docs only; full gate re-run |
| Item 6 (ops) | — | — | n/a — no product-code commit |

Mid-Item-3 sub-step state: after step 3 (P3.1) #1 and #4 are green, #3 and
#5-13 still red; after step 4 (P3.2) #3 green; after step 6 (P3.4) all
green. No commit occurs at any of those boundaries (commit is §4 step 5,
after the review loop, which requires the item gates green). The
surviving substring test (core `apply_patch_cli.rs:707`) stays green under
P3.1 because the extended message keeps `is not a valid hunk header`
(verified: the old message is a strict prefix of the new one per spec
§3.3.1, and the test's patch — `*** Begin Patch\n*** Frobnicate File: foo\n
*** End Patch` — is exactly the StartedPatch case, sent via
the freeform handler to the shared `parse_patch` path). P3.3 creates no
hidden red: the old handler message string occurs nowhere in any test
(repo-wide grep — only the implementation site apply_patch.rs:539). P3.4
changes only parser.rs strings; the streaming boundary strings it must not
touch are at streaming_parser.rs:168/:184/:374 (verified, see 1(c)).
Scenario fixtures assert final filesystem state only (scenarios.rs —
"intentionally do not assert on the exit status"), so message-only changes
cannot break them; 013_rejects_invalid_hunk_header is the StartedPatch
case (still rejected under P3.1).

**No item may commit a red tree — and none does as the ordering is written.**
The one tension is inside Item 3's wording ("run the item gates" after each
sub-step while the tree is deliberately red mid-item) — see K-M4.

### 1(c) Invariants (breakdown §5 vs spec §3.4)

Bullet-by-bullet:

1. "Outbound request bytes for providers named `OpenAI` are unchanged (the
   freeform tool spec, the `base_instructions` bytes, and everything else on
   the wire)" — matches spec §3.4's exact invariant (line 589); the
   parenthetical is a faithful elaboration (spec §3.4 defers the
   base_instructions change to the revisit trigger; the in-scope fix is the
   P1 backslash-n rule, function-tool-only).
2. "P2/P3 are provider-agnostic parser/handler changes that affect only
   inputs upstream rejects … no behavior change for canonical patches
   (golden scenarios prove it)" — matches spec §3.4 ("for every input
   upstream accepts, behavior is byte-identical; for inputs upstream rejects,
   P2 turns some Add-File rejections into accepted content and P3 turns some
   rejections into teachable errors"); the added examples
   (raw/missing-prefix content, missing boundary markers,
   absent/non-string `patch`) are all genuinely upstream-rejected inputs.
3. "P3.4 touches the `parse_patch` pre-pass surface only (function path +
   CLI + shell-intercept); the diff consumer's parallel streaming boundary
   messages (`streaming_parser.rs`:168/:184/:374) stay unchanged" — matches
   spec §3.4's surface refinement. The three added line cites verify at HEAD:
   :168 = End-marker string in `finish()`; :184 = Begin-marker string in
   `NotStarted`; :374 = End-marker string in the `EndedPatch` arm
   (content-after-end). All three are reachable via
   `ApplyPatchArgumentDiffConsumer` (apply_patch.rs:90-91, a
   `StreamingPatchParser` field fed by `push_delta`, no pre-pass) and are
   shadowed by the pre-pass on the `parse_patch` path (pre-pass runs first in
   `parse_patch_text`, parser.rs:193-199 → check at :221/:256-274), so the
   characterization is accurate.
4. "No existence check on any path (Add-File overwrites; Delete-File never
   reads); no Update-File leniency; no `strict:true`; no client-side retry
   of `{}` calls; no per-model parser modes" — the last four clauses match
   the spec §3.4 non-goals verbatim in substance. The first clause
   over-claims: spec §3.4's decision is Add-File-specific ("No existence
   check for Add-File on any path"), and "Delete-File never reads" is false
   at HEAD (lib.rs:536-552: the DeleteFile branch calls
   `note_existing_path_delta_support`, `fs.read_file_text` to capture
   deleted content for the delta, and `ensure_not_directory`; a delete of a
   missing file fails — scenario 007_rejects_missing_file_delete; Update
   likewise requires an existing file — scenario 009). See K-M6.
5. "No new public API surface; no `Cargo.toml`/lock changes (if unavoidable,
   `just bazel-lock-update` lands in Item 5)" — matches spec §3
   ("No capability/routing changes; no new `is_openai()` branches") and
   spec §6's lock note. The capability-gate cites verify at HEAD:
   spec_plan.rs:1257-1271 (function-tool registration) and
   provider.rs:372 (`apply_patch_function_tool: !is_openai()`);
   `is_openai()` is `name == OPENAI_PROVIDER_NAME` with no Azure branch
   (model-provider-info/src/lib.rs:546-547, constant :40 — as Item 5 states).

### 1(d) Item 5 seam-doc update list vs spec §6

Spec §6 lists six seam-doc updates; breakdown Item 5 step 1 carries all six,
item-for-item, with no additions or omissions:

| Spec §6 item | Breakdown Item 5 |
|---|---|
| §2: correct stale "plus an Azure branch" description of `is_openai()` (name match only; Azure-named in seam scope) | yes — with verified cites (model-provider-info/src/lib.rs:546-547, constant :40) |
| §3: add P1/P2/P3 as the format-remediation layer with the 2026-09-13..15 evidence (26/37 ≈ 70% pinned; 09-15 alone 13/21 ≈ 62% vs qwen 5/413 ≈ 1%) | yes — same figures |
| Divergence table: one row, provider-agnostic; surface = function-tool path + freeform diff consumer + CLI (P2) plus `parse_patch` pre-pass surface (P3/P3.4); diff-consumer streaming boundary messages intentionally unchanged; re-apply on upstream restructure | yes — all clauses present |
| §6 conflict sites: add `streaming_parser.rs` and `parser.rs` | yes |
| §6 verification gate: add `just test -p codex-apply-patch` + raw-format probe (spec §5.4) | yes |
| Invariants: add the §3.4 exact-invariant text | yes (request-bytes; provider-agnostic; P3.4 pre-pass-only) |

The Beads close + follow-up bead from spec §6 are correctly placed in
Item 6 step 8 (operational), not Item 5. Item 5 step 2 (spec status line →
implemented with the five commit SHAs; §7 and archives untouched) and step 3
(full gates incl. `just test -p codex-model-provider`; no `--all-features`;
no full `just test` without asking — consistent with the repo rule) are
faithful to T5's "then at the end" gate run.

### 1(e) Item 6 vs spec §5 steps 1-7

1:1 correspondence, verified step-by-step:

| Spec §5 | Breakdown Item 6 |
|---|---|
| 1. Build release `codex-cli` from this branch | step 1 |
| 2. New name (e.g. `codex-aplfix1`); production untouched; rollback in `codex-bin-backups/` | step 2 (+ open question 4: confirm rollback state with operator first) |
| 3. Wiretap (`python3 ~/bin/wiretap.py <port>` + one-turn session pinned) — confirm function tool with new format description, vLLM shims active | step 3 (notes wiretap is passive logging) |
| 4. Exact-failure replay (glm-5.2, new markdown file, H1 first line = precise F1; pass = byte-identical on disk) **plus** raw-format probe (standalone `apply_patch` binary from same release build, scratch dir, F1-shaped raw patch; pass = exit 0 + byte-identical; binary shares `parse_patch` with the function handler) | step 4 — both parts, same pass criteria, same rationale for the probe |
| 5. qwen3.8-27b regression: one Add-File (code file) + one Update-File hunk; pass = both apply cleanly | step 5 |
| 6. Watch metrics: PARSE rate per model (glm-5.2 <10%, ideally ~1%) **and** empty-`{}`-arg rate on glm seats (>1% triggers §3.4 retry re-evaluation + deployment-side #49249 request) | step 6 — both rates and both thresholds present |
| 7. Cutover only on operator greenlight; previous binary stays as rollback | step 7 |

Plus step 8 (Beads close `apex-ayl.52` with the evidence pointer list from
spec §6, and the optional #49249 follow-up bead). Item 6 is correctly
marked operational/post-commit, coordinator-executed with the operator in
the loop, and commits no product code.

## Mandate 2 — Red-state ledger correctness

Open the spec §3.3 enumeration and the actual test files at HEAD, then the
ledger (breakdown §1). Result: **exactly 13 red assertions; the ledger's
per-assertion mapping is correct; the surviving list is accurate except the
scenario count (K-M2) and one file path (K-M1).**

### The 13 assertions, verified line-by-line at HEAD

| # | Ledger row (breakdown §1) | Verified at HEAD | New expectation ↔ P-site |
|---|---|---|---|
| 1 | `test_streaming_patch_parser_returns_errors` (streaming_parser.rs:814), assert :828 | :814 fn; :828 = the StartedPatch `'bad'` `InvalidHunkError` message assert (push_delta `*** Begin Patch\nbad\n`) | old message + §3.3.1 appended guidance = **P3.1** (site at :193, sentence extension) ✓ |
| 2 | same, assert :838 | :838 = the AddFile `'bad'` assert (push_delta `*** Begin Patch\n*** Add File: file.txt\nbad\n`) | behavior removed by P2 → `Ok`, contents `bad\n` = **P2** ✓ |
| 3 | same, assert :848 | :848 = the DeleteFile `'bad'` assert | §3.3.2 replacement message = **P3.2** (site at :222, wholesale replacement — shares no prefix with the old text) ✓ |
| 4 | `test_apply_patch_cli_rejects_invalid_hunk_header` (:386), assert :393 | fn at :386 in `codex-rs/apply-patch/tests/suite/tool.rs`; :393 = the exact `.stderr("Invalid patch hunk on line 2: '*** Frobnicate File: foo' is not a valid hunk header. …\n")` assert. **Ledger's path says `codex-rs/core/tests/suite/tool.rs` — that file does not exist** (K-M1) | CLI stderr of the new StartedPatch message (CLI formats `InvalidHunkError` as `Invalid patch hunk on line {n}: {message}`, lib.rs:378-383) = **P3.1** via the shared path ✓ |
| 5 | `test_parse_patch` (parser.rs:277), assert :281 | :277 fn; :281 = `parse_patch_text("bad", Strict)` → Begin message | §3.3.4a new Begin message = **P3.4** ✓ |
| 6 | same, assert :287 | :287 = `parse_patch_text("*** Begin Patch\nbad", Strict)` → End message | §3.3.4b new End message = **P3.4** ✓ |
| 7-13 | `test_parse_patch_lenient` (parser.rs:558), asserts :579, :594, :609, :624, :628, :635, :639-642 | :579 = Strict `<<EOF` heredoc → Begin; :594 = Strict `<<'EOF'` → Begin; :609 = Strict `<<\"EOF\">` → Begin; :624 = Strict mismatched-quotes → Begin; :628 = Lenient mismatched-quotes → Begin; :635 = Strict missing-closing → Begin; :639-642 = Lenient missing-closing → End (string at :642). That is exactly the spec's "6 Begin (4 Strict heredoc variants + Lenient mismatched-quotes + Strict missing-closing) + 1 End (Lenient missing-closing)" | all = **P3.4** (the pre-pass owns both strings; lenient mode routes through `check_patch_boundaries_lenient` → `check_start_and_end_lines_strict` for these shapes) ✓ |

Total: 3 + 1 + 2 + 7 = 13, matching spec §3.3 line 542 ("Total red
assertions: 13 (3 streaming + 1 CLI + 2 + 7)"). The green `Ok` assertions in
the lenient test (:583, :598, :613) are correctly **not** in the ledger.

### Red-state completeness (no hidden 14th assertion)

Repo-wide greps at HEAD (all `.rs`, excluding target/):

- `"not a valid hunk header"`: implementation sites streaming_parser.rs
  :193/:211/:222 only; test sites :828/:838/:848 (the 3 red),
  apply-patch/tests/suite/tool.rs:393 (red #4), core/tests/suite/
  apply_patch_cli.rs:707 (surviving substring) — nothing else.
- Boundary strings (`must be '*** Begin Patch'` / `must be '*** End
  Patch'`): only streaming_parser.rs (impl :168/:184/:374; test :784/:797/:819
  — all surviving) and parser.rs (impl :268/:271; test :281/:287/:576/:642
  — :281/:287/:642 red via #5-13, :576 is the shared `expected_error`
  binding feeding the 6 red Begin asserts) — nothing else. Matches spec
  §3.3's "repo-wide search … found assertions only in those two parser
  tests plus the streaming-parser tests".
- Old handler message `missing the required `patch` argument`: only the
  implementation site apply_patch.rs:539 — **no test asserts it**, so P3.3
  (split + re-quote) introduces no hidden red assertion.
- Scenario fixtures: the harness (`test_apply_patch_scenarios`,
  apply-patch/tests/suite/scenarios.rs) runs **every** subdirectory of
  `tests/fixtures/scenarios` and asserts only final filesystem state (no
  exit-status or stderr assertions). All Add-File fixtures (001, 002, 011,
  015) use canonical `+`-prefixed content (P2 outcome-neutral); 013 is the
  StartedPatch non-header case (still rejected under P3.1, message-only
  change); 007/009/012 pin the existing Delete/Update existence behavior
  (unchanged). No scenario is sensitive to P2/P3.

### Surviving tests (breakdown §1 list) — verified

- core `apply_patch_cli.rs:707` — `out.contains("is not a valid hunk
  header")` in `apply_patch_cli_rejects_invalid_hunk_header`
  (core/tests/suite/apply_patch_cli.rs:686-711); the test's patch is
  `*** Begin Patch\n*** Frobnicate File: foo\n*** End Patch` — the
  StartedPatch case, as the breakdown states; the substring survives P3.1
  (sentence extension keeps the original text verbatim, spec §3.3.1).
- streaming :819 (Begin message via `push_delta` → NotStarted arm :184,
  untouched), :784 (`finish_requires_end_patch` → `finish()` string at :168,
  untouched), :797 (`rejects_content_after_end_patch` → EndedPatch string at
  :374, untouched) — all green under P2/P3.
- Scenario fixtures: **25 directories on HEAD** (001-024 with a duplicate
  `020_` prefix: `020_delete_file_success` and
  `020_whitespace_padded_patch_marker_lines`; plus `README.md` and
  `.gitattributes` files), each with a `patch.txt`. The breakdown's "24
  golden scenario fixtures" (and "the 24 scenario fixtures" in Item 1) is
  off by one — see K-M2. (The spec's T1.13 states no number, so the "24"
  is a breakdown-introduced count.)

## Mandate 3 — TDD protocol vs AGENTS.md (stage 5 + Rust/test conventions)

Checked the breakdown §0/§4/§8 protocol against the repo-root AGENTS.md
"Working Principles" §0 stage 5 and the Rust/test conventions:

- **Workflow-driven, coordinator never hand-edits product code**: §0
  ("Execution is workflow-driven (AGENTS.md stage 5): the coordinator never
  hand-edits product code; every code edit happens inside a workflow
  subagent") and §4 step preamble ("the coordinator orchestrates; all code
  edits happen inside workflow subagents"). Item 6 (coordinator-executed)
  touches no product code (deployment artifacts + Beads close only). ✓
- **Red → green, one item at a time**: §0 (items strictly sequential;
  "shared test files and the red-state ledger make parallelism unsafe") +
  §4 step 2 (per sub-case: write failing test/rewrite ledger assertion →
  run → confirm RED with exact captured failure → implement minimal change →
  GREEN → item gates). Regression-lock sub-cases (T2.2, T4.1, T4.2, T4.2b)
  are explicitly exempt from prior red with the record obligation — matching
  the spec's "green at write time" designations. ✓
- **Per-item multi-agent review LOOP until a full round is 0B+0M, commit
  only after**: §4 step 4 ("dispatch MULTIPLE independent review agents …
  on any Blocking/Major: fix (inside a workflow subagent) → re-run gates →
  FRESH review round → repeat until a full round returns 0 Blocking + 0
  Major") then step 5 (commit). Every item's DoD ends with "item review
  loop 0B+0M; one commit". ✓
- **Re-review after every fix**: the "FRESH review round" after each fix,
  explicitly (not a single pass). ✓
- **Captured red+green evidence per sub-case**: §4 step 2 ("No sub-case is
  'done' without captured red and green output") + §8 execution-log columns
  (red evidence / green evidence / review rounds (0B+0M at) / commit). ✓
- **Per-round records**: §4 step 4 ("Record each round's findings +
  resolutions in `docs/reviews/impl-<item>-r<N>.md`") — auditable
  convergence, as AGENTS.md requires. ✓
- **Review brief severity rules — sane**: Blocking = wrong behavior vs
  spec, invariant violation (OpenAI request bytes changed), or a red tree in
  the commit; Major = spec string drift, missing ledger assertion, unrun
  gate, convention violation CI would reject; Minor/Nit = the rest. The
  "red tree in the commit" clause independently guards the green-tree rule;
  the brief also requires re-derivation at source (open every cited line,
  re-run the test command, run the gates) and diff minimality. ✓
- **Repo conventions carried into the brief**: `just fmt` after every item
  (repo rule), `just test` via the justfile (never bare `cargo test`),
  `just fix -p <crate>` scoping, no `--all-features`, no full `just test`
  without asking, snapshot policy, integration-test conventions
  (`wait_for_event`, `mount_sse_once`), clippy/`format!`/method-ref/
  `/*param_name*/`/exhaustive-match/private-module rules, no new small
  single-reference helpers, module-size targets. ✓

**Holes that would let an unreviewed edit reach a commit: none found.**
The commit is strictly gated: per-sub-case red+green evidence → item gates
green → multi-agent review loop to a full 0B+0M round → one commit. Fixes
always re-enter the loop as a fresh round. Item 5 (docs + spec status line)
is not carved out of §4 — it runs the same loop over its diff. Item 6
commits nothing.

Two protocol ambiguities worth fixing (both Minor; neither opens a
commit hole because the green-gate + review-loop + "red tree in the commit
= Blocking" clauses are the actual gate):

- K-M4: Item 3's "After each sub-step: run the item gates; never leave the
  tree red at a sub-step boundary that ends with a gate run" is
  self-contradictory mid-Item-3 (after sub-steps 3-5 the tree is
  deliberately red for the not-yet-landed P3.2/P3.4 assertions, so a full
  `just test -p codex-apply-patch` cannot pass). Literal reading is
  unsatisfiable; it needs the qualified reading: mid-item gate runs may
  show exactly the not-yet-implemented ledger assertions failing; full
  green gates are required before the review loop and commit.
- K-M5: Item 3 steps 5 vs 7 read as implementation-before-test for T3.1
  ("Land P3.3 … → new T3.1 tests (see below) green" precedes the T3.1 test
  description in step 7). §4 step 2 mandates captured RED before
  implementation; the item steps should say "write T3.1 tests → RED
  (current collapsed message for both shapes) → land P3.3 → GREEN".

## Findings

### K-M1 (Minor) — Ledger row 4 cites a non-existent file path

- Where: breakdown §1, ledger row 4 — "`test_apply_patch_cli_rejects_invalid_hunk_header`
  (`codex-rs/core/tests/suite/tool.rs`:386)".
- Evidence: `codex-rs/core/tests/suite/tool.rs` does not exist (164 files in
  that dir, no `tool.rs`; the only `test_apply_patch_cli_rejects_invalid_hunk_header`
  in the repo is `codex-rs/apply-patch/tests/suite/tool.rs:386`, which is
  exactly where the spec cites it — spec §3.3 and T1.12). The assert line
  :393, the fn name, and the owning item are all correct; only the crate
  prefix is wrong (the breakdown's own Item 3 "Files touched" list correctly
  says `tests/suite/tool.rs`, relative to the apply-patch crate).
- Impact: the hazard is conflation with the same-based-name core-suite test
  `apply_patch_cli_rejects_invalid_hunk_header`
  (core/tests/suite/apply_patch_cli.rs:686-711, which contains the :707
  surviving substring assertion). A subagent following the ledger path,
  failing to find the file, and grabbing the core test instead would rewrite
  the wrong assertions and leave ledger #4 (the exact-stderr CLI test) red
  at Item 3's end — the item gates would then block the commit (so the worst
  case is a stalled workflow, not a red commit or wrong implementation),
  but it is exactly the kind of locator error the "re-verify at HEAD" step
  exists to catch, and it contradicts the breakdown's "all verified at HEAD"
  header.
- Resolution: correct the path to
  `codex-rs/apply-patch/tests/suite/tool.rs`:386 and add a one-line note
  distinguishing it from the core-suite test of the same base name (the
  surviving :707 substring test).

### K-M2 (Minor) — "24 golden scenario fixtures" is 25 at HEAD

- Where: breakdown §1 surviving list ("the 24 golden scenario fixtures") and
  Item 1 TDD step 5 ("T1.13 golden scope: the 24 scenario fixtures …").
- Evidence: `codex-rs/apply-patch/tests/fixtures/scenarios/` contains **25**
  fixture directories at HEAD — 001-024 with a duplicate `020_` prefix
  (`020_delete_file_success`, `020_whitespace_padded_patch_marker_lines`) —
  plus `README.md` and `.gitattributes`. The harness
  (`test_apply_patch_scenarios`, scenarios.rs) runs every subdirectory, so
  25 fixtures execute. All 25 have a `patch.txt`. The spec's T1.13 states
  no number; "24" is breakdown-introduced, and the breakdown's "all
  verified at HEAD" header doesn't cover it.
- Impact: none on the expectation (T1.13 is "pass unmodified"); a subagent
  who treats "24" as an acceptance count is off by one and may mis-attribute
  a passing run. Low, but it is a stated HEAD fact that is wrong.
- Resolution: say "25 scenario fixture directories (001-024; note the
  duplicate 020_ prefix)" or simply "all scenario directories under …".

### K-M3 (Minor) — T2.3 instructs an insta update for a non-insta test

- Where: breakdown Item 2, TDD step 5 (T2.3): "update the snapshot
  (`just test -p codex-core apply_patch` → `cargo insta pending-snapshots`
  → review → `cargo insta accept -p codex-core` scoped to this test's
  snapshot)".
- Evidence: `create_apply_patch_function_tool_matches_expected_spec`
  (apply_patch_spec_tests.rs:40) is a plain `assert_eq!` against an inline
  `ResponsesApiTool` literal (tool description + `patch` parameter
  description); the file has no `insta` usage and no `.snap` file exists
  under `codex-rs/core/src/tools/handlers/`. The spec's T2.3 ("updated to
  the new exact text — an intentional change, not a regression") is
  satisfied by editing that inline literal.
- Impact: the subagent runs a no-op insta flow (no pending snapshots exist
  for this test) and the breakdown gives no correct mechanism; worst case it
  accepts unrelated pending snapshots in the crate. The expectation itself
  (test updated to the spec §3.1 exact text) is correct, so this is a
  misleading step, not a mis-mapping.
- Resolution: replace the insta steps with "update the inline expected
  `ResponsesApiTool` literal (description + `patch` parameter description)
  to the spec §3.1 exact text; it is an `assert_eq!` test, not an insta
  snapshot".

### K-M4 (Minor) — Item 3 gate instruction is self-contradictory mid-red-window

- Where: breakdown Item 3, TDD step 6: "After each sub-step: run the item
  gates; never leave the tree red at a sub-step boundary that ends with a
  gate run (the red window is assertion-rewrite → site implementation
  within one item, as designed)."
- Evidence: after Item 3 step 2, twelve ledger assertions are red by design;
  after step 3 (P3.1) only #1/#4 are green; after step 4 (P3.2) #3 green;
  #5-13 stay red until step 6 (P3.4). A full `just test -p codex-apply-patch`
  run mid-item therefore cannot pass, so "run the item gates" after each
  sub-step is literally unsatisfiable while the constraint forbids ending a
  gate-run boundary red.
- Impact: a subagent following the item steps literally either blocks itself
  or interprets the mid-item red gate as an implementation failure. The
  intended (correct) reading — mid-item gate runs may show exactly the
  not-yet-landed ledger assertions failing, and full green gates are
  required before the review loop/commit — should be the written one.
- Resolution: reword step 6: "After each sub-step, run the item test
  commands; until step 6 the only permitted failures are the ledger
  assertions of sites not yet landed (name them per sub-step). Full-green
  item gates are required before the review loop and commit."

### K-M5 (Minor) — Item 3 step order reads implementation-before-test for T3.1

- Where: breakdown Item 3, TDD steps 5 and 7 (step 5: "Land P3.3 (split
  handler messages) → new T3.1 tests (see below) green"; step 7: the T3.1
  test description).
- Evidence: read literally, the P3.3 implementation lands before the T3.1
  tests are written, which inverts §4 step 2's "write the failing test →
  confirm RED → implement" for that sub-case (the captured-RED obligation:
  "No sub-case is 'done' without captured red and green output"). The red
  for T3.1 is straightforward (current collapsed handler returns the same
  old message for absent and non-string `patch`), so red-first is cheap and
  should be stated.
- Impact: a subagent following the item steps over §4 could commit T3.1
  without red evidence; §4 governs and a reviewer re-checks the evidence,
  but the document should not contain an ordering that contradicts its own
  protocol.
- Resolution: reorder/reword — "7a. Write the T3.1 tests (absent → exact
  §3.3.3a; non-string → exact §3.3.3b) → RED under the current collapsed
  message; 5. land P3.3 → GREEN".

### K-M6 (Minor) — §5 invariant bullet over-claims: "No existence check on any path (… Delete-File never reads)"

- Where: breakdown §5, bullet 4.
- Evidence: spec §3.4's decision is Add-File-specific ("No existence check
  for Add-File on any path", line 556). At HEAD the DeleteFile apply branch
  (lib.rs:536-552) does read: `note_existing_path_delta_support`,
  `fs.read_file_text` (deleted-content capture for the delta), and
  `ensure_not_directory`; deleting a missing file fails
  (scenario 007_rejects_missing_file_delete), and Update requires an
  existing file (scenario 009; VERIFY "Failed to read file" class). So both
  "No existence check on any path" (as a global statement) and "Delete-File
  never reads" are wrong at the source.
- Impact: item reviewers are told to check this invariant; a false
  current-state clause can misdirect a review (e.g. flagging legitimate
  Delete/Update behavior) or mislead the implementer about existing
  semantics. It does not change what this work implements (the real
  invariant — no new existence check added on any path — is intact in the
  spec and elsewhere in the breakdown's Item 3 scope line).
- Resolution: align with spec §3.4 wording: "No existence check is added on
  any path — Add-File to an existing file overwrites, as today (pinned by
  `apply_patch_cli_add_overwrites_existing_file` and scenario 011);
  Update/Delete keep their existing existence behavior".

### K-N1 (Nit) — "≈60×" vs the exact 58.0× (inherited from spec §1.2)

- Where: spec §1.2 Conclusion (lines 208-210), inherited by the breakdown
  only indirectly (the breakdown quotes the pinned 26/37 ≈ 70% and
  5/413 ≈ 1% figures in Item 5, not the ratio).
- Evidence: 26/37 = 70.27% and 5/413 = 1.21% are exact; their ratio is
  58.04×, while the R5 map (line 1018) and round-5 summary (line 989) say
  "≈ 58×". "≈60×" overstates by ~3.4% and is internally inconsistent with
  the same document's ≈58×.
- Impact: none on the breakdown's correctness; recorded because Mandate 0
  asked for the arithmetic check and the residual is a spec-side wording
  nit.
- Resolution (spec-side, optional): "≈58× (≈60×)".

### K-N2 (Nit) — ResponseMock range cite ends 2 lines short

- Where: breakdown Item 4, TDD step 1 ("`ResponseMock` `.single_request()`/
  `.requests()`, `responses.rs`:39-58").
- Evidence: struct `ResponseMock` :39-41; `single_request` :50-56;
  `requests` :58-60 (responses.rs). The range covers both method starts but
  ends 2 lines before `requests()` closes.
- Impact: cosmetic.
- Resolution: cite ":39-60" or cite by name only.

### K-N3 (Nit) — §8 execution-log updates land in the next item's commit unreviewed

- Where: breakdown §4, TDD step 6 ("Update this breakdown's §8 execution
  log, then start the next item") — after step 5's commit.
- Evidence: the log row is written post-commit, so it rides into the next
  item's commit; the next item's review scope is "the item's diff (files
  listed in the item)", which does not include the breakdown doc.
- Impact: none practical (the rows are factual pointers to the
  `docs/reviews/impl-<item>-r<N>.md` records, which are themselves
  reviewed artifacts), but as written a doc edit reaches a commit outside
  any review loop.
- Resolution: either include the §8 row in the item's diff before its
  commit, or state explicitly that §8 log rows are coordinator bookkeeping
  exempt from the item review loop.

## Re-verified clean

**Mandate 0 (v5 fixes)** — all five, with source evidence:
- I-M1: spec §1.2 line 165 + R5 map line 1015; both named calls located
  (`call_10f109a3…` 09-14, `call_8ee1b0c1…` 09-15), both at/before the glm
  pin, both patch final lines byte-exactly `+*** End Patch`; pre-pin sweep
  finds exactly these 2 missing-End and exactly 5 missing-Begin glm
  apply_patch calls.
- I-N3/J-M1: spec §1.2 line 143 carries `Failed to find context`
  (file_update.rs:110); file_update.rs:110 produces that string; both
  named calls (`call_a9cf9f97…` 23:56:08.699Z, `call_69f201be…`
  23:56:49.864Z) carry it in their output and sit after the qwen pin.
- J-N1: `ev_function_call` spans responses.rs:933-:943 at HEAD; spec T4.2
  (line 711) and R2-map D-M2 row (line 882) cite :933-943; breakdown Item 4
  step 2 cites :933-943.
- I-N1: percentages exact (70.27% / 1.21%); fix direction correct; residual
  "≈60×" vs "≈58×" wording logged as K-N1.
- I-N2: R3-map E-N1 row (line 923) opening restated; no leftover
  "Snapshot pinned (2026-09-15T23:30:11Z, 83/162/86 rollout files)"
  pairing; 83/162/86 appears only in the mid-row v3-superseded annotation
  and in the correctly-paired §1.2 Freshness bullet.

**Mandate 1** — spec↔breakdown conformance:
- (a) All sub-cases T1.1-T1.14 (incl. 9b), T2.1-T2.4, T3.1-T3.2,
  T4.1/T4.2/T4.2b, T5 appear exactly once in the correct item with the same
  expectation (table in 1(a)).
- (b) Item split internally consistent; per-commit green-tree logic holds —
  no item commits a red tree; Item 3's 12-assertion red window opens and
  closes inside Item 3 (table in 1(b)); red-state share column sums to 13.
- (c) Invariants match spec §3.4 in substance on every clause except the
  K-M6 clause (request-bytes invariant; no Update leniency; no strict:true;
  no retry; no per-model modes; P3.4 pre-pass-only with :168/:184/:374
  verified; no new public API; lock-file rule).
- (d) Item 5 seam-doc list = spec §6 list, item-for-item (6/6) + Beads
  close correctly in Item 6.
- (e) Item 6 = spec §5 steps 1-7 one-for-one (build; new name + rollback
  backup; wiretap; F1 replay + raw-format probe; qwen regression; both
  watch metrics with thresholds; operator-gated cutover) + Beads close.

**Mandate 2** — red-state ledger:
- Exactly 13 red assertions at HEAD, line-verified (3 streaming
  :828/:838/:848; 1 CLI :393; 2 `test_parse_patch` :281/:287; 7
  `test_parse_patch_lenient` :579/:594/:609/:624/:628/:635/:639-642).
- Per-assertion new-expectation mapping correct (#1→P3.1 sentence
  extension; #2→P2 `Ok` `bad\n`; #3→P3.2 replacement; #4→P3.1 via CLI;
  #5-13→P3.4 §3.3.4a/b), including the lenient-test composition
  (6 Begin + 1 End as the spec enumerates).
- Red-state completeness: repo-wide string sweeps prove no 14th red
  assertion (P3.3's old handler message is asserted by no test; all
  boundary-string and hunk-header-string occurrences are accounted for).
- Surviving list accurate: core :707 substring test (StartedPatch case;
  substring survives P3.1), streaming :819/:784/:797 (their strings at
  :184/:168/:374 untouched), scenario fixtures P2/P3-outcome-neutral.
  Exceptions logged: K-M1 (ledger path), K-M2 (24 vs 25 count).

**Mandate 3** — TDD protocol:
- Workflow-driven (coordinator never hand-edits), one item at a time,
  red→green with captured evidence per sub-case, per-item multi-agent
  review loop until a full round is 0B+0M, commit only after, fresh review
  round after every fix, per-round records (`docs/reviews/impl-<item>-r<N>.md`),
  §8 execution log — all present in §0/§4/§8; review brief severity rules
  sane (incl. "red tree in the commit" as Blocking). No path found by which
  an unreviewed edit reaches a commit.

**Line-reference sweep (supporting evidence)** — verified at HEAD, all
correct unless noted: streaming_parser.rs :168/:184/:193/:198-215
(arm :198, header check :199, `+` branch :202-208, trailing Err
:209-214)/:222/:374, tests :784/:797/:814-852; parser.rs :56-57
(`#[error("invalid patch: {0}")]` + variant)/:145/:193-199/:256-274
(messages :268/:271)/:277/:281/:287/:558/:576/:579-642; lib.rs :370/:378-383
(CLI `Invalid patch hunk on line {n}: {message}`)/:536-552 (DeleteFile
branch); invocation.rs :116/:123/:170/:175; file_update.rs :110;
model-provider-info/src/lib.rs :40/:546-547; spec_plan.rs :1257-1271;
provider.rs :372; apply_patch.rs :411/:508-560 (chain :534-541, message
:539)/:603-605; registry.rs :548-556 (message :550); responses.rs :39-60
(K-N2)/:933-943/:1025-1028/:1030-1035; test_codex.rs :347/:839-859;
apply_patch_cli.rs :228/:323/:686-711; apply_patch_tests.rs :45;
session/tests.rs :5882; apply_patch_spec_tests.rs :40;
apply-patch/tests/suite/tool.rs :386/:393 (ledger path defect K-M1);
scenarios.rs harness (runs every subdirectory); `handle_hunk_headers_and_end_patch`
(:84) structural-line semantics (Environment ID honored only in
StartedPatch; trimmed `*** End Patch` structural; marker-looking content
lines stay structural). P1 text: 126 + 2,198 = 2,324 chars; 21 + 380 = 401
words (exact); all 7 drift-guard substrings present verbatim; T2.2
extraction rule sound (single `Example:` line, index 19; remainder ends
with the `*** End Patch` line).

## Counts

| Severity | Count | IDs |
|---|---|---|
| Blocking | 0 | — |
| Major | 0 | — |
| Minor | 6 | K-M1, K-M2, K-M3, K-M4, K-M5, K-M6 |
| Nit | 3 | K-N1, K-N2, K-N3 |

**Verdict: APPROVED — 0 Blocking, 0 Major, 6 Minor, 3 Nit.** Per the
campaign convention (round-5 spec seats: APPROVED with minor/nit), the
stage-4 termination condition (zero Blocking + zero Major) is met.
Recommendation: coordinator applies K-M1..K-M6 (documentation-only edits to
the breakdown; K-N1 is spec-side and optional) in a recorded v5.1 pass
before launching the stage-5 workflow; K-M1 and K-M3 are the two most
subagent-impactful and should be fixed first.

— Seat K (leaf reviewer; no subagents; snapshot /tmp/rout_snap_k1 taken
2026-09-16T02:47:34Z; no files modified other than this report)
