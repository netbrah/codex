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
/// Tool-output items (`function_call_output`, `custom_tool_call_output`)
/// carry their payload in `output` rather than `content`; shims that fold
/// such items into a user message pass those parts through to the
/// ChatCompletions user-content union, which hard-rejects part types it does
/// not know (e.g. `input_text`). An all-text `output` array therefore
/// collapses to a single string (blank segments filtered, segments joined
/// with `"\n"`) — the most conservative legal shape — and a mixed array has
/// its text parts relabelled in place with undecryptable `encrypted_content`
/// parts dropped.
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
        if let Some(content) = item.get_mut("content").and_then(|v| v.as_array_mut()) {
            for part in content.iter_mut() {
                let type_str = part.get("type").and_then(|t| t.as_str());
                if matches!(type_str, Some("input_text") | Some("output_text")) {
                    relabel_text_part(part);
                } else if type_str == Some("encrypted_content") && is_agent_message {
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
            // Drop any encrypted_content parts NOT relabelled above: they carry
            // genuine, undecryptable provider-side ciphertext (e.g. reasoning) that a
            // Responses->ChatCompletions shim hard-rejects.
            content.retain(|part| {
                part.get("type").and_then(|t| t.as_str()) != Some("encrypted_content")
            });
        }
        normalize_tool_output(item);
    }
}

/// Rewrites an `input_text` or `output_text` content part type to the
/// ChatCompletions `text` type in place. Returns `true` when the part was a
/// text part and was relabelled.
fn relabel_text_part(part: &mut serde_json::Value) -> bool {
    let is_text_part = matches!(
        part.get("type").and_then(|t| t.as_str()),
        Some("input_text") | Some("output_text")
    );
    if is_text_part && let Some(obj) = part.as_object_mut() {
        obj.insert(
            "type".to_string(),
            serde_json::Value::String("text".to_string()),
        );
        return true;
    }
    false
}

/// Normalizes a tool-output item's `output` field (see
/// [`normalize_content_types`]): all-text arrays collapse to a single string,
/// mixed arrays keep their shape with text parts relabelled in place and
/// undecryptable `encrypted_content` parts dropped.
fn normalize_tool_output(item: &mut serde_json::Value) {
    let Some(output) = item.get_mut("output") else {
        // No `output` field: untouched.
        return;
    };
    let Some(parts) = output.as_array_mut() else {
        // `null`, a plain string, or any other non-array value: untouched.
        return;
    };
    if parts.is_empty() {
        // No parts that could mismatch the shim union: passthrough.
        return;
    }
    let all_text = parts.iter().all(|part| {
        let Some(obj) = part.as_object() else {
            return false;
        };
        let text_type = matches!(
            obj.get("type").and_then(|t| t.as_str()),
            Some("input_text") | Some("output_text") | Some("text")
        );
        text_type && obj.get("text").and_then(|t| t.as_str()).is_some()
    });
    if all_text {
        // Collapse to a single string, mirroring
        // `function_call_output_content_items_to_text` (protocol `models.rs`):
        // filter empty/whitespace-only segments first, then join the rest.
        let joined = parts
            .iter()
            .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        *output = serde_json::Value::String(joined);
        return;
    }
    for part in parts.iter_mut() {
        relabel_text_part(part);
    }
    // Drop undecryptable provider-side ciphertext (same rationale as the
    // content path): a shim cannot surface it to the model.
    parts.retain(|part| part.get("type").and_then(|t| t.as_str()) != Some("encrypted_content"));
    if parts.is_empty() {
        *output = serde_json::Value::String(String::new());
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
