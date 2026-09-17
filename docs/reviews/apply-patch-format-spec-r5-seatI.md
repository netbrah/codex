# Round-5 Seat I — apply_patch format spec v4 review

- **Role:** seat I of round 5 (two-seat independent review of v4)
- **Artifact:** `docs/responses-compat-apply-patch-format.md` (v4; status line
  "SPEC — v4 (post round-4 review; resolution maps in §7)")
- **Branch:** `feat/normalize-content-types-vllm`
- **Review date:** 2026-09-15 (EDT) / 2026-09-16 (UTC)
- **Independence:** nothing trusted from the spec, prior rounds, or the brief;
  every figure below re-derived from source in this session.

## Verdict

**APPROVED — 0 Blocking, 0 Major, 1 Minor, 3 Nit**

Every load-bearing pinned claim reproduces exactly at its stated pin/window on
a fresh snapshot (Mandate 2: 0 mismatches). All six Round-4 resolution-map rows
landed at their stated locations, including the G2 special-attention claim that
the glm row is 36, not 37, at the round-4-suggested 23:31:03.433Z instant
(Mandate 0). All nine v3→v4 hunks trace to an R4-map row or §7 bookkeeping, and
the P1 code block, §3.2 edge list, and §3.4 invariants are byte-identical
v3→v4 (Mandate 1). The 1 Minor is a data-label undercount in the F4 breakdown
parenthetical (§1.2); the 3 Nits are wording/labeling. None changes a row, a
pin, or the design.

## Method

**Snapshot (method-of-record).** Before any re-derivation I copied the three
live day-dirs to a frozen location:

- Source: `~/.codex/sessions/2026/09/{13,14,15}` (on-disk layout `YYYY/MM/DD`,
  confirmed by listing `~/.codex/sessions/2026/09/`).
- Snapshot: `/tmp/r5seatI_snap_20260916T022538Z`, copied at
  **2026-09-16T02:25:38Z** (UTC); 83/162/109 files for 09-13/09-14/09-15 at copy
  time; ~1.0 GB. All re-derivation below ran against this snapshot only.
- The v4 file's own mtime is 2026-09-16 02:24:46Z (52 s before my copy), so
  "as of the v4 snapshot" claims are checkable in my snapshot: no glm apply_patch
  event exists between 2026-09-16T01:44:55.879Z and 02:25:38Z, so the live glm
  row is identical at the v4 write instant and at my copy (30/41).

**Pipeline (re-implemented from scratch; no prior run reused).**
Python over the snapshot's JSONL rollouts:
1. Per file: first-line `timestamp` (file-counting rule); model = first
   `turn_context` payload's `model`, falling back to `session_meta` only if no
   turn_context model exists (fallback never triggered: every file with an
   apply_patch call carries a turn_context model — 484 qwen3.8-27b, 41
   glm-5.2 calls, zero other).
2. Calls: unique `call_id` for `function_call` entries with `name == "apply_patch"`,
   first occurrence kept, call's timestamp used. **Zero call_ids occur in more
   than one file** (525 unique calls), so the dedupe rule is a no-op here.
3. Outcome: the matching `function_call_output` text (present for all 525).
4. Buckets (strict): OK = success output; PARSE = parse-path rejections
   (`is not a valid hunk header`, `The first/last line of the patch must be …`,
   `Unexpected line found in update hunk`, `Expected update hunk to start with
   a @@ context marker`, `Update hunk does not contain any lines`); VERIFY =
   `Failed to find expected lines` / `Failed to read file` / `Failed to write
   file`; MISSING = `{}` args; UNSUPPORTED = legacy `{"cmd": …}` shape (output
   `unsupported call: apply_patch`); OTHER = other rejections (`invalid patch:
   multiple operations target …`). All 525 calls classify with zero
   unclassified; shape-based (args) and message-based (output) classification
   agree on every call.

**Diff audit.** `diff -u docs/reviews/apply-patch-format-spec-v3-draft.md
docs/responses-compat-apply-patch-format.md` (9 hunks / 110 changed lines);
structural sections extracted and compared byte-for-byte.

**Code line-refs.** Verified directly in the source tree
(`codex-rs/apply-patch/src/streaming_parser.rs`,
`codex-rs/core/tests/common/responses.rs`).

**Non-findings disclosed up front.**
- The 11-char GLM argument-value opening tag cited in spec §1.3 is
  deliberately **not quoted** in this report (0 occurrences, verified by
  grep); it is a vLLM-source claim, outside the rollout re-derivation
  scope of this review.

- The brief describes v4 as 972 lines; the file is 971 lines (65,170 bytes,
  newline-terminated). The spec makes no self-referential line-count claim, so
  this is immaterial to the review.
- Live-data drift vs the pinned claims (qwen 484 / glm 41 whole-snapshot
  totals, 109 09-15 files) is expected per the spec's live-directory contract
  and is disclosed in Mandate 2, not a finding.

