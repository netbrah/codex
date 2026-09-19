# APPLY-LAX-1 — apply-patch strict-first shape-repair pre-pass (SDD)

**SDD status:** DRAFT v1.4 CONVERGED — R1: M1 1B/2M/4m/1n on v1.0 (all 8 applied, §10 rows 1–8), N1 0B/0M/2m/2n on v1.2 (full 0B/0M round on the latest artifact); v1.3 = N1 pin/wording polish (row 10); v1.4 = post-convergence coordinator fix: T13 heredoc-marker bytes corrected to the code-recognized forms (row 11); verdicts archived to `/tmp/applylax-r1-seat{M,N}-verdict.md`; TDD item A GO
**Bead:** `apex-xt2.11` (central ledger `$HOME/Projects/bitbucket/apex_tracking`)
**Repo/branch:** `/Users/palanisd/Projects/upstream/codex` @ `feat/normalize-content-types-vllm` (post-merge; HEAD at spec time `9ac22bb79c`)
**Binary affected:** `codex-combined` / `codex-combined-v2` (fork build only; npm `codex` is upstream openai/codex and never receives this code)

**Coordinator note (dirty tree at spec time):** concurrent workers in flight — `codex-rs/tools/` (apex-xt2.7 `strict_int`, worker `xt27_item1_tdd`) and `codex-rs/models-manager/` + `manager_tests.rs` (apex-xt2.9 item 1, worker `xt29_item1_tdd`). This campaign touches ONLY `codex-rs/apply-patch/` + one small seam in `codex-rs/core/src/tools/handlers/apply_patch.rs`; staging MUST be pathspec-strict so item commits never swallow in-flight hunks from the other two campaigns.

---

## 1. Problem

glm-5.2 (on-prem worker model, `-p glm`) emits `apply_patch` function-tool arguments in deterministic broken shapes that the strict parser rejects. This is the `apply_patch_reject / fmt` subclass from the apex-xt2.2 census. The apex-ayl.52 campaign (teachable errors + P2 lenient AddFile arm) fixed the qwen class and two of glm's three shapes, but NOT all of them: root-attributed census on the NEW build (v2 `5789dd8500`, cutover 2026-09-16T22:44Z) shows **glm 40 hits/26 sess (fmt 36) vs OLD 42/27 (fmt 33) — no drift**.

Operator ruling (apex-xt2.2 decision note, 2026-09-17): glm worker profiles STAY live, conditional on this fix; the fix must be a **model-agnostic ratchet** — no provider/model gating, no overfitting to glm.

## 2. Evidence (§2.1–§2.3 verified first-hand at spec-time HEAD; §2.4 live — re-derived at `c2c1b8a620`)

### 2.1 The three glm shapes (verbatim from the census; bead `apex-xt2.11`)

| shape | form | example | today |
|---|---|---|---|
| A | first line = `*** Add File: …` (missing `*** Begin Patch`); content correctly `+`-prefixed | 2026-09-17T16:47Z sid `01a0b11f` (`grok/plans/crosswire-preplan-glm-20260917.md`) | **REJECTED** |
| B | `*** Begin Patch` + `*** Add File:` then RAW unprefixed content lines | 09-17T14:15Z `01a0b094`; 09-17T12:30Z `01a0b034` | parses OK (P2 arm) |
| C | stray bare `@@` line at hunk position 3 inside Add File | 09-17T00:08Z `01a0ad8d` | parses OK (P2 arm) |
| AB | `*** Add File:` first, THEN `*** Begin Patch`, duplicate AddFile, bare `@@` | repro `b943dedf` | **REJECTED** |

### 2.2 Reproduction (evidence on disk)

`/tmp/glm-repro-out.txt` (from `/tmp/glm_repro_tmp.rs.retired`, run against this tree):

```
REPRO 74246043 (B: Begin+AddFile+raw-lines): PARSED OK hunks=1
REPRO 91b01ea3 (C: Begin+AddFile+bare-@@+plus-lines): PARSED OK hunks=1
REPRO 7584b626 (A: missing-Begin, plus-prefixed): REJECTED: invalid patch: The first line of the patch must be '*** Begin Patch'. …
REPRO b943dedf (AB: AddFile-first, then Begin, duplicate AddFile, @@): REJECTED: invalid patch: The first line of the patch must be '*** Begin Patch'. …
```

Raw rejected patches: `/tmp/glm-fail-{74246043,7584b626,91b01ea3,b943dedf}.txt` — TDD fixtures MUST be modeled on these real byte sequences, not invented shapes.

### 2.3 Current non-streaming parse path (first-hand, `codex-rs/apply-patch/src/parser.rs`)

```
parse_patch(patch)                                  :146  pub entry (function-tool seam)
  └─ mode = PARSE_IN_STRICT_MODE ? Strict : Lenient  :54 (const = false → ALWAYS Lenient today)
  └─ parse_patch_text(patch, mode)                   :194
       ├─ patch.trim().lines()
       ├─ Strict  → check_patch_boundaries_strict    :216
       ├─ Lenient → check_patch_boundaries_lenient   :233  (tries strict FIRST, then — only if
       │                                                    first line is `<<EOF`/`<<'EOF'`/`<<"EOF"` AND last
       │                                                    line ends with `EOF` AND len ≥ 4 — strips the
       │                                                    heredoc wrapper and re-calls strict on the inner
       │                                                    lines; else returns the ORIGINAL strict error)
       ├─ check_start_and_end_lines_strict           :257  (teachable boundary errors, apex-ayl.52 item-3:
       │                                                    first-line text :269 (arm :268–:270), last-line text :272 (arm :271–:273))
       └─ StreamingPatchParser::push_delta/finish    :203–:204  (hunk-level state machine; constructed :202)
```

