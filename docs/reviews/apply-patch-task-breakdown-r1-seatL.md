# Task-Breakdown Cross-Review — Round 1, Seat L

- **Artifact:** `docs/responses-compat-apply-patch-task-breakdown.md` (460 lines, stage 3)
- **SoT spec:** `docs/responses-compat-apply-patch-format.md` (v5) · Companion: `docs/responses-compat-seam.md`
- **Branch / HEAD:** `feat/normalize-content-types-vllm` · Seat L mandate: **implementability and reference accuracy** (a different seat reviews spec-conformance and TDD protocol)
- **Reviewer:** seat L, leaf (no subagents), fresh verification of every claim at HEAD

## Verdict

**CHANGES-REQUESTED — 1 Blocking, 1 Major, 3 Minor, 4 Nit**

The breakdown is 95% execution-ready: 60+ of the line citations are exact at HEAD,
every item's mechanism exists in the tree, the T2.2 example probes byte-exact on the
unmodified debug binary, and all gate commands are real justfile recipes. One item
(Item 2, T2.3) prescribes an insta-snapshot flow against a test that is an inline
`assert_eq!` literal (no insta snapshot exists — the flow dead-ends), and one ledger
row points at a file path that does not exist (`core/tests/suite/tool.rs` vs the real
`apply-patch/tests/suite/tool.rs`). Both are small, mechanical fixes to the breakdown.

## Method

