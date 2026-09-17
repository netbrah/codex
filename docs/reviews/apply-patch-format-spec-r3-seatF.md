# Review — apply-patch function-tool format spec (v2), Round 3, Seat F

Target: `docs/responses-compat-apply-patch-format.md` (v2, 846 lines), branch
`feat/normalize-content-types-vllm`
Reviewer: seat F (independent leaf reviewer. Mandated scope: (1) re-derive the
§1.2 evidence table from the rollouts, (2) re-probe the parser claims with the
unmodified debug binary, (3) verify test-plan references and red-state
completeness, (4) consistency checks. Settled design decisions (no existence
check, no Update-File leniency, …) are not re-litigated; the facts v2 relies
on were re-verified.) Date: 2026-09-15, ~20:20–21:40 EDT (2026-09-16T00:20–01:40Z).

## Verdict

**CHANGES-REQUESTED — 0 Blocking, 2 Major, 3 Minor, 2 Nit.**

The design (P1/P2/P3/P3.4) is implementable as written: every parser claim
re-probed passes on the unmodified binary, the glm evidence row reproduces
exactly, and every code line-reference I opened verifies. The two Majors are
both in the §1.2 evidence narrative: (F1) the qwen row is a point-in-time
tally of a still-growing live directory and no longer reproduces (439 in a
fresh strict recount), and (F2) the §1.2 glm PARSE breakdown miscounts the
F1-class (13 should be 17; the breakdown sums to 22, not 26).

Relationship to the same-round sibling report
(`docs/reviews/apply-patch-format-spec-r3-seatE.md`): F1/F2/F4/F5/F6
independently corroborate seat E's N1/N2/N5/N4/N6 — every one was
re-derived from the raw rollouts and source in this session, not read from E.
F1 additionally pins the exact drift mechanism (the spec's 413 is the
timestamp-ordered prefix of qwen calls through 2026-09-15T23:30:11Z), F2 adds
the 17 F1-class call_ids, and F3/F7 are seat-F-only.

Method (everything re-verified at the source this session):
- Snapshot of `~/.codex/sessions/2026/09/{13,14,15}` (83/162/86 files) copied
  to `/tmp/seatf/snap` at ~00:22Z; every `apply_patch` call reclassified from
  the raw rollouts (script `/tmp/seatf/final.py`, full 476-row classification
  in `/tmp/seatf/calls2.tsv`); the recount was re-run fresh in this session.
- Parser probes against the unmodified `codex-rs/target/debug/apply_patch`
  (the `apply-patch` crate is `git status`-clean; the branch makes no changes
  to that crate).
- Every cited file/line opened at HEAD of `feat/normalize-content-types-vllm`;
  vLLM issue states re-checked on GitHub.

## Findings

### F1 [Major] (EVIDENCE) — the qwen row (413) is a point-in-time tally of a still-growing live directory; a fresh strict recount gives 439

**Where:** §1.2 recount-history bullet 4 — "Under this method the
qwen3.8-27b table **reproduces exactly** (413 = 374 + 5 + 31 + 1 + 1 + 1),
which validates the recount pipeline" — the qwen table row ("MISSING 1,
UNSUPPORTED 1, OTHER 1"), the TL;DR rate (~1%), R2 row D-M1 ("qwen 413 exact
reproduction named as method validation"), and §6 ("qwen 5/413 ≈ 1%").

**Evidence:**
- Fresh strict recount of the 00:22Z snapshot, implementing §1.2's documented
  method (first `turn_context` model per file, unique `call_id`, outcome from
  the matching `function_call_output`): **476 unique calls = glm 37
  (26 PARSE + 11 OK + 0 VERIFY — exact) + qwen 439**. In the spec's own
  bucketing (the two `invalid patch: multiple operations target …`
  rejections counted with the `verification failed` prefix family):
  439 = 396 OK + 6 PARSE + 34 VERIFY + 1 OTHER + 1 MISSING + 1 UNSUPPORTED.
- The spec's 413 is the **first 413 qwen calls in timestamp order** — a
  cutoff at **2026-09-15T23:30:11.662Z**: at that point all six bucket totals
  match the spec row (374/5/31/1/1/1). The method pins no snapshot time and
  the sessions directory is live.
- The +26 delta is exactly the trailing tail after that cutoff
  (2026-09-15T23:46:42Z → 2026-09-16T00:23:58Z): 22 OK + 1 PARSE
  (`call_b5eef8ab…`, "invalid hunk at line 52, '---'…") + 3 VERIFY
  (`call_a9cf9f97…`, `call_69f201be…`, `call_5187758d…`).
