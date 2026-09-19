//! Ratchet A coverage for string integer coercion of tool arguments.
//!
//! Tests 1-2 are the behavior change: string integer literals become
//! accepted, and malformed strings are rejected with the pinned integer
//! error. The remaining tests pin the no-regression guarantee that plain
//! JSON numbers still parse and execute for every touched tool.
use anyhow::Result;
use codex_core::config::CurrentTimeReminderConfig;
use codex_features::CurrentTimeSource;
use codex_features::Feature;
use codex_protocol::ThreadId;
use core_test_support::assert_regex_match;
use core_test_support::responses::ResponsesRequest;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_custom_tool_call;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_function_call_with_namespace;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_function_call_agent_response;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_host_windows;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

// Byte-identical copy of the private `expected an integer` message owned by
// codex-tools `strict_int` (see `strict_int_tests.rs` there). Drift pair:
// change both when one changes.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

// Suite-level v2 multi-agent namespace: the suite's built-in `openai`
// provider has `namespace_tools: true` and `multi_agent_v2.tool_namespace`
// defaults to `Some("collaboration")`, so plain-named v2 handlers only route
// through the namespaced call form.
const MULTI_AGENT_V2_NAMESPACE: &str = "collaboration";

fn enable_sleep_tool(config: &mut codex_core::config::Config) {
    config.include_environment_context = false;
    config
        .features
        .enable(Feature::CurrentTimeReminder)
        .expect("test config should allow current-time reminders");
    config.current_time_reminder = Some(CurrentTimeReminderConfig {
        reminder_interval_seconds: 3_000,
        clock_source: CurrentTimeSource::System,
        ..CurrentTimeReminderConfig::default()
    });
    config
        .current_time_reminder
        .as_mut()
        .expect("current-time reminder config should be present")
        .sleep_tool = true;
}

fn custom_tool_output_items(req: &ResponsesRequest, call_id: &str) -> Vec<Value> {
    match req.custom_tool_call_output(call_id).get("output") {
        Some(Value::Array(items)) => items.clone(),
        Some(Value::String(text)) => {
            vec![serde_json::json!({ "type": "input_text", "text": text })]
        }
        _ => panic!("custom tool output should be serialized as text or content items"),
    }
}

fn function_tool_output_items(req: &ResponsesRequest, call_id: &str) -> Vec<Value> {
    match req.function_call_output(call_id).get("output") {
        Some(Value::Array(items)) => items.clone(),
        Some(Value::String(text)) => {
            vec![serde_json::json!({ "type": "input_text", "text": text })]
        }
        _ => panic!("function tool output should be serialized as text or content items"),
    }
}

fn text_item(items: &[Value], index: usize) -> &str {
    items[index]
        .get("text")
        .and_then(Value::as_str)
        .expect("content item should be input_text")
}

