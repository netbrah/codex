use super::AuthRequestTelemetryContext;
use super::ModelClient;
use super::PendingUnauthorizedRetry;
use super::UnauthorizedRecoveryExecution;
use codex_otel::SessionTelemetry;
use codex_protocol::ThreadId;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use pretty_assertions::assert_eq;
use serde_json::json;

fn test_model_client(session_source: SessionSource) -> ModelClient {
    let provider = crate::model_provider_info::create_oss_provider_with_base_url(
        "https://example.com/v1",
        crate::model_provider_info::WireApi::Responses,
    );
    ModelClient::new(
        None,
        ThreadId::new(),
        provider,
        session_source,
        None,
        /*tool_choice*/ None,
        None,
        false,
        false,
        None,
    )
}

fn test_model_info() -> ModelInfo {
    serde_json::from_value(json!({
        "slug": "gpt-test",
        "display_name": "gpt-test",
        "description": "desc",
        "default_reasoning_level": "medium",
        "supported_reasoning_levels": [
            {"effort": "medium", "description": "medium"}
        ],
        "shell_type": "shell_command",
        "visibility": "list",
        "supported_in_api": true,
        "priority": 1,
        "upgrade": null,
        "base_instructions": "base instructions",
        "model_messages": null,
        "supports_reasoning_summaries": false,
        "support_verbosity": false,
        "default_verbosity": null,
        "apply_patch_tool_type": null,
        "truncation_policy": {"mode": "bytes", "limit": 10000},
        "supports_parallel_tool_calls": false,
        "supports_image_detail_original": false,
        "context_window": 272000,
        "auto_compact_token_limit": null,
        "experimental_supported_tools": []
    }))
    .expect("deserialize test model info")
}

fn test_session_telemetry() -> SessionTelemetry {
    SessionTelemetry::new(
        ThreadId::new(),
        "gpt-test",
        "gpt-test",
        None,
        None,
        None,
        "test-originator".to_string(),
        false,
        "test-terminal".to_string(),
        SessionSource::Cli,
    )
}

#[test]
fn build_subagent_headers_sets_other_subagent_label() {
    let client = test_model_client(SessionSource::SubAgent(SubAgentSource::Other(
        "memory_consolidation".to_string(),
    )));
    let headers = client.build_subagent_headers();
    let value = headers
        .get("x-openai-subagent")
        .and_then(|value| value.to_str().ok());
    assert_eq!(value, Some("memory_consolidation"));
}

#[tokio::test]
async fn summarize_memories_returns_empty_for_empty_input() {
    let client = test_model_client(SessionSource::Cli);
    let model_info = test_model_info();
    let session_telemetry = test_session_telemetry();

    let output = client
        .summarize_memories(Vec::new(), &model_info, None, &session_telemetry)
        .await
        .expect("empty summarize request should succeed");
    assert_eq!(output.len(), 0);
}

#[test]
fn auth_request_telemetry_context_tracks_attached_auth_and_retry_phase() {
    let auth_context = AuthRequestTelemetryContext::new(
        Some(crate::auth::AuthMode::Chatgpt),
        &crate::api_bridge::CoreAuthProvider::for_test(Some("access-token"), Some("workspace-123")),
        PendingUnauthorizedRetry::from_recovery(UnauthorizedRecoveryExecution {
            mode: "managed",
            phase: "refresh_token",
        }),
    );

    assert_eq!(auth_context.auth_mode, Some("Chatgpt"));
    assert!(auth_context.auth_header_attached);
    assert_eq!(auth_context.auth_header_name, Some("authorization"));
    assert!(auth_context.retry_after_unauthorized);
    assert_eq!(auth_context.recovery_mode, Some("managed"));
    assert_eq!(auth_context.recovery_phase, Some("refresh_token"));
}

mod tool_choice_tests {
    use super::ModelClient;
    use super::SessionSource;
    use super::ThreadId;
    use super::test_model_info;
    use codex_protocol::config_types::ToolChoice;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn client_with_tool_choice(tool_choice: Option<ToolChoice>) -> ModelClient {
        let provider = crate::model_provider_info::create_oss_provider_with_base_url(
            "https://example.com/v1",
            crate::model_provider_info::WireApi::Responses,
        );
        ModelClient::new(
            None,
            ThreadId::new(),
            provider,
            SessionSource::Cli,
            None,
            tool_choice,
            None,
            false,
            false,
            None,
        )
    }

