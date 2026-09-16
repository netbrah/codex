/// Content-type compatibility for providers whose Responses→ChatCompletions
/// shim does not map `input_text`/`output_text` content part types to the
/// ChatCompletions `text` type (e.g. SGLang, vLLM).
///
/// See [`normalize_content_types`].

/// Rewrites Responses API content part types (`input_text`, `output_text`) to
/// the ChatCompletions `text` type in the serialized request body.
///
/// Some providers (SGLang, vLLM) expose a `/v1/responses` endpoint backed by a
/// Responses→ChatCompletions shim. When that shim converts message content
/// arrays it passes `input_text`/`output_text` through unchanged, causing
/// pydantic validation errors (`Input should be 'text'`). This function walks
/// the `input` array of a serialized [`crate::common::ResponsesApiRequest`]
/// and rewrites those type strings so the shim accepts the request.
///
/// `encrypted_content` content parts are handled per item type. An
/// `agent_message`'s part holds the plain-text inter-agent message (codex never
/// encrypts it — it is a label for OpenAI's server-side handling), so it is
/// relabelled to `text` to preserve it for the receiving model. Everywhere else
/// the same part is genuine provider-side ciphertext that a
/// Responses→ChatCompletions shim hard-rejects, so it is dropped.
pub fn normalize_content_types(value: &mut serde_json::Value) {
    let Some(input) = value.get_mut("input").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for item in input.iter_mut() {
        let is_agent_message = item.get("type").and_then(|t| t.as_str()) == Some("agent_message");
        let Some(content) = item.get_mut("content").and_then(|v| v.as_array_mut()) else {
            continue;
        };
        for part in content.iter_mut() {
            if let Some(type_str) = part.get("type").and_then(|t| t.as_str()) {
                if type_str == "input_text" || type_str == "output_text" {
                    if let Some(obj) = part.as_object_mut() {
                        obj.insert(
                            "type".to_string(),
                            serde_json::Value::String("text".to_string()),
                        );
                    }
                } else if type_str == "encrypted_content" && is_agent_message {
                    // Relabel the plain-text inter-agent payload to a `text` part
                    // so the receiving model actually reads it.
                    if let Some(obj) = part.as_object_mut() {
                        let blob = obj
                            .get("encrypted_content")
                            .cloned()
                            .unwrap_or(serde_json::Value::String(String::new()));
                        obj.clear();
                        obj.insert(
                            "type".to_string(),
                            serde_json::Value::String("text".to_string()),
                        );
                        obj.insert("text".to_string(), blob);
                    }
                }
            }
        }
        // Drop any encrypted_content parts NOT relabelled above: they carry
        // genuine, undecryptable provider-side ciphertext (e.g. reasoning) that a
        // Responses->ChatCompletions shim hard-rejects.
        content
            .retain(|part| part.get("type").and_then(|t| t.as_str()) != Some("encrypted_content"));
    }
}

/// Rewrites Codex/OpenAI-specific `agent_message` input items into plain
/// `message` items so Responses-compatible providers (vLLM, SGLang) that only
/// accept the standard item types do not reject the request with
/// `Unsupported Responses API input item type: 'agent_message'`.
///
/// This is the wire-side counterpart of multi-agent V2's inter-agent messaging.
/// It runs on the *serialized* request body, not the typed `ResponseItem`s:
/// the in-process routing that depends on `author`/`recipient` reads the typed
/// items, so it is unaffected. Each `agent_message` item is rebuilt as a
/// minimal `{"type":"message","role":"user","content":[...]}`; the `content`
/// array is carried through as-is (already text-normalized by
/// [`normalize_content_types`], which must run first). OpenAI-only fields
/// (`id`, `author`, `recipient`) are dropped so a strict provider does not
/// reject them as unknown.
pub fn translate_agent_messages(value: &mut serde_json::Value) {
    let Some(input) = value.get_mut("input").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for item in input.iter_mut() {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        if obj.get("type").and_then(|t| t.as_str()) != Some("agent_message") {
            continue;
        }
        let content = obj.get("content").cloned();
        let mut message = serde_json::Map::new();
        message.insert(
            "type".to_string(),
            serde_json::Value::String("message".to_string()),
        );
        message.insert(
            "role".to_string(),
            serde_json::Value::String("user".to_string()),
        );
        if let Some(content) = content {
            message.insert("content".to_string(), content);
        }
        *item = serde_json::Value::Object(message);
    }
}

#[cfg(test)]
#[path = "content_type_compat_tests.rs"]
mod tests;
