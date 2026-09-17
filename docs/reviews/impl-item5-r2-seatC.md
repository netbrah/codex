# Item 5 — Docs, gates, spec final pass: review round 2, SEAT C (independent verification of r1 resolutions + source verify)

Bead: `apex-ayl.52` · Branch: `feat/normalize-content-types-vllm` · Date: 2026-09-16
Reviewer: `/root/item5_r2_seat_c` (independent; r1 seats A/B on record)

## Method

- Read all five artifacts (seam doc, spec, breakdown Item 5, evidence doc incl.
  r1 resolution log, both r1 seat reports) and re-derived every check against
  the current tree. Did not trust the resolution log: each "FIXED" claim was
  located in the artifact text and each factual claim was re-verified at
  source (`git show` on the four item commits, direct file reads, `od` on file
  tails, `diff` of the spec vs the v4 archive).
- Gates: not re-run (per r2 mandate; SCS build / Mac load). Attestation
  checked for internal consistency + corroboration against r1 seat A's
  independent gate re-run.
- READ-ONLY: no product code touched, no artifact edits, no git write ops.

## Verdict

**REQUEST CHANGES — 0 Blocking / 1 Major / 0 Minor / 1 Nit.**

The round does NOT meet the clean bar (0 Blocking AND 0 Major). The single
Major is a residual of r1 finding M-A1: the scoping was appended to the
Decision sentence but the stale over-broad subject was not reworded, so the
sentence still contradicts its own parenthetical on the face of the text.
Everything else in the r1 resolution log verified as fixed/consistent, and
every load-bearing factual claim in the seam doc verified at source.

## Findings

### F-C1 [Major] — M-A1 only partially fixed: stale "OpenAI/Azure providers are untouched" subject survives (seam doc :81-86)

`docs/responses-compat-seam.md:81-86` ("### Decision"), current text:

> all other codex tools use). **OpenAI/Azure providers are untouched: they
> keep the native custom+grammar path** — scoped to providers **named
> `OpenAI`** (spec §3.4 invariant: outbound request bytes for `OpenAI`-named
> providers are unchanged; `is_openai()` is name-anchored, so Azure-*named*
> providers take the function tool, as pinned by the capability test).

Claim: the r1 fix (per the resolution log) "reworded" the Decision paragraph
by scoping "untouched / native custom+grammar path" to providers named
`OpenAI` with the §3.4 invariant inline.

Evidence: the scope + invariant are present, but the main clause —
"OpenAI/Azure providers are untouched: they keep the native custom+grammar
path" — is the exact sentence r1 seat A rated Major (Finding A-1) and was not
reworded; the r1 fix appended qualifiers after it rather than replacing the
subject. Seat A's stated fix was a one-line reword ("e.g. 'OpenAI-*named*
providers are untouched …'"). As delivered the single sentence now contains
both "OpenAI/Azure providers are untouched" (main clause, false for
Azure-*named* providers) and "Azure-*named* providers take the function
tool" (parenthetical), i.e. it contradicts itself on the face of the text,
and still contradicts the corrected §2 (:56-58) and invariants bullet 1
(:274-277) in the same doc. Source agrees with the spec, not the main
clause: `provider.rs:372` computes `apply_patch_function_tool:
!self.info.is_openai()` and the capability test
(`provider.rs:707`) pins the Azure-named case → `true` (verified this round).
Classification follows r1's own rule: a seam-doc statement the spec
contradicts = Major. Fix: one-line reword of the subject, e.g. "Providers
named `OpenAI` are untouched: they keep the native custom+grammar path"
(drop the em-dash scoping clause, which becomes redundant).

### F-C2 [nit] — evidence doc "Final state" line count/anchors stale after the r1 fixes

`docs/reviews/impl-item5-tdd-evidence.md` "Final state" (seam file "287
lines (pre-edit 226; +61)" with post-edit anchors §3 :126-156, §4 row 7
:188, §6 conflict sites :235-238, gates :245-251, invariants :271-282).

The current seam doc is **294 lines**; those anchors now read §3
format-remediation layer :130-161, row 7 :193, conflict sites :240-243,
gates :250-256, invariants bullet 2 :278-289 (shift +4..+7). Delta is fully
accounted for by the three r1 text fixes — M-A1 (+4, Decision 2→6 lines),
M-A2 (+2, invariants bullet 1 2→4 lines), N-1 (+1, P3 bullet 3→4 lines) =
+7 = 287→294 — all recorded in the resolution log, so this is explainable,
not a misattestation of the worker's deliverable. But the Final-state
section presents those anchors as the "final content state", and any future
line-ref audit against the current tree will mismatch — the same defect
class r1 rated minor (m-F2). Suggested fix: one-line note in Final state
("anchors below are pre-r1-fixes; post-fix positions per resolution log") or
re-derivation of the six anchors.

