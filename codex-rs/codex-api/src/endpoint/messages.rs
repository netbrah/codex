//! HTTP endpoint client for Anthropic `/messages`.

use crate::auth::AuthProvider;
use crate::common::ResponseStream;
use crate::endpoint::session::EndpointSession;
use crate::error::ApiError;
use crate::provider::Provider;
use crate::sse::spawn_messages_stream;
use codex_client::HttpTransport;
use codex_client::RequestTelemetry;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;

/// Request body for Anthropic `/v1/messages`.
#[derive(Debug, Clone, Serialize)]
pub struct MessagesApiRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub max_tokens: u32,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

pub struct MessagesClient<T: HttpTransport, A: AuthProvider> {
    session: EndpointSession<T, A>,
}

impl<T: HttpTransport, A: AuthProvider> MessagesClient<T, A> {
    pub fn new(transport: T, provider: Provider, auth: A) -> Self {
        Self {
            session: EndpointSession::new(transport, provider, auth),
        }
    }

    pub fn with_telemetry(self, request: Option<Arc<dyn RequestTelemetry>>) -> Self {
        Self {
            session: self.session.with_request_telemetry(request),
        }
    }

    #[instrument(
        name = "messages.stream_request",
        level = "info",
        skip_all,
        fields(
            transport = "messages_http",
            http.method = "POST",
            api.path = "messages"
        )
    )]
    pub async fn stream_request(
        &self,
        request: MessagesApiRequest,
        extra_headers: HeaderMap,
    ) -> Result<ResponseStream, ApiError> {
        let body = serde_json::to_value(&request)
            .map_err(|e| ApiError::Stream(format!("failed to encode messages request: {e}")))?;

        let mut headers = extra_headers;
        headers.insert(
            http::header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );

        headers.insert(
            http::HeaderName::from_static("anthropic-version"),
            HeaderValue::from_static("2023-06-01"),
        );

        // Required for extended thinking (interleaved thinking blocks).
        headers.insert(
            http::HeaderName::from_static("anthropic-beta"),
            HeaderValue::from_static("interleaved-thinking-2025-05-14"),
        );

        let stream_response = self
            .session
            .stream_with(Method::POST, Self::path(), headers, Some(body), |_| {})
            .await?;

        Ok(spawn_messages_stream(
            stream_response.bytes,
            self.session.provider().stream_idle_timeout,
        ))
    }

    /// Relative path appended to the provider's base URL.
    ///
    /// The `Provider` joins `base_url + "/" + path`. For Anthropic direct API
    /// the base URL should be `https://api.anthropic.com/v1`; for LiteLLM
    /// proxies it's typically `https://proxy-host` (the proxy routes
    /// `POST /messages` internally).
    fn path() -> &'static str {
        "messages"
    }
}
