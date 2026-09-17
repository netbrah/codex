# Item 5 — Docs, gates, spec final pass: review round 3, SEAT E (from-scratch re-review + r2 resolution verification)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` (HEAD `799ec98053`) · Date: 2026-09-16
Reviewer: `/root/item5_r3_seat_e` (independent; r1 seats A/B, r2 seats C/D on record)

## Method

- Read both artifacts in full from scratch (seam doc 297 lines; evidence doc
  621 lines incl. r1+r2 resolution logs), then verified every r2 "FIXED"
  claim against the artifact text and every load-bearing factual claim
  against source (direct file reads, `grep -n`, `git show --stat`, `diff -u`).
  Did not trust the resolution log.
- Gates: not re-run (mandate: static verification only; Mac load-constrained,
  SCS release build in flight). Attestation checked for internal consistency
  and against r1/r2 seats' independent gate runs.
- READ-ONLY: no product code touched, no artifact edits, no git write ops.

## Verdict

**APPROVE — 0 Blocking / 0 Major / 0 Minor / 0 Nit.**

Round meets the clean bar (0 Blocking AND 0 Major). All seven r2 findings
verified fixed at the cited locations; every from-scratch source check
reproduces the artifacts' claims; the spec-vs-v4-archive hunk discrepancy
(r2 C: 10 vs r2 D: 8) is adjudicated to 8 under standard `diff -u` with all
hunks explained (see below).

## r2-fix verification (per ID, verified at artifact text + source)