Streaming parser (same crate, `streaming_parser.rs`):
- `NotStarted` arm: any line ≠ `*** Begin Patch` → `InvalidPatchError("The first line of the patch must be '*** Begin Patch'")` (`:186–:194`, short text at `:192` — preview/streaming-seam only).
- `StartedPatch` arm: non-hunk-header line → `InvalidHunkError` with the long teachable text (`:199–:204`, string at `:201`).
- **P2 AddFile arm (fork, commit `e21f608ac4`):** in `AddFile` mode, a non-`+`-prefixed line is appended verbatim as file content (arm `:206–:222`; P2 verbatim branch `:217–:220`, `push_str(line)` at `:218`) — this is why shapes B and C parse today. `+`-prefixed lines keep canonical prefix-strip semantics.

### 2.4 Call sites and capability seam (first-hand `rg`; live — re-derived at `c2c1b8a620`)

- **Function-tool seam (where glm hits):** `core/src/tools/handlers/apply_patch.rs:411` — `parse_patch(patch_input)`; `Err` → `FunctionCallError::RespondToModel("apply_patch verification failed: {parse_error}")` (`:413–:417`); `Ok` → `verify_apply_patch_args_with_mode(args, …)` (takes ownership of `args`) → on `Body(changes)`: `execute_verified_patch(…)` → `ApplyPatchToolOutput::from_text(content)` (`:452–:459`).
- **Local-shell invocation:** `apply-patch/src/invocation.rs:116` / `:123` — `parse_patch` on the `apply_patch`/`apply-patch` command bodies; `:170` / `:175` — `parse_patch(…).is_ok()` used as a *detection* predicate (is this input an apply-patch?). The pre-pass makes shape-A/AB inputs parse → they are now also *detected* on the fork binary (intended benefit, not a regression — see §7).
- **Streaming preview seam:** `apply_patch.rs:45` (`use StreamingPatchParser`), `:90–:91` (`ApplyPatchArgumentDiffConsumer` — TUI preview `PatchApplyUpdated` events only) — never calls `parse_patch` → untouched. **R1-M1 [M] correction:** the OpenAI NATIVE handler is `ApplyPatchHandler` (`:79`, freeform `Custom` payload, `handle_call` `:360` → `run_apply_patch_text` `:381` → `parse_patch` `:411`) — so the native path DOES get the pre-pass + note (consistent with the model-agnostic ruling); the function-tool handler (`FunctionApplyPatchHandler`, `:504`/`:560`) shares the same text path.
- **Capability:** `model-provider/src/provider.rs:402` — `apply_patch_function_tool: !self.info.is_openai()`. The freeform apply_patch function tool (the seam glm/qwen use) is exposed ONLY to non-OpenAI models; OpenAI models use the native streaming tool.
- `ApplyPatchArgs` (`apply-patch/src/lib.rs:152–:162`): `#[derive(Debug, PartialEq)]` — **no serde derives, not serialized** → adding a field cannot change any wire/resume schema.
- FORK-MANIFEST: `parser.rs` (`:86`), `streaming_parser.rs` (`:87`), `apply_patch.rs` (`:92`) already registered MUST SURVIVE from the apex-ayl.52 campaign.

## 3. Design (narrowed strict-first retry — supersedes the bead's wide pre-pass)

### 3.1 Invariants (the review-critical properties)

1. **Retry-only:** the normalization pre-pass runs ONLY after a strict parse failure. Zero behavior change for every patch that parses today (valid canonical patches, P2-eligible B/C patches, heredoc-wrapped patches).
2. **Signature-gated:** the pre-pass fires ONLY on the two observed glm signatures (shape A, shape AB — §2.1). Any other malformed input falls through to the original error, byte-identical.
3. **Error-text parity:** on retry failure the error returned is the LENIENT attempt's — byte-identical to today's error for EVERY input (today's `parse_patch` runs Lenient unconditionally; that error is whatever today's Lenient path produces for the input — the strict boundary error for boundary-class failures, the state-machine `InvalidHunkError` (`streaming_parser.rs:199–:204`) once boundaries pass, and the INNER error for heredoc-wrapped input (`parser.rs:325`)) — so the model-visible text is unchanged across the board (R1-M1 [B]).
4. **No capability gate:** the pre-pass is not per-model/per-provider (operator ruling: model-agnostic ratchet). OpenAI-wire safety follows from invariants 1–3: an OpenAI-family model's well-formed patch parses on the first strict attempt (invariant 1); a pathological malformed one either matches no signature (invariant 2) or is conservatively repaired with a transparency line (invariant 3's flip side, §3.5).
5. **Streaming preview + P2 arm untouched:** `streaming_parser.rs` (incl. the P2 AddFile arm) and the preview-only `ApplyPatchArgumentDiffConsumer` (`handlers/apply_patch.rs:90–:91`) are unchanged. Both TOOL handlers — native/freeform `ApplyPatchHandler` (:79, Custom payload, `handle_call` :360 → `run_apply_patch_text` :381) and function-tool `FunctionApplyPatchHandler` (:504, Function payload, :560) — share `run_apply_patch_text` → `parse_patch` (:411), so both receive the pre-pass and the note (R1-M1 [M] — the v1.0 "native path untouched" claim was a false call-graph read: the :91 parser is the preview consumer, not the apply path).

