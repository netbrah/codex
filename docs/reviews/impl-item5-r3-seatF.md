# Item 5 — Docs, gates, spec final pass: review round 3, SEAT F (independent r2-fix verification + fresh eyes)

- **Bead:** `apex-ayl.52` · **Branch:** `feat/normalize-content-types-vllm` @ HEAD `799ec98053`
- **Date:** 2026-09-16 · **Seat:** F (fully independent of the artifacts; r1 not seen; r2 seat C/D reports read only to audit the evidence doc's Round-2 record, per mandate)
- **Mode:** static source verification only (no `cargo`/`just` runs, no git writes, per task constraint); read-only on all artifacts; raw gate logs re-audited statically where they survive.
- **Artifacts under review (all untracked):** `docs/responses-compat-seam.md` (297 lines), `docs/reviews/impl-item5-tdd-evidence.md` (621 lines); frozen SoT `docs/responses-compat-apply-patch-format.md` (1029 lines)
- **SoT:** spec v5 (§1.2, §3.1–§3.4, §5.4, §6, §7), breakdown v3 "Item 5" (`docs/responses-compat-apply-patch-task-breakdown.md:327`), r2 seat reports `impl-item5-r2-seatC.md` / `impl-item5-r2-seatD.md`.

## Method

- Read both r2 seat reports in full first (verdicts, counts, per-finding text), so the Round-2 record could be audited finding-by-finding against the actual seat findings.
- Re-derived every r2 fix at the byte level: programmatic (Python) whitespace-normalized and byte-exact string comparisons for all quoted text; `grep -n` line-anchor re-measurement of every post-r2 anchor; independent arithmetic re-derivation of every quotient/rate.
- Re-verified every load-bearing source claim at the current tree (git log/show, direct file reads at cited lines).
- Re-derived the gate-attestation sums and the docs-only tree state at this seat.

## A. r2-fix verification (per ID)

**A1. F-C1 (seat C, Major) + n-D5 (seat D, nit) — Decision sentence — FIXED, VERIFIED.**
`docs/responses-compat-seam.md:81-85`. The sentence now opens with the required
subject-scoped clause; programmatic check: the exact string
"Providers named `OpenAI` are untouched: they keep the native custom+grammar path
(spec §3.4 invariant: outbound request bytes for `OpenAI`-named providers are
unchanged; `is_openai()` is name-anchored, so Azure-*named* providers take the
function tool, as pinned by the capability test)"
is present verbatim (whitespace-normalized; doc is hard-wrapped at ~72 cols).
Sweep of the whole doc: `OpenAI/Azure` = **0 hits** (was the r1/r2 defect); the
only other `untouched` hit is :271 (production-binary step, unrelated); every
Azure mention (:14 deployment name, :56/:58 §2, :84 Decision, :280 invariants
bullet 1, :296 test-case list) is correctly scoped — no remaining claim that
Azure-*named* providers are untouched. Consistency with the same doc:
- §2 gate semantics (:53-61): "exact, case-sensitive match of the provider
  `name` … against `OPENAI_PROVIDER_NAME` … There is **no Azure branch** …
  any Azure-*named* provider fails the gate and is in seam scope (function
  tool + P1 format text)".
- Invariants bullet 1 (:277-280): "Outbound request bytes for providers named
  `OpenAI` are byte-identical to upstream (all capability flags false on
  `is_openai()`, which is name-anchored on `OpenAI`; Azure-*named* providers
  take the function tool per spec §3.4)".
All three statements agree (name-anchored gate; OpenAI-named unchanged;
Azure-named takes the function tool). "as pinned by the capability test"
re-verified at source: `configured_provider_apply_patch_function_tool_matches_provider_support`
(`model-provider/src/provider.rs:707`) carries the `name: "Azure"` case with
`assert_eq!` on `apply_patch_function_tool`. No new claims introduced by the fix.

**A2. n-D4 (seat D, nit) — helper name — FIXED, VERIFIED.**
Seam `:116-118` now names `run_apply_patch_text(...)`. Source:
`core/src/tools/handlers/apply_patch.rs` — fn def at **:400**, call sites
**:381** (custom handler) and **:555** (function handler); repo-wide grep for
the old `run_patch_text` returns only the r2-record rename description
(evidence :585) and the seat-D report (historical). Exactly the line refs the
Round-2 record states (`:400`, `:381`, `:555`).

**A3. n-D3 (seat D, nit) — composite quote → byte-exact quotes — FIXED, VERIFIED.**
Seam Components spec bullet (:97-102). Programmatic byte comparison (not by
eye):
- Quote 1: "The complete patch goes in the `patch` argument." — appears
  exactly once in the seam doc (normalized for the hard wrap at :98-99);
  byte-identical to the tail of
  `APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION` (`apply_patch_spec.rs:37`, 126 chars,
  ends with this exact string); the spec §3.1 fenced block (spec :294-296,
  126 chars) is **byte-identical to the .rs constant** (programmatic `==`).
- Quote 2: "The ENTIRE patch as a single string" — appears exactly once
  (unwrapped, single line); byte-identical to the head of
  `APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION` (`apply_patch_spec.rs:42`,
  2,198 chars, starts with this exact string); the spec §3.1 fenced block
  (spec :300-332, 2,198 chars) is **byte-identical to the .rs constant**.
  Length claims 126 + 2,198 reproduced exactly.
- Anti-wrapping rule: unquoted paraphrase "no markdown fences, code-block
  markers, or shell heredoc wrapper" faithfully renders the P1 text's actual
  sentence, present byte-exact in both the .rs constant and spec §3.1: "Do not
  add markdown fences, code-block markers, or a shell heredoc wrapper around
  the patch." The old composite quote ("put the ENTIRE patch text in `patch`;
  do not wrap in JSON or markdown") is **gone** from the seam doc (grep: 0).
  The record's note that the "JSON" wording was dropped from the P1 text by
  design is consistent: the pinned P1 block contains no JSON-wrapping clause
  (the JSON anti-wrap lives only in the freeform description, `apply_patch_spec.rs`
  line 20 of the pre-P1 blob).

**A4. M-D1 (seat D, minor) — false quotient 70.27/1.21 = 58.04 — FIXED, VERIFIED.**
`grep -n "58.04"` over the evidence doc returns 7 hits; classified each:
- :191, :219, :512, :545, :602 — all are the **exact-ratio** statement
  "10738/185 = 58.0432" (correct: 10738/185 = 58.043243…).
- :546 (Round-1 record, n-F4 line) — `the "58.04" phrasing`: a *phrasing*
  citation tied to the exact ratio, not the raw quotient 70.27/1.21; it records
  r1 seat B's actual n-F4 finding ("58.04 is the exact fraction (26/37)/(5/413)
  = 10738/185 = 58.0432") — faithful, compliant.
- :600 (Round-2 record, M-D1 line) — the ONLY raw-quotient citation: the
  intentional quoted defect `"70.27 / 1.21 = 58.04"`, exactly as the task
  whitelists.
The false equation as an assertion appears **zero** times outside that quoted
defect. Arithmetic re-derived: 70.27/1.21 = 58.0744 (the record's rounded note
"70.27/1.21 = 58.07" is correct; stated once as a claim at :219-220 Factual
verification; the other two 58.07 mentions are review-log lines :547/:604).
The record's "all three occurrences (Factual verification; Worker attestation;
Changes #8 text — found during the fix pass)" reconciles with seat D's two
(pre-fix :219/:502): the third (Changes #8, now :191) was found during the fix
pass — honestly disclosed. Pinned rates re-derived: 26/37 = 70.27%, 5/413 =
1.21%, 13/21 = 61.9% ≈ 62% — all as stated in the seam table (:143-146) and
matching spec §1.2 byte-for-byte (see C3).

**A5. M-D2 (seat D, minor) — Final-state tree claims stale vs HEAD — FIXED, VERIFIED.**
Addendum at evidence :437-442: "the tree claims above are as of the
pre-baseline tree (HEAD `3db1b1381a`); ~14s after this doc's last write,
operator-directed baseline commit `799ec98053` … committed exactly that
12-file / 809+ / 20− delta. The item-5 docs themselves remain untracked and
uncommitted." Verified against git:
- `git log --oneline -3` = `799ec98053`, `3db1b1381a`, `a90b15d600` — the
  baseline commit sits directly on the attested pre-baseline HEAD.
- `git show --stat 799ec98053` = **12 files changed, 809 insertions(+),
  20 deletions(-)** — 1:1 with the attested delta; the committed file list is
  1:1 with the evidence's 12-file list (:420-430).
- Commit timestamp 2026-09-16 19:58:59 −0400; seat D's M-D2 timeline pins the
  pre-r2 doc's last write at 19:58:45 (doc mtime at r2) — the "≈14s" detail
  matches that verified timeline. (The file's current mtime, 21:41:21, is the
  r2 fix pass that added this addendum; the addendum is explicitly
  "post-worker" and describes the pre-baseline state — checked and cleared,
  no finding.)
- `git status --short | grep -v '^??'` = empty; item-5 docs remain untracked
  (see E).
- D2 note cross-checked: the baseline commit's `amazon_bedrock/mod.rs` diff is
  exactly the three `apply_patch_function_tool: false` lines the D2 note
  describes (one struct field + two test-expectation literals) — nothing near
  the auth/TLS path.

**A6. F-C2 (seat C, nit) — Final-state line count/anchors — FIXED, VERIFIED.**
- 297 lines claimed; `wc -l` = **297** ✓.
- Delta math re-derived: 226 (pre-edit) + 61 (worker edits) = 287; r1 +7 =
  M-A1 +4 + M-A2 +2 + N-1 +1 (matches seat C's independent r1 verification,
  287→294); r2 +3 = F-C1/n-D5 −1 (Decision sentence 6→5 lines) + n-D3 +4
  (description bullet 2→6 lines) + n-D4 line-neutral (in-line rename) = +3 →
  **297** ✓. Components sum exactly as stated.
- Post-r2 anchors re-measured at this seat (all six, all exact):
  | Anchor (evidence :449-453) | Re-measured |
  |---|---|
  | §2 gate semantics :53-61 | :53-61 ✓ (exact start "Gate semantics:" / end "is made by OpenAI".) |
  | §3 "Format-remediation layer (P1/P2/P3)" :133-164 | :133 heading / :164 "is recorded under §6 of this doc." ✓ |
  | §4 row 7 :196 | :196 = the row-7 table line ✓ |
  | §6 conflict-site bullets :243-246 | :243-244 `streaming_parser.rs` bullet, :245-246 `parser.rs` bullet ✓ |
  | §6 gate bullets :253-259 | :253-254 `just test -p codex-apply-patch`, :255-259 raw-format probe ✓ |
  | §6 invariant bullet 2 :281-292 | :281-292 exact ✓ |

**A7. Round-2 record accuracy (evidence :563-621) — VERIFIED, no misstatements.**
- Seat verdicts/counts: seat C "REQUEST CHANGES, 0 Blocking / 1 Major / 0 minor
  / 1 nit" ✓ (matches `impl-item5-r2-seatC.md` verdict line); seat D "APPROVED,
  0 Blocking / 0 Major / 2 minor / 3 nit" ✓ (matches `impl-item5-r2-seatD.md`
  verdict line + counts).
- All seven findings are individually present in the seat reports and the
  record's dispositions match: F-C1 (Major, stale "OpenAI/Azure … untouched"
  subject surviving M-A1's appended qualifier) ✓; F-C2 (nit, Final-state
  anchors stale post-r1) ✓; M-D1 (false quotient, cited at pre-fix :219/:502)
  ✓; M-D2 (stale tree-state block, baseline commit unmentioned) ✓; n-D3
  (composite description quote at pre-fix :98-99) ✓; n-D4 (`run_patch_text` at
  pre-fix :112-116) ✓; n-D5 (same residual as F-C1, independent phrasing) ✓.
