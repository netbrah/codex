#![expect(clippy::expect_used)]

//! Live smoke tests for the Anthropic /messages wire protocol.
//!
//! `#[ignore]` by default — run locally with:
//! ```bash
//! CODEX_LLM_PROXY_KEY=sk-... \
//! CODEX_PROXY_BASE_URL=https://your-proxy/v1 \
//!   cargo test --test live_messages -- --ignored
//! ```
//!
//! S-013: Validates that the /messages wire produces real responses via
//! a Claude-compatible endpoint. Complements the fixture-based unit tests
//! in codex-api and the headless e2e tests in exec/tests/proxy_e2e_messages.rs.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use tempfile::TempDir;

fn require_proxy_env() -> (String, String) {
    let key = std::env::var("CODEX_LLM_PROXY_KEY")
        .expect("CODEX_LLM_PROXY_KEY env var not set — skip running live messages tests");
    let url = std::env::var("CODEX_PROXY_BASE_URL")
        .expect("CODEX_PROXY_BASE_URL env var not set — skip running live messages tests");
    (key, url)
}

/// Spawns codex-rs configured for the Messages wire against a live proxy.
fn run_messages_live(prompt: &str) -> (assert_cmd::assert::Assert, TempDir) {
    #![expect(clippy::unwrap_used)]
    let (api_key, base_url) = require_proxy_env();
    let dir = TempDir::new().unwrap();

    // Write a config.toml that uses the Messages wire
    let codex_home = dir.path().join(".codex");
    std::fs::create_dir_all(&codex_home).unwrap();
    let config = format!(
        r#"model = "claude-sonnet-4.6"
model_provider = "messages-proxy"
approval_policy = "never"

[model_providers.messages-proxy]
name = "Messages Wire Proxy"
base_url = "{base_url}"
env_key = "ANTHROPIC_API_KEY"
wire_api = "messages"

[projects."{workdir}"]
trust_level = "trusted"
"#,
        workdir = dir.path().display(),
    );
    std::fs::write(codex_home.join("config.toml"), config).unwrap();

    let mut cmd = Command::new(codex_utils_cargo_bin::cargo_bin("codex-rs").unwrap());
    cmd.current_dir(dir.path());
    cmd.env("CODEX_HOME", codex_home.to_str().unwrap());
    cmd.env("ANTHROPIC_API_KEY", &api_key);
    cmd.env("CODEX_SANDBOX_NETWORK_DISABLED", "");

    cmd.arg("--allow-no-git-exec")
        .arg("-v")
        .arg("--")
        .arg(prompt);

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("failed to spawn codex-rs");

    // Send terminating newline so Session::run exits after the first turn.
    child
        .stdin
        .as_mut()
        .expect("child stdin unavailable")
        .write_all(b"\n")
        .expect("failed to write to child stdin");

    fn tee<R: Read + Send + 'static>(
        mut reader: R,
        mut writer: impl Write + Send + 'static,
    ) -> thread::JoinHandle<Vec<u8>> {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        writer.write_all(&chunk[..n]).ok();
                        writer.flush().ok();
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(_) => break,
                }
            }
            buf
        })
    }

    let stdout_handle = tee(
        child.stdout.take().expect("child stdout"),
        std::io::stdout(),
    );
    let stderr_handle = tee(
        child.stderr.take().expect("child stderr"),
        std::io::stderr(),
    );

    let status = child.wait().expect("failed to wait on child");
    let stdout = stdout_handle.join().expect("stdout thread panicked");
    let stderr = stderr_handle.join().expect("stderr thread panicked");

    let output = std::process::Output {
        status,
        stdout,
        stderr,
    };

    (output.assert(), dir)
}

/// Basic smoke test: prompt → response via /messages wire.
#[ignore]
#[test]
fn live_messages_basic_response() {
    if std::env::var("CODEX_LLM_PROXY_KEY").is_err() {
        eprintln!("skipping live_messages_basic_response – CODEX_LLM_PROXY_KEY not set");
        return;
    }

    let (assert, _dir) = run_messages_live("Reply with exactly the word 'pong' and nothing else.");
    assert.success().stdout(predicate::str::contains("pong"));
}

/// Tool call round-trip: model calls shell, gets result, responds.
#[ignore]
#[test]
fn live_messages_shell_tool_call() {
    if std::env::var("CODEX_LLM_PROXY_KEY").is_err() {
        eprintln!("skipping live_messages_shell_tool_call – CODEX_LLM_PROXY_KEY not set");
        return;
    }

    let (assert, _dir) =
        run_messages_live("Use the shell tool to run 'echo XLI_MESSAGES_TEST'. Report what it printed.");
    assert
        .success()
        .stdout(predicate::str::contains("XLI_MESSAGES_TEST"));
}

/// Verify the binary doesn't crash on the Messages wire with thinking enabled.
#[ignore]
#[test]
fn live_messages_thinking_no_crash() {
    if std::env::var("CODEX_LLM_PROXY_KEY").is_err() {
        eprintln!("skipping live_messages_thinking_no_crash – CODEX_LLM_PROXY_KEY not set");
        return;
    }

    let (assert, _dir) = run_messages_live("What is 7 * 8? Think step by step. Answer with the number only.");
    assert.success().stdout(predicate::str::contains("56"));
}
