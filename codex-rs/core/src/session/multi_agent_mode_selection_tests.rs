//! Unit tests for `effective_multi_agent_mode` (apex-xt2.9 item 2, spec
//! `docs/onprem-mode-catalog-spec.md` §5.3/§7.2).
//!
//! The selector is driven through the real seam constructors
//! (`Session::make_turn_context` + `StepContext::for_test`) so the
//! assertions run against the same settings/config state production turns
//! capture. Fixtures use a catalogue model carrying the BUNDLED multi-agent
//! messages (no catalogue `mode.proactive`/`mode.explicit`/`hint_text`
//! overrides), so the expected results are the builtin enums plus bundled
//! text.

use std::sync::Arc;

use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_models_manager::test_support::construct_model_info_offline_for_tests;
use codex_models_manager::test_support::get_model_offline_for_tests;
use codex_otel::SessionTelemetry;
use codex_otel::TelemetryAuthMode;
use codex_prompts::ResolvedModelMessages;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::config_types::MultiAgentMode;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::MultiAgentMessages;
use codex_protocol::openai_models::MultiAgentModeMessages;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::InternalSessionSource;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_skills_extension::HostSkillsSnapshot;
use codex_skills_extension::SkillLoadOutcome;
use pretty_assertions::assert_eq;

use crate::config::Config;
use crate::context::world_state::MultiAgentModeState;
use crate::context::world_state::PreviousSectionState;
use crate::context::world_state::WorldStateSection;
use crate::environment_selection::TurnEnvironmentSnapshot;
use crate::session::multi_agents::effective_multi_agent_mode;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::step_settings::ResolvedStepSettings;
use crate::session::tests::make_session_configuration_for_tests;
use crate::shell::default_user_shell;
use crate::test_support::models_manager_with_provider;

/// Builds a real `StepContext` for the selection function with per-test
/// overrides: multi-agent version, session source, effective reasoning
/// effort, and direct config/model-info mutators.
async fn make_step_context(
    multi_agent_version: MultiAgentVersion,
    session_source: SessionSource,
    reasoning_effort: Option<ReasoningEffort>,
    config_mutator: impl FnOnce(&mut Config),
    model_mutator: impl FnOnce(&mut ModelInfo),
) -> Arc<StepContext> {
    let mut session_configuration = make_session_configuration_for_tests().await;
    let mut per_turn_config = session_configuration
        .original_config_do_not_use
        .as_ref()
        .clone();
    config_mutator(&mut per_turn_config);

    let model = get_model_offline_for_tests(per_turn_config.model.as_deref());
    let mut model_info = construct_model_info_offline_for_tests(
        model.as_str(),
        &per_turn_config.to_models_manager_config(),
    );
    model_mutator(&mut model_info);

    let mut step_settings = session_configuration.step_settings.as_ref().clone();
    step_settings.collaboration_mode.settings.reasoning_effort = reasoning_effort;
    session_configuration.session_source = session_source.clone();

    let thread_id = ThreadId::new();
    let auth_manager = AuthManager::from_auth_for_testing(CodexAuth::from_api_key("Test API Key"));
    let models_manager = models_manager_with_provider(
        per_turn_config.codex_home.to_path_buf(),
        auth_manager.clone(),
        per_turn_config.model_provider.clone(),
    );
    let session_telemetry = SessionTelemetry::new(
        thread_id,
        model.as_str(),
        model_info.slug.as_str(),
        /*account_id*/ None,
        Some("test@test.com".to_string()),
        Some(TelemetryAuthMode::Chatgpt),
        "test_originator".to_string(),
        /*log_user_prompts*/ false,
        "test".to_string(),
        session_source,
    );
    let user_shell = default_user_shell();
    let turn_context = Session::make_turn_context(
        thread_id,
        SessionId::from(thread_id),
        Some(Arc::clone(&auth_manager)),
        &session_telemetry,
        session_configuration.provider.clone(),
        &session_configuration,
        multi_agent_version,
        &user_shell,
        /*shell_zsh_path*/ None,
        /*main_execve_wrapper_exe*/ None,
        per_turn_config,
        Arc::new(ResolvedStepSettings::new(
            Arc::new(step_settings),
            Arc::new(model_info),
            /*fast_mode_enabled*/ false,
        )),
        &models_manager,
        /*network*/ None,
        TurnEnvironmentSnapshot::default(),
        session_configuration.cwd().clone(),
        "turn_id".to_string(),
        HostSkillsSnapshot::new(Arc::new(SkillLoadOutcome::default())),
    );
    StepContext::for_test(Arc::new(turn_context))
}

