use super::*;
use codex_apply_patch::MaybeApplyPatchVerified;
use codex_exec_server::LOCAL_FS;
use codex_login::CodexAuth;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
use codex_protocol::models::SearchToolCallParams;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSandboxPolicyContext;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::FileChange;
use codex_utils_absolute_path::AbsolutePathBuf;
use core_test_support::PathBufExt;
use core_test_support::PathExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

fn local_context(cwd: &PathUri) -> FileSystemSandboxPolicyContext<'_> {
    FileSystemSandboxPolicyContext {
        cwd,
        workspace_roots: std::slice::from_ref(cwd),
        user_home_dir: None,
        temporary_directories: None,
    }
}

use crate::config::Constrained;
use crate::config::Permissions;
use crate::session::step_context::StepContext;
use crate::session::tests::make_session_and_context;
use crate::session::tests::make_session_and_context_with_auth_and_config_and_rx;
use crate::tools::context::ToolInvocation;
use crate::tools::hook_names::HookToolName;
use crate::tools::registry::PostToolUsePayload;
use crate::tools::registry::PreToolUsePayload;
use crate::turn_diff_tracker::TurnDiffTracker;

fn sample_patch() -> &'static str {
    r#"*** Begin Patch
*** Add File: hello.txt
+hello
*** End Patch"#
}

async fn invocation_from_session(
    payload: ToolPayload,
    session: Arc<Session>,
    turn: Arc<TurnContext>,
) -> (ToolInvocation, PathUri) {
    let step_context = StepContext::for_test(Arc::clone(&turn));
    let cwd = resolve_tool_environment(&step_context.environments, /*environment_id*/ None)
        .expect("primary turn environment must resolve")
        .expect("primary turn environment must exist")
        .cwd()
        .clone();
    (
        ToolInvocation {
            session,
            step_context,
            turn,
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            tracker: Arc::new(Mutex::new(TurnDiffTracker::new())),
            call_id: "call-apply-patch".to_string(),
            tool_name: codex_tools::ToolName::plain("apply_patch"),
            source: crate::tools::context::ToolCallSource::Direct,
            payload,
        },
        cwd,
    )
}

async fn invocation_for_payload(payload: ToolPayload) -> (ToolInvocation, PathUri) {
    let (session, turn) = make_session_and_context().await;
    invocation_from_session(payload, Arc::new(session), Arc::new(turn)).await
}

#[tokio::test]
async fn file_update_mode_follows_preserve_line_endings_feature() {
    let (_, mut turn) = make_session_and_context().await;
    assert_eq!(
        apply_patch_file_update_mode(&turn),
        codex_apply_patch::ApplyPatchFileUpdateMode::NormalizeToLf
    );

    Arc::make_mut(&mut turn.config)
        .features
        .enable(codex_features::Feature::ApplyPatchPreserveLineEndings)
        .expect("feature should be enabled");
    assert_eq!(
        apply_patch_file_update_mode(&turn),
        codex_apply_patch::ApplyPatchFileUpdateMode::PreserveLineEndings
    );
}

#[tokio::test]
async fn pre_tool_use_payload_uses_freeform_patch_input() {
    let patch = sample_patch();
    let payload = ToolPayload::Custom {
        input: patch.to_string(),
    };
    let (invocation, _) = invocation_for_payload(payload).await;
    let handler = ApplyPatchHandler::default();

    assert_eq!(
        handler.pre_tool_use_payload(&invocation),
        Some(PreToolUsePayload {
            tool_name: HookToolName::apply_patch(),
            tool_input: json!({ "command": patch }),
        })
    );
}

#[tokio::test]
async fn post_tool_use_payload_uses_patch_input_and_tool_output() {
    let patch = sample_patch();
    let payload = ToolPayload::Custom {
        input: patch.to_string(),
    };
    let (invocation, _) = invocation_for_payload(payload).await;
    let output = ApplyPatchToolOutput::from_text("Success. Updated files.".to_string());
    let handler = ApplyPatchHandler::default();

    assert_eq!(
        handler.post_tool_use_payload(&invocation, &output),
        Some(PostToolUsePayload {
            tool_name: HookToolName::apply_patch(),
            tool_use_id: "call-apply-patch".to_string(),
            tool_input: json!({ "command": patch }),
            tool_response: json!("Success. Updated files."),
        })
    );
}

