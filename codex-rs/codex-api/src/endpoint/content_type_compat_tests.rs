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
