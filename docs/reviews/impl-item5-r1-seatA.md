# Item 5 — Docs, gates, spec final pass: review round 1, SEAT A (factual accuracy + spec conformance)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` · Date: 2026-09-16
Reviewer: `/root/item5_review_seat_a` (independent; parallel seat B on process/gates/minimality)

## Scope

Item 5 deliverable (docs only, per breakdown `docs/responses-compat-apply-patch-task-breakdown.md`
"### Item 5" + spec v5 `docs/responses-compat-apply-patch-format.md` §6):

- `docs/responses-compat-seam.md` (untracked, 287 lines post-edit): §2 `is_openai()`
  correction; new §3 "Format-remediation layer (P1/P2/P3)" subsection; §4 divergence
  table row 7; §6 conflict sites; §6 verification gate; §6 invariants bullet.
- `docs/responses-compat-apply-patch-format.md` (untracked, 1029 lines post-edit):
  status line → IMPLEMENTED with the four item SHAs + `<<ITEM5-SHA>>` placeholder;
  §1.2 Conclusion `≈60×` → `≈58×`; §7 review log and everything else untouched.
- Full gate re-run (mandate 7), independently of the worker's recorded results.

Seat A lens: factual accuracy of every doc claim (numbers, code refs, dates, file
names, verbatim-invariant text) and conformance of the docs to the spec (LOCKED v5,
SoT). Process/minimality is seat B's lens; I note observations where the two lenses
touch but judge only my lens.

## Method

- Read breakdown Item 5 + §0 + §5, spec §1.2/§3.1-§3.4/§5.4/§6/§7, seam doc (whole,
  post-edit state), `docs/reviews/impl-item5-tdd-evidence.md` (worker D1-D5), and the
  four item evidence docs (headers/pre-checks).
- Grep/line-verified every code citation in the seam doc against the current tree
  (`model-provider-info/src/lib.rs:40, :546-547`).
- Re-derived all arithmetic myself: 26/37, 5/413, 13/21, the ~70%/~1%/≈62%/≈58×
  roundings.
- Cross-checked the named files of P1/P2/P3 against the committed diffs
  (`git show --stat e21f608ac4 a4d5af1f62 939a6dc6f4 a90b15d600`), commit subjects,
  and commit dates (`git log`).
- Verified the "diff consumer's parallel streaming boundary messages intentionally
  unchanged" claim by diffing `streaming_parser.rs` at pre-campaign HEAD
  `197ea1642c` (parent of `e21f608ac4`) against the current tree.
- **Spec byte-integrity:** the spec is untracked, so a full-file `diff` of the
  current spec against the v4 archive
  (`docs/reviews/apply-patch-format-spec-v4-draft.md`) enumerates *every* byte
  difference across the v5 + item-5 history. Exactly 10 hunks, all classified
  below — nothing else in the file moved.
- Re-verified the pre-edit v5 state independently via seat K's spec line cites
  (`docs/reviews/apply-patch-task-breakdown-r1-seatK.md`: Conclusion at pre-edit
  :208-210 with `≈60×`; R5 map I-N1 row at pre-edit :1018; round-5 summary at
  :989) against the post-edit file (all shifted by exactly +4 lines = the status
  block growth 4→8 lines; 7/7 spot-checked cites match the +4 offset).
- Gates re-run by this seat from `codex-rs/` (patient; no kills): full outputs in
  "Gate outputs" below.

---

## Mandate 1 — seam doc §2 `is_openai()` correction (:53-61)

(a) **Description now exact.** :53-61 reads: "Gate semantics: `is_openai()` is an
exact, case-sensitive match of the provider `name` field in `config.toml` against
`OPENAI_PROVIDER_NAME` — `model-provider-info/src/lib.rs:546-547` (constant
`OPENAI_PROVIDER_NAME: &str = \"OpenAI\"` at `:40`)". Verified at current tree:
`codex-rs/model-provider-info/src/lib.rs:40` is exactly
`const OPENAI_PROVIDER_NAME: &str = "OpenAI";` and `:546-547` is exactly
`pub fn is_openai(&self) -> bool { self.name == OPENAI_PROVIDER_NAME }` — a plain
`&str ==`, case-sensitive, single hits (same greps the worker ran). **Verified.**

(b) **Stale "plus an Azure branch" wording is GONE.** `grep -in "azure"` over the
seam doc returns 6 hits, none the stale gate claim:
- `:14` — "A. raw OpenAI / Azure" (§1 deployment diagram; pre-existing; describes
  the wire deployment class, not the gate; no contradiction).
- `:56-58` — the corrected in-scope statement (this edit).
- `:81` — "OpenAI/Azure providers are untouched: they keep the native
  custom+grammar path." — **pre-existing §3 Decision sentence; CONTRADICTS spec
  §3.4 and the corrected §2 two lines above. See Finding A-1 (Major).**
- `:269` — "OpenAI/Azure provider behavior is byte-identical to upstream (all
  capability flags false on `is_openai()`)." — **pre-existing first invariants
  bullet; contradicted by spec §3.4's resolved position and by the new adjacent
  bullet item 5 added. See Finding A-2 (Major).**
- `:286` — "covered by a unit test in `model-provider/src/provider.rs` (cases:
  OpenAI, Azure, custom)" — a test-coverage statement, accurate (the uncommitted
  seam test `configured_provider_apply_patch_function_tool_matches_provider_support`
  has exactly those three cases, Azure → `true`).

