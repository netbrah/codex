# Review — apply-patch function-tool format spec (v3), Round 4, Seat H

Target: `docs/responses-compat-apply-patch-format.md` (v3, 915 lines), branch
`feat/normalize-content-types-vllm`
Reviewer: seat H (independent leaf reviewer; nothing trusted from the spec or prior
rounds — every count re-run, every cited file/line opened, probes re-run). Mandated
scope: (1) verify the round-3 fixes landed correctly, (2) red-state completeness,
(3) implementability of T1–T5, (4) line-ref sweep, (5) v2→v3 regression hunt.
Date: 2026-09-15, ~23:40–01:00 EDT (2026-09-16T03:40–05:00Z).

## Verdict

**APPROVED — 0 Blocking, 0 Major, 1 Minor, 1 Nit.**

All nine round-3 findings are resolved in v3: eight fully, one (E-N3) partially — the
§1.1 examples and header were corrected to the real session-dir layout, but the §1.2
Method line still carries a third, still-wrong notation, and the R3 resolution map
records the wrong layout as the actual one (H1, Minor — the class the original N3
was rated in). The red-state list is exactly right (all 13 asserted locations opened
and counted; a repo-wide sweep finds no other test asserting any changed string).
Every T1–T5 item is executable as written against HEAD, and every `file.rs:NNN`
citation I opened verifies. No v2-correct content was lost or contradicted by the v3
edits (13/13 diff hunks trace to the R3 map or bookkeeping).

Method (everything re-verified at the source this session):
- Every red-state string rg'd across the whole `codex-rs` tree; every assertion hit
  opened and attributed to its test function and line.
- Every `file.rs:NNN` citation in v3 extracted by regex and opened at HEAD (31 distinct).
- T1–T5 walked item by item; each named symbol/file/line confirmed to exist and the
  item executable as written.
- Three probes on the unmodified `codex-rs/target/debug/apply_patch` (built 2026-09-15
  19:25; the `apply-patch` crate is git-clean with no source newer than the binary, so
  it is unmodified for P2/P3 purposes — same state rounds 1–3 used): (a) the T2.2
  embedded example applies byte-exact; (b) the F1-shaped raw Add-File patch is
  rejected pre-P2; (c) context anchoring is exact line match.
- The §3.1 P1 code block was extracted from the spec text and checked mechanically:
  all 7 drift-guard substrings literal; the extraction rule (split at first line
  exactly `Example:`; remainder ends `*** End Patch`) works on the v3 text; 126 /
  2,198 / 2,324 chars exact.
- `diff -u` of the v2 archive → v3 (13 hunks); each hunk audited against the R3 map.
- v3's new evidence claims spot-checked against the live rollouts: the two OTHER
  call_ids and their `multiple operations target …` rejections, the pinned file
  counts for 09-13/09-14 (83/162 — still exact on disk), and the F1 arg sizes.

## Mandate 0 — Round-3 fixes verification (v3's own claim, per the R3 map)