    // --- Responses API (OpenAI) tool_choice conversion tests ---

    #[test]
    fn responses_api_default_none_is_auto() {
        let client = client_with_tool_choice(None);
        let session = client.new_session();
        assert_eq!(session.responses_api_tool_choice(), "auto");
    }

    #[test]
    fn responses_api_auto() {
        let client = client_with_tool_choice(Some(ToolChoice::Auto));
        let session = client.new_session();
        assert_eq!(session.responses_api_tool_choice(), "auto");
    }

    #[test]
    fn responses_api_required() {
        let client = client_with_tool_choice(Some(ToolChoice::Required));
        let session = client.new_session();
        assert_eq!(session.responses_api_tool_choice(), "required");
    }

    #[test]
    fn responses_api_none() {
        let client = client_with_tool_choice(Some(ToolChoice::None));
        let session = client.new_session();
        assert_eq!(session.responses_api_tool_choice(), "none");
    }

    #[test]
    fn responses_api_specific_maps_to_required() {
        let client = client_with_tool_choice(Some(ToolChoice::Specific {
            name: "shell".to_string(),
        }));
        let session = client.new_session();
        assert_eq!(session.responses_api_tool_choice(), "required");
    }

    // --- Messages API (Anthropic) tool_choice conversion tests ---

    #[test]
    fn messages_api_default_none_is_auto() {
        let client = client_with_tool_choice(None);
        let session = client.new_session();
        assert_eq!(session.messages_api_tool_choice(), json!({"type": "auto"}));
    }

    #[test]
    fn messages_api_auto() {
        let client = client_with_tool_choice(Some(ToolChoice::Auto));
        let session = client.new_session();
        assert_eq!(session.messages_api_tool_choice(), json!({"type": "auto"}));
    }

    #[test]
    fn messages_api_required_maps_to_any() {
        let client = client_with_tool_choice(Some(ToolChoice::Required));
        let session = client.new_session();
        assert_eq!(session.messages_api_tool_choice(), json!({"type": "any"}));
    }

    #[test]
    fn messages_api_specific_includes_name() {
        let client = client_with_tool_choice(Some(ToolChoice::Specific {
            name: "bash_20250306".to_string(),
        }));
        let session = client.new_session();
        assert_eq!(
            session.messages_api_tool_choice(),
            json!({"type": "tool", "name": "bash_20250306"})
        );
    }

    #[test]
    fn messages_api_none_falls_back_to_auto() {
        // Anthropic doesn't support "none" tool_choice; the caller omits
        // tools entirely. The method falls back to "auto".
        let client = client_with_tool_choice(Some(ToolChoice::None));
        let session = client.new_session();
        assert_eq!(session.messages_api_tool_choice(), json!({"type": "auto"}));
    }

    // --- Full request building tests ---

    #[test]
    fn build_responses_request_uses_configured_tool_choice() {
        let provider = crate::model_provider_info::create_oss_provider_with_base_url(
            "https://example.com/v1",
            crate::model_provider_info::WireApi::Responses,
        );
        let api_provider = codex_api::Provider {
            name: "test".to_string(),
            base_url: "https://example.com/v1".to_string(),
            query_params: None,
            headers: Default::default(),
            retry: codex_api::provider::RetryConfig {
                max_attempts: 1,
                base_delay: std::time::Duration::from_millis(100),
                retry_429: false,
                retry_5xx: false,
                retry_transport: false,
            },
            stream_idle_timeout: std::time::Duration::from_secs(30),
        };
        let client = ModelClient::new(
            None,
            ThreadId::new(),
            provider,
            SessionSource::Cli,
            None,
            Some(ToolChoice::Required),
            None,
            false,
            false,
            None,
        );
        let session = client.new_session();

        let model_info = test_model_info();
        let prompt = crate::client_common::Prompt::default();
        let request = session
            .build_responses_request(
                &api_provider,
                &prompt,
                &model_info,
                Option::None,
                codex_protocol::config_types::ReasoningSummary::Auto,
                Option::None,
                crate::config::SamplingParams::default(),
            )
            .expect("build request");
        assert_eq!(request.tool_choice, "required");
    }
}

