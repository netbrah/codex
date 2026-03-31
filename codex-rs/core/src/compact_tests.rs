use super::*;
use pretty_assertions::assert_eq;

async fn process_compacted_history_with_test_session(
    compacted_history: Vec<ResponseItem>,
    previous_turn_settings: Option<&PreviousTurnSettings>,
) -> (Vec<ResponseItem>, Vec<ResponseItem>) {
    let (session, turn_context) = crate::codex::make_session_and_context().await;
    session
        .set_previous_turn_settings(previous_turn_settings.cloned())
        .await;
    let initial_context = session.build_initial_context(&turn_context).await;
    let refreshed = crate::compact_remote::process_compacted_history(
        &session,
        &turn_context,
        compacted_history,
        InitialContextInjection::BeforeLastUserMessage,
    )
    .await;
    (refreshed, initial_context)
}

#[test]
fn content_items_to_text_joins_non_empty_segments() {
    let items = vec![
        ContentItem::InputText {
            text: "hello".to_string(),
        },
        ContentItem::OutputText {
            text: String::new(),
        },
        ContentItem::OutputText {
            text: "world".to_string(),
        },
    ];

    let joined = content_items_to_text(&items);

    assert_eq!(Some("hello\nworld".to_string()), joined);
}

#[test]
fn content_items_to_text_ignores_image_only_content() {
    let items = vec![ContentItem::InputImage {
        image_url: "file://image.png".to_string(),
    }];

    let joined = content_items_to_text(&items);

    assert_eq!(None, joined);
}

#[test]
fn collect_user_messages_extracts_user_text_only() {
    let items = vec![
        ResponseItem::Message {
            id: Some("assistant".to_string()),
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "ignored".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: Some("user".to_string()),
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "first".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Other,
    ];

    let collected = collect_user_messages(&items);

    assert_eq!(vec!["first".to_string()], collected);
}

#[test]
fn collect_user_messages_filters_session_prefix_entries() {
    let items = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: r#"# AGENTS.md instructions for project

<INSTRUCTIONS>
do things
</INSTRUCTIONS>"#
                    .to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "<ENVIRONMENT_CONTEXT>cwd=/tmp</ENVIRONMENT_CONTEXT>".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "real user message".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
    ];

    let collected = collect_user_messages(&items);

    assert_eq!(vec!["real user message".to_string()], collected);
}

#[test]
fn build_token_limited_compacted_history_truncates_overlong_user_messages() {
    // Use a small truncation limit so the test remains fast while still validating
    // that oversized user content is truncated.
    let max_tokens = 16;
    let big = "word ".repeat(200);
    let history = super::build_compacted_history_with_limit(
        Vec::new(),
        std::slice::from_ref(&big),
        "SUMMARY",
        max_tokens,
    );
    assert_eq!(history.len(), 2);

    let truncated_message = &history[0];
    let summary_message = &history[1];

    let truncated_text = match truncated_message {
        ResponseItem::Message { role, content, .. } if role == "user" => {
            content_items_to_text(content).unwrap_or_default()
        }
        other => panic!("unexpected item in history: {other:?}"),
    };

    assert!(
        truncated_text.contains("tokens truncated"),
        "expected truncation marker in truncated user message"
    );
    assert!(
        !truncated_text.contains(&big),
        "truncated user message should not include the full oversized user text"
    );

    let summary_text = match summary_message {
        ResponseItem::Message { role, content, .. } if role == "user" => {
            content_items_to_text(content).unwrap_or_default()
        }
        other => panic!("unexpected item in history: {other:?}"),
    };
    assert_eq!(summary_text, "SUMMARY");
}

#[test]
fn build_token_limited_compacted_history_appends_summary_message() {
    let initial_context: Vec<ResponseItem> = Vec::new();
    let user_messages = vec!["first user message".to_string()];
    let summary_text = "summary text";

    let history = build_compacted_history(initial_context, &user_messages, summary_text);
    assert!(
        !history.is_empty(),
        "expected compacted history to include summary"
    );

    let last = history.last().expect("history should have a summary entry");
    let summary = match last {
        ResponseItem::Message { role, content, .. } if role == "user" => {
            content_items_to_text(content).unwrap_or_default()
        }
        other => panic!("expected summary message, found {other:?}"),
    };
    assert_eq!(summary, summary_text);
}

