//! SSE parser for the Anthropic `/messages` wire protocol.
//!
//! Maps Anthropic SSE events into [`ResponseEvent`] so the rest of codex-rs
//! is wire-protocol agnostic.

use crate::common::ResponseEvent;
use crate::error::ApiError;
use codex_client::ByteStream;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::TokenUsage;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

use crate::common::ResponseStream;

/// Tracks in-flight content blocks by index.
struct BlockTracker {
    blocks: HashMap<u64, BlockState>,
}

enum BlockState {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
        signature: String,
    },
    ToolUse {
        call_id: String,
        name: String,
        arguments: String,
    },
    RedactedThinking {
        data: String,
    },
}

impl BlockTracker {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }
}

/// Raw deserialized SSE data from an Anthropic `/messages` event.
#[derive(Debug, Deserialize)]
struct MessagesStreamEvent {
    #[serde(rename = "type")]
    kind: String,

    #[serde(default)]
    index: Option<u64>,

    #[serde(default)]
    content_block: Option<serde_json::Value>,

    #[serde(default)]
    delta: Option<serde_json::Value>,

    #[serde(default)]
    message: Option<serde_json::Value>,

    #[serde(default)]
    usage: Option<serde_json::Value>,

    #[serde(default)]
    error: Option<serde_json::Value>,
}

/// Spawns a task that reads SSE events from a `/messages` byte stream and maps
/// them into `ResponseEvent`s on the returned channel.
pub fn spawn_messages_stream(stream: ByteStream, idle_timeout: Duration) -> ResponseStream {
    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);
    tokio::spawn(process_messages_sse(stream, tx_event, idle_timeout));
    ResponseStream { rx_event }
}