#[test]
fn test_anthropic_max_output_tokens_claude_opus() {
    assert_eq!(super::anthropic_max_output_tokens("claude-opus-4-6"), 128_000);
}

#[test]
fn test_anthropic_max_output_tokens_claude_haiku() {
    assert_eq!(super::anthropic_max_output_tokens("claude-haiku-3-5"), 8_192);
}

#[test]
fn test_anthropic_max_output_tokens_claude_sonnet() {
    assert_eq!(super::anthropic_max_output_tokens("claude-sonnet-4-6"), 64_000);
}

#[test]
fn test_anthropic_max_output_tokens_proxy_opus_no_claude_prefix() {
    // Proxy model names that happen to contain "opus" should NOT get 128K
    assert_eq!(super::anthropic_max_output_tokens("my-opus-proxy"), 64_000);
}

#[test]
fn test_anthropic_max_output_tokens_proxy_opus_with_company() {
    // Company-namespaced model with "opus" substring should NOT get 128K
    assert_eq!(super::anthropic_max_output_tokens("company/opus-tuned"), 64_000);
}

#[test]
fn test_anthropic_max_output_tokens_real_anthropic_slug() {
    // Real Anthropic model slug format
    assert_eq!(
        super::anthropic_max_output_tokens("claude-3-opus-20240229"),
        128_000
    );
}

// ── T-4-B: anthropic_max_output_tokens extended coverage ───────────

#[test]
fn test_anthropic_max_output_tokens_vertex_aliased_slug() {
    // Proxy may use vertex_ai/claude-sonnet-4-6 format — doesn't start with "claude"
    let cap = super::anthropic_max_output_tokens("vertex_ai/claude-sonnet-4-6");
    assert_eq!(cap, 64_000, "vertex_ai prefix means not starts_with(claude) — gets default");
}

#[test]
fn test_anthropic_max_output_tokens_unknown_claude_model() {
    let cap = super::anthropic_max_output_tokens("claude-unknown-model-999");
    assert!(
        cap > 0 && cap <= 200_000,
        "unknown Claude model should get a sane default: {cap}"
    );
}

#[test]
fn test_anthropic_max_output_tokens_case_insensitive() {
    assert_eq!(super::anthropic_max_output_tokens("Claude-Opus-4-6"), 128_000);
    assert_eq!(super::anthropic_max_output_tokens("CLAUDE-HAIKU-3-5"), 8_192);
}

#[test]
fn test_anthropic_max_output_tokens_non_claude_default() {
    assert_eq!(super::anthropic_max_output_tokens("gpt-5"), 64_000);
    assert_eq!(super::anthropic_max_output_tokens("llama-3"), 64_000);
}

// ── T-1-F: WireApi serde tests (merge regression guard) ────────────

#[test]
fn wire_api_deserializes_messages() {
    use crate::model_provider_info::WireApi;
    let wire_api: WireApi =
        serde_json::from_str(r#""messages""#).expect("WireApi should deserialize 'messages'");
    assert!(matches!(wire_api, WireApi::Messages));
}

#[test]
fn wire_api_deserializes_responses() {
    use crate::model_provider_info::WireApi;
    let wire_api: WireApi = serde_json::from_str(r#""responses""#).unwrap();
    assert!(matches!(wire_api, WireApi::Responses));
}

#[test]
fn wire_api_rejects_unknown() {
    use crate::model_provider_info::WireApi;
    let result: Result<WireApi, _> = serde_json::from_str(r#""generateContent""#);
    assert!(result.is_err(), "unknown wire_api should be rejected");
}

#[test]
fn wire_api_rejects_chat_with_error() {
    use crate::model_provider_info::WireApi;
    let err = serde_json::from_str::<WireApi>(r#""chat""#).expect_err("'chat' should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("removed") || msg.contains("chat") || msg.contains("no longer"),
        "error should explain chat removal: {msg}"
    );
}

#[test]
fn wire_api_messages_round_trips() {
    use crate::model_provider_info::WireApi;
    let original = WireApi::Messages;
    let serialized = serde_json::to_string(&original).unwrap();
    assert_eq!(serialized, r#""messages""#);
    let back: WireApi = serde_json::from_str(&serialized).unwrap();
    assert!(matches!(back, WireApi::Messages));
}