1. **Line-ref sweep (Mandate 1):** extracted every `file:NNN` / `file:NNN-MMM` /
   `:NNN` citation in the breakdown (regex over the artifact, then re-read the
   artifact's ledger/item sections to catch bare `:NNN` refs), opened each location
   at HEAD with `cat -n`, and classified exact / off-by-N (with the symbol's
   position relative to the range) / wrong. Counted the scenario fixtures.
2. **Implementability walkthrough (Mandate 2):** per item, opened every cited
   mechanism (arm shape, test fns, scaffolding signatures, helpers, harness
   builder, gate recipes, operational artifacts). Ran the T2.2 extraction rule
   against the spec §3.1 embedded text and executed the result through the
   unmodified debug binary `codex-rs/target/debug/apply_patch` (built
   2026-09-15 19:25; `codex-rs/apply-patch` git-clean) in a /tmp scratch dir.
   Mechanically counted the §3.1 P1 text (chars/words) and checked all 7
   drift-guard substrings against the new argument description. Read
   `~/bin/wiretap.py` in full to confirm passivity.
3. **Convention audit (Mandate 3):** checked the breakdown's item instructions
   against the repo-root AGENTS.md conventions (inline `format!` args,
   collapsible_if, method refs, no single-reference product helpers,
   `/*param_name*/` literal comments, snapshot policy, integration-test
   conventions, `codex_utils_cargo_bin` for spawned workspace binaries) and for
   under-specification that would cause rework (pre-cleared options in open
   questions are allowed, not findings).
4. Probes ran only in /tmp scratch dirs; the debug binary was used read-only;
   no file was modified other than this report.

Severity scale (per mandate): **Blocking** = item not executable as written
(missing symbol, wrong mechanism, snapshot flow that can't work) · **Major** = a
cited line/file that is actually wrong, or an instruction producing a convention
violation CI rejects · **Minor** = under-specification likely to cause rework or
an off-by-N ref that could mislead · **Nit** = cosmetic.

## Mandate 1 — Line-ref sweep (all citations opened at HEAD)

Status key: **EXACT** = cited line(s) are exactly where the symbol/text is ·
**OK (range)** = range contains the symbol, endpoints slightly loose (see note) ·
**WRONG** = see finding. All refs below were re-verified by this seat; the
breakdown's own claim ("round-4/5 seats' line-ref sweeps: 0 wrong") is not
reproduced — one wrong file path (L-M2) and one wrong count (L-M3).

### `codex-rs/apply-patch/src/streaming_parser.rs`

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `:198-214` | AddFile arm (Item 1 scope) | EXACT | arm spans :198-215 (`}` at :215); sub-cites all exact: header check `handle_hunk_headers_and_end_patch` :199-201, `+` branch :202-208, trailing `Err(InvalidHunkError …)` :209-214 |
| `:193` | P3.1 StartedPatch message | EXACT | :193 = the StartedPatch `InvalidHunkError` message string (block :191-196) |
| `:222` | P3.2 DeleteFile message | EXACT | :222 = DeleteFile arm message string (block :220-225) |
| `:168` | untouched parallel boundary msg (finish) | EXACT | :168 = "The last line of the patch must be '*** End Patch'" in `finish()` |
| `:184` | untouched parallel boundary msg (NotStarted) | EXACT | :184 = "The first line of the patch must be '*** Begin Patch'" |
| `:374` | untouched parallel boundary msg (EndedPatch content) | EXACT | :374 = same End message in the EndedPatch arm |
| `:814` | `test_streaming_patch_parser_returns_errors` | EXACT | fn at :814 (inline `mod tests` at :382-383) |
| `:828` | red assert 1 (StartedPatch 'bad') | EXACT | :828 = message line of the StartedPatch assert (:825-832) |
| `:838` | red assert 2 (AddFile 'bad') | EXACT | :838 = message line of the AddFile assert (:835-842); currently expects `Err(InvalidHunkError { … line_number: 3 })` — the rejection Item 1 will rewrite |
| `:848` | red assert 3 (DeleteFile 'bad') | EXACT | :848 = message line of the DeleteFile assert (:845-852) |
| `:819` | surviving test msg | EXACT | :819 = first-line Begin message (assert :816-821, same fn) |
| `:784` | surviving test msg | EXACT | :784 = End message in finish-error test (:781-786) |
| `:797` | surviving test msg | EXACT | :797 = End message in `test_streaming_patch_parser_rejects_content_after_end_patch` |
| `:307-349` (spec §3.1 note) | auto-chunk branch span | EXACT | :307-316 (empty), :318-327 (' '), :329-338 ('+'), :340-349 ('-') |

### `codex-rs/apply-patch/src/parser.rs`

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `:256-274` | `check_start_and_end_lines_strict` fn | EXACT | fn spans :256-274 exactly |
| `:268` | Begin message | EXACT | :268 = "The first line of the patch must be '*** Begin Patch'" |
| `:271` | End message | EXACT | :271 = "The last line of the patch must be '*** End Patch'" |
| `:193-199` | `parse_patch_text` pre-pass call site | EXACT | fn starts :193; strict/lenient boundary check calls at :195-198, inside range |
| `:57` | `InvalidPatchError` Display | EXACT | :57 = `#[error("invalid patch: {0}")]` |
| `:277` | `test_parse_patch` fn | EXACT | fn at :277 |
| `:281` | missing-Begin assert | EXACT | :281 = message line of assert (:278-283) |
| `:287` | missing-End assert | EXACT | :287 = message line of assert (:284-289) |
| `:402` | upstream "no explicit @@" pin | EXACT (as cited) | :402 = the comment "// Update hunk without an explicit @@ header for the first chunk should parse."; the `assert_eq!` it describes starts :404 |
| `:558` | `test_parse_patch_lenient` fn | EXACT | fn at :558 |
| `:579` | lenient Begin assert (Strict `<<EOF`) | EXACT | :579 = assert of `expected_error` (Begin msg) |
| `:594` | lenient Begin assert (Strict `<<'EOF'`) | EXACT | same |
| `:609` | lenient Begin assert (Strict `<<"EOF"`) | EXACT | same |
| `:624` | lenient Begin assert (Strict mismatched quotes) | EXACT | same |
| `:628` | lenient Begin assert (Lenient mismatched quotes) | EXACT | same |
| `:635` | lenient Begin assert (Strict missing-closing) | EXACT | same |
| `:639-642` | lenient End assert (Lenient missing-closing) | EXACT (range) | assert block :639-644; message string at :642 inside the cited range — 6 Begin + 1 End = the ledger's 7 assertions, composition matches spec §3.3 exactly |

### `codex-rs/core/src/tools/handlers/apply_patch.rs`

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `:508-560` | `FunctionApplyPatchHandler::handle_call` (P3.3 site) | OK (range) | fn spans :508-563; cited range ends at :560, mid `run_apply_patch_text` call args. All P3.3-relevant content (:534-539) is inside the range → L-N1 |
| `:534-536` | collapsed `get("patch").and_then(Value::as_str)` | EXACT | :534-536 = `value` / `.get("patch")` / `.and_then(serde_json::Value::as_str)` (`.ok_or_else` continues :537) |
| `:539` | current single message | EXACT | :539 = `"apply_patch is missing the required \`patch\` argument".to_string(),` (backtick-quoted arg name, as the site-3 re-quote status says) |
| `:411` | handler parse call | EXACT | :411 = `let args = match codex_apply_patch::parse_patch(patch_input) {` in shared `run_apply_patch_text` |
| `:603-605` | `matches_kind` Function-only | EXACT | :603-605 = `fn matches_kind … matches!(payload, ToolPayload::Function { .. })` |

### `codex-rs/core/src/tools/registry.rs`

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `:548-556` | kind-mismatch rejection | OK (range) | the `if !tool.matches_kind(…)` check is at :549 (range starts :548, a `}`); `FunctionCallError::Fatal(message)` at :562 is past the range end :556 → L-N2 |
| `:550` | rejection message | EXACT | :550 = `format!("tool {tool_name} invoked with incompatible payload")` — exact text the breakdown quotes |

### `codex-rs/apply-patch/src/invocation.rs`

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `:116` | shell-intercept parse call (direct) | EXACT | :116 = `[cmd, body] … => match parse_patch(body)` |
| `:123` | shell-intercept parse call (heredoc form) | EXACT | :123 = `Ok((body, workdir)) => match parse_patch(&body)` |
| `:170` | shell-intercept parse call (implicit body) | EXACT | :170 = `&& parse_patch(body).is_ok()` |
| `:175` | shell-intercept parse call (implicit script) | EXACT | :175 = `&& parse_patch(script).is_ok()` |
| `:235-241` | multiple-operations rejection | EXACT | :235-241 = `if changes.contains_key(&path) { … Err(InvalidPatchError("multiple operations target …")) }` |
| `:237` | its message | EXACT | :237 = `"multiple operations target {}",` |
| `:242-245` | no existence check (AddFile arm) | EXACT | :242-245 = `Hunk::AddFile { … } => changes.insert(path, ApplyPatchFileChange::Add { … })` — no existence check anywhere in the arm |

### `codex-rs/apply-patch/src/lib.rs`

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `:370` | CLI parse call (P3.4 surface) | EXACT | :370 = `let hunks = match parse_patch(patch) {` in `apply_patch_with_options` |
| `:374-384` | CLI error formatting | OK (range) | `InvalidPatchError` arm fully inside (:374-377); `InvalidHunkError` arm's message string at :385-386, just past the range end → L-N4 (the breakdown's own Item 3 cites only `:370`) |
| `:505-535` | Add-File apply (overwrite, `A <path>` output) | EXACT | :505-535 = the AddFile hunk arm in the apply loop (resolve, read-optional-for-delta, `try_write!`, `changes.push(Add)`, `added.push`) |

### Other cited files

| Cite | Breakdown use | Status | Evidence at HEAD |
|---|---|---|---|
| `codex-rs/apply-patch/src/file_update.rs:110` | "Failed to find context" | EXACT | :110 = `"Failed to find context '{ctx_line}' in {path}"` |
| `codex-rs/core/src/tools/spec_plan.rs:1257-1271` | capability-gated apply_patch registration | EXACT | :1257-1271 = the `apply_patch_function_tool` gate: `FunctionApplyPatchHandler` (:1267) vs `ApplyPatchHandler` (:1269) |
| `codex-rs/model-provider/src/provider.rs:372` | capability gate | EXACT | :372 = `apply_patch_function_tool: !self.info.is_openai(),` |
| `codex-rs/model-provider-info/src/lib.rs:40` | `OPENAI_PROVIDER_NAME` | EXACT | :40 = `const OPENAI_PROVIDER_NAME: &str = "OpenAI";` |
| `codex-rs/model-provider-info/src/lib.rs:546-547` | `is_openai` name-match, no Azure branch | EXACT | :546-547 = `pub fn is_openai(&self) -> bool { self.name == OPENAI_PROVIDER_NAME }` — no Azure branch, confirming the seam-doc correction Item 5 must make |
| `codex-rs/protocol/src/prompts/base_instructions/default.md:132` | legacy `{"command": […]}` example | EXACT | file located as the spec says; :132 = the `apply_patch` command-array example with literal `\\n` sequences, Update-only prefixes |
| `codex-rs/core/src/tools/handlers/apply_patch_spec_tests.rs:40` | `create_apply_patch_function_tool_matches_expected_spec` | fn name + line EXACT; "full-snapshot" characterization WRONG | :40 = the fn; body :41-60 is an inline `assert_eq!` against a literal `ToolSpec::Function(ResponsesApiTool { … })` — **not** an insta snapshot → L-M1 |
| `codex-rs/core/src/tools/handlers/apply_patch_tests.rs:45` | `invocation_for_payload` | EXACT | :45 = `async fn invocation_for_payload(payload: ToolPayload) -> ToolInvocation` — the signature the T3.1 tests imply (payload in, invocation out) |
| `codex-rs/core/src/session/tests.rs:5882` | `make_session_and_context` | EXACT | :5882 = `pub(crate) async fn make_session_and_context() -> (Session, TurnContext)` |
| `codex-rs/core/tests/common/test_codex.rs:347` | `with_config` | EXACT | :347 = `pub fn with_config<T>(mut self, mutator: T) -> Self where T: FnOnce(&mut Config) + Send + 'static` — can rename `model_provider.name` |
| `codex-rs/core/tests/common/test_codex.rs:839-859` | default provider "OpenAI" (built-in clone) | EXACT (range) | `ModelProviderInfo` literal :839-845 (built-in clone `..built_in_model_providers(/*openai_base_url*/ None)["openai"].clone()` at :844); assigned to `config.model_provider` at :859; built-in name is `OPENAI_PROVIDER_NAME` = "OpenAI" (`create_openai_provider`, model-provider-info lib.rs:463-465) — so the default `test_codex()` provider is named "OpenAI" and takes the freeform path |
| `codex-rs/core/tests/common/responses.rs:39-58` | `ResponseMock` `.single_request()`/`.requests()` | OK (range) | struct :39-41, `single_request` :50-56 inside; `requests` fn starts at :58 with body :59-60 just past the end → L-N3 |
| `codex-rs/core/tests/common/responses.rs:933-943` | `ev_function_call` | EXACT | fn spans :933-943; signature `pub fn ev_function_call(call_id: &str, name: &str, arguments: &str) -> Value` |
| `codex-rs/core/tests/common/responses.rs:1025-1028` | `ev_exec_command_call_with_args` | EXACT | fn spans :1025-1028 (binds serialized args to a local, passes `&arguments`) |
| `codex-rs/core/tests/common/responses.rs:1030-1035` | `ev_apply_patch_exec_command_call_via_heredoc` | EXACT | fn spans :1030-1035, same local-bind pattern |
| `codex-rs/core/tests/suite/apply_patch_cli.rs:228` | `mount_apply_patch` | EXACT | :228 = `pub async fn mount_apply_patch(…)`; it always builds `apply_patch_responses(…, ev_apply_patch_custom_tool_call)` (:236-241) — the breakdown's "always emits `custom_tool_call`" claim is correct |
| `codex-rs/core/tests/suite/apply_patch_cli.rs:323` | `read_file_text` harness method | EXACT | :323 = `assert_eq!(harness.read_file_text(file_name).await?, expected);` |
| `codex-rs/core/tests/suite/apply_patch_cli.rs:689` | surviving substring test (fn) | EXACT | :689 = `apply_patch_cli_rejects_invalid_hunk_header`; its patch (`*** Begin Patch\n*** Frobnicate File: foo\n*** End Patch`) is indeed the StartedPath case, so P3.1 (sentence extension) keeps the asserted substring reachable |
| `codex-rs/core/tests/suite/apply_patch_cli.rs:707` | substring assertion | EXACT | :707 = `out.contains("is not a valid hunk header")` — survives P3.1 because §3.3.1 keeps the old text as an exact prefix |
| `codex-rs/core/tests/suite/tool.rs:386` (ledger row 4) | `test_apply_patch_cli_rejects_invalid_hunk_header` | **WRONG FILE PATH** | `codex-rs/core/tests/suite/tool.rs` does not exist (verified: no `tool.rs` under `core/tests/`; `find` locates exactly one `tool.rs` under tests). The test is at **`codex-rs/apply-patch/tests/suite/tool.rs:386`** (line 386 = the fn), with the exact-stderr assertion at **:393** of that file — the spec §3.3 citation (`codex-rs/apply-patch/tests/suite/tool.rs:386`) is correct; the breakdown's transcription dropped the `apply-patch/` crate → L-M2 |
| "24 scenario fixtures under `codex-rs/apply-patch/tests/fixtures/scenarios`" (ledger surviving-tests + Item 1 T1.13) | golden scope count | **WRONG COUNT** | 25 scenario directories at HEAD (26 entries minus `README.md`; note two dirs share the `020_` prefix: `020_delete_file_success`, `020_whitespace_padded_patch_marker_lines`). The runner (`tests/suite/scenarios.rs`) iterates `read_dir` dynamically, so executability is unaffected → L-M3 |

## Mandate 2 — Implementability walkthrough (per item)

### Item 1 — P2 Add-File leniency: **executable as written** (one Minor)

- The AddFile arm at `streaming_parser.rs:198-214` is the right place and the
  described shape matches the code exactly (header check :199, `+` branch
  :202-208, trailing `Err(InvalidHunkError …)` :209-214).
- The spec §3.2 semantics are implementable inside that arm: keep
  `handle_hunk_headers_and_end_patch` (structural markers stay structural,
  trimmed-line match), keep the `+` branch verbatim, replace the trailing Err
  with a verbatim append (`contents.push_str(line); contents.push('\n')` on the
  raw, untrimmed line — matching "raw line, then `\n`" and the no-trim
  indentation rule). Deleting 6 lines and adding ~4-5 makes the "~8 lines net"
  estimate plausible. No state-machine restructure needed.
- Assertion :838 currently expects the rejection (`Err(InvalidHunkError {
  message: "'bad' is not a valid hunk header…", line_number: 3 })` for
  `*** Begin Patch\n*** Add File: file.txt\nbad\n`) — rewriting it to
  `Ok(vec![AddFile { path: "file.txt", contents: "bad\n" }])` is exactly what
  P2 produces (lossy `++` case, End-Patch truncation, etc. all follow from the
  same two-branch shape).
- Test-file convention — the open question is well-posed, and the facts:
  `streaming_parser.rs` declares its tests **inline** (`#[cfg(test)] mod tests
  {` at :382-383, file is 924 LoC). The same crate also uses the sibling
  convention (`file_update.rs:15-17` → `#[path = "file_update_tests.rs"]
  mod tests;`). AGENTS.md's rule (new test modules → sibling `*_tests.rs` with
  `#[path]`) supports the breakdown's sibling-file choice; but the literal
  instruction `mod tests;` would collide with the existing inline module
  (duplicate name, compile error) — see L-M4.