## r1-resolution verification

| ID | Claim | Verified against artifact | Result |
|---|---|---|---|
| M-A1 | Decision paragraph reworded/scoped | Scope + inline invariant present at :81-86, but stale subject survives | **NOT fully fixed → F-C1 (Major)** |
| M-A2 | Invariants bullet 1 reworded to §3.4 scope | :274-277 now "Outbound request bytes for providers named `OpenAI` are byte-identical to upstream (all capability flags false on `is_openai()`, which is name-anchored on `OpenAI`; Azure-*named* providers take the function tool per spec §3.4)" — fixes both halves of A-2 (Azure subject + "request bytes" not "behavior") | **Fixed** |
| m-F1 | "46" → "40" lib tests | Gate-2 section now "All 40 `codex-core` lib unit tests … passed on first try"; `grep '46'` over the evidence doc hits only `:546-547` line-refs and the quoted pre-fix text in the resolution log — no residual 46-count claim | **Fixed** |
| m-F2 | pre-edit anchors corrected | Pre-check now "Components :79, Testing :121, Out of scope :131"; monotonic with §4 :139 (old impossible pair "Out of scope :141 > §4 :139" resolved) | **Fixed** |
| N-1 | P3 bullet names all three files | :155-158 names `streaming_parser.rs`, `parser.rs` (P3.4) **and** "plus the P3.3 handler error split (`apply_patch.rs`)"; all three attributions source-verified (item-3 diff touches all three) | **Fixed** |
| N-2 | ACCEPTED — mixed path spelling | :240-243 keep the locked spec spelling (`codex-rs/` prefix) while siblings omit it — exactly the accepted state (D1) | **Consistent** |
| N-3 | ACCEPTED — `just test` gate bullet | :250 keeps `just test -p codex-apply-patch`; siblings use scoped `cargo` filters — accepted state intact | **Consistent** |
| N-4 | ACCEPTED — invariants bullet = faithful condensation | :278-289 vs spec §3.4 :592-612: core clauses verbatim ("Outbound **request bytes for providers named `OpenAI` are unchanged**"; "P2 and P3 are **provider-agnostic parser changes**: for every input upstream accepts, behavior is byte-identical; for inputs upstream *rejects*…"; "does **not** touch the diff consumer's parallel streaming boundary messages"); the four small trims ("by this work", the diff-consumer parenthetical, "v2 refinement:", the `invocation.rs:116/123/170/175` cite) and both omitted meta-sentences are accounted for in Changes #6 | **Consistent** |
| N-5 | status cell → "IMPLEMENTED (items 1-3, 2026-09-16)" | :51 reads exactly `IMPLEMENTED (items 1-3, 2026-09-16)`; date source-verified (all four item commits dated 2026-09-16) | **Fixed** |
| n-F3 | RECORDED — trailing newline normalized | `od -c` on spec tail: single trailing `\n` | **Consistent** |
| n-F4 | RECORDED — ≈58× stands | spec ships "≈58×" (:213); 58.0432 exact / 58.07 rounded-division both round to ≈58×; "58.04" phrasing only in the evidence doc, untouched as recorded | **Consistent** |
| n-F5 | RECORDED — spec §3.4 historical narrative untouched | :607-609 still carry "the seam doc §2 … 'plus an Azure branch' description is stale and is corrected in the same PR set (v1, seat B M5 / seat A M2)" — untouched per contract | **Consistent** |

## Item-5 change list re-review (breakdown §"Item 5" vs seam doc)

- **§2 `is_openai()` correction** — present (:53-61) and accurate at source:
  exact case-sensitive match vs `OPENAI_PROVIDER_NAME`, `:546-547`, constant
  `:40`, explicit no-Azure-branch with the §3.4 cite, Azure-*named*
  providers in seam scope. ✓
- **§3 P1/P2/P3 format-remediation layer** — present (:130-161) with all
  three pinned numbers matching spec §1.2 exactly: glm-5.2 37/26 **~70%
  (70.27%) "as of the pinned snapshot"**; 09-15 alone **13 of 21 ≈ 62%**;
  qwen 413/5 **~1% (1.21%)**. Arithmetic re-derived: 26/37=70.270…%,
  5/413=1.211…%, 13/21=61.9…%. Rejection classes (F1/F2 raw content + F4
  missing boundaries) match the spec breakdown (17+2+5+2=26). P1/P2/P3 file
  attributions all source-verified (see Source verification). "landed
  2026-09-16" verified against commit dates. ✓