#[test]
fn diff_consumer_streams_apply_patch_changes() {
    let mut consumer = ApplyPatchArgumentDiffConsumer::default();
    assert!(
        consumer
            .push_delta("call-1".to_string(), "*** Begin Patch\n")
            .is_none()
    );

    let event = consumer
        .push_delta("call-1".to_string(), "*** Add File: hello.txt\n+hello")
        .expect("progress event");
    assert_eq!(
        (event.call_id, event.changes),
        (
            "call-1".to_string(),
            HashMap::from([(
                PathBuf::from("hello.txt"),
                FileChange::Add {
                    content: String::new(),
                },
            )]),
        )
    );

    assert!(
        consumer
            .push_delta("call-1".to_string(), "\n+world")
            .is_none()
    );
    assert!(
        consumer
            .push_delta("call-1".to_string(), "\n*** End Patch")
            .is_none()
    );

    let event = consumer
        .finish_update_on_complete()
        .expect("finish parser")
        .expect("progress event");
    assert_eq!(
        (event.call_id, event.changes),
        (
            "call-1".to_string(),
            HashMap::from([(
                PathBuf::from("hello.txt"),
                FileChange::Add {
                    content: "hello\nworld\n".to_string(),
                },
            )]),
        )
    );
}

#[test]
fn diff_consumer_streams_apply_patch_changes_with_environment_header() {
    let mut consumer = ApplyPatchArgumentDiffConsumer::default();
    assert!(
        consumer
            .push_delta(
                "call-1".to_string(),
                "*** Begin Patch\n*** Environment ID: remote\n",
            )
            .is_none()
    );

    let event = consumer
        .push_delta("call-1".to_string(), "*** Add File: hello.txt\n+hello")
        .expect("progress event");
    assert_eq!(
        event.changes,
        HashMap::from([(
            PathBuf::from("hello.txt"),
            FileChange::Add {
                content: String::new(),
            },
        )])
    );
}

#[test]
fn diff_consumer_sends_next_update_after_buffer_interval() {
    let mut consumer = ApplyPatchArgumentDiffConsumer::default();
    consumer.push_delta("call-1".to_string(), "*** Begin Patch\n");
    let first = consumer
        .push_delta("call-1".to_string(), "*** Add File: hello.txt\n+hello")
        .expect("first progress event");
    assert_eq!(
        first.changes,
        HashMap::from([(
            PathBuf::from("hello.txt"),
            FileChange::Add {
                content: String::new(),
            },
        )])
    );

    consumer.last_sent_at =
        Some(std::time::Instant::now() - APPLY_PATCH_ARGUMENT_DIFF_BUFFER_INTERVAL);
    let second = consumer
        .push_delta("call-1".to_string(), "\n+world")
        .expect("second progress event");
    assert_eq!(
        second.changes,
        HashMap::from([(
            PathBuf::from("hello.txt"),
            FileChange::Add {
                content: "hello\n".to_string(),
            },
        )])
    );
}

#[test]
fn reconcile_environment_id_requires_selection_when_enabled() {
    assert_eq!(
        require_environment_id(Some("remote"), /*allow_environment_id*/ false),
        Err(FunctionCallError::RespondToModel(
            "apply_patch environment selection is unavailable for this turn".to_string(),
        ))
    );
    assert_eq!(
        require_environment_id(
            /*parsed_environment_id*/ None, /*allow_environment_id*/ true
        ),
        Ok(None)
    );
}

#[tokio::test]
async fn approval_keys_include_move_destination() {
    let tmp = TempDir::new().expect("tmp");
    let cwd_path = tmp.path();
    let cwd = cwd_path.abs();
    std::fs::create_dir_all(cwd_path.join("old")).expect("create old dir");
    std::fs::create_dir_all(cwd_path.join("renamed/dir")).expect("create dest dir");
    std::fs::write(cwd_path.join("old/name.txt"), "old content\n").expect("write old file");
    let patch = r#"*** Begin Patch
*** Update File: old/name.txt
*** Move to: renamed/dir/name.txt
@@
-old content
+new content
*** End Patch"#;
    let argv = vec!["apply_patch".to_string(), patch.to_string()];
    // TODO(anp): Keep apply_patch handler test cwd values as PathUri.
    let cwd = PathUri::from_abs_path(&cwd);
    let action = match codex_apply_patch::maybe_parse_apply_patch_verified(
        &argv,
        &cwd,
        LOCAL_FS.as_ref(),
        /*sandbox*/ None,
    )
    .await
    {
        MaybeApplyPatchVerified::Body(action) => action,
        other => panic!("expected patch body, got: {other:?}"),
    };

    let keys = file_paths_for_action(&action);
    assert_eq!(keys.len(), 2);
}

