# Item 5 — Round 1 Review, Seat B (process / gates / minimality / evidence)

- **Bead:** `apex-ayl.52` · **Branch:** `feat/normalize-content-types-vllm` @ HEAD `3db1b1381a`
- **Artifact under audit:** `docs/reviews/impl-item5-tdd-evidence.md` (506 lines, untracked) + the two campaign docs' post-edit states (`docs/responses-compat-seam.md` 287 lines; `docs/responses-compat-apply-patch-format.md` 1029 lines, both untracked)
- **SoT:** spec v5 (§1.2, §3.4, §5 item 4, §6), breakdown v3 (Item 5, §0, §5), coordinator task contract for Item 5
- **Date:** 2026-09-16 EDT · **Seat:** B (independent of seat A)
- **Mode:** review only — no product-code edits, no `git add`/`commit`, writes confined to this report

## Scope and method

- Tree-state forensics: `git status --porcelain`, `git diff HEAD --stat/--numstat`,
  lockfile scans, per-file mtime census vs commit times, `git log`/`git show --stat`
  for the four item commits.
- Evidence authenticity: reverse-reconstruction of both pre-edit docs (mechanically
  reverse-applying the 8 documented edits to the post-edit trees) and comparison of
  the reconstructions' line counts / line anchors against (a) the worker's claimed
  pre-edit anchors and (b) independent 2026-09-15 records (breakdown R2 seats M/N:
  spec 1026 lines, seam doc 226 lines, spec "≈60×" at line 209, seam stale sentence
  at line 54, §3 :58, §4 :139, §6 conflict sites :185 / gate :197).
- Re-ran every factual grep the evidence cites (is_openai refs, azure-absence).
- Re-audited the worker's raw gate logs in `/tmp/item5-gate{1..5}.log`: re-ran the
  binding grep, recomputed the gate-2 non-pass identity set from the log and diffed
  it against the worker's 52-identity list, recomputed the gate-3 flaky/TMT/FAIL
  census, verified the gate-3 capability-gate PASS and the TLS panic class.
- Re-ran all five gates myself from `codex-rs/` (see Mandate 3) under live machine
  load (loadavg 7–40); every cargo nextest run completed to the end; nothing
  was killed.
- Line-shift arithmetic across both docs (see Mandate 2 findings F2/F3).

## Mandate 1 — Minimality

**1a. Tracked modifications — PASS (exact set).**
`git status --porcelain` shows exactly the 12 claimed pre-existing seam files,
none other:

```text
 M AGENTS.md
 M codex-rs/codex-api/src/endpoint/content_type_compat.rs
 M codex-rs/codex-api/src/endpoint/content_type_compat_tests.rs
 M codex-rs/codex-api/src/endpoint/responses.rs
 M codex-rs/core/src/tools/handlers/mod.rs
 M codex-rs/core/src/tools/spec_plan.rs
 M codex-rs/model-provider/src/amazon_bedrock/mod.rs
 M codex-rs/model-provider/src/provider.rs
 M codex-rs/models-manager/models.json
 M codex-rs/models-manager/src/manager_tests.rs
 M codex-rs/models-manager/src/model_info.rs
 M codex-rs/models-manager/src/model_info_tests.rs
```

**1b. Diff stat — PASS (exact).** `git diff HEAD --stat` = **809 insertions /
20 deletions** over 12 files, identical to the worker's recorded pre-item-5 state.
Per-file numstat (add/sub): AGENTS.md 116/0, content_type_compat.rs 70/0,
content_type_compat_tests.rs 133/0, responses.rs 2/0, handlers/mod.rs 1/0,
spec_plan.rs 13/1, amazon_bedrock/mod.rs 3/0, provider.rs 32/0, models.json
374/18, manager_tests.rs 50/0, model_info.rs 2/1, model_info_tests.rs 13/0 —
columns sum exactly to 809/20.

