#!/usr/bin/env node
// XLI — Cross-LLM Interface
// Thin shim that spawns the compiled Rust binary.

import { spawn } from "node:child_process";
import { existsSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

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
  env: process.env,
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