#[test]
fn write_permissions_for_paths_skip_dirs_already_writable_under_workspace_root() {
    let tmp = TempDir::new().expect("tmp");
    let cwd_path = tmp.path();
    let cwd = cwd_path.abs();
    let nested = cwd_path.join("nested");
    std::fs::create_dir_all(&nested).expect("create nested dir");
    let file_path = AbsolutePathBuf::try_from(nested.join("file.txt"))
        .expect("nested file path should be absolute");
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ false,
    );

    let permissions = write_permissions_for_paths(
        &[file_path.into()],
        &sandbox_policy,
        &local_context(&cwd.into()),
        PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled),
    );

    assert_eq!(permissions, None);
}

#[test]
fn write_permissions_for_paths_keep_dirs_outside_workspace_root() {
    let tmp = TempDir::new().expect("tmp");
    let cwd = tmp.path().join("workspace");
    let outside = tmp.path().join("outside");
    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&outside).expect("create outside dir");
    let file_path = AbsolutePathBuf::try_from(outside.join("file.txt"))
        .expect("outside file path should be absolute");
    let cwd_abs = cwd.abs();
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );

    let permissions = write_permissions_for_paths(
        &[file_path.into()],
        &sandbox_policy,
        &local_context(&cwd_abs.into()),
        PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled),
    );
    let expected_outside = outside.abs();

    assert_eq!(
        permissions
            .and_then(|profile| profile.file_system)
            .and_then(|fs| fs.legacy_read_write_roots())
            .and_then(|roots| roots.write),
        Some(vec![expected_outside])
    );
}

#[test]
fn write_permissions_for_paths_do_not_widen_workspace_root_target() {
    let tmp = TempDir::new().expect("tmp");
    let cwd = tmp.path().join("workspace").abs();
    std::fs::create_dir_all(&cwd).expect("create workspace");
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );
    let permissions = write_permissions_for_paths(
        &[cwd.clone().into()],
        &sandbox_policy,
        &local_context(&cwd.into()),
        PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled),
    );

    assert_eq!(permissions, None);
}

#[test]
fn write_permissions_for_paths_do_not_regrant_an_already_writable_parent() {
    let tmp = TempDir::new().expect("tmp");
    let cwd = tmp.path().abs();
    let file_path = cwd.join("protected.txt");
    let sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(cwd.clone().into(), FileSystemAccessMode::Write),
        FileSystemSandboxEntry::new(file_path.clone().into(), FileSystemAccessMode::Read),
    ]);

    let permissions = write_permissions_for_paths(
        &[file_path.into()],
        &sandbox_policy,
        &local_context(&cwd.into()),
        PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled),
    );

    assert_eq!(permissions, None);
}

#[test]
fn write_permissions_for_windows_paths_uses_executor_uris() {
    let cwd = PathUri::parse("file:///C:/workspace").expect("Windows cwd");
    let context = local_context(&cwd);
    let policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );
    let outside = PathUri::parse("file:///C:/outside/out.txt").expect("outside");

    assert_eq!(
        write_permissions_for_paths(
            &[outside],
            &policy,
            &context,
            PatchSandboxRoute::ExecutorManaged,
        )
        .and_then(|profile| profile.file_system)
        .map(|permissions| permissions.entries),
        Some(vec![FileSystemSandboxEntry::new(
            PathUri::parse("file:///C:/outside")
                .expect("outside parent")
                .into(),
            FileSystemAccessMode::Write,
        )]),
    );
}

#[test]
fn with_environment_id_line_inserts_after_begin_patch_header() {
    let patch = "*** Begin Patch\n*** Add File: hello.txt\n+hi\n*** End Patch\n";
    let out = with_environment_id_line(patch.to_string(), "env-1");
    assert_eq!(
        out,
        "*** Begin Patch\n*** Environment ID: env-1\n*** Add File: hello.txt\n+hi\n*** End Patch\n"
    );
}