async fn process_messages_sse(
    stream: ByteStream,
    tx_event: mpsc::Sender<Result<ResponseEvent, ApiError>>,
    idle_timeout: Duration,
) {
    let mut sse_stream = stream.eventsource();
    let mut tracker = BlockTracker::new();
    let mut response_id = String::new();
    let mut usage_holder: Option<AnthropicUsage> = None;
    let mut stop_reason: Option<String> = None;

    loop {
        let response = timeout(idle_timeout, sse_stream.next()).await;

        let sse = match response {
            Ok(Some(Ok(sse))) => sse,
            Ok(Some(Err(e))) => {
                debug!("Messages SSE error: {e:#}");
                let _ = tx_event.send(Err(ApiError::Stream(e.to_string()))).await;
                return;
            }
            Ok(None) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream(
                        "messages stream closed before message_stop".into(),
                    )))
                    .await;
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream(
                        "idle timeout waiting for messages SSE".into(),
                    )))
                    .await;
                return;
            }
        };

        if sse.data.is_empty() {
            continue;
        }

        trace!("Messages SSE event: {}", &sse.data);

        let event: MessagesStreamEvent = match serde_json::from_str(&sse.data) {
            Ok(event) => event,
            Err(e) => {
                debug!(
                    "Failed to parse messages SSE event: {e}, data: {}",
                    &sse.data
                );
                continue;
            }
        };

        match event.kind.as_str() {
            "message_start" => {
                if let Some(msg) = &event.message {
                    if let Some(id) = msg.get("id").and_then(|v| v.as_str()) {
                        response_id = id.to_owned();
                    }
                    if let Some(u) = msg.get("usage") {
                        if let Ok(u) = serde_json::from_value::<AnthropicUsage>(u.clone()) {
                            usage_holder = Some(u);
                        }
                    }
                    if let Some(model) = msg.get("model").and_then(|v| v.as_str()) {
                        if tx_event
                            .send(Ok(ResponseEvent::ServerModel(model.to_owned())))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                }
                if tx_event.send(Ok(ResponseEvent::Created)).await.is_err() {
                    return;
                }
            }

            "content_block_start" => {
                if let (Some(index), Some(block)) = (event.index, &event.content_block) {
                    if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                        match block_type {
                            "text" => {
                                tracker.blocks.insert(
                                    index,
                                    BlockState::Text {
                                        text: String::new(),
                                    },
                                );
                                let item = ResponseItem::Message {
                                    id: None,
                                    role: "assistant".to_owned(),
                                    content: vec![],
                                    end_turn: None,
                                    phase: None,
                                };
                                if tx_event
                                    .send(Ok(ResponseEvent::OutputItemAdded(item)))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }
                            "thinking" => {
                                tracker.blocks.insert(
                                    index,
                                    BlockState::Thinking {
                                        thinking: String::new(),
                                        signature: String::new(),
                                    },
                                );
                                let item = ResponseItem::Reasoning {
                                    id: String::new(),
                                    summary: Vec::new(),
                                    content: None,
                                    encrypted_content: None,
                                    raw_wire_block: None,
                                };
                                if tx_event
                                    .send(Ok(ResponseEvent::OutputItemAdded(item)))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }
                            "tool_use" => {
                                let call_id = block
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_owned();
                                let name = block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_owned();
                                tracker.blocks.insert(
                                    index,
                                    BlockState::ToolUse {
                                        call_id: call_id.clone(),
                                        name: name.clone(),
                                        arguments: String::new(),
                                    },
                                );
                                let item = ResponseItem::FunctionCall {
                                    id: None,
                                    name: name.clone(),
                                    namespace: None,
                                    arguments: String::new(),
                                    call_id: call_id.clone(),
                                };
                                if tx_event
                                    .send(Ok(ResponseEvent::OutputItemAdded(item)))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }
                            "redacted_thinking" => {
                                let data = block
                                    .get("data")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_owned();
                                tracker
                                    .blocks
                                    .insert(index, BlockState::RedactedThinking { data });
                            }
                            _ => {
                                trace!("ignoring unknown content_block type: {block_type}");
                            }
                        }
                    }
                }
            }

            "content_block_delta" => {
                if let (Some(index), Some(delta)) = (event.index, &event.delta) {
                    if let Some(delta_type) = delta.get("type").and_then(|v| v.as_str()) {
                        match delta_type {
                            "text_delta" => {
                                if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                    if let Some(BlockState::Text { text: acc, .. }) =
                                        tracker.blocks.get_mut(&index)
                                    {
                                        acc.push_str(text);
                                        if tx_event
                                            .send(Ok(ResponseEvent::OutputTextDelta(
                                                text.to_owned(),
                                            )))
                                            .await
                                            .is_err()
                                        {
                                            return;
                                        }
                                    }
                                }
                            }
                            "thinking_delta" => {
                                if let Some(thinking) =
                                    delta.get("thinking").and_then(|v| v.as_str())
                                {
                                    if let Some(BlockState::Thinking { thinking: acc, .. }) =
                                        tracker.blocks.get_mut(&index)
                                    {
                                        acc.push_str(thinking);
                                        if tx_event
                                            .send(Ok(ResponseEvent::ReasoningContentDelta {
                                                delta: thinking.to_owned(),
                                                content_index: index as i64,
                                            }))
                                            .await
                                            .is_err()
                                        {
                                            return;
                                        }
                                    }
                                }
                            }
                            "signature_delta" => {
                                if let Some(sig) = delta.get("signature").and_then(|v| v.as_str()) {
                                    if let Some(BlockState::Thinking { signature: acc, .. }) =
                                        tracker.blocks.get_mut(&index)
                                    {
                                        acc.push_str(sig);
                                    }
                                }
                            }
                            "input_json_delta" => {
                                if let Some(partial) =
                                    delta.get("partial_json").and_then(|v| v.as_str())
                                {
                                    if let Some(BlockState::ToolUse { arguments: acc, .. }) =
                                        tracker.blocks.get_mut(&index)
                                    {
                                        acc.push_str(partial);
                                    }
                                }
                            }
                            _ => {
                                trace!("ignoring unknown delta type: {delta_type}");
                            }
                        }
                    }
                }
            }

            "content_block_stop" => {
                if let Some(index) = event.index {
                    match tracker.blocks.remove(&index) {
                        Some(BlockState::ToolUse {
                            call_id,
                            name,
                            arguments,
                        }) => {
                            let item = ResponseItem::FunctionCall {
                                id: None,
                                name,
                                namespace: None,
                                arguments,
                                call_id,
                            };
                            if tx_event
                                .send(Ok(ResponseEvent::OutputItemDone(item)))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        Some(BlockState::Text { text }) => {
                            let item = ResponseItem::Message {
                                id: None,
                                role: "assistant".to_owned(),
                                content: vec![ContentItem::OutputText { text }],
                                end_turn: None,
                                phase: None,
                            };
                            if tx_event
                                .send(Ok(ResponseEvent::OutputItemDone(item)))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        Some(BlockState::Thinking {
                            thinking,
                            signature,
                        }) => {
                            // Build the raw wire block for byte-identical replay.
                            // This is the exact JSON block Anthropic expects when
                            // the conversation history is sent back.
                            let raw_block = if signature.is_empty() {
                                serde_json::json!({
                                    "type": "thinking",
                                    "thinking": &thinking,
                                })
                            } else {
                                serde_json::json!({
                                    "type": "thinking",
                                    "thinking": &thinking,
                                    "signature": &signature,
                                })
                            };
                            let item = ResponseItem::Reasoning {
                                id: String::new(),
                                summary: vec![
                                    codex_protocol::models::ReasoningItemReasoningSummary::SummaryText {
                                        text: thinking,
                                    },
                                ],
                                content: None,
                                encrypted_content: if signature.is_empty() {
                                    None
                                } else {
                                    Some(signature)
                                },
                                raw_wire_block: Some(raw_block),
                            };
                            if tx_event
                                .send(Ok(ResponseEvent::OutputItemDone(item)))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        Some(BlockState::RedactedThinking { data }) => {
                            // Build raw wire block for byte-identical replay.
                            let raw_block = serde_json::json!({
                                "type": "redacted_thinking",
                                "data": &data,
                            });
                            // Sentinel prefix "\0REDACTED\0" distinguishes redacted thinking
                            // from real Anthropic signatures (which are base64 and cannot
                            // contain null bytes). Consumed by messages_wire.rs translator.
                            let item = ResponseItem::Reasoning {
                                id: String::new(),
                                summary: Vec::new(),
                                content: None,
                                encrypted_content: Some(format!("\0REDACTED\0{data}")),
                                raw_wire_block: Some(raw_block),
                            };
                            if tx_event
                                .send(Ok(ResponseEvent::OutputItemDone(item)))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        None => {}
                    }
                }
            }

            "message_delta" => {
                if let Some(delta) = &event.delta {
                    if let Some(reason) = delta.get("stop_reason").and_then(|r| r.as_str()) {
                        trace!("stop_reason: {reason}");
                        stop_reason = Some(reason.to_owned());
                    }
                }
                if let Some(usage_val) = &event.usage
                    && let Ok(u) = serde_json::from_value::<AnthropicUsage>(usage_val.clone())
                {
                    usage_holder = Some(merge_usage(usage_holder, u));
                }
            }

            "message_stop" => {
                let token_usage = usage_holder.map(|u| {
                    let input = u.input_tokens.unwrap_or(0);
                    let output = u.output_tokens.unwrap_or(0);
                    let cached = u.cache_read_input_tokens.unwrap_or(0);
                    let cache_created = u.cache_creation_input_tokens.unwrap_or(0);
                    TokenUsage {
                        input_tokens: input,
                        cached_input_tokens: cached,
                        cache_creation_input_tokens: cache_created,
                        output_tokens: output,
                        // Anthropic does not currently expose thinking token counts.
                        reasoning_output_tokens: 0,
                        // Anthropic reports cache_read_input_tokens separately from
                        // input_tokens, so total = input + cached + output.
                        total_tokens: input + cached + output,
                    }
                });
                if tx_event
                    .send(Ok(ResponseEvent::Completed {
                        stop_reason: stop_reason.take(),
                        response_id: response_id.clone(),
                        token_usage,
                    }))
                    .await
                    .is_err()
                {
                    return;
                }
                return;
            }

            "ping" => {}

            "error" => {
                let message = event
                    .error
                    .as_ref()
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown anthropic error");

                let error_type = event
                    .error
                    .as_ref()
                    .and_then(|e| e.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                let api_error = match error_type {
                    "overloaded_error" => ApiError::ServerOverloaded,
                    "rate_limit_error" => ApiError::RateLimit(message.to_owned()),
                    _ => ApiError::Stream(format!("Anthropic API error: {message}")),
                };

                let _ = tx_event.send(Err(api_error)).await;
                return;
            }

            _ => {
                trace!("ignoring unknown /messages SSE event type: {}", event.kind);
            }
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct AnthropicUsage {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    cache_read_input_tokens: Option<i64>,
    cache_creation_input_tokens: Option<i64>,
}

fn merge_usage(existing: Option<AnthropicUsage>, new: AnthropicUsage) -> AnthropicUsage {
    match existing {
        None => new,
        Some(prev) => AnthropicUsage {
            input_tokens: new.input_tokens.or(prev.input_tokens),
            output_tokens: new.output_tokens.or(prev.output_tokens),
            cache_read_input_tokens: new.cache_read_input_tokens.or(prev.cache_read_input_tokens),
            cache_creation_input_tokens: new
                .cache_creation_input_tokens
                .or(prev.cache_creation_input_tokens),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio_util::io::ReaderStream;

    fn fixture_to_byte_stream(lines: &[&str]) -> ByteStream {
        let mut content = String::new();
        for line in lines {
            content.push_str(line);
            content.push('\n');
        }
        let reader = std::io::Cursor::new(content);
        let stream = ReaderStream::new(reader)
            .map(|r| r.map_err(|e| codex_client::TransportError::Network(e.to_string())));
        Box::pin(stream)
    }

    #[tokio::test]
    async fn test_basic_text_response() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" world\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        assert!(events.len() >= 4);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Ok(ResponseEvent::ServerModel(_)))),
            "must emit ServerModel from message_start"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Ok(ResponseEvent::Created))),
            "must emit Created"
        );

        let mut found_text_delta = false;
        let mut found_output_item_done = false;
        let mut found_completed = false;
        for event in &events {
            match event {
                Ok(ResponseEvent::OutputTextDelta(t)) if t == "Hello" => {
                    found_text_delta = true;
                }
                Ok(ResponseEvent::OutputItemDone(ResponseItem::Message { content, .. })) => {
                    if let Some(ContentItem::OutputText { text }) = content.first() {
                        assert_eq!(text, "Hello world");
                    }
                    found_output_item_done = true;
                }
                Ok(ResponseEvent::Completed {
                    response_id,
                    token_usage,
                    ..
                }) => {
                    assert_eq!(response_id, "msg_123");
                    assert!(token_usage.is_some());
                    found_completed = true;
                }
                _ => {}
            }
        }
        assert!(found_text_delta);
        assert!(found_output_item_done);
        assert!(found_completed);
    }

    #[tokio::test]
    async fn test_tool_use_response() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_456\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":20,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_01\",\"name\":\"shell\",\"input\":{}}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"command\\\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\": \\\"ls\\\"}\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":15}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let mut found_tool_call = false;
        for event in &events {
            if let Ok(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                call_id,
                name,
                arguments,
                ..
            })) = event
            {
                assert_eq!(call_id, "toolu_01");
                assert_eq!(name, "shell");
                assert_eq!(arguments, "{\"command\": \"ls\"}");
                found_tool_call = true;
            }
        }
        assert!(found_tool_call);
    }

    #[tokio::test]
    async fn test_error_event_extracted() {
        let fixture = vec![
            "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut found_error = false;
        while let Some(event) = rx.recv().await {
            if let Err(ApiError::ServerOverloaded) = event {
                found_error = true;
            }
        }
        assert!(found_error);
    }

    #[tokio::test]
    async fn test_rate_limit_error() {
        let fixture = vec![
            "data: {\"type\":\"error\",\"error\":{\"type\":\"rate_limit_error\",\"message\":\"Rate limited\"}}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut found_error = false;
        while let Some(event) = rx.recv().await {
            if let Err(ApiError::RateLimit(msg)) = event {
                assert_eq!(msg, "Rate limited");
                found_error = true;
            }
        }
        assert!(found_error);
    }

    #[tokio::test]
    async fn test_thinking_block_emits_reasoning_with_signature() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_789\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"Let me think\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\" about this\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig_abc123\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"Here is my answer\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":1}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":20}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let mut found_reasoning = false;
        let mut found_text = false;
        let mut found_thinking_delta = false;
        for event in &events {
            match event {
                Ok(ResponseEvent::ReasoningContentDelta { delta, .. })
                    if delta.contains("Let me think") =>
                {
                    found_thinking_delta = true;
                }
                Ok(ResponseEvent::OutputItemDone(ResponseItem::Reasoning {
                    summary,
                    encrypted_content,
                    ..
                })) => {
                    assert!(!summary.is_empty());
                    assert_eq!(
                        encrypted_content.as_deref(),
                        Some("sig_abc123"),
                        "signature must be preserved"
                    );
                    found_reasoning = true;
                }
                Ok(ResponseEvent::OutputItemDone(ResponseItem::Message { content, .. })) => {
                    if let Some(ContentItem::OutputText { text }) = content.first() {
                        assert_eq!(text, "Here is my answer");
                        found_text = true;
                    }
                }
                _ => {}
            }
        }
        assert!(found_thinking_delta, "should emit thinking deltas");
        assert!(found_reasoning, "should emit Reasoning item on block stop");
        assert!(found_text, "should emit text block");
    }

    #[tokio::test]
    async fn test_redacted_thinking_block() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_red\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"redacted_thinking\",\"data\":\"opaque_encrypted_data_xyz\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"Done\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":1}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":3}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let mut found_redacted = false;
        for event in &events {
            if let Ok(ResponseEvent::OutputItemDone(ResponseItem::Reasoning {
                encrypted_content,
                ..
            })) = event
            {
                if let Some(ec) = encrypted_content {
                    if ec.starts_with("\0REDACTED\0") {
                        assert_eq!(ec, "\0REDACTED\0opaque_encrypted_data_xyz");
                        found_redacted = true;
                    }
                }
            }
        }
        assert!(found_redacted, "should emit redacted thinking as Reasoning");
    }

    #[tokio::test]
    async fn test_interleaved_text_and_tool_use() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_multi\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":30,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Let me check\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_a\",\"name\":\"read_file\",\"input\":{}}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\": \\\"main.rs\\\"}\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":1}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":2,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_b\",\"name\":\"shell\",\"input\":{}}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"command\\\": \\\"ls\\\"}\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":2}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":40}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let mut tool_calls = Vec::new();
        let mut found_text = false;
        for event in &events {
            match event {
                Ok(ResponseEvent::OutputItemDone(ResponseItem::Message { .. })) => {
                    found_text = true;
                }
                Ok(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                    call_id,
                    name,
                    ..
                })) => {
                    tool_calls.push((call_id.clone(), name.clone()));
                }
                _ => {}
            }
        }
        assert!(found_text, "should emit text block before tools");
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(
            tool_calls[0],
            ("toolu_a".to_string(), "read_file".to_string())
        );
        assert_eq!(tool_calls[1], ("toolu_b".to_string(), "shell".to_string()));
    }

    #[tokio::test]
    async fn test_usage_token_tracking() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_tok\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":100,\"output_tokens\":0,\"cache_read_input_tokens\":50,\"cache_creation_input_tokens\":25}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":42}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let mut found_usage = false;
        for event in &events {
            if let Ok(ResponseEvent::Completed { token_usage, .. }) = event {
                let usage = token_usage.as_ref().expect("usage should be present");
                assert_eq!(usage.input_tokens, 100);
                assert_eq!(usage.output_tokens, 42);
                assert_eq!(usage.cached_input_tokens, 50);
                assert_eq!(usage.cache_creation_input_tokens, 25);
                assert_eq!(usage.total_tokens, 192);
                found_usage = true;
            }
        }
        assert!(
            found_usage,
            "should track token usage across message events"
        );
    }

    #[tokio::test]
    async fn test_ping_events_ignored() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_ping\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"ping\"}",
            "",
            "data: {\"type\":\"ping\"}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hi\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        assert!(
            events
                .iter()
                .any(|e| matches!(e, Ok(ResponseEvent::Completed { .. }))),
            "should complete despite ping events"
        );
    }

    #[tokio::test]
    async fn test_unknown_event_types_ignored() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_unk\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"some_future_event\",\"data\":{\"foo\":\"bar\"}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        assert!(
            events
                .iter()
                .any(|e| matches!(e, Ok(ResponseEvent::Completed { .. }))),
            "should complete despite unknown events"
        );
    }

    #[tokio::test]
    async fn test_output_item_added_precedes_text_deltas() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_oia\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let ok_events: Vec<_> = events.iter().filter_map(|e| e.as_ref().ok()).collect();

        let added_idx = ok_events
            .iter()
            .position(|e| {
                matches!(
                    e,
                    ResponseEvent::OutputItemAdded(ResponseItem::Message { .. })
                )
            })
            .expect("must emit OutputItemAdded(Message)");
        let delta_idx = ok_events
            .iter()
            .position(|e| matches!(e, ResponseEvent::OutputTextDelta(_)))
            .expect("must emit OutputTextDelta");
        assert!(
            added_idx < delta_idx,
            "OutputItemAdded must precede OutputTextDelta"
        );
    }

    #[tokio::test]
    async fn test_thinking_output_item_added_precedes_reasoning_delta() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_toia\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"hmm\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig1\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"done\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":1}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let ok_events: Vec<_> = events.iter().filter_map(|e| e.as_ref().ok()).collect();

        let added_idx = ok_events
            .iter()
            .position(|e| {
                matches!(
                    e,
                    ResponseEvent::OutputItemAdded(ResponseItem::Reasoning { .. })
                )
            })
            .expect("must emit OutputItemAdded(Reasoning)");
        let delta_idx = ok_events
            .iter()
            .position(|e| matches!(e, ResponseEvent::ReasoningContentDelta { .. }))
            .expect("must emit ReasoningContentDelta");
        assert!(
            added_idx < delta_idx,
            "OutputItemAdded(Reasoning) must precede ReasoningContentDelta"
        );
    }

    #[tokio::test]
    async fn test_tool_use_output_item_added() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_tuoia\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_oia\",\"name\":\"shell\",\"input\":{}}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"cmd\\\": \\\"ls\\\"}\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":10}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let ok_events: Vec<_> = events.iter().filter_map(|e| e.as_ref().ok()).collect();

        let added = ok_events.iter().find(|e| {
            matches!(
                e,
                ResponseEvent::OutputItemAdded(ResponseItem::FunctionCall { name, .. })
                if name == "shell"
            )
        });
        assert!(
            added.is_some(),
            "must emit OutputItemAdded(FunctionCall) for tool_use blocks"
        );

        let added_idx = ok_events
            .iter()
            .position(|e| {
                matches!(
                    e,
                    ResponseEvent::OutputItemAdded(ResponseItem::FunctionCall { .. })
                )
            })
            .unwrap();
        let done_idx = ok_events
            .iter()
            .position(|e| {
                matches!(
                    e,
                    ResponseEvent::OutputItemDone(ResponseItem::FunctionCall { .. })
                )
            })
            .expect("must emit OutputItemDone");
        assert!(
            added_idx < done_idx,
            "OutputItemAdded must precede OutputItemDone"
        );
    }

    #[tokio::test]
    async fn test_usage_includes_cache_read_in_total() {
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_cache\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":100,\"output_tokens\":0,\"cache_read_input_tokens\":50,\"cache_creation_input_tokens\":25}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":10}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        for event in &events {
            if let Ok(ResponseEvent::Completed { token_usage, .. }) = event {
                let usage = token_usage.as_ref().expect("should have token usage");
                assert_eq!(usage.input_tokens, 100);
                assert_eq!(usage.cached_input_tokens, 50);
                assert_eq!(usage.cache_creation_input_tokens, 25);
                assert_eq!(usage.output_tokens, 10);
                // total = input + cached + output = 100 + 50 + 10 = 160
                assert_eq!(
                    usage.total_tokens, 160,
                    "total must include cache_read_input_tokens"
                );
                return;
            }
        }
        panic!("did not find Completed event");
    }

    #[tokio::test]
    async fn test_stop_reason_propagated_to_completed() {
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_sr","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":10,"output_tokens":0}}}"#,
            "",
            r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":0}"#,
            "",
            r#"data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":5}}"#,
            "",
            r#"data: {"type":"message_stop"}"#,
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut found_stop_reason = false;
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::Completed { stop_reason, .. }) = event {
                assert_eq!(
                    stop_reason.as_deref(),
                    Some("end_turn"),
                    "stop_reason must be propagated from message_delta"
                );
                found_stop_reason = true;
            }
        }
        assert!(found_stop_reason, "must find Completed with stop_reason");
    }

    #[tokio::test]
    async fn test_stop_reason_tool_use() {
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_tu","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":10,"output_tokens":0}}}"#,
            "",
            r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"call_1","name":"read_file","input":{}}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"path\":\"/tmp/test\"}"}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":0}"#,
            "",
            r#"data: {"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":15}}"#,
            "",
            r#"data: {"type":"message_stop"}"#,
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut found = false;
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::Completed { stop_reason, .. }) = event {
                assert_eq!(stop_reason.as_deref(), Some("tool_use"));
                found = true;
            }
        }
        assert!(found, "must find Completed with stop_reason=tool_use");
    }

    #[tokio::test]
    async fn test_stop_reason_max_tokens() {
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_mt","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":10,"output_tokens":0}}}"#,
            "",
            r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Truncated"}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":0}"#,
            "",
            r#"data: {"type":"message_delta","delta":{"stop_reason":"max_tokens"},"usage":{"output_tokens":4096}}"#,
            "",
            r#"data: {"type":"message_stop"}"#,
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut found = false;
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::Completed { stop_reason, .. }) = event {
                assert_eq!(stop_reason.as_deref(), Some("max_tokens"));
                found = true;
            }
        }
        assert!(found, "must find Completed with stop_reason=max_tokens");
    }

    #[tokio::test]
    async fn test_stop_reason_stop_sequence() {
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_ss","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":10,"output_tokens":0}}}"#,
            "",
            r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Count 1 2 3"}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":0}"#,
            "",
            r#"data: {"type":"message_delta","delta":{"stop_reason":"stop_sequence","stop_sequence":"STOP"},"usage":{"output_tokens":8}}"#,
            "",
            r#"data: {"type":"message_stop"}"#,
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut found = false;
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::Completed { stop_reason, .. }) = event {
                assert_eq!(stop_reason.as_deref(), Some("stop_sequence"));
                found = true;
            }
        }
        assert!(found, "must find Completed with stop_reason=stop_sequence");
    }

    // ── T-3-A: idle timeout fires ──────────────────────────────────────

    #[tokio::test]
    async fn test_incomplete_stream_emits_error() {
        // Stream that starts with message_start but ends without message_stop.
        // Should emit a stream-closed error (either timeout or premature close).
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_t","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":1,"output_tokens":0}}}"#,
            "",
            // No further events — stream ends prematurely
        ];
        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_millis(50));
        let mut rx = response_stream.rx_event;
        let mut got_error = false;
        while let Some(event) = rx.recv().await {
            if let Err(_) = event {
                got_error = true;
            }
        }
        assert!(got_error, "incomplete stream should emit an error");
    }

    // ── T-3-B: error event from API ────────────────────────────────────

    #[tokio::test]
    async fn test_error_event_overloaded_standalone() {
        // Error event without preceding message_start — should still propagate.
        let fixture = vec![
            r#"data: {"type":"error","error":{"type":"overloaded_error","message":"Anthropic API temporarily overloaded"}}"#,
            "",
        ];
        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(5));
        let mut rx = response_stream.rx_event;
        let mut found_error = false;
        while let Some(event) = rx.recv().await {
            if let Err(ApiError::ServerOverloaded) = event {
                found_error = true;
            }
        }
        assert!(found_error, "overloaded error should be propagated as ApiError::ServerOverloaded");
    }

    // ── T-3-C: malformed JSON SSE data skipped ─────────────────────────

    #[tokio::test]
    async fn test_malformed_sse_data_skipped() {
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_m","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":1,"output_tokens":0}}}"#,
            "",
            "data: this is not json at all {{{{",
            "",
            r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"recovered"}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":0}"#,
            "",
            r#"data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":1}}"#,
            "",
            r#"data: {"type":"message_stop"}"#,
            "",
        ];
        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(5));
        let mut rx = response_stream.rx_event;
        let mut completed = false;
        while let Some(event) = rx.recv().await {
            if matches!(event, Ok(ResponseEvent::Completed { .. })) {
                completed = true;
            }
        }
        assert!(
            completed,
            "stream should complete despite malformed intermediate events"
        );
    }

    // ── T-3-D: thinking + tool_use interleaved ─────────────────────────

    #[tokio::test]
    async fn test_thinking_then_tool_use_interleaved() {
        let fixture = vec![
            r#"data: {"type":"message_start","message":{"id":"msg_it","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4.6","usage":{"input_tokens":10,"output_tokens":0}}}"#,
            "",
            r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":""}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"I need a shell command"}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"signature_delta","signature":"SIG="}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":0}"#,
            "",
            r#"data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_it1","name":"shell","input":{}}}"#,
            "",
            r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"cmd\": \"ls\"}"}}"#,
            "",
            r#"data: {"type":"content_block_stop","index":1}"#,
            "",
            r#"data: {"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":15}}"#,
            "",
            r#"data: {"type":"message_stop"}"#,
            "",
        ];
        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(10));
        let mut rx = response_stream.rx_event;
        let mut events: Vec<Result<ResponseEvent, _>> = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        // Must have Reasoning OutputItemDone
        let has_reasoning = events.iter().any(|e| {
            matches!(
                e,
                Ok(ResponseEvent::OutputItemDone(ResponseItem::Reasoning { .. }))
            )
        });
        assert!(has_reasoning, "must emit Reasoning OutputItemDone");

        // Must have FunctionCall OutputItemDone with correct call_id
        let has_tool = events.iter().any(|e| {
            matches!(
                e,
                Ok(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall { call_id, .. }))
                if call_id == "toolu_it1"
            )
        });
        assert!(has_tool, "must emit FunctionCall OutputItemDone with call_id=toolu_it1");

        // Reasoning must precede tool in event order
        let reasoning_idx = events.iter().position(|e| {
            matches!(
                e,
                Ok(ResponseEvent::OutputItemDone(ResponseItem::Reasoning { .. }))
            )
        });
        let tool_idx = events.iter().position(|e| {
            matches!(
                e,
                Ok(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall { .. }))
            )
        });
        assert!(
            reasoning_idx < tool_idx,
            "thinking must complete before tool_use"
        );
    }

    #[tokio::test]
    async fn test_text_delta_gated_on_tracked_block_index() {
        // text_delta at index 0 is tracked (content_block_start precedes it),
        // text_delta at index 99 is untracked (no content_block_start) and must NOT emit.
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_gate\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}",
            "",
            // Tracked: index 0 has a content_block_start
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"tracked\"}}",
            "",
            // Untracked: index 99 has no content_block_start
            "data: {\"type\":\"content_block_delta\",\"index\":99,\"delta\":{\"type\":\"text_delta\",\"text\":\"untracked\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut text_deltas: Vec<String> = Vec::new();
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::OutputTextDelta(t)) = event {
                text_deltas.push(t);
            }
        }

        assert_eq!(
            text_deltas,
            vec!["tracked".to_string()],
            "only text_delta with a tracked block index should be emitted"
        );
    }

    #[tokio::test]
    async fn test_thinking_delta_gated_on_tracked_block_index() {
        // thinking_delta at index 0 is tracked, thinking_delta at index 99 is not.
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_tgate\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}",
            "",
            // Tracked: index 0
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"tracked thinking\"}}",
            "",
            // Untracked: index 99
            "data: {\"type\":\"content_block_delta\",\"index\":99,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"untracked thinking\"}}",
            "",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig_test\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut thinking_deltas: Vec<String> = Vec::new();
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::ReasoningContentDelta { delta, .. }) = event {
                thinking_deltas.push(delta);
            }
        }

        assert_eq!(
            thinking_deltas,
            vec!["tracked thinking".to_string()],
            "only thinking_delta with a tracked block index should be emitted"
        );
    }

    #[tokio::test]
    async fn test_input_json_delta_gated_on_tracked_block_index() {
        // input_json_delta only accumulates into tracked blocks; untracked indices
        // should not affect the final tool arguments.
        let fixture = vec![
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_jgate\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4.6\",\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}",
            "",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_gate\",\"name\":\"shell\",\"input\":{}}}",
            "",
            // Tracked: index 0
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"cmd\\\": \\\"ls\\\"}\"}}",
            "",
            // Untracked: index 99 — should be silently ignored
            "data: {\"type\":\"content_block_delta\",\"index\":99,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"bad\\\": true}\"}}",
            "",
            "data: {\"type\":\"content_block_stop\",\"index\":0}",
            "",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":5}}",
            "",
            "data: {\"type\":\"message_stop\"}",
            "",
        ];

        let stream = fixture_to_byte_stream(&fixture);
        let response_stream = spawn_messages_stream(stream, Duration::from_secs(30));
        let mut rx = response_stream.rx_event;

        let mut tool_args = String::new();
        while let Some(event) = rx.recv().await {
            if let Ok(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                arguments, ..
            })) = event
            {
                tool_args = arguments;
            }
        }

        assert_eq!(
            tool_args, "{\"cmd\": \"ls\"}",
            "only input_json_delta with a tracked block index should accumulate"
        );
    }
}