- The directory is still growing at review time: the snapshot had 86 day-15
  files; the live dir has 97, and its newest rollout line is timestamped
  within seconds of now (01:36Z).
- Corroborated by seat E (N1), whose independent recount gives 434 at the v2
  write instant (00:18:42Z) and — at the 413 point — the strict-definition
  split 30 VERIFY / 2 OTHER, not the spec's 31/1: the two
  `multiple operations target` rejections (`call_30690924…` 20:20:08Z,
  `call_096536c4…` 21:25:21Z) are OTHER under the spec's own definitions
  (VERIFY = post-parse filesystem failures only; OTHER = other rejections).

**Impact:** The one sentence the spec uses to validate its recount pipeline
no longer reproduces: a reader re-running the documented method today gets
439 (and more by the time they finish — the dir is still growing), not 413;
and even at the 413 point the row's 31 VERIFY/1 OTHER split contradicts the
method's own definitions (30/2 strict). The ~1% rate the TL;DR leans on is
still directionally right (now 6/439 ≈ 1.4%), but the "reproduces exactly /
validates the recount pipeline" claim is false without a pinned snapshot.

**Suggested resolution:** Pin a snapshot in the §1.2 method (e.g. "as of
2026-09-15T23:30:11Z; 83/162/86 files") and re-state the row as
"reproduces exactly **as of that pinned instant**"; re-tabulate at a fresh
pinned snapshot if the numbers are to stay current (439 as of 00:22Z). Fix
the 31 VERIFY/1 OTHER split to 30/2 under the strict definitions, or state
the coarser bucketing explicitly.

### F2 [Major] (EVIDENCE) — §1.2 glm PARSE breakdown miscounts the F1-class: 13 should be 17 (sum 22 ≠ 26)

**Where:** §1.2: "glm PARSE breakdown (26): **13** F1-class (raw Add-File
content, including `@@`/`---`-looking first content lines), 2 F2-class (raw
Update content), 5 missing `*** Begin Patch` (F4), 2 missing `*** End Patch`
(F4 …)."

**Evidence:** 13 + 2 + 5 + 2 = 22 ≠ 26, the number the breakdown is
introduced as decomposing. My event-level recount of all 26 glm PARSE
rejections (ids below; all in `/tmp/seatf/calls2.tsv`) gives **17
F1-class** ("invalid hunk at line 3/4, … is not a valid hunk header": 10 raw
`#`-first-line + 3 `VERDICT: …` + 3 `@@` + 1 `---`) + 2 F2-class
("Unexpected line found in update hunk") + 5 missing Begin + 2 missing End
= 26. The three TSV messages truncated at 120 chars were re-checked against
the raw snapshot output — all end "…is not a valid hunk header".
- F1-class (17): `call_f71e73a1…`, `call_bb84f634…`, `call_9e549b1d…`,
  `call_140c8387…`, `call_9d8f6aad…`, `call_c8da43ef…`, `call_01de78b4…`,
  `call_50a59b34…`, `call_3e7c2d5d…`, `call_aeabdca0…`, `call_c8cfde6a…`,
  `call_20e5a450…`, `call_a00bebb7…`, `call_63a6921e…`, `call_caf7ba09…`,
  `call_c4531ddc…`, `call_1c41b322…`.
- F2-class (2): `call_d694eb14…`, `call_cda093b5…`.
- Missing Begin (5): `call_17d7edbb…`, `call_6d38fade…`, `call_6f673754…`,
  `call_bf4d74bc…`, `call_1bcbc827…`. Missing End (2): `call_10f109a3…`,
  `call_8ee1b0c1…`.
- The TL;DR "19 of 26" (F1/F2) and R2 row D-M1 "19 content-class + 7
  boundary-class" are both consistent with 17 (17 + 2 = 19) — only the §1.2
  breakdown's "13" is wrong.

**Impact:** A load-bearing evidence figure is arithmetically wrong and
self-contradictory with the spec's own TL;DR/R2-map 19+7 split. (The glm
26/37 ≈ 70% rate itself is exact — see Checked-and-clean.)

**Suggested resolution:** Correct "13 F1-class" → "17 F1-class"
(17 + 2 + 5 + 2 = 26). Corroborated by seat E (N2), which independently
counted 17 with the same sub-breakdown.

