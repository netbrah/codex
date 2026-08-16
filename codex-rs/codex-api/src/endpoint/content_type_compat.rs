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
pub fn normalize_content_types(value: &mut serde_json::Value) {
    let Some(input) = value.get_mut("input").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for item in input.iter_mut() {
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
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "content_type_compat_tests.rs"]
mod tests;