#[tokio::test]
async fn process_compacted_history_replaces_developer_messages() {
    let compacted_history = vec![
        ResponseItem::Message {
            id: None,
            role: "developer".to_string(),
            content: vec![ContentItem::InputText {
                text: "stale permissions".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "summary".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "developer".to_string(),
            content: vec![ContentItem::InputText {
                text: "stale personality".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
    ];
    let (refreshed, mut expected) = process_compacted_history_with_test_session(
        compacted_history,
        /*previous_turn_settings*/ None,
    )
    .await;
    expected.push(ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "summary".to_string(),
        }],
        end_turn: None,
        phase: None,
    });
    assert_eq!(refreshed, expected);
}

#[tokio::test]
async fn process_compacted_history_reinjects_full_initial_context() {
    let compacted_history = vec![ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "summary".to_string(),
        }],
        end_turn: None,
        phase: None,
    }];
    let (refreshed, mut expected) = process_compacted_history_with_test_session(
        compacted_history,
        /*previous_turn_settings*/ None,
    )
    .await;
    expected.push(ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "summary".to_string(),
        }],
        end_turn: None,
        phase: None,
    });
    assert_eq!(refreshed, expected);
}

#[tokio::test]
async fn process_compacted_history_drops_non_user_content_messages() {
    let compacted_history = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: r#"# AGENTS.md instructions for /repo

<INSTRUCTIONS>
keep me updated
</INSTRUCTIONS>"#
                    .to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: r#"<environment_context>
  <cwd>/repo</cwd>
  <shell>zsh</shell>
</environment_context>"#
                    .to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: r#"<turn_aborted>
  <turn_id>turn-1</turn_id>
  <reason>interrupted</reason>
</turn_aborted>"#
                    .to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "summary".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "developer".to_string(),
            content: vec![ContentItem::InputText {
                text: "stale developer instructions".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
    ];
    let (refreshed, mut expected) = process_compacted_history_with_test_session(
        compacted_history,
        /*previous_turn_settings*/ None,
    )
    .await;
    expected.push(ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "summary".to_string(),
        }],
        end_turn: None,
        phase: None,
    });
    assert_eq!(refreshed, expected);
}

#[tokio::test]
async fn process_compacted_history_inserts_context_before_last_real_user_message_only() {
    let compacted_history = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "older user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: format!("{SUMMARY_PREFIX}\nsummary text"),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "latest user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
    ];

    let (refreshed, initial_context) = process_compacted_history_with_test_session(
        compacted_history,
        /*previous_turn_settings*/ None,
    )
    .await;
    let mut expected = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "older user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: format!("{SUMMARY_PREFIX}\nsummary text"),
            }],
            end_turn: None,
            phase: None,
        },
    ];
    expected.extend(initial_context);
    expected.push(ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "latest user".to_string(),
        }],
        end_turn: None,
        phase: None,
    });
    assert_eq!(refreshed, expected);
}

#[tokio::test]
async fn process_compacted_history_reinjects_model_switch_message() {
    let compacted_history = vec![ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "summary".to_string(),
        }],
        end_turn: None,
        phase: None,
    }];
    let previous_turn_settings = PreviousTurnSettings {
        model: "previous-regular-model".to_string(),
        realtime_active: None,
    };

    let (refreshed, initial_context) = process_compacted_history_with_test_session(
        compacted_history,
        Some(&previous_turn_settings),
    )
    .await;

    let ResponseItem::Message { role, content, .. } = &initial_context[0] else {
        panic!("expected developer message");
    };
    assert_eq!(role, "developer");
    let [ContentItem::InputText { text }, ..] = content.as_slice() else {
        panic!("expected developer text");
    };
    assert!(text.contains("<model_switch>"));

    let mut expected = initial_context;
    expected.push(ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: "summary".to_string(),
        }],
        end_turn: None,
        phase: None,
    });
    assert_eq!(refreshed, expected);
}

#[test]
fn insert_initial_context_before_last_real_user_or_summary_keeps_summary_last() {
    let compacted_history = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "older user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "latest user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: format!("{SUMMARY_PREFIX}\nsummary text"),
            }],
            end_turn: None,
            phase: None,
        },
    ];
    let initial_context = vec![ResponseItem::Message {
        id: None,
        role: "developer".to_string(),
        content: vec![ContentItem::InputText {
            text: "fresh permissions".to_string(),
        }],
        end_turn: None,
        phase: None,
    }];

    let refreshed =
        insert_initial_context_before_last_real_user_or_summary(compacted_history, initial_context);
    let expected = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "older user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "developer".to_string(),
            content: vec![ContentItem::InputText {
                text: "fresh permissions".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "latest user".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: format!("{SUMMARY_PREFIX}\nsummary text"),
            }],
            end_turn: None,
            phase: None,
        },
    ];
    assert_eq!(refreshed, expected);
}