#[test]
fn with_environment_id_line_leaves_malformed_patch_unchanged() {
    let patch = "not a patch\n";
    assert_eq!(
        with_environment_id_line(patch.to_string(), "env-1"),
        "not a patch\n"
    );
}

#[test]
fn apply_patch_handler_patch_text_extracts_patch_argument() {
    let payload = ToolPayload::Function {
        arguments: r#"{"patch":"*** Begin Patch\n*** Add File: a.txt\n+x\n*** End Patch\n","environment_id":"env-1"}"#
            .to_string(),
    };
    assert_eq!(
        apply_patch_handler_patch_text(&payload).as_deref(),
        Some("*** Begin Patch\n*** Add File: a.txt\n+x\n*** End Patch\n")
    );
}

#[test]
fn apply_patch_handler_patch_text_extracts_custom_input_verbatim() {
    let input = "*** Begin Patch\n*** End Patch\n".to_string();
    let payload = ToolPayload::Custom {
        input: input.clone(),
    };
    assert_eq!(
        apply_patch_handler_patch_text(&payload),
        Some(input),
        "the extractor must return the Custom input verbatim (no JSON sniff)"
    );
}

#[test]
fn apply_patch_handler_patch_text_extracts_custom_input_json_shaped_verbatim() {
    // The exact wire shape the s5.2-rejected "JSON sniff" design would have
    // parsed (a JSON object with a string `patch` field): the extractor must
    // still return the raw JSON string verbatim, never the decoded patch.
    let input = r#"{"patch":"*** Begin Patch\n*** End Patch\n"}"#.to_string();
    let payload = ToolPayload::Custom {
        input: input.clone(),
    };
    assert_eq!(
        apply_patch_handler_patch_text(&payload),
        Some(input),
        "a JSON-shaped Custom input must still be returned verbatim (sniff rejected, s5.2)"
    );
}

#[tokio::test]
async fn function_apply_patch_rejects_missing_patch_argument_with_teachable_error() {
    let payload = ToolPayload::Function {
        arguments: r#"{}"#.to_string(),
    };
    let (invocation, _) = invocation_for_payload(payload).await;
    let handler = FunctionApplyPatchHandler::default();

    let err = match handler.handle(invocation).await {
        Err(err) => err,
        Ok(_) => panic!("a missing patch argument must be rejected"),
    };
    assert_eq!(
        err,
        FunctionCallError::RespondToModel(
            "apply_patch is missing the required 'patch' argument; pass the full patch text starting with '*** Begin Patch' in 'patch'".to_string(),
        )
    );
}

#[tokio::test]
async fn function_apply_patch_rejects_non_string_patch_argument_with_teachable_error() {
    let payload = ToolPayload::Function {
        arguments: json!({ "patch": { "raw": "not a string" } }).to_string(),
    };
    let (invocation, _) = invocation_for_payload(payload).await;
    let handler = FunctionApplyPatchHandler::default();

    let err = match handler.handle(invocation).await {
        Err(err) => err,
        Ok(_) => panic!("a non-string patch argument must be rejected"),
    };
    assert_eq!(
        err,
        FunctionCallError::RespondToModel(
            "apply_patch 'patch' argument must be a string containing the full patch text starting with '*** Begin Patch'".to_string(),
        )
    );
}