- The record's per-finding line cites (e.g. n-D4's `:400/:381/:555`) match the
  actual source (A2). The cross-check note (:618-621) matches seat D's report
  one-for-one (40-lib recount in §D1; 8 classified hunks in §C3; 2,198+126
  byte-identity in §A3; zero tracked modifications in §F).
- Round-1 record spot-check (audit trail): m-F1 (46→40) and n-F4 ("58.04"
  phrasing) match r1 seat B's actual F1/F4 findings verbatim in substance.

## B. Fresh-eyes pass (new defects introduced by r2 fixes)

None found. Every r2 edit was re-derived from source (A1–A6); no r2 edit
introduced a stale anchor, an arithmetic error, or an over-claim:
- F-C1/n-D5: the rewrite only changed the subject and the dash→paren; the
  parenthetical's claims predate r2 and are all source-verified (A1).
- n-D3: the new quotes are byte-exact (A3); the only new *assertion* is that
  the anti-wrapping rule is the P1 text's actual rule — verified true (A3), and
  the seam bullet presents it unquoted (paraphrase), not as a quotation.
- M-D1: replaced with the exact ratio; every "10738/185 = 58.0432" statement
  re-derived correct (A4).
- M-D2/F-C2: all addendum/anchor claims re-verified against git and the file
  (A5, A6).