### Item 2 — P1 function-tool spec text: **mechanism error (Blocking), everything else executable**

- `create_apply_patch_function_tool` exists (`apply_patch_spec.rs:40-67`).
  Current text verified: tool-level description (line 57) = "The
  `apply_patch` tool can be used to edit files. The complete patch goes in the
  `patch` argument as plain text." — the phrase the spec quotes is verbatim in
  this (tool-level) description; the current `patch` *argument* description
  (line 44) is a different, also-short string ("The ENTIRE patch text,
  starting with `*** Begin Patch` … do not wrap it in JSON or markdown.").
  Item 2 replaces **both** (tool desc + arg desc), which matches spec §3.1
  exactly, so the TL;DR's phrasing conflation does not affect execution.
- P1 text counts mechanically verified: tool description **126 chars / 21
  words**, argument description **2,198 chars / 380 words**, total **2,324
  chars / 401 words** — all exactly as the breakdown and spec state. All 7
  drift-guard substrings are present in the new argument description; none is
  present in the current text (so T2.1 is genuinely red at write time).
- The 7 substrings in breakdown Item 2 vs spec §3.1 design note: **identical
  lists, empty diff** (`first line is `*** Begin Patch`` / `real newline
  characters` / `bare '+'` / `at most one hunk per patch` / `starts with '+'`
  / `multiple `@@` chunks` / `must change at least one line`).
- **T2.2 probe — PASS (byte-exact).** Extraction rule run on the spec §3.1
  embedded text: first line exactly `Example:` found; remainder ends with the
  line `*** End Patch`. Executed through the unmodified
  `codex-rs/target/debug/apply_patch` (git-clean crate) in a /tmp scratch dir:
  exit 0; `notes/todo.md` = `# TODO\n\n1. ship the fix\n` **byte-exact**;
  `src/main.rs` update applied as one chunk with `@@ fn main` context — one
  removal (`    old_call();`), one addition (`    new_call();`), one context
  line (`    shared();`) preserved. (First probe run failed only because my
  pre-created file lacked an exact `fn main` line — the context anchor is an
  exact-line match; with the anchor present the result matches the spec's
  asserted hunks.)
- **T2.3 — the prescribed snapshot flow cannot work (L-M1, Blocking).** The
  test at `apply_patch_spec_tests.rs:40` is an inline `assert_eq!` against a
  literal tool spec — no `assert_snapshot!`/`insta::` anywhere in
  `core/src/tools/handlers/`, no `.snap` file for this test (the only
  core-suite snapshot mentioning apply_patch,
  `all__suite__scenarios__astra_settings_release_check_tool_shapes.snap`,
  stores a tool-call transcript, not spec text, and is unaffected by Item 2).
  `cargo insta pending-snapshots` / `cargo insta accept` (insta 1.46.3 is
  installed) would find nothing and leave the test red. Correct mechanism:
  edit the expected literals in the test (tool description at line 45, arg
  description at line 52) to the spec §3.1 text.
- T2.4 (freeform unchanged) is executable: `create_apply_patch_freeform_tool`
  and its two tests are separate; no edits required.

### Item 3 — P3 teachable errors: **executable as written**

Current message text at the four sites (what the executor extends/replaces):

1. **P3.1** `streaming_parser.rs:191-196` (msg :193, StartedPatch arm):
   `"'{trimmed}' is not a valid hunk header. Valid hunk headers: '*** Add
   File: {path}', '*** Delete File: {path}', '*** Update File: {path}'"` —
   §3.3.1 says keep this verbatim as prefix + append the guidance sentence.
2. **P3.2** `streaming_parser.rs:220-225` (msg :222, DeleteFile arm): same
   generic hunk-header text — §3.3.2 is a wholesale replacement with
   `'Delete File' hunks take no content lines; the next line must be another
   hunk header or '*** End Patch'` (no shared prefix — matches "replacement").
3. **P3.3** `apply_patch.rs:534-542`: collapsed
   `value.get("patch").and_then(Value::as_str).ok_or_else(…)` with one message
   (:539, backtick-quoted `patch`). Splitting into absent vs non-string is a
   local restructure of that chain; §3.3.3a/3b strings re-quote the arg name
   with single quotes — matches the site-3 status exactly.
4. **P3.4** `parser.rs:256-274` (msgs :268/:271): the two parser-level
   boundary strings, single owner, reached via `parse_patch_text`
   (:193-199) from the function path (:411), the CLI (`lib.rs:370`), and the
   four shell-intercept calls (`invocation.rs:116/:123/:170/:175`) — all
   verified present. The parallel streaming messages (:168/:184/:374) are
   distinct strings in a different file and stay untouched.

- All four new strings are present in spec §3.3 verbatim and copy-able
  (quoted in the spec body, not just referenced).
- Per-site prefix status (1 extension, 2 replacement, 3 re-quote + split, 4
  pre-pass-only) matches the code's actual behavior — verified above.
- T3.1 scaffolding: `invocation_for_payload` (`apply_patch_tests.rs:45`,
  `ToolPayload → ToolInvocation`) and `make_session_and_context`
  (`session/tests.rs:5882`, `→ (Session, TurnContext)`) exist with the
  signatures the new tests imply.
- T3.2 (handler-level F1-shaped raw Add-File executing against the
  sandboxed test filesystem) is executable with the same scaffolding:
  `handle_call` → `run_apply_patch_text` → parse/verify/execute with the
  session's local FS; P2 (Item 1) is a prerequisite, and the item order
  (1 before 3) provides it.
- Ledger arithmetic checks out: rows 1, 3-13 = 12 assertions for Item 3;
  row 2 = 1 for Item 1; total 13.

### Item 4 — Integration tests: **executable as written** (one Minor)

- `mount_apply_patch` (`apply_patch_cli.rs:228`) indeed always builds a
  `custom_tool_call` (it passes `ev_apply_patch_custom_tool_call` at :240
  into `apply_patch_responses`, :266, which takes the event constructor as a
  `fn(&str, &str) -> Value` parameter — so "apply_patch_responses with a
  function-call variant" is a clean fit: pass a new
  `ev_apply_patch_function_call` function pointer).
- On the renamed non-OpenAI provider only `FunctionApplyPatchHandler` is
  registered (`spec_plan.rs:1267`); `matches_kind` accepts only
  `ToolPayload::Function` (:603-605); a `custom_tool_call` payload hits the
  registry's kind check (:549) and fails with "tool apply_patch invoked with
  incompatible payload" (:550) — the breakdown's rejection chain is exactly
  right, so "do not reuse `mount_apply_patch`" is correct.
- `with_config` (`test_codex.rs:347`) accepts a `&mut Config` mutator;
  provider-name renames are an established pattern in the test suite (8+
  existing sites, e.g. `compact.rs:1313`, `client.rs:2514`,
  `settings_commits.rs:317`) with the same dummy auth — "default dummy auth
  works for any name" is corroborated.
- `apply_patch_cli.rs` already has `apply_patch_harness_with` (:88) taking a
  `TestCodexBuilder` configurator, and `with_config` is used 9 times in this
  file — the T4.1 harness shape is directly implementable.
- `ResponseMock.single_request()/requests()` (`responses.rs:39-58`) expose the
  captured `/v1/responses` bodies for the T4.1 assertion.
- The proposed `ev_apply_patch_function_call` body does not type-check as
  written (String vs `&str`) → L-M5.
- `read_file_text` (`apply_patch_cli.rs:323`) is a harness method, usable as
  claimed for T4.2/2b assertions.
- Item 4 needs **no spawned workspace binary** — all flows are mock-SSE; the
  existing scenario runner's `codex_utils_cargo_bin::cargo_bin("apply_patch")`
  is untouched, so no `codex_utils_cargo_bin` requirement arises for the new
  tests.

### Item 5 — Docs + gates: **executable as written**

- Seam doc `docs/responses-compat-seam.md` contains every named edit target:
  - §2 stale line present at :54 — "Gate semantics: `is_openai()` is a string
    match on the provider `name` field in `config.toml` (`== "OpenAI"`),
    **plus an Azure branch**." (Stale: `is_openai` is a pure name match,
    model-provider-info lib.rs:546-547.)
  - §3 "Design: apply_patch function tool" section (Problem/Decision/
    Components/Testing) — natural home for the P1/P2/P3 remediation layer +
    evidence numbers.
  - Divergence table: §4 "Divergence map" rows 1-6 — one new row for the
    apply-patch parser change fits the existing column shape.
  - §6 conflict sites: the step-2 expected-conflict list — `streaming_parser.rs`
    and `parser.rs` can be added as written.
  - §6 verification gate: the step-3 gate list — `just test -p
    codex-apply-patch` + the raw-format probe can be added as written.
  - Invariants section: "Invariants that must survive every rebase:" at the
    end of §6 — the §3.4 exact-invariant text can be added.
