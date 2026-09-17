# Review — apply-patch function-tool format spec (v2), Round 3, Seat E

Target: `docs/responses-compat-apply-patch-format.md` (v2, 846 lines)
Reviewer: seat E (independent leaf reviewer; nothing trusted from the spec or
prior rounds — every figure re-run, every cited file/line opened, all six
mandated probes executed). Date: 2026-09-15, ~20:30–21:00 EDT.

## Verdict

**CHANGES-REQUESTED — 0 Blocking, 2 Major, 3 Minor, 2 Nit.**

The design (P1/P2/P3/P3.4) is implementable as written, the parser claims all
probe true on the unmodified binary, every code line-reference in the Round-2
resolution map verifies (modulo two minor drift nits), and the glm evidence
row — the spec's crux — reproduces exactly under an independent recount.
The two Majors are both in the §1.2 evidence narrative: (N1) the qwen line's
"reproduces exactly … validates the recount pipeline" claim does not
reproduce under the documented method, and (N2) the new §1.2 glm PARSE
breakdown does not sum to 26 and contradicts the TL;DR/R2-map "19
content-class" figure.

Method (everything re-verified at the source):
- Independent full recount of every rollout under the real session dirs
  (script implementing §1.2's documented method: first `turn_context` model
  per file, unique `call_id`, outcome from the matching
  `function_call_output`), run both as-of the v2 write time and as of review
  time; non-OK calls individually inspected.
- All 22 hunks of `diff -u v1-draft v2` audited against the R2 map.
- Every R2-map row checked against current source
  (`streaming_parser.rs`, `parser.rs`, `invocation.rs`, `lib.rs`,
  `apply_patch.rs`, `apply_patch_spec.rs(+_tests)`, `registry.rs`,
  `model-provider-info/src/lib.rs`, `provider.rs`, `responses.rs`,
  `apply_patch_cli.rs`, `tool.rs`, `default.md`, `apply_patch.lark`,
  `spec_plan.rs`, `test_codex.rs`).
- Six mandated probes on the unmodified
  `codex-rs/target/debug/apply_patch` (built 2026-09-15 19:25; no
  `codex-rs/apply-patch` source newer than the binary; the branch makes no
  changes to that crate, so it is unmodified for P2/P3 purposes).
- F1 verbatim example, §1.3 sizes, and all three F4 rollout examples
  re-checked byte-exact in the rollouts.

## Mandate 1 — Diff audit (v1 → v2, 22 hunks)

Every v1→v2 change traced to a Round-2 resolution-map row (or §7
bookkeeping), except two hunks.