| ID | Claim | Verified against artifact | Result |
|---|---|---|---|
| F-C1 (Major) | Decision sentence subject scoped; no Azure-untouched assertion remains in the sentence; no "OpenAI/Azure" string anywhere in the doc; consistent with §2 and invariants bullet 1 | Seam :81-86 now opens the clause "Providers named `OpenAI` are untouched: they keep the native custom+grammar path (spec §3.4 invariant: outbound request bytes for `OpenAI`-named providers are unchanged; `is_openai()` is name-anchored, so Azure-*named* providers take the function tool, as pinned by the capability test)". The parenthetical asserts the OPPOSITE of Azure-untouched. `grep -n "OpenAI/Azure"` over the whole seam doc: **0 hits** (exit 1). §2 gate semantics (:53-61) and invariants bullet 1 (:277-280) both say Azure-*named* providers fail the gate / take the function tool — same direction, no contradiction. Source agrees: `provider.rs:372` `apply_patch_function_tool: !self.info.is_openai()`; capability test `provider.rs:707` pins Azure-named → `true` | **Fixed** |
| n-D5 (nit) | Same Decision sentence, seat D's independent phrasing | Same edit as F-C1; re-verified above | **Fixed** |
| n-D3 (nit) | Helper name `run_apply_patch_text`; real fn at `apply_patch.rs:400`, callers :381/:555 | Seam :117-119: "extract the post-parse body of the existing `handle_call` into a private `run_apply_patch_text(...)` used by both handlers". Source: `grep -n 'run_apply_patch_text'` → exactly `381:` (custom handler call), `400:` (`async fn run_apply_patch_text`), `555:` (function handler call). Old name `run_patch_text` absent | **Fixed** |
| n-D4 (nit) | Two quoted phrases byte-exact from `apply_patch_spec.rs` and spec §3.1 | Seam :95-101 quotes (a) "The complete patch goes in the `patch` argument." and (b) "The ENTIRE patch as a single string". (a) is the byte-exact tail of `apply_patch_spec.rs:37` (`APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION`) and of spec :296; (b) is the byte-exact opening of `apply_patch_spec.rs:42` (`APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION`) and of spec :302. Anti-wrapping wording is now UNQUOTED parenthetical ("no markdown fences, code-block markers, or shell heredoc wrapper") matching the source rule "Do not add markdown fences, code-block markers, or a shell heredoc wrapper around the patch." — no quotation marks around a paraphrase anywhere in the bullet. The false composite quote ("put the ENTIRE patch text in `patch`; do not wrap in JSON or markdown") is gone (grep: 0 hits). Full P1 constants verified byte-identical to spec §3.1 through the FORMAT section header line; the remainder is drift-guarded in `apply_patch_spec_tests.rs` (ran green in gates) | **Fixed** |
| M-D1 (minor) | False equation "70.27 / 1.21 = 58.04" (and variants) absent except the intentional quoted-defect citation in the Round-2 record; correct phrasing = 10738/185 = 58.0432 (≈58×) | `grep -n "58\.04"` over the whole evidence doc — all 7 hits classified: :191 (Changes #8), :219 (Factual verification), :512 (Worker attestation), :545 (r1 record n-F4) all read the correct exact ratio "10738/185 = 58.0432"; :546 (r1 record n-F4) carries the quoted token "58.04" as a historical record — arithmetically sound (58.04324… rounds to 58.04 at 2dp) and NOT the false equation (no "70.27/1.21 = 58.04" assertion); :600 (r2 record M-D1) is the intentional quoted-defect citation 'the false quotient "70.27 / 1.21 = 58.04"'; :602 reads the correct replacement statement. `grep -n "70\.27 *\/\|/1\.21"`: hits are the correct rounded-division note "70.27/1.21 = 58.07" (:220, :604) and the spec §7 R5-map quote "70.27% / 1.21% ≈ 58×" (:221) plus the :600 citation — no "58.04" on the right-hand side anywhere outside :600 | **Fixed** |
| M-D2 (minor) | "Final state" addendum: 12-file/809+/20− claims as of pre-baseline tree (HEAD `3db1b1381a`); baseline commit `799ec98053` committed exactly that delta; item-5 docs still untracked | Addendum present in Final state. `git log --oneline -3` → `799ec98053`, `3db1b1381a`, `a90b15d600` (order and subjects match). `git show --stat 799ec98053`: **12 files, 809 insertions(+), 20 deletions(-)** — identical to the attested delta; file list matches the evidence doc's named 12 files 1:1 (AGENTS.md, content_type_compat{,_tests}.rs, responses.rs, handlers/mod.rs, spec_plan.rs, amazon_bedrock/mod.rs, provider.rs, models.json, manager_tests.rs, model_info{,_tests}.rs). Both item-5 artifacts + this review doc are `??` (untracked) in `git status --short`. The "~14s after this doc's last write" sub-claim is not independently re-derivable (original mtime superseded by in-place r2 edits; file mtime now post-commit), but every load-bearing half of the addendum is verified above | **Fixed** |
| F-C2 (nit) | Seam-doc stats bullet reads 297 lines with full accounting (226 → 287 → 294 → 297) and post-r2 anchors | Final-state bullet: "297 lines (item-5 pre-edit 226; +61 → 287 at this doc's writing; r1 fixes +7 (M-A1 +4, M-A2 +2, N-1 +1) → 294; r2 fixes +3 (F-C1/n-D5 −1, n-D3 +4, n-D4 line-neutral) → 297)" — arithmetic closes (226+61=287, +7=294, −1+4+0=+3, 294+3=297). `wc -l` = **297** ✓. Post-r2 anchors re-measured this round — all six exact: §2 gate semantics :53-61 ✓; §3 "Format-remediation layer (P1/P2/P3)" :133-164 (header :133, last content line :164, `### Testing` :166) ✓; §4 row 7 :196 ✓; §6 conflict-site bullets :243-246 ✓; §6 gate bullets :253-259 ✓; §6 invariants bullet 2 :281-292 ✓ (bullet 1 :277-280, as cited by F-C1's own resolution text). Spec anchors also re-measured: Status :7-14 with `<<ITEM5-SHA>>` at :10 ✓; §1.2 Conclusion `≈58×` at :213 ✓; §7 I-N1 row at :1022 ✓; `wc -l` = 1029 ✓ (pre-edit 1026; +3) | **Fixed** |

## From-scratch review

### Seam doc vs source (fresh pass, beyond r2 fixes)

- **§2 capability table + gate semantics** — all three gates `!is_openai()`
  match `provider.rs` computation (`flatten_namespace_tools`/
  `normalize_content_types`/`apply_patch_function_tool` at :369/:371/:372,
  fields :61-63, defaults :74-76). Commit SHAs `b4d4b12`→`b4d4b125cc`
  ("feat: normalize content types for non-OpenAI providers (vLLM/SGLang)",
  2026-08-16) and `197ea16`→`197ea1642c` ("feat: flatten namespace tools for
  non-OpenAI providers (vLLM/SGLang)", 2026-08-16) exist in `git log` with
  subjects matching the table's effects; touched file sets match §4 rows 1-2
  (`endpoint/{mod,responses}.rs`, `client.rs`, `provider.rs` / `tool_spec.rs`,
  `tool_name.rs`, `router.rs`, `client.rs`). `is_openai()` refs exact:
  `model-provider-info/src/lib.rs:40` `const OPENAI_PROVIDER_NAME: &str =
  "OpenAI";`, `:546-547` `pub fn is_openai(&self) -> bool { self.name ==
  OPENAI_PROVIDER_NAME }` — plain `&str ==`, exact/case-sensitive;
  `grep -ci azure` over the file = **0** (no Azure branch in the gate).
  Record note: `provider.rs:361` uses `is_azure_responses_provider` for
  `remote_compaction` — upstream mechanism, separate from the capability
  gate; contradicts no seam claim (same note as r2 seat C).
- **§3 Components vs code** — (1) capability field `provider.rs:63`;
  (2) `create_apply_patch_function_tool(include_environment_id: bool) ->
  ToolSpec` at `apply_patch_spec.rs:79`, `strict: false` at :97, both quoted
  description phrases byte-exact (n-D4 above); (3) `FunctionApplyPatchHandler`
  at `apply_patch.rs:480`, `CoreToolRuntime::matches_kind` at :607,
  `ToolPayload::Function` argument parsing with missing-`patch`/invalid-JSON →
  `FunctionCallError::RespondToModel` (:525-542), shared `run_apply_patch_text`
  delegation (:381/:555 → :400); (4) registration at `spec_plan.rs:1257-1271`
  — exactly the `environment_mode.has_environment() &&
  context.model_info.apply_patch_tool_type.is_some()` site, capability-gated
  if/else, exactly one handler registered. All four component bullets hold.
- **§3 Format-remediation layer vs code** — P1: full format taught in the
  function-tool `patch` argument description (non-OpenAI path only),
  `apply_patch_spec.rs:42` const, verbatim from spec §3.1, drift-guarded in
  `apply_patch_spec_tests.rs`. P2: `streaming_parser.rs` module doc :3
  ("Lenient add-file content (P2)…") + the AddFile arm :209-222 — `+`-prefixed
  lines stripped, non-structural lines (raw/empty/whitespace) appended
  verbatim: exactly "raw/empty/whitespace content lines accepted verbatim",
  provider-agnostic (no provider/model check in the arm). P3: teachable
  errors at the named sites — bad hunk header message :200-205 (lists valid
  headers + guidance), Delete-File content error :225-230, P3.3 absent/
  non-string `patch` handler errors `apply_patch.rs:525-542`, P3.4 `parse_patch`
  pre-pass boundary strings `parser.rs:269/272/282/288` ("The first line of
  the patch must be '*** Begin Patch'…" / "The last line of the patch must be
  '*** End Patch'…"). Pinned numbers in the evidence table (glm 37/26/~70%
  (70.27%) "as of the pinned snapshot"; qwen 413/5/~1% (1.21%); 09-15 alone
  13/21 ≈ 62%) reproduce spec §1.2 pinned table rows :162-163 (:151-173
  region), the :173 "13 of 21 glm calls rejected (~62%)" sentence, and the
  :211-213 Conclusion "≈58× qwen's ~1% (26/37 = 70.27% vs 5/413 = 1.21%)"
  exactly.
- **§4 divergence rows vs commits** — row 7 file attribution re-verified via
  `git show --stat`: `e21f608ac4` (P2: streaming_parser.rs + parser.rs +1
  module-doc + new test file), `a4d5af1f62` (P1: apply_patch_spec.rs + tests),
  `939a6dc6f4` (P3: parser.rs + streaming_parser.rs + apply_patch.rs +
  tests) — every file named in row 7 and the §6 conflict-site bullets carries
  the change; nothing over-attributed. Row 4 verified: `model_info.rs:140`
  `apply_patch_tool_type: Some(ApplyPatchToolType::Freeform)` fallback.
- **§6 conflict sites + gate list vs breakdown "Item 5" (:327)** — both
  conflict-site bullets present with the spec/task-locked `codex-rs/`
  spelling (D1 accepted state); both gate bullets present
  (`just test -p codex-apply-patch` + raw-format probe per spec §5.4) —
  1:1 with the breakdown's Item-5 step-1 list. Invariants bullet 2 (:281-292)
  vs spec §3.4 :592-612: the three clauses are verbatim-condensed (trims
  "by this work", the diff-consumer parenthetical, "v2 refinement:", the
  `invocation.rs:116/123/170/175` cite; both trailing meta-sentences
  deliberately omitted and accounted for in Changes #6) — faithful, no
  semantic drift; the spec's "no Azure branch" gate clause is carried by
  seam §2 + invariants bullet 1 instead.
- **Invariants vs spec §3.4** — bullet 1 (:277-280) "Outbound request bytes
  for providers named `OpenAI` are byte-identical to upstream (… name-
  anchored on `OpenAI`; Azure-*named* providers take the function tool per
  spec §3.4)" matches the spec invariant + capability-test pin (source
  verified above). Bullet 3/4 (only the three §2 capabilities branch on
  `is_openai()`; new capabilities need a unit test with OpenAI/Azure/custom
  cases) matches the existing test shape at `provider.rs:707-733` (exactly
  those three cases).

### Evidence doc internal consistency

- **Gate arithmetic** — Gate 1 115/115 (log quoted, exit 0). Gate 2
  "118 tests run: 66 passed (2 flaky), 1 failed, 51 timed out, 4251 skipped":
  66+1+51 = 118 ✓; 52 non-passes = 51 TMT + 1 FAIL ✓ and the identity list
  contains exactly **52** `suite::` lines (counted) — all `codex-core::all`
  suite, matching "all load-noise class". 40 lib tests: 118 = 40 lib + 78
  suite, 52 non-pass all suite → 40/40 lib first-try is consistent with the
  66-pass count (40 + 26 suite-pass). Gate 3 "84 tests run: 81 passed (11
  slow, 12 flaky), 2 failed, 1 timed out, 0 skipped": 81+2+1 = 84 ✓; the 12
  flaky + 1 TMT = 13 non-pass ✓; the 2 named amazon_bedrock TLS failures
  match D2. Capability-gate target
  `configured_provider_apply_patch_function_tool_matches_provider_support`
  named at `provider.rs:707` ✓. Gate 4/5 exit-0 records consistent with D3
  (restore verified by r2 seat C; `git diff HEAD --stat` empty this round,
  so no clippy residue remains).
- **Pinned numbers / arithmetic** — 26/37 = 70.270…% ≈ 70% (70.27%) ✓;
  13/21 = 61.9% ≈ 62% ✓; 5/413 = 1.2106% ≈ 1% (1.21%) ✓; exact rate-ratio
  (26/37)/(5/413) = (26·413)/(37·5) = 10738/185 = 58.04324… ≈ 58× ✓; rounded
  70.27/1.21 = 58.0743… ≈ 58.07, which the doc records only as the
  "also rounds to ≈58×" note ✓. All match spec §1.2 (table + Conclusion)
  byte-for-byte in the cited fractions.
- **r1/r2 record sections vs artifact text** — every "FIXED" entry located in
  the current artifacts (F-C1/n-D5 seam :81-86; n-D3 seam :117-119; n-D4
  seam :95-101; M-D1 evidence :191/:219/:512; M-D2 addendum; F-C2 final-stats
  bullet; m-F1 "All 40" at gate-2 section — `grep -n "46 "` hits only the
  quoted pre-fix text in the r1 record :535; N-1 P3 bullet names all three
  files at seam :155-158; N-5 status cell "IMPLEMENTED (items 1-3, 2026-09-16)"
  at seam :51, date source-verified (all four item commits dated 2026-09-16));
  every "ACCEPTED"/"RECORDED" entry matches its recorded state. Cross-check
  note's claims: 40 lib tests ✓ (above); spec-vs-v4 `diff -u` = 8 hunks ✓
  (re-run this round, see adjudication); docs-only tree state ✓ (below).
- **Pre-check anchor math** — spec pre-edit 1026 → post-edit 1029 (+3) with
  status line 4→8 lines (+4, :7-10 → :7-14) and trailing-blank-line
  normalization (−1; `od -c` on spec tail: single `\n`; v4 archive tail:
  `.\n\n`) reconciles exactly; anchor shifts :209→:213 and :1018→:1022 (+4)
  match the +4 status-line delta. Seam pre-edit 226 → 287 (+61) at writing;
  +7 (r1) → 294; +3 (r2) → 297; per-fix line deltas (−1/+4/0) not
  independently re-derivable (pre-r2 file state superseded in place) but the
  totals close and all current anchors measured exact — accepted as recorded,
  consistent with two prior convergent rounds.

## Source verification (consolidated evidence list)

- `model-provider-info/src/lib.rs` — :40 constant, :546-547 fn (single
  grep hits each), 0 case-insensitive azure matches.
- `model-provider/src/provider.rs` — :63 field, :372 gate computation,
  :707-733 capability test with OpenAI(false)/Azure-named(true)/custom(true).
- `core/src/tools/handlers/apply_patch.rs` — :381/:400/:555 shared-path
  refs; :480/:607 handler + matches_kind; :525-542 P3.3 RespondToModel
  errors.
- `core/src/tools/handlers/apply_patch_spec.rs` — :37/:42 P1 constants,
  byte-exact vs spec §3.1 :296/:302 (verified through the FORMAT header
  line); :79/:97 function-tool ctor + `strict: false`.
- `core/src/tools/spec_plan.rs` — :1257-1271 capability-gated registration.
- `apply-patch/src/streaming_parser.rs` — :3 P2 module doc, :209-222 AddFile
  leniency arm, :200-205/:225-230 P3.1/P3.2 teachable messages.
- `apply-patch/src/parser.rs` — :269/:272/:282/:288 P3.4 pre-pass boundary
  strings.
- `models-manager/src/model_info.rs` — :140 Freeform fallback.
- `git show --stat` — `e21f608ac4`, `a4d5af1f62`, `939a6dc6f4`,
  `a90b15d600` (item 1-4 file sets + 2026-09-16 dates); `b4d4b125cc`,
  `197ea1642c` (capability commits, subjects match table); `799ec98053`
  (12 files, 809+/20−).
- `diff -u` spec vs v4 archive — 8 hunks, all classified (next section).

## Docs-only verification

- `git status --short | grep -v '^??'` → **empty** (exit 1); `git diff HEAD
  --stat` → **empty** (at HEAD `799ec98053`). Zero tracked modifications,
  zero staged, zero unmerged.
- Untracked set: item-5 artifacts + campaign docs (`docs/responses-compat-
  seam.md`, `docs/responses-compat-apply-patch-format.md`,
  `docs/reviews/impl-item5-*`, prior-round review docs, spec/breakdown
  archives), `docs/superpowers/`, `docs/vllm-glm-toolcall-research.md`, and
  **1335** `codex-rs/vendor/**` build artifacts (mandate: ignored). No
  non-doc untracked source changes.
- Spec frozen: `docs/responses-compat-apply-patch-format.md` = **1029
  lines** (mandate value); Status :7-14 = IMPLEMENTED with the five item
  SHAs `e21f608ac4`/`a4d5af1f62`/`939a6dc6f4`/`a90b15d600` +
  `<<ITEM5-SHA>>` placeholder at :10; §1.2 Conclusion :213 = `≈58×`; §7
  intact (Round-5 map :1015-1029, I-N1 row :1022 still carries the
  historical "≈60×" log entry; `grep -n "≈60×\|≈58×"` → exactly :213 +
  :1022, matching the evidence doc's own post-edit claim). The v4→spec
  delta contains ONLY the item-5 final pass (2 hunks) + the round-5 v5
  fixes (6 hunks) — no off-contract section touched.

## Hunk-count adjudication (r2 seat C: 10 vs r2 seat D: 8)

Re-ran `diff -u docs/reviews/apply-patch-format-spec-v4-draft.md
docs/responses-compat-apply-patch-format.md` (v4 archive is the closest —
and only — v4/v5-era archive; no v5 archive file exists):

- **8 hunks** at `diff -u` default (U=3), and 8 in the reversed direction.
- Context sensitivity: U=0 → **10**, U=1/U=2 → 8, U=5 → 8, U=10 → 7.

**Conclusion: seat D's 8 is the count under the standard `diff -u`
(mandate command); seat C's 10 equals the distinct-change-site count
(U=0) — the two are the same delta measured at different context widths,
not a real disagreement, and both are "each classified" correctly within
their own frame.** All 8 U=3 hunks are explained; none unexplained:

| Hunk | v4→spec region | Explanation |
|---|---|---|
| 1 | :2-16 header | item-5 final pass: Evidence line gains v4-archive ref; Status SPEC-v4 → IMPLEMENTED with five item SHAs + `<<ITEM5-SHA>>` (evidence doc Changes #7) |
| 2 | §1.2 definition | v5 I-N3/J-M1: VERIFY exemplar list gains `Failed to find context` (file_update.rs:110) |
| 3 | §1.2 glm breakdown | v5 I-M1: "one patch ends with the `+`-prefixed terminator" → both patches (byte-exact re-derivation) |
| 4 | §1.2 Conclusion :210-213 | v5 I-N1 as applied by item-5 step 2: "an order of magnitude above" → "≈58× qwen's ~1% (26/37 = 70.27% vs 5/413 = 1.21%)" |
| 5 | T4.2 cite | v5 J-N1: `responses.rs:933-942` → `:933-943` |
| 6 | R2-map D-M2 row | v5 J-N1: second cite of the same range fix |
| 7 | R3-map E-N1 row | v5 I-N2: row opening restated (superseded pin/triple pairing moved) |
| 8 | §7 tail :974-1029 | v5 loop closure: "Round 5: pending" → full round record (seats I/J, 0B+0M), Round-5 resolution map, "Spec review stage: COMPLETE" line; absorbs the v4 trailing blank line (v5 n-F3 single-`\n` tail) |

Hunks 1 and 4 are the item-5 change set; hunks 2, 3, 5, 6, 7, 8 are the
round-5 v5 fixes — each traceable to a row of the spec's own Round-5
resolution map. No hunk outside those two origin sets.

---

**SEAT E round 3: APPROVE — 0 Blocking, 0 Major, 0 Minor, 0 Nit.** Clean
bar met: all seven r2 findings (F-C1, n-D3, n-D4, M-D1, M-D2, F-C2, n-D5)
verified fixed at the cited locations with source corroboration; the full
from-scratch pass (gate semantics, capability table + SHAs, P1/P2/P3 code
attribution, §6 vs breakdown Item 5, invariants vs spec §3.4, gate
arithmetic, pinned numbers, docs-only tree, frozen spec) reproduced every
load-bearing claim; the 10-vs-8 hunk discrepancy is adjudicated (8 is
correct under standard `diff -u`; 10 is the U=0 change-site count) with
all hunks explained. Item 5 is ready for the coordinator to commit and
fill `<<ITEM5-SHA>>`.