Checked-and-cleared observations (not counted as findings):
- **O1.** Evidence :546 n-F4 line's `the "58.04" phrasing` is a phrasing
  citation of the exact-ratio value (58.0432 → "58.04"), not the raw quotient
  70.27/1.21; it faithfully records r1 seat B's F4. A future mechanical
  "58.04" sweep will land here — expected, and compliant with the whitelist
  (only the quoted defect at :600 asserts the raw quotient).
- **O2.** Seam :51 status cell "IMPLEMENTED (items 1-3, 2026-09-16)" is a
  status shorthand blessed by r1 (N-5) and r2; the date is verified true (all
  four item commits + the baseline commit are dated 2026-09-16); the base
  capability code's provenance (working-tree seam carried by item 2/3 commits
  + baseline `799ec98053`, per `a4d5af1f62`'s message "Carries the branch's
  uncommitted function-tool seam") is documented in the evidence doc and
  breakdown §0 — no over-claim in the cell.
- **O3.** Seam §4 row 1's Files column lists `endpoint/{mod,responses}.rs,
  client.rs, provider.rs` — the module's own file (`content_type_compat.rs`)
  is named in the Change column and declared at `endpoint/mod.rs:1`
  (`pub(crate) mod content_type_compat;`); the §6 conflict-site list
  (`endpoint/{mod,responses}.rs`) is correct for conflict prediction (our
  fork-specific module file only conflicts if upstream creates the same path).
  Pre-existing row, outside item 5's change list; r1/r2 seats tracked the
  effect to the module and verified it. Optional completeness cleanup only.
- **O4.** Evidence "Changes" section anchors (e.g. :126-156, :188, :235-238)
  are the worker's post-edit (287-line file) positions — a before/after log of
  the worker's own edits, not a present-state claim; the present-state anchors
  live in Final state (A6, all exact). Consistent with the doc's point-in-time
  convention (header pins its HEAD).

## C. Source verification (seam doc vs tree; spec vs tree)

- **C1. Capability table SHAs.** `b4d4b12` = `b4d4b125cc` "feat: normalize
  content types for non-OpenAI providers (vLLM/SGLang)"; `197ea16` =
  `197ea1642c` "feat: flatten namespace tools for non-OpenAI providers
  (vLLM/SGLang)" — both exist in `git log --all` with subjects matching the
  table's capability names. All four item SHAs exist with the item/P-level
  subjects the spec status line maps (item 1/P2 `e21f608ac4`, item 2/P1
  `a4d5af1f62`, item 3/P3 `939a6dc6f4`, item 4/T4 `a90b15d600`), all dated
  2026-09-16 ("landed 2026-09-16" holds).
- **C2. `is_openai()` claims.** `model-provider-info/src/lib.rs:40` =
  `const OPENAI_PROVIDER_NAME: &str = "OpenAI";`; `:546-547` = `pub fn
  is_openai(&self) -> bool { self.name == OPENAI_PROVIDER_NAME }` — plain
  `&str ==`, exact, case-sensitive; `grep -in azure` over the file: **0
  matches** — the seam §2 "no Azure branch" claim is accurate. Gates at
  `provider.rs:369/:371/:372` all `!self.info.is_openai()`; fields :61-63,
  defaults :74-76 — matches the §2 table. "Router splits dotted names back":
  `ToolName::from_response_fields` def at `protocol/src/tool_name.rs:54`
  (doc: "splits it back out"), called at `core/src/tools/router.rs:254,289` ✓.
  `flatten_namespace_specs` at `tools/src/tool_spec.rs:158` ✓.
- **C3. P1/P2/P3 layer vs source.** P1: byte-exact per A3; spec §3.1 blocks
  (126 + 2,198 chars) byte-identical to the .rs constants. P2:
  `streaming_parser.rs:206-222` AddFile arm — structural lines handled first
  (:207), `+`-prefixed → prefix-stripped (:210-215), every other line appended
  verbatim (:217-219, empty/whitespace included), arm ends `Ok(())` (:221) —
  "raw/empty/whitespace content lines accepted verbatim" is exact. P3: P3.1
  extended hunk-header sentence at :199-204 (keeps the original sentence as an
  exact prefix); P3.2 DeleteFile message at :227-231; P3.3 absent/non-string
  `patch` split at `apply_patch.rs:534-545` (the two `RespondToModel`
  teachable strings); P3.4 `check_start_and_end_lines_strict` at
  `parser.rs:257-274` with the Begin/End boundary messages — all four sites
  present as attributed in the seam (layer bullets :153-161, row 7 :196,
  conflict sites :243-246).
- **C4. Seam §6 conflict sites + gates vs breakdown Item 5 (:327-364).**
  Breakdown requires exactly: §2 is_openai correction ✓, §3 P1/P2/P3 layer with
  the three pinned numbers ✓ (seam :133-164), divergence row (provider-agnostic;
  surface; pre-pass surface; boundary messages unchanged) ✓ (row 7 :196), §6
  conflict sites `codex-rs/apply-patch/src/{streaming_parser,parser}.rs` with
  the `codex-rs/` prefix ✓ (:243-246), §6 gates `just test -p codex-apply-patch`
  + raw-format probe per spec §5.4 ✓ (:253-259), invariants = spec §3.4
  exact-invariant text ✓ (:281-292), spec final pass (status + ≈58×, §7
  untouched) ✓ (D below). Every breakdown-mandated addition is present.
- **C5. Invariants vs spec §3.4.** Bullet 1 (:277-280) matches the spec §3.4
  gate sentence (`is_openai()` name-anchored, no Azure branch, Azure-named in
  seam scope) + the byte-identity invariant. Bullet 2 (:281-292) is a faithful
  condensation of the spec §3.4 invariant paragraph: verbatim on the load-bearing
  clauses (outbound request bytes for `OpenAI`-named unchanged; provider-agnostic
  parser changes; byte-identical for accepted inputs; rejection→acceptance /
  rejection→teachable-error for rejected inputs; P3.4 pre-pass-only, diff
  consumer's parallel streaming boundary messages untouched); the trims are the
  four small justified trims + the two deliberately-omitted meta-sentences, all
  accounted for in the evidence's Changes #6 (:163-168).
- **C6. Spec frozen-state.** 1029 lines ✓; status :7-14 = `Status: IMPLEMENTED`
  with the four real SHAs + `<<ITEM5-SHA>>` placeholder at :10 ✓; §1.2
  Conclusion at :213 "≈58× qwen's ~1% (26/37 = 70.27% vs 5/413 = 1.21%)" ✓;
  `grep "≈60×"` = exactly one hit, :1022 (§7 R5-map I-N1 historical row,
  untouched) ✓; `grep "≈58×"` = exactly :213 ✓; §7 review log intact: five
  round resolution maps (:823/:879/:923/:966/:1015) + "Spec review stage:
  COMPLETE" (:1025) ✓.

## D. Evidence doc — gate attestation internal consistency

- Gate 1: 115/115, exit 0 — self-consistent (115 run = 115 passed).
- Gate 2: "118 tests run: 66 passed (2 flaky), 1 failed, 51 timed out, 4251
  skipped" → 66+1+51 = **118** ✓. "52 unique = 51 TMT + 1 FAIL" ✓; the listed
  non-pass identities (:294-345) number exactly **52**, and all 52 contain
  `apply_patch` (consistent with the `just test -p codex-core apply_patch`
  filter). "All 40 `codex-core` lib unit tests … passed on first try"
  (:356-357) — seat D independently re-counted 40 lib-target tests in the raw
  log (118 = 40 lib + 78 suite); the corrected count (m-F1) stands.
- Gate 3: "84 tests run: 81 passed (11 slow, 12 flaky), 2 failed, 1 timed out,
  0 skipped" → 81+2+1 = **84** ✓. The 2 FAILED = the two named amazon_bedrock
  TLS tests (D2: third-party `aws-smithy-http-client` rustls trust-store panic;
  the baseline commit's amazon_bedrock diff contains nothing near the auth/TLS
  path — A5); capability-gate regression target named and green.
- Pinned numbers vs spec §1.2: glm-5.2 | 37 | **26 (~70%)** (row pin
  2026-09-15T23:34:04.678Z) ✓; qwen3.8-27b | 413 | **5 (~1%)** (row pin
  2026-09-15T23:30:11.662Z) ✓; "13 of 21 glm calls rejected (~62%)" (spec
  :171-173) ✓; Conclusion "26/37 = 70.27% vs 5/413 = 1.21%" (spec :213-214) ✓.
  Seam table (:143-144) mirrors the pinned rows exactly, "as of the pinned
  snapshot" qualifier included.
- Worker attestation (:504-517) matches the five gate sections one-for-one;
  "no commits made" is the worker's own-action attestation — accurate as of the
  worker run, and the operator-directed baseline commit is explicitly covered
  by the M-D2 addendum (A5).

## E. Docs-only verification

- `git status --short | grep -v '^??'` → **empty** (zero modified/deleted/staged
  tracked files). `git diff`/`git diff --cached` empty.
- Untracked: 1,366 entries = 1,335 `codex-rs/vendor/**` (cargo-vendor build
  artifacts, ignored per task) + **31 non-vendor, all under `docs/`** (the two
  review targets, the spec, the full campaign `docs/reviews/*` tree,
  `docs/superpowers/`, `docs/vllm-glm-toolcall-research.md`,
  `engine-redesign-charter.md`-class pre-existing untracked docs).
- No `Cargo.toml`/`Cargo.lock`/`MODULE.bazel.lock` in the item-5 change set
  (untracked set contains none; tracked set clean). Docs-only: **HOLDS**.

## Findings

**None counted.** 0 Blocking, 0 Major, 0 Minor, 0 Nit.
Observations O1–O4 (§B) are checked-and-cleared records, not defects: none is
a misstatement, stale anchor, arithmetic error, over-claim, or audit-trail
corruption, and each was either pre-existing/out-of-scope (O3, O4) or
explicitly accounted for by the artifacts themselves (O1, O2).

## Verdict

The three r2 fixes that carried weight (F-C1/n-D5 subject reword, M-D1
quotient correction, M-D2/F-C2 tree-state + anchor repair) are all verified
true at the byte level against source and git; the two nits (n-D3 byte-exact
quotes, n-D4 helper name) are fixed exactly as the seat reports demanded, with
the record's own line cites matching the source 1:1; the Round-2 record
misstates nothing in seat C's or seat D's findings, counts, or dispositions;
no r2 fix introduced a new defect; and the secondary pass (capability table,
is_openai, P1/P2/P3 layer, breakdown Item 5 coverage, spec §3.4 invariants,
gate arithmetic, pinned numbers, docs-only tree, frozen spec) all verify clean.

**Counts: 0 Blocking, 0 Major, 0 Minor, 0 Nit.**

SEAT F round 3: **APPROVE** — 0 Blocking, 0 Major, 0 Minor, 0 Nit. Clean bar
(0 Blocking AND 0 Major) met; the round-3 review loop for item 5 (docs +
gates) can be declared clean, subject to the coordinator's `<<ITEM5-SHA>>`
fill after commit.