### F3 [Minor] (EVIDENCE/parser) — §1.1's F4 error quotes omit the `invalid patch: ` prefix present in the real harness output

**Where:** §1.1 F4 (section heading: "Failure modes observed (exact harness
errors)"), the two quoted messages.

**Evidence:** The spec quotes

    apply_patch verification failed: The first line of the patch must be '*** Begin Patch'

(or the End variant). The actual harness output — byte-verified from the raw
rollouts, e.g. `call_17d7edbb…` (09-14 07:34:44Z) and `call_10f109a3…`
(09-14 12:03:39Z) — is

    apply_patch verification failed: invalid patch: The first line of the patch must be '*** Begin Patch'

(and the End variant with the same prefix). The prefix comes from
`ParseError::InvalidPatchError`'s `#[error("invalid patch: {0}")]`
(`codex-rs/apply-patch/src/parser.rs:57`); the `apply_patch` CLI formats the
same error separately as `Invalid patch: {message}`
(`codex-rs/apply-patch/src/lib.rs:378-385`), which is what the probe captures.
- P3.4 itself is unaffected: its target strings at `parser.rs:268/:271` do
  not include the prefix, and P3.4 correctly appends to them. Only the §1.1
  rollout-output quotes are missing it.

**Impact:** §1.1's contract is "exact harness errors"; anyone copying the
quoted strings for expectations — or for the post-P3.4 harness output, which
will read `apply_patch verification failed: invalid patch: <new full text>` —
is off by the prefix.

**Suggested resolution:** Add `invalid patch: ` to both §1.1 F4 quotes, with
one clause distinguishing the parser-level string (what P3.4 edits) from the
harness-level output (what the model sees and the rollouts record).

### F4 [Minor] (TEST-PLAN) — red state undercounts `test_parse_patch_lenient`: 7 assertions go red, not 5

**Where:** §3.3 red-state paragraph: "`test_parse_patch_lenient`
(parser.rs:558 — **5** exact-string assertions across its Strict-mode and
missing-closing-heredoc cases)."

**Evidence:** `test_parse_patch_lenient` (parser.rs:558-647) contains **7**
boundary-string assertions: the Begin message (`expected_error`, :575-576)
is asserted at :579, :594, :609, :624 (Strict, the four heredoc variants),
:628 (Lenient, mismatched quotes), :635 (Strict, missing closing); the End
message at :639-642 (Lenient, missing closing). All 7 compare the exact
pre-P3.4 strings, so all 7 go red under P3.4. The two the spec omits:
Lenient mismatched-quotes (Begin) and Lenient missing-closing (End).
- `test_parse_patch` (parser.rs:277) has exactly 2 boundary assertions
  (:281/:287) — the spec's count there is correct; and the 3-assertion red
  list for `test_streaming_patch_parser_returns_errors`
  (streaming_parser.rs:814 → :828/:838/:848) is exact, with the :819
  NotStarted assertion staying green (its message is out of P3.4 scope).

**Impact:** The red-state enumeration is the contract for the implementer's
red log at T1. The spec currently implies 11 red assertions
(3 streaming + 1 CLI + 2 `test_parse_patch` + 5 lenient); the true count is
13 (3 + 1 + 2 + 7). Undercounting risks an incomplete red log being accepted
silently.

**Suggested resolution:** Change "5" → "7" and enumerate: 6 Begin-message
assertions (4 Strict heredoc variants + Lenient mismatched-quotes + Strict
missing-closing) + 1 End-message assertion (Lenient missing-closing).
Corroborated by seat E (N5, same 7).

### F5 [Minor] (CONSISTENCY) — §3.3's blanket "exact prefix substring" claim is false for P3.2 and P3.3 (2 of the 4 sites)

**Where:** §3.3 intro: "in each case the pre-existing text remains an exact
prefix substring of the new string."

**Evidence:**
- Site 2 (P3.2, DeleteFile): an explicit replacement — the old message
  (`streaming_parser.rs:222`: `'{trimmed}' is not a valid hunk header. Valid
  hunk headers: …`) shares no prefix with the new text (`'Delete File' hunks
  take no content lines; …`). The spec's own red-state list (the DeleteFile
  'bad' assertion goes red) confirms the old text is not preserved.