- Gate commands are real recipes in the repo-root `justfile` (`set
  working-directory := "codex-rs"`, so they run from `codex-rs/` as the
  breakdown says): `test *args` (:87, `cargo nextest run --no-fail-fast`),
  `fix *args` (:57, `cargo clippy --fix --tests`), `fmt` (:50),
  `bazel-lock-update` (:144). `just test -p codex-core apply_patch` is valid
  nextest syntax (crate filter + test-name filter). Crate names verified:
  `codex-apply-patch`, `codex-core`, `codex-model-provider`.

### Item 6 — Real-environment verification + bead close: **executable as written**

- `~/bin/wiretap.py` exists (3,243 bytes). Read in full: a stdlib-only
  `BaseHTTPRequestHandler` that logs model + tool names per POST and forwards
  body/headers unchanged to upstream, streaming the response back
  chunked — **passive (log + forward, no injection)**, as claimed.
- `codex-bin-backups/` located at `~/Projects/upstream/codex-bin-backups/`
  (matches the seam doc's `Projects/upstream/codex-bin-backups/` reference);
  contains `codex-combined.bak-20260913` — rollback state exists.
- Standalone `apply_patch` binary target confirmed:
  `codex-rs/apply-patch/Cargo.toml` `[[bin]] name = "apply_patch" path =
  "src/main.rs"`; release build via `cargo build --release -p codex-apply-patch`
  (or the workspace release path) as implied.
- Bead close command (`command bd -C
  /Users/palanisd/Projects/bitbucket/apex_tracking close apex-ayl.52`) matches
  the repo AGENTS.md guidance for non-shell-function environments.

## Mandate 3 — Convention & diff-minimality audit

Checked the breakdown's item instructions against the repo-root AGENTS.md
rules the per-item review brief invokes. Result: **no instruction pushes the
executor toward a convention violation CI would reject.** Detail:

- **Inlined `format!` args / collapsible_if / method refs:** the P2 edit is
  two `push_str`/`push` calls plus the removed Err; P3.1/P3.2 keep the
  existing `format!("'{trimmed}' …")` shape with the arg already inlined;
  P3.3/P3.4 introduce plain string literals. Nothing in the breakdown's
  instructions invites a clippy violation, and the per-item review brief
  re-states the rules.
- **No new small single-reference helpers:** Item 4's two new helpers
  (`ev_apply_patch_function_call`, `mount_apply_patch_function_call`) are
  test scaffolding that each mirror an existing sibling helper
  (`ev_exec_command_call_with_args` / `mount_apply_patch`) and are used by
  ≥2 tests; they are the same shape the file already carries. No new
  *product-code* helpers are introduced by any item.
- **`/*param_name*/` literal comments:** the new tests pass no opaque
  positional literals beyond the patterns already in the files
  (`/*include_environment_id*/ false` style exists in
  `apply_patch_spec_tests.rs`); the review brief covers the rest.
- **Snapshot policy:** AGENTS.md's insta requirement targets user-visible TUI
  UI; Item 2 changes wire spec text, not UI. The only snapshot-adjacent
  problem is the wrong mechanism in T2.3 (L-M1), which is a
  mechanism-accuracy finding, not a policy violation.
- **Integration-test conventions:** Item 4's "prefer `wait_for_event` /
  `mount_sse_once`" step is compatible with the design: T4.1 (request-body
  assertion) can be a single-POST test; T4.2/2b necessarily need the two-SSE
  `apply_patch_responses` shape (mirroring `mount_apply_patch`), and the
  breakdown pre-clears "a parallel `mount_sse_sequence` call" as an option —
  allowed under the mandate (pre-cleared options, not under-specification).