| Finding | v3 resolution claim | Verification (at source) | Result |
|---|---|---|---|
| E-N1 / F-F1 (qwen pin) | Snapshot pinned 2026-09-15T23:30:11Z, 83/162/86 files; 30 VERIFY / 2 OTHER re-tabulation; pinned-instant wording; freshness note (2026-09-16T00:22Z: 439, 6 PARSE ≈ 1.4%); glm row snapshot-stable | §1.2 Snapshot bullet present with pin + file counts (09-13: 83 and 09-14: 162 re-counted on disk, exact; 09-15 dir is live-growing as disclosed — 104 at review time); table row sums 374+5+30+1+1+2=413; the two named OTHER rejections (`call_30690924…`, `call_096536c4…`) located in the 09-15 rollouts with `multiple operations target <path>` text; TL;DR freshness note present; “reproduces exactly” reworded to the pinned instant with the independent reproductions named | **LANDED** |
| E-N2 / F-F2 (glm 17) | 17 F1-class (10 `#` + 3 `VERDICT:` + 3 `@@` + 1 `---`) + 2 F2 + 7 F4 = 26; v2 sum-22 error named | §1.2 breakdown now 17/2/5/2 (sums to 26; sub-breakdown 10+3+3+1=17); v2 miscount explicitly named; TL;DR 19+7 split consistent (17+2=19, 5+2=7) | **LANDED** |
| E-N3 (session paths) | “unified to the actual layout in §1.1/§1.2” | Header (line 5) and all three §1.1 example paths now use the real `2026/09/{13,14,15}` layout (verified on disk: `~/.codex/sessions/2026/09/13|14|15` exist; `~/.codex/sessions/2026-09/` does not). **But §1.2 Method (line 134) still says `~/.codex/sessions/2026-09/{13,14,15}/` — wrong**, and the R3 map row + round-3 summary record `2026-09/{13,14,15}` as the actual layout, contradicting seat E's own finding, which quoted the real layout as `2026/09/{13,14,15}/rollout-*.jsonl` (verified on disk) | **PARTIAL — H1** |
| E-N4 / F-F5 (prefix status) | Sites 1 and 4 preserve prefix; site 2 replacement; site 3 re-quotes arg name (backticks → single quotes) and splits the collapsed error | §3.3 intro now states exactly this, per site. Confirmed against source: site 1 (streaming_parser.rs:193) and site 4 (parser.rs:268/:271) are sentence extensions; site 2 (streaming_parser.rs:222) old text shares no prefix with the new text; site 3 (apply_patch.rs:539) is backtick-quoted today and the new text diverges at the quote char | **LANDED** |
| E-N5 / F-F4 (red count 7) | `test_parse_patch_lenient` corrected to 7 (6 Begin + 1 End, enumerated); total 13 | Opened parser.rs:558-644: `expected_error` bound at :575-576; Begin assertions at :579/:594/:609/:624 (Strict, the four heredoc variants) / :628 (Lenient mismatched-quotes) / :635 (Strict missing-closing); End assertion at :639-642 (Lenient missing-closing). All 7 compare the exact pre-P3.4 strings, so all 7 go red. Total 3+1+2+7=13 | **LANDED** |
| F-F3 (`invalid patch: ` prefix) | Prefix added to both §1.1 F4 quotes; parser-level vs harness-level strings distinguished | Both quotes now read `apply_patch verification failed: invalid patch: The …`; the distinguishing sentence names `ParseError::InvalidPatchError`'s Display format at parser.rs:57 (verified: the attribute is exactly at :57) and states the prefix-less strings are what P3.4 edits | **LANDED** |
| E-N6 / F-F6 (401 words) | 2,324 chars / 401 words, 380 noted as arg-only | Re-measured the extracted §3.1 block: tool description 126 chars/21 words, argument description 2,198 chars/380 words, combined 2,324 chars/401 words — exact | **LANDED** |
| F-F7 (§6 date conflation) | 26/37 is the 2026-09-13..15 aggregate; 09-15 alone 13/21 ≈ 62% | §6 now reads “the 2026-09-13..15 glm-5.2 evidence (26/37 ≈ 70% as of the §1.2 pinned snapshot; 09-15 alone 13/21 ≈ 62%)” | **LANDED** |
| E-N7 (line ranges) | auto-chunk 307-347; pre-pass 256-274 (messages :268/:271); responses.rs patterns cited by name + line | streaming_parser.rs auto-chunk branches (context/`+`/`-` auto-create) at :318-:343, inside 307-347; `check_start_and_end_lines_strict` spans :256-:274 with the two messages exactly at :268/:271; responses.rs `ev_exec_command_call_with_args` exactly :1025-:1028 and `ev_apply_patch_exec_command_call_via_heredoc` exactly :1030-:1035 — all match v3's corrected cites (the v2 cites 316-360 / 256-272 / 1030-1038 are gone from live text) | **LANDED** |

## Mandate 1 — Red-state completeness (the implementer's red-log contract)

Every message string P2/P3/P3.4 changes or replaces was rg'd across the whole
`codex-rs` tree. Occurrences and their status:

| String (site) | Source owners | Test assertions | Status under P2/P3 |
|---|---|---|---|
| StartedPatch 'not a valid hunk header…' (P3.1) | streaming_parser.rs:193 | :828 ('bad'), exact | **red — enumerated** (T1.4 rewrites) |
| same text, AddFile arm (P2) | streaming_parser.rs:211 | :838 ('bad'), exact | **red — enumerated** (T1.6: becomes `Ok([AddFile{contents: 'bad\n'}])`) |
| same text, DeleteFile arm (P3.2) | streaming_parser.rs:222 | :848 ('bad'), exact | **red — enumerated** (T1.5 rewrites to the new message) |
| same text, CLI exact-stderr | — (CLI formats 'Invalid patch hunk on line 2: …') | tool.rs:393, one exact-stderr assertion (test at :386) | **red — enumerated** (T1.12 rewrites) |
| same text, substring | — | core/tests/suite/apply_patch_cli.rs:707 `out.contains('is not a valid hunk header')` | **stays green — verified**: the test's patch ('*** Frobnicate File: foo' as first line after Begin) hits the StartedPatch arm, whose sentence P3.1 keeps verbatim |
| Begin boundary message (P3.4) | parser.rs:268 (edited); streaming_parser.rs:184 (untouched) | parser.rs:281 (test_parse_patch); parser.rs:575-576 expected_error binding feeding 6 assertions in test_parse_patch_lenient; streaming_parser.rs:819 (NotStarted) | parser sites **red — enumerated** (1 in test_parse_patch + 6 in test_parse_patch_lenient); :819 stays green (untouched message) |
| End boundary message (P3.4) | parser.rs:271 (edited); streaming_parser.rs:168, :374 (untouched) | parser.rs:287 (test_parse_patch); parser.rs:642 (test_parse_patch_lenient); streaming_parser.rs:784 (finish_requires_end_patch), :797 (rejects_content_after_end_patch) | parser sites **red — enumerated** (1 in test_parse_patch + 1 in test_parse_patch_lenient); :784/:797 stay green (untouched messages) |
| 'apply_patch is missing the required patch argument' (P3.3) | apply_patch.rs:539 (single collapsed path for absent + non-string via get('patch').and_then(Value::as_str) at :535-536) | **none** — no test in the tree asserts this message (only unrelated user-verification strings match 'is missing the required') | red N/A; T3.1 covers behavior; P3.3's new texts are asserted nowhere today (they are new) |
| P3.2 new text / P3.3 new texts | (new) | none anywhere | new, no pre-existing assertions |