#[tokio::test]
async fn function_apply_patch_handler_accepts_custom_payload() {
    // Unit-level mirror of triad run A: a Custom (custom_tool_call)
    // apply_patch payload must apply. F1 shape (spec §1.1): raw markdown
    // Add-File — content lines carry no per-line `+` prefix and the first
    // content line is an H1. The literal mirrors the F1 test, including
    // the trailing newline after `*** End Patch` that `sample_patch()`
    // lacks.
    let patch = "*** Begin Patch\n*** Add File: xt214-t1/custom-payload-add.md\n# XT2.14 T1 — CUSTOM PAYLOAD ADD (RED PHASE)\nFreeze holds until the T1 RED evidence lands.\n\n| field | value |\n| --- | --- |\n*** End Patch\n";
    let payload = ToolPayload::Custom {
        input: patch.to_string(),
    };
    // The default test session is read-only with approval on request, so this
    // write would stall on an unanswered approval prompt; build the session
    // with the integration harness shape instead.
    let (session, turn, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config.permissions = Permissions::from_approval_and_profile(
                Constrained::allow_any(AskForApproval::Never),
                Constrained::allow_only(PermissionProfile::Disabled),
            )
            .expect("test permissions should be valid");
        },
    )
    .await;
    let (invocation, cwd) = invocation_from_session(payload, session, turn).await;
    let handler = FunctionApplyPatchHandler::default();

    match handler.handle(invocation).await {
        Ok(_) => {}
        Err(err) => panic!("a Custom-payload apply_patch call must apply: {err:?}"),
    }

    let file_path = cwd.to_path_buf().join("xt214-t1/custom-payload-add.md");
    assert_eq!(
        std::fs::read_to_string(&file_path)
            .expect("the Custom-payload Add-File must create the file"),
        "# XT2.14 T1 — CUSTOM PAYLOAD ADD (RED PHASE)\nFreeze holds until the T1 RED evidence lands.\n\n| field | value |\n| --- | --- |\n"
    );
    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_dir(cwd.to_path_buf().join("xt214-t1"));
}

#[test]
fn function_apply_patch_matches_kind_accepts_function_and_custom() {
    let handler = FunctionApplyPatchHandler::default();
    assert!(
        handler.matches_kind(&ToolPayload::Function {
            arguments: r#"{"patch":"*** Begin Patch\n*** End Patch\n"}"#.to_string(),
        }),
        "matches_kind must accept Function payloads"
    );
    assert!(
        handler.matches_kind(&ToolPayload::Custom {
            input: "*** Begin Patch\n*** End Patch\n".to_string(),
        }),
        "matches_kind must accept Custom payloads"
    );
    assert!(
        !handler.matches_kind(&ToolPayload::ToolSearch {
            arguments: SearchToolCallParams {
                query: "apply_patch".to_string(),
                limit: None,
            },
        }),
        "matches_kind must reject ToolSearch payloads"
    );
}

#[tokio::test]
async fn function_apply_patch_pre_and_post_hook_payloads_cover_both_payload_kinds() {
    let patch = "*** Begin Patch\n*** Add File: a.txt\n+x\n*** End Patch\n";
    let function_payload = ToolPayload::Function {
        arguments: json!({ "patch": patch }).to_string(),
    };
    let custom_payload = ToolPayload::Custom {
        input: patch.to_string(),
    };
    let handler = FunctionApplyPatchHandler::default();
    let output = ApplyPatchToolOutput::from_text("Success. Updated files.".to_string());

    for payload in [function_payload, custom_payload] {
        let (invocation, _) = invocation_for_payload(payload.clone()).await;
        assert_eq!(
            handler.pre_tool_use_payload(&invocation),
            Some(PreToolUsePayload {
                tool_name: HookToolName::apply_patch(),
                tool_input: json!({ "command": patch }),
            }),
            "pre_tool_use_payload must report the patch command for both payload kinds"
        );
        assert_eq!(
            handler.post_tool_use_payload(&invocation, &output),
            Some(PostToolUsePayload {
                tool_name: HookToolName::apply_patch(),
                tool_use_id: "call-apply-patch".to_string(),
                tool_input: json!({ "command": patch }),
                tool_response: json!("Success. Updated files."),
            }),
            "post_tool_use_payload must report the patch command for both payload kinds"
        );
    }
}

#[tokio::test]
async fn function_apply_patch_with_updated_hook_input_rewrites_custom_payload() {
    let payload = ToolPayload::Custom {
        input: "*** Begin Patch\n*** Add File: a.txt\n+x\n*** End Patch\n".to_string(),
    };
    let (invocation, _) = invocation_for_payload(payload).await;
    let handler = FunctionApplyPatchHandler::default();
    let updated_patch = "*** Begin Patch\n*** Add File: b.txt\n+y\n*** End Patch\n";

    let updated = handler
        .with_updated_hook_input(invocation, json!({ "command": updated_patch }))
        .expect("a hook-updated command must be accepted");
    let ToolPayload::Custom { input } = &updated.payload else {
        panic!("with_updated_hook_input must keep the Custom payload kind");
    };
    assert_eq!(
        input.as_str(),
        updated_patch,
        "the Custom input must be rewritten to the hook-updated patch"
    );
}