- **`codex_utils_cargo_bin` for spawned binaries:** not required — Item 4
  spawns no workspace binary (mock SSE throughout); the existing scenario
  runner already uses `cargo_bin("apply_patch")` and is left unmodified.
- **Diff minimality:** each item's "Files touched" list is tight and the
  review brief bans drive-by edits; the only cross-file ripples are the
  ledger assertion rewrites (explicitly ledgered) and the Item 2 expected-
  literal update (inside an already-listed file).
- **Open questions 1-4** are well-posed pre-cleared options (module name,
  :707 substring check, T4.2 helper shape, rollback state) — none is a
  defect; they each name the verification point and a fallback.

Under-specifications likely to cause rework (filed as findings, not
convention violations): L-M4 (sibling `mod tests` name collision) and
L-M5 (missing `&` in the proposed helper body). Both are caught on first
compile and are one-line breakdown fixes.

## Findings

### L-M1 — Blocking — Item 2, TDD step 5 (T2.3): insta snapshot flow prescribed for a non-insta test

- **Where:** breakdown Item 2 scope line ("…+ snapshot") and TDD step 5
  ("update the snapshot (`just test -p codex-core apply_patch` → `cargo insta
  pending-snapshots` → review → `cargo insta accept -p codex-core` scoped to
  this test's snapshot) → green").
- **Evidence:** `create_apply_patch_function_tool_matches_expected_spec`
  (`apply_patch_spec_tests.rs:40-61`) is an inline `assert_eq!` comparing the
  built `ToolSpec` to a fully-literal `ResponsesApiTool` — no
  `assert_snapshot!`/`insta::` anywhere in `core/src/tools/handlers/`, no
  `.snap` file for this test exists, and the only core-suite snapshot that
  mentions apply_patch is a tool-call transcript (unaffected by spec text).
  The spec's own T2.3 wording ("updated to the new exact text") describes an
  edit to the expected literal; the breakdown turned it into an insta flow
  that has no target.
- **Impact:** Item 2 cannot complete as written: after the implementation the
  test fails with an ordinary assert_eq diff, `cargo insta pending-snapshots`
  reports nothing, `cargo insta accept` does nothing, and the tree stays red
  until the executor deviates from the prescribed mechanism.
- **Suggested resolution:** rewrite T2.3: "the test fails with an assert_eq
  diff on the new text → update the expected literals in
  `apply_patch_spec_tests.rs` (tool description, line 45; `patch` argument
  description, line 52) to the spec §3.1 exact text → green. (Not an insta
  snapshot — no `cargo insta` step involved.)" Also drop "(+ snapshot)" from
  the item-overview files-touched cell.

### L-M2 — Major — Ledger row 4 cites a nonexistent file path

- **Where:** breakdown §1 red-state ledger, row 4: "`test_apply_patch_cli_
  rejects_invalid_hunk_header` (`codex-rs/core/tests/suite/tool.rs`:386)".
- **Evidence:** `codex-rs/core/tests/suite/tool.rs` does not exist (no
  `tool.rs` anywhere under `core/tests/`; `find codex-rs -name tool.rs
  -path '*tests*'` returns exactly one file). The test is at
  **`codex-rs/apply-patch/tests/suite/tool.rs:386`**; the exact-stderr
  assertion is at **:393** of that file. Spec §3.3 cites the correct path —
  the breakdown's transcription dropped the `apply-patch/` crate. (Item 3's
  files-touched cell writes the crate-less `tests/suite/tool.rs (:393)`,
  which is ambiguous between the two suites.)
- **Impact:** an executor opening the cited path finds nothing; recovery is
  trivial (unique test name), but the citation is actually wrong and Item 3's
  first step ("re-verify all ledger lines… at HEAD") would burn a cycle on it.
- **Suggested resolution:** correct the path to
  `codex-rs/apply-patch/tests/suite/tool.rs:386` in the ledger row and make
  Item 3's files-touched entry `codex-rs/apply-patch/tests/suite/tool.rs
  (:393)`.

### L-M3 — Minor — "24 scenario fixtures" is 25 at HEAD

- **Where:** breakdown §1 surviving-tests line ("the 24 golden scenario
  fixtures") and Item 1 T1.13 ("the 24 scenario fixtures under
  `codex-rs/apply-patch/tests/fixtures/scenarios`").
- **Evidence:** 25 scenario directories at HEAD (26 entries minus
  `README.md`); two dirs share the `020_` prefix (`020_delete_file_success`,
  `020_whitespace_padded_patch_marker_lines`), each complete
  (input/patch.txt/expected). The runner iterates `read_dir` dynamically, so
  the test is count-independent.
- **Impact:** a reviewer counting fixtures would flag a "missing" scenario;
  no executability impact.
- **Suggested resolution:** say "the 25 scenario fixtures" — or better, "all
  scenario fixtures under …" so the count cannot drift again.

### L-M4 — Minor — Item 1 step 6: `mod tests;` sibling declaration collides with the existing inline module

- **Where:** breakdown Item 1 TDD step 6 ("`#[cfg(test)] #[path =
  \"streaming_parser_p2_tests.rs\"] mod tests;`").