| # | v1→v2 change | R2 row | Traceable? |
|---|---|---|---|
| 1 | Header Evidence line: session path `2026/09-1{3,4,5}` → `2026/09/{13,14,15}` (v2 header is the actual layout); v1-archive pointer added | (archive pointer: §7 bookkeeping) | **NO for the path change — see N3** (unexplained fix; v2 header is correct but inconsistent with the §1.1/§1.2 lines) |
| 2 | Status line v1→v2 + "round 3" | §7 round-3 pending | yes (bookkeeping) |
| 3 | TL;DR: "method and full recount history"; glm 37/26/~70%; 19+7 class split; P3.4 mention | D-M1; D-M1 (class) F4; D-M1 (recommend) | yes |
| 4 | §1.1 F1: "re-verified by round-1 and round-2 seats"; `@@`/`---`-variant parenthetical | D-M1 glm table | yes |
| 5 | §1.1: new F4 section (messages, pre-pass origin, 3 rollout examples) | D-M1 (class) F4 | yes (content verified byte-exact; path notation issue → N3) |
| 6 | §1.2 Method: session path `2026/09-{13,14,15}` → `2026-09-{13,14,15}` | none | **NO — see N3** |
| 7 | §1.2: strict classification definitions; 37/26/11/0 table; PARSE breakdown; recount history; conclusion; SHA-256 prefix | D-M1; C-N5 | yes (breakdown arithmetic → N2) |
| 8 | §2.2: "(26 of 37 calls rejected at parse, §1.2 …)" | D-M1 | yes |
| 9 | §3 scope: `streaming_parser.rs` "(AddFile state)"; adds `parser.rs` (P3.4) | (scope, v2) | yes |
| 10 | §3.1 Rules 1/2/4 additions (empty file; multiple `@@`; Delete+Add rename) | D-n1; C-N3; D-n2 | yes |
| 11 | §3.1 notes: false-`@@` replacement; empty-file note; rename-only note; 5→7 drift substrings; T2.2 re-verification; length re-measure | C-N1/D-m1; D-n1; D-n2; C-N3; C-N4/D-m3 | yes |
| 12 | §3.2: "v2 additions marked"; End-Patch truncation justification corrected; zero-content and zero-chunk edges added | D-m2; D-n1; D-n2 | yes |
| 13 | §3.3: four sites; P3.4; new intro sentence ("pre-existing text remains an exact prefix substring"); red state extended to the two parser tests | D-M1 (recommend) | hunk yes; the new intro sentence is factually wrong for 2 of 4 sites → N4 |
| 14 | §3.4: revisit trigger +F4; invariant P3.4 pre-pass-only refinement | (scope, v2) | yes |
| 15 | §4 intro: "four existing assertions" → generic rewrite sentence | bookkeeping (T1 grew) | yes |
| 16 | §4 T1: title gains "+ boundary messages"; item 14 added; T1.9b both variants | D-M1 (recommend); D-m2 | yes |
| 17 | §4 T2.1: 5 → 7 drift substrings | C-N3 | yes |
| 18 | §4 T4.2: scaffold block (new helper pair, `mount_apply_patch` unusable) | D-M2 | yes |
| 19 | §5.4: raw-format probe mechanism pinned to standalone binary; wiretap passivity | C-N2 | yes |
| 20 | §6: 26/37 evidence; divergence table + pre-pass surface; conflict sites + `parser.rs`; gate wording | D-M1; D-M1 (recommend); (scope, v2) | yes |
| 21 | §7: R1-map v2 notes; round-2 seat summaries; R2 map; round-3 pending line | the map itself | yes |
| 22 | No other unexplained changes | — | — |

## Mandate 2 — Round-2 resolution map audit (v1 → v2)

Status per row: VERIFIED / PARTIAL / MISSING. All file/line claims below were
re-opened in the current tree; probes P(a)–P(f) refer to the unmodified
`codex-rs/target/debug/apply_patch` binary.