/// Renders the developer fragment a mode emits from an empty prior state, so
/// tests can pin the exact text that rides with the selected enum.
fn rendered_mode_body(mode: &MultiAgentMode) -> String {
    let state = MultiAgentModeState::new(Some(mode.clone()));
    let fragment = WorldStateSection::render_diff(&state, PreviousSectionState::Absent)
        .expect("mode change from absent state must render");
    fragment.body()
}

// T2a: v2 + no hint + non-Ultra effort (on-prem xhigh default) selects the
// Proactive builtin AND the rendered body is the bundled proactive text —
// the fork-seam ratchet.
#[tokio::test]
async fn xhigh_without_hint_defaults_to_proactive_with_bundled_text() {
    let step_context = make_step_context(
        MultiAgentVersion::V2,
        SessionSource::Cli,
        Some(ReasoningEffort::XHigh),
        |_| {},
        |_| {},
    )
    .await;

    let mode = effective_multi_agent_mode(&step_context).expect("v2 root session selects a mode");
    assert_eq!(mode, MultiAgentMode::Proactive);
    assert_eq!(
        rendered_mode_body(&mode),
        ResolvedModelMessages::bundled()
            .multi_agent()
            .proactive
            .text()
    );
}

// T2b: v2 + no hint + Ultra keeps the upstream Proactive path intact.
#[tokio::test]
async fn ultra_without_hint_stays_proactive() {
    let step_context = make_step_context(
        MultiAgentVersion::V2,
        SessionSource::Cli,
        Some(ReasoningEffort::Ultra),
        |_| {},
        |_| {},
    )
    .await;

    assert_eq!(
        effective_multi_agent_mode(&step_context),
        Some(MultiAgentMode::Proactive)
    );
}

// T2c: a configured `multi_agent_mode_hint_text` keeps full precedence.
#[tokio::test]
async fn config_hint_text_keeps_precedence_over_default() {
    let step_context = make_step_context(
        MultiAgentVersion::V2,
        SessionSource::Cli,
        Some(ReasoningEffort::XHigh),
        |config| {
            config.multi_agent_v2.multi_agent_mode_hint_text = Some("H".to_string());
        },
        |_| {},
    )
    .await;

    assert_eq!(
        effective_multi_agent_mode(&step_context),
        Some(MultiAgentMode::Custom("H".to_string()))
    );
}

// T2d: a catalogue `mode.hint_text` keeps full precedence.
#[tokio::test]
async fn catalogue_hint_text_keeps_precedence_over_default() {
    let step_context = make_step_context(
        MultiAgentVersion::V2,
        SessionSource::Cli,
        Some(ReasoningEffort::XHigh),
        |_| {},
        |model_info| {
            let model_messages = model_info
                .model_messages
                .get_or_insert_with(Default::default);
            model_messages.multi_agent = Some(MultiAgentMessages {
                role: None,
                mode: Some(MultiAgentModeMessages {
                    explicit: None,
                    proactive: None,
                    hint_text: Some("C".to_string()),
                }),
            });
        },
    )
    .await;

    assert_eq!(
        effective_multi_agent_mode(&step_context),
        Some(MultiAgentMode::Custom("C".to_string()))
    );
}

// T2e: v1 selects no mode at all.
#[tokio::test]
async fn v1_session_selects_no_mode() {
    let step_context = make_step_context(
        MultiAgentVersion::V1,
        SessionSource::Cli,
        Some(ReasoningEffort::XHigh),
        |_| {},
        |_| {},
    )
    .await;

    assert_eq!(effective_multi_agent_mode(&step_context), None);
}

// T2e: internal session sources select no mode.
#[tokio::test]
async fn internal_session_source_selects_no_mode() {
    let step_context = make_step_context(
        MultiAgentVersion::V2,
        SessionSource::Internal(InternalSessionSource::Guardian),
        Some(ReasoningEffort::XHigh),
        |_| {},
        |_| {},
    )
    .await;

    assert_eq!(effective_multi_agent_mode(&step_context), None);
}

// T2e: non-ThreadSpawn subagents select no mode.
#[tokio::test]
async fn non_thread_spawn_subagent_selects_no_mode() {
    let step_context = make_step_context(
        MultiAgentVersion::V2,
        SessionSource::SubAgent(SubAgentSource::Review),
        Some(ReasoningEffort::XHigh),
        |_| {},
        |_| {},
    )
    .await;

    assert_eq!(effective_multi_agent_mode(&step_context), None);
}