#[test]
fn insert_initial_context_before_last_real_user_or_summary_keeps_compaction_last() {
    let compacted_history = vec![ResponseItem::Compaction {
        encrypted_content: "encrypted".to_string(),
    }];
    let initial_context = vec![ResponseItem::Message {
        id: None,
        role: "developer".to_string(),
        content: vec![ContentItem::InputText {
            text: "fresh permissions".to_string(),
        }],
        end_turn: None,
        phase: None,
    }];

    let refreshed =
        insert_initial_context_before_last_real_user_or_summary(compacted_history, initial_context);
    let expected = vec![
        ResponseItem::Message {
            id: None,
            role: "developer".to_string(),
            content: vec![ContentItem::InputText {
                text: "fresh permissions".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Compaction {
            encrypted_content: "encrypted".to_string(),
        },
    ];
    assert_eq!(refreshed, expected);
}

#[test]
fn find_compact_split_point_empty_history() {
    let items: Vec<ResponseItem> = vec![];
    assert_eq!(find_compact_split_point(&items), 0);
}

#[test]
fn find_compact_split_point_splits_at_user_boundary() {
    let items = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "a".repeat(700),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "b".repeat(700),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "c".repeat(300),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "d".repeat(300),
            }],
            end_turn: None,
            phase: None,
        },
    ];

    let split = find_compact_split_point(&items);
    assert!(
        split >= 1 && split <= 3,
        "split should land between first and third items, got {split}"
    );
    if split < items.len() {
        assert_eq!(
            match &items[split] {
                ResponseItem::Message { role, .. } => role.as_str(),
                _ => "",
            },
            "user",
            "split must land on a user message boundary"
        );
    }
}

#[test]
fn find_compact_split_point_preserves_roughly_30_percent() {
    let items: Vec<ResponseItem> = (0..10)
        .map(|i| ResponseItem::Message {
            id: None,
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: vec![ContentItem::InputText {
                text: format!("message {i} content padding"),
            }],
            end_turn: None,
            phase: None,
        })
        .collect();

    let split = find_compact_split_point(&items);
    let preserved_count = items.len() - split;
    assert!(
        preserved_count >= 2 && preserved_count <= 5,
        "should preserve ~30% of items, preserved {preserved_count}/{}",
        items.len()
    );
}

#[test]
fn find_compact_split_point_skips_summary_messages() {
    let items = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "a".repeat(500),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: format!("{SUMMARY_PREFIX}\nold summary"),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "real user message".to_string(),
            }],
            end_turn: None,
            phase: None,
        },
    ];

    let split = find_compact_split_point(&items);
    if split < items.len() {
        let text = match &items[split] {
            ResponseItem::Message { content, .. } => {
                content_items_to_text(content).unwrap_or_default()
            }
            _ => String::new(),
        };
        assert!(
            !is_summary_message(&text),
            "split should not land on a summary message"
        );
    }
}

#[test]
fn find_compact_split_point_returns_len_when_no_user_boundary_in_tail() {
    let items = vec![
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "a".repeat(200),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "b".repeat(500),
            }],
            end_turn: None,
            phase: None,
        },
        ResponseItem::FunctionCall {
            id: None,
            name: "shell".to_string(),
            namespace: None,
            arguments: r#"{"cmd":"ls"}"#.to_string(),
            call_id: "t1".to_string(),
        },
    ];

    let split = find_compact_split_point(&items);
    assert_eq!(
        split,
        items.len(),
        "when no user message in the 30% tail, split returns items.len() (empty preserved portion)"
    );
}

#[test]
fn split_not_applied_below_min_items_threshold() {
    let items: Vec<ResponseItem> = (0..5)
        .map(|i| ResponseItem::Message {
            id: None,
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: vec![ContentItem::InputText {
                text: format!("msg {i}"),
            }],
            end_turn: None,
            phase: None,
        })
        .collect();

    assert!(
        items.len() < super::MIN_ITEMS_FOR_SPLIT,
        "test items should be below threshold"
    );
}

// ── T-9: Compaction path tests (Messages wire stays on /messages) ────

#[test]
fn compaction_uses_inline_for_messages_wire_provider() {
    use crate::model_provider_info::WireApi;
    let provider = crate::model_provider_info::create_oss_provider_with_base_url(
        "https://proxy.example.com/v1",
        WireApi::Messages,
    );
    let use_remote = super::should_use_remote_compact_task(&provider);
    assert!(
        !use_remote,
        "Messages wire provider should use inline compaction, not remote task"
    );
}

#[test]
fn compaction_uses_remote_for_openai_responses_provider() {
    use crate::model_provider_info::WireApi;
    let provider = crate::model_provider_info::create_oss_provider_with_base_url(
        "https://api.openai.com/v1",
        WireApi::Responses,
    );
    // OSS providers are not is_openai(), so this should be false too
    let use_remote = super::should_use_remote_compact_task(&provider);
    assert!(
        !use_remote,
        "OSS Responses provider is not OpenAI, should use inline compaction"
    );
}