### 3.2 New entry flow (`parser.rs:143`)

```rust
pub fn parse_patch(patch: &str) -> Result<ApplyPatchArgs, ParseError> {
    match parse_patch_text(patch, ParseMode::Strict) {
        Ok(args) => Ok(args),
        Err(_strict_err) => {
            // Existing heredoc leniency (gpt-4.1 era) — unchanged behavior, unchanged order.
            // Its ERROR is the parity reference (R1-M1 [B]): it is whatever today's
            // Lenient path produces — boundary-class or hunk-class for non-heredoc
            // input, the INNER error for heredoc-wrapped input (check_patch_boundaries_lenient
            // re-calls strict on inner lines, :325).
            let lenient_err = match parse_patch_text(patch, ParseMode::Lenient) {
                Ok(args) => return Ok(args),
                Err(e) => e,
            };
            // Fork ratchet (apex-xt2.11): strict-first shape repair, retry ONCE.
            // Gated on the lenient error; the fallback below returns IT, so every
            // currently-rejected input yields byte-identical error text to today.
            if let Some(normalized) = normalize_patch_shape(&lenient_err, patch)
                && let Ok(mut args) = parse_patch_text(&normalized, ParseMode::Strict)
            {
                args.repair_note = Some(PATCH_REPAIR_NOTE.to_string());
                return Ok(args);
            }
            Err(lenient_err)
        }
    }
}
```

