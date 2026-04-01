#![expect(clippy::expect_used)]

//! Live smoke tests for the Anthropic /messages wire protocol.
//!
//! `#[ignore]` by default — run locally with:
//! ```bash
//!   cargo test --test all -- live_messages --ignored --test-threads=1
//! ```
//!
//! Uses CODEX_LLM_PROXY_KEY/ANTHROPIC_API_KEY and
//! CODEX_PROXY_BASE_URL/ANTHROPIC_BASE_URL from the environment.
//!
//! S-013: Validates that the /messages wire produces real responses via
//! a Claude-compatible endpoint.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn resolve_xli_binary() -> PathBuf {
    // Allow overriding the binary via env var for release testing
    for var in ["XLI_BINARY", "CODEX_BINARY"] {
        if let Ok(path) = std::env::var(var) {
            let p = PathBuf::from(&path);
            if p.is_relative() {
                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let workspace_root = manifest_dir.parent().unwrap();
                let resolved = workspace_root.join(&p);
                if resolved.exists() {
                    return resolved;
                }
            } else if p.exists() {
                return p;
            }
        }
    }
    codex_utils_cargo_bin::cargo_bin("xli").expect("xli binary not found in target/debug")
}

fn proxy_key() -> String {
    std::env::var("CODEX_LLM_PROXY_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .unwrap_or_default()
}

fn proxy_base_url() -> String {
    let url = std::env::var("CODEX_PROXY_BASE_URL")
        .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
        .unwrap_or_default();
    // Ensure /v1 suffix — the Messages endpoint appends /messages to base_url
    if !url.is_empty() && !url.ends_with("/v1") {
        format!("{}/v1", url.trim_end_matches('/'))
    } else {
        url
    }
}

fn skip_unless_configured() -> bool {
    if proxy_key().is_empty() {
        eprintln!("Skipping live messages test — CODEX_LLM_PROXY_KEY/ANTHROPIC_API_KEY not set");
        return true;
    }
    if proxy_base_url().is_empty() {
        eprintln!("Skipping live messages test — CODEX_PROXY_BASE_URL/ANTHROPIC_BASE_URL not set");
        return true;
    }
    false
}

struct RunResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn run_messages_exec(prompt: &str) -> RunResult {
    #![expect(clippy::unwrap_used)]
    let dir = TempDir::new().unwrap();
    let codex_home = dir.path().join(".xli");
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
        base_url = proxy_base_url(),
        workdir = dir.path().display(),
    );
    std::fs::write(codex_home.join("config.toml"), config).unwrap();

    let binary = resolve_xli_binary();

    let output = Command::new(&binary)
        .arg("exec")
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg(prompt)
        .env("CODEX_HOME", codex_home.to_str().unwrap())
        .env("ANTHROPIC_API_KEY", proxy_key())
        .env("CODEX_SANDBOX_NETWORK_DISABLED", "")
        .current_dir(dir.path())
        .output()
        .expect("failed to spawn xli");

    RunResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

#[ignore]
#[test]
fn live_messages_basic_response() {
    if skip_unless_configured() {
        return;
    }

    let result = run_messages_exec("Reply with exactly the word 'pong' and nothing else.");
    assert_eq!(
        result.exit_code, 0,
        "exit code should be 0\nstderr: {}",
        result.stderr
    );
    // JSONL output should contain an agent_message item with the response
    assert!(
        result.stdout.contains("pong"),
        "response should contain 'pong'\nstdout: {}",
        result.stdout
    );
}

#[ignore]
#[test]
fn live_messages_shell_tool_call() {
    if skip_unless_configured() {
        return;
    }

    let result = run_messages_exec(
        "Use the shell tool to run 'echo XLI_LIVE_MSG_TEST'. Report what it printed.",
    );
    assert_eq!(
        result.exit_code, 0,
        "exit code should be 0\nstderr: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("XLI_LIVE_MSG_TEST"),
        "should contain tool output\nstdout: {}",
        result.stdout
    );
}

#[ignore]
#[test]
fn live_messages_thinking_no_crash() {
    if skip_unless_configured() {
        return;
    }

    let result =
        run_messages_exec("What is 7 * 8? Think step by step. Answer with the number only.");
    assert_eq!(
        result.exit_code, 0,
        "exit code should be 0\nstderr: {}",
        result.stderr
    );
    assert!(
        result.stdout.contains("56"),
        "should produce correct answer\nstdout: {}",
        result.stdout
    );
}