## Mandate 0 — Round-4 fix verification (§7 R4 map, six rows)

Every cited location was opened; every "landed" claim below was independently
re-derived from the snapshot or source tree (not taken from the map's word).

| R4 row | Stated resolution | v4 location (opened) | Status |
|---|---|---|---|
| G1/H1 session path | §1.2 Method line = actual layout `~/.codex/sessions/2026/09/{13,14,15}/` (`YYYY/MM/DD`); R3-map E-N3 row and round-3 seat-E summary restated | §1.2 Method (line 134); R3-map E-N3 row (line 920); round-3 seat-E N3 (line 900) | **LANDED** — see evidence below |
| G2 pin conflation | Two explicit pins (qwen 23:30:11.662Z / triple 83/162/72; glm 23:34:04.678Z / `call_d6cac804…` OK); 83/162/86 moved to Freshness bullet with window 00:23:58.623Z ≤ t < 00:27:48.898Z; stability qualified to the freshness copy; R3-map E-N1 row annotated | §1.2 Snapshot (lines 144–152); seat-D bullet (lines 178–182); Freshness bullet (lines 191–199); TL;DR parenthetical (lines 29–31); R3-map E-N1 row (line 918) | **LANDED** — all sub-claims re-derived, incl. the 36-vs-37 claim |
| G3 30th VERIFY | VERIFY exemplars extended with `Failed to write file`; 30 VERIFY = 27 + 2 + 1, write-file call named | §1.2 definition (lines 139–142) | **LANDED** — decomposition re-derived exactly |
| G4 auto-chunk tail | `307-347` → `307-349` in all three cites; branch span `:307-316 / :318-327 / :329-338 / :340-349` | §3.1 note (line 332); R2-map C-N1/D-m1 row (line 878); R3-map E-N7 row (line 926) | **LANDED** — 3 live cites all `307-349`; only remaining `307-347` is the map's own fix notation (line 964); spans verified in source |
| G5 unitemized wording | F-F3 row extended to cover §1.1 re-verification credit (byte-exact, all 7 F4 rollouts) | R3-map F-F3 row (line 923) | **LANDED** — row now reads "…and §1.1's re-verification credit extended to round 3 (byte-exact, all 7 F4 rollouts — v4: itemized)" |
| H2 stale R2-map range | D-M2 row annotated: v2 `:1030-1038` corrected in v3 to `:1025-1028/:1030-1035`, verified exact in v4 | R2-map D-M2 row (line 877) | **LANDED** — annotation present; both ranges re-verified in source |

### G1/H1 evidence
- §1.2 Method line 135: "every rollout under
  `~/.codex/sessions/2026/09/{13,14,15}/` (on-disk layout is `YYYY/MM/DD`, v4)"
  — the exact on-disk layout (I listed `~/.codex/sessions/2026/09/`; `13/`,
  `14/`, `15/` day dirs exist, dash form does not).
- R3-map E-N3 row now states §1.1/header were fixed to
  `~/.codex/sessions/2026/09/{13,14,15}/` in v3 and that "§1.2's Method line
  still carried the non-existent dash form in v3 (v2's was `2026-09-{13,14,15}`)
  and was corrected in v4". I verified the historical forms: v2 archive line 127
  reads `` `~/.codex/sessions/2026-09-{13,14,15}` `` and v3 archive line 134
  reads `` `~/.codex/sessions/2026-09/{13,14,15}/` `` — both as quoted.
- Round-3 seat-E summary N3 (line ~899) now reads "vs the actual
  `2026/09/{13,14,15}/` layout" (was `2026-09/…` in v3).

### G2 evidence (special attention)
Re-derived event-by-event from the snapshot (full table in Mandate 2):
- glm row at ≤ **2026-09-15T23:34:04.678Z** (inclusive): **37 = 26 PARSE +
  11 OK + 0 VERIFY** — the v4-stated glm pin reproduces exactly.
- glm row at ≤ **2026-09-15T23:31:03.433Z** (the round-4-suggested pin):
  **36 = 26 PARSE + 10 OK** — v4's G2 note claim ("at that instant the row is
  26 PARSE + 10 OK = 36") holds exactly.
- The 11th OK is `call_d6cac804ac454340a0551353` @ **2026-09-15T23:34:04.678Z**
  (output: `Success. Updated the following files: …`) — matches the note.
- Attribution of the suggested pin to the round-4 report verified:
  `docs/reviews/apply-patch-format-spec-r4-seatG.md` §G2 "Suggested
  resolution" proposes "glm row at the last glm event 23:31:03.433Z (19:31:03
  EDT)" — i.e. the v4 note's "the round-4 report's suggested 23:31:03.433Z glm
  pin" is correctly attributed, and that suggestion was indeed one event short
  (seat G's own recount missed `call_d6cac804…`; v4's note says precisely this).
  (Separately noted: seat G's impact text says applying the qwen pin to both
  models yields "glm 24/35"; my derivation gives 24 PARSE / 34 total at
  23:30:11.662Z. That figure lives only in the r4 seat-G report, not in v4 —
  v4 never repeats 24/35 — so it is not a v4 finding.)
- The 83/162/86 triple now appears only in the Freshness bullet (with the
  reproducible window) and in the annotated R3-map E-N1 row; the pin-instant
  sentence carries 83/162/72, which I reproduced (Mandate 2).
- Stability sentence: "no glm data after its row pin (09-15 19:34:04 EDT) as of
  the round-3 freshness snapshot (next glm event: 2026-09-16T00:39:58Z)" — the
  first glm event after the glm pin is `call_95179bbe…` @ 2026-09-16T00:39:58.042Z
  (Mandate 2); 19:34:04 EDT = 23:34:04Z ✓.

### G3 evidence
§1.2 definition now reads: VERIFY = "post-parse filesystem verification
failures only (`Failed to find expected lines`, `Failed to read file`,
`Failed to write file` — the qwen row's 30 VERIFY = 27 find-expected + 2
read-file + 1 write-file; the sole write-file failure is `call_be0ac666…`,
09-14 08:22:44Z)". Re-derived at the qwen pin: 27 + 2 + 1 = 30 exactly, and the
write-file call is `call_be0ac66658ae41ffa61898d2` @ 2026-09-14T08:22:44.003Z.

### G4 evidence
- `grep 307-349` v4: exactly the three live cites (lines 332, 878, 926); the
  only `307-347` left is the R4-map G4 row's own "old → new" notation (line 964).
- Source `codex-rs/apply-patch/src/streaming_parser.rs`: :307-316 =
  empty-line branch, :318-327 = ` ` context branch, :329-338 = `+` branch,
  :340-349 = `-` branch — exactly the four auto-chunk spans claimed; :349 is
  the closing brace of the last branch (:351 begins the error branch).

### H2 evidence
Source `codex-rs/core/tests/common/responses.rs`: `ev_function_call` at
:933 (body through :942, matching "generic constructor at :933-942");
`ev_exec_command_call_with_args` at :1025-1028;
`ev_apply_patch_exec_command_call_via_heredoc` at :1030-1035 — exactly the
annotated v3 ranges. The D-M2 row carries "— v3: corrected to
:1025-1028/:1030-1035, verified exact in v4" as stated.

## Mandate 1 — v3→v4 diff audit

`diff -u docs/reviews/apply-patch-format-spec-v3-draft.md
docs/responses-compat-apply-patch-format.md` → **9 hunks, 110 changed
(+/-) lines, 0 hunks untraceable**.

| Hunk (v3 start → v4 start) | Content | Traces to |
|---|---|---|
| @@ -2 → +2 | Evidence line gains v2/v3 archive refs; Status line v3→v4, "round 4"→"round 5" | §7 bookkeeping (status/archive lines) |
| @@ -26 → +26 | TL;DR parenthetical: "qwen count as of the pinned snapshot instant … a fresh 2026-09-16 recount" → "each row as of its own pinned instant … the round-3 freshness snapshot gives qwen 439 calls, 6 PARSE ≈ 1.4%" | G2 (two-pin wording) |
| @@ -131 → +131 | §1.2 Method line: path fix + `(on-disk layout is YYYY/MM/DD, v4)`; VERIFY exemplars + 27/2/1 decomposition; Snapshot (v3) single-pin paragraph → Snapshot (v4) two-pin paragraph | G1/H1 (path), G3 (VERIFY), G2 (pins) |
| @@ -170 → +176 | Recount-history seat-D bullet: "no glm data after 09-15 19:31 EDT, so snapshot-stable" → "reproduced … by round-3 seats E and F and v4 (glm row pin restated: 2026-09-15T23:34:04.678Z); … stable — no glm data after its row pin … as of the round-3 freshness snapshot (next glm event: 2026-09-16T00:39:58Z)" | G2 (pin restatement + stability qualification) |
| @@ -180 → +188 | Freshness (v3) "00:22Z … qwen 439 … glm row is unchanged" → Freshness (v4): reproducible window 00:23:58.623Z ≤ t < 00:27:48.898Z, 83/162/86 triple, 6 PARSE ≈1.4%, "unchanged at that copy (next glm event 00:39:58Z)", v3-label-nominal note, four post-snapshot glm PARSE calls enumerated, live row 30/41 ≈ 73% | G2 (triple move + qualification + supporting post-snapshot data) |
| @@ -313 → +329 | §3.1 design note: `streaming_parser.rs:307-347` → `307-349` | G4 |
| @@ -858 → +874 | R2-map D-M2 row gains "(— v3: corrected to :1025-1028/:1030-1035, verified exact in v4)"; R2-map C-N1/D-m1 row `307-347` → `307-349 … v4: tail :347 → :349` | H2; G4 |
| @@ -881 → +897 | Round-3 seat-E summary N3: "the actual `2026-09/{13,14,15}/` layout" → "the actual `2026/09/{13,14,15}/` layout" | G1/H1 (seat-E summary restated) |
| @@ -899 → +915 | R3-map: E-N1 row annotated (two pins; 86-triple attribution; stability qualification); E-N3 row restated (slash form actual; §1.2 dash form corrected in v4; YYYY/MM/DD); F-F3 row extended (re-verification credit, byte-exact, all 7 F4); E-N7 row `307-347` → `307-349 (v4: tail)`; "Round 4: pending" → full round-4 seat summaries + Round-4 resolution map + "Round 5: pending" | G2 (E-N1), G1/H1 (E-N3), G5 (F-F3), G4 (E-N7), §7 bookkeeping (round record) |

### Structural preservation (byte-identity checks)

| Section (v3 → v4) | Result |
|---|---|
| §3.1 P1 code block (tool-level description + `patch` argument description, both fenced blocks) | **byte-identical** (2,407 bytes each) |
| §3.2 (P2, entire section incl. edge list) | **byte-identical** (5,883 bytes each) |
| §3.4 (non-goals, decisions, exact invariant) | **byte-identical** (3,873 bytes each) |
| §3.1 as a whole | differs in exactly one line: the G4 cite `307-347` → `307-349` |

No v3-correct content is lost or contradicted: every deleted line is either
(a) replaced by its R4-mandated correction in the same location (path, pins,
stability, VERIFY exemplars, 307-349), or (b) a superseded bookkeeping line
(status, archive list, "Round 4: pending"). The one historical statement the
diff *retains* despite being superseded — the R3-map E-N1 row's opening
"Snapshot pinned (2026-09-15T23:30:11Z, 83/162/86 rollout files)" — carries an
in-line `(v4: restated — …)` correction in the same row, which is exactly what
the R4 map's G2 row promised ("the R3-map E-N1 row annotated"); see finding
I-N2 for the residual readability nit.

## Mandate 2 — Pinned-row re-derivation (fresh snapshot)

Snapshot: `/tmp/r5seatI_snap_20260916T022538Z` @ **2026-09-16T02:25:38Z**
(83/162/109 files). Whole-snapshot totals: 525 unique apply_patch call_ids
(no cross-file duplicates; every call has its output) = 484 qwen3.8-27b +
41 glm-5.2. Result per load-bearing claim:

| # | Spec claim (location) | Re-derived at stated pin/window | Match |
|---|---|---|---|
| 1 | qwen pin = 2026-09-15T23:30:11.662Z, the instant the cumulative qwen count first reached 413 (§1.2 Snapshot) | qwen call #413 in ts order = `call_8a6dbaa5efe149e58e2eec35` @ **2026-09-15T23:30:11.662Z** exactly; #412 @ 23:29:53.080Z; #414 @ 23:46:42.979Z; no tie at the pin ts | **EXACT** |
| 2 | qwen row: 413 = 374 OK + 5 PARSE + 30 VERIFY + 1 MISSING + 1 UNSUPPORTED + 2 OTHER (§1.2 table) | 413 = 374 + 5 + 30 + 1 + 1 + 2, each bucket re-derived (identities below) | **EXACT** |
| 3 | File triple 83/162/72 at the qwen pin (first-line ts ≤ pin) | (83, 162, 72) | **EXACT** |
| 4 | glm pin = 2026-09-15T23:34:04.678Z = last included event `call_d6cac804…`, an OK call (§1.2 Snapshot) | glm call #37 = `call_d6cac804ac454340a0551353` @ **2026-09-15T23:34:04.678Z** (output `Success. Updated…`); single call at that ts | **EXACT** |
| 5 | glm row: 37 = 26 PARSE + 11 OK + 0 VERIFY (§1.2 table) | 37 = 26 + 11 + 0 | **EXACT** |
| 6 | (G2 note) at 2026-09-15T23:31:03.433Z the glm row is 26 PARSE + 10 OK = 36 | 36 = 26 PARSE + 10 OK | **EXACT** |
| 7 | glm's final two PARSE events `call_1bcbc827…` @ 23:30:46.881Z, `call_1c41b322…` @ 23:31:03.433Z, both after the qwen pin | `call_1bcbc8272d3a4ff180fec781` @ 2026-09-15T23:30:46.881Z; `call_1c41b322a0e1414c9a3d8c53` @ 2026-09-15T23:31:03.433Z; both > 23:30:11.662Z | **EXACT** |
| 8 | glm PARSE breakdown 26 = 17 F1 (10 `#` + 3 `VERDICT:` + 3 `@@` + 1 `---`) + 2 F2 + 5 missing Begin + 2 missing End (§1.2) | 17 + 2 + 5 + 2 = 26; sub-counts 10/3/3/1 exact; for all 17 F1-class calls the error's quoted line == the first content line after the first `*** Add File:` header (17/17, all "invalid hunk at line 3") | **EXACT** |
| 9 | 30 qwen VERIFY = 27 find-expected + 2 read-file + 1 write-file; write-file = `call_be0ac666…`, 09-14 08:22:44Z (§1.2 definition) | 27/2/1; write-file = `call_be0ac66658ae41ffa61898d2` @ 2026-09-14T08:22:44.003Z | **EXACT** |
| 10 | Freshness window: 439th qwen call = 2026-09-16T00:23:58.623Z (`call_ade5c987bda…`), 440th = 2026-09-16T00:27:48.898Z (§1.2 Freshness) | #439 = `call_ade5c987bdaa4980bd5a03f2` @ **2026-09-16T00:23:58.623Z** exactly; #440 = `call_8be015c65c6a4e0d9d356b92` @ **2026-09-16T00:27:48.898Z** exactly | **EXACT** |
| 11 | Copy in that window: 83/162/86 files; qwen 439 calls, 6 PARSE (≈1.4%); glm unchanged; next glm event 2026-09-16T00:39:58Z | triple (83,162,86) at both window boundaries (file #86 first-line ts 00:22:54.283Z < window lo; file #87 first-line ts 00:30:50.143Z > window hi; no 09-15 file starts inside the window); qwen ≤ any in-window instant = 439 calls, 6 PARSE (6/439 = 1.367%); glm = 37 (26+11); first glm event after pin = `call_95179bbe…` @ 00:39:58.042Z | **EXACT** |
| 12 | Post-snapshot: exactly four further glm PARSE calls — `call_95179bbe…` 00:39:58Z, `call_35381ef9…` 00:40:02Z, `call_2e757768…` 01:40:35Z, `call_0d06d3f7…` 01:44:55Z (2026-09-16 UTC), through 09-15 21:44:55 EDT | exactly four, all PARSE: @ 00:39:58.042Z / 00:40:02.307Z / 01:40:35.075Z / 01:44:55.879Z; call-id prefixes match; 01:44:55Z = 09-15 21:44:55 EDT (UTC−4); no glm event after 01:44:55.879Z through my snapshot → live glm row 30/41 (30/41 = 73.2%) | **EXACT** |
| 13 | "13 of 21 glm calls rejected on 09-15 alone (~62%)", day attribution by file directory (§1.2) | glm calls in day-15 files at the glm pin: 21, of which 13 PARSE (13/21 = 61.9%) | **EXACT** |
| 14 | "Zero glm call in the three days failed with a filesystem verification error" (§1.2) | glm VERIFY = 0 at the pin and whole-snapshot | **EXACT** |

### qwen non-OK call identities at the pin (all five PARSE verified parse-path)
- PARSE: `call_b735206c5356444fabb9421f` @ 09-14T04:03:56.846Z (`'```' is not a
  valid hunk header`, line 216); `call_4c2fba7afc3c433d9cbd07e5` @
  09-14T06:54:21.668Z (`Update hunk does not contain any lines`, line 109);
  `call_e8bedd24848f42abb7a7dd2d` @ 09-15T19:12:24.870Z (`'@@' …`, line 3);
  `call_7b1d28aa04b54b278ff61a69` @ 09-15T20:17:13.863Z (`'set -u' …`, line 49);
  `call_08bc613797ae4f4f9b17fce3` @ 09-15T21:19:58.084Z (`'# apex-ayl.28 —
  step-machine desync …'`, line 500).
- OTHER ×2: `call_306909242611434cbdd57001` @ 2026-09-15T20:20:08.933Z and
  `call_096536c41e0e415ab2afe730` @ 21:25:21.392Z — both
  `invalid patch: multiple operations target …` (spec names both prefixes and
  both second-resolution timestamps; match).
- MISSING: `call_26324b8d66f14373a4258ff1` @ 09-15T22:13:05.749Z, args exactly
  `{}`, output `apply_patch is missing the required 'patch' argument`; next
  call in the same session (`call_a687a588…`, 22:14:51.543Z) is canonical and
  succeeded (`A …/plans-census-20260915.md`) — §1.1 F3 "self-corrected" holds.
- UNSUPPORTED: `call_3c03fe7957cb4be59119a793` @ 09-13T16:54:27.298Z, args in
  legacy `{"cmd": "[\"apply_patch\", …]"}` shape, output `unsupported call:
  apply_patch`.
- VERIFY read-file ×2: `call_5a4468f05ebb40db98dd603a` @ 09-14T12:09:40.296Z;
  `call_21cd85e372e64e8e9a65b157` @ 09-15T21:23:28.361Z.

### §1.1/§1.3 content claims re-derived from the same snapshot
- §1.1 F1 verbatim example: `call_caf7ba097fbe46f282cd26fc` @
  2026-09-15T22:57:48.900Z patch begins exactly
  `*** Begin Patch\n*** Add File: grok/plans/spec-freeze-r23-glm.md\n#
  SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)\n…` and ends
  `*** End Patch`; the error output matches the spec's quoted F1 error
  byte-for-byte (whitespace-normalized). **EXACT**
- §1.3 "24,230 chars (the `patch` string itself 23,972)": same call —
  len(arguments JSON) = **24,230**, len(patch) = **23,972**. **EXACT**
- §1.1 F2: `call_d694eb148672418d834a6184` output matches the spec's quoted
  error byte-for-byte; the same session's retry `call_8d7a04fb51344aa4a4cc7406`
  (20:01:47.746Z) targets the same file (`spec-audit-concord.md`) with the
  `## PART B concordance (L2838-5833, sections 5-10)` section carried as `-`-
  prefixed lines and succeeded. "Self-correction-on-retry evidenced, not
  assumed" holds. **EXACT**
- §1.1 F4 rollout examples: `rollout-2026-09-14T03-25-55-*` patch is exactly
  `*** Add File: …/w1-review-glm.md\n*** End Patch` (no Begin, no content);
  `rollout-2026-09-14T07-58-02-*` patch's last line is `+*** End Patch` (never
  bare); `rollout-2026-09-15T14-33-27-*` first line is `*** Add File:` (Begin
  missing). All three files exist in the snapshot with the described patches.
  **EXACT** (see finding I-M1 on the sibling "one patch" label in §1.2).
- §1.2 base-instructions claim: every glm- and qwen-seat file carries the
  same `base_instructions.text` — **20,751 chars, byte-identical, SHA-256
  prefix `ac8ae107a0d7`** (full prefix `ac8ae107a0d72fe3`). **EXACT**
- §3.1/R2-map C-N4 P1 length: tool description 126 chars/21 words + argument
  description 2,198 chars/380 words = **2,324 chars / 401 words** total.
  **EXACT**

### Post-instant live deltas (disclosed, not findings)
At my snapshot (02:25:38Z) beyond the pinned states: qwen 484 total (45 calls
after the freshness window, incl. 6 further PARSE and 2
`Failed to find context` calls at 23:56:08.699Z / 23:56:49.864Z — see I-N3);
glm 41 total (the four post-snapshot PARSE calls, all inside day-15-directory
files — sessions started 09-15, calls timestamped 09-16 UTC); 109 09-15 files
(23 started after the freshness window). None of these enters a pinned row.

## Mandate 3 — Arithmetic & consistency sweep

**Row sums** (all re-derived counts):
- glm: 26 PARSE + 11 OK + 0 VERIFY + 0 other = 37 ✓ (TL;DR and §1.2 tables
  agree; §2 "26 of 37" agrees).
- qwen: 374 + 5 + 30 + 1 + 1 + 2 = 413 ✓ (TL;DR and §1.2 tables agree; the
  inline "413 = 374 + 5 + 30 + 1 + 1 + 2" matches).
- VERIFY sub-sum: 27 + 2 + 1 = 30 ✓. F4 sub-sum: 5 + 2 = 7 ✓. F1+F2+F4:
  17 + 2 + 7 = 26 ✓; F1 sub-sum 10 + 3 + 3 + 1 = 17 ✓. TL;DR class split
  "19 of 26" (F1/F2) + "7 of 26" (F4) = 26 ✓.
- R2-map D-M1 row: 19 content-class + 7 F4 = 26, + 11 OK = 37 ✓. R3-map E-N2
  row: 17 + 2 + 7 = 26 ✓. v1-era bullet: 19 + 10 + 7 = 36 ✓ (historical,
  internally consistent). v2 table (archive line 140/162): 374 + 5 + 31 + 1 +
  1 + 1 = 413 ✓ — v4's "the v2 table's 31 VERIFY / 1 OTHER" characterization
  is accurate.

**Rate math:**
- 26/37 = 70.27% → "~70%" ✓ (TL;DR, §1.2, §6, R2/R3/R4 maps).
- 5/413 = 1.21% → "~1%" ✓ (TL;DR, §1.2, §6; §3.4 "Frequency 1/413" for the
  single MISSING ✓).
- 13/21 = 61.9% → "~62%" ✓ (§1.2, §6).
- 6/439 = 1.367% → "≈1.4%" ✓ (TL;DR, §1.2 Freshness).
- 30/41 = 73.17% → "≈73%" ✓ (§1.2 Freshness live row).
- "24 KB patches" (TL;DR) vs 24,230 chars = 23.7 KB ✓.
- One nit: "an order of magnitude above qwen's ~1%" (§1.2 Conclusion) — the
  actual ratio is 70.27/1.21 ≈ 58× (log10 ≈ 1.76), i.e. between one and two
  orders of magnitude, closer to two; see I-N1.

**Cross-references:**
- TL;DR table ↔ §1.2 table ↔ §1.2 prose ↔ §2 ↔ §6: all agree (37/26, 413/5,
  pins, freshness 439/6/≈1.4%, 13/21). ✓
- TL;DR "each row as of its own pinned instant in §1.2" ↔ §1.2's two explicit
  pins ✓. §6 "as of the §1.2 pinned snapshot" and §1.2 Conclusion "as of the
  pinned snapshot" are generic phrasings of the pinned state; no sentence in
  v4 implies a single shared pin instant (I grepped every "pin" usage; the
  qwen-row bullets say "the pinned instant" only in the qwen-row context).
  The sole vestige of the old single-pin sentence is the R3-map E-N1 row's
  opening, which is annotated in-line (I-N2).
- §1.1 F4 "7 of 26 PARSE (5 Begin, 2 End)" ↔ §1.2 "5 missing `*** Begin
  Patch` … 2 missing `*** End Patch`" ✓. §1.1 "(v1 mis-bucketed these seven
  glm rejections as 'VERIFY')" ↔ §1.2 recount history ("v1's 7 glm 'VERIFY'
  bucket contained no filesystem verification failures") ↔ R2-map D-M1 ✓.
- Recount-history vs the cited reports: r1-seatA (its line 69–70) reports
  "glm-5.2: 36 calls → PARSE 18 … qwen3.8-27b: 405 calls → PARSE 4" — §1.2's
  "glm 18/36, qwen 4/405" and the "(±8 qwen calls …; ±1 glm PARSE …)"
  parenthetical are the v1-vs-seat-A deltas (413−405 = 8; 19−18 = 1),
  consistent with the R1 map row "±1 vs seat A's recount disclosed". ✓
- Timezone conversions: 23:34:04.678Z = 19:34:04 EDT ✓; 01:44:55Z = 09-15
  21:44:55 EDT ✓ (UTC−4 in September, per the brief's own conversion).

**Design-content check (not re-litigated, only consistency):** P1/P2/P3/P3.4
and the §3.4 invariant are byte-identical v3→v4 (Mandate 1), so the settled
design is exactly what prior rounds approved; v4 touched only
evidence/refs/bookkeeping, as the R4 map states.

## Findings

### I-M1 [Minor] (EVIDENCE label) — §1.2 F4 parenthetical: "one patch" is two patches
- **Where:** §1.2, glm PARSE breakdown line (line 162): "… 5 missing
  `*** Begin Patch` (F4), 2 missing `*** End Patch` (F4; one patch ends with
  the `+`-prefixed terminator)."
- **Evidence re-derived at source:** at the glm pin, the two missing-End
  rejections are `call_10f109a345a4439094df5930` (2026-09-14T12:03:39.612Z,
  file `rollout-2026-09-14T07-58-02-01a09fc8-…`) and
  `call_8ee1b0c158f849c28f6f9da4` (2026-09-15T03:17:57.239Z, file
  `rollout-2026-09-14T23-11-53-01a0a30c-…`). Both patches' final line is
  exactly `+*** End Patch` — i.e. **both** of the two end with the
  `+`-prefixed terminator, not one.
- **Impact:** none on any count or pin (both are F4 boundary rejections; the
  5/2 split and the 26 total are unchanged). A re-counter walking the 2
  missing-End patches with the spec's method would find 2-of-2 and flag the
  label — hence Minor, not Major.
- **Suggested resolution:** change "one patch ends with the
  `+`-prefixed terminator" to "both patches end with the `+`-prefixed
  terminator (e.g. `+*** End Patch`, never a bare terminator)".

### I-N1 [Nit] (wording) — "an order of magnitude above" is imprecise
- **Where:** §1.2 Conclusion (line 203): "… an order of magnitude above
  qwen's ~1%."
- **Evidence:** 26/37 = 70.27% vs 5/413 = 1.21% → ratio ≈ 58× (log10 ≈ 1.76).
- **Impact:** cosmetic; the direction and the pinned rates themselves are
  correct.
- **Suggested resolution:** "≈60× qwen's ~1%" or "roughly two orders of
  magnitude".

### I-N2 [Nit] (bookkeeping readability) — R3-map E-N1 row retains the superseded opening
- **Where:** R3-map E-N1 row (line 918): the row still opens with v3's exact
  words "Snapshot pinned (2026-09-15T23:30:11Z, 83/162/86 rollout files)",
  which pairs the qwen pin instant with the freshness-copy triple.
- **Evidence:** the same row carries "(v4: restated — two explicit pins, …;
  the file triple 83/162/86 belongs to the snapshot copy, the pin-instant
  triple is 83/162/72; stability qualified to the snapshot copy — see the
  Round-4 map, G2 row)", and the R4 map's G2 row states the intended fix was
  exactly "the R3-map E-N1 row annotated". So the row is self-correcting and
  the R4 fix landed as recorded.
- **Impact:** none — a skim that stops before the annotation would misread,
  but the correction sits mid-row.
- **Suggested resolution (optional, future round):** restate the opening as
  "Snapshot pinned (qwen 2026-09-15T23:30:11Z; 83/162/72 rollout files at
  that instant)" so the pre-annotation text is not itself the wrong pairing.

### I-N3 [Nit] (method completeness) — VERIFY exemplar list omits a 4th live post-parse variant
- **Where:** §1.2 strict definition: VERIFY = "post-parse filesystem
  verification failures only (`Failed to find expected lines`, `Failed to
  read file`, `Failed to write file` …)".
- **Evidence:** the live data contains a fourth post-parse verification
  failure message, `apply_patch verification failed: Failed to find context
  '<line>' in <file>` — twice, both qwen, both after every pin/window
  (`call_a9cf9f970e8b4405af8c5bbf` @ 2026-09-15T23:56:08.699Z and
  `call_69f201bef73d4b7fba575661` @ 23:56:49.864Z). No pinned row contains
  either call, so no pinned claim is affected (at the qwen pin the 30 VERIFY
  are exactly the three listed variants).
- **Impact:** a re-counter doing a full live recount (rather than a
  pinned-window one) must classify these two; the definition's lead-in
  ("post-parse filesystem verification failures only") covers them, but the
  exemplar list alone does not.
- **Suggested resolution (optional):** append `Failed to find context …` to
  the exemplar list.

## Re-verified clean

Beyond the per-claim tables above, these checks returned clean:

1. **All 14 load-bearing pinned claims** reproduce exactly at their stated
   pins/windows (Mandate 2, table) — including the qwen #413 instant, the
   83/162/72 pin-instant triple, the glm 26/37 row, the 36-row at the
   round-4-suggested instant, the F1 17/17 quoted-line↔first-content-line
   cross-check, the 83/162/86 freshness-window triple and its no-file-inside
   the window property, the four post-snapshot glm PARSE calls, and 13-of-21
   day attribution.
2. **All six R4-map rows landed** at their stated locations (Mandate 0),
   with the code-carrying claims (G4 spans, H2 ranges) re-verified in source.
3. **All nine v3→v4 hunks trace** to an R4-map row or §7 bookkeeping; no
   v3-correct content lost or contradicted; P1 code block / §3.2 / §3.4
   byte-identical (Mandate 1).
4. **§1.1 error quotes** (F1, F2, F3, both F4) match rollout output
   byte-for-byte; §1.1 F1 "verbatim" patch, §1.3 24,230/23,972 lengths, §1.1
   F2 same-session `-`-prefixed retry success, §1.1 F3 same-session canonical
   retry success, §1.1 F4's three named rollout files, and the 20,751-char
   byte-identical base instructions (SHA prefix `ac8ae107a0d7`) all verified
   at source.
5. **No timestamp ties** at either pin; no cross-file duplicate call_ids;
   every call has its output; model attribution unambiguous (turn_context
   first-payload rule, no fallback needed).
6. **Arithmetic** (row sums, sub-sums, all five rate figures, P1 2,324 chars
   / 401 words, TZ conversions, cross-section references) consistent
   throughout (Mandate 3); only the I-N1 wording imprecision.
7. **Two-pin wording**: no sentence in v4 implies a single shared pin
   instant; the one retained v3 pairing (R3-map E-N1 opening) is annotated
   in-line — I-N2.

## Counts

| Severity | Count | IDs |
|---|---|---|
| Blocking | 0 | — |
| Major | 0 | — |
| Minor | 1 | I-M1 |
| Nit | 3 | I-N1, I-N2, I-N3 |

**Verdict: APPROVED — 0 Blocking, 0 Major, 1 Minor, 3 Nit.** The v4 spec's
evidence, references, and bookkeeping are reproducible at their stated
pins/windows on an independent fresh snapshot, the round-4 fixes all landed
as recorded, and the v3→v4 delta is fully traceable. The single Minor (I-M1)
is a one-word data-label undercount in an F4 parenthetical; it does not
change any row, pin, rate, or design decision, and can be fixed in a future
bookkeeping pass without re-review of the settled design.

— seat I, round 5 (leaf reviewer; no subagents; only this report file
written). Snapshot of record: `/tmp/r5seatI_snap_20260916T022538Z`, taken
2026-09-16T02:25:38Z UTC.
