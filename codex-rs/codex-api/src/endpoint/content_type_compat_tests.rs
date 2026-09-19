use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn normalizes_input_text_to_text_in_message_content() {
    let mut value = json!({
        "model": "glm-5.2",
        "input": [
            {
                "type": "message",
                "role": "user",
                "content": [
                    {"type": "input_text", "text": "hello world"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(value["input"][0]["content"][0]["type"], "text");
    assert_eq!(value["input"][0]["content"][0]["text"], "hello world");
}

#[test]
fn normalizes_output_text_to_text_in_assistant_message() {
    let mut value = json!({
        "input": [
            {
                "type": "message",
                "role": "assistant",
                "content": [
                    {"type": "output_text", "text": "here is the result"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(value["input"][0]["content"][0]["type"], "text");
    assert_eq!(
        value["input"][0]["content"][0]["text"],
        "here is the result"
    );
}

#[test]
fn leaves_reasoning_content_untouched() {
    let mut value = json!({
        "input": [
            {
                "type": "reasoning",
                "summary": [{"type": "summary_text", "text": "thinking..."}],
                "content": [{"type": "reasoning_text", "text": "internal deliberation"}]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(value["input"][0]["content"][0]["type"], "reasoning_text");
    assert_eq!(value["input"][0]["summary"][0]["type"], "summary_text");
}

#[test]
fn normalizes_multipart_content_array() {
    let mut value = json!({
        "input": [
            {
                "type": "message",
                "role": "user",
                "content": [
                    {"type": "input_text", "text": "first"},
                    {"type": "output_text", "text": "second"},
                    {"type": "input_text", "text": "third"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    for part in value["input"][0]["content"].as_array().unwrap() {
        assert_eq!(part["type"], "text");
    }
    assert_eq!(value["input"][0]["content"][0]["text"], "first");
    assert_eq!(value["input"][0]["content"][1]["text"], "second");
    assert_eq!(value["input"][0]["content"][2]["text"], "third");
}

#[test]
fn handles_missing_input_field() {
    let mut value = json!({"model": "glm-5.2", "instructions": "test"});
    super::normalize_content_types(&mut value);
    assert_eq!(value["model"], "glm-5.2");
}

#[test]
fn preserves_non_content_fields_in_message() {
    let mut value = json!({
        "input": [
            {
                "type": "message",
                "id": "msg_123",
                "role": "user",
                "content": [{"type": "input_text", "text": "hi"}],
                "phase": "commentary"
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(value["input"][0]["type"], "message");
    assert_eq!(value["input"][0]["id"], "msg_123");
    assert_eq!(value["input"][0]["role"], "user");
    assert_eq!(value["input"][0]["phase"], "commentary");
    assert_eq!(value["input"][0]["content"][0]["type"], "text");
}

#[test]
fn normalizes_agent_message_content() {
    let mut value = json!({
        "input": [
            {
                "type": "agent_message",
                "author": "/root",
                "recipient": "worker",
                "content": [
                    {"type": "input_text", "text": "do the thing"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(value["input"][0]["content"][0]["type"], "text");
    assert_eq!(value["input"][0]["author"], "/root");
    assert_eq!(value["input"][0]["recipient"], "worker");
}

#[test]
fn translate_rewrites_agent_message_to_message() {
    let mut value = json!({
        "input": [
            {
                "type": "agent_message",
                "id": "amsg_1",
                "author": "/root",
                "recipient": "worker",
                "content": [{"type": "text", "text": "do the thing"}]
            }
        ]
    });

    super::translate_agent_messages(&mut value);

    let item = &value["input"][0];
    assert_eq!(item["type"], "message");
    assert_eq!(item["role"], "user");
    assert_eq!(item["content"][0]["text"], "do the thing");
    // OpenAI-only fields must not leak onto the wire for a `message` item.
    assert!(item.get("author").is_none());
    assert!(item.get("recipient").is_none());
    assert!(item.get("id").is_none());
}

#[test]
fn translate_runs_after_content_normalization_end_to_end() {
    // Mirrors responses.rs: normalize_content_types runs first (input_text ->
    // text), then translate_agent_messages rewrites the item type.
    let mut value = json!({
        "input": [
            {
                "type": "agent_message",
                "author": "worker",
                "recipient": "root",
                "content": [{"type": "input_text", "text": "child result"}]
            },
            {
                "type": "message",
                "role": "user",
                "content": [{"type": "input_text", "text": "prompt"}]
            }
        ]
    });

    super::normalize_content_types(&mut value);
    super::translate_agent_messages(&mut value);

    let agent = &value["input"][0];
    assert_eq!(agent["type"], "message");
    assert_eq!(agent["role"], "user");
    assert_eq!(agent["content"][0]["type"], "text");
    assert_eq!(agent["content"][0]["text"], "child result");
    assert!(agent.get("author").is_none());
    assert!(agent.get("recipient").is_none());

    // A real message item keeps its own role and is only content-normalized.
    let user = &value["input"][1];
    assert_eq!(user["type"], "message");
    assert_eq!(user["role"], "user");
    assert_eq!(user["content"][0]["type"], "text");
}

#[test]
fn translate_ignores_missing_input() {
    let mut value = json!({"model": "qwen3.8-27b"});
    super::translate_agent_messages(&mut value);
    assert_eq!(value["model"], "qwen3.8-27b");
}

#[test]
fn normalizes_drops_encrypted_content_parts() {
    let mut value = json!({
        "input": [
            {
                "type": "message",
                "role": "user",
                "content": [
                    {"type": "input_text", "text": "visible"},
                    {"type": "encrypted_content", "encrypted_content": "blo..."}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    let content = value["input"][0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "visible");
}

#[test]
fn translate_agent_message_preserves_encrypted_payload_as_text() {
    // On non-OpenAI providers the agent_message `encrypted_content` part is the
    // plain-text inter-agent message: codex never encrypts it, it's a label for
    // OpenAI's server-side handling (vLLM leaves `encrypted_function_args` empty,
    // so `communication_from_tool_message` always stores the plain text there).
    // It must be relabelled to `text` so the receiving model actually reads the
    // message — NOT dropped.
    let mut value = json!({
        "input": [
            {
                "type": "agent_message",
                "author": "worker",
                "recipient": "root",
                "content": [
                    {"type": "input_text", "text": "Message Type: MESSAGE\nTask name: root\nSender: worker\nPayload:\n"},
                    {"type": "encrypted_content", "encrypted_content": "child says 408"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);
    super::translate_agent_messages(&mut value);

    let item = &value["input"][0];
    assert_eq!(item["type"], "message");
    assert_eq!(item["role"], "user");
    // Both parts survive: header (input_text->text) and payload
    // (encrypted_content->text). Nothing is dropped.
    let content = item["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[1]["type"], "text");
    assert_eq!(content[1]["text"], "child says 408");
    assert!(item.get("author").is_none());
    assert!(item.get("recipient").is_none());
}

// Tool-output (`output` field) normalization: the Responses->ChatCompletions
// shim gap for function_call_output / custom_tool_call_output items (their
// payload rides `output`, not `content`).

#[test]
fn collapses_all_text_tool_output_filtering_blank_segments() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": [
                    {"type": "input_text", "text": "A"},
                    {"type": "input_text", "text": "   "},
                    {"type": "input_text", "text": "B"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": "A\nB"
        })
    );
}

#[test]
fn collapses_mixed_text_label_tool_output() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": [
                    {"type": "input_text", "text": "A"},
                    {"type": "output_text", "text": "B"},
                    {"type": "text", "text": "C"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": "A\nB\nC"
        })
    );
}

#[test]
fn relabels_text_part_in_mixed_tool_output_keeps_array() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": [
                    {"type": "input_text", "text": "A"},
                    {"type": "input_image", "image_url": "https://example.com/diagram.png", "detail": "high"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    // Array kept: text part relabelled, image part byte-identical.
    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": [
                {"type": "text", "text": "A"},
                {"type": "input_image", "image_url": "https://example.com/diagram.png", "detail": "high"}
            ]
        })
    );
}

#[test]
fn leaves_string_tool_output_untouched() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": "already a string"
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": "already a string"
        })
    );
}

#[test]
fn leaves_missing_and_null_tool_output_untouched() {
    let mut value = json!({
        "input": [
            {"type": "function_call_output", "call_id": "call-1"},
            {"type": "function_call_output", "call_id": "call-2", "output": null}
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value,
        json!({
            "input": [
                {"type": "function_call_output", "call_id": "call-1"},
                {"type": "function_call_output", "call_id": "call-2", "output": null}
            ]
        })
    );
}

// Pinned incident fixture (thread 01a0b07a: codegraph_explore on glm-5.2 via
// the LiteLLM Responses->ChatCompletions shim). The real part[1] text (11,933
// chars) contains the literal substring `input_text`, so these stand-ins keep
// the shape (short wall-time fragment + long source dump ending in the same
// tail) while staying substring-free; the padding choice is recorded in the
// campaign report / commit body.
const T6_PART_0: &str = "Wall time 0.0412 seconds\nChunk ID: 83e5c1";
const T6_PART_1: &str = "Here are the relevant symbols, grouped by file, with verbatim source:\n\ncodex-api/src/endpoint/content_type_compat.rs\n  fn normalize_content_types(value: &mut Value) {\n      // rewrites part types for shim-compatible providers\n  }\n\nCall path: responses endpoint -> provider request -> serialized body.\n\nThe source above is already read-equivalent; do not re-open those files. Synthesize once you've used 2.";

#[test]
fn incident_repro_codegraph_tool_output_collapses_end_to_end() {
    let mut value = json!({
        "model": "glm-5.2",
        "input": [
            {
                "type": "message",
                "role": "user",
                "content": [
                    {"type": "input_text", "text": "Explore the content-type seam and report the call path."},
                    {"type": "input_text", "text": "Keep the answer short."}
                ]
            },
            {
                "type": "function_call",
                "name": "mcp__codegraph.codegraph_explore",
                "call_id": "call-cg1",
                "arguments": "{\"query\":\"normalize_content_types content_type_compat serialized body\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call-cg1",
                "output": [
                    {"type": "input_text", "text": T6_PART_0},
                    {"type": "input_text", "text": T6_PART_1}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    // Primary: the 2-part codegraph-shaped output collapses to one string.
    assert_eq!(
        value["input"][2]["output"],
        format!("{T6_PART_0}\n{T6_PART_1}")
    );
    // The message content parts relabel on the content path; the function_call
    // item is untouched.
    assert_eq!(value["input"][0]["content"][0]["type"], "text");
    assert_eq!(value["input"][0]["content"][1]["type"], "text");
    assert_eq!(
        value["input"][1]["name"],
        "mcp__codegraph.codegraph_explore"
    );
    assert_eq!(
        value["input"][1]["arguments"],
        "{\"query\":\"normalize_content_types content_type_compat serialized body\"}"
    );

    // Secondary: the whole serialized body is free of the rejected label.
    let serialized = serde_json::to_string(&value).expect("body should serialize");
    assert!(
        !serialized.contains("input_text"),
        "body still contains input_text: {serialized}"
    );

    // Tertiary: both part texts survive in the collapsed string (the
    // model-visible payload), in order.
    let collapsed = value["input"][2]["output"]
        .as_str()
        .expect("output should be a string");
    let first = collapsed.find(T6_PART_0).expect("part 0 text present");
    let second = collapsed.find(T6_PART_1).expect("part 1 text present");
    assert!(first < second, "part texts out of order");
}

#[test]
fn collapses_custom_tool_call_output_text_array() {
    let mut value = json!({
        "input": [
            {
                "type": "custom_tool_call_output",
                "call_id": "call-1",
                "output": [
                    {"type": "input_text", "text": "first"},
                    {"type": "input_text", "text": "second"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "custom_tool_call_output",
            "call_id": "call-1",
            "output": "first\nsecond"
        })
    );
}

#[test]
fn collapses_all_encrypted_tool_output_to_empty_string() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": [{"type": "encrypted_content", "encrypted_content": "cipher"}]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": ""
        })
    );
}

#[test]
fn relabels_and_drops_encrypted_in_mixed_tool_output() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": [
                    {"type": "input_text", "text": "A"},
                    {"type": "encrypted_content", "encrypted_content": "cipher"}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": [{"type": "text", "text": "A"}]
        })
    );
}

#[test]
fn leaves_empty_tool_output_array_untouched() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": []
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": []
        })
    );
}

#[test]
fn collapses_blank_only_tool_output_to_empty_string() {
    let mut value = json!({
        "input": [
            {
                "type": "function_call_output",
                "call_id": "call-1",
                "output": [
                    {"type": "input_text", "text": " "},
                    {"type": "input_text", "text": ""}
                ]
            }
        ]
    });

    super::normalize_content_types(&mut value);

    assert_eq!(
        value["input"][0],
        json!({
            "type": "function_call_output",
            "call_id": "call-1",
            "output": ""
        })
    );
}