Per-test counts, each opened and line-checked:

| Test | Cite | Red assertions | Verified |
|---|---|---|---|
| test_streaming_patch_parser_returns_errors | streaming_parser.rs:814 | 3 — :828 (StartedPatch 'bad', P3.1), :838 (AddFile 'bad', P2), :848 (DeleteFile 'bad', P3.2); :819 (NotStarted Begin message) stays green; the empty-Update-hunk assertion (~:855, 'Update file hunk for path file.txt is empty') also stays green (untouched message) | exact |
| test_apply_patch_cli_rejects_invalid_hunk_header | apply-patch/tests/suite/tool.rs:386 | 1 — the exact CLI-stderr assertion at :393 | exact |
| test_parse_patch | parser.rs:277 | 2 — :281 (Begin), :287 (End) | exact |
| test_parse_patch_lenient | parser.rs:558 | 7 — Begin at :579/:594/:609/:624 (Strict heredoc variants), :628 (Lenient mismatched-quotes), :635 (Strict missing-closing); End at :639-642 (Lenient missing-closing) | exact |

**Total: 13, matching the spec's '3 streaming + 1 CLI + 2 + 7.'**

Completeness sweep results (no other test asserts any changed string):
- 'is not a valid hunk header': 8 tree hits — the 3 source arms, the 3 streaming test
  assertions, tool.rs:393, apply_patch_cli.rs:707. Nothing else (no golden scenario,
  no core-suite test, no TUI/app-server test).
- Boundary strings: only the source sites above and the enumerated test assertions.
- The 24 golden scenario fixtures: none asserts stderr or exit status (the scenario
  runner explicitly compares final filesystem state only), and none of the four
  Add-File scenario patches (001/002/011/015) contains a raw unprefixed content line,
  so P2 changes no scenario's final state — the spec's T1.13 'pass unmodified' claim
  holds for both messages and behavior.
- Core diff-consumer tests (apply_patch_tests.rs) feed only canonical + -prefixed
  Add-File content — P2 flips none of them.

## Mandate 2 — Implementability (T1–T5, item by item)