#[test]
fn function_apply_patch_handler_diff_consumer_is_some() {
    let handler = FunctionApplyPatchHandler::default();
    assert!(
        handler.create_diff_consumer().is_some(),
        "the function-form handler must expose a diff consumer, like ApplyPatchHandler"
    );
}

#[tokio::test]
async fn function_apply_patch_applies_f1_shaped_raw_add_file_patch() {
    // F1 shape (spec §1.1): raw markdown Add-File — content lines carry
    // no per-line `+` prefix and the first content line is an H1.
    let patch = "*** Begin Patch\n\
*** Add File: grok/plans/spec-freeze-r23-glm.md\n\
# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)\n\
Freeze holds until the round-23 review lands.\n\
\n\
| field | value |\n\
| --- | --- |\n\
*** End Patch\n";
    let payload = ToolPayload::Function {
        arguments: json!({ "patch": patch }).to_string(),
    };
    // The default test session is read-only with approval on request, so this
    // write would stall on an unanswered approval prompt; build the session
    // with the integration harness shape instead.
    let (session, turn, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config.permissions = Permissions::from_approval_and_profile(
                Constrained::allow_any(AskForApproval::Never),
                Constrained::allow_only(PermissionProfile::Disabled),
            )
            .expect("test permissions should be valid");
        },
    )
    .await;
    let (invocation, cwd) = invocation_from_session(payload, session, turn).await;
    let handler = FunctionApplyPatchHandler::default();

    match handler.handle(invocation).await {
        Ok(_) => {}
        Err(err) => panic!("an F1-shaped raw Add-File patch must apply: {err:?}"),
    }

    let file_path = cwd.to_path_buf().join("grok/plans/spec-freeze-r23-glm.md");
    assert_eq!(
        std::fs::read_to_string(&file_path).expect("the raw Add-File must create the file"),
        "# SPEC-FREEZE-1 ROUND 23 — REVIEW RUN 2 of 3 (apex-ayl.45)\nFreeze holds until the round-23 review lands.\n\n| field | value |\n| --- | --- |\n"
    );
    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_dir(cwd.to_path_buf().join("grok/plans"));
    let _ = std::fs::remove_dir(cwd.to_path_buf().join("grok"));
}

// Pinned to spec §3.4 (apex-xt2.11): byte-identical to the private
// PATCH_REPAIR_NOTE const in codex-apply-patch. The const stays private
// (crate API surface rule); the core crate pins the model-visible string at
// the handler seam instead.
const PATCH_REPAIR_NOTE: &str = "Note: the patch shape was repaired by the parser (missing or stray '*** Begin Patch'/'*** End Patch' boundary lines); the applied file content is exactly as provided.";

#[tokio::test]
async fn function_apply_patch_shape_a_missing_begin_surfaces_repair_note() {
    // Shape A (spec §2.1): first line is the hunk header — the
    // `*** Begin Patch` boundary is missing — content lines carry the
    // canonical `+` prefix, and there is no trailing `*** End Patch`.
    let patch = "*** Add File: notes/t12.md\n+line one\n+line two\n";
    let payload = ToolPayload::Function {
        arguments: json!({ "patch": patch }).to_string(),
    };
    let (session, turn, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config.permissions = Permissions::from_approval_and_profile(
                Constrained::allow_any(AskForApproval::Never),
                Constrained::allow_only(PermissionProfile::Disabled),
            )
            .expect("test permissions should be valid");
        },
    )
    .await;
    let (invocation, cwd) = invocation_from_session(payload, session, turn).await;
    let handler = FunctionApplyPatchHandler::default();

    let output = match handler.handle(invocation).await {
        Ok(output) => output,
        Err(err) => panic!("a shape-A missing-Begin patch must apply: {err:?}"),
    };
    let text = output.log_output();
    assert!(
        text.starts_with(&format!("{PATCH_REPAIR_NOTE}\n")),
        "tool output must start with the repair note: {text:?}"
    );

    let file_path = cwd.to_path_buf().join("notes/t12.md");
    assert_eq!(
        std::fs::read_to_string(&file_path).expect("the repaired Add-File must create the file"),
        "line one\nline two\n"
    );
    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_dir(cwd.to_path_buf().join("notes"));
}