- Ordering rationale: (a) strict first — fast path, identical result for valid patches (today's Lenient mode calls strict internally first anyway, so this reorders nothing observable); (b) heredoc leniency second — preserves today's `<<'EOF'` behavior EXACTLY, including heredoc-wrapped-but-inner-malformed → the INNER error (`check_patch_boundaries_lenient` :325 returns `check_patch_boundaries_strict(inner_lines)`; R1-M1 [B]) — which is why the fallback returns the LENIENT error, not the outer strict error (the pre-pass sees `<<'EOF'` as line 1, matches no signature, declines); (c) repair last; (d) the lenient error (== today's error for every input).
- The `PARSE_IN_STRICT_MODE` const (spec-time doc `:48–:53`, const `:54`) was dead — **removed by item A (`4be0d017ab`; grep-clean at HEAD)**; its gpt-4.1 rationale already lives in the `ParseMode::Lenient` doc (`:176–:207`, variant at `:208`).

### 3.3 The pre-pass: `fn normalize_patch_shape(err: &ParseError, patch: &str) -> Option<String>`

New private fn in `parser.rs` (pure; no I/O; applied at most once; deterministic).

**Gate:** `matches!(err, ParseError::InvalidPatchError(_))` — boundary-class failures only. Hunk-class errors (`InvalidHunkError`) are never pre-passed: a patch with valid boundaries already reached the state machine, and "fixing" its body would silently change file contents (forbidden).

Let `lines = patch.trim().split('\n')`. **Marker predicate (R1-M1 [m]):** a line IS a marker iff its `trim()` equals the marker const — `BEGIN_PATCH_MARKER` (`parser.rs:38`), hunk markers `:40–:42` — trimmed equality, matching `check_start_and_end_lines_strict` (`:342`) and the state machine's `trim` (`streaming_parser.rs:184`). A `+`-prefixed `+*** Begin Patch` content line is NOT a Begin line (the prefix breaks trimmed equality).

**Case A — no `*** Begin Patch` line anywhere (shape A):**
line 1's trim starts with `*** Add File: ` / `*** Delete File: ` / `*** Update File: ` (the `ADD_FILE_MARKER` / `DELETE_FILE_MARKER` / `UPDATE_FILE_MARKER` consts, `parser.rs:40–:42`)
⇒ **header wrap:** prepend a `*** Begin Patch` line; if the last non-blank line's trim ≠ `*** End Patch`, append a `*** End Patch` line. Return `Some(joined)`.

**Case AB — a `*** Begin Patch` line exists at index i > 0 (shape AB):**
line 1's trim starts with one of the three hunk markers (the leading-stray signature)
⇒ **leading strip:** drop `lines[0..i]`; then the same End-append check as Case A. Return `Some(joined)`.

**Otherwise:** `None` (no signature match — e.g. prose before a later `Begin`, heredoc wrappers, pure garbage → original error).

**Deliberately NOT normalized** (conservatism, each with a reason):
- AddFile content lines lacking `+` prefix — already accepted verbatim by the P2 arm (`streaming_parser.rs:217–:220`); the bead's original sub-step (b) is subsumed, and double-normalization would corrupt content.
- Update/Delete hunks with unprefixed lines — ambiguous (`+`/`-`/space change meaning); the error is kept.
- A `Begin` at index 0 is not "leading strip" material (that's the canonical case, handled by the first strict attempt).
- Heredoc-wrapped shape A (inner line 3 = hunk header, no inner Begin) — not in the census; line 1 is `<<'EOF'` → no signature → unchanged error. Documented out of scope (§9).

### 3.4 Transparency note + handler seam

- `ApplyPatchArgs` gains one field: `pub repair_note: Option<String>` (doc comment: set when the parser repaired a malformed-but-intent-clear patch; surfaced to the model as ONE leading line of the tool output). Schema-safe: no serde derives (§2.4); all constructors set `None` (one production init site at the `parse_patch_text` struct literal `parser.rs:223–:229`; the five in-crate test literals at `:513/:666/:682/:698/:742` (literal starts) plus the core-crate literal at `apply_patch_spec_tests.rs:139` — item A/B compile closure, §5).
- Exact note text (single line, constant `PATCH_REPAIR_NOTE` in `parser.rs`):
  `Note: the patch shape was repaired by the parser (missing or stray '*** Begin Patch'/'*** End Patch' boundary lines); the applied file content is exactly as provided.` Test-local byte-identical copy: `core/src/tools/handlers/apply_patch_tests.rs:561` (comment `:557–:560`: the const stays private; T12 `:564`, starts-with assert `:593` — drift pair with the `parser.rs:51` const, any drift fails T12).
- Handler seam (`core/src/tools/handlers/apply_patch.rs`, `run_apply_patch_text`): capture `let repair_note = args.repair_note.clone();` immediately after the `parse_patch` match (`:411` area, before `verify_apply_patch_args_with_mode` takes ownership of `args`); at the `Body(changes)` success point (`:452–:459`) prepend: `let content = match repair_note { Some(note) => format!("{note}\n{content}"), None => content };`. 5 lines as landed (item B, `0002ba5747`): capture at `:419` + note match block `:455–:458`; no other handler changes. Both tool handlers (native `:381`, function `:560`) route through `run_apply_patch_text` → the `:411` note seam covers both (R1-M1 [M]).
- Model-visible context rules: the note is bounded (one line, ~35 tokens), per-occurrence, no history rewrite ✓.

### 3.5 Error-text parity statement (required)

- Function-tool seam (glm/qwen path): today, every non-heredoc rejection returns the strict boundary error from `check_start_and_end_lines_strict` (`parser.rs:346` first-line string, `:349` last-line string) or the state-machine `InvalidHunkError` (`streaming_parser.rs:199–:204`). After this change: identical values in every case that still fails (invariant 3) — pinned by byte-equality tests T7/T8/T9.
- Streaming seam: unchanged (its short NotStarted text at `streaming_parser.rs:191–:193` (string `:192`) stays preview/streaming-only).
- No NEW error text is introduced anywhere; the only new model-visible string is the success-path `PATCH_REPAIR_NOTE` line.

## 4. Impact analysis / breaking-surface check

| surface | impact |
|---|---|
| app-server / wire schemas | none — `ApplyPatchArgs` is not serialized; no request/response field changes |
| CLI parameters | none |
| config loading | none |
| session resume | none (parsed args never persisted; the note is transient tool output) |
| `parse_patch` callers | core handler `apply_patch.rs:411` (benefit + note seam — shared by BOTH tool handlers: native `:381`, function `:560`); `apply-patch/src/lib.rs:375` `apply_patch_with_options` (production callers: `core/src/tools/runtimes/apply_patch.rs:179` local-shell runtime, `arg0/src/lib.rs:136`, `apply-patch/src/standalone_executable.rs:71` — repaired+applied, **no note line — accepted**, §7); `intercept_apply_patch` (`handlers/apply_patch.rs:709`, applies via `execute_verified_patch` `:741`, no note); `invocation.rs:116/:123` local-shell parse (benefit); `invocation.rs:170/:175` detection predicate — shape-A/AB inputs now *detect* as apply-patch on the fork binary (intended, §7); tests (R1-M1 [M]) |
| OpenAI native tool path | pre-pass + note via the shared `run_apply_patch_text` (R1-M1 [M]); preview diff-consumer untouched |
| P2 leniency (e21f608ac4) | untouched — still the sole mechanism accepting unprefixed AddFile content |
| `codex-rs/tools` (xt2.7), `models-manager` (xt2.9) | disjoint crates; no shared hunks |

## 5. Test plan (TDD; red → green per item)

New test file: `codex-rs/apply-patch/src/parser_shape_retry_tests.rs`, wired with `#[cfg(test)] #[path = "parser_shape_retry_tests.rs"] mod parser_shape_retry_tests;` (repo test-module convention).

| # | test | asserts |
|---|---|---|
| T1 | canonical AddFile patch (Begin + `+` lines + End) | parses; hunks byte-equal to pre-change; `repair_note == None` (zero-change property) |
| T2 | canonical Update patch with `@@`/`+`/`-`/space lines | parses; `repair_note == None` |
| T3 | heredoc-wrapped valid patch (`<<'EOF'` … `EOF`) | parses via leniency; `repair_note == None` (today's behavior preserved) |
| T4 | shape A, End present — `/tmp/glm-fail-7584b626.txt` real bytes (R1-verified: zero Begin/End markers) **with an appended `*** End Patch` line** | parses; file content exactly the `+`-stripped lines; `repair_note == Some(PATCH_REPAIR_NOTE)` |
| T5 | shape A, End absent — `/tmp/glm-fail-7584b626.txt` real bytes **as-is** | parses with appended End; content exact; note `Some` |
| T6 | shape AB (fixture from `/tmp/glm-fail-b943dedf.txt`: stray AddFile first, Begin, duplicate AddFile, bare `@@`) | leading strip fires; the FIRST (stray) AddFile is dropped, the post-`Begin` AddFile applies; bare `@@` content survives via the P2 arm; note `Some` |
| T7 | no signature, boundary class (plain garbage, e.g. `"bad"`) | `Err` byte-identical to pre-change first-line text (`parser.rs:346` string as pinned constant) |
| T8 | valid boundaries, hunk-class failure (e.g. `*** Begin Patch\n*** Update File: f.txt\n*** End Patch` → "Update file hunk … is empty") | `Err` unchanged; pre-pass provably not fired (gate) |
| T9 | Update hunk with unprefixed content line | `Err` unchanged (conservatism: no Update/Delete normalization) |
| T10 | shape A with trailing blank lines | last-non-blank End check; content exact (trailing blanks not injected into file content) |
| T11 | `normalize_patch_shape` purity/idempotence | exact string equality on A and AB inputs; a repaired input re-gated (Begin at line 1, no stray) → `None` |
| T12 | handler-level transparency (crate `codex-core`, existing `apply_patch_tests.rs` harness — lightest existing seam, choice noted in commit body) | `run_apply_patch_text` with a shape-A patch against a temp env → tool output text STARTS WITH `PATCH_REPAIR_NOTE` + `\n`, and the file exists with the exact content |
| T13 | heredoc parity (R1-M1 [B]) — two subcases: (i) heredoc-wrapped, inner missing End (e.g. `<<'EOF'\n*** Begin Patch\n*** Update File: f.py\nEOF`); (ii) heredoc-wrapped, inner hunk error (empty update hunk) | byte-identical to today's INNER errors (the last-line boundary text, and the line-numbered `InvalidHunkError`, respectively) — proves the lenient-error fallback restores parity; no repair fires (line 1 = `<<'EOF'`) |

**Fixture mechanism (R1-M1 [m]):** T4–T6 fixtures are byte-identical INLINE copies of the /tmp failure files (24 KB / 14 KB — not repo-tracked, so they are inlined in the test file; T6 = `/tmp/glm-fail-b943dedf.txt` as-is). Content assertions are re-derived from the same inline bytes.

**Compile closure (R1-M1 [m]):** item A updates the six in-crate `ApplyPatchArgs` literals (`parser.rs:223/:513/:666/:682/:698/:742` — the six literal starts); item B updates `apply_patch_spec_tests.rs:139` (core crate's TEST target — first compiles at item B's gate; the core lib itself compiles after item A — that literal is `#[cfg(test)]`-gated, `apply_patch_spec.rs:108–:110`; item B's T12 note text is a byte-identical test-local copy of the `parser.rs:51` const (`apply_patch_tests.rs:561`) — drift pair, T12 catches drift).

Gates (repo rules + bead GATES): `just fmt` (in `codex-rs/`) · `CARGO_BUILD_JOBS=2 just fix -p codex-apply-patch` · `CARGO_BUILD_JOBS=2 just test -p codex-apply-patch` · scoped core run for T12 (`CARGO_BUILD_JOBS=2 just test -p codex-core --lib apply_patch`) · `CARGO_BUILD_JOBS=2 just test -p codex-models-manager` NOT required (disjoint).

Implementation order (one item at a time, each red→green→gates→review before the next):
1. **Item A:** `repair_note` field + `PATCH_REPAIR_NOTE` const + `normalize_patch_shape` + new entry flow + T1–T11 + T13 (all in `codex-apply-patch`; incl. the six in-crate literal updates).
2. **Item B:** handler seam (2–3 lines in `apply_patch.rs`) + `apply_patch_spec_tests.rs:139` literal + T12.
3. **Item C:** FORK-MANIFEST per §6.

## 6. Docs / manifest

1. **FORK-MANIFEST.md:**
**[COMPLETED at `c2c1b8a620` (apex-xt2.9 item 3 re-derivation): purpose note extended, test file registered in the Code table, counts re-derived from the live tree, Last verified bumped — do not redo.]**
   - `parser.rs` row (`:86` at `c2c1b8a620`): extend the purpose note → "P2 lenient AddFile state (accepts unprefixed content lines) + strict-first shape-repair pre-pass (apex-xt2.11)" — file already registered, **no count change**.
   - New fork-only test file `codex-rs/apply-patch/src/parser_shape_retry_tests.rs` → register under "Fork-only files → Code — MUST SURVIVE" (purpose: shape-repair ratchet tests, apex-xt2.11); **Code count +1** — re-derive the "N added / Code (k) / Docs (d)" block at commit time from the live tree (the xt2.9 manifest item lands first per campaign ordering; take its post-state as the base).
   - Bump "Last verified" after the campaign commits (same convention as the 2026-09-17 ratchet block).
2. **Campaign doc:** this file (`docs/apply-patch-lax-retry-spec.md`) is the campaign record (companion to `docs/responses-compat-apply-patch-format.md`, which stays the format SoT for P1/P2).

## 7. Risks / accepted

- **Conservative signature set:** only A/AB are repaired. Future glm drift to a new shape → new bead + new pre-pass rule (ratchet pattern); each repair is evidence-gated the same way.
- **Detection predicate side effect** (`invocation.rs:170/:175`): shape-A/AB local-shell inputs are now *detected* as apply-patch on the fork binary. Intended (that's the seam glm uses); npm codex unaffected.
- **Worst case for a well-behaved model:** a currently-rejected patch that happens to match a signature gets repaired + applied. Bounded by the signature set (hunk-header-first + missing/stray boundaries). The pre-pass never rewrites SURVIVING lines: Case A only adds boundary lines; Case AB drops the leading stray hunk (header + any content lines before the first `*** Begin Patch`) by design (T6) — the observed glm stray (b943dedf) is header-only; a stray with content would be dropped (R1-M1 [m]). Transparency: the note line covers the tool-handler seams; `apply_patch_with_options` consumers (local-shell runtime, arg0, standalone) get repaired+applied **without** the note — accepted (the success output is unambiguous; §4).
- **Note token cost:** one bounded line per repaired call (§3.4).
- **Upstream divergence:** the pre-pass lives in `parser.rs` (the `:143` entry + pre-pass `normalize_patch_shape` + `with_end_boundary` `:240–:289` (fn `:240–:275`) + boundary fns `:293–:332` strict/lenient + `:334–:352` start/end checks); the P2 verbatim seam it must not touch is in a DIFFERENT file, `streaming_parser.rs:217–:220` (AddFile arm `:206–:222`). Re-derive at each upstream ratchet (FORK-MANIFEST MUST SURVIVE).

## 8. Acceptance (post-merge, live)

- Live glm worker session: a new file created via `apply_patch` with NO heredoc fallback (the model's raw shape-A/AB output is repaired, transparency line visible in the tool output, file content byte-exact).
- Census re-run (apex-xt2.2 method, root sessions, N glm sessions post-cutover): shape-A/AB `fmt` class → **0**; any remaining rejections are hunk-class with the teachable text (out of scope by design).
- qwen regression: canonical + P2 behavior unchanged (spot-check 2–3 qwen sessions).
- OpenAI-native path spot check (R1-M1 [M]): valid patch unchanged; a malformed signature-matching patch via the native/freeform handler → same repair + note behavior, no crash.

## 9. Out of scope

- Streaming seam (OpenAI native tool) — unchanged by design (§3.5).
- Update/Delete content-line normalization (ambiguity, §3.3).
- Heredoc-wrapped shape A (not in census, §3.3).
- Per-model/per-provider gating (operator ruling: model-agnostic).
- MCP tool-output wire-shape 400 class (separate: `apex-xt2.10` / `apex-xt2.12`).
- Tool-args numeric-type ratchets (separate: `apex-xt2.7`, disjoint crate `codex-tools`).

## 10. Review log

| round | date | verdicts | findings applied | status |
|---|---|---|---|---|
| R1 | 2026-09-17 | M1 1B/2M/4m/1n (fresh seat, READ-ONLY, on v1.0); N1 0B/0M/2m/2n (fresh seat, READ-ONLY, on v1.2) | all 8 applied in v1.1 (rows 1–8) + row 9 (v1.2 pin sweep) + row 10 (v1.3 N1 polish) | **CONVERGED** — full 0B/0M round on the latest artifact; N1 minors/nits landed as v1.3 pin/wording polish, no new round (xt2.9 R4 precedent: converged rounds take coordinator polish without re-review) |
| 1 | M1 | B | §3.2/inv3/§3.5: fallback returned the OUTER strict error; today's Lenient returns the INNER error for heredoc-wrapped-inner-malformed input (`check_patch_boundaries_lenient` :248) → model-visible error + teachability regression on the gpt-4.1 seam, with zero test coverage | fallback returns the LENIENT attempt's error + pre-pass gated on it (parity restored for both classes); T13 added; §3.2(b) + invariant 3 reworded |
| 2 | M1 | M | §4 caller list incomplete — `apply_patch_with_options` (lib.rs:370) + production call sites (`runtimes/apply_patch.rs:179`, `arg0:136`, `standalone:71`) + `intercept_apply_patch` (:704) apply with no note | §4 row extended; accepted-silence documented (§4/§7) |
| 3 | M1 | M | §2.4/§3.4/§4/inv5 false call-graph claim — the OpenAI NATIVE handler (`ApplyPatchHandler` :79, Custom, :360→:381) DOES call parse_patch via run_apply_patch_text; the :91 parser is the preview-only diff consumer; :499 is the function-tool handler (:555) | invariant 5 + §2.4 + §3.4 + §4 reworded (both handlers share pre-pass + note — consistent with the model-agnostic ruling); §8 native-path spot check added |
| 4 | M1 | m | §5 T4: the real 7584b626 bytes contain zero Begin/End markers (grep-verified) — contradicted "End present (fixture modeled on real bytes)" | T4 = real bytes + appended End; T5 = real bytes as-is; fixture mechanism (inline byte-identical copies) stated |
| 5 | M1 | m | §5 item split: `repair_note` breaks the core-crate literal `apply_patch_spec_tests.rs:139` (surfaces at item B's gate) + the five in-crate parser.rs literals | item A += six in-crate literals; item B += spec_tests:139 literal |
| 6 | M1 | m | §3.3 Begin-line predicate underspecified (trimmed equality vs `+`-prefixed content lines) | predicate defined: trimmed equality with marker consts (:38, :40–:42); `+*** Begin Patch` is not a Begin line |
| 7 | M1 | m | §7 risk 3: "never rewrites content lines" inaccurate for Case AB in general (leading strip can drop a stray hunk's content) | reworded (drops the leading stray hunk header+content by design; observed stray is header-only) |
| 8 | M1 | n | range-pin drift (named-entity pins all exact): markers :26–:28→:40–:42; Lenient doc :165–:193→:159–:190 (variant :191); last-line text :275–:277→:272 (arm :271–:273); first-line arm ends :270 not :274; init site → literal :206–:211; NotStarted arm :189–:193→:186–:194 (text :192); hunk error :198–:208→:199–:204 (string :201); P2 arm :216–:219→:217–:220 (arm :206–:222, push_str :218 ✓) | re-derived at v1.1 from a fresh numbered read |
| 9 | Coord | — | §7 upstream-divergence line still carried v1.0 ranges (`:216–:219`, `:216–:277`) — found in post-R1 residual-pin sweep (the other grep hit, log row 8, is an intentional quote); not a review finding | re-derived at v1.2 from a fresh numbered read at `02beac536d` (no apply-patch diff since `9ac22bb79c`): streaming P2 arm `:217–:220` (arm `:206–:222`), parser boundary fns `:216–:255` + `:257–:275`, entry `:146` ✓ |
| 10 | N1 | 2m/2n | [m]1 inner-strict pin `:249`→`:248` at five sites (inv 3, §3.2 code comment, ordering (b), row 1, coordinator para); [m]2 inv-3/§3.2 "strict boundary error itself" over-generalization (false for hunk-class input — §3.5 already stated it correctly); [n]1 seven residual range/pointer overruns (§2.3 push/finish `:203–:204` constructed `:202`; §3.2 const doc `:48–:53` + `:54`; T7 first-line string `:269`; §4 intercept fn `:704`; §2.4 `ApplyPatchArgs` `:151–:157`; coord-para freeform call `:348` + `FunctionApplyPatchHandler` struct `:480`/handle `:499`); [n]2 "core crate — first compiles at item B's gate" overstates the breakage class (core lib compiles after item A; the literal is `#[cfg(test)]`-gated, `apply_patch_spec.rs:108–:110`) | all sites first-hand verified by the coordinator at `02beac536d` before applying (numbered reads: parser.rs `:248`/`:202–:204`/`:48–:54`/`:269`, lib.rs `:151–:157`, handlers `:347–:348`/`:480`/`:499`/`:703–:704`, apply_patch_spec.rs `:108–:110`); v1.3 anchored edits |
| 11 | Coord | n | T13 example wrote the heredoc marker without its closing quote (<<'EOF, no closing quote) — the pre-existing lenient code recognizes only <<EOF / <<'EOF' / <<"EOF" (parser.rs:320); the worker RED run proved the spec-literal bytes never fire the heredoc branch | code is truth: T13 implemented with <<'EOF'; spec notations corrected at all 5 sites / 8 tokens (this row + §2.3 line-51 list, §3.2 ordering (b), §3.3 not-normalized bullet, T3 row, T13 row); header v1.3 to v1.4; recorded in the item-A commit body |
| 12 | Coord | n | Seat B N1: spec 3.2 code-block comment pin :248 (inner-strict re-call) is stale post-item-A — item A inserted lines above it; the re-call now sits at parser.rs:325 (verified by both item-A seats). The code comment deliberately carries no pin (drift hygiene) | comment-only; code is truth; the FULL spec pin set (all parser.rs line pins) is re-derived against the post-item-A(+B) tree at item C (spec commit) per rows 8/9 practice — this row exists so that sweep is not missed; no code change |
| 13 | Coord | — | item-C spec-commit pin re-derivation at `c2c1b8a620` (row-12 sweep; first-hand numbered reads): post-item-A(+B) tree — parser.rs entry `:146`→`:143` (+ pre-pass block `:240–:289`: `normalize_patch_shape` `:240–:275` + `with_end_boundary` `:280–:289`, `PATCH_REPAIR_NOTE` const `:51`), inner re-call `:248`→`:325` (as row 12 predicted), `ParseMode::Lenient` doc/variant `:159–:190`/`:191`→`:176–:207`/`:208`, boundary fns `:293–:332` (strict `:293–:301` / lenient `:310–:332`), start/end checks `:334–:352` (teachable error text `:346`/`:349`), production literal `:223–:229` + five test literals (starts `:513/:666/:682/:698/:742`); handlers/apply_patch.rs native call `:381` unchanged, function handler `:480`→`:485` / `:499`→`:504` / `:555`→`:560`, intercept `:704`→`:709` (call `:741`), Body point `:450–:454`→`:452–:459`; lib.rs struct `:151–:157`→`:152–:162`, `apply_patch_with_options` `:361`→`:366` (parse `:370`→`:375`); FORK-MANIFEST rows `:68/:69/:71`→`:86/:87/:92` (manifest re-derived at `c2c1b8a620`); unchanged (verified): all streaming pins, invocation/provider/runtimes/arg0/standalone, spec_tests `:139`, gate `:108–:110`, marker consts `:38`/`:40–:42`; test file = 12 test fns (t1–t11, t13) + 1 helper, T12 in core `apply_patch_tests.rs:564` | live pins updated in place (§2.4, §3.1–§3.5, §4, §5, §7); §2 stays frozen at spec-time `9ac22bb79c` (re-verified accurate via git show); no design change |
| 14 | Coord | — | item-C fix round — seats on the item-C artifact: A 0B/1M/4m/3n · B 0B/1M/2m/3n (verdicts /tmp/applylax-itemc-seat{A,B}-verdict.md): [M]×2 ([A-F1=B-m1] §3.1 invariant-5 pins `:499`/`:555`→`:504`/`:560`; [B-M1] acceptance record overstated §8 criterion 1: cp-01 wirecap (SCS read-only, first-hand by both seats) = fully CANONICAL glm patch → pre-pass never fired, no repair-note line, case check `exists`+`contains` (not byte-exact `equals_text`) → record rewritten (live repair firing + note + byte-exact = unit-level only, T4–T6 on the 24,113/14,705 B fixtures; OPEN with criteria 2+4)); [m]×5 (A-F2 `PARSE_IN_STRICT_MODE` removal recorded, item A `4be0d017ab`; A-F3 row-13 strict `:293–:301`; A-F4 row-13 `ParseMode::Lenient` doc/variant `:176–:207`/`:208` added; A-F5 §6 `:86` + completed-at-`c2c1b8a620` banner; B-m2 crit-3 P2 half — folded into the B-M1 rewrite); [n]×4 (A-F6/A-F7 = B-n1: pre-pass block `:240–:289` + five test literals → starts `:513/:666/:682/:698/:742` at all 3 sites; A-F8 §2.4 scoped live; B-n2 §3.4 seam +5 as landed `:419`+`:455–:458`; B-n3 test-local note const `apply_patch_tests.rs:561` drift pair in §3.4+§5) | all 11 fixed (pin/wording-level; no seat re-round — converged-round coordinator-polish precedent (row 10 / xt2.9 R4)); no design change; header stays v1.4 |

**Coordinator R1 verification (first-hand at HEAD `9ac22bb79c` before applying):** [B] confirmed — `check_patch_boundaries_lenient` returns `check_patch_boundaries_strict(inner_lines)` (parser.rs:248); v1.0's fallback would have swapped the inner error for the outer one; the lenient-error fallback restores byte parity for every input (today's `parse_patch` IS the Lenient path). [M] caller list confirmed by direct read: `apply_patch_with_options` (lib.rs:361, parse at :370), `runtimes/apply_patch.rs:179`, `arg0/src/lib.rs:136`, `standalone_executable.rs:71`, `intercept_apply_patch` (:703, `execute_verified_patch` :736, no note) — silence accepted + documented. [M] call graph confirmed by direct read: `ApplyPatchHandler` (:79, `create_apply_patch_freeform_tool` call :348 in `fn spec` :347, `ToolPayload::Custom` arm, `run_apply_patch_text` :381) and `FunctionApplyPatchHandler` (struct :480, `fn handle` :499, :555) both reach `parse_patch` (:411); `ApplyPatchArgumentDiffConsumer` (:90, parser field :91) is preview-only. T4 byte claim confirmed (7584b626: zero markers). Compile closure confirmed (six parser.rs literals :206/:436/:588/:603/:618/:661; spec_tests:139). Pin re-derivation: the coordinator's fresh numbered read matches M1 within ±1 at arm boundaries; v1.1 uses the coordinator's pins. M1's Verified-clean list (invariant-1 adversarial sweep, gate safety, fixture traces, P2 subsumption, harness feasibility, format-SoT coherence) accepted. **N1 (fresh seat) completed on the v1.2 artifact: 0B/0M/2m/2n — R1 CONVERGED; v1.3 polish applied (row 10).**

R1 verdicts to be archived at `/tmp/applylax-r1-seat{M,N}-verdict.md`; per-finding rows appended below per repo SDD rules (re-review loop to a full 0B/0M round).

**Acceptance record (2026-09-19, post-gate; dated appendix — no design change, header stays v1.4):** codex-harness run on the new release binary (SCS, read-only; coordinator-reported results — evidence: `/x/eng/ai_engineering/APEX/codex-harness/report/20260919-121730-2390379/report.md` + log `codex-harness-run-item2-acceptance.log`; cp-01 wirecap dir `report/20260919-121730-2390379/cp-01/wirecap/` read first-hand by both item-C seats): cp-01 (glm; same task as the 2026-09-13 F1 failure) **PASS** — the 2026-09-13 failure did NOT recur on the new build (task completed via apply_patch): the wirecap (`resp-000.jsonl`) shows glm emitted a fully CANONICAL patch (`*** Begin Patch` / `+` lines / `*** End Patch` all present) — the shape-repair pre-pass **never fired** this run, the tool output (`req-001.json` `function_call_output`) carries **no** repair-note line, and the case file check is `exists` + `contains` (not byte-exact `equals_text`) — so cp-01 met criterion 1's lead clause only (live glm session, new file, no heredoc fallback); the repair-firing + transparency-line + byte-exact parenthetical is validated at UNIT level only (T4–T6 on the real 24,113/14,705 B byte fixtures; T12 at the handler seam) and remains **OPEN** in the live environment. cp-02 (qwen regression) **PASS** — both qwen patches canonical (`cp-02/wirecap/resp-000.jsonl`/`resp-001.jsonl`; byte-exact `equals_text` on greet.py), so criterion 3's canonical half is met at this checkpoint; its P2 half is NOT exercised (no unprefixed AddFile content in the wire — the lenient AddFile arm never fired) and the remaining qwen sessions stay for the operator environment (spec's 2–3-session depth), P2 additionally covered by the P2 unit tests (apply-patch crate). a3/a4 (proactive-default cross-check, apex-xt2.9) **PASS**. §8 coverage: criterion 1 **NOT met by this run** as stated (live repair firing + note + byte-exact — unit-level only; open); criterion 2 (census re-run, apex-xt2.2 method → shape-A/AB `fmt` class 0) NOT covered — open; criterion 3 canonical half met at the cp-02 checkpoint, P2 half open; criterion 4 (OpenAI-native spot check) NOT covered — open. Open live-environment list: criteria 1, 2, 4, and criterion 3's P2 half (+ remaining qwen sessions).