| Item | Claim | Verification | Status |
|---|---|---|---|
| T1.1-T1.3 (F1 replay, empty line, mixed prefixes) | P2 makes raw Add-File lines verbatim content | Code: AddFile arm (streaming_parser.rs:198-214) — structural check first, then + branch (untouched), then the final Err that P2 replaces; 'verbatim' matches the + branch's no-trim semantics; push_delta strips one trailing CR per line (:133-151) so CRLF→LF holds | executable |
| T1.4-T1.6 | rewrite the 3 streaming 'bad' assertions | assertions located exactly at :828/:838/:848 (Mandate 1) | executable |
| T1.7 (raw '*** ' line → content; real markers structural; padded End structural) | handle_hunk_headers_and_end_patch (streaming_parser.rs:84-135) runs on the trimmed line: exact END_PATCH_MARKER match, strip_prefix for the 3 file markers, env-id gated to StartedPatch | a typo'd marker (e.g. '*** Ad File: x') falls through to Ok(false) → content under P2; '*** End Patch' with whitespace trims to the marker → structural | executable |
| T1.8 (typo'd structural line swallowed) | same code path as T1.7 | verified in code; the §3.2 'silent-partial-apply class' description matches | executable |
| T1.9 (raw leading-+ lossy case; existing-file overwrite) | + branch strips the leading +; Add-File arm in lib.rs:508-535 writes unconditionally (no existence check — invocation.rs:242-245 likewise) | verified in code (lib.rs write at :517-526 via write_file_with_missing_parent_retry) | executable |
| T1.9b (+End-Patch content vs raw truncation) | structural check runs on the trimmed line, which retains the + for the canonical form, so +*** End Patch takes the + branch (content); bare marker takes the structural branch (truncate) | verified in code; both r3 seats probed the same on the unmodified binary | executable |
| T1.10 (whitespace-only / env-id / unclosed / CRLF edges) | push_delta CR strip verified; env-id gating verified; unclosed patch → finish() boundary error untouched | verified in code | executable |
| T1.11 (Update raw line still rejected, message unchanged) | UpdateFile arm fall-through (streaming_parser.rs:362-368) raises the exact F2 message for unprefixed lines in both empty and non-empty chunk states | verified in code; message untouched by P2/P3 | executable |
| T1.12 | tool.rs:386 test updated to new StartedPatch message | test + exact stderr at :393 verified | executable |
| T1.13 (scenarios + canonical tests unmodified) | no scenario asserts stderr; no raw Add-File lines in scenario patches; + branch untouched | verified (Mandate 1 sweep) | executable |
| T1.14 (parse_patch_text Strict boundary paths) | parse_patch_text (parser.rs:193) takes ParseMode; both Strict and Lenient paths run check_start_and_end_lines_strict (lenient first calls strict, :232-250) — so P3.4's two new messages are reachable from both tests as written; both tests call parse_patch_text directly with both modes | verified in code | executable |
| T2.1 (7 drift-guard substrings) | all 7 literal substrings of the §3.1 argument description | extracted the code block from the spec text and checked each mechanically: first line is **Begin Patch** / real newline characters / bare '+' / at most one hunk per patch / starts with '+' / multiple **@@** chunks / must change at least one line — **7/7 literal** | verified |
| T2.2 (self-consistency extraction) | split at first line exactly Example:; remainder ends *** End Patch; parses to the asserted hunks | Example: line unique (index 19 of 31); remainder ends with the marker line; probed the remainder through the unmodified binary in a scratch dir: exit 0; notes/todo.md byte-exact (# TODO, blank line, 1. ship the fix, trailing LF — xxd-verified); src/main.rs = one chunk, context fn main, 1 removal (old_call();) / 1 addition (new_call();) / 1 context (shared();) — exactly the asserted hunks | verified |
| T2.3 (snapshot test update) | create_apply_patch_function_tool_matches_expected_spec at apply_patch_spec_tests.rs:40 exists and is the full-snapshot test of the function tool | verified (fn at :40; asserts name/description/strict/parameters verbatim); current shape matches the spec's description (one required string patch, optional environment_id, strict false, additionalProperties false) | verified |
| T2.4 (freeform unchanged) | create_apply_patch_freeform_tool spec text untouched by P1 | verified in apply_patch_spec.rs:13-38 | executable |
| T3.1 (absent / non-string patch) | handle_call (apply_patch.rs:508-560) collapses both via get+and_then+as_str (.ok_or_else at :537-541, message :539) — the split into two messages is a minimal in-place edit | verified in code | executable |
| T3.2 (handler-level F1 replay) | scaffolds exist: invocation_for_payload (apply_patch_tests.rs:45), make_session_and_context (session/tests.rs:5882) | both exist at the cited lines | executable |
| T4.1 (non-OpenAI provider request-body assert) | with_config at test_codex.rs:347; default provider cloned from built_in_model_providers[openai] (name 'OpenAI', test_codex.rs:839-846, assigned :858) → freeform path; renaming via the mutator is the documented pattern; request body inspectable via mount_sse_sequence → ResponseMock::single_request()/requests() (responses.rs:39-58) | all verified; the dummy api-key auth is provider-name-agnostic | executable |
| T4.2 (function-call scaffold) | mount_apply_patch (apply_patch_cli.rs:228-243) always passes ev_apply_patch_custom_tool_call; FunctionApplyPatchHandler.matches_kind is Function-only (apply_patch.rs:603-605); registry rejects the kind-mismatched payload ('tool {name} invoked with incompatible payload', registry.rs:550, block :549-562); new ev_apply_patch_function_call fits apply_patch_responses' fn(&str, &str) -> Value slot (apply_patch_cli.rs:266-281); pattern helpers verified at responses.rs:1025-1028 and :1030-1035; generic ev_function_call at :933-942 | all verified; the spec's 'not usable' claim for mount_apply_patch on the renamed provider is correct (spec_plan.rs:1257-1271 registers only the Function handler when capabilities().apply_patch_function_tool, which is !is_openai() at model-provider/src/provider.rs:372) | verified |
| T4.2b (existing-file overwrite variant) | same shape as T4.2 + a pre-existing file | T1.9 pins the parser/apply behavior; integration follows T4.2 | executable |
| T5 (gates) | just fmt / just test -p … / just fix -p … | the repo justfile has fmt, fix *args, and test *args (cargo nextest run --no-fail-fast); codex-apply-patch, codex-core, and codex-model-provider crates all exist; the scoped invocations are syntactically valid nextest/clippy arguments | executable |

Probe log (unmodified codex-rs/target/debug/apply_patch; binary built 2026-09-15
19:25, apply-patch crate git-clean, no source newer than the binary):
1. **T2.2 example replay** — scratch src/main.rs with exactly fn main / old_call(); /
   shared(); → exit 0; A notes/todo.md and M src/main.rs; notes/todo.md = 2320 544f
   444f 0a 0a 312e … (byte-exact # TODO + blank line + 1. ship the fix + LF);
   src/main.rs = fn main / new_call(); / shared(); → PASS, exactly the asserted hunks.
2. **F1 raw Add-File** — the spec's verbatim first three lines (# SPEC-FREEZE-1
   ROUND 23 … em dash …) as raw content → exit 1, stderr 'Invalid patch hunk on line
   3: … is not a valid hunk header. Valid hunk headers: …', no file written → PASS
   (F1 must still be rejected by the unmodified binary).
3. **Exact context match** — same T2.2 update against a file whose anchor line is
   fn main() { (not exactly fn main) → exit 1, 'Failed to find context 'fn main'' →
   PASS (context anchoring is exact line match, as the T2.2 assertions require).

## Mandate 3 — Line-ref sweep (every file.rs:NNN citation in v3)

| Citation (v3 location) | Verified at HEAD |
|---|---|
| streaming_parser.rs:307-347 (auto-chunk) ×3 | auto-create branches at :318/:329/:340 (+ empty-line case :302-315); all inside 307-347 |
| streaming_parser.rs:814 (test) | fn at :814; assertions :819/:828/:838/:848 as claimed |
| streaming_parser.rs:168 / :184 / :374 (parallel boundary messages, P3.4 out-of-scope) | finish() at :168, NotStarted at :184, EndedPatch at :374 — exactly as cited, untouched by P3.4 |
| parser.rs:256-274 (check_start_and_end_lines_strict) ×3 | fn :256-:274; messages exactly at :268/:271 |
| parser.rs:57 (#error('invalid patch: {0}')) | attribute exactly at :57 |
| parser.rs:193-199 (parse_patch_text pre-pass call) | fn at :193; Strict/Lenient match arms :196-199 calling check_patch_boundaries_* |
| parser.rs:277 / :558 (tests) | fn at :277 and :558 |
| parser.rs:402 (upstream @@-less pin) | comment 'Update hunk without an explicit @@ header …' at :402, assertion immediately below |
| invocation.rs:235-241 (multiple-operations; message :237) | if at :235, message at :237 |
| invocation.rs:242-245 (no existence check) | match at :242, AddFile arm :243-245 (direct insert, no fs read) |
| invocation.rs:116 / :123 / :170 / :175 (shell-intercept parse calls) | parse_patch at :116 (direct), :123 (heredoc), :170/:175 (implicit-invocation checks) |
| lib.rs:370 (CLI parse) | let hunks = match parse_patch(patch) at :370; CLI error formatting :374-384 |
| lib.rs:505-528 (Add-File apply/overwrite) | AddFile arm actually :508-:535; the cited range contains the unconditional write (:517-526) — range start two lines early, harmless |
| apply_patch.rs:411 (handler parse) | parse_patch call at :411 in run_apply_patch_text (the function handler's execution path) |
| apply_patch.rs:508-560 (argument errors) | handle_call :508; collapsed get/and_then/as_str :534-536; message :539 |
| apply_patch.rs:603-605 (matches_kind Function-only) | fn at :603, matches!(payload, ToolPayload::Function { .. }) at :604 |
| registry.rs:548-556 (kind-mismatch rejection) | if at :549, message at :550, block to :562 — cited range starts one line early, message inside |
| model-provider-info/src/lib.rs:546 (is_openai, no Azure branch) | fn at :546, body name == OPENAI_PROVIDER_NAME at :547; OPENAI_PROVIDER_NAME = 'OpenAI' at :40 |
| default.md:132 (legacy command-array example) | the {"command":["apply_patch", …\\n…]} line exactly at :132, Update-file prefixes only, literal backslash-n |
| spec_plan.rs:1257-1271 (capability-gated registration) | function-vs-freeform branch exactly :1257-:1271 (core/src/tools/spec_plan.rs) |
| responses.rs:933-942 (ev_function_call) | fn :933-:942 |
| responses.rs:1025-1028 / :1030-1035 (pattern helpers) | ev_exec_command_call_with_args :1025-:1028; ev_apply_patch_exec_command_call_via_heredoc :1030-:1035 — exact |
| apply_patch_cli.rs:228 (mount_apply_patch) | fn at :228; always passes ev_apply_patch_custom_tool_call |
| apply_patch_cli.rs:323 (read_file_text harness method) | :323 is a usage site (harness.read_file_text); the method itself is defined at tests/common/test_codex.rs:1209 |
| apply_patch_cli.rs:707 (surviving substring test) | :707 is the out.contains('is not a valid hunk header') assertion in the test starting at :689 |
| apply_patch_spec_tests.rs:40 (snapshot test) | fn at :40 |
| tests/suite/tool.rs:386 (CLI test) | fn at :386; exact-stderr assertion at :393 |
| core/tests/suite/apply_patch_cli.rs:707 | see above (same line) |
| apply_patch_tests.rs:45 (invocation_for_payload) | fn at :45 |
| session/tests.rs:5882 (make_session_and_context) | fn at :5882 |
| test_codex.rs:347 (with_config) | pub fn with_config at :347 (core/tests/common/test_codex.rs) |
| test_codex.rs:839-859 (default provider 'OpenAI') | ModelProviderInfo literal :839-:846 (clone of built-in 'openai'), assigned :858 |

## Mandate 4 — v2→v3 regression hunt (diff -u v2 archive → v3)

13 hunks. Every hunk audited: all trace to a round-3 resolution-map row or §7
bookkeeping; no v2-correct content lost or contradicted.

| # | v2→v3 change | R3 row | Traceable? | Lost/contradicted v2-correct content? |
|---|---|---|---|---|
| 1 | Status line v2→v3 + 'round 4' | bookkeeping | yes | no |
| 2 | TL;DR: qwen pinned-snapshot parenthetical + freshness note | E-N1/F-F1 | yes | no (glm row, class split, invariant sentence untouched) |
| 3 | §1.1 F4: invalid-patch: prefix on both quotes; distinguishing sentence; 256-272→256-274; §1.1 rollout paths → 2026/09/14…; 're-verified byte-exact by rounds 2 and 3' | F-F3; E-N7; E-N3 (§1.1 half) | yes | no — the v2 F4 content (origin, three examples, mis-bucket note) all retained |
| 4 | §1.2: Snapshot bullet; qwen row 31 VERIFY/1 OTHER → 30/2; glm breakdown 13→17 with sub-split; 'disclosed in full; v3' | E-N1/F-F1; E-N2/F-F2 | yes | no — the 36→37 denominator note and the full recount-history preamble retained. (The Method-line path change in this hunk is the H1 defect: v2's 2026-09-{13,14,15} became 2026-09/{13,14,15}/ — still not the on-disk 2026/09/{13,14,15}.) |
| 5 | Recount history: seat-D row reworded (glm snapshot-stable), new round-3 E/F row, new Freshness row; Conclusion gains '(as of the pinned snapshot)' | E-N1/F-F1 | yes | no — v0/seatA/v1 bullets retained verbatim |
| 6 | §3.1 note: auto-chunk range 316-360 → 307-347 | E-N7 | yes | no (rest of the note untouched) |
| 7 | §3.1 note: 380 → 401 words with v3 parenthetical | E-N6/F-F6 | yes | no |
| 8 | §3.3 intro: blanket prefix claim → per-site prefix status | E-N4/F-F5 | yes | no (all four sites' new-text strings untouched) |
| 9 | §3.3.4: 256-272 → 256-274 | E-N7 | yes | no |
| 10 | §3.3 red state: 5 → 7 enumerated; 'Total red assertions: 13' added | E-N5/F-F4 | yes | no (the other three red tests' descriptions untouched) |
| 11 | T4.2: responses.rs ranges 1030-1038 → 1025-1028 and 1030-1035 | E-N7 | yes | no (the scaffold description untouched) |
| 12 | §6: 2026-09-15 → 2026-09-13..15 with pinned-snapshot + 09-15-alone qualifiers | F-F7 | yes | no |
| 13 | §7: R2-map v3 notes (D-M1, C-N1/D-m1, C-N4/D-m3); round-3 summary; R3 map; round-4 pending line | the map itself | yes | no — every v2 map row retained, notes appended rather than replacing |

Structural preservation checks (beyond the hunks):
- §3.2 P2 edge list (lossy + case, silent-partial-apply class, End-Patch truncation,
  zero-content, zero-chunk, env-id, CRLF, structural-line list): byte-identical v2→v3.
- §3.4 invariants (no Update leniency, no existence check, no strict:true,
  base-instructions scope, no auto-retry, no heredoc extension, no per-model modes,
  the exact request-bytes invariant with the P3.4 pre-pass-only refinement):
  byte-identical v2→v3.
- T-plan items T1.1-T1.13, T2.4, T3.1-T3.2, T4.1, T4.2b, T5: untouched by v3 edits.
- §2 root cause, §5 verification plan, §6 rollout plan (except hunk 12): untouched.
- The P1 code block (tool description + argument description) is byte-identical
  v2→v3, so the round-3 seats' P1 checks (drift substrings, char/word counts, Example
  probe) carry over — and I re-verified them on the v3 text anyway (Mandate 2).
- R1 and R2 maps: retained with v2 notes; the R2-map D-M2 row's stale range is H2.

## Findings

### H1 [Minor] — E-N3 fix not landed in §1.2; R3 map records the wrong layout as the actual one

**Where:** §1.2 Method line (spec line 134); §7 round-3 seat-E summary (line 884);
§7 R3 resolution map, E-N3 row (line 904).

**Evidence:** The actual session-dir layout, verified on disk this session, is
~/.codex/sessions/2026/09/{13,14,15}/ (directories 2026/09/13, 2026/09/14,
2026/09/15 exist with 83/162/104 rollout files at review time; there is no
~/.codex/sessions/2026-09/ directory). v3 fixed the header (line 5) and all three
§1.1 example paths (lines 124/126/128) to that layout — but §1.2's Method line, the
exact line seat E's N3 quoted as wrong, now reads 'every rollout under
~/.codex/sessions/2026-09/{13,14,15}/' — a third notation, still not a real path.
Worse, the R3 resolution map states the paths were 'unified to the actual
~/.codex/sessions/2026-09/{13,14,15}/ layout' and the round-3 summary quotes seat E's
finding as 'vs the actual 2026-09/{13,14,15}/ layout' — both recording the wrong form
as the actual one. Seat E's actual finding text reads: 'The actual layout is
~/.codex/sessions/2026/09/{13,14,15}/rollout-*.jsonl (verified on disk)'.

**Impact:** A re-verifier following the §1.2 Method literally (it is the documented
method for re-deriving the evidence table, and the pin's file counts point at the same
dirs) will not find the sessions at the cited path; the resolution-map claim that E-N3
is fixed is not true for §1.2, and the map's 'actual layout' citation would mislead a
future round the same way N3 misled round 3. No design, test-plan, or red-state
content is affected — implementer impact is limited to evidence re-verification, which
is why this is Minor rather than Major (matching the severity of the original N3).

**Suggested resolution:** §1.2 line 134 → '~/.codex/sessions/2026/09/{13,14,15}/';
correct the 'actual layout' citations in the E-N3 resolution row and the round-3
summary to 2026/09/{13,14,15}.

### H2 [Nit] — R2-map D-M2 row retains the pre-correction responses.rs range with no v3 note

**Where:** §7 round-2 resolution map, D-M2 row (spec line ~861).

**Evidence:** The row records the scaffold as 'ev_apply_patch_function_call
(responses.rs, pattern :933-942/:1030-1038)'. The live T4.2 text (corrected this
round per E-N7) cites 'responses.rs:1025-1028 and :1030-1035', which is exactly what
the tree has (ev_exec_command_call_with_args :1025-:1028;
ev_apply_patch_exec_command_call_via_heredoc :1030-:1035 — both opened and verified).
Sibling R2 rows that v3 corrected each received a '(v3: …)' note (D-M1, C-N1/D-m1,
C-N4/D-m3); this row got none, so the map now carries a range contradicted by the
live text it documents.

**Impact:** cosmetic inconsistency in the historical map; an implementer working from
T4.2's live text is unaffected. Pure map hygiene.

**Suggested resolution:** append '(v3: ranges corrected per E-N7 — :1025-1028 and
:1030-1035)' to the D-M2 row, or explicitly mark the row as historical.

## Re-verified clean (appendix)

1. **Red-state list exact** — 13 assertions (3 streaming :828/:838/:848 + 1 CLI
   tool.rs:393 + 2 parser.rs:281/:287 + 7 parser.rs:579/:594/:609/:624/:628/:635/
   :639-642); every line opened; total matches the spec.
2. **No other red** — repo-wide rg for every changed string: the only other hits are
   the untouched streaming sources (:168/:184/:374), the green streaming assertions
   (:819/:784/:797), the green substring test (apply_patch_cli.rs:707, StartedPatch
   arm), unrelated user-verification strings, and no pre-existing assertion of the
   P3.2/P3.3 new texts.
3. **Surviving tests** — apply_patch_cli.rs:689-711 (both substring assertions green
   under P3.1: the patch hits the StartedPatch arm, whose sentence is kept verbatim);
   the three green streaming boundary tests target messages P3.4 leaves untouched.
4. **P2 code shape** — AddFile arm: structural check first (handle_hunk_headers_and_
   end_patch on the trimmed line), + branch untouched (no-trim content), single final
   Err to remove; 'verbatim' and '~8 lines net' claims match the arm.
5. **P3.4 reachability** — check_start_and_end_lines_strict is the single owner of
   both boundary strings; parse_patch (PARSE_IN_STRICT_MODE=false → Lenient) always
   runs it, directly or after heredoc-strip; the full caller set is exactly
   apply_patch.rs:411, lib.rs:370, invocation.rs:116/:123/:170/:175 (+ test callers).
6. **T2.1 drift guard** — 7/7 substrings literal in the v3 argument description;
   126 + 2,198 = 2,324 chars and 21 + 380 = 401 words re-measured, exact.
7. **T2.2 extraction + example** — unique Example: line (index 19); remainder ends
   with *** End Patch; unmodified-binary replay byte-exact (probe 1); the
   naive-substring caveat is accurate (first sentence contains the marker phrase).
8. **F1 probe** — unmodified binary rejects the raw F1 patch at line 3 with the exact
   harness message (em dash intact), exit 1, no file written (probe 2).
9. **Context exactness** — fn main() { does not anchor the fn main context (probe 3);
   T2.2's scratch-context guidance is correct.
10. **T3/T4 scaffolds** — invocation_for_payload (apply_patch_tests.rs:45),
    make_session_and_context (session/tests.rs:5882), with_config (test_codex.rs:347),
    default provider 'OpenAI' (built-in clone :839-:846, assigned :858),
    ev_function_call (:933-942), pattern helpers (:1025-1028, :1030-1035),
    mount_apply_patch (:228, custom_tool_call-only), matches_kind Function-only
    (:603-605), registry rejection (message :550), ResponseMock.single_request/requests
    (responses.rs:39-58) — all exist; T4.2's helper signature fits the
    apply_patch_responses fn(&str, &str) -> Value slot; the capability gate
    (spec_plan.rs:1257-1271 + provider.rs:372) confirms only the Function handler
    registers for renamed non-OpenAI providers.
11. **P3.3 site** — handle_call :508-560; the absent/non-string collapse is exactly
    get('patch').and_then(Value::as_str).ok_or_else(...) with the backtick-quoted
    message at :539; no test asserts it.
12. **Grammar claims** — apply_patch.lark read verbatim: add_line '+' /(.*)/ LF,
    add_hunk … add_line+ (zero-content Add-File not admitted), change (…)+ eof_line?
    with update_hunk … change? (zero-chunk move-only admitted) — the spec's
    parser-vs-grammar delta notes match the grammar text.
13. **Scenario suite** — 24 fixtures, none asserts stderr/exit status, no raw
    Add-File content lines → T1.13 'pass unmodified' holds for P2/P3.
14. **§1.2 arithmetic** — qwen row sums to 413 (374+5+30+1+1+2); glm breakdown sums
    to 26 (17+2+5+2; 10+3+3+1=17); TL;DR 19+7 consistent; 13/21 ≈ 62%; 6/439 ≈ 1.4%.
15. **Pinned-snapshot spot checks** — 09-13/09-14 file counts still 83/162 on disk;
    the two OTHER call_ids (call_30690924…, call_096536c4…) present in the 09-15
    rollouts with 'multiple operations target <path>' rejections; F1 args JSON 24,230
    chars / patch 23,972 chars / line 3 em-dash exact (re-derived from the rollout).
16. **§1.1 F3 mechanism** — the glm47-parser empty-args mechanism (vLLM #49248, the
    parser returning {} when the model omits the opening <arg_value> tag) is as
    characterized in docs/vllm-glm-toolcall-research.md and the round-3 reports; the
    single observed F3 was on qwen, so the 'glitch' classification stands.
17. **base_instructions example** — default.md:132 is the legacy {"command": [...]}
    apply_patch line with literal backslash-n and Update-file prefixes only, as
    claimed in §2.1/§3.4.
18. **justfile executability** — fmt, fix *args, test *args (nextest) recipes exist at
    the repo root (working-directory codex-rs); codex-apply-patch / codex-core /
    codex-model-provider crates exist; T5's scoped commands are valid as written.
19. **v2→v3 preservation** — §3.2 edge list, §3.4 invariants, T-plan items (except
    the corrected T4.2 ranges), R1/R2 maps, §2/§5/§6 bodies: no v2-correct content
    lost or contradicted (Mandate 4).
20. **r3 fix verification** — 8 of 9 findings fully landed (Mandate 0 table); E-N3
    partial (H1); the v3-corrected ranges (307-347, 256-274/:268/:271, :1025-1028/
    :1030-1035) are all exact against the current tree.

## Counts

- Findings: **0 Blocking, 0 Major, 1 Minor (H1), 1 Nit (H2)** — verdict APPROVED (as
  in the header).
- Mandate 0 (r3 fixes): 9/9 verified — 8 LANDED, 1 PARTIAL (E-N3 → H1).
- Mandate 1 (red state): list exact — 13 red assertions enumerated and line-verified;
  surviving tests (apply_patch_cli.rs:707 substring; streaming :819/:784/:797) green;
  repo-wide sweep found no other assertion of any changed string.
- Mandate 2 (implementability): T1–T5 all executable as written; 3/3 mandated probes
  PASS on the unmodified binary; 7/7 drift substrings literal; extraction rule works.
- Mandate 3 (line refs): 31 distinct citations verified; 0 wrong (two benign range
  starts one-to-two lines early — registry.rs:548-556, lib.rs:505-528 — neither
  load-bearing, both accepted by round-3 seats).
- Mandate 4 (regression): 13/13 hunks trace to the R3 map or bookkeeping; structural
  preservation checks clean; one stale historical cite (H2).