(c) **Consistency with spec §3.4.** Spec §3.4 invariant: "`is_openai()` is
`name == \"OpenAI\"` with **no Azure branch** … so any Azure-*named* provider is
in seam scope (receives the function tool + P1 text)". Seam :56-58: "There is
**no Azure branch** … any Azure-*named* provider fails the gate and is in seam
scope (function tool + P1 format text)". Semantics match exactly ("fails the gate"
= `is_openai()` false = capability true). **Consistent.**

(d) **Trailing sentences preserved.** The pre-edit paragraph (quoted in
`docs/reviews/impl-item5-tdd-evidence.md` :79-82) ended with "The gate means
\"this provider's *deployment* implements the full OpenAI responses spec\", not
\"the model is made by OpenAI\"." — present verbatim at :59-61. Section flow
(capability table → gate semantics → the three sentences) is coherent. **Preserved.**

## Mandate 2 — seam doc §3 "Format-remediation layer (P1/P2/P3)" (:126-156)

(a) **Numbers vs spec §1.2 pinned snapshot — all exact, arithmetic re-derived:**
- :136 "glm-5.2 (vLLM-served) | 37 | 26 | **~70%** (70.27%), as of the pinned
  snapshot" — spec §1.2 pinned table row `glm-5.2 | 37 | 26 (~70%) | 11 | 0 | —`
  (row pin 2026-09-15T23:34:04.678Z). 26/37 = 0.70270… → 70.27% ✓; 70.27% rounds
  to ~70% ✓.
- :137 "qwen3.8-27b | 413 | 5 | **~1%** (1.21%)" — spec §1.2 pinned row
  `qwen3.8-27b | 413 | 5 (~1%) | 374 | 30 | …` (row pin 2026-09-15T23:30:11.662Z).
  5/413 = 0.012106… → 1.21% ✓; rounds to ~1% ✓.
- :139 "glm-5.2 on 09-15 alone: 13 of 21 calls rejected ≈ 62%" — spec §1.2
  "Most recent day (09-15) alone: 13 of 21 glm calls rejected (~62%)". 13/21 =
  0.61904… → ≈62% ✓.
- Ratio context (not in the seam table, checked anyway): 26/37 ÷ 5/413 =
  10738/185 = 58.04× — the spec's corrected `≈58×` (mandate 5).
- :140-142 "two format classes — raw unprefixed content lines (F1/F2) and missing
  `*** Begin Patch`/`*** End Patch` boundary markers (F4)" — spec TL;DR: "raw
  unprefixed content lines (F1/F2, 19 of 26) and missing boundary markers (F4,
  7 of 26)". Consistent (seam omits the 19/7 split — summary, not a
  contradiction).

(b) **P1/P2/P3 bullets vs the committed diffs:**
- P1 (:146-147) "the full patch format is taught in the function-tool `patch`
  argument description (non-OpenAI path only; `apply_patch_spec.rs`)" — item 2
  commit `a4d5af1f62` touches exactly `core/src/tools/handlers/apply_patch_spec.rs`
  (+78) + `apply_patch_spec_tests.rs` (+125); spec §3.1: text sent "for
  non-OpenAI providers only". **Matches.**
- P2 (:148-150) "Add-File leniency in the streaming parser: raw/empty/whitespace
  content lines accepted verbatim (defense in depth; provider-agnostic;
  `streaming_parser.rs`)" — item 1 commit `e21f608ac4` (numstat):
  `apply-patch/src/streaming_parser.rs` +21/−11 (the AddFile-arm leniency +
  module doc) + `parser.rs` +1/−0 (doc-comment line only — no behavior) + new
  `streaming_parser_p2_tests.rs` +334. Naming only `streaming_parser.rs` is
  accurate (the `parser.rs` hunk is a doc note). **Matches.**
- P3 (:151-153) "teachable parse errors (bad hunk headers, Delete-File content,
  absent/non-string `patch`), including the `parse_patch` pre-pass boundary
  strings (P3.4; `streaming_parser.rs`, `parser.rs`)" — item 3 commit
  `939a6dc6f4` contains all four sites: P3.1 StartedPatch sentence extension and
  P3.2 DeleteFile replacement in `streaming_parser.rs` (diff-verified; texts
  byte-match spec §3.3.1/§3.3.2), P3.4 pre-pass strings in `parser.rs`
  (diff-verified; byte-match spec §3.3.4a/b), P3.3 absent/non-string `patch` in
  `core/src/tools/handlers/apply_patch.rs` (diff-verified; byte-match
  spec §3.3.3). The bullet's parenthetical names 2 of the 3 files carrying P3
  sites; the third (`apply_patch.rs`) is named in divergence row 7's Files column
  ("`apply_patch.rs` (P3.3 handler errors)"). See nit N-1. **Matches.**

(c) **"landed 2026-09-16"** (:143): `git log --date=format` on the four commits —
`e21f608ac4` Wed Sep 16 00:23:08 2026, `a4d5af1f62` Wed Sep 16 02:44:16,
`939a6dc6f4` Wed Sep 16 09:51:52, `a90b15d600` Wed Sep 16 14:44:12. All four
2026-09-16. **Verified.** "(commit SHAs recorded in the spec's status line)" —
true (mandate 5).

(d) **Placement.** Heading `### Format-remediation layer (P1/P2/P3)` at :126,
section body :128-156, immediately after `### Components` (:84-124) and before
`### Testing` (:158). Coherent: the remediation layer extends the function-tool
design it sits in. **Verified.**

