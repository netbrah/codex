//! Translators between codex-rs internal types and the Anthropic `/messages`
//! wire format.

use crate::client_common::tools::ToolSpec;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::ResponseItem;
use serde_json::Value;
use serde_json::json;

/// Translates the codex-rs conversation history (`&[ResponseItem]`) into
/// Anthropic's `messages` array, extracting the system prompt from the first
/// system-role message if present.
pub(crate) fn conversation_to_anthropic_messages(input: &[ResponseItem]) -> Vec<Value> {
    let mut messages: Vec<Value> = Vec::new();

    for item in input {
        match item {
            ResponseItem::Message { role, content, .. } => {
                let anthropic_role = match role.as_str() {
                    "system" => continue,
                    "user" => "user",
                    "assistant" => "assistant",
                    "developer" => continue, // already injected via system parameter
                    _ => "user",
                };

                let content_blocks: Vec<Value> = content
                    .iter()
                    .map(|c| match c {
                        ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                            json!({
                                "type": "text",
                                "text": text,
                            })
                        }
                        ContentItem::InputImage { image_url } => json!({
                            "type": "image",
                            "source": {
                                "type": "url",
                                "url": image_url,
                            },
                        }),
                    })
                    .collect();

                if content_blocks.is_empty() {
                    continue;
                }

                append_to_role(&mut messages, anthropic_role, content_blocks);
            }

            ResponseItem::FunctionCall {
                call_id,
                name,
                arguments,
                ..
            } => {
                let input_val: Value = serde_json::from_str(arguments).unwrap_or_else(|e| {
                    tracing::warn!("malformed tool arguments JSON, using empty object: {e}");
                    json!({})
                });
                let block = json!({
                    "type": "tool_use",
                    "id": call_id,
                    "name": name,
                    "input": input_val,
                });
                append_to_role(&mut messages, "assistant", vec![block]);
            }

            ResponseItem::FunctionCallOutput {
                call_id, output, ..
            } => {
                let content_text = output_to_text(output);
                let block = json!({
                    "type": "tool_result",
                    "tool_use_id": call_id,
                    "content": content_text,
                });
                append_to_role(&mut messages, "user", vec![block]);
            }

            ResponseItem::CustomToolCall {
                call_id,
                name,
                input: input_str,
                ..
            } => {
                let input_val: Value = serde_json::from_str(input_str).unwrap_or_else(|e| {
                    tracing::warn!("malformed tool arguments JSON, using empty object: {e}");
                    json!({})
                });
                let block = json!({
                    "type": "tool_use",
                    "id": call_id,
                    "name": name,
                    "input": input_val,
                });
                append_to_role(&mut messages, "assistant", vec![block]);
            }

            ResponseItem::CustomToolCallOutput {
                call_id, output, ..
            } => {
                let content_text = output_to_text(output);
                let block = json!({
                    "type": "tool_result",
                    "tool_use_id": call_id,
                    "content": content_text,
                });
                append_to_role(&mut messages, "user", vec![block]);
            }

            ResponseItem::LocalShellCall {
                call_id, action, ..
            } => {
                let codex_protocol::models::LocalShellAction::Exec(exec) = action;
                let args = json!({ "command": exec.command.join(" ") });
                let block = json!({
                    "type": "tool_use",
                    "id": call_id.clone().unwrap_or_default(),
                    "name": "shell",
                    "input": args,
                });
                append_to_role(&mut messages, "assistant", vec![block]);
            }

            ResponseItem::Reasoning {
                summary,
                content,
                encrypted_content,
                raw_wire_block,
                ..
            } => {
                use codex_protocol::models::ReasoningItemContent;
                use codex_protocol::models::ReasoningItemReasoningSummary;

                // Prefer the raw wire block for byte-identical replay. This
                // avoids any mutation from decomposition/reconstruction and
                // satisfies Anthropic's cryptographic verification of thinking
                // blocks in the latest assistant message.
                if let Some(raw_block) = raw_wire_block {
                    append_to_role(&mut messages, "assistant", vec![raw_block.clone()]);
                    continue;
                }

                // Fallback: reconstruct from decomposed fields. Used for
                // Reasoning items from the Responses wire (OpenAI) or older
                // session files that predate raw_wire_block.
                if let Some(ec) = encrypted_content {
                    if let Some(data) = ec.strip_prefix("\0REDACTED\0") {
                        let block = json!({
                            "type": "redacted_thinking",
                            "data": data,
                        });
                        append_to_role(&mut messages, "assistant", vec![block]);
                        continue;
                    }
                }

                let signature = encrypted_content.as_deref().unwrap_or("");

                if let Some(content_items) = content {
                    for item in content_items {
                        let text = match item {
                            ReasoningItemContent::ReasoningText { text }
                            | ReasoningItemContent::Text { text } => text.as_str(),
                        };
                        if !text.is_empty() {
                            let block = json!({
                                "type": "thinking",
                                "thinking": text,
                                "signature": signature,
                            });
                            append_to_role(&mut messages, "assistant", vec![block]);
                        }
                    }
                } else {
                    let text = summary
                        .iter()
                        .map(|s| match s {
                            ReasoningItemReasoningSummary::SummaryText { text } => text.as_str(),
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !text.is_empty() {
                        let block = json!({
                            "type": "thinking",
                            "thinking": text,
                            "signature": signature,
                        });
                        append_to_role(&mut messages, "assistant", vec![block]);
                    }
                }
            }

            _ => {
                tracing::trace!("messages_wire: skipping unhandled ResponseItem variant");
            }
        }
    }

    strip_thinking_from_non_latest_assistant_messages(&mut messages);
    messages
}


/// Extracts text content from `developer`-role messages.
///
/// Developer-role messages carry AGENTS.md contents, permission directives,
/// personality config, and other project instructions. In the Anthropic
/// `/messages` wire format these must be injected into the `system` parameter
/// rather than appearing in the `messages[]` array.
///
/// Returns a `Vec<String>` of developer text blocks in the order they appear
/// in the conversation. The caller should append them to the `system` array
/// after `base_instructions`.
pub(crate) fn extract_developer_blocks(input: &[ResponseItem]) -> Vec<String> {
    let mut blocks = Vec::new();
    for item in input {
        if let ResponseItem::Message { role, content, .. } = item {
            if role == "developer" {
                for c in content {
                    match c {
                        ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                            if !text.is_empty() {
                                blocks.push(text.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    blocks
}

/// Strips `thinking` and `redacted_thinking` content blocks from all assistant
/// messages except the last one in the array.
///
/// The Anthropic API requires that thinking blocks in the **latest** assistant
/// message be byte-identical to the original response. Earlier assistant
/// messages can have thinking blocks omitted entirely. Stripping them prevents
/// issues from compaction merging, proxy translation, or serialization
/// round-trips that could subtly modify the blocks.
fn strip_thinking_from_non_latest_assistant_messages(messages: &mut Vec<Value>) {
    let last_assistant_idx = messages.iter().rposition(|m| {
        m.get("role").and_then(|r| r.as_str()) == Some("assistant")
    });

    let Some(last_idx) = last_assistant_idx else {
        return;
    };

    for (i, msg) in messages.iter_mut().enumerate() {
        if i >= last_idx {
            break;
        }
        if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
            continue;
        }
        if let Some(content) = msg.get_mut("content").and_then(|c| c.as_array_mut()) {
            content.retain(|block| {
                let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                block_type != "thinking" && block_type != "redacted_thinking"
            });
        }
    }

    // Remove assistant messages that became empty after stripping
    messages.retain(|msg| {
        if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
            return true;
        }
        msg.get("content")
            .and_then(|c| c.as_array())
            .map_or(true, |arr| !arr.is_empty())
    });
}

/// Translates OpenAI Responses API tool specs to Anthropic `/messages` format.
///
/// Only `Function` tools are translated; server-side tool types (local_shell,
/// web_search, etc.) are skipped since Anthropic doesn't have equivalents.
pub(crate) fn tools_to_anthropic_format(tools: &[ToolSpec]) -> Vec<Value> {
    let mut result: Vec<Value> = tools
        .iter()
        .filter_map(|tool| match tool {
            ToolSpec::Function(f) => Some(json!({
                "name": f.name,
                "description": f.description,
                "input_schema": f.parameters,
            })),
            _ => None,
        })
        .collect();

    if let Some(last) = result.last_mut() {
        last.as_object_mut()
            .map(|obj| obj.insert("cache_control".to_owned(), json!({"type": "ephemeral"})));
    }
    result
}

fn output_to_text(output: &codex_protocol::models::FunctionCallOutputPayload) -> String {
    match &output.body {
        FunctionCallOutputBody::Text(text) => text.clone(),
        FunctionCallOutputBody::ContentItems(items) => items
            .iter()
            .filter_map(|item| {
                if let codex_protocol::models::FunctionCallOutputContentItem::InputText { text } =
                    item
                {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// Appends content blocks to the last message if it has the matching role,
/// or creates a new message. This ensures Anthropic's alternating
/// user/assistant constraint is met by merging consecutive same-role messages.
fn append_to_role(messages: &mut Vec<Value>, role: &str, blocks: Vec<Value>) {
    if let Some(last) = messages.last_mut()
        && last.get("role").and_then(|r| r.as_str()) == Some(role)
        && let Some(content) = last.get_mut("content").and_then(|c| c.as_array_mut())
    {
        content.extend(blocks);
        return;
    }
    messages.push(json!({
        "role": role,
        "content": blocks,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::FunctionCallOutputPayload;

    #[test]
    fn test_simple_user_assistant_messages() {
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Hello".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Hi there".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"][0]["type"], "text");
        assert_eq!(messages[0]["content"][0]["text"], "Hello");
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["content"][0]["text"], "Hi there");
    }

    #[test]
    fn test_tool_use_roundtrip() {
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "List files".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"ls"}"#.to_string(),
                call_id: "toolu_01".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "toolu_01".to_string(),
                output: FunctionCallOutputPayload::from_text("file1.txt\nfile2.txt".to_string()),
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["content"][0]["type"], "tool_use");
        assert_eq!(messages[1]["content"][0]["id"], "toolu_01");
        assert_eq!(messages[2]["role"], "user");
        assert_eq!(messages[2]["content"][0]["type"], "tool_result");
        assert_eq!(messages[2]["content"][0]["tool_use_id"], "toolu_01");
    }

    #[test]
    fn test_consecutive_same_role_merged() {
        let input = vec![
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"ls"}"#.to_string(),
                call_id: "toolu_01".to_string(),
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "read_file".to_string(),
                namespace: None,
                arguments: r#"{"path":"foo.txt"}"#.to_string(),
                call_id: "toolu_02".to_string(),
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "assistant");
        assert_eq!(messages[0]["content"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_tools_translation() {
        use crate::client_common::tools::ResponsesApiTool;
        use crate::tools::spec::JsonSchema;

        let tools = vec![ToolSpec::Function(ResponsesApiTool {
            name: "shell".to_string(),
            description: "Run a shell command".to_string(),
            strict: true,
            defer_loading: None,
            parameters: JsonSchema::Object {
                properties: Default::default(),
                required: None,
                additional_properties: None,
            },
            output_schema: None,
        })];

        let anthropic_tools = tools_to_anthropic_format(&tools);
        assert_eq!(anthropic_tools.len(), 1);
        assert_eq!(anthropic_tools[0]["name"], "shell");
        assert!(anthropic_tools[0].get("input_schema").is_some());
    }

    #[test]
    fn test_system_messages_skipped() {
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "system".to_string(),
                content: vec![ContentItem::InputText {
                    text: "You are helpful".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Hello".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1, "system messages should be skipped");
        assert_eq!(messages[0]["role"], "user");
    }

    #[test]
    fn test_empty_content_messages_skipped() {
        let input = vec![ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![],
            end_turn: None,
            phase: None,
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert!(
            messages.is_empty(),
            "empty content messages should be skipped"
        );
    }

    #[test]
    fn test_raw_wire_block_used_verbatim() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        // When raw_wire_block is present, it should be used directly
        // instead of reconstructing from summary/encrypted_content.
        let raw_block = json!({
            "type": "thinking",
            "thinking": "Exact wire text with special chars: \n\t\"quotes\"",
            "signature": "ErUmExactSignature=="
        });
        let input = vec![ResponseItem::Reasoning {
            id: String::new(),
            summary: vec![ReasoningItemReasoningSummary::SummaryText {
                text: "DIFFERENT summary text".to_string(),
            }],
            content: None,
            encrypted_content: Some("DIFFERENT_signature".to_string()),
            raw_wire_block: Some(raw_block.clone()),
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        let block = &messages[0]["content"][0];
        // Must use the raw block, NOT the decomposed fields
        assert_eq!(
            block["thinking"], "Exact wire text with special chars: \n\t\"quotes\"",
            "must use raw_wire_block thinking, not summary"
        );
        assert_eq!(
            block["signature"], "ErUmExactSignature==",
            "must use raw_wire_block signature, not encrypted_content"
        );
    }

    #[test]
    fn test_raw_wire_block_redacted_used_verbatim() {
        // When raw_wire_block is present for redacted thinking, use it directly
        let raw_block = json!({
            "type": "redacted_thinking",
            "data": "ExactOpaqueData123=="
        });
        let input = vec![ResponseItem::Reasoning {
            id: String::new(),
            summary: Vec::new(),
            content: None,
            encrypted_content: Some("\0REDACTED\0DIFFERENT_data".to_string()),
            raw_wire_block: Some(raw_block.clone()),
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        let block = &messages[0]["content"][0];
        assert_eq!(block["type"], "redacted_thinking");
        assert_eq!(
            block["data"], "ExactOpaqueData123==",
            "must use raw_wire_block data, not encrypted_content"
        );
    }

    #[test]
    fn test_fallback_when_no_raw_wire_block() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        // When raw_wire_block is None (old sessions, Responses wire),
        // must fall back to reconstruction from decomposed fields
        let input = vec![ResponseItem::Reasoning {
            id: String::new(),
            summary: vec![ReasoningItemReasoningSummary::SummaryText {
                text: "Reconstructed text".to_string(),
            }],
            content: None,
            encrypted_content: Some("reconstructed_sig".to_string()),
            raw_wire_block: None,
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        let block = &messages[0]["content"][0];
        assert_eq!(block["type"], "thinking");
        assert_eq!(block["thinking"], "Reconstructed text");
        assert_eq!(block["signature"], "reconstructed_sig");
    }

    #[test]
    fn test_reasoning_with_signature_preserved() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        let input = vec![ResponseItem::Reasoning {
            id: String::new(),
            summary: vec![ReasoningItemReasoningSummary::SummaryText {
                text: "Deep thoughts".to_string(),
            }],
            content: None,
            encrypted_content: Some("sig_real_signature_abc".to_string()),
            raw_wire_block: None,
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "assistant");
        assert_eq!(messages[0]["content"][0]["type"], "thinking");
        assert_eq!(messages[0]["content"][0]["thinking"], "Deep thoughts");
        assert_eq!(
            messages[0]["content"][0]["signature"], "sig_real_signature_abc",
            "signature must be preserved from encrypted_content"
        );
    }

    #[test]
    fn test_redacted_thinking_roundtrip() {
        let input = vec![ResponseItem::Reasoning {
            id: String::new(),
            summary: Vec::new(),
            content: None,
            encrypted_content: Some("\0REDACTED\0opaque_data_xyz".to_string()),
            raw_wire_block: None,
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "assistant");
        assert_eq!(
            messages[0]["content"][0]["type"], "redacted_thinking",
            "should emit redacted_thinking block"
        );
        assert_eq!(messages[0]["content"][0]["data"], "opaque_data_xyz");
    }

    #[test]
    fn test_multi_turn_tool_loop() {
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Deploy the app".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"npm build"}"#.to_string(),
                call_id: "toolu_01".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "toolu_01".to_string(),
                output: FunctionCallOutputPayload::from_text("Build successful".to_string()),
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"npm deploy"}"#.to_string(),
                call_id: "toolu_02".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "toolu_02".to_string(),
                output: FunctionCallOutputPayload::from_text("Deployed!".to_string()),
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Done deploying".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);

        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["content"][0]["type"], "tool_use");
        assert_eq!(messages[2]["role"], "user");
        assert_eq!(messages[2]["content"][0]["type"], "tool_result");
        assert_eq!(messages[3]["role"], "assistant");
        assert_eq!(messages[3]["content"][0]["type"], "tool_use");
        assert_eq!(messages[4]["role"], "user");
        assert_eq!(messages[4]["content"][0]["type"], "tool_result");
        assert_eq!(messages[5]["role"], "assistant");
        assert_eq!(messages[5]["content"][0]["type"], "text");

        for msg in &messages {
            let role = msg["role"].as_str().unwrap();
            assert!(
                role == "user" || role == "assistant",
                "only user/assistant roles allowed"
            );
        }
    }

    #[test]
    fn test_non_function_tools_filtered() {
        let tools = vec![
            ToolSpec::LocalShell {},
            ToolSpec::WebSearch {
                external_web_access: None,
                filters: None,
                user_location: None,
                search_context_size: None,
                search_content_types: None,
            },
        ];

        let anthropic_tools = tools_to_anthropic_format(&tools);
        assert!(
            anthropic_tools.is_empty(),
            "non-function tools should be filtered out"
        );
    }

    #[test]
    fn test_local_shell_call_translated() {
        use codex_protocol::models::LocalShellAction;
        use codex_protocol::models::LocalShellExecAction;
        use codex_protocol::models::LocalShellStatus;

        let input = vec![ResponseItem::LocalShellCall {
            id: None,
            call_id: Some("shell_01".to_string()),
            status: LocalShellStatus::InProgress,
            action: LocalShellAction::Exec(LocalShellExecAction {
                command: vec!["ls".to_string(), "-la".to_string()],
                timeout_ms: None,
                working_directory: None,
                env: None,
                user: None,
            }),
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "assistant");
        assert_eq!(messages[0]["content"][0]["type"], "tool_use");
        assert_eq!(messages[0]["content"][0]["id"], "shell_01");
        assert_eq!(messages[0]["content"][0]["name"], "shell");
        assert_eq!(messages[0]["content"][0]["input"]["command"], "ls -la");
    }

    #[test]
    fn test_custom_tool_call_roundtrip() {
        let input = vec![
            ResponseItem::CustomToolCall {
                id: None,
                status: None,
                call_id: "custom_01".to_string(),
                name: "apply_patch".to_string(),
                input: r#"{"patch":"diff content"}"#.to_string(),
            },
            ResponseItem::CustomToolCallOutput {
                call_id: "custom_01".to_string(),
                name: None,
                output: FunctionCallOutputPayload::from_text("Patch applied".to_string()),
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["content"][0]["type"], "tool_use");
        assert_eq!(messages[0]["content"][0]["id"], "custom_01");
        assert_eq!(messages[0]["content"][0]["name"], "apply_patch");
        assert_eq!(messages[1]["content"][0]["type"], "tool_result");
        assert_eq!(messages[1]["content"][0]["tool_use_id"], "custom_01");
    }

    #[test]
    fn test_thinking_precedes_tool_use_in_same_message() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Deploy".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "I should run the build".to_string(),
                }],
                content: None,
                encrypted_content: Some("sig_xyz".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"npm build"}"#.to_string(),
                call_id: "toolu_01".to_string(),
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "user");

        let assistant_content = messages[1]["content"].as_array().unwrap();
        assert_eq!(assistant_content.len(), 2);
        assert_eq!(
            assistant_content[0]["type"], "thinking",
            "thinking must come before tool_use"
        );
        assert_eq!(
            assistant_content[1]["type"], "tool_use",
            "tool_use must come after thinking"
        );
    }

    #[test]
    fn test_tool_cache_control_on_last_tool() {
        use crate::client_common::tools::ResponsesApiTool;
        use crate::tools::spec::JsonSchema;

        let tools = vec![
            ToolSpec::Function(ResponsesApiTool {
                name: "shell".to_string(),
                description: "Run shell".to_string(),
                strict: true,
                defer_loading: None,
                parameters: JsonSchema::Object {
                    properties: Default::default(),
                    required: None,
                    additional_properties: None,
                },
                output_schema: None,
            }),
            ToolSpec::Function(ResponsesApiTool {
                name: "read_file".to_string(),
                description: "Read a file".to_string(),
                strict: true,
                defer_loading: None,
                parameters: JsonSchema::Object {
                    properties: Default::default(),
                    required: None,
                    additional_properties: None,
                },
                output_schema: None,
            }),
        ];

        let anthropic_tools = tools_to_anthropic_format(&tools);
        assert_eq!(anthropic_tools.len(), 2);
        assert!(
            anthropic_tools[0].get("cache_control").is_none(),
            "first tool should not have cache_control"
        );
        assert_eq!(
            anthropic_tools[1]["cache_control"]["type"], "ephemeral",
            "last tool must have cache_control for prompt caching"
        );
    }

    #[test]
    fn test_empty_tools_no_panic() {
        let tools: Vec<ToolSpec> = vec![];
        let anthropic_tools = tools_to_anthropic_format(&tools);
        assert!(anthropic_tools.is_empty());
    }

    #[test]
    fn test_developer_messages_skipped() {
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "developer".to_string(),
                content: vec![ContentItem::InputText {
                    text: "You have full permissions".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Hello".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1, "developer messages should be skipped");
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"][0]["text"], "Hello");
    }

    #[test]
    fn test_malformed_arguments_uses_empty_object() {
        let input = vec![ResponseItem::FunctionCall {
            id: None,
            name: "shell".to_string(),
            namespace: None,
            arguments: "not valid json{{{".to_string(),
            call_id: "toolu_bad".to_string(),
        }];

        let messages = conversation_to_anthropic_messages(&input);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["content"][0]["type"], "tool_use");
        assert_eq!(
            messages[0]["content"][0]["input"],
            json!({}),
            "malformed arguments should fall back to empty object"
        );
    }

    #[test]
    fn test_thinking_stripped_from_earlier_assistant_messages() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        // Turn 1: user → thinking + tool_use → tool_result
        // Turn 2: user → thinking + text (latest)
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "First question".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "Old thinking".to_string(),
                }],
                content: None,
                encrypted_content: Some("old_sig".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"ls"}"#.to_string(),
                call_id: "toolu_01".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "toolu_01".to_string(),
                output: FunctionCallOutputPayload::from_text("files".to_string()),
            },
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Second question".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "Latest thinking".to_string(),
                }],
                content: None,
                encrypted_content: Some("latest_sig".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Final answer".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);

        // First assistant message should have thinking stripped, only tool_use remains
        let first_assistant = messages
            .iter()
            .find(|m| {
                m["role"] == "assistant"
                    && m["content"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|b| b["type"] == "tool_use")
            })
            .expect("should have an assistant message with tool_use");
        for block in first_assistant["content"].as_array().unwrap() {
            assert_ne!(
                block["type"], "thinking",
                "thinking should be stripped from earlier assistant messages"
            );
        }

        // Last assistant message should preserve thinking
        let last_assistant = messages.last().unwrap();
        assert_eq!(last_assistant["role"], "assistant");
        let content = last_assistant["content"].as_array().unwrap();
        assert!(
            content.iter().any(|b| b["type"] == "thinking"),
            "latest assistant message must keep thinking blocks"
        );
        assert_eq!(
            content
                .iter()
                .find(|b| b["type"] == "thinking")
                .unwrap()["signature"],
            "latest_sig"
        );
    }

    #[test]
    fn test_redacted_thinking_stripped_from_earlier_messages() {
        // Turn 1: user → redacted_thinking + text
        // Turn 2: user → text (latest)
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Q1".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: Vec::new(),
                content: None,
                encrypted_content: Some("\0REDACTED\0old_opaque".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "A1".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Q2".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "A2".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);

        // First assistant message should NOT have redacted_thinking
        let first_assistant = &messages[1];
        assert_eq!(first_assistant["role"], "assistant");
        for block in first_assistant["content"].as_array().unwrap() {
            assert_ne!(
                block["type"], "redacted_thinking",
                "redacted_thinking should be stripped from earlier assistant messages"
            );
        }
    }

    #[test]
    fn test_empty_assistant_messages_removed_after_stripping() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        // An assistant message that contains ONLY a thinking block (no text/tool_use)
        // should be removed entirely after stripping (if it's not the last)
        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Q1".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "Only thinking, no text".to_string(),
                }],
                content: None,
                encrypted_content: Some("sig_only_think".to_string()),
                raw_wire_block: None,
            },
            // No text message follows — this creates an assistant msg with only thinking
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Q2".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "A2".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);

        // The thinking-only assistant message should be removed
        // We should have: user(Q1) → user(Q2) → assistant(A2)
        // But Anthropic requires alternation, so the two user messages may be merged
        // The key assertion: no empty assistant messages
        for msg in &messages {
            if msg["role"] == "assistant" {
                let content = msg["content"].as_array().unwrap();
                assert!(
                    !content.is_empty(),
                    "assistant messages with empty content should be removed"
                );
            }
        }
    }

    #[test]
    fn test_single_assistant_message_thinking_preserved() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        let input = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Hello".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "The only thinking block".to_string(),
                }],
                content: None,
                encrypted_content: Some("sig_only".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Answer".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);
        let last = messages.last().unwrap();
        assert_eq!(last["role"], "assistant");
        let content = last["content"].as_array().unwrap();
        assert!(
            content.iter().any(|b| b["type"] == "thinking"),
            "single assistant message must keep its thinking block"
        );
        assert_eq!(
            content
                .iter()
                .find(|b| b["type"] == "thinking")
                .unwrap()["thinking"],
            "The only thinking block"
        );
    }

    #[test]
    fn test_compacted_history_thinking_stripped() {
        use codex_protocol::models::ReasoningItemReasoningSummary;

        // Simulates post-compaction preserved items:
        // summary user msg → ack assistant → preserved reasoning (old turn)
        // → tool_use → tool_result → new user → new reasoning + text (latest turn)
        let input = vec![
            // Compaction summary
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "Summary of prior conversation...".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Understood, continuing.".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            // Preserved old turn with reasoning
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "Old preserved thinking".to_string(),
                }],
                content: None,
                encrypted_content: Some("old_preserved_sig".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: r#"{"command":"cat foo"}"#.to_string(),
                call_id: "toolu_p1".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "toolu_p1".to_string(),
                output: FunctionCallOutputPayload::from_text("foo content".to_string()),
            },
            // New turn
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "New question".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
            ResponseItem::Reasoning {
                id: String::new(),
                summary: vec![ReasoningItemReasoningSummary::SummaryText {
                    text: "Fresh thinking".to_string(),
                }],
                content: None,
                encrypted_content: Some("fresh_sig".to_string()),
                raw_wire_block: None,
            },
            ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "Fresh answer".to_string(),
                }],
                end_turn: None,
                phase: None,
            },
        ];

        let messages = conversation_to_anthropic_messages(&input);

        // Find all assistant messages
        let assistant_msgs: Vec<_> = messages
            .iter()
            .filter(|m| m["role"] == "assistant")
            .collect();

        // All non-last assistant messages should have no thinking blocks
        for msg in &assistant_msgs[..assistant_msgs.len() - 1] {
            for block in msg["content"].as_array().unwrap() {
                assert_ne!(
                    block["type"], "thinking",
                    "preserved old thinking must be stripped from non-latest assistant"
                );
            }
        }

        // Last assistant message should keep fresh thinking
        let last = assistant_msgs.last().unwrap();
        let content = last["content"].as_array().unwrap();
        assert!(
            content.iter().any(|b| b["type"] == "thinking"),
            "latest assistant must keep fresh thinking"
        );
        assert_eq!(
            content
                .iter()
                .find(|b| b["type"] == "thinking")
                .unwrap()["signature"],
            "fresh_sig"
        );
    }
}
