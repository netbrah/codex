#!/usr/bin/env node
// XLI — Cross-LLM Interface
// Thin shim that spawns the compiled Rust binary.
//
// Home isolation: XLI defaults runtime state to ~/.xli so it never
// collides with stock Codex ~/.codex installs.
//
//   XLI_HOME  — override to relocate XLI state (default: ~/.xli)
//   CODEX_HOME — if explicitly set, XLI honors it as-is.
//                if unset, XLI bridges it to XLI_HOME so the Rust
//                engine writes under ~/.xli transparently.

import { spawn } from "node:child_process";
import { existsSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import os from "os";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ── Home isolation ──────────────────────────────────────────────────
const homeDir = os.homedir();
const XLI_HOME = process.env.XLI_HOME || path.join(homeDir, ".xli");

// Build env for the child process.  Start with a shallow copy of the
// current environment so we never mutate process.env itself.
const childEnv = { ...process.env, XLI_HOME };

// Bridge CODEX_HOME to XLI_HOME when the operator hasn't explicitly
// set CODEX_HOME.  This makes the Rust engine (which reads CODEX_HOME)
// store its state under ~/.xli without any engine changes.
if (!process.env.CODEX_HOME) {
  childEnv.CODEX_HOME = XLI_HOME;
}
// ────────────────────────────────────────────────────────────────────

const TARGETS = {
  "darwin-arm64":  "aarch64-apple-darwin",
  "darwin-x64":    "x86_64-apple-darwin",
  "linux-arm64":   "aarch64-unknown-linux-musl",
  "linux-x64":     "x86_64-unknown-linux-musl",
};

const key = `${process.platform}-${process.arch}`;
const triple = TARGETS[key];
if (!triple) {
  console.error(`XLI: unsupported platform ${key}`);
  process.exit(1);
}

const binaryPath = path.join(__dirname, "..", "vendor", triple, "xli", "xli");
if (!existsSync(binaryPath)) {
  console.error(`XLI: binary not found at ${binaryPath}`);
  console.error(`This package was built for a different platform.`);
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env: childEnv,
});

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => {
    try { child.kill(sig); } catch {}
  });
});

const result = await new Promise((resolve) => {
  child.on("exit", (code, signal) => {
    resolve(signal ? { type: "signal", signal } : { type: "code", exitCode: code ?? 1 });
  });
});

if (result.type === "signal") {
  process.kill(process.pid, result.signal);
} else {
  process.exit(result.exitCode);
}
