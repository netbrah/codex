//! Proxy end-to-end tests for the Anthropic /messages wire protocol.
//!
//! Spawns the actual codex binary headless with `--json` against a live
//! Anthropic-compatible API endpoint. Gated behind `CODEX_PROXY_E2E=1`.
//!
//! Required env vars:
//!   CODEX_PROXY_E2E=1              — enable these tests (skipped by default)
//!   CODEX_LLM_PROXY_KEY            — API key for the proxy/endpoint
//!   CODEX_PROXY_BASE_URL           — base URL (e.g. https://api.anthropic.com/v1)
//!
//! Run: `CODEX_PROXY_E2E=1 CODEX_PROXY_BASE_URL=https://your-proxy/v1 CODEX_LLM_PROXY_KEY=sk-... \
//!        cargo test -p codex-exec --test proxy_e2e_messages -- --test-threads=1`

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

fn proxy_base_url() -> String {
    let url = std::env::var("CODEX_PROXY_BASE_URL")
        .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
        .expect("CODEX_PROXY_BASE_URL or ANTHROPIC_BASE_URL must be set");
    // Ensure /v1 suffix — the Messages endpoint appends /messages to base_url
    if !url.ends_with("/v1") {
        format!("{}/v1", url.trim_end_matches('/'))
    } else {
        url
    }
}
const DEFAULT_MODEL: &str = "claude-sonnet-4.6";

fn skip_unless_proxy_e2e() -> bool {
    if std::env::var("CODEX_PROXY_E2E").unwrap_or_default() != "1" {
        eprintln!("Skipping proxy-e2e test (set CODEX_PROXY_E2E=1 to enable)");
        return true;
    }
    if std::env::var("CODEX_LLM_PROXY_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .is_none()
    {
        eprintln!("Skipping proxy-e2e test (CODEX_LLM_PROXY_KEY not set)");
        return true;
    }
    if std::env::var("CODEX_PROXY_BASE_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .is_none()
    {
        eprintln!("Skipping proxy-e2e test (CODEX_PROXY_BASE_URL not set)");
        return true;
    }
    false
}

#[derive(Debug, Clone, Deserialize)]
struct JsonlEvent {
    #[serde(rename = "type")]
    kind: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

#[derive(Debug)]
struct ProxyRunResult {
    events: Vec<JsonlEvent>,
    response: String,
    exit_code: i32,
    input_tokens: i64,
    output_tokens: i64,
    #[allow(dead_code)]
    raw_stderr: String,
}

struct RunConfig {
    prompt: String,
    model: String,
    reasoning_effort: String,
    fixture_files: HashMap<String, String>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            model: DEFAULT_MODEL.to_string(),
            reasoning_effort: "none".to_string(),
            fixture_files: HashMap::new(),
        }
    }
}

fn run_codex_messages(config: RunConfig) -> ProxyRunResult {
    let tmp_dir = tempfile::TempDir::new().expect("create temp dir");
    let codex_home = tmp_dir.path().join(".codex");
    std::fs::create_dir_all(&codex_home).expect("create codex home");

    let api_key = std::env::var("CODEX_LLM_PROXY_KEY").expect("CODEX_LLM_PROXY_KEY");

    let config_content = format!(
        r#"model = "{model}"
model_provider = "anthropic-proxy"
approval_policy = "never"
model_reasoning_effort = "{effort}"

[model_providers.anthropic-proxy]
name = "Anthropic via LiteLLM"
base_url = "{base_url}"
env_key = "ANTHROPIC_API_KEY"
wire_api = "messages"

[projects."{workdir}"]
trust_level = "trusted"
"#,
        model = config.model,
        effort = config.reasoning_effort,
        base_url = proxy_base_url(),
        workdir = tmp_dir.path().display(),
    );
    std::fs::write(codex_home.join("config.toml"), config_content).expect("write config");

    for (name, content) in &config.fixture_files {
        let path = tmp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, content).expect("write fixture");
    }

    let binary = codex_binary_path();
    let output = Command::new(&binary)
        .arg("exec")
        .arg("--json")
        .arg("--skip-git-repo-check")
        .arg(&config.prompt)
        .env("CODEX_HOME", codex_home.to_str().unwrap())
        .env("ANTHROPIC_API_KEY", &api_key)
        .env("CODEX_SANDBOX_NETWORK_DISABLED", "")
        .current_dir(tmp_dir.path())
        .output()
        .expect("spawn codex");

    let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let raw_stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    let mut events = Vec::new();
    let mut response = String::new();
    let mut input_tokens: i64 = 0;
    let mut output_tokens: i64 = 0;

    for line in raw_stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<JsonlEvent>(line) {
            if event.kind == "item.completed" {
                if let Some(item) = event.data.get("item") {
                    if item.get("type").and_then(|t| t.as_str()) == Some("agent_message") {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            if !response.is_empty() {
                                response.push('\n');
                            }
                            response.push_str(text);
                        }
                    }
                }
            }
            if event.kind == "turn.completed" {
                if let Some(usage) = event.data.get("usage") {
                    input_tokens = usage
                        .get("input_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    output_tokens = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                }
            }
            events.push(event);
        }
    }

    ProxyRunResult {
        events,
        response,
        exit_code,
        input_tokens,
        output_tokens,
        raw_stderr,
    }
}