- Site 3 (P3.3, `apply_patch.rs:539`): the existing message wraps the
  argument name in backticks ("…is missing the required `patch` argument");
  the new text uses single quotes ("…is missing the required 'patch'
  argument; …") — the prefix diverges at the first quote character after
  "required ". The non-string variant's new message (`apply_patch 'patch'
  argument must be a string …`) shares no prefix with the old one at all.
- Sites 1 (P3.1, "keep the original sentence verbatim … append") and 4
  (P3.4, the `parser.rs:268/:271` sentences extended) do preserve the prefix.

**Impact:** Prefix status is the natural verification hook (predict which
substring assertions survive); the blanket claim misleads at exactly the two
replacement-semantics sites. No existing test is affected — a repo-wide
search finds no assertion on the P3.3 message, and the core-suite substring
at `apply_patch_cli.rs:707` targets a different message — so this is
wording, not a test-plan break.

**Suggested resolution:** Reword: "sites 1 and 4 extend the pre-existing
sentence (prefix preserved); site 2 is a replacement; site 3 re-quotes the
argument name (backticks → single quotes) and splits one collapsed error
into two." Corroborated by seat E (N4).

### F6 [Nit] (CONSISTENCY) — "2,324 chars / 380 words": 380 is the argument-description-only count; the combined text is 401 words

**Where:** §3.1 notes: "…tool description 126 chars + `patch` argument
description 2,198 chars = 2,324 chars / 380 words ≈ 550–615 tokens…".

**Evidence:** 2,324 chars is exact (126 tool description + 2,198 argument
description). Word count: the argument description alone is 380 words; the
tool description adds 21 more → combined = **401**. The token estimate
(≈550–615) and the 1K gate are unaffected.

**Suggested resolution:** Change 380 → 401 (or drop the word count and keep
chars/tokens). Corroborated by seat E (N6, independently measured 401).

### F7 [Nit] (CONSISTENCY) — §6's "2026-09-15 glm-5.2 evidence (26/37 ≈ 70%…)" conflates the 3-day aggregate with the single day

**Where:** §6: "…with the 2026-09-15 glm-5.2 evidence (26/37 ≈ 70% vs qwen
5/413 ≈ 1%, §1.2)."

**Evidence:** 26/37 is the 2026-09-13..15 aggregate (§1.2). On 09-15 alone
the glm split is 13 PARSE / 8 OK of 21 calls (~62%, as §1.2 itself states).
Dating the aggregate "2026-09-15" is misleading.

**Suggested resolution:** "with the 2026-09-13..15 glm-5.2 evidence
(26/37 ≈ 70%; 09-15 alone 13/21 ≈ 62%)", or drop the date.

## Mandate 1 — §1.2 evidence table re-derivation

Method: snapshot copy (00:22Z) + fresh classification run this session
(`/tmp/seatf/final.py`); every non-OK call inspected individually; day-level
splits computed from the raw rollouts.

| Model | calls | PARSE | OK | VERIFY | other |
|---|---|---|---|---|---|
| glm-5.2 | **37 — exact** | 26 (~70%) | 11 | 0 | — |
| qwen3.8-27b | **439 (spec: 413)** | 6 | 396 | 34 | MISSING 1, UNSUPPORTED 1, OTHER 1 |

VERIFY column in the spec's own bucketing (the two
`multiple operations target` rejections counted with the
`verification failed` prefix family); the strict-definition reading
(F1/E-N1) is 33 VERIFY + 2 OTHER.

- **glm row: EXACT.** 09-14: 13 PARSE / 3 OK of 16; 09-15: 13 PARSE / 8 OK
  of 21 → the spec's "Most recent day (09-15) alone: 13 of 21 glm calls
  rejected (~62%)" is correct. Zero glm call in the three days failed with a
  filesystem verification error, as claimed.
- **qwen row: 413 → 439; mechanism in F1.** The spec's 413 matches the
  first-413-by-timestamp prefix exactly on all six bucket totals (cutoff
  2026-09-15T23:30:11.662Z); the +26 delta is the trailing tail
  (22 OK + 1 PARSE + 3 VERIFY). Conclusions intact: qwen emits the canonical
  prefixed format from priors; the rate is now 6/439 ≈ 1.4% (~1%), still an
  order of magnitude below glm's ~70%.
- qwen non-OK composition at 439 (all ids in `/tmp/seatf/calls2.tsv`):
  PARSE 6; VERIFY 34 = 28 "Failed to find expected lines" + 2 "Failed to
  read file" + 2 "Failed to find context" + 2 "multiple operations target";
  MISSING 1 (`call_26324b8d…`, args `{}`, 22:13:05Z — the single observed
  F3-shaped event, self-corrected by the next call `call_a687a588…` at
  22:14:51Z); UNSUPPORTED 1 (`call_3c03fe79…`, legacy `{"cmd": […]}` shape,
  09-13, pre-seam); OTHER 1 (`call_be0ac666…`, exec-style "Exit code: 1 …
  Failed to write file", 09-14).

## Mandate 2 — parser claims re-probed

All probes run on the unmodified `codex-rs/target/debug/apply_patch`
(apply-patch crate `git status`-clean; the branch makes no changes to that
crate, so it is unmodified for P2/P3 purposes). Stderr/exit captured under
`/tmp/seatf/*.err`; file outputs checked byte-wise (incl. `xxd`).

| Probe | Spec claim | Result |
|---|---|---|
| Raw (unprefixed) Add-File content, F1-shaped | rejected pre-P2 | PASS — `Invalid patch hunk on line 3: '…' is not a valid hunk header. Valid hunk headers: …` (exit 1, no file written) |
| Missing `*** Begin Patch` / `*** End Patch` | boundary pre-pass messages | PASS — `Invalid patch: The first/last line of the patch must be '*** Begin/End Patch'` (CLI format; the harness-level output carries the `invalid patch:` Display prefix — see F3) |
| Update hunk with no `@@` | auto-chunk, accepted | PASS — exit 0, file written (auto-chunk branch, `streaming_parser.rs` ~:306-349; `@@` pin at `parser.rs:402`) |
| Move-only Update (`*** Move to:` + no lines) | exact empty-hunk error | PASS — `Invalid patch hunk on line 2: Update file hunk for path 'a.txt' is empty` |
| Zero-content Add-File | creates a 0-byte file | PASS |
| Final line `+*** End Patch` | preserved verbatim as content | PASS — file is `line1\n*** End Patch\n` (20 bytes, `xxd`-verified); a bare final `*** End Patch` truncates as claimed |
| Second `@@` inside an empty chunk | rejected | PASS |
| New `@@` after `*** End of File` | parse-accepted | PASS |
| T2.2 embedded example (extracted per the pinned rule: exactly one `Example:` line, at index 19; the remainder ends `*** End Patch`) | end-to-end byte-exact | PASS — `notes/todo.md` = `# TODO\n\n1. ship the fix\n`; `src/main.rs` = one chunk, context `fn main`, 1 removal (`    old_call();`) / 1 addition / 1 context |
| `.lark` grammar | `add_line: "+" /(.*)/ LF -> line`; `add_hunk … add_line+`; `change: (change_context \| change_line)+ eof_line?` | VERIFIED verbatim (`codex-rs/core/assets/tools/apply_patch.lark:6,11,14,17`) |

T1.9b's raw-truncation half is post-P2 behavior (code-derived, not probeable
on the pre-P2 binary) — consistent with the spec scoping it post-P2.

Corroboration: seat E ran the same six mandated probes on the same binary
with the same results.

## Mandate 3 — test-plan references and red-state completeness

Every reference I opened verifies at HEAD (exact lines unless noted):

- **Red state:** `streaming_parser.rs:814` — 3 red assertions at
  :828/:838/:848, with the :819 NotStarted assertion staying green (its
  message is out of P3.4 scope) ✓; `parser.rs:277` — 2 boundary assertions
  (:281/:287) ✓; `parser.rs:558` — **7, not 5** → F4; `tool.rs:386`
  (`test_apply_patch_cli_rejects_invalid_hunk_header`, exact CLI stderr; the
  CLI formats at `apply-patch/src/lib.rs:378-385`) ✓;
  `apply_patch_cli.rs:707` (`out.contains("is not a valid hunk header")`
  substring — survives P3.1) ✓.
- **P3 sites:** `apply_patch.rs:411` (handler `parse_patch`; the
  `RespondToModel` error prefix at :415), :508-560 (absent/non-string
  `patch` collapsed at :535-541; existing backtick-quoted message at :539),
  :603-605 (`matches_kind` Function-only) ✓; `parser.rs:268/:271` (P3.4
  targets; both strings owned by `check_start_and_end_lines_strict`,
  defined at :256 — seat E's N7 notes the cited span "256-272" is actually
  256-274; the message lines themselves are exactly as cited) ✓.
- **P3.4 shell-intercept surface:** `invocation.rs:116/:123` (direct +
  heredoc forms), :170/:175 (implicit-invocation checks), :235-245
  ("multiple operations target" message at :236-241; no existence check at
  :242-245) ✓.
- **Routing/seam:** `model-provider-info/src/lib.rs:40`
  (`OPENAI_PROVIDER_NAME`), :465, :546-547 (`is_openai()` is a name match —
  no Azure branch) ✓; `model-provider/src/provider.rs:372`
  (`apply_patch_function_tool: !self.info.is_openai()`) ✓.
- **Test scaffolds:** `core/tests/common/responses.rs:933-942`
  (`ev_function_call`), :1008-1018 (`ev_apply_patch_custom_tool_call`),
  :1030-1038 (`ev_apply_patch_exec_command_call_via_heredoc`) ✓;
  `test_codex.rs:347` (`with_config`), :844 (openai provider clone),
  :858-881 (provider assigned at :859, mutators after), :1376 ✓;
  `apply_patch_spec_tests.rs:40` ✓; `apply_patch_tests.rs:45`
  (`invocation_for_payload`) ✓; `session/tests.rs:5882`
  (`make_session_and_context`) ✓.
- **Prompt:** `default.md:132` (the legacy `{"command": […]}` apply_patch
  example) ✓.
- **Sweep:** repo-wide search finds no other test asserting the changed
  strings; "missing the required" is asserted by no existing test ✓.
- **Scenarios:** 24 numbered dirs under
  `codex-rs/apply-patch/tests/fixtures/scenarios/` incl.
  `003_multiple_chunks` and `011_add_overwrites_existing_file` ✓.
- **Drift:** the three ranges in seat E's N7 (auto-chunk cited 316-360 vs
  actual ~306-349; `ev_exec_command_call_with_args` cited 1030-1038 vs
  actual :1025-1028; pre-pass span 256-272 vs 256-274) — noted, not
  re-flagged.

Red-state completeness: with F4's correction (7 not 5), the enumeration is
complete — 13 exact-string assertions must go red (3 streaming + 1 CLI + 2
`test_parse_patch` + 7 `test_parse_patch_lenient`); the spec as written
implies 11.

## Mandate 4 — consistency checks

- Drift-guard: all 7 substrings literal in the final P1 argument
  description ✓ (prior session check, re-confirmed against the final text).
- FORMAT block: description column starts at column 31 in every row ✓.
- Char counts: 126 + 2,198 = 2,324 ✓; word count → F6.
- "plain text" / "wrap it in JSON" absent from the new text ✓.
- T2.2 naive-substring caveat accurate: the `Example:` line is unique
  (index 19) and the remainder ends `*** End Patch` ✓.
- vLLM citations re-checked on GitHub this session: #49248 open (title:
  "glm47 tool parser silently returns empty arguments when the model omits
  the opening <arg_value> tag"; ~5–17% of calls at 6-way concurrency, 0
  sequential, temperature 0); #49249 open (the fix, linked as the open PR on
  #49248 — the duplicate #51364 is CLOSED, which does not affect the
  spec's claim about #49249); #47504 / #55541 / #49981 open (forced
  `tool_choice` unsafe cluster) ✓.
- base_instructions parity: all 36 glm + 85 qwen session files share one
  20,751-char text (SHA-256 prefix `ac8ae107a0d7`); the 17,730-char variant
  (`cbefa6b0…`) appears only on gpt-5.6-*/gpt-6-astra/gemma seats ✓.
- P3.1/P3.4 prefix-substring claims: hold at their own sites (F5 covers
  P3.2/P3.3 only).
- Model-switch file (`rollout-2026-09-13T18-30-29-…`, first `turn_context`
  qwen → later glm): its 2 OK calls are qwen-attributed per the documented
  method — spec and recount agree ✓.

## Checked and clean

1. **glm row exact:** 37 = 26 PARSE + 11 OK + 0 VERIFY; day splits 09-14
   13P/3OK of 16, 09-15 13P/8OK of 21; "13 of 21 (~62%)" correct; zero glm
   filesystem-verification failures in the three days.
2. **All 26 glm PARSE call_ids classified** (17 F1-class / 2 F2 / 5 Begin /
   2 End; ids in F2); the three TSV messages truncated at 120 chars
   re-checked against raw snapshot output (all end "…is not a valid hunk
   header").
3. **qwen non-OK composition at 439** (Mandate 1): 6 PARSE / 34 VERIFY
   (28+2+2+2) / 1 MISSING / 1 UNSUPPORTED / 1 OTHER, all call_ids in
   `/tmp/seatf/calls2.tsv`; MISSING self-correction re-verified (next call
   `call_a687a588…`, 22:14:51Z, OK).
4. **Delta +26 mechanism** (F1): the spec's 413 = first-413-by-timestamp
   prefix (cutoff 2026-09-15T23:30:11.662Z; all six bucket totals match at
   that point); the delta is exactly the trailing tail
   (2026-09-15T23:46:42Z → 2026-09-16T00:23:58Z; 22 OK + 1 PARSE + 3
   VERIFY); the live dir is still growing (86 → 97 day-15 files between
   00:22Z and 01:36Z; newest rollout line timestamped within seconds of
   now).
5. **Byte-exact F1 evidence:** `call_caf7ba09…` — args JSON 24,230 chars,
   `patch` string 23,972 chars, line 3 quoted exactly as sent (§1.3 sizes
   match).
6. **Byte-exact F2/F3 evidence:** the glm raw-content examples (incl. the
   `# SPEC-FREEZE-1 ROUND 23 — …` first line) and both F4 harness outputs
   (with the `invalid patch:` prefix — F3) verified against the raw
   rollouts.
7. **Error plumbing:** `ParseError::InvalidPatchError`
   `#[error("invalid patch: {0}")]` at `parser.rs:57`; CLI formats at
   `lib.rs:378-385` (`Invalid patch: …` / `Invalid patch hunk on line N: …`).
8. **Parser probes:** all 10 in the Mandate 2 table PASS on the unmodified
   binary.
9. **`.lark` verbatim:** `add_line: "+" /(.*)/ LF -> line`; `add_hunk …
   add_line+`; `change: (change_context | change_line)+ eof_line?`;
   `eof_line: "*** End of File" LF`.
10. **Test-plan references:** the full Mandate 3 list verifies exact at
    HEAD (N7's three ranges excepted).
11. **Red-state counts:** `test_parse_patch` 2 (spec correct);
    :819 NotStarted stays green; true red total 13 (spec implies 11 — F4).
12. **Repo-wide sweep:** no other tests assert the changed strings;
    "missing the required" asserted nowhere.
13. **Scenarios:** 24 numbered dirs incl. `003_multiple_chunks`,
    `011_add_overwrites_existing_file`.
14. **P1 text:** all 7 drift-guard substrings literal; 126/2,198/2,324
    chars; "plain text"/"wrap it in JSON" absent; FORMAT desc column at col
    31; `Example:` line unique.
15. **vLLM:** #49248 open; #49249 open (dup #51364 closed — spec's claim
    holds); #47504/#55541/#49981 open.
16. **base_instructions parity:** 36 glm + 85 qwen files, one 20,751-char
    text, SHA-256 prefix `ac8ae107a0d7`; the 17,730-char variant only on
    gpt-5.6-*/gpt-6-astra/gemma seats.
17. **Model-switch 09-13 file:** 2 OK calls, qwen-attributed per the
    documented method (spec and recount agree).
18. **P3.1/P3.4 prefix claims:** hold at their own sites (F5 is P3.2/P3.3
    only).
19. **T2.2:** naive-substring caveat accurate; the example applies
    byte-exact end-to-end (probe).
20. **Executability of §4 commands:** Justfile `test` recipe takes `*args`
    and `~/bin/wiretap.py` is passive (per seat E; not independently
    re-run).

## Counts

- Findings: **0 Blocking, 2 Major (F1, F2), 3 Minor (F3, F4, F5), 2 Nit
  (F6, F7)** — verdict CHANGES-REQUESTED (as in the header).
- Overlap with seat E (same round): F1↔N1, F2↔N2, F4↔N5, F5↔N4, F6↔N6 —
  each independently re-derived in this session; F1 adds the drift
  mechanism, F2 the 17 call_ids. F3 and F7 are seat-F-only.
- Mandate 1: glm row EXACT; qwen row drifted 413 → 439 (F1); conclusions
  (glm ~70% vs qwen ~1%) intact.
- Mandate 2: 10/10 probes PASS on the unmodified binary.
- Mandate 3: all references verified; red state undercounted by 2 (F4) →
  13 red assertions, not 11.
- Mandate 4: two wording nits (F6, F7); no other inconsistencies found.
- Checked-and-clean items: 20 (above).