- **Evidence:** `streaming_parser.rs` already declares an inline
  `#[cfg(test)] mod tests {` at :382-383; a second `mod tests` in the same
  file is a duplicate-name compile error. The file's "existing pattern" is
  precisely the colliding name; the crate's sibling example
  (`file_update.rs:15-17`) also names its module `tests` (safe there only
  because that file has no inline module). Open question 1 does ask for
  name confirmation before coding, which contains the issue — but the
  literal instruction is still wrong.
- **Impact:** first compile of Item 1 fails with a duplicate module; fix is
  one word (module name), but it is a guaranteed rework cycle if taken
  literally.
- **Suggested resolution:** state the module name explicitly, e.g.
  `#[cfg(test)] #[path = "streaming_parser_p2_tests.rs"] mod
  streaming_parser_p2_tests;`, and note the inline `mod tests` at
  :382-383 as the reason the name must differ.

### L-M5 — Minor — proposed `ev_apply_patch_function_call` body does not type-check as written

- **Where:** breakdown Item 4 TDD step 2 (and spec §4 T4.2 scaffold, same
  text): "ev_function_call(call_id, \"apply_patch\",
  serde_json::to_string(&json!({ \"patch\": patch })).unwrap())".
- **Evidence:** `ev_function_call` (`responses.rs:933`) takes
  `arguments: &str`; the expression as written yields an owned `String`,
  which does not coerce to `&str` at an argument position. The two sibling
  pattern helpers the breakdown points at both bind first
  (`let arguments = serde_json::to_string(…).expect(…);
  ev_function_call(call_id, "…", &arguments)` — :1025-1028, :1030-1035).
- **Impact:** compile error on the first build of the helper; immediately
  obvious and one-character fix (`&` / local bind), but the "copy the
  scaffold" instruction would not compile verbatim.
- **Suggested resolution:** write the helper with the sibling local-bind
  pattern explicitly (it also matches the `.expect("serialize … arguments")`
  message style of the neighbors).

### L-N1 — Nit — `apply_patch.rs:508-560` range ends 3 lines before `handle_call` closes

The fn spans :508-563; the cited range ends at :560, mid-`run_apply_patch_text`
call. All P3.3-relevant lines (:534-539) are inside the range. Cosmetic;
`:508-563` would be exact.

### L-N2 — Nit — `registry.rs:548-556` range is off by one at the start and short at the end

The `matches_kind` check is at :549 (the range starts at :548, a `}`), and the
`FunctionCallError::Fatal(message)` at :562 falls past the range end (:556).
The cited message (:550) is exact; the rejection behavior is fully described.
`:549-562` would be exact.

### L-N3 — Nit — `responses.rs:39-58` ends on the `requests` signature line

`ResponseMock` struct :39-41 and `single_request` :50-56 are inside; the
`requests` fn's body (:59-60) is just past the range end. Both cited symbols
are present. `:39-60` would be exact.

### L-N4 — Nit — `lib.rs:374-384` (mandate-list cite) ends before the hunk-arm message string

The `InvalidPatchError` arm (:374-377) is fully inside; the
`InvalidHunkError` arm's message string sits at :385-386, just past the
range. The breakdown's own Item 3 text cites only `lib.rs:370` (exact), so
this only affects the broader cite list; `:374-389` would be exact.

## Re-verified clean

Claims checked and confirmed true at HEAD (no finding raised):

- All `streaming_parser.rs` refs: arm :198-214 (with sub-cites :199/:202-208/
  :209-214), :193, :222, :168, :184, :374, :814, :828, :838, :848, :819,
  :784, :797, :307-349 — exact.
- All `parser.rs` refs: :256-274, :268, :271, :193-199, :57, :277, :281,
  :287, :402, :558, :579/:594/:609/:624/:628/:635/:639-642 (7 = 6 Begin + 1
  End, matching the spec's v3-corrected composition) — exact.
- `apply_patch.rs` (core): :411, :534-536, :539, :603-605 — exact.
- `invocation.rs`: :116/:123/:170/:175, :235-241, :237, :242-245 — exact.
- `lib.rs` (apply-patch): :370, :505-535 — exact.
- `file_update.rs:110`, `spec_plan.rs:1257-1271`, `provider.rs:372`,
  `model-provider-info/src/lib.rs:40` and :546-547 (no Azure branch),
  `default.md:132` (file path as given by the spec) — exact.
- Scaffolding: `apply_patch_tests.rs:45` (`invocation_for_payload`,
  `ToolPayload → ToolInvocation`), `session/tests.rs:5882`
  (`make_session_and_context`), `test_codex.rs:347` (`with_config` mutator)
  and :839-859 (provider literal/clone/assignment; default name "OpenAI"),
  `apply_patch_cli.rs:228/:323/:689/:707`, `responses.rs:933-943/:1025-1028/
  :1030-1035` — exact.
- P1 text: 126/2,198/2,324 chars and 21/380/401 words verified
  mechanically; 7/7 drift-guard substrings present in the new argument
  description and absent from the current one; the breakdown's 7-substring
  list == spec §3.1's 7-substring list (empty diff).
- T2.2 probe on the unmodified debug binary (git-clean crate): exit 0,
  `notes/todo.md` byte-exact `# TODO\n\n1. ship the fix\n`, `src/main.rs`
  one chunk / `@@ fn main` / 1 removal / 1 addition / 1 context — PASS.
- Item 3: four sites exist with the current texts quoted above; spec §3.3.1/
  .2/.3a/.3b/.4a/.4b strings present verbatim and copy-able; per-site prefix
  status matches code behavior; the :707 surviving substring stays reachable
  under §3.3.1's prefix-preserving extension.
- Item 4: rejection chain (custom_tool_call → kind check :549 → :550 message)
  correct; `with_config` provider-rename pattern precedented in 8+ existing
  tests; `apply_patch_responses` accepts a new event-constructor fn pointer.
- Item 5: every seam-doc edit target exists (§2 Azure line :54, §3 design
  section, §4 divergence table, §6 conflict sites, §6 verification gate,
  §6 invariants block); all gate recipes exist in the root justfile with
  `working-directory := "codex-rs"`; crate names verified; cargo-insta
  1.46.3 installed.
- Item 6: `~/bin/wiretap.py` passive (read in full: log + forward + stream
  back, no injection); `~/Projects/upstream/codex-bin-backups/` exists with
  a rollback binary; `[[bin]] apply_patch` in `codex-rs/apply-patch/
  Cargo.toml:12-14`.
- Ledger arithmetic: 13 red assertions = 1 (Item 1) + 12 (Item 3, rows 1,
  3-13); item order makes P2 land before P3 depends on it.

## Counts

| Severity | Count | Findings |
|---|---|---|
| Blocking | 1 | L-M1 |
| Major | 1 | L-M2 |
| Minor | 3 | L-M3, L-M4, L-M5 |
| Nit | 4 | L-N1, L-N2, L-N3, L-N4 |
| **Total** | **9** | |

**Verdict: CHANGES-REQUESTED** — all findings are breakdown-text fixes
(≤ 4 lines each); none changes the design, the spec strings, the red-state
ledger, or the item set. After L-M1/L-M2/L-M4/L-M5 are applied, a fresh
round should be able to approve on implementability grounds.

— Seat L, round 1 (leaf reviewer; all evidence at HEAD of
`feat/normalize-content-types-vllm`; probes confined to /tmp)