fn extract_running_cell_id(text: &str) -> String {
    text.strip_prefix("Script running with cell ID ")
        .and_then(|rest| rest.split('\n').next())
        .expect("running header should contain a cell ID")
        .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_string_int_args_execute_end_to_end() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call(
                    "exec-strings",
                    "exec_command",
                    r#"{"cmd":"echo RATCHET-A-OK","timeout_ms":"120000","yield_time_ms":"10000"}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("run the ratchet probe").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let output = requests[1]
        .function_call_output_text("exec-strings")
        .expect("exec_command should produce function call output");
    assert!(
        !output.contains("failed to parse function arguments"),
        "string integer arguments should parse, got: {output}"
    );
    assert!(
        output.contains("RATCHET-A-OK"),
        "command stdout should be reported, got: {output}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_sleep_float_string_rejected_with_pinned_error() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call_with_namespace(
                    "sleep-1",
                    "clock",
                    "sleep",
                    r#"{"duration_ms":"12.5"}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| enable_sleep_tool(config))
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("sleep a bit").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let output = requests[1]
        .function_call_output_text("sleep-1")
        .expect("sleep should produce function call output");
    assert!(
        output.contains(EXPECTED_INTEGER_ERROR),
        "float string should be rejected with the pinned integer error, got: {output}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_exec_command_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call(
                    "exec-num",
                    "exec_command",
                    r#"{"cmd":"echo RATCHET-A-NUM","timeout_ms":120000,"yield_time_ms":10000,"max_output_tokens":1000}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("run the number probe").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let output = requests[1]
        .function_call_output_text("exec-num")
        .expect("exec_command should produce function call output");
    assert!(
        !output.contains("failed to parse function arguments"),
        "JSON number arguments should still parse, got: {output}"
    );
    assert!(
        output.contains("RATCHET-A-NUM"),
        "command stdout should be reported, got: {output}"
    );
    assert_regex_match(
        r"(?s)^(?:Chunk ID: [^\n]+\n)?Wall time: [0-9]+(?:\.[0-9]+)? seconds\nProcess exited with code 0\n(?:Original token count: \d+\n)?Output:\nRATCHET-A-NUM\n?$",
        &output,
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_write_stdin_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_host_windows!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call(
                    "wstd-open",
                    "exec_command",
                    r#"{"cmd":"/bin/bash -i","yield_time_ms":200,"tty":true}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_function_call(
                    "wstd-write",
                    "write_stdin",
                    r#"{"session_id":1000,"chars":"echo WSTDIN-RATCHET\n","yield_time_ms":800,"max_output_tokens":1000}"#,
                ),
                ev_completed("resp-2"),
            ]),
            sse(vec![
                ev_response_created("resp-3"),
                ev_assistant_message("msg-3", "done"),
                ev_completed("resp-3"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("open a shell and write to it").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 3);
    let output = requests[2]
        .function_call_output_text("wstd-write")
        .expect("write_stdin should produce function call output");
    assert!(
        !output.contains("failed to parse function arguments"),
        "JSON number arguments should still parse, got: {output}"
    );
    assert!(
        !output.contains("Unknown process id 1000"),
        "write_stdin should resolve the live session, got: {output}"
    );
    assert!(
        output.contains("Process running with session ID 1000"),
        "write_stdin should report the live session still running, got: {output}"
    );
    assert!(
        output.contains("WSTDIN-RATCHET"),
        "written command output should be captured from the live session, got: {output}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_sleep_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call_with_namespace(
                    "sleep-1",
                    "clock",
                    "sleep",
                    r#"{"duration_ms":5}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| enable_sleep_tool(config))
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("sleep a bit").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let sleep_output = requests[1]
        .function_call_output_text("sleep-1")
        .expect("sleep should produce function call output");
    assert!(
        sleep_output.ends_with("Sleep completed."),
        "sleep with a JSON number duration should complete, got: {sleep_output}"
    );
    assert_regex_match(
        r"^Wall time: [0-9]+\.[0-9][0-9][0-9][0-9] seconds\nSleep completed\.\z",
        &sleep_output,
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_code_mode_wait_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let mut builder = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            let _ = config.features.enable(Feature::CodeMode);
            let _ = config.features.enable(Feature::CodeModeHost);
            let _ = config.features.enable(Feature::ExecutedToolCallMetadata);
            config.code_mode.disable_in_process_fallback = true;
        });
    let test = builder.build_with_auto_env(&server).await?;
    let started = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_custom_tool_call(
                    "call-exec",
                    "exec",
                    r#"await tools.test_sync_tool({}); text("started"); yield_control(); await new Promise(() => {});"#,
                ),
                ev_completed("resp-start"),
            ]),
            sse(vec![
                ev_assistant_message("msg-start", "running"),
                ev_completed("resp-start-done"),
            ]),
        ],
    )
    .await;
    test.submit_turn("Start a cell").await?;
    let requests = started.requests();
    assert_eq!(requests.len(), 2);
    let first_items = custom_tool_output_items(&requests[1], "call-exec");
    let cell_id = extract_running_cell_id(text_item(&first_items, /*index*/ 0));

    let yielded = mount_function_call_agent_response(
        &server,
        "call-wait",
        &json!({ "cell_id": cell_id, "yield_time_ms": 1 }).to_string(),
        "wait",
    )
    .await;
    test.submit_turn("Wait once").await?;
    yielded.function_call.single_request();
    let yielded_items =
        function_tool_output_items(&yielded.completion.single_request(), "call-wait");
    assert_eq!(
        extract_running_cell_id(text_item(&yielded_items, /*index*/ 0)),
        cell_id,
    );
    assert_eq!(yielded_items.len(), 1);
    assert_regex_match(
        &format!(r"^Script running with cell ID {cell_id}\nWall time \d+\.\d seconds\nOutput:\n\z"),
        text_item(&yielded_items, /*index*/ 0),
    );
    let wait_output = yielded_items
        .iter()
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !wait_output.contains("failed to parse function arguments"),
        "JSON number wait arguments should still parse, got: {wait_output}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_wait_agent_v2_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call_with_namespace(
                    "wait-v2",
                    MULTI_AGENT_V2_NAMESPACE,
                    "wait_agent",
                    r#"{"timeout_ms":50}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| {
            let _ = config.features.enable(Feature::MultiAgentV2);
            config.multi_agent_v2.min_wait_timeout_ms = 50;
        })
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("wait for agents").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[1].function_call_output_text("wait-v2").as_deref(),
        Some(r#"{"message":"Wait timed out.","timed_out":true}"#),
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_wait_agent_v1_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let missing_thread = ThreadId::new();
    let wait_args = json!({
        "targets": [missing_thread.to_string()],
        "timeout_ms": 1000,
    });
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call_with_namespace(
                    "wait-v1",
                    "multi_agent_v1",
                    "wait_agent",
                    &wait_args.to_string(),
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| {
            let _ = config.features.enable(Feature::Collab);
        })
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("wait for agents").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let expected_status = json!({ missing_thread.to_string(): "not_found" }).to_string();
    let expected = format!(r#"{{"status":{expected_status},"timed_out":false}}"#);
    assert_eq!(
        requests[1].function_call_output_text("wait-v1").as_deref(),
        Some(expected.as_str()),
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_a_test_sync_number_args_no_regression() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let sync_args = json!({
        "sleep_before_ms": 1,
        "sleep_after_ms": 1,
        "barrier": {
            "id": "ratchet-a-sync",
            "participants": 1,
            "timeout_ms": 5000,
        },
    });
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call("sync-1", "test_sync_tool", &sync_args.to_string()),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("sync").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let output = requests[1]
        .function_call_output_text("sync-1")
        .expect("test_sync_tool should produce function call output");
    assert!(
        !output.contains("failed to parse function arguments"),
        "JSON number arguments should still parse, got: {output}"
    );
    assert_eq!(output, "ok");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ratchet_b_wait_agent_unknown_field_names_tool_and_parameter() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call_with_namespace(
                    "wait-b",
                    MULTI_AGENT_V2_NAMESPACE,
                    "wait_agent",
                    r#"{"target":"task_1"}"#,
                ),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| {
            let _ = config.features.enable(Feature::MultiAgentV2);
            config.multi_agent_v2.min_wait_timeout_ms = 50;
        })
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("wait for agents").await?;

    let requests = responses.requests();
    assert_eq!(requests.len(), 2);
    let output = requests[1]
        .function_call_output_text("wait-b")
        .expect("wait_agent should produce function call output");
    assert!(
        output.contains("failed to parse arguments for wait_agent"),
        "parse error should name the tool, got: {output}"
    );
    assert!(
        output.contains("(parameter \"target\")"),
        "parse error should name the unknown parameter, got: {output}"
    );

    Ok(())
}