- **Divergence-table row (parser change)** — present as row 7 (:193):
  provider-agnostic; P2 surface = function-tool path + freeform diff
  consumer + `apply_patch` CLI; P3/P3.4 = `parse_patch` pre-pass surface
  (function path + CLI + shell-intercept); diff-consumer boundary messages
  intentionally unchanged; re-apply on upstream restructure of
  `streaming_parser.rs` or `parser.rs`. Every required element present. ✓
- **§6 conflict sites** — both `codex-rs/apply-patch/src/streaming_parser.rs`
  (P2 Add-File arm + P3.1/P3.2 error-message sites) and
  `codex-rs/apply-patch/src/parser.rs` (P3.4 boundary strings) present
  (:240-243), locked spec spelling per D1. ✓
- **§6 verification gate** — `just test -p codex-apply-patch` (:250-251) and
  the raw-format probe per spec §5.4 (:252-256) present; probe wording
  matches spec §5 item 4 (standalone binary, same release build, scratch
  dir, F1-shape raw Add-File patch, exit 0 + byte-identical). ✓
- **Invariants** — spec §3.4 exact-invariant text present as bullet 2
  (:278-289, faithful condensation per N-4); bullet 1 correctly
  §3.4-scoped (M-A2). ✓
- **Spec status line** — `Status: IMPLEMENTED` with item 1 `e21f608ac4`,
  item 2 `a4d5af1f62`, item 3 `939a6dc6f4`, item 4 `a90b15d600` (each an
  exact match to the `(item N) (apex-ayl.52)` commits in `git log`,
  subjects cross-checked) + `<<ITEM5-SHA>>` placeholder for item 5 (no
  `(item 5)` commit exists yet — docs uncommitted per the task contract;
  placeholder documented in evidence Changes #7 and D5). ✓
- **Spec §1.2 Conclusion** — `≈58× qwen's ~1% (26/37 = 70.27% vs 5/413 =
  1.21%)` at :213-214. The only remaining `≈60×` in the file is the §7 R5
  map I-N1 row at :1022 (frozen historical log). ✓
- **Spec §7 + archives frozen** — `diff` of current spec vs
  `docs/reviews/apply-patch-format-spec-v4-draft.md`: exactly 10 hunks; the
  item-5 contribution is only the status block (hunk `7,8c7,14`) and the
  Conclusion (hunk `204,205c213,214`); the §7 hunks (`968,970c977,1006`,
  `971a1008,1029`) are v5 review-loop bookkeeping (round-5 seats I/J + R5
  map), pre-existing to item 5; the other six hunks are the five v5
  minor/nit fixes + the v4-archive Evidence-line entry, all mapped in the R5
  map. No archive files in the diff. ✓
- **Gates vs breakdown gate list** — 1:1 match: `just test -p
  codex-apply-patch` (115/115, exit 0), `just test -p codex-core
  apply_patch` (green criterion: 0 deterministic failures; binding grep
  empty; all 92 panics at `lib.rs:388`), `just test -p codex-model-provider`
  (capability-gate target green; 2 recorded pre-existing env TLS bedrock
  failures), `just fix -p` over the three touched crates (exit 0; D3 hazard
  triggered + restored), `just fmt` (exit 0, no changes). No
  `--all-features`/full-suite run claimed; no `Cargo.toml`/lock changes
  (verified: none in tree). ✓
- **Gate-attestation internal consistency** — Gate 2: 66+1+51=118 ✓;
  "52 unique = 51 TMT + 1 FAIL" ✓; 118 = 40 lib + 78 suite (seat B's log
  decomposition) with 40/40 lib first-try ✓. Gate 3: 81+2+1=84 ✓. D2 (TLS
  failures) and D3 (out-of-scope auto-fix) deviations recorded with
  analyses; D3 verified held this round (`git diff HEAD --
  codex-rs/core/tests/suite/openai_file_mcp.rs` empty). Corroboration: r1
  seat A's independent re-run reproduces Gate 1 115/115 exactly and shows
  the same Gate 2/3 noise class with different per-test victims — the
  stochastic signature the attestation relies on. No count mismatches found
  beyond F-C2's stale anchors (worker-run state, accounted for).

## Source verification

- **`is_openai()` name anchor** — `codex-rs/model-provider-info/src/lib.rs`:
  `:40` `const OPENAI_PROVIDER_NAME: &str = "OpenAI";`; `:546` `pub fn
  is_openai(&self) -> bool {`; `:547` `self.name == OPENAI_PROVIDER_NAME` —
  plain `&str ==`, exact/case-sensitive, single hits (same greps as worker).
  `grep -ci azure` over the file: **0** — no Azure name or branch. The seam
  §2 "no Azure branch" claim (scoped to the gate) is accurate. Note for the
  record: `provider.rs:360-366` contains an Azure-aware branch
  (`is_azure_responses_provider`) for `remote_compaction` — that is upstream
  (present at `44b9011611`, absent from the fork baseline diff
  `799ec98053`) and a separate mechanism, not a capability gate; it does not
  contradict any seam-doc claim.
- **Capability table (`model-provider/src/provider.rs`)** —
  `ProviderCapabilities` fields `flatten_namespace_tools`/`normalize_content_types`/
  `apply_patch_function_tool` at :61-63 (default false :74-76); all three
  computed `!self.info.is_openai()` at :369/:371/:372 — matches the seam §2
  table gates. Capability test at :707 has exactly the three cases the seam
  §6 invariants bullet claims (OpenAI → `false`, Azure-named → `true`,
  custom → `true`), corroborating "Azure-*named* providers take the function
  tool, as pinned by the capability test". Table commit SHAs `b4d4b12` /
  `197ea16` exist with matching subjects (normalize content types / flatten
  namespace tools).
- **No product code modified by item 5** — `git status --short`: **zero**
  modified/deleted/staged tracked files; `git diff --stat` empty. Untracked
  set = campaign docs (`docs/responses-compat-*`, `docs/reviews/impl-item5-*`,
  prior-round review docs) + pre-existing untracked `docs/superpowers/`,
  `engine-redesign-charter.md` (out of item-5 scope). **Zero non-doc
  changes. No deviation.**
- **Tree drift note (post-worker, not an item-5 deviation)** — HEAD is now
  `799ec98053` ("commit working-tree function-tool seam baseline",
  2026-09-16, "at operator direction" per message): exactly the 12 tracked
  files / 809 insertions / 20 deletions attested in the evidence doc's
  Final state (verified 1:1 via `git show --stat`), i.e. the campaign seam
  baseline the worker had documented as uncommitted. It touches no item-5
  doc and no gate-4 restored file; the commit message states doc line refs
  remain valid ("working-tree state, which now equals this HEAD"). Item-5
  docs correctly remain untracked ("commit separately after their review
  loop closes").
- **P2/P3 change sites (row 7 + §6 conflict bullets)** — `git show` on the
  item commits: item 1 `e21f608ac4` adds the P2 Add-File leniency arm to
  `apply-patch/src/streaming_parser.rs` (module doc: "Lenient add-file
  content (P2)…") + `parser.rs` module-doc line + new P2 test file; item 3
  `939a6dc6f4` adds P3.1/P3.2 teachable hunk-header/Delete-File error
  messages in `streaming_parser.rs`, the P3.4 `*** Begin Patch`/`*** End
  Patch` boundary strings in `parser.rs`, and the P3.3 handler split
  (`run_apply_patch_text` shared path + `patch`-argument handling) in
  `core/src/tools/handlers/apply_patch.rs`. Every file named in the seam
  doc's P2/P3 attributions confirmed; no file over-attributed.
- **Commit dates** — all four `(item N)` commits dated 2026-09-16; subjects
  match the spec status-line item/P-level mapping (item 1/P2, item 2/P1,
  item 3/P3, item 4/T4). "landed 2026-09-16 in the same PR set" holds.

## Gate attestation sanity (mandate 5)

- Gate list matches the breakdown Item-5 gate list exactly (5/5; no
  `--all-features`, no full `just test`, no lock changes — as contracted).
- All summary arithmetic internally consistent (Gate 1 115/115; Gate 2
  66+1+51=118 with 40/40 lib first-try; Gate 3 81+2+1=84).
- The two recorded deviations (D2 env TLS, D3 clippy hazard) are explained
  and D3's restore verified this round; D2's failures are third-party
  (`aws-smithy-http-client` rustls trust-store panic) in tests on a code
  path the item did not touch, identical on both attempts — consistent with
  pre-existing/environmental, and Gate 3's stated target (the
  capability-gate test) passed in both the worker's run and seat A's
  independent run.
- No attestation found internally inconsistent beyond the F-C2 stale
  anchors (worker-run state, fully accounted for by the recorded r1 fixes).

---

**SEAT C round 2: REQUEST CHANGES — 0 Blocking, 1 Major (F-C1: M-A1 residual
in seam doc :81-86), 0 Minor, 1 Nit (F-C2: evidence Final-state anchors
stale post-r1-fixes).** Clean bar not met. The F-C1 fix is a one-line
reword; after it lands, a fresh r3 round (or a targeted re-review of the
single sentence) is required before the loop can be declared clean.
