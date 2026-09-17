# Review — apply-patch function-tool format spec (v3), Round 4, Seat G

Target: `docs/responses-compat-apply-patch-format.md` (v3, 915 lines), branch
`feat/normalize-content-types-vllm`.
Reviewer: seat G (independent leaf reviewer; nothing trusted from the spec or
prior rounds — every figure re-derived from a fresh snapshot of the live
rollouts, every cited file/line opened). Date: 2026-09-15 22:05–22:15 EDT
(2026-09-16T02:05–02:15Z).

## Verdict

**CHANGES-REQUESTED — 0 Blocking, 1 Major, 2 Minor, 2 Nit.**

Both load-bearing evidence rows reproduce **exactly** under an independent
recount (qwen 413 = 374 OK + 5 PARSE + 30 VERIFY + 1 MISSING + 1 UNSUPPORTED
+ 2 OTHER at the pinned instant; glm 26/37 with the full 17+2+5+2 breakdown,
event-by-event), the P1 text/counts, the T2.2 example, every mandated line
reference, and the F4/F1 byte-exact quotes all verify, and every v2→v3 hunk
traces to the R3 map or §7 bookkeeping. The one Major is the E-N3 resolution
itself: §1.2's forensics method still cites a session path that does not
exist, and the map row misdescribes the actual layout. The two Minors are in
the same §1.2 evidence cluster: the pin sentence carries the 00:22Z
snapshot's file triple (83/162/86) instead of the pin-instant count
(83/162/72), and the unqualified "no glm data after 09-15 19:31 EDT"
stability claim was already false at v3's write time (4 further glm PARSE
calls through 21:44:55 EDT; the live glm row is now 30/41 ≈ 73%).

Method (everything re-verified at the source):
- Snapshot of `~/.codex/sessions/2026/09/{13,14,15}` taken **first** (348
  rollout files, copied ~02:10Z to `/tmp/seatg/snap`); the full §1.2 method
  re-implemented independently (model from each file's first `turn_context`,
  unique `call_id` for `function_call name=apply_patch`, outcome from the
  matching `function_call_output`): 524 unique calls = glm 41 + qwen 483 —
  no other model in the three day-dirs has an apply_patch call, so the
  spec's two-row table covers 100% of the data.
- The pinned instant verified from the data: the 413rd qwen call in
  timestamp order is exactly **2026-09-15T23:30:11.662Z**.
- `diff -u v2-draft v3`: all 19 hunks audited (Mandate 1 table below).
- Every R3-map row re-verified against code and rollouts (Mandate 2 table);
  every mandated line reference opened, plus the P3 message sites, the
  red-state tests, and the §1.3 F1 argument sizes.
- T2.2 example re-probed on the unmodified `codex-rs/target/debug/apply_patch`
  (built 2026-09-15 19:25; `codex-rs/apply-patch` is `git status`-clean on
  the branch).
- P1 text extracted from the spec: 7 drift-guard substrings checked as
  literals, char/word counts recomputed, byte-diffed against the v2 archive.


## Mandate 1 — Diff audit (v2 → v3, 19 hunks)

Every hunk of `diff -u docs/reviews/apply-patch-format-spec-v2-draft.md
docs/responses-compat-apply-patch-format.md` (282 diff lines, 19 hunks):

