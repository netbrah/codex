#!/usr/bin/env node
// S-040 — XLI Home Isolation + Branding acceptance tests
//
// Exercises the env-bridging logic and branding features from xli.js
// without spawning the Rust binary.

import os from "os";
import path from "path";
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const XLI_JS = path.join(__dirname, "..", "bin", "xli.js");

/**
 * Replicate the env-bridging logic from xli.js so we can unit-test
 * it in isolation.  If the launcher logic changes, keep this in sync.
 *
 * The launcher only sets XLI_HOME in the child env.  CODEX_HOME is no
 * longer bridged — the Rust engine reads XLI_HOME natively (with
 * CODEX_HOME as a legacy fallback).
 */
function resolveXliEnv(processEnv) {
  const homeDir = os.homedir();
  const XLI_HOME = processEnv.XLI_HOME || path.join(homeDir, ".xli");
  const childEnv = { ...processEnv, XLI_HOME };
  return childEnv;
}

// ── Home Isolation Tests ────────────────────────────────────────────

describe("S-040 Home Isolation", () => {
  const home = os.homedir();

  it("Case 1: both unset → XLI_HOME defaults to ~/.xli", () => {
    const env = resolveXliEnv({ HOME: home });
    assert.equal(env.XLI_HOME, path.join(home, ".xli"));
    // CODEX_HOME is NOT bridged — Rust engine reads XLI_HOME natively
    assert.equal(env.CODEX_HOME, undefined);
  });

  it("Case 2: XLI_HOME set → used as-is", () => {
    const env = resolveXliEnv({ HOME: home, XLI_HOME: "/tmp/custom-xli" });
    assert.equal(env.XLI_HOME, "/tmp/custom-xli");
  });

  it("Case 3: CODEX_HOME explicitly set → passed through (legacy compat)", () => {
    const env = resolveXliEnv({
      HOME: home,
      CODEX_HOME: "/opt/my-codex-home",
    });
    // CODEX_HOME passes through from the parent env unchanged
    assert.equal(env.CODEX_HOME, "/opt/my-codex-home");
    // XLI_HOME still defaults
    assert.equal(env.XLI_HOME, path.join(home, ".xli"));
  });

  it("Case 4: both XLI_HOME and CODEX_HOME set → both preserved", () => {
    const env = resolveXliEnv({
      HOME: home,
      XLI_HOME: "/tmp/xli-override",
      CODEX_HOME: "/tmp/codex-override",
    });
    assert.equal(env.XLI_HOME, "/tmp/xli-override");
    assert.equal(env.CODEX_HOME, "/tmp/codex-override");
  });

  it("XLI_HOME is always present in child env", () => {
    const env = resolveXliEnv({});
    assert.ok(env.XLI_HOME, "XLI_HOME must be set");
    assert.ok(
      env.XLI_HOME.endsWith(".xli"),
      `XLI_HOME should end with .xli, got: ${env.XLI_HOME}`
    );
  });
});

// ── Branding Tests ──────────────────────────────────────────────────

describe("S-040 Branding", () => {
  it("--version prints xli version", () => {
    const out = execFileSync("node", [XLI_JS, "--version"], {
      encoding: "utf8",
    }).trim();
    assert.match(out, /^xli \d+\.\d+\.\d+$/);
    assert.ok(!out.includes("codex"), "--version should say xli, not codex");
  });

  it("-V prints xli version", () => {
    const out = execFileSync("node", [XLI_JS, "-V"], {
      encoding: "utf8",
    }).trim();
    assert.match(out, /^xli \d+\.\d+\.\d+$/);
  });

  it("--version reads version from package.json", () => {
    const out = execFileSync("node", [XLI_JS, "--version"], {
      encoding: "utf8",
    }).trim();
    // Should contain the version from package.json
    assert.ok(out.includes("0.1.0"), `Expected 0.1.0 in: ${out}`);
  });
});