## Mandate 3 — seam doc §4 divergence row 7 (:188)

- **5-column format** (# | Change | Files | Purpose | Rebase risk): row 7 at :188
  follows rows 1-6 exactly. **Verified.**
- **Surface description vs the breakdown** (breakdown Item 5 bullet 1, divergence
  row): "provider-agnostic; surface = function-tool path + freeform diff consumer
  + `apply_patch` CLI (P2) plus the `parse_patch` pre-pass surface for P3/P3.4
  (function path + CLI + shell-intercept); the diff consumer's parallel streaming
  boundary messages intentionally unchanged; re-apply on upstream restructure of
  `streaming_parser.rs` or `parser.rs` during rebase" — row 7's Change cell:
  "apply-patch parser change: P2 Add-File leniency + P3/P3.4 teachable errors —
  **provider-agnostic**; surface = function-tool path + freeform diff consumer +
  `apply_patch` CLI (P2), plus the `parse_patch` pre-pass surface for P3/P3.4
  (function path + CLI + shell-intercept)"; Rebase-risk cell: "low-medium —
  re-apply on upstream restructure of `streaming_parser.rs` or `parser.rs`; the
  diff consumer's parallel streaming boundary messages intentionally unchanged
  (spec §3.4 divergence note)". All required elements present, wording tracks the
  breakdown. **Verified.**
- **"boundary messages intentionally unchanged" is TRUE.** Diffed
  `streaming_parser.rs` at `197ea1642c` (pre-campaign parent of `e21f608ac4`)
  against the current tree: the three parallel streaming boundary messages —
  `finish()` :168, `NotStarted` :184, `EndedPatch`-content :374 pre-shift — read
  byte-identically at current :176/:192/:380 ("The last line of the patch must be
  '*** End Patch'" / "The first line of the patch must be '*** Begin Patch'").
  The campaign diffs (items 1+3) touch only the AddFile arm, the StartedPatch and
  DeleteFile error texts, the module doc, and tests. Matches the item-3 evidence's
  scope-boundary record (that doc's post-shift cite of :381 is a cosmetic
  off-by-one vs the actual :380 — observation only, not an item-5 artifact).
  **Verified true.**
- **Named files** (`apply-patch/src/streaming_parser.rs`,
  `apply-patch/src/parser.rs`, `apply_patch.rs` (P3.3 handler errors)) — exactly
  the behavior-bearing files of items 1+3 per `git show --stat`. **Verified.**
- Purpose cell "~70% … (spec §1.2 pinned snapshot); P2/P3 affect only inputs
  upstream rejects (spec §3.4)" — consistent with spec §1.2 and §3.4.

## Mandate 4 — seam doc §6

(a) **Conflict sites** (:235-238): both required entries present —
`codex-rs/apply-patch/src/streaming_parser.rs` ("P2 Add-File arm + P3.1/P3.2
error-message sites (the format-remediation layer, §3)") and
`codex-rs/apply-patch/src/parser.rs` ("P3.4 `parse_patch` pre-pass boundary
strings (the Begin/End patch error messages)"). **D1 adjudication (mixed path
spelling):** the two new entries carry the `codex-rs/` prefix while siblings
(`codex-api/src/…`, `core/src/client.rs`, `models-manager/models.json`,
`model-provider/src/provider.rs`) are codex-rs-relative. The spec §6 requirement
and the breakdown spell the new entries WITH the prefix, so the worker followed
the locked SoT verbatim; the divergence-table row 7 (same doc) uses the
codex-rs-relative `apply-patch/src/…` spelling to match its own table convention
(row 1 style). Each site matches its own source-of-truth spelling; both spellings
resolve to the same files; zero functional impact. **Adjudged: acceptable —
recorded as nit N-2, not a defect.**

(b) **Verification gate** (:245-251): adds `just test -p codex-apply-patch`
("P2/P3 parser behavior — a rebase that silently reverts P2 fails here") and the
raw-format probe bullet referencing `docs/responses-compat-apply-patch-format.md`
§5.4 ("standalone `apply_patch` binary from the same release build, scratch dir,
constructed raw Add-File patch in the F1 shape (no `+` prefixes, `# …` first
content line) — pass = exit 0, contents byte-identical to the raw content").
Matches spec §5.4 step-4 probe text element-for-element; matches spec §6's
requirement (`just test -p codex-apply-patch` is the spec-mandated command
string). The section's siblings use `cargo test -p …` while the new bullet uses
`just test` — the repo's own rule ("Do not run `cargo test` directly. Use
`just test`") makes `just test` the correct invocation, and the spec dictated the
exact string. **Verified** (see nit N-3).

(c) **Invariants bullet** (:271-282) vs spec §3.4 — clause-by-clause:
- Clause 1 (OpenAI-named request bytes unchanged): "outbound **request bytes for
  providers named `OpenAI` are unchanged** (P1 touches only the function-tool
  spec, which never ships to them)" — spec: same text plus "by this work".
  Verbatim except the dropped "by this work" (contextually implied by the
  enclosing "Invariants that must survive every rebase" header).
- Clause 2 (P2/P3 provider-agnostic, rejected-inputs-only, full surface): "P2 and
  P3 are **provider-agnostic parser changes**: for every input upstream accepts,
  behavior is byte-identical; for inputs upstream *rejects*, P2 turns some
  Add-File rejections into accepted content and P3 turns some rejections into
  teachable errors — full surface: the function-tool execution path, the
  freeform diff consumer, and the `apply_patch` CLI" — spec: same, with
  "Full surface of that divergence:" and a diff-consumer parenthetical
  ("(`ApplyPatchArgumentDiffConsumer` streams the same parser — raw Add-File
  content now streams as diffs instead of erroring)"). Verbatim except the
  dropped parenthetical elaboration (the surface trio is intact).
- Clause 3 (P3.4 pre-pass-only): "P3.4 applies to the `parse_patch` pre-pass
  surface only (function path + CLI + shell-intercept); it does **not** touch
  the diff consumer's parallel streaming boundary messages" — spec: same, with
  the "v2 refinement:" tag and "via invocation.rs:116/123/170/175". Verbatim
  except those two spec-versioning/meta trims (the tag would be nonsense in the
  seam doc; the line cites live in spec §3.3/§3.4).
- **Verdict on "VERBATIM":** the three clauses are present with the spec's own
  wording; four small trims total (two clearly justified, two condensation).
  The bullet's framing "holds the spec's exact invariant (…§3.4)" is a pointer,
  not a quotation. Recorded as nit N-4 (a strict byte-identical copy would have
  been marginally preferable, but the trims drop nothing load-bearing).
- **Deliberate omissions — both justified:**
  1. Spec §3.4 meta-sentence "The capability gate itself is unchanged:
     `is_openai()` is `name == \"OpenAI\"` with **no Azure branch** … the seam
     doc §2 \"plus an Azure branch\" description is stale and is corrected in
     the same PR set (v1, seat B M5 / seat A M2)" — the factual half (gate
     unchanged, name match, no Azure branch) is now carried by the corrected
     seam §2 paragraph itself, which is the natural home; the second half is a
     review-meta note that is FALSE post-correction (the §2 description is no
     longer stale). Neither half is a rebase invariant. **Omission justified.**
  2. Spec §3.4 meta-sentence "This change is **not verified against any Azure
     deployment** (none in use); Principle-2 verification covers the
     vLLM/LiteLLM deployments only" — a one-time verification-scope caveat about
     this change's testing, not an invariant that "must survive every rebase".
     **Omission justified.** (One could argue the caveat belongs in the doc
     somewhere; the corrected §2 + spec §3.4 remain the source of record. Not a
     defect.)

## Mandate 5 — spec changes

(a) **Status line** (:7-14): "Status: IMPLEMENTED — all five items landed on
branch `feat/normalize-content-types-vllm` (item 1 / P2 `e21f608ac4`, item 2 /
P1 `a4d5af1f62`, item 3 / P3 `939a6dc6f4`, item 4 / T4 `a90b15d600`, item 5 /
docs + gates `<<ITEM5-SHA>>` — coordinator fills after commit). Spec review loop
terminated at round 5 … Task-breakdown review loop terminated at round 2 …".
- `<<ITEM5-SHA>>`: exact token, exactly one occurrence, at **line 10**.
- SHA↔item mapping vs `git log --oneline -1` + `git show --stat`:
  - `e21f608ac4` = "apply-patch compat: P2 Add-File leniency (item 1)" —
    streaming_parser.rs + parser.rs(+1 doc) + p2 tests. → item 1 / P2 ✓
  - `a4d5af1f62` = "apply-patch compat: P1 function-tool spec text (item 2)" —
    apply_patch_spec.rs + tests. → item 2 / P1 ✓
  - `939a6dc6f4` = "apply-patch compat: P3 teachable errors (item 3)" —
    parser.rs, streaming_parser.rs, apply_patch.rs + tests. → item 3 / P3 ✓
  - `a90b15d600` = "apply-patch compat: T4 integration tests on renamed
    non-OpenAI provider (item 4)" — responses.rs helper + apply_patch_cli.rs.
    → item 4 / T4 ✓
- "all five items landed" while item 5's own SHA is a placeholder: the wording is
  the coordinator-contracted form (placeholder fill happens at commit); not a
  defect.

(b) **§1.2 Conclusion** (:212-214): "parse-rejection rate is ~70% (as of the
pinned snapshot), ≈58× qwen's ~1% (26/37 = 70.27% vs 5/413 = 1.21%)".
- `≈58× qwen's` present ✓; arithmetic 10738/185 = 58.04× ✓.
- `grep -n "≈60×"` over the current spec returns **exactly one line: :1022**,
  the §7 Round-5 resolution map I-N1 row: `| I-N1 (Nit) | Conclusion "an order
  of magnitude above" → "≈60× qwen's ~1%" (70.27% / 1.21% ≈ 58×) | §1.2
  Conclusion |`. That row is the R5 round's own record: it states the v5 fix as
  applied, in the original wording ("→ \"≈60× qwen's ~1%\""), and is unchanged
  by item 5 — in the v4-archive diff it appears as a pure v5 addition
  (:1008-1029 block), and the K-N1 deferral (breakdown §9 v1→v2 map row 13:
  "Deferred to Item 5 step 2 … the reviewed spec is deliberately untouched by
  this breakdown revision") is exactly what produced the two-stage
  ≈60×(v5) → ≈58×(item 5) history. The related §7 line :993 (round-5 seat-I
  summary, "I-N1 (\"an order of magnitude above\" ≈ 58×)") contains no ≈60×.
  **§7 byte-identical to what the R5 round recorded — verified.**

(c) **Everything else untouched.** Byte-level proof: full-file
`diff v4-archive current-spec` = **exactly 10 hunks**, each classified:
(1) :5 Evidence line gains "· v4 archive: …" (v5 bookkeeping); (2) status block
4→8 lines (item 5); (3) §1.2 VERIFY exemplar gains `Failed to find context`
(R5 I-N3/J-M1); (4) §1.2 glm breakdown "both patches end with … v5: corrected
from \"one\"" (R5 I-M1); (5) §1.2 Conclusion (R5 I-N1 + item 5's single-token
≈58×); (6) T4.2 ":933-942"→":933-943" (R5 J-N1); (7) R2-map D-M2 row same cite
(R5 J-N1); (8) R3-map E-N1 row opening restated (R5 I-N2); (9) §7 round-5 entry
replaces "pending" with the completed-round record (v5 bookkeeping); (10) §7
Round-5 resolution map appended (v5 bookkeeping). No hunk touches the TL;DR,
§2, §3.1-§3.4, §4, §5, §6, or any round-1..4 map — i.e. the TL;DR and all
non-§7/§1.2/spec-header content is byte-identical to v4, and within v4→v5 only
the five recorded R5 fixes + the §7 round-5 record changed. `##`-heading list
matches the v4 archive's structure with only "### Round-5 resolution map (v4 →
v5)" added under §7. **Verified — no incidental damage.**

## Mandate 6 — cross-doc consistency

- Seam §3 remediation section vs spec §3.1-§3.3: P1/P2/P3 characterizations
  match (see mandate 2(b)); "first-shipped … taught **no** patch format
  (\"the complete patch goes in the `patch` argument as plain text\")" matches
  spec TL;DR; "exactly what P1 teaches; P2/P3 are the defense-in-depth layer
  below it" matches spec §3.2's "(defense in depth)" framing and §3.3.
  **No contradiction.**
- Seam §6 invariants (new bullet) vs spec §3.4: three clauses present, wording
  as in mandate 4(c). **No contradiction** (trim nits recorded).
- Seam §4 row 7 vs spec §3.4 divergence note: "the diff consumer's parallel
  streaming boundary messages intentionally unchanged" is the spec §3.4 v2
  refinement sentence verbatim in substance, and is TRUE at source (mandate 3).
  **No contradiction.**
- **However, two pre-existing seam-doc statements the spec contradicts survive
  in the delivered doc** — see Findings A-1 (seam :81) and A-2 (seam :269-270).
  Per the seat contract, a seam-doc statement the spec contradicts = Major.

## Discrepancy adjudications (worker D1-D5)

- **D1 (conflict-site path style):** adjudicated in mandate 4(a) — worker
  followed the SoT/task spelling verbatim; sibling style omits the prefix.
  Acceptable; **nit N-2** at most. The worker's read is correct.
- **D2 (Gate 3 amazon_bedrock TLS failures pre-existing/environmental):**
  corroborated by this seat's independent checks: (a) the working-tree
  `amazon_bedrock/mod.rs` delta is exactly one `apply_patch_function_tool: false`
  field (:219) + two matching test-expectation literals (:628, :650) — nowhere
  near auth/TLS; (b) the new capability test in `provider.rs` asserts the three
  cases (OpenAI false / Azure true / custom true) and is the gate's stated
  target. **This seat's independent Gate 3 run reproduced the exact same two
  test identities with the exact same third-party trust-store panic
  (`aws-smithy-http-client-1.1.12 … rustls_provider.rs:116`)** — D2's
  pre-existing/environmental classification is corroborated (see Gate 3
  section).
- **D3 (Gate 4 `just fix` re-applied the known `openai_file_mcp.rs:47`
  unused-import fix; restored via `git checkout --`):** this seat's Gate 4 run
  re-checked `git status` on that file afterward (below).
- **D4 (Gate 2 first attempt lost to session cleanup; successful run recorded):**
  process fact, seat B's lens; no bearing on Seat A's gate results (this seat
  re-ran everything itself).
- **D5 (status line keeps "maps in §7" phrasing):** verified — the R5 map does
  live in §7 (:1017+); no §7 bytes touched by item 5 (mandate 5(b)/(c)).
  **Corroborated.**

## Findings

### A-1 [Major] — seam doc :81 "OpenAI/Azure providers are untouched" contradicts spec §3.4 (and the corrected §2)

`docs/responses-compat-seam.md:81` (in "### Decision", pre-existing text not
in item 5's change list): "OpenAI/Azure providers are untouched: they keep the
native custom+grammar path." The sentence admits a charitable reading
("providers named `OpenAI`, even when served via an Azure endpoint"), but the
reading consistent with the rest of the doc and the code is the plain one —
and under it the sentence is false: spec §3.4 says "any Azure-*named* provider
is in seam scope (receives the function tool + P1 text)", and the corrected §2
paragraph (seam :56-58, item 5's own edit) states it too: "any Azure-*named*
provider fails the gate and is in seam scope (function tool + P1 format
text)". The committed/uncommitted code agrees with the spec: `provider.rs:372`
computes `apply_patch_function_tool: !self.info.is_openai()` and the capability
test asserts the Azure-named case → `true` (an Azure-named provider gets the
function tool + P1 text, and already gets normalize/flatten). So an
Azure-*named* provider does NOT "keep the native custom+grammar path".
Context: spec-review round 1 flagged exactly this claim class (seat B M5: the
seam doc's Azure statements are stale; resolution: "providers whose `name` is
not exactly `OpenAI` — including anything named Azure — are in scope"). Item 5
corrected §2 (the named target) but the same stale "OpenAI/Azure" grouping
survives one section down, where it now directly contradicts the doc's own
corrected §2 and the spec it is supposed to companion. Root cause: the spec §6
/ breakdown Item 5 change list named only the §2 paragraph, not this sibling
sentence. Fix is a one-line reword (e.g. "OpenAI-*named* providers are
untouched …"). Classification per seat contract: a seam-doc statement the
spec contradicts = Major. (Pre-existing wording, but the delivered SoT doc is
self-contradictory; item 5 is the final-pass item whose DoD is a correct
companion doc.)

### A-2 [Major] — seam doc :269-270 first invariants bullet ("OpenAI/Azure provider behavior is byte-identical to upstream") contradicts spec §3.4 and the adjacent new bullet

`docs/responses-compat-seam.md:269-270`: "- OpenAI/Azure provider behavior is
byte-identical to upstream (all capability flags false on `is_openai()`)."
Two independent contradictions:
1. **Azure half** — an Azure-*named* provider has `is_openai()` false, so the
   parenthetical's condition doesn't even hold for it, and its behavior is NOT
   byte-identical to upstream (it gets the three capabilities; the capability
   test pins Azure → `true`). Spec §3.4: Azure-named providers are in seam
   scope. Same staleness class as A-1.
2. **"behavior is byte-identical" half** — even for OpenAI-*named* providers,
   P2/P3 changed behavior for inputs upstream rejects, on the shared surface
   that INCLUDES the freeform diff consumer (spec §3.4 full-surface clause;
   spec §3.3 scope boundary). This is precisely the sentence spec-review
   round-1 seat A M2 flagged as "literally violated for malformed input" and
   resolved by re-scoping the claim to "outbound **request** bytes" (which is
   what the spec §3.4 invariant — and item 5's new bullet at :271-282, three
   lines below — says). As delivered, the invariants list contains both the
   pre-M2 overclaim and the corrected invariant side by side; a rebase
   auditor following "Invariants that must survive every rebase" gets a false
   invariant the same list refutes.
Root cause as A-1: the spec §6 change list scoped the invariants edit to
"add the §3.4 exact-invariant text" — an addition — and did not list the
reword of the pre-existing first bullet that M2's resolution implied. Fix is a
one-line reword (e.g. "OpenAI-*named* provider **request bytes** are
byte-identical to upstream (all capability flags false on `is_openai()`)"),
which also removes the duplicate of A-1's Azure error.

### N-1 [nit] — seam doc :151-153 P3 bullet names 2 of the 3 files carrying P3 sites

The P3 bullet's parenthetical "(P3.4; `streaming_parser.rs`, `parser.rs`)"
lists the files for the P3 layer but omits `core/src/tools/handlers/apply_patch.rs`
(the P3.3 absent/non-string `patch` site — in the item-3 commit, 325 lines
changed). The description text does cover the P3.3 behavior ("absent/non-string
`patch`"), and the file IS named in divergence row 7's Files column ("
`apply_patch.rs` (P3.3 handler errors)"), so the doc as a whole is complete;
the bullet alone is not. No spec contradiction.

### N-2 [nit] — seam doc §6 mixed path spelling (D1)

Conflict-site entries :235-238 use repo-root-relative
`codex-rs/apply-patch/src/…` while siblings are codex-rs-relative
(`codex-api/src/…`, `core/src/…`). Adjudicated acceptable: the locked spec §6
and the breakdown spell the two entries with the prefix, so the worker
followed the SoT verbatim; each site matches its own source-of-truth spelling;
row 7's Files column correctly uses the table-local convention. A future
consistency pass could normalize the siblings or the new entries; no action
required for item 5.

### N-3 [nit] — seam doc :245 gate bullet uses `just test` while siblings use `cargo test -p`

Cosmetic invocation-style mismatch inside the §6 gate list. The spec §6
mandates the exact string `just test -p codex-apply-patch`, and the repo rule
forbids running `cargo test` directly, so the new bullet is the more correct
one; no action required. (Noted for completeness; the other four bullets'
`cargo test` forms pre-date this item and were out of scope.)

### N-4 [nit] — seam doc :271-282 invariant bullet is a faithful condensation, not a byte-identical copy

The bullet introduces the spec's invariant as "the spec's exact invariant
(…§3.4)" and then carries the three clauses in the spec's wording with four
small trims: "by this work" (clause 1), the
`ApplyPatchArgumentDiffConsumer` explanatory parenthetical (clause 2), the
"v2 refinement:" tag and "via invocation.rs:116/123/170/175" cites (clause 3).
Nothing load-bearing is dropped (the three surfaces, the rejected-inputs
scope, and the pre-pass-only restriction are all intact); the spec remains the
byte-level source of record and is cited. If "exact" is read as byte-identical,
the parenthetical and the invocation.rs cites are the two trims worth
restoring; as delivered the bullet is a correct, clearly-attributed summary
of the exact invariant.

### N-5 [nit] — seam doc :51 capability-table status cell still says "SPECIFIED BELOW"

`apply_patch_function_tool` row status (pre-existing cell) predates
implementation; the §3 design section below now also documents the
remediation layer and the "landed 2026-09-16" commits, so the cell's pointer
is still literally true but stale in spirit (the capability is implemented on
this branch, though its registration seam remains uncommitted working-tree
state per breakdown §0 — a "shipped (commit …)" label would be premature
anyway). Not in item 5's change list; no spec contradiction. Cosmetic.

## Gate outputs (this seat's independent re-run, from `codex-rs/`)

Environment note: seat B was re-running the same gate suite concurrently on
this box (two `cargo-nextest -p codex-core apply_patch` processes observed
in parallel from ~17:34-17:39 onward). All runs completed; slow-test retries
and timeouts below are consistent with that contention + machine load, and
are classified per the task's load-noise definition (lib.rs:388 panics +
TMTs).

### Gate 1 — `just test -p codex-apply-patch`

- **GATE1_EXIT=0.**
- `Summary [   7.900s] 115 tests run: 115 passed, 0 skipped`
- **115/115, exit 0 — matches the required result and the worker's record.**

### Gate 2 — `just test -p codex-core apply_patch`

- **GATE2_EXIT=100** (non-zero only because of load-noise non-passes; see
  criterion below).
- `Summary [6566.756s] 118 tests run: 45 passed (10 flaky), 1 failed,
  72 timed out, 4251 skipped`
- **Binding check (verbatim, over the full Gate-2 log region):**
  `grep -E "thread .+ panicked" LOG | grep -v 'lib.rs:388' | sort -u` →
  **empty (0 lines)**. Panic-site census: 121 panics, ALL at
  `core/tests/common/lib.rs:388` (`timeout waiting for event:
  Elapsed(())`) — the named load-noise class. **0 DETERMINISTIC failures —
  the stated green criterion is MET.**
- Non-pass identities (recorded per task; all `codex-core::all` suite, all
  load-noise class):
  - **1 FAILED** (both attempts, same `lib.rs:388` timeout panic):
    `suite::apply_patch_cli::intercepted_apply_patch_verification_uses_local_sandbox`
  - **72 TMT** (60s harness timeouts, both attempts): the `suite::apply_patch_cli::*`
    and `suite::apply_patch_serialization::*` integration tests plus the
    approvals/hooks/prompt-caching/request-permissions/shell-snapshot/
    tool-harness/unified-exec/code-mode tests matching the `apply_patch`
    filter — full list in the raw log `/tmp/item5-gates-seatA.log`
    (GATE2 region, `TRY 2 TMT` lines).
  - **10 FLAKY** (TRY-1 fail → TRY-2 pass, 29s each under load):
    `tools::runtimes::apply_patch::tests::{wants_no_sandbox_approval_granular_respects_sandbox_flag,
    approval_keys_include_environment_id, approval_action_preserves_patch_path_uris,
    file_system_sandbox_context_preserves_executor_workspace_permissions,
    permission_request_payload_uses_apply_patch_hook_name_and_aliases,
    sandbox_cwd_uses_patch_action_cwd, file_system_sandbox_context_respects_sandbox_request}`,
    `tools::runtimes::tests::maybe_wrap_shell_lc_with_snapshot_restores_apply_patch_rollout_state`,
    and the two P1 drift-guard tests
    `tools::handlers::apply_patch_spec::tests::{create_apply_patch_function_tool_matches_expected_spec,
    create_apply_patch_function_tool_teaches_patch_format_in_argument_description}`.
- **Context:** the TMT rate (72/118) is higher than the worker's recorded run
  (51/118, 1 failed, 66 passed) because this seat's Gate 2 ran concurrently
  with seat B's identical Gate 2 on the same box (two
  `cargo-nextest -p codex-core apply_patch` processes, 17:34-~19:25) plus
  general machine load; the worker's run was also under "heavy machine load
  (loadavg observed 5-35)" per their evidence. Both runs meet the same stated
  criterion with the same noise class (lib.rs:388 + TMT); the specific
  non-pass victims differ between runs, which is exactly the signature of
  stochastic load noise rather than a deterministic regression.

### Gate 3 — `just test -p codex-model-provider`

- **GATE3_EXIT=100** (non-zero only because of the recorded pre-existing
  environmental bedrock failures + load TMTs).
- `Summary [ 345.112s] 84 tests run: 66 passed (22 slow, 5 flaky), 1 failed,
  17 timed out, 0 skipped` — the single summary "failed" is the first bedrock
  test below (FAIL on both attempts); the second bedrock test FAILed on
  TRY 1 (same panic) but hit the 60s harness TMT on TRY 2 under load, so it
  lands in the "timed out" count.
- **Capability-gate regression target PASSES:**
  `provider::tests::configured_provider_apply_patch_function_tool_matches_provider_support`
  → `TRY 2 PASS [ 30.880s] (69/84)` (TRY 1 was a 60s load TMT, not a test
  failure; flaky-by-retry classification 2/2). **Gate 3's stated purpose is
  MET** — including the Azure-named case → `true`.
- **The two D2 bedrock tests — same identities, same third-party panic as
  the worker's record (per-attempt detail in this run):**
  `amazon_bedrock::tests::command_auth_resolves_configured_and_regional_base_urls`
  — **TRY 1 FAIL [22.7s], TRY 2 FAIL [53.3s]**, both attempts panicking at
  third-party
  `aws-smithy-http-client-1.1.12/src/client/tls/rustls_provider.rs:116`:
  "TrustStore configured to enable native roots but no valid root
  certificates parsed!" (macOS native-roots store unreadable in this
  environment during AWS-SDK HTTP-client construction on the
  `provider.auth().await` path); and
  `amazon_bedrock::tests::configured_profile_takes_precedence_over_managed_auth`
  — **TRY 1 FAIL [23.9s]** (identical trust-store panic) **+ TRY 2 TMT
  [60.2s]**. **This is D2's exact classification, reproduced independently
  by this seat (the worker's run saw both FAIL on both attempts; this run's
  second test timed out on retry instead — same environmental cause).**
  Combined with this seat's source check that the working-tree
  `amazon_bedrock/mod.rs` delta is the single `apply_patch_function_tool: false`
  field (+2 test literals, nothing near auth/TLS), D2's pre-existing/
  environmental call is **corroborated**.
- Load noise: 17 TMT + 5 flaky (incl. the
  `models_endpoint::tests::command_auth_refresh_fetches_a_catalog_for_the_current_credentials`
  TRY-1 timing flake at `models_endpoint.rs:607` that the worker also
  recorded; it passed on TRY 2 here).

### Gate 4 — `just fix -p codex-apply-patch -p codex-core -p codex-model-provider`

- **GATE4_EXIT=0** (`cargo clippy --fix --tests --allow-dirty`, finished in
  1m21s warm; no clippy diagnostics required fixes in the three crates).
- **D3 hazard re-triggered, as pre-declared:** `just fix` re-applied the
  pre-existing unused-import removal at
  `codex-rs/core/tests/suite/openai_file_mcp.rs:47`
  (`- use wiremock::matchers::body_json;`) — an out-of-scope tracked file,
  present at HEAD, unrelated to the campaign. **Restored via
  `git checkout -- codex-rs/core/tests/suite/openai_file_mcp.rs`** (this
  seat's action, per the task contract); post-restore `git status` on the
  file is clean and the tracked diff is back to the exact 12-file /
  809-insertions / 20-deletions baseline — **D3 corroborated**.

### Gate 5 — `just fmt`

- **GATE5_EXIT=0.** No files reformatted (tree was already formatted; the
  tracked-diff stat is unchanged by the fmt run).

### Gate summary

| Gate | Command | Result | Verdict vs stated criterion |
|---|---|---|---|
| 1 | `just test -p codex-apply-patch` | exit 0; `115 tests run: 115 passed, 0 skipped` | **MET** (115/115) |
| 2 | `just test -p codex-core apply_patch` | exit 100; 45 passed (10 flaky), 1 failed, 72 TMT; binding grep **empty** (all 121 panics at lib.rs:388) | **MET** (0 deterministic failures; non-pass identities recorded) |
| 3 | `just test -p codex-model-provider` | exit 100; capability-gate target **PASS** (TRY 2); 2 bedrock TLS FAILs identical to D2; 17 TMT | **MET** (target green; D2 corroborated) |
| 4 | `just fix -p …×3` | exit 0; D3 hazard re-triggered → restored | **MET** (hazard as documented) |
| 5 | `just fmt` | exit 0 | **MET** |

## Observations (not findings)

- `docs/reviews/impl-item3-tdd-evidence.md` :23-26 records the
  EndedPatch-content boundary message at post-shift `:381`; the current file
  has it at `:380` (one-line cosmetic drift in the item-3 evidence doc, which
  is not an item-5 artifact; the item-5 seam doc cites no streaming_parser
  line numbers, so nothing in the delivered docs depends on it).
- Working-tree state matches the breakdown §0 execution baseline and the
  worker's record: `git diff HEAD --stat` over the tracked set = 12 files,
  809 insertions / 20 deletions (the uncommitted campaign seam + AGENTS.md);
  item 5 modified zero tracked files; the two campaign docs + this report are
  untracked.
- The status line's "all five items landed" coexists with the uncommitted
  item-5 placeholder — the coordinator-contracted form (the placeholder is
  filled at commit time); not flagged.

## Verdict

**What the item did right (for the record):** every in-scope edit is
factually accurate and spec-conformant. The §2 correction is exact and
code-verified; the §3 remediation section's numbers, files, classes, and
dates all reproduce from spec §1.2 and the four commits; the divergence row
matches the breakdown and its "boundary messages intentionally unchanged"
claim is TRUE at source (byte-identical vs `197ea1642c`); the §6 conflict
sites, gate additions, and invariant bullet are all present and correct
(verbatim up to four small, justified trims — N-4); the two meta-sentence
omissions from spec §3.4 are both justified; the spec diff is exactly the
status line + the single `≈58×` token (10-hunk byte-level proof; §7, TL;DR,
and everything else untouched); all five gates meet their stated criteria on
this seat's independent re-run (Gate 1 115/115; Gate 2 binding grep empty /
0 deterministic failures; Gate 3 capability-gate target PASS + D2
corroborated; Gate 4 exit 0 with the D3 hazard re-triggering exactly as
predicted and restored; Gate 5 exit 0).

**Why this round is not approvable:** the delivered companion SoT still
contains two pre-existing statements the spec contradicts (A-1 at
`docs/responses-compat-seam.md:81`, A-2 at `:269-270`). Per the seat
contract, a seam-doc statement the spec contradicts = Major. Item 5 is the
final-pass item whose DoD is a correct companion doc, and the spec's own
review history (round-1 seat A M2 / seat B M5, both resolved in v5)
identified exactly this claim class as needing correction — the v5 §6 change
list scoped the fix to the §2 paragraph and the invariants *addition*,
leaving these two sibling sentences stale. The fix is small: reword the two
sentences (drop "Azure" from the provider grouping; scope the first bullet
to request bytes) and re-run a fresh review round on the revised doc.

Counts: **0 Blocking, 2 Major, 0 minor, 5 nit.**

SEAT A round 1: REQUEST CHANGES — 0 Blocking, 2 Major, 0 minor, 5 nit