| Row | Status | Note |
|---|---|---|
| D-M1 glm table | PARTIAL | glm row **reproduces exactly** under my independent recount: 37 unique calls = 26 PARSE + 11 OK + 0 VERIFY (identical as-of the v2 write time and at review time — no new glm data since 19:31 EDT); F4 = 5 Begin + 2 End verified event-by-event; 19 content-class verified (17 F1-shaped + 2 F2 — see N2 for the spec's wrong 13/2 split); 09-15 subset 13/21 verified; recount-history disclosure present and accurate (v0 retracted, seat A 18/36 + 4/405, v1 19/36 mis-bucketed, denominator 36→37). **But the same row's "qwen 413 exact reproduction named as method validation" is not reproducible — see N1.** |
| D-M1 (class) F4 | VERIFIED | §1.1 F4 present with the exact two messages; origin `check_start_and_end_lines_strict` verified at `parser.rs:256` (fn spans 256-274; spec cites 256-272, messages exactly at :268/:271 ✓); run by `parse_patch_text` at :193-199 before the streaming parser ✓; all three rollout examples byte-exact: `…T03-25-55` call `call_17d7edbb…` = exactly `*** Add File: …` + `*** End Patch` (no Begin, no content) → Begin error; `…T07-58-02` call `call_10f109a3…` last line `+*** End Patch` → End error; `…T14-33-27` call `call_bf4d74bc…` first line `*** Add File: …` → Begin error. (Path notation wrong — N3.) |
| D-M1 (recommend) | PARTIAL | P3.4 present: one-sentence append at both pre-pass sites (Begin :268, End :271); streaming parallels at `streaming_parser.rs:168/:184/:374` verified to carry the same strings and explicitly left out of scope ✓; `parse_patch` caller set verified (function handler `apply_patch.rs:411`, CLI `lib.rs:370`, shell-intercept `invocation.rs:116/123/170/175`); red state named for `test_parse_patch` (2 assertions ✓) and `test_parse_patch_lenient` — **but the claimed "5 exact-string assertions" is actually 7 (N5).** |
| D-M2 T4.2 scaffold | VERIFIED | `mount_apply_patch` (`apply_patch_cli.rs:228`) verified to always pass `ev_apply_patch_custom_tool_call`; `FunctionApplyPatchHandler::matches_kind` Function-only (`apply_patch.rs:603-605`) ✓; registry mismatch message "tool … invoked with incompatible payload" (`registry.rs:549-550`) ✓; generic `ev_function_call` constructor at `responses.rs:933-942` ✓; proposed `ev_apply_patch_function_call` fits `apply_patch_responses`' `fn(&str, &str) -> Value` parameter (apply_patch_cli.rs:266-283) — scaffold executable; `read_file_text` harness method (apply_patch_cli.rs:323) ✓. Cited range `responses.rs:1030-1038` is slightly loose (`ev_exec_command_call_with_args` is at :1025-1028) — N7. |
| C-N1 / D-m1 false `@@` note | VERIFIED | Replacement statement verified: P(b) Update hunk with no `@@` applies cleanly (auto-chunk); auto-creation branches verified in code (`streaming_parser.rs:307-347`, spec cites 316-360 — N7); upstream pin verified at `parser.rs:402` (comment + assertion); zero-chunk rejection pinned at `streaming_parser.rs:858` and reproduced by P(c) ("Update file hunk for path 'a' is empty"); second `@@` on an empty chunk rejected (code :265-271); new `@@` after `*** End of File` accepted (code :233-259); grammar deltas (`change?` admits move-only; `change` ends at `eof_line?`; `add_line+`) verified against `apply_patch.lark`. |
| C-N2 §5.4 probe | VERIFIED | §5.4 now pins the standalone-binary mechanism; I executed it: the unmodified binary rejects the F1-shaped raw patch (P(f): "Invalid patch hunk at line 3: '# heading' is not a valid hunk header…", exit 1, no file written) and shares `parse_patch` with the function handler (`apply_patch.rs:411`) ✓; `~/bin/wiretap.py` verified passive (stdlib-only forwarder; `do_POST` = log + `urlopen` forward; no injection path). |
| C-N3 multi-`@@` rule | VERIFIED | P1 Rule 2 teaches multiple `@@` chunks; `multiple `+"`@@` chunks`" is a literal substring of the embedded text (all 7 drift-guard substrings verified literal); `scenarios/003_multiple_chunks` fixture exists; Example unchanged; P(a) re-verified the T2.2 assertions (Add `notes/todo.md` = `# TODO\n\n1. ship the fix\n` byte-exact; Update `src/main.rs` one chunk, context `fn main`, 1 removal/1 addition/1 context). |
| C-N4 / D-m3 length | PARTIAL | Re-measured on the final v2 text: tool description 126 chars ✓, `patch` argument description 2,198 chars ✓, sum 2,324 ✓ — all three exact. Word count "380" measured 401 (N6). 1K-gate note present. |
| C-N5 disclosure | VERIFIED | Full recount history present in §1.2 (v0 27/276 retracted; seat A 18/36 + 4/405 with ±8/±1 notes; v1 19/36 mis-bucket + 36→37; canonical 26/37). |
| C-N6 / D-n3 line refs | VERIFIED | `invocation.rs:235-241` with message at :237 ✓ (exact); boundary messages at `parser.rs:268/:271` ✓ (exact); `apply_patch.rs:508-560` / :411 / :603-605 ✓; `lib.rs:370` ✓; `model-provider-info/src/lib.rs:546` ✓; `apply_patch_spec_tests.rs:40` ✓; `tool.rs:386` ✓; `streaming_parser.rs:814` ✓; `default.md:132` ✓ (the legacy `{"command": [...]}` example is exactly on line 132). Remaining minor drift → N7. "Re-verified and corrected" is thus slightly overclaimed, not wrong in substance. |
| D-m2 truncation justification | VERIFIED | Corrected text verified against P(d): prefixed `+*** End Patch` preserved verbatim (file = `line1\n*** End Patch\n`), raw final `*** End Patch` truncates (file = `line1\n`); mechanism confirmed in code (structural check on trimmed line, `+` branch takes raw line); T1.9b pins both variants. |
| D-n1 empty file | VERIFIED | P1 Rule 1 teaches it; P(e) confirms zero-content Add-File creates a 0-byte file on the unmodified binary; grammar `add_line+` would not admit it (verified); T1 edge items present. |
| D-n2 rename-only | VERIFIED | P1 Rule 4 teaches Delete+Add; P(c) confirms the zero-chunk rejection is pre-existing with the exact message; "future-P3 candidate" noted; Delete+Add rename is representable (distinct paths, no one-hunk-per-file conflict). |
| (scope, v2) | VERIFIED | §3 scope gains `parser.rs` ✓; §6 conflict sites gain `parser.rs` ✓; §3.4 invariant gains the P3.4 pre-pass-only surface refinement ✓; revisit trigger now F1/F2/F3/F4 ✓. |

## Mandate 3 — New problems introduced or exposed by v1→v2

### N1 [Major] — qwen row's "reproduces exactly (413 = 374+5+31+1+1+1) … validates the recount pipeline" is not reproducible under the documented method

**Where:** §1.2 recount-history bullet 4 — "Under this method the qwen3.8-27b table **reproduces exactly** (413 = 374 + 5 + 31 + 1 + 1 + 1), which validates the recount pipeline" — and the qwen table row ("MISSING 1, UNSUPPORTED 1, OTHER 1"); named as the R2 row D-M1 fix ("qwen 413 exact reproduction named as method validation").

**Evidence:** Independent full recount implementing the documented method (model from each file's first `turn_context`, unique `call_id`, outcome from the matching `function_call_output`) over the live session dirs:
- as-of the v2 write (2026-09-16T00:18:42Z): qwen total = 434 = 392 OK + 6 PARSE + 32 VERIFY + 2 OTHER + 1 MISSING + 1 UNSUPPORTED (full run since 09-13: 440);
- at the unique cumulative point where the total first reaches 413 (2026-09-15T23:30:11Z): 374 OK + 5 PARSE + **30 VERIFY + 2 OTHER** + 1 MISSING + 1 UNSUPPORTED — the spec's total, but not its 31 VERIFY / 1 OTHER split;
- the two OTHER events are both plain `invalid patch: multiple operations target …` rejections (2026-09-15T20:20:08Z and 21:25:21Z) — OTHER under the spec's own definitions (VERIFY = post-parse filesystem failures only; OTHER = other rejections), so one appears mis-bucketed to VERIFY in the table;
- the method pins no snapshot time and the data is live, so no cutoff reproduces "413 = 374 + 5 + 31 + 1 + 1 + 1".

**Impact:** The one sentence the spec uses to validate its recount pipeline (and the R2 row's named fix) does not reproduce under the method it names; a re-running reader gets 434 now and 30/2 at the 413 point. The "exact" status of the qwen row that the TL;DR ~1% rate leans on is not supported.

**Suggested resolution:** Pin a snapshot timestamp in the method; re-tabulate the qwen row at that pinned snapshot (or mark the row "as of <ts>"); fix the VERIFY/OTHER mis-bucket; re-derive (or soften) the "reproduces exactly" claim from the pinned numbers.

### N2 [Major] — §1.2 glm PARSE breakdown sums to 22, not 26, and contradicts the spec's own 19+7 split

**Where:** §1.2: "glm PARSE breakdown (26): 13 F1-class …, 2 F2-class …, 5 missing `*** Begin Patch` (F4), 2 missing `*** End Patch` (F4)."

**Evidence:** 13 + 2 + 5 + 2 = 22 ≠ 26, the number the breakdown is introduced as decomposing, and it contradicts the TL;DR ("F1/F2, 19 of 26"; "F4, 7 of 26") and R2 row D-M1 ("19 content-class + 7 boundary-class F4"). My event-level recount of the 26 PARSE rejections: F1-class = 17 (10 raw `#`-first-line + 3 `VERDICT:` + 3 `@@` + 1 `---`), F2-class = 2, F4 = 7 (5 missing Begin + 2 missing End) → 26. The 5/2 F4 split and the 2 F2 are correct; the F1 count is 17, not 13.

**Impact:** A load-bearing evidence figure is arithmetically wrong and self-contradictory: the breakdown cannot reproduce the 19 content-class figure the TL;DR, the R2 map, and the §6 evidence all rely on. (The glm 26/37 ≈ 70% rate itself is exact — see Checked and clean.)

**Suggested resolution:** Correct to "17 F1-class …, 2 F2-class …, 5 missing `*** Begin Patch`, 2 missing `*** End Patch`" (17 + 2 + 5 + 2 = 26).

### N3 [Minor] — session-path notation wrong in §1.2 and §1.1, inconsistent with the (correct) header, and unexplained in the R2 map

**Where:** §1.2 Method line ("every rollout under `~/.codex/sessions/2026-09-{13,14,15}`"); §1.1 F4 rollout examples ("`~/.codex/sessions/2026-09-14/rollout-2026-09-14T03-25-55-*`", "`…/2026-09-15/rollout-2026-09-15T14-33-27-*`"); the header (line 5) is the one correct notation.

**Evidence:** The actual layout is `~/.codex/sessions/2026/09/{13,14,15}/rollout-*.jsonl` (verified on disk). v1 wrote `2026/09-1{3,4,5}` (header) and `2026/09-{13,14,15}` (§1.2) — both wrong. v2 hunk 1 fixes the header to the actual layout, but the R2 map credits only the added archive pointer; hunk 6 rewrites §1.2 to yet another wrong notation (`2026-09-{13,14,15}`) with no map row; the new §1.1 F4 examples (hunk 5; content byte-verified) use a third wrong notation (flat `2026-09-14/` day dirs).

**Impact:** A reader who tries to open the cited rollout paths (to re-verify the N1/N2 recounts or the F4 examples) will not find the sessions at the cited paths; the spec contradicts itself about where its own evidence lives.

**Suggested resolution:** Use `~/.codex/sessions/2026/09/{13,14,15}/rollout-*` in §1.2 and the §1.1 examples; add an R2-map note for the header path fix.

### N4 [Minor] — §3.3 intro's "exact prefix substring" claim is false for 2 of the 4 sites

**Where:** §3.3 intro: "in each case the pre-existing text remains an exact prefix substring of the new string."

**Evidence:** False for site 2 — P3.2 is an explicit replacement ("replace the generic 'not a valid hunk header' text"), and the old DeleteFile message (`streaming_parser.rs:222`: `'{trimmed}' is not a valid hunk header. Valid hunk headers: …`) shares no prefix with the new text. False for site 3a — the existing handler message (`apply_patch.rs:539`) wraps the argument name in backticks ("…is missing the required `patch` argument") while the new text uses single quotes ("…is missing the required 'patch' argument; …"), so the pre-existing text diverges at the first quote character. Sites 1 (explicitly "keep the original sentence verbatim … append") and 4 (Begin `parser.rs:268` / End `parser.rs:271` sentences extended) do preserve the prefix.

**Impact:** Prefix-substring status is the natural verification hook (grep for old text as prefix of new; predict which substring assertions survive), and it is wrong exactly at the two sites with replacement semantics that the red-state list must account for.

**Suggested resolution:** Reword: "sites 1 and 4 extend the pre-existing sentence (prefix preserved); site 2 is a replacement; site 3a re-quotes the argument name (backticks → single quotes)."

### N5 [Minor] — `test_parse_patch_lenient` red-state count is 7, not 5

**Where:** §3.3 red-state paragraph: "`test_parse_patch_lenient` (parser.rs:558 — 5 exact-string assertions across its Strict-mode and missing-closing-heredoc cases)."

**Evidence:** `test_parse_patch_lenient` (parser.rs:558-645) contains 7 boundary-string assertions: the Begin message (`expected_error`, :575-576) is asserted at :579, :594, :609, :624 (Strict, the four heredoc variants), :628 (Lenient, mismatched quotes), :635 (Strict, missing closing); the End message at :639-642 (Lenient, missing closing). (`test_parse_patch` at parser.rs:277 has 2 exact-string assertions, :281/:287 — the spec's count for that test is correct.)

**Impact:** The red-state enumeration is the contract for what the implementer must record as red at T1; undercounting the lenient test by 2 risks an incomplete red log being accepted silently.

**Suggested resolution:** Change "5" to "7" and enumerate: 6 Begin-message assertions (4 Strict heredoc variants + 1 Lenient mismatched-quotes + 1 Strict missing-closing) + 1 End-message assertion (Lenient missing-closing).

### N6 [Nit] — P1 text "380 words" is actually 401

**Where:** §3.1 notes: "…description 2,198 chars = 2,324 chars / 380 words ≈ 550–615 tokens…".

**Evidence:** Measured on the final v2 text: 2,324 chars exact (126 tool description + 2,198 argument description); word count 401. The token estimate (≈550–615) and the 1K gate are unaffected.

**Suggested resolution:** Change 380 → 401 (or drop the word count, keep chars/tokens).

### N7 [Nit] — three line references drift from the current tree

**Where:** §3.1 notes (auto-chunk branch range); §4 T4.2 scaffold (cited `responses.rs` range); §3.3 site 4 (`check_start_and_end_lines_strict` range, cited "parser.rs:256-272").

**Evidence:** The auto-chunk branches are at `streaming_parser.rs:307-347` (spec cites 316-360); `ev_exec_command_call_with_args` is at `responses.rs:1025-1028` (spec cites 1030-1038 — that range is actually `ev_apply_patch_exec_command_call_via_heredoc` at :1030-1035, which the spec cites correctly elsewhere); `check_start_and_end_lines_strict` spans `parser.rs:256-274` (spec cites 256-272; the two message strings are exactly at :268/:271 as cited).

**Suggested resolution:** Update the three ranges; cite by symbol name + message line where possible so P3's own edits do not re-drift them.

## Checked and clean

- All six mandated probes pass on the unmodified `codex-rs/target/debug/apply_patch` (built 2026-09-15 19:25; no `codex-apply-patch` source newer than the binary; the branch makes no changes to that crate): (a) the T2.2 example applies exactly as T2.2 asserts (Add `notes/todo.md` byte-exact `# TODO\n\n1. ship the fix\n`; Update `src/main.rs` one chunk, context `fn main`, 1 removal / 1 addition / 1 context); (b) an Update hunk without `@@` applies (auto-chunk); (c) move-only Update → exact error `Update file hunk for path 'a' is empty`; (d) a `+*** End Patch` final line is preserved verbatim (file = `line1\n*** End Patch\n`) while a bare final `*** End Patch` is truncated (file = `line1\n`); (e) a zero-content Add-File creates a 0-byte file; (f) a raw `# heading` patch is rejected pre-P2 ("Invalid patch hunk at line 3: '# heading' is not a valid hunk header…", exit 1, no file written).
- The glm row reproduces exactly under the independent recount: 37 unique calls = 26 PARSE + 11 OK + 0 VERIFY, identical as-of the v2 write and as-of review time (no new glm data since 19:31 EDT); F4 = 5 missing Begin (09-14 07:34:44Z / 12:48:08Z / 22:48:54Z; 09-15 18:42:47Z / 23:30:46Z) + 2 missing End (09-14 12:03:39Z; 09-15 03:17:57Z), verified event-by-event; 09-15 subset 13/21; the full recount-history disclosure (v0 retraction; seat A 18/36 + 4/405 with ±8/±1 notes; v1 19/36 mis-bucket; denominator 36→37) is accurate.
- F1 verbatim example byte-exact: `call_caf7ba09…` in `rollout-2026-09-15T18-42-54…` (args 24,230 chars; patch 23,972 chars; the `*** Begin Patch` / `*** Add File: grok/plans/spec-freeze-r23-glm.md` / `# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)` lines); the recorded error matches §1.1. All three F4 rollout examples byte-exact (`…T03-25-55` / `call_17d7edbb…` = Begin marker only; `…T07-58-02` / `call_10f109a3…` ends `+*** End Patch`; `…T14-33-27` / `call_bf4d74bc…` missing Begin).
- Every other code line reference verifies exact (drift limited to N7's three ranges): `invocation.rs:235-241` (message at :237 "multiple operations target …"), :242-245 (no existence check), :116/:123/:170/:175; `lib.rs:370` (CLI parse), :504-535 (overwrite); `apply_patch.rs:411` (handler parse), :508-560 (argument errors; existing :539 message backtick-quoted), :603-605 (`matches_kind` Function-only); `registry.rs:549-550`; `model-provider-info/src/lib.rs:40` and :546 (no Azure branch); `provider.rs:372` (`!is_openai()`); `spec_plan.rs:1257-1271`; `default.md:132` (the legacy `{"command": [...]}` example exactly on line 132); `test_codex.rs:347` and :839-859 (default provider "OpenAI"); `responses.rs:933-942` (`ev_function_call`); `apply_patch_cli.rs:228`, :266-283, :323, :668, :689-711 (the :707 substring test survives P3.1); `tool.rs:386` and :393; `apply_patch_spec_tests.rs:40`; `streaming_parser.rs:814`, :828, :838, :848, :858 and the out-of-scope parallel messages at :168/:184/:374; `parser.rs:402` (upstream `@@` pin, comment + assertion).
- Boundary pre-pass structure as claimed: `check_start_and_end_lines_strict` is the single owner of both strings (messages exactly at parser.rs:268/:271), runs via `parse_patch_text` (:193-199) before the streaming parser; a repo-wide search finds the two boundary strings asserted only in `test_parse_patch` (2) and `test_parse_patch_lenient` (7 — see N5) plus the untouched streaming-parser tests; "is missing the required" is asserted by no existing test; the Lark grammar admits `add_line+` / `change?` / `eof_line?` as cited.
- `~/bin/wiretap.py` is passive (stdlib-only forwarder; `do_POST` = log + forward; no injection path); the Justfile `test` recipe takes `*args`, so the scoped test commands in §4 are executable as written.
- P1 text: all 7 drift-guard substrings literal in the final text; 126 + 2,198 = 2,324 chars exact; the `Example:` line is unique; the text ends with `*** End Patch`.
- §1.3: F1 argument sizes byte-verified in the rollout (args JSON 24,230 chars; patch string 23,972 chars, arrived intact); the F3 mechanism matches `docs/vllm-glm-toolcall-research.md` (glm47 parser; vllm-project/vllm#49248 open, fix #49249 open; ~5–17% of calls at 6-way concurrency, 0 sequential; temp 0), including the 11-char opening argument-value tag <arg_value> quoted in §1.3 (line 185) and the research doc; the single observed F3 is the 1 MISSING in the qwen row under both recount snapshots (self-correcting, as stated).

## Counts

- Findings: **0 Blocking, 2 Major (N1, N2), 3 Minor (N3, N4, N5), 2 Nit (N6, N7)** — verdict CHANGES-REQUESTED (as in the header).
- Mandate 1: 20/22 hunks traceable to an R2-map row or §7 bookkeeping; 2 path-notation changes unexplained (hunks 1, 6 → N3).
- Mandate 2: 14 rows — 11 VERIFIED, 3 PARTIAL (D-M1 glm table → N1; D-M1 (recommend) → N5; C-N4/D-m3 length → N6), 0 MISSING.
