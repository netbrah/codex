# Item 1 (P2 Add-File leniency) — review round 1, SEAT A (apex-ayl.52)

- **Item:** Item 1 — P2: Add-File leniency in the streaming parser (spec §3.2;
  breakdown v3 "Item 1"; spec T1 sub-cases T1.1–T1.3, T1.6–T1.11, T1.13).
- **Files under review (item diff only):**
  - `codex-rs/apply-patch/src/streaming_parser.rs` (module doc, AddFile arm, test-module
    declaration, ledger #2 assertion rewrite)
  - `codex-rs/apply-patch/src/parser.rs` (exactly one module-doc line — documented,
    spec-mandated deviation folded in by coordinator)
  - `codex-rs/apply-patch/src/streaming_parser_p2_tests.rs` (NEW, 16 tests)
  - `docs/reviews/impl-item1-tdd-evidence.md` (worker evidence, audited)
- **HEAD:** `197ea1642c` on `feat/normalize-content-types-vllm`; working tree = HEAD +
  pre-existing seam changes (out of scope, confirmed untouched by this item) + item diff.
- **Seat:** A — lead lens SPEC CONFORMANCE and parser semantics (fresh, independent,
  leaf; no subagents). Full brief verified regardless of lens split.
- **SoT:** spec v5 (LOCKED) `docs/responses-compat-apply-patch-format.md` §3.2/§3.4/§4;
  breakdown v3 `docs/responses-compat-apply-patch-task-breakdown.md` (Item 1, §1, §4, §5).

## VERDICT

**APPROVED — 0 Blocking, 0 Major, 0 Minor, 1 Nit** (A-N1, cosmetic; non-blocking).

Every clause of spec §3.2 re-derived at source and matches the implementation; all
§3.4/§5 invariants hold; the ledger state is exactly as the breakdown requires; every
in-scope T1 sub-case has a genuine pinning test (16/16, assertions read, not just names);
both item gates re-run by this seat and green (115/115; clippy clean); the red/green
evidence is internally consistent and shows no signs of fabrication.

## Method

1. Read SoT first: spec v5 §3.2 (exact semantics + implementation note), §3.4
   (decisions + invariant), §4 (T1 sub-cases); breakdown v3 §1 (red-state ledger),
   Item 1 section (scope/TDD/Gates/DoD), §4 (protocol + review brief), §5 (invariants).
2. Established scope from `git status` / `git diff`: exactly the four files above;
   `git status --short codex-rs/apply-patch/` shows only `parser.rs` +
   `streaming_parser.rs` modified and the new test file untracked; `tests/` (incl. the 25
   scenario fixtures) is byte-untouched; no P2-related content leaked into any other
   working-tree diff (grep of all tracked diffs for "lenient add-file" / "verbatim as
   file content" / "streaming_parser_p2" hits only the two apply-patch source files).
3. Re-derived §3.2 clause by clause against the current source (arm at
   `streaming_parser.rs:206-222`, `handle_hunk_headers_and_end_patch` at :92-145,
   `push_delta` at :147-160, `finish` at :162-181) — see Mandate 1.
4. Cross-checked pre-change line references via `git show HEAD:...` (arm :198-214,
   header check :199, `+` branch :202-208, trailing Err :209-214; ledger fn :814 with
   asserts at :828/:838/:848) — all match the breakdown and the evidence pre-check.
5. Read all 16 assertions in the new test file and verified each pins the claimed
   behavior (deep whole-object equality; exact content strings; exact error strings +
   line numbers).
6. Re-ran both item gates myself from `codex-rs/` (plus a P2-filtered run):
   `just test -p codex-apply-patch` → 115/115; `just fix -p codex-apply-patch` → clean.
7. Audited `docs/reviews/impl-item1-tdd-evidence.md`: red capture, per-sub-case green
   captures, gate summaries, final diff stat; cross-checked its numbers against the
   actual tree (byte-exact diff of the parser.rs line; hunk arithmetic 21+/11-).

Constraints honored: read-only on the repo except this report; no commits/stashes/
checkouts; pre-existing seam files never touched; `cargo test` never invoked directly;
no `--all-features`; Rust target-lock contention with the concurrent seat handled by
waiting, never by killing.

## Mandate 1 — Spec §3.2 semantics, clause by clause (re-derived at source)

Current-tree line refs: `AddFile` arm `streaming_parser.rs:206-222` (header check :207,
`+` branch :210-216, new lenient branch :217-221, `Ok(())` :221);
`handle_hunk_headers_and_end_patch` :92-145 (`END_PATCH_MARKER` trimmed match :110);
`push_delta` :147-160 (CRLF strip :151); `finish` :162-181.

- **Raw line → verbatim content (raw line, then `\n`)** — new branch :217-220 pushes
  `line` (the RAW line, not `trimmed`; only the single trailing `\r` was stripped by
  `push_delta` :151) + `'\n'`. Confirmed for the three named shapes: empty line (raw
  `""` → `"\n"`; T1.2 test pins `"a\n\nb\n"`), whitespace-only (`"   "` → `"   \n"`;
  T1.10 pins it), and `*** `-prefixed-no-marker (`*** Note: keep this line` → verbatim;
  T1.7 pins it, since `trimmed` fails every marker check at :92-145 and falls through).
  ✓ Matches spec: "appended verbatim as content (raw line, then `\n`)".
- **`+`-prefixed → prefix-stripped** — the `+` branch (:210-216) is byte-identical to
  pre-change (diff shows it as context only); it strips the prefix from the RAW line and
  preserves everything after it, so leading spaces survive. Lossy case `++44 …` →
  `+44 …` is the pre-existing canonical semantics, untouched. ✓ (T1.9 pins
  `++42 20 7946 0958` → `"+42 20 7946 0958\n"`, exactly the spec §3.2 probe string.)
- **`*** End Patch` / whitespace-padded → structural** — `handle_hunk_headers_and_end_patch`
  runs FIRST in the arm (:207) and compares the TRIMMED line to `END_PATCH_MARKER` (:110),
  so surrounding whitespace stays structural (pre-existing). Leading-pad pinned by
  `test_p2_add_file_whitespace_padded_end_patch_is_structural` (`"   *** End Patch"`);
  trailing-pad pinned by unmodified golden fixtures `018_whitespace_padded_patch_markers`
  (`*** End Patch ` — trailing space) and `020_whitespace_padded_patch_marker_lines`
  (` *** End Patch` / `*** Begin Patch `), both green in the 115/115 run. ✓
- **Real headers inside Add-File → structural** — `*** Add File:` / `*** Delete File:` /
  `*** Update File:` all match inside `handle_hunk_headers_and_end_patch` (:114-144)
  regardless of the current mode (each pushes its hunk and returns `true` → early return
  at :208 before any content branch). `test_p2_add_file_real_headers_inside_add_file_remain_structural`
  pins all three plus `*** End Patch` in one patch, asserting the full 4-hunk vector with
  deep equality (including the Update chunk's `old_lines`/`new_lines`/
  `context_line_indices`). ✓
- **Raw `*** End Patch` as LAST content line → truncates** — structural check precedes
  content branches, so an unprefixed final `*** End Patch` ends the patch and is not
  content. `test_p2_add_file_last_line_raw_end_patch_truncates` pins contents
  `"line1\n"` for `line1\n*** End Patch\n`. ✓ (Matches spec: "truncated at that line
  (structural check runs before content branches) — a lenient-form-only loss".)
- **Canonical `+*** End Patch` → content verbatim** — for line `+*** End Patch`,
  `trimmed` is `"+*** End Patch"` (≠ marker, so not structural) and the `+` branch takes
  the raw line, yielding content `*** End Patch`. `test_p2_add_file_last_line_canonical_end_patch_is_content`
  pins contents `"line1\n*** End Patch\n"` — byte-identical to the spec §3.2 probe on the
  unmodified binary. ✓
- **Zero-content Add-File accepted** — the `*** Add File:` header unconditionally pushes
  `AddFile { contents: String::new(), .. }` (:114-121); an immediately-following marker is
  structural, leaving an empty-contents hunk. That path is untouched by the diff (the
  changed code is only the arm's residual branch, reached solely by non-structural,
  non-`+` lines). Accepted pre- and post-P2, per spec (probed on the unmodified binary;
  not a T1 sub-case, so no new test is required by the spec — none missing). ✓
- **CRLF handling unchanged path** — `push_delta` :151 (`strip_suffix('\r')` truncate,
  one trailing `\r` per line) is untouched by the diff; `test_p2_add_file_crlf_raw_lines_are_lf_equivalent`
  pins a full-CRLF raw patch → LF content `"raw line\n"`. ✓
- **Update-File and Delete-File arms UNTOUCHED and still strict** — the diff contains no
  hunk in either arm (DeleteFile :223-233, UpdateFile :234-375 unchanged); the Update
  arm's trailing rejection at :369-374 still emits the pre-existing message, pinned
  byte-exact by T1.11 (see Mandate 4); the DeleteFile arm still returns the same
  `InvalidHunkError` for content lines (ledger #3, untouched, green). ✓
- **Removed trailing `Err` — no other path relied on it** — pre-change, the arm's
  trailing `Err(InvalidHunkError "not a valid hunk header …")` was at :209-214 (verified
  via `git show HEAD`); it fired exactly for non-structural, non-`+` lines in AddFile
  mode. The identical message string still exists — unchanged — in the StartedPatch arm
  (pre-change :193) and the DeleteFile arm (pre-change :222). The only test asserting the
  AddFile-arm instance was ledger #2 (rewrite mandated by spec T1.6/ledger row 2). After
  the change, the residual is fully covered by the lenient branch + `Ok(())` (:217-221),
  so the spec's "unreachable after the two branches" reading holds: no error-producing
  path was lost (the arm's remaining errors are only `?`-propagated from the header
  handler, which in AddFile mode cannot fire the Update-emptiness checks since the last
  hunk is an AddFile). The 115/115 green run confirms nothing else depended on it. ✓

## Mandate 2 — Invariants (breakdown §5 + spec §3.4)

- **Canonical patches parse byte-identically** — the `+` branch, the header handler, the
  other arms, `push_delta`, and `finish` are all untouched by the diff; the only behavior
  change is the arm's residual branch (previously an error). The 25 golden scenario
  fixture dirs under `codex-rs/apply-patch/tests/fixtures/scenarios` are UNMODIFIED
  (`git status --short codex-rs/apply-patch/tests/` → empty; 25 numbered dirs +
  README.md confirmed) and `suite::scenarios::test_apply_patch_scenarios` (runs all 25)
  PASS in my gate run. ✓
- **No existence check added anywhere for Add-File** — the diff adds no filesystem,
  path-existence, or `fs` code of any kind; the new branch only appends to a `String`.
  The overwrite outcome for existing paths remains the untouched apply layer (pinned by
  unmodified fixture `011_add_overwrites_existing_file` and
  `test_apply_patch_cli_add_overwrites_existing_file`, both green in my run). ✓
- **No Update-File leniency** — UpdateFile arm unchanged (Mandate 1); T1.11 pins the
  rejection. ✓
- **No error-message text changed elsewhere** — the diff removes exactly one message
  instance (the AddFile-arm trailing one the spec mandates removing) and adds none;
  every other message in the crate is byte-identical (verified by diff scope: the four
  streaming_parser.rs hunks and the one parser.rs doc line). ✓
- **OpenAI request bytes unchanged** — the item diff touches nothing outside
  `codex-rs/apply-patch/src` + the docs file. All other working-tree modifications
  (codex-api `content_type_compat*`/`responses.rs`, core `handlers/*`/`spec_plan.rs`,
  model-provider, models-manager, AGENTS.md) are pre-existing seam state: I inspected
  their diffs (content-type normalization/translation; function-tool seam) and grepped
  every tracked diff for P2 content — zero hits outside the two apply-patch source
  files. This item changes no request construction, no tool spec, no wire format. ✓
- **No new public API surface** — the new module is `#[cfg(test)]`-gated and private; no
  `pub` items, no `Cargo.toml`/lock/BUILD changes (`git status` for the crate shows only
  the two source files + the new test file). ✓

## Mandate 3 — Ledger state

Pre-change refs verified via `git show HEAD:codex-rs/apply-patch/src/streaming_parser.rs`:
fn `test_streaming_patch_parser_returns_errors` at :814; ledger assertions at :828 (#1,
StartedPatch non-header), :838 (#2, Add-File `bad`), :848 (#3, Delete-File `bad`) —
exactly the breakdown §1 line refs.

- **Ledger #2 rewritten correctly** — the only rewritten assertion (now
  `streaming_parser.rs:845-852`) expects `Ok(vec![AddFile { path: PathBuf::from("file.txt"),
  contents: "bad\n".to_string() }])` for `push_delta("*** Begin Patch\n*** Add File:
  file.txt\nbad\n")`. This matches breakdown §1 ledger row 2 ("Add-File raw line (e.g.
  `bad`) → **`Ok`**, content `bad\n` (behavior removed by P2; §3.2)") and spec T1.6
  verbatim. ✓
- **Ledger #1 UNTOUCHED** — still asserts the OLD message
  `"'bad' is not a valid hunk header. Valid hunk headers: '*** Add File: {path}', '*** Delete File: {path}', '*** Update File: {path}'"`
  with `line_number: 2` for the StartedPatch case (now :836-843). Not in the diff; green
  in my run; turns red only when Item 3 extends that message (§3.3.1). ✓
- **Ledger #3 UNTOUCHED** — still asserts the same OLD message with `line_number: 3` for
  the Delete-File case (now :854-862). Not in the diff; green in my run; Item 3 owns its
  rewrite (§3.3.2). ✓
- **Surviving asserts** — the boundary-message asserts at pre-change :784/:797/:819
  ("The last line of the patch must be '*** End Patch'" / "The first line of the patch
  must be '*** Begin Patch'") are outside all diff hunks and green in my run. ✓

## Mandate 4 — Test coverage vs spec T1 sub-cases

`streaming_parser_p2_tests.rs` (334 lines, module declared at
`streaming_parser.rs:389-391` as `#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"]
mod streaming_parser_p2_tests;` — correctly NOT named `tests`, per breakdown Item 1
step 6). 16 tests; every in-scope T1 sub-case covered; assertions read individually:

| T1 sub-case | Test(s) (file:line) | Pin verified |
|---|---|---|
| T1.1 F1 raw markdown, byte-identical to canonical | `test_p2_add_file_raw_markdown_f1_replay_matches_canonical` (:10) | Deep whole-vector equality of raw vs canonical parse (`assert_eq!(raw_hunks, canonical_hunks)`) PLUS equality against the exact content string (`# H1`, blank lines, markdown table). The canonical body is built as `+`-prefixed lines incl. a bare `+` for the empty line — a correct canonical encoding. Genuine pin. |
| T1.2 empty line | `test_p2_add_file_empty_line_becomes_empty_content_line` (:42) | Contents exactly `"a\n\nb\n"` (empty line preserved as empty line). |
| T1.3 mixed lines | `test_p2_add_file_mixed_prefixed_and_raw_lines_concatenate` (:59) | `+hello`/`world`/`+there` → exactly `"hello\nworld\nthere\n"` (order preserved, `+` stripped only from prefixed). |
| T1.7a `*** ` no-marker line | `test_p2_add_file_raw_stars_line_matching_no_marker_is_content` (:76) | `*** Note: keep this line` → verbatim content. |
| T1.7b real headers structural | `test_p2_add_file_real_headers_inside_add_file_remain_structural` (:93) | One patch with `*** Add File: b.txt`, `*** Delete File: d.txt`, `*** Update File: u.txt` inside Add-File; asserts the full 4-hunk vector with deep equality incl. `UpdateFileChunk { change_context: None, old_lines: ["z"], new_lines: ["z"], context_line_indices: [(0,0)], is_end_of_file: false }`. |
| T1.7c padded End Patch | `test_p2_add_file_whitespace_padded_end_patch_is_structural` (:131) | Leading-padded `"   *** End Patch"` structural; `push_delta` result AND `finish()` both asserted. Trailing-pad variant additionally pinned by golden fixtures 018/020 (unmodified). |
| T1.8 typo'd marker swallowed | `test_p2_add_file_typoed_structural_line_swallowed_as_content` (:154) | `*** Ad File: x` → content; `push_delta` AND `finish()` both assert contents `"*** Ad File: x\nreal content\n"` (rest of patch applies — pins the §3.2 silent-partial-apply class). |
| T1.9a lossy `+` strip | `test_p2_add_file_raw_plus_prefixed_line_strips_leading_plus` (:179) | `++42 20 7946 0958` → exactly `"+42 20 7946 0958\n"` (spec §3.2 probe string). |
| T1.9 existing-path half | `test_p2_add_file_to_existing_path_parses_without_existence_check` (:197) | Parser-level: raw Add-File to an existing path parses as-is into the AddFile hunk (no existence check). The apply-layer OVERWRITE outcome is pinned (unmodified) by fixture `011_add_overwrites_existing_file` (green in my run) and core-suite `apply_patch_cli_add_overwrites_existing_file` per the spec §3.2 explicit deferral to T4.2b (Item 4) — absence of an apply-level raw test here is therefore NOT a gap (per brief). |
| T1.9b-canonical final-line variant | `test_p2_add_file_last_line_canonical_end_patch_is_content` (:216) | `+*** End Patch` → content `"line1\n*** End Patch\n"` (matches spec probe on the unmodified binary). |
| T1.9b-raw final-line variant | `test_p2_add_file_last_line_raw_end_patch_truncates` (:235) | Unprefixed final `*** End Patch` → contents `"line1\n"` (truncation pinned). |
| T1.10a whitespace-only line | `test_p2_add_file_whitespace_only_line_is_whitespace_content` (:252) | Line `"   "` → exactly `"   \n"` (no trimming). |
| T1.10b Environment ID in content | `test_p2_add_file_environment_id_line_is_content` (:269) | `*** Environment ID: env1` → verbatim content AND `parser.environment_id() == None` (env-id honored only in StartedPatch). |
| T1.10c unclosed patch | `test_p2_add_file_unclosed_patch_finish_error_unchanged` (:288) | `finish()` → `Err(InvalidPatchError("The last line of the patch must be '*** End Patch'"))` — byte-identical to the pre-change message (`finish` untouched by the diff). |
| T1.10d CRLF | `test_p2_add_file_crlf_raw_lines_are_lf_equivalent` (:303) | Full-CRLF patch → LF-equivalent contents `"raw line\n"`. |
| T1.11 Update stays strict | `test_p2_update_file_raw_unprefixed_lines_still_rejected` (:319) | Raw `bad` inside Update File → `Err(InvalidHunkError { message: "Unexpected line found in update hunk: 'bad'. Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)", line_number: 3 })`. The message string was compared against the pre-change arm text (arm untouched by the diff; message string in the arm at :371 is byte-identical to the assertion) — message UNCHANGED, as required. |
| T1.13 golden scope | (no new test — regression lock per spec) | 25 scenario fixture dirs unmodified (`git status` empty on `tests/`); `test_apply_patch_scenarios` + canonical parser tests green in my 115/115 run. |

Sub-cases T1.4/T1.5/T1.12/T1.14 are Item 3's (P3) by the breakdown's item→sub-case
mapping and are correctly absent here (ledger #1/#3 and
`test_apply_patch_cli_rejects_invalid_hunk_header` all untouched and green — verified in
`tests/suite/tool.rs` being outside the diff).

No T1 sub-case lacks a genuine pinning test; no test asserts less than its claim.

## Mandate 5 — Implementation detail judgment

New branch (`streaming_parser.rs:217-221`):
`if let Some(AddFile { contents, .. }) = self.state.hunks.last_mut() { contents.push_str(line); contents.push('\n'); } Ok(())`

- **State-machine invariant (AddFile mode ⇒ last hunk on the stack is an AddFile):**
  HOLDS. `StreamingParserMode::AddFile` is set in exactly one place — the
  `*** Add File:` branch of `handle_hunk_headers_and_end_patch` (:114-121), which
  first pushes an `AddFile` hunk, making it the stack's last. While in AddFile mode the
  arm's first statement is the same header handler (:207); any header/End-Patch match
  pushes a new hunk (or ends the patch) and returns `true` → early return at :208, so the
  content branches below only run when the handler returned `false`, i.e. the stack is
  unchanged. Nothing in the crate ever pops `self.state.hunks` (only `push` /
  `last` / `last_mut`), and the vector is non-empty whenever the mode is AddFile.
  Therefore `last_mut()` is necessarily `Some(AddFile { .. })` at :217 — the `None`
  fall-through is unreachable.
- **Silent no-op vs `expect`:** acceptable, and I record this as a VERIFIED CLEAN
  judgment rather than a finding: (a) it exactly mirrors the pre-existing sibling `+`
  branch pattern (:210-211), so the arm is internally consistent; (b) `expect` would
  turn a (theoretically) unreachable state into a panic inside a STREAMING parser that
  consumes arbitrary model output — silent drop is the safer failure mode; (c) spec §3.2
  mandates neither, and with the invariant holding the two are behaviorally identical.
  No spec or behavioral deviation. (If a future reviewer wants belt-and-braces, an
  `expect` with a message would be equally defensible; that is a style choice, not a
  defect.)

## Mandate 6 — Item gates (re-run by this seat, from `codex-rs/`)

- `just test -p codex-apply-patch` → **`Summary [ 3.886s] 115 tests run: 115 passed,
  0 skipped`** (exit 0). Includes `suite::scenarios::test_apply_patch_scenarios`
  (all 25 golden fixtures, PASS at 115/115), `test_apply_patch_cli_add_overwrites_existing_file`
  (PASS at 89/115), `test_apply_patch_cli_rejects_invalid_hunk_header` (PASS at 106/115 —
  ledger #4's exact-stderr test, correctly still on the OLD message), the untouched
  ledger #1/#3 asserts inside `test_streaming_patch_parser_returns_errors`, and all
  canonical parser tests. 115 = 99 pre-existing + 16 new, matching the breakdown's
  expectation.
- `just fix -p codex-apply-patch` → **exit 0, clippy clean, no fixes applied**
  (`Checking codex-apply-patch … Finished dev profile`, no diagnostics).
- Supplemental (not a gate, extra confidence): `just test -p codex-apply-patch
  streaming_parser_p2_tests` → `16 tests run: 16 passed, 99 skipped`.
- No `cargo test` invocation; no `--all-features`. Concurrency with the other seat's
  gate runs was handled by the Rust target lock (waited; never killed by PID).

## Mandate 7 — Evidence audit (`docs/reviews/impl-item1-tdd-evidence.md`)

- **Red capture for ledger #2 — GENUINE, pre-implementation.** The captured panic
  (`streaming_parser.rs:835:9`, `assertion failed: (left == right)`) shows
  `left` (actual) = `Err(InvalidHunkError { message: "'bad' is not a valid hunk header.
  …", line_number: 3 })` vs `right` (expected) = `Ok([AddFile { path: "file.txt",
  contents: "bad\n" }])` — i.e. the assert expects `Ok` against the OLD arm's actual
  `Err(InvalidHunkError)`, exactly the required red shape. The panic line :835 is
  internally consistent with strict TDD ordering: at red time the tree had the rewritten
  assertion but not yet the module-doc (+8) or arm changes, so the `assert_eq!` sat at
  the pre-change line (:835); after the green-step changes the same assert sits at :845
  (matches the current diff hunk `@@ -834,11 +845,10 @@`). Summary line `11 tests run:
  10 passed, 1 failed, 88 skipped` is consistent with the 99-test pre-existing suite
  filtered on `streaming_parser`.
- **Green captures per sub-case — present and consistent.** Sub-cases 3–11 each show a
  captured PASS with the exact test names that exist in the test file. The skipped-count
  arithmetic is consistent end to end: 99 (pre-existing) → +1 T1.1 (100) → +1 T1.2 (101)
  → +1 T1.3 (102; 3-run shows 99 skipped) → +3 T1.7 (105; 3-run shows 102 skipped) →
  +1 T1.8 (106) → +2 T1.9 (108) → +2 T1.9b (110) → +4 T1.10 (114) → +1 T1.11 (115;
  1-run shows 114 skipped) → final 115/115. No gaps, no impossible counts.
- **Red-state accounting — correct and pre-declared.** The evidence states P2 is ONE
  behavior change with exactly ONE red state (ledger #2) and that sub-cases 3–10 land as
  green locks (their behavior is the consequence of the sub-case 2 implementation),
  while T1.11/T1.13 are regression locks with no prior red, stated explicitly — this
  matches breakdown §4 step 2 ("RED if new behavior, green lock otherwise") and the
  fact that each sub-case's behavior is a logical consequence of the single arm change
  (re-verified at source in Mandate 1).
- **Gate summary lines — present and matching my own runs.** `115 tests run: 115
  passed, 0 skipped`; `just fix` clean; `just fmt` exit 0. My independent re-runs
  (Mandate 6) reproduce the test and fix outcomes.
- **Final `git diff --stat` — accurate.** Evidence's `streaming_parser.rs | 32
  (+21/-11)` matches the current tree exactly, and the hunk arithmetic reconciles
  precisely: module doc +8, arm −6/+5, test-mod declaration +4, ledger rewrite −5/+4
  → 21 insertions / 11 deletions. The later `parser.rs` `+1` doc line is covered by a
  separate "coordinator-directed" section with its own gate re-run (115/115 + clean
  fix), and the added line is byte-identical to what the diff actually contains
  (verified programmatically).
- **Transparency — good.** The evidence flags its own discrepancy (spec §3.2 mandates
  the parser.rs clarifying line, which the breakdown's Item 1 file list did not
  include) and records the coordinator's decision to fold it into this item as a
  documented, spec-mandated deviation — with the exact line quoted and a +1 line-shift
  note for downstream items (I verified the shift: e.g. `test_parse_patch` pre-change
  :277 → :278 in the working tree). The pre-check section's line refs (arm :198-215,
  header :199, `+` :202-208, trailing Err :209-214, P3.1 :193, P3.2 :222, boundary
  :168/:184/:374, ledger :828/:838/:848, fn :814, surviving :784/:797/:819, green
  99/99 pre-tree) all check out against `git show HEAD`.
- **No fabrication indicators.** No test was added after the evidence's claimed order
  (file content matches the claimed tests, names, and behaviors); no capture contradicts
  the final tree; the one deviation is disclosed, spec-sourced, and minimal.

## Findings

**A-N1 (Nit)** — `Where`: `codex-rs/apply-patch/src/streaming_parser_p2_tests.rs` —
5 of the 16 test items (`#[test]` at :153, :178, :215, :251, :318) have no blank line
between the preceding function's closing brace and the attribute, while the other 11 do.
`Evidence`: verified by scanning every `#[test]` line's predecessor (5 show `'}'`, 11
show `''`). `Impact`: purely cosmetic; `just fmt` passes (rustfmt does not enforce
blank lines between items) and CI gates are unaffected; it is only a minor stylistic
inconsistency within the new file. `Suggested resolution`: optional — add the 5 blank
lines in a future drive-by (e.g. the Item 5 pass); not required for this item to
commit.

No Blocking, Major, or Minor findings.

## Re-verified clean

- Spec §3.2 clause-by-clause conformance at source (Mandate 1) — all clauses hold.
- Canonical byte-identity: `+` branch / header handler / other arms / `push_delta` /
  `finish` untouched; 25 golden fixtures unmodified and green.
- No existence check added on any path (parser or otherwise) for Add-File.
- No Update-File leniency; Update rejection message byte-identical (T1.11 pins it).
- No error-message text changed elsewhere; the only removed message is the spec-mandated
  AddFile-arm trailing one; the identical string survives in the StartedPatch and
  DeleteFile arms unchanged.
- OpenAI request bytes untouched: item diff confined to `codex-rs/apply-patch/src`
  (+ docs); all other working-tree diffs are pre-existing seam state, verified by
  content inspection and P2-content grep (zero hits outside the item's files).
- `parser.rs` addition is exactly ONE `//!` module-doc line, byte-identical to the
  evidence quote, placed after the existing final doc line; the grammar listing
  (`add_hunk: "*** Add File: " filename LF add_line+`, :12) is untouched; no `.lark`
  file exists in the crate, so no grammar reconciliation was attempted (spec forbids it).
- Ledger #2 rewritten exactly per breakdown §1 row 2 / spec T1.6; ledger #1/#3 and the
  surviving boundary asserts untouched and green; pre-change line refs (:814/:828/:838/
  :848, arm :198-214) all verified.
- Every in-scope T1 sub-case (T1.1–T1.3, T1.6–T1.11, T1.13) has a genuine pinning test;
  all 16 assertions read and verified (whole-object deep equality; exact strings).
- State-machine invariant (AddFile mode ⇒ last hunk is the AddFile that set the mode)
  proven from source; silent no-op on unreachable `None` judged acceptable (mirrors the
  sibling `+` branch; no panic on streaming input; spec mandates neither).
- New test module follows repo convention (sibling file, `#[path]`, not named `tests`,
  `pretty_assertions::assert_eq`); no new public API; no build-system changes needed.
- Item gates re-run independently: 115/115 tests; clippy clean.
- Evidence doc: genuine red, consistent greens, accurate stats, disclosed deviation —
  no fabrication indicators.

## Counts

| Severity | Count | IDs |
|---|---|---|
| Blocking | 0 | — |
| Major | 0 | — |
| Minor | 0 | — |
| Nit | 1 | A-N1 |
| **Total** | **1** | |

**VERDICT: APPROVED — 0 Blocking, 0 Major** (1 non-blocking Nit recorded).