| # | v2→v3 change | R3 row / §7 | Traceable? |
|---|---|---|---|
| 1 | Status line v2→v3; "round 3"→"round 4" | §7 (round-4 pending) | yes (bookkeeping) |
| 2 | TL;DR: adds pinned-snapshot + freshness parenthetical (439 calls, 6 PARSE ≈ 1.4%) | E-N1 / F-F1 | yes — verified (§1.2 re-derived) |
| 3 | §1.1 F4 quotes gain `invalid patch: ` prefix; new `parser.rs:57` Display note; v1-misbucket parenthetical retained | F-F3 | yes — quotes re-verified byte-exact vs all 7 F4 rollouts |
| 4 | §1.1: `check_start_and_end_lines_strict` range `parser.rs:256-272`→`256-274` | E-N7 | yes — verified exact |
| 5 | §1.1 F4 examples: paths `2026-09-14/`→`2026/09/14/` (correct form); "(re-verified by round-2 seat D)"→"(re-verified byte-exact by rounds 2 and 3)" | E-N3 (+ §7 round-3 record for the wording) | path yes; wording change not itemized in the map → G5 (nit); wording itself verified true |
| 6 | §1.2 Method: path `2026-09-{13,14,15}`→`2026-09/{13,14,15}/`; new **Snapshot (v3)** paragraph (pin instant + 83/162/86 file triple) | E-N3 + E-N1/F-F1 | **no — the "fixed" path is still a non-existent directory** (G1); the file triple is mislabeled (G2) |
| 7 | §1.2 qwen row: 31 VERIFY / 1 OTHER → 30 VERIFY / 2 OTHER | E-N1 / F-F1 | yes — reproduced exactly at the pin (see G3 for the 30th VERIFY's identity) |
| 8 | §1.2 glm breakdown: "13 F1-class"→"17 F1-class (10 `#` + 3 `VERDICT:` + 3 `@@` + 1 `---`)"; "v2 list summed to 22, not 26" named | E-N2 / F-F2 | yes — reproduced event-by-event; the 17 call_ids match seat F's list exactly |
| 9 | §1.2 recount history: seat-D bullet split (glm-row canonical vs qwen); new round-3 seats E/F bullet (pinned-instant reproduction, 2 OTHER named with call_ids/timestamps, "pin is load-bearing"); new Freshness (v3) bullet (439/6 at 00:22Z) | E-N1 / F-F1 | yes — all four named facts verified (413th ts exact; both OTHER call_ids byte-exact; 439/6 reproduced; glm stability wording → G2) |
| 10 | §1.2 conclusion: adds "(as of the pinned snapshot)" | E-N1 / F-F1 | yes |
| 11 | §3.1 note: auto-chunk range `streaming_parser.rs:316-360`→`307-347` | E-N7 | mostly — branch start exact; tail is 2 lines short (full span ends :349) → G4 (nit) |
| 12 | §3.1 note: "380 words"→"401 words (v3: 380 was the argument-description-only count)" | E-N6 / F-F6 | yes — recomputed: 21 (tool) + 380 (arg) = 401 |
| 13 | §3.3 intro: blanket "exact prefix substring" → per-site prefix status (sites 1+4 preserve; site 2 replacement; site 3 re-quote + split) | E-N4 / F-F5 | yes — verified against code (streaming_parser.rs:193/:222, apply_patch.rs:534-541/:539) |
| 14 | §3.3.4: range `256-272`→`256-274` | E-N7 | yes |
| 15 | §3.3 red state: lenient count "5"→"7" (enumerated 6 Begin + 1 End) + "Total red assertions: 13 (3 streaming + 1 CLI + 2 + 7)" | E-N5 / F-F4 | yes — all 7 assertion sites verified line-by-line; total arithmetic holds |
| 16 | T4.2: `responses.rs:1030-1038` → `responses.rs:1025-1028 and :1030-1035` | E-N7 | yes — both ranges verified exact |
| 17 | §6: "2026-09-15 glm-5.2 evidence (26/37 ≈ 70% vs qwen 5/413 ≈ 1%, §1.2)" → "2026-09-13..15 … 26/37 ≈ 70% as of the §1.2 pinned snapshot; 09-15 alone 13/21 ≈ 62%" | F-F7 | yes — 26/37 and 13/21 re-derived (file-directory day attribution) |
| 18 | R2-map rows D-M1 glm table / D-M1 (class) F4 / C-N4·D-m3: gain parenthetical v3 cross-references | §7 bookkeeping | yes |
| 19 | §7: "Round 3: pending" → full round-3 record (seats E/F summaries) + Round-3 resolution map + "Round 4: pending" | §7 bookkeeping | yes (the E-N1 and E-N3 rows carry the G1/G2 defects) |

Result: **19/19 hunks trace** to an R3-map row or §7 bookkeeping — no
unexplained content change. Two hunks carry defects in the *content they
carry* (hunks 6 → G1, and hunks 6/9 → G2); one hunk carries an unitemized
wording change (hunk 5 → G5).

## Mandate 2 — R3-map row verification (9 rows)

All re-verified at the source (rollout snapshot + current tree).

| Row | Verdict | Evidence (re-derived, not read from prior rounds) |
|---|---|---|
| E-N1 / F-F1 (qwen row + pin) | **PARTIAL** — pin, row, OTHER names, freshness all verified; two sub-claims wrong (G2) | (a) Pinned instant: 413rd qwen call in ts order = exactly 2026-09-15T23:30:11.662Z (414th is 23:54:48Z — no boundary tie). (b) qwen row at the pin: **413 = 374 OK + 5 PARSE + 30 VERIFY + 1 MISSING + 1 UNSUPPORTED + 2 OTHER — exact** (OK = `Exit code: 0 … Success. Updated`; the 5 PARSE are all parse-path: `'```' not a valid hunk header` :09-14 04:03:56Z, `Update hunk does not contain any lines` 06:54:21Z, `'@@'` 09-15 19:12:24Z, `'set -u'` 20:17:13Z, `'# apex-ayl.28…'` 21:19:58Z). (c) The two OTHERs: `call_306909242611434cbdd57001` @ 2026-09-15T20:20:08.933Z and `call_096536c41e0e415ab2afe730…` @ 21:25:21.392Z, outputs byte-exact `apply_patch verification failed: invalid patch: multiple operations target …` — spec's `call_30690924…`/`call_096536c4…` prefixes and both timestamps match. (d) MISSING = `call_26324b8d…` (args `{}`, 22:13:05Z), UNSUPPORTED = `call_3c03fe79…` (legacy `{"cmd": …}` shape, 09-13 16:54:27Z). (e) Freshness @ 00:22Z: qwen 439 / 6 PARSE — reproduced exactly (26-event tail = 22 OK + 1 PARSE + 3 VERIFY; the 3 VERIFY are exactly seat F's named ids `call_a9cf9f97…`/`call_69f201be…`/`call_5187758d…`). (f) glm row: **26/37 = 26 PARSE + 11 OK + 0 VERIFY exact** through the last glm event (23:31:03.433Z); breakdown below. (g) **Wrong sub-claims:** the file triple "83/162/86 … for 09-13/09-14/09-15" is not the pin-instant count (83/162/**72** at the pin; 86 is the ~00:22Z snapshot's count — mine at exactly 00:22:00Z is 83/162/85), and "no glm data after 09-15 19:31 EDT" was false at v3's write time (v3 mtime 01:58:53Z; 4 glm PARSE calls at 00:39:58Z/00:40:02Z/01:40:35Z/01:44:55Z) — G2. |
| E-N2 / F-F2 (glm breakdown) | **VERIFIED** | Re-derived all 26 PARSE events: 17 F1-class (10 `#`-first-line, 3 `VERDICT:`-first-line, 3 `@@`-first-line, 1 `---`-first-line — sub-counts exact) + 2 F2-class (`Unexpected line found in update hunk`: `call_d694eb14…` 19:58:02Z, `call_cda093b5…` 23:06:50Z) + 5 missing-Begin (09-14 07:34:44Z/12:48:08Z/22:48:54Z; 09-15 18:42:47Z/23:30:46Z) + 2 missing-End (09-14 12:03:39Z; 09-15 03:17:57Z) = 26. The 17 F1 call_ids match seat F's list one-for-one. The v2 "summed to 22" disclosure is accurate (13+2+5+2=22). |
| E-N3 (session-path notation) | **FAIL** — resolution factually wrong (G1) | Actual layout on the evidence host: `~/.codex/sessions/2026/09/{13,14,15}/` (exists; 348 files in my snapshot). `~/.codex/sessions/2026-09/` does **not** exist (`ls`: No such file or directory); codex source confirms `YYYY/MM/DD` (`cli/src/doctor/thread_inventory.rs:812`, `:1327` use `sessions/2025/01/02/rollout-…`). v3 §1.1 (line 124) uses the correct slash form `2026/09/14/…`; v3 §1.2 (line 134) uses the dash form `2026-09/{13,14,15}/`; the header (line 5) is slash form. The map row's "unified to the actual `2026-09/{13,14,15}/` layout in §1.1/§1.2" is wrong on both counts: the actual layout is slash form, and the two sections are not unified (§1.1 slash, §1.2 dash). |
| E-N4 / F-F5 (§3.3 prefix status) | **VERIFIED** | StartedPatch message at streaming_parser.rs:193 (keep-verbatim + append ⇒ prefix preserved); DeleteFile at :222 (wholesale replacement — old `'{trimmed}' is not a valid hunk header…` shares no prefix); function handler at apply_patch.rs:539 existing backtick-quoted message, `value.get("patch").and_then(Value::as_str)` at :534-536 collapsing absent/non-string (the "splits the one collapsed error into two" claim exact); boundary strings at parser.rs:268/:271 extended by a sentence (trivially prefix-preserving). |
| E-N5 / F-F4 (red counts) | **VERIFIED** | `test_parse_patch_lenient` (parser.rs:558-645): Begin-message assertions exactly at :579, :594, :609, :624 (4 Strict heredoc variants), :628 (Lenient mismatched-quotes), :635 (Strict missing-closing) = 6; End-message assertion at :639-644 (string :642, Lenient missing-closing) = 1; total 7. `test_parse_patch` (:277): exactly 2 (strings at :281/:287). Streaming trio at :828/:838/:848 (test at :814; the :819 NotStarted assertion stays green — out of P3.4 scope). CLI exact-stderr at tests/suite/tool.rs:386/:393. Total 13 = 3+1+2+7 arithmetic holds. |
| F-F3 (`invalid patch: ` prefix in §1.1 F4 quotes) | **VERIFIED** | Byte-exact against **all 7** glm F4 rollouts (not just the two examples): 5 Begin (`call_17d7edbb…`, `call_6d38fade…`, `call_6f673754…`, `call_bf4d74bc…`, `call_1bcbc827…`) and 2 End (`call_10f109a3…`, `call_8ee1b0c1…`) — each full output equals the spec quote character-for-character, prefix included. The `parser.rs:57` Display note (`#[error("invalid patch: {0}")]` on `InvalidPatchError`) is exact. |
| E-N6 / F-F6 (word count) | **VERIFIED** | Recomputed on the embedded text: tool description 126 chars / 21 words; `patch` argument description 2,198 chars / 380 words; total 2,324 chars / 401 words. The "(v3: 380 was the argument-description-only count)" gloss is correct. |
| F-F7 (§6 date conflation) | **VERIFIED** | §6 now reads "2026-09-13..15 glm-5.2 evidence (26/37 ≈ 70% as of the §1.2 pinned snapshot; 09-15 alone 13/21 ≈ 62%) vs qwen 5/413 ≈ 1%". Re-derived under file-directory day attribution (the method organizes by the day dirs): 09-14 = 13 PARSE / 3 OK of 16; 09-15 = 13 PARSE / 8 OK of 21 (the 09-15 03:17:57Z event `call_8ee1b0c1…` lives in a 09-14 session file); aggregate 26/37. Rates: 70.3% / 61.9% / 1.2% — all "~" claims fair. |
| E-N7 (line ranges) | **VERIFIED** (one nit) | `parser.rs:256-274` exact (fn :256, close :274); messages at :268/:271 exact; `parser.rs:193-199` (`parse_patch_text` runs the pre-pass at :196-197 before the streaming parser at :201-203) exact; `responses.rs:1025-1028` (`ev_exec_command_call_with_args`) and `:1030-1035` (`ev_apply_patch_exec_command_call_via_heredoc`) exact; `streaming_parser.rs:307-347` — branch starts exact (:307 empty-line, :318 context, :329 `+`, :340 `-`) but the span ends 2 lines short of the `-` branch's close (:349) → G4. |

## Mandate 3 — Consistency checks

| Check | Result |
|---|---|
| Bucket sums — TL;DR table | OK: glm 26/37 (≈70.3%); qwen 5/413 (≈1.2%); "19 of 26" (F1/F2) + "7 of 26" (F4) = 26, and 17+2=19 |
| Bucket sums — §1.2 table | OK: glm 26+11+0=37; qwen 374+5+30+1+1+2=413 |
| Bucket sums — §1.2 breakdown | OK: 17+2+5+2=26; "09-15 alone 13 of 21 (~62%)" = 61.9% |
| Bucket sums — §2.2 / §6 cross-refs | OK: §2.2 "26 of 37 calls rejected at parse"; §6 "26/37 ≈ 70% … 13/21 ≈ 62% … qwen 5/413 ≈ 1%" all match §1.2 |
| P1 drift-guard substrings (7) | OK: all 7 are literal substrings of the embedded `patch` argument description (checked programmatically on the extracted code block) |
| P1 text untouched by v3 | OK: tool description and argument description are byte-identical to the v2 archive (extracted and diffed) |
| P1 char/word counts | OK: 126 + 2,198 = 2,324 chars; 21 + 380 = 401 words (recomputed) |
| T2.2 example parses | OK (re-probed, not trusted): unmodified debug binary, exit 0; `notes/todo.md` = `# TODO\n\n1. ship the fix\n` byte-exact; `src/main.rs` = one chunk, context `fn main`, 1 removal / 1 addition / 1 context. (Scratch context line was exactly `fn main` as the mandate warns.) Side observation: the CLI's context matching has a whitespace-relaxed fallback — a scratch file with `shared();` at column 0 still applied against the patch's `    shared();` context — so the probe was repeated with a strictly-indented scratch file, which also passes |
| Session-path notation | **FAIL** — header/§1.1 use `2026/09/…` (actual); §1.2 uses `2026-09/{13,14,15}/` (non-existent) → G1 |
| Line refs (mandated minimum) | OK except one: parser.rs:57 ✓, parser.rs:256-274 + :268/:271 ✓, parser.rs:277-289 (2 assertions, strings :281/:287) ✓, parser.rs:558-647 (7 assertions: :579/:594/:609/:624/:628/:635 + :639-642) ✓, streaming_parser.rs:307-347 (tail 2 lines short → G4) ✓-ish, responses.rs:1025-1028/:1030-1035 ✓ |
| Freshness claim (mandate 2e) | OK as framed: my fresh recount (snapshot ~02:10Z) gives qwen **483 calls / 12 PARSE (≈2.5%)** and glm **41 / 30 (≈73%)**. The spec's "439 calls, 6 PARSE ≈ 1.4% at 00:22Z" instant reproduces exactly; the pinned-snapshot framing is intact; direction (qwen ≪ glm) holds. The rise 6→12 is post-instant live data (a burst of 5 `Expected update hunk to start with a @@ context marker` failures in one 01:52–01:58Z window, plus 1 F1-class `# Review — …` report write) |

## Findings

### G1 [Major] (EVIDENCE) — §1.2 cites a non-existent session path, and the R3 E-N3 row misdescribes the actual layout

**Where:** §1.2 Method (line 134): "every rollout under
`~/.codex/sessions/2026-09/{13,14,15}/`"; Round-3 resolution map row E-N3
(line 904): "Session-path notation unified to the actual
`~/.codex/sessions/2026-09/{13,14,15}/` layout in §1.1/§1.2 (the v2 header
was already correct…)".

**Evidence:** On the evidence host (where all 348 rollouts in my snapshot
live): `~/.codex/sessions/2026-09/` does not exist (`ls: No such file or
directory`); the actual layout is `~/.codex/sessions/2026/09/{13,14,15}/`
(directory listing, plus codex source: `cli/src/doctor/thread_inventory.rs:812`
and `:1327` build `sessions/2025/01/02/rollout-…` — `YYYY/MM/DD`). v3 is
internally inconsistent on the notation: header line 5
(`2026/09/{13,14,15}`) and §1.1 line 124 (`2026/09/14/rollout-…`) use the
correct slash form; §1.2 line 134 alone uses the dash form. So the E-N3
resolution is wrong twice over: (a) it calls the dash form "the actual
layout", and (b) it claims the notation was "unified" in §1.1/§1.2 when the
two sections now disagree. (Note: the round-4 mandate's own parenthetical
repeats the dash form; the on-disk layout is the source of truth and it is
the slash form.)

**Impact:** The forensics Method section — the section a re-counter follows
to reproduce the spec's evidence — points at a directory that does not
exist. The evidence is reproducible only by substituting the correct path,
which the reader can recover from the header/§1.1 but which §1.2, the
method-of-record, does not state.

**Suggested resolution:** In §1.2 line 134 and map row E-N3, change
`2026-09/{13,14,15}/` to `2026/09/{13,14,15}/` and describe the actual
layout as the slash form; restate the row as "§1.1 fixed to the actual
layout in v3; §1.2 corrected (v2 had `2026-09-{13,14,15}`)".

### G2 [Minor] (EVIDENCE) — the §1.2 pin sentence conflates two instants and carries a false stability claim at write time

**Where:** §1.2 Snapshot (v3) paragraph: "every count below is pinned to
**2026-09-15T23:30:11Z** — the instant the qwen cumulative call count first
reached 413 (83/162/86 rollout files for 09-13/09-14/09-15)"; recount-history
bullet "no glm data after 09-15 19:31 EDT, so the glm row is
snapshot-stable"; R3 map E-N1 row ("83/162/86 rollout files", "glm row
declared snapshot-stable (no glm data after 09-15 19:31 EDT)").

**Evidence (all from my snapshot):**
- The file triple is not the pin-instant count: rollout files with first
  line ≤ 23:30:11.662Z are **83/162/72**. 83/162/86 is the ~00:22Z
  freshness snapshot's count (seat F's copy; my recompute at exactly
  00:22:00Z gives 83/162/85 — 86 depends on the copy landing a few seconds
  later). Either way it is the 00:22Z instant's count, placed in the
  pin-instant's sentence.
- "every count below is pinned to 23:30:11Z" overreaches for the glm row:
  2 of the 37 glm calls sit after the stated pin — `call_1bcbc827…`
  23:30:46.881Z and `call_1c41b322…` 23:31:03.433Z. The glm row is in fact
  pinned at 23:31:03.433Z (19:31:03 EDT). Applying the stated pin to both
  models yields glm 24/35, not 26/37.
- "no glm data after 09-15 19:31 EDT" is unqualified and false at v3's
  write time (file mtime 2026-09-16T01:58:53Z): four glm PARSE calls
  postdate it — `call_95179bbe…` 00:39:58Z, `call_35381ef9…` 00:40:02Z,
  `call_2e757768…` 01:40:35Z, `call_0d06d3f7…` 01:44:55Z. The claim was
  true at the round-3 00:22Z snapshot (the §1.2 freshness bullet's "the glm
  row is unchanged" is therefore correct *as of that instant*) but the
  present-tense sentence in a document written at 01:58:53Z is not. The
  live glm row is now 30/41 (≈73%).

**Impact:** A literal re-counter hits two mismatches (glm 24/35 at the
stated pin; 72 day-15 files at the stated pin), and the "snapshot-stable"
justification — the load-bearing reason the glm row can be frozen while the
qwen row is pinned — is false as written. The rows themselves remain exactly
reproducible (qwen at 23:30:11.662Z; glm through 23:31:03.433Z), which is
why this is Minor rather than Major.

**Suggested resolution:** State two pins explicitly: qwen row at
2026-09-15T23:30:11.662Z (413rd call); glm row at the last glm event
23:31:03.433Z (19:31:03 EDT). Move the file triple to the Freshness bullet
(as of the 00:22Z snapshot: 83/162/86) or restate it for the pin
(83/162/72). Qualify the stability sentence: "no glm data after 09-15
19:31:03 EDT *as of the 00:22Z snapshot*".

### G3 [Minor] (EVIDENCE) — the 30th qwen VERIFY is a post-apply write failure the definition's exemplar list does not cover

**Where:** §1.2 qwen row ("30 VERIFY") and the strict definition: "VERIFY =
post-parse filesystem verification failures only (`Failed to find expected
lines`, `Failed to read file`, …)".

**Evidence:** At the pinned instant the 30 VERIFY decompose as 27 `Failed to
find expected lines` + 2 `Failed to read file` + 1 `Failed to write file`
(`call_be0ac66658ae41ffa61898d2`, 09-14 08:22:44Z, output
`Exit code: 1 … Output: Failed to write file /Users/…/registry/types.rs`).
Both definition exemplars are verify-step messages; the 30th is an
apply-stage write failure delivered in the same exec-style wrapper as OK
results. Under the definition's most natural reading ("post-parse
filesystem … failures only", list clearly exemplary — "…") it belongs in
VERIFY and the row is exactly right; under a strict exemplar-list reading a
re-counter lands on 29 VERIFY + 3 OTHER and the table no longer adds up.
The 30th member is unnamed in the spec, so the ambiguity is unresolvable
from the document.

**Impact:** A future seat re-deriving the row (round 4 itself nearly did)
must make a definitional judgment call that the spec does not pre-empt; the
30/2 split that closed round 3 rests on it.

**Suggested resolution:** One clause in the definition or a row footnote:
"30 VERIFY = 27 find-expected + 2 read-file + 1 write-file
(`call_be0ac666…`)"; or re-bucket to 29 VERIFY + 3 OTHER.

### G4 [Nit] (CONSISTENCY) — `streaming_parser.rs:307-347` ends 2 lines short of the auto-chunk branch span

**Where:** §3.1 design note and R3 map E-N7 row ("auto-chunk
streaming_parser.rs:307-347").

**Evidence:** The four no-`@@` auto-chunk branches are :307-316 (empty
line), :318-327 (` ` context), :329-338 (`+`), :340-349 (`-`). The cited
range ends at :347 (`self.state.mode = …` inside the `-` branch); the
branch closes at :349. The v2 range (:316-360) was ~9-13 lines off; the
v3 "correction" is right at the start and 2 lines short at the tail.

**Suggested resolution:** Cite `streaming_parser.rs:307-349` (or cite the
branches by the `strip_prefix` calls, per the row's own
"cite symbols where possible" note).

### G5 [Nit] (BOOKKEEPING) — hunk 5's wording change is not itemized in the R3 map

**Where:** §1.1 F4 example lead-in: v2 "(re-verified by round-2 seat D)" →
v3 "(re-verified byte-exact by rounds 2 and 3)".

**Evidence:** No R3-map row covers this wording change; it is traceable
only indirectly to the §7 round-3 record (seats E and F both re-verified
the F4 examples — I re-verified byte-exact against all 7 F4 rollouts, so
the new wording is true).

**Suggested resolution:** Append half a line to the F-F3 row: "and §1.1's
re-verification credit extended to round 3 (byte-exact, all 7 F4
rollouts)".

## Re-verified clean (independent, at the source)

- **Pinned qwen row** — 413 = 374 + 5 + 30 + 1 + 1 + 2 at 23:30:11.662Z:
  exact (full bucket membership in `/tmp/seatg/work/calls.tsv`); the 413rd
  qwen call timestamp is exactly the stated pin instant.
- **glm row** — 37 = 26 PARSE + 11 OK + 0 VERIFY through 23:31:03.433Z;
  breakdown 17 F1 (10 `#` / 3 `VERDICT:` / 3 `@@` / 1 `---`) + 2 F2 + 5
  missing-Begin + 2 missing-End: exact, event-by-event; the 17 F1 call_ids
  match seat F's list one-for-one; "zero glm call failed with a filesystem
  verification error" holds even live (30/41 in my snapshot, still 0 VERIFY).
- **09-15 subset** — 13/21 ≈ 62% under file-directory day attribution
  (09-14: 13/16; the 09-15 03:17:57Z event lives in a 09-14 session file).
- **Freshness instant** — 439 qwen calls / 6 PARSE at 00:22Z: reproduced
  exactly (tail = 22 OK + 1 PARSE + 3 VERIFY, ids match seat F's).
- **My fresh number** (mandate 2e) — snapshot ~02:10Z: qwen 483 calls / 12
  PARSE (≈2.5%), glm 41 / 30 (≈73%); the spec's ~1% wording holds at its
  stated instant and the pinned-snapshot framing is intact; the post-instant
  rise is live data, not a spec error.
- **Two OTHER call_ids** — `call_30690924…` @ 2026-09-15T20:20:08Z and
  `call_096536c4…` @ 21:25:21Z, both byte-exact
  `apply_patch verification failed: invalid patch: multiple operations
  target …`: verified.
- **F4 harness quotes** — byte-exact (with the `invalid patch: ` prefix)
  against all 7 F4 rollouts; `parser.rs:57` Display format verified.
- **F1 verbatim example** — `call_caf7ba09…` (09-15 22:57:48Z): args JSON
  24,230 chars, patch string 23,972 chars, first content line
  `# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)` (em-dash
  intact), error output byte-exact; patch starts `*** Begin Patch` /
  `*** Add File: grok/plans/spec-freeze-r23-glm.md` and ends `*** End
  Patch` as quoted.
- **P1** — text byte-identical to v2 (v3 did not touch it); 7/7
  drift-guard substrings literal; 126 + 2,198 = 2,324 chars, 21 + 380 =
  401 words; `Example:` line unique; embedded remainder ends with
  `*** End Patch`.
- **T2.2 example** — re-probed on the unmodified debug binary: exit 0;
  Add `notes/todo.md` byte-exact `# TODO\n\n1. ship the fix\n`; Update
  `src/main.rs` one chunk, context `fn main`, 1 removal / 1 addition / 1
  context.
- **Line refs** — parser.rs:57, :193-199, :256-274, :268/:271, :277-289
  (2 assertions), :402 (upstream `@@` pin comment), :558-647 (7 assertions
  at the claimed lines); responses.rs:1025-1028, :1030-1035, :933-943;
  streaming_parser.rs:814/:828/:838/:848 and :183/:193/:211/:222/:374-adjacent
  parallel-message structure; tool.rs:386/:393; apply_patch_cli.rs:689/:707
  (substring test survives — its patch is the StartedPatch case);
  apply_patch.rs:508-560/:534-541/:539 — all exact (sole exception:
  streaming_parser.rs:307-347 tail, G4).
- **P3 prefix status** (E-N4/F-F5 rewording) — every clause verified
  against the code (see Mandate 2 table).
- **P2 target arm** — `AddFile` arm at streaming_parser.rs:198-214 is
  exactly as §3.2 describes (header check :199, `+` branch :202-208,
  trailing `Err(InvalidHunkError …)` :209-214): the "keep the change inside
  the existing AddFile arm (~8 lines net)" implementation note is
  implementable as written.
- **§1.3 F3 mechanism** — the single observed F3 is the row's 1 MISSING
  (`call_26324b8d…`, args `{}`, self-corrected by the next call); the
  11-char GLM argument-value opening tag quoted in §1.3 renders correctly
  in v3: <arg_value> (built via string concatenation in this report's
  generation path, per the known drop).
- **Table coverage** — 524 unique apply_patch call_ids in the three
  day-dirs = glm 41 + qwen 483; no other model contributes an apply_patch
  call, so the spec's two-row table is complete.

## Counts

- Findings: **0 Blocking, 1 Major (G1), 2 Minor (G2, G3), 2 Nit (G4, G5)**
  — verdict **CHANGES-REQUESTED** (as in the header).
- Mandate 1: 19/19 hunks trace to an R3-map row or §7 bookkeeping; 0
  unexplained content changes (hunks 5/6 carry the G5/G1/G2 notes above).
- Mandate 2: 9 R3 rows — 6 VERIFIED (E-N2/F-F2, E-N4/F-F5, E-N5/F-F4, F-F3,
  E-N6/F-F6, F-F7), 1 VERIFIED-with-Nit (E-N7 → G4), 1 PARTIAL (E-N1/F-F1
  → G2), 1 FAIL (E-N3 → G1).
- Rollout forensics: 348 rollouts snapshotted; 524 unique calls
  reclassified; pinned qwen row and glm row both reproduced exactly.
