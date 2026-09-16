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