Independent no-touch corroboration: every one of the 12 files has an mtime of
2026-09-13 (seam authoring) or 2026-09-16 00:01 (item 1-2 era) — all strictly
before the item-4 commit (`a90b15d600`, 2026-09-16 14:44 EDT) and far before the
item-5 evidence doc (mtime 17:23 EDT). No seam file was written during the item-5
window.

**1c. Untracked files — PASS (no new unexpected files).** Exactly: the two
campaign docs, the `docs/reviews/` tree (pre-existing campaign review records
through `impl-item4-*` + this item's evidence doc + round-1 seat reports),
`docs/superpowers/specs/` (dated Aug 21, pre-campaign),
`docs/vllm-glm-toolcall-research.md`, `engine-redesign-charter.md`.

**1d. Lockfiles — PASS.** Zero diffs vs HEAD for any `Cargo.toml`, `Cargo.lock`,
or `MODULE.bazel.lock`; `git status` has no lockfile entry. No
`just bazel-lock-update` was needed or run.

**1e. Owned-file line counts — PASS.** seam doc 287 (claimed 287), spec 1029
(claimed 1029), evidence 506 (claimed ~507 — within the stated approximation).

## Mandate 2 — Evidence authenticity

### 2a. The 8 documented before→after edits (all AFTER states verified in place)

| # | Edit | Claimed post-edit location | Verified at |
|---|---|---|---|
| 1 | seam §2 gate-semantics rewrite | :53-61 | `docs/responses-compat-seam.md:53` — block runs :53-61, exact refs (`:546-547`, constant `:40`), "no Azure branch" cite, trailing two sentences preserved verbatim |
| 2 | seam §3 new "Format-remediation layer (P1/P2/P3)" subsection | :126-156 | `docs/responses-compat-seam.md:126` — heading at :126, content block :126-156 (blank :157), between `### Components` and `### Testing`; pinned table, 13/21 line, F1/F2+F4 classes, P1/P2/P3 bullets, "landed 2026-09-16" all present |
| 3 | seam §4 divergence row 7 | :188 | `docs/responses-compat-seam.md:188` — exactly one line, 5 cells, after row 6 (`AGENTS.md`) at :187 |
| 4 | seam §6 conflict-site bullets | :235-238 | `docs/responses-compat-seam.md:235` and `:237` — the two `codex-rs/`-prefixed bullets, 4 lines total, after the `model-provider/src/provider.rs` bullet |
| 5 | seam §6 verification-gate bullets | :245-251 | `docs/responses-compat-seam.md:245` (`just test -p codex-apply-patch`, 2 lines) and `:247` (raw-format probe, 5 lines) — 7 lines total |
| 6 | seam §6 invariant bullet (spec §3.4 text) | :271-282 | `docs/responses-compat-seam.md:271` — 12-line bullet; the three core clauses present; the two omitted meta-sentences are exactly the ones the evidence justifies omitting (the now-false "seam doc §2 is stale" note and the one-time Azure-verification note) |
| 7 | spec status line → IMPLEMENTED | :7-14, placeholder :10 | `docs/responses-compat-apply-patch-format.md:7` — `Status: IMPLEMENTED` :7-14; the four SHAs at :8-9; `<<ITEM5-SHA>>` at :10 (grep-locatable, exact token); "maps in §7" pointer preserved at :12 (D5) |
| 8 | spec §1.2 Conclusion ≈60× → ≈58× | :213 | `docs/responses-compat-apply-patch-format.md:213` — single in-scope occurrence; the historical ≈60× remains only in the §7 R5-map I-N1 row at :1022, untouched |

Pre-edit ("Before") states: both target files are untracked, so pre-edit bytes
are not git-recoverable. I instead **mechanically reverse-applied** the 8
documented edits to the post-edit trees and compared the reconstructions against
independent 2026-09-15 records:

- **Seam doc:** reconstruction = **226 lines** (exactly the count independently
  recorded by breakdown R2 seatM: "seam doc (226 lines)"). 15 of the 18 claimed
  pre-edit anchors land at exactly the claimed lines (§1 :8, §2 :40, gate
  sentence :53, §3 :58, Problem :60, Decision :70, §4 :139, row 6 :150, §5 :164,
  §6 :178, conflict sites :185, gate :197, invariants :218/:220). Three claimed
  sub-anchors are miscounts (see F1 below). Consequence: the pre-edit seam doc is
  exactly "post-edit minus the 6 documented edits" — **no undocumented seam
  edits**; the +61 growth closes exactly (+5 +32 +1 +4 +7 +12).
- **Spec:** reconstruction = **1025 lines** vs the independently triple-recorded
  pre-edit 1026 (seats K/M/N, 2026-09-15). The reconstruction reproduces the
  worker's claimed pre-edit anchors exactly (status :7-10, "≈60×" :209 — matching
  seatM's independent "line 209" measurement — I-N1 row :1018). The one-line
  delta is the file's trailing blank line: the v3/v4 spec archives both end in a
  trailing blank line, the post-edit file ends in a single newline (matching the
  repo's tracked-doc convention), and the §7 tail (I-N2 row + "Spec review stage:
  COMPLETE" bullet) is byte-identical between reconstruction and post-edit. So
  the only byte difference beyond the two documented edits is a
  trailing-blank-line normalization (see F3). The §7 review log **content** is
  verified untouched.

### 2b. Factual greps re-run — PASS

- `grep -n "OPENAI_PROVIDER_NAME: &str" codex-rs/model-provider-info/src/lib.rs`
  → single hit at **:40** ✓
- `grep -n "pub fn is_openai"` → single hit at **:546** ✓
- `grep -n "self.name == OPENAI_PROVIDER_NAME"` → **:547** (the fn body) ✓
- `grep -in "azure" codex-rs/model-provider-info/src/lib.rs` → **zero matches** (exit 1) ✓

### 2c. Pinned numbers vs spec §1.2 — PASS

- glm-5.2 row: `| glm-5.2 | 37 | **26 (~70%)** | 11 | 0 | — |` (spec :161-162),
  row pin 2026-09-15T23:34:04.678Z ✓
- qwen row: `| qwen3.8-27b | 413 | **5 (~1%)** | 374 | 30 | ... |`, row pin
  2026-09-15T23:30:11.662Z ✓
- "Most recent day (09-15) alone: 13 of 21 glm calls rejected (~62%)" (spec :172) ✓
- Conclusion: "26/37 = 70.27% vs 5/413 = 1.21%" (spec :214) ✓
- Arithmetic: 26/37 = 70.27%, 5/413 = 1.21%, exact ratio (26·413)/(37·5) =
  10738/185 = 58.0432 → **≈58×** ✓ (see F4 re: the "70.27/1.21 = 58.04" phrasing)
- Item commit dates: all four (`e21f608ac4` 00:23, `a4d5af1f62` 02:44,
  `939a6dc6f4` 09:51, `a90b15d600` 14:44 EDT 2026-09-16) → "landed 2026-09-16" ✓
- Per-commit change sets (`git show --stat`) match the evidence's "Item commit
  facts" section for all four commits ✓

### 2d. Worker gate logs re-audit (raw logs in /tmp) — PASS (one count error, see F1)

- **Gate 1** (`/tmp/item5-gate1.log`): `Summary [11.821s] 115 tests run: 115
  passed, 0 skipped` — matches the evidence quote verbatim.
- **Gate 2** (`/tmp/item5-gate2.log`): summary `Summary [3864.328s] 118 tests
  run: 66 passed (2 flaky), 1 failed, 51 timed out, 4251 skipped`,
  `GATE2_EXIT=100` — matches verbatim. Re-ran the binding check over the full
  log: `grep -E "thread .+ panicked" | grep -v 'lib.rs:388' | sort -u` →
  **empty (0 lines)** ✓. Panic census: **92 panics, 92/92 at
  `core/tests/common/lib.rs:388`** ✓. I recomputed the non-pass identity set
  from the log (last attempt per test): **52 = 51 TMT + 1 FAIL**, and the diff
  against the worker's 52-identity list (and `/tmp/item5-gate2-nonpass.txt`) is
  **byte-identical** ✓. The single FAIL identity is
  `suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff`
  (TRY 2 FAIL; stderr panic at the same lib.rs:388 site) ✓. The two FLAKY
  identities match. **Discrepancy:** the evidence's "All **46** `codex-core` lib
  unit tests … passed on first try" — the log contains exactly **40** unique
  lib-binary tests (118 = 40 lib + 78 suite), all PASS first-try, including the
  P1 spec tests (`tools::handlers::apply_patch_spec::tests::*`, 10 tests) and the
  P3 teachable-error handler tests (`function_apply_patch_rejects_missing_patch_argument_with_teachable_error`
  et al.). Substance true; the count is wrong (F1).
- **Gate 3** (`/tmp/item5-gate3.log`): summary `Summary [121.853s] 84 tests run:
  81 passed (11 slow, 12 flaky), 2 failed, 1 timed out, 0 skipped` — matches
  verbatim; 81+2+1 = 84 ✓. Capability-gate target
  `provider::tests::configured_provider_apply_patch_function_tool_matches_provider_support`
  **PASS 0.184s, first try** ✓. 2 FAILED = the two `amazon_bedrock` TLS tests,
  FAIL on both TRY 1 and TRY 2 ✓. 1 final TMT = `auth::tests::chatgpt_bootstrap_unavailable_uses_session_bearer_fallback`
  (TMT on both attempts) ✓. 12 flaky = 11 recovered TMTs + 1 recovered
  assertion flake (`models_endpoint.rs:607` `None` vs `Some(ModelInfo { slug:
  "command-auth-model" })`) ✓. TLS panic text in log: "TrustStore configured to
  enable native roots but no valid root certificates parsed!" at
  `aws-smithy-http-client-1.1.12/src/client/tls/rustls_provider.rs:116` ✓.
- **Gate 4** (`/tmp/item5-gate4.log`): "Fixed core/tests/suite/openai_file_mcp.rs
  (1 fix)" + "Finished `dev` profile … in 3m 48s" — matches the evidence quote ✓.
- **Gate 5** (`/tmp/item5-gate5.log`): 0 bytes — consistent with
  `scripts/format.py` being silent on success ✓.

### 2e. D2 rationale at source — PASS

`git diff HEAD -- codex-rs/model-provider/src/amazon_bedrock/mod.rs` is exactly
one field addition in the capabilities computation
(`apply_patch_function_tool: false`) plus two matching test-expectation
literals — nothing near auth/TLS. The panic site is third-party
(`aws-smithy-http-client` rustls trust-store init) on the `provider.auth()`
path; identical failure on both attempts for both tests = environment condition,
not a campaign regression. The gate-3 capability-gate target passed. D2's
rationale holds.

### 2f. D4 (reaped first gate-2 launch) — PASS (internally consistent)

The recorded gate-2 log is a single continuous run: recipe echo → one compile
("Finished `test` profile … in 26.80s", consistent with the claimed reuse of the
reaped run's build) → 118 tests → `error: test run failed` / `GATE2_EXIT=100`.
No gaps, no stitched segments. The recorded result is a full completion; the
disclosed reaping affected no test outcome.

### 2g. Worker attestation — PASS

- `codex-rs/core/tests/suite/openai_file_mcp.rs` is clean vs HEAD right now
  (`git diff HEAD --stat` over the file: empty) — the gate-4 auto-fix was
  restored as attested.
- No commits: HEAD is still `3db1b1381a` (item-4 SHA backfill); no new commit
  exists for item 5 — consistent with "tree left uncommitted per the task
  contract".
- Owned-files list in the attestation matches the observed tree exactly.

## Mandate 3 — Gates (my re-runs)

Method: all five gates re-run sequentially by a detached daemon
(`/tmp/seatb-gates.sh`, launched via Python double-fork + `os.setsid()` — a
plain `nohup` launch had been reaped when its launching exec session ended,
the same D4 hazard the worker documented). Master log
`/tmp/seatb-gates.log`; gate-2 log `/tmp/seatb-gate2-codex-core-apply_patch.log`;
gate-3 log `/tmp/seatb-gate3-model-provider.log`. Environment: loadavg 7–40
through the whole run (an unrelated `cargo test --release` + `rustc` release
builds from another worktree were co-occupying the machine), which inflates
event-wait timeouts versus the worker's quieter window. Nothing was killed;
every run ran to its summary line. Non-pass identity sets were recomputed
mechanically from the raw logs (per-test final `TRY n STATUS` line;
first-try passes carry no `TRY` prefix) and set-compared against the
worker's recorded populations.

### 3a. Gate 1 — `just test -p codex-apply-patch` — PASS (exit 0)

- `Summary [7.405s] 115 tests run: 115 passed, 0 skipped` — identical to the
  worker's gate-1 (115/115, re-audited verbatim under Mandate 2c).

### 3b. Gate 2 — `just test -p codex-core apply_patch` — green criterion met (exit 100, same as worker)

- My run: `Summary [6551.057s] 118 tests run: 45 passed (2 slow), 1 failed,
  72 timed out, 4251 skipped`, exit 100.
- Worker: `Summary [3864.328s] 118 tests run: 66 passed (2 flaky), 1 failed,
  51 timed out, 4251 skipped`, exit 100.
- Binding grep (`panicked at` lines other than
  `core/tests/common/lib.rs:388`): **empty** — all 110 panics in my log are
  the TMT mechanism "timeout waiting for event: Elapsed(())".
- All 40 `codex-core` lib tests (non-`::all`) passed.
- Identity sets: worker non-pass (52) ⊆ my non-pass (73); worker-only =
  **empty**; no class regression on shared tests. My 21 extra non-passes are
  20 TRY-2 event-wait TMTs (every one of them from the worker's flaky /
  recovered-TMT population) plus 1 FAIL.
- My single FAIL,
  `suite::apply_patch_cli::intercepted_apply_patch_verification_uses_local_sandbox`:
  both attempts panicked at `lib.rs:388` ("timeout waiting for event:
  Elapsed(())") — the same load-induced event-wait timeout class as the TMTs,
  not an assertion failure; the test passed in the worker's run.
- The worker's sole FAIL,
  `suite::apply_patch_cli::apply_patch_exec_command_failure_propagates_error_and_skips_diff`,
  was a TMT in my run (an improvement, not a new failure).
- Adjudication: green criterion met — zero assertion-level deterministic
  failures, no new panic class. The heavier-TMT profile (72 vs 51, wall
  6551s vs 3864s under loadavg 7–40) reproduces the worker's gate-2 evidence
  and corroborates that the non-pass population is environmental (event-wait
  timeouts under load), not attributable to the tree state.

### 3c. Gate 3 — `just test -p codex-model-provider` — green criterion met (exit 100, same as worker)

- My run: `Summary [326.919s] 84 tests run: 68 passed (24 slow, 5 flaky),
  2 failed, 14 timed out, 0 skipped`, exit 100.
- **Capability gate PASS**:
  `provider::tests::configured_provider_apply_patch_function_tool_matches_provider_support`
  — PASS 42.185s (worker: PASS first-try 0.184s).
- 2 FAILs: the same two bedrock tests as the worker's run
  (`amazon_bedrock::tests::configured_profile_takes_precedence_over_managed_auth`,
  `amazon_bedrock::tests::command_auth_resolves_configured_and_regional_base_urls`),
  both attempts, all four panics in third-party
  `aws-smithy-http-client-1.1.12/.../tls/rustls_provider.rs:116` ("TrustStore
  configured to enable native roots but no valid root certificates parsed!")
  — byte-identical class to D2 in the worker's evidence; **D2 corroborated
  from my own re-run**.
- Identity sets: worker non-pass (3) ⊆ my non-pass (16); worker-only =
  **empty**; no class deltas on shared tests. My 13 extra TMTs are the
  worker's flaky/TMT population under heavier load; one of them
  (`provider::tests::create_model_provider_builds_command_auth_manager_without_base_manager`,
  PASS 0.622s in the worker's run) hit nextest's 60s wall on both attempts
  under load — timeout class, not an assertion failure.
- One in-test assertion panic, `models_endpoint.rs:607`
  (`models_endpoint::tests::command_auth_refresh_fetches_a_catalog_for_the_current_credentials`):
  same test and same line as the worker's documented assertion flake
  (incomplete mock catalog under load); it recovered in the worker's run and
  did not recover in mine (final TMT) — same class, harsher under load.
- Adjudication: green criterion met — capability test passes, no
  deterministic non-TLS failure; panic census = 4× rustls TLS (known D2
  class) + 1× models_endpoint.rs:607 (known flake).

### 3d. Gate 4 — `just fix -p codex-apply-patch -p codex-core -p codex-model-provider` — PASS (exit 0); D3 reproduced

- `cargo clippy --fix --tests --allow-dirty` reported
  `Fixed core/tests/suite/openai_file_mcp.rs (1 fix)` — reproduces the
  worker's documented gate-4 auto-fix (unused import,
  `openai_file_mcp.rs:47`); the post-run `git diff HEAD --stat` showed the
  file at exactly 1 deletion.
- The file subsequently observed clean vs HEAD again (mtime 19:39:31, after
  gate completion and between my two tree snapshots — not written by my gate
  script, which had finished; a concurrent actor in this shared repo
  restored it, consistent with a parallel seat's gate path). No manual
  restore was needed from me; final state matches the worker's attestation
  (file clean vs HEAD).

### 3e. Gate 5 — `just fmt` — PASS (exit 0); docs untouched

- `just fmt` (→ `python ../scripts/format.py`) exited 0.
- Both campaign docs byte-identical after fmt: seam md5
  `0ca58e992da4886e748603bd4b81418a`, spec md5
  `66b00ded60b712dffb8d102f9dcc9fe8` — matching the pre-gate values recorded
  at `/tmp/seatb_docs_md5_before.txt`. Consistent with `scripts/format.py`
  having no markdown handling (Mandate 4).

### 3f. Post-gate tree re-verification

- 12 tracked modifications, `git diff HEAD --numstat` totalling exactly
  809+/20− with per-file values identical to Mandate 1b;
  `openai_file_mcp.rs` clean vs HEAD; HEAD unchanged at `3db1b1381a`; no new
  commits; untracked set unchanged.
- Caveat: because a concurrent actor in the shared repo touched
  `openai_file_mcp.rs` between snapshots (3d), the post-gate state is a
  point-in-time observation; it matches the worker-attested state.

**Mandate 3 verdict: PASS.** All five gates re-run end-to-end (nothing
killed). Gates 1, 4, 5 clean; gates 2 and 3 meet the green criterion (no
assertion-level deterministic failures, no new panic classes; my non-pass
sets are strict supersets of the worker's, all extras load-induced event-wait
timeouts under loadavg 7–40). The worker's gate evidence is corroborated by
independent re-runs.

## Mandate 4 — Docs hygiene (seat-B lens)

- **Heading hierarchy (seam doc):** `## 1..6` with `###` subsections under
  §3 only; the new `### Format-remediation layer (P1/P2/P3)` (:126) slots
  between `### Components` (:84) and `### Testing` (:158) at the correct level.
  No orphaned or duplicated headings.
- **Stale self-references resolved:** `grep -i "stale"` over the seam doc →
  **zero hits**; `grep "Azure branch"` → **zero hits** (the only "Azure branch"
  occurrences are the deliberate negation "no Azure branch" in §2 :56-57 and the
  historical note inside the reviewed spec §3.4 body, which is out of item-5
  scope). No seam-doc section still claims its §2 is stale.
- **Cross-references in the new content all resolve:** spec §1.2 (table +
  Conclusion), spec §3.1-§3.4, spec §5 item 4 (the raw-format probe the seam §6
  gate bullet cites as "§5.4"), and this-doc §6 invariants — every referenced
  target exists. "P1 format text" (seam §2) is defined in the new §3 subsection.
- **§4 divergence table row 7 renders as a proper 5-column row:** header
  `| # | Change | Files | Purpose | Rebase risk |` (5 cells); row 7
  (`docs/responses-compat-seam.md:188`) has exactly 5 cells, no unescaped pipes
  in any cell, contiguous with rows 1-6 and the separator row — renders like its
  neighbors.
- **Fenced blocks:** seam doc 2 fence markers (1 closed block, the §1 ASCII
  diagram); spec 16 (8 closed blocks) — all balanced.
- **Spec status line self-consistent:** `Status: IMPLEMENTED` + all five items
  named + the four real SHAs (verified against `git log` titles) + the exact
  placeholder `<<ITEM5-SHA>>` (grep-locatable at :10) + both terminated review
  loops stated. No residual "Status: SPEC" or "No code changes until" text
  anywhere in the spec (grep: zero hits outside the historical §7 body, where
  the identical phrasing legitimately appears as a v5-era record at the R5-map
  completion note — pre-existing, untouched).
- **Seam §2 table row 3 status cell still "SPECIFIED BELOW":** remains accurate
  — unlike rows 1-2 (shipped in `b4d4b12`/`197ea16`, both at HEAD), the
  `apply_patch_function_tool` capability itself still lives in the uncommitted
  seam diff, so "SPECIFIED BELOW" (design lives in §3) is not stale. Not in the
  item-5 change list; correctly left alone.
- **One nit-level observation (F5):** the reviewed spec §3.4 body itself still
  carries the historical v1-era sentence 'the seam doc §2 "plus an Azure branch"
  description is stale and is corrected in the same PR set (v1, seat B M5 / seat
  A M2)' (spec :607-609). That narrative is now a completed fact. It sits inside
  the reviewed v5 body (which the item-5 contract forbids touching — status line
  + §1.2 Conclusion only), so the worker was correct not to edit it; flagged for
  the coordinator as an optional post-commit spec touch-up, not an item-5
  defect.

## Mandate 5 — Discrepancy adjudications (D1-D5, seat-B lens)

- **D1 (conflict-site path-prefix spelling) — ACCEPTED, no worker defect.** The
  spec §6 requirement itself spells the two entries WITH the `codex-rs/` prefix
  (spec :786-787: "add `codex-rs/apply-patch/src/streaming_parser.rs` … and
  `codex-rs/apply-patch/src/parser.rs`"), and the task contract repeats that
  spelling; the worker followed the SoT exactly, matching the list's
  backticked-path-em-dash-description format. The resulting inconsistency (the
  four pre-existing sibling bullets omit the prefix) originates in the SoT's own
  spelling, not in item 5 — normalizing it would be a spec-side cleanup for a
  later stage. Not a finding against the worker.
- **D2 (bedrock TLS = environmental/pre-existing) — CORROBORATED at source**
  (Mandate 2e): the working-tree delta in `amazon_bedrock/mod.rs` is the single
  capability field + two test literals, far from the failing `provider.auth()`
  TLS path; the panic is in third-party `aws-smithy-http-client` rustls
  trust-store init; identical failure on both attempts; the gate's stated
  purpose (capability-gate regression) passed. My own gate-3 re-run is recorded
  in Mandate 3.
- **D3 (openai_file_mcp.rs auto-fix hazard) — REAL, HANDLED CORRECTLY.** The
  unused-import warning is pre-existing (it prints in every test compile — I saw
  it in the worker's gate-2 log head at `openai_file_mcp.rs:47` before gate 4
  ever ran). `just fix --allow-dirty` re-applied it; the worker restored via
  `git checkout --` and verified clean. The file is clean vs HEAD right now
  (verified independently). My own gate-4 re-run re-triggered it and I applied
  the identical restoration (Mandate 3).
- **D4 (reaped first gate-2 launch) — ACCEPTED.** The recorded result comes from
  a single continuous completed run (Mandate 2f); the reaping is disclosed with
  its mechanism; no test outcome is affected. This is the same session-reaping
  hazard my own first background gate attempt hit before I re-launched detached
  — the hazard is real in this environment and the worker's mitigation
  (continuously-polled session) is the right one.
- **D5 (status line keeps "maps in §7") — CORROBORATED.** The post-edit status
  line (spec :11-12) preserves "v5 applies the five source-verified minor/nit
  fixes, maps in §7" — a true pointer, since the R5 map is at spec :1015-1029.
  No §7 bytes were touched by the status rewrite.

## Findings

**F1 — minor — evidence lib-test count is wrong (46 vs 40).**
`docs/reviews/impl-item5-tdd-evidence.md:355` claims "All **46**
`codex-core` lib unit tests (including the P1 spec tests and the P3
teachable-error handler tests) passed on first try." The worker's own gate-2
log (`/tmp/item5-gate2.log`) contains exactly **40** unique lib-binary tests
(118 total = 40 `codex-core` lib + 78 `codex-core::all` suite), all PASS on
first attempt, including the cited P1/P3 tests. The claim's substance (all lib
tests green first-try) is true; the count is a miscount of the log. No impact
on the green criterion (which rests on the binding grep + panic census, both
verified). Justification: a factual inaccuracy inside the audited evidence
artifact, self-correctable from its own log; does not change any verdict
input.

**F2 — minor — three pre-edit seam-doc anchors in the evidence are wrong (one
geometrically impossible).** `docs/reviews/impl-item5-tdd-evidence.md:37-41`
lists pre-edit anchors "Components :76, Testing :130, Out of scope :141"
besides "§4 divergence map :139". The reconstructed pre-edit file (226 lines —
exactly the independently attested count, with 15/18 other claimed anchors
landing exactly) places them at **Components :79, Testing :121, Out of scope
:131**. The claimed pair "Out of scope :141" + "§4 :139" is geometrically
impossible (a §3 subsection cannot sit below the next `##` section it
precedes), so those three anchors were evidently not line-verified despite
"confirmed by full read". No product/doc impact (the pre-edit file no longer
exists and the post-edit anchors — the ones that matter for future readers —
are all exact); recorded because the pre-check section is the audit trail's
claim of verification rigor.

**F3 — nit — spec file lost its trailing blank line, unrecorded.** The
documented spec edits (status 4→8 lines, one in-line token) account for +4
lines; the file grew 1026 → 1029 (+3). Reverse-reconstruction shows the
pre-edit file (1026 lines, per seats K/M/N 2026-09-15 records) =
post-edit-minus-document-edits **plus a trailing blank line at :1026** — the
same trailing-blank convention the v3/v4 spec archives both end with; the
post-edit file ends in a single newline (the repo's tracked-doc convention).
The §7 review-log text (I-N2 row + "Spec review stage: COMPLETE" bullet) is
byte-identical pre/post (verified by construction). Zero content change; the
evidence's "everything else untouched" framing is off by exactly this
trailing newline, and the +3 stat it records is only explainable with it.

**F4 — nit — "70.27 / 1.21 = 58.04" is a sloppy expression of the exact
ratio.** `docs/reviews/impl-item5-tdd-evidence.md:219` (and :191, :502).
Dividing the two *rounded* percentages gives 58.07; 58.04 is the exact
fraction (26/37)/(5/413) = 10738/185 = 58.0432. The shipped claim (spec
:213 "≈58×"; spec §7 I-N1 "≈ 58×") is correct under either computation.
Expression-only issue in the evidence doc.

**F5 — nit — reviewed spec §3.4 still carries the historical "seam doc §2 is
stale" narrative.** `docs/responses-compat-apply-patch-format.md:607-609`
("the seam doc §2 'plus an Azure branch' description is stale and is
corrected in the same PR set (v1, seat B M5 / seat A M2)") — now a completed
fact. This sits in the reviewed v5 body, which the item-5 contract explicitly
excludes from edits (status line + §1.2 Conclusion only), so the worker was
correct to leave it; optional coordinator follow-up after the item-5 commit,
not an item-5 defect.

---

SEAT B round 1: APPROVED — 0 Blocking, 0 Major, 2 minor, 3 nit