fn codex_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("CODEX_BINARY") {
        return PathBuf::from(path);
    }
    codex_utils_cargo_bin::cargo_bin("codex").expect("codex binary not found")
}

// ─── Smoke Tests ───

#[test]
fn claude_basic_prompt_via_messages_api() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt: "What is 2+2? Answer with just the number.".to_string(),
        ..Default::default()
    });

    assert_eq!(
        result.exit_code, 0,
        "exit code should be 0\nstderr: {}",
        result.raw_stderr
    );
    assert!(
        result.response.contains('4'),
        "response should contain '4', got: {}",
        result.response
    );
    assert!(result.input_tokens > 0, "should report input tokens");
    assert!(result.output_tokens > 0, "should report output tokens");
}

#[test]
fn claude_streaming_produces_nonempty_response() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt: "Write a haiku about rust programming.".to_string(),
        ..Default::default()
    });

    assert_eq!(result.exit_code, 0, "stderr: {}", result.raw_stderr);
    assert!(
        result.response.len() > 20,
        "response should be non-trivial, got {} chars: {}",
        result.response.len(),
        result.response
    );
}

#[test]
fn claude_jsonl_event_structure() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt: "Say hello.".to_string(),
        ..Default::default()
    });

    let event_types: Vec<_> = result.events.iter().map(|e| e.kind.as_str()).collect();
    assert!(
        event_types.contains(&"thread.started"),
        "events: {event_types:?}"
    );
    assert!(
        event_types.contains(&"turn.started"),
        "events: {event_types:?}"
    );
    assert!(
        event_types.contains(&"item.completed"),
        "events: {event_types:?}"
    );
    assert!(
        event_types.contains(&"turn.completed"),
        "events: {event_types:?}"
    );
}

// ─── Tool Tests ───

#[test]
fn claude_reads_file_via_tool_call() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt: "Read the file secret.txt and tell me the secret number. Just the number."
            .to_string(),
        fixture_files: HashMap::from([(
            "secret.txt".to_string(),
            "The secret number is 42.".to_string(),
        )]),
        ..Default::default()
    });

    assert_eq!(result.exit_code, 0, "stderr: {}", result.raw_stderr);
    assert!(
        result.response.contains("42"),
        "response should contain '42', got: {}",
        result.response
    );
}

#[test]
fn claude_runs_shell_command() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt: "Run 'echo MESSAGES_WIRE_OK' in the shell and tell me exactly what it printed."
            .to_string(),
        ..Default::default()
    });

    assert_eq!(result.exit_code, 0, "stderr: {}", result.raw_stderr);
    assert!(
        result.response.contains("MESSAGES_WIRE_OK"),
        "response should contain 'MESSAGES_WIRE_OK', got: {}",
        result.response
    );
}

#[test]
fn claude_multi_tool_chain() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt:
            "Create a file called chain_test.txt containing 'hello chain'. Then read it back and tell me what it says."
                .to_string(),
        ..Default::default()
    });

    assert_eq!(result.exit_code, 0, "stderr: {}", result.raw_stderr);
    assert!(
        result.response.to_lowercase().contains("hello chain"),
        "response should contain file content, got: {}",
        result.response
    );
}

// ─── Thinking Tests ───

#[test]
fn claude_extended_thinking_produces_response() {
    if skip_unless_proxy_e2e() {
        return;
    }
    let result = run_codex_messages(RunConfig {
        prompt: "What is 15 * 17? Think step by step. Answer with just the number.".to_string(),
        reasoning_effort: "low".to_string(),
        ..Default::default()
    });

    assert_eq!(result.exit_code, 0, "stderr: {}", result.raw_stderr);
    assert!(
        result.response.contains("255"),
        "response should contain '255', got: {}",
        result.response
    );
}
