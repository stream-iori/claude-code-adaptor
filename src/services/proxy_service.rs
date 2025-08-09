use crate::adapters::{
    Adaptor, openai_response_adapter::OpenAIToClaudeMessagesAdapter,
    openai_stream_adapter::OpenAIStreamToClaudeStreamAdapter,
    request_adapter::ClaudeMessagesToOpenAIAdapter,
};
use crate::config::Config;
use crate::models::openai::{OpenAIRequest, OpenAIStreamResponse};
use crate::models::{claude_count_tokens::*, claude_messages::*};
use crate::services::openai_service::{OpenAIClient, OpenAIService};
use crate::services::token_counter::TokenCounter;
use axum::body::to_bytes;
use axum::http::Request;
use axum::response::Response;
use axum::middleware::Next;
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::State,
    http::StatusCode,
    http::{HeaderMap, Method, Uri},
    middleware,
    response::IntoResponse,
    response::Sse,
    response::sse::Event,
    routing::post,
};
use futures::StreamExt;
use std::convert::Infallible;
use tracing::{error, info, instrument, warn};

#[derive(Clone)]
pub struct AppState {
    pub openai_service: OpenAIService,
    pub messages_request_adapter: ClaudeMessagesToOpenAIAdapter,
    pub response_adapter: OpenAIToClaudeMessagesAdapter,
    pub stream_adapter: OpenAIStreamToClaudeStreamAdapter,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let openai_service = OpenAIService::new(config.openai);
        Self {
            openai_service,
            messages_request_adapter: ClaudeMessagesToOpenAIAdapter,
            response_adapter: OpenAIToClaudeMessagesAdapter,
            stream_adapter: OpenAIStreamToClaudeStreamAdapter,
        }
    }
}

#[instrument]
pub async fn health_check() -> Result<&'static str, StatusCode> {
    info!("Health check endpoint accessed");
    Ok("OK")
}

#[instrument(skip(state, request), fields(model = %request.model, stream = %request.stream.unwrap_or(false)
))]
pub async fn handle_claude_messages(
    State(state): State<AppState>,
    Json(request): Json<ClaudeMessagesRequest>,
) -> Result<axum::response::Response, StatusCode> {
    let is_stream = request.stream.unwrap_or(false);
    info!("Received Claude messages request");

    if is_stream {
        handle_claude_messages_stream(State(state), Json(request)).await
    } else {
        handle_claude_messages_non_stream(State(state), Json(request)).await
    }
}

#[instrument(skip(state, request), fields(model = %request.model))]
pub async fn handle_claude_messages_non_stream(
    State(state): State<AppState>,
    Json(request): Json<ClaudeMessagesRequest>,
) -> Result<axum::response::Response, StatusCode> {
    info!("Processing non-streaming Claude messages request");

    // Transform Claude Messages request to OpenAI request
    let openai_request = state.messages_request_adapter.adapt(request).map_err(|e| {
        error!("Failed to adapt Claude request to OpenAI format: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    info!("Adapted request to OpenAI format");

    // Send request to Qwen service
    let response = state
        .openai_service
        .send_request(openai_request)
        .await
        .map_err(|e| {
            error!("Failed to send request to OpenAI service: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Received response from OpenAI service");

    // Parse OpenAI response
    let openai_response = response
        .json::<crate::models::openai::OpenAIResponse>()
        .await
        .map_err(|e| {
            error!("Failed to parse OpenAI response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Parsed OpenAI response successfully");

    // Transform to Claude Messages format
    let claude_response = state.response_adapter.adapt(openai_response).map_err(|e| {
        error!("Failed to adapt OpenAI response to Claude format: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Transformed response to Claude format");
    Ok(Json(claude_response).into_response())
}

#[instrument(skip(state, request), fields(model = %request.model))]
pub async fn handle_claude_messages_stream(
    State(state): State<AppState>,
    Json(request): Json<ClaudeMessagesRequest>,
) -> Result<axum::response::Response, StatusCode> {
    info!("Processing streaming Claude messages request");

    // Transform Claude Messages request to OpenAI request
    let mut openai_request = state.messages_request_adapter.adapt(request).map_err(|e| {
        error!("Failed to adapt Claude request to OpenAI format: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Ensure streaming is enabled
    openai_request.stream = Some(true);
    info!("Enabled streaming mode");

    // Send streaming request to openai service
    let response = state
        .openai_service
        .send_request(openai_request)
        .await
        .map_err(|e| {
            error!("Failed to send streaming request to OpenAI service: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Received streaming response from OpenAI service");

    let stream_adapter = state.stream_adapter.clone();
    let stream = response.bytes_stream().map(move |result| {
        let event = match result {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);

                // Handle SSE format
                if text.starts_with("data: ") {
                    let json_str = text.trim_start_matches("data: ").trim();

                    if json_str == "[DONE]" {
                        info!("Stream completed - received [DONE] signal");
                        Event::default().data("[DONE]")
                    } else {
                        // Parse OpenAI stream response
                        match serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                            Ok(stream_response) => {
                                info!("Received stream chunk from OpenAI");
                                // Transform to Claude format
                                match stream_adapter.adapt(stream_response) {
                                    Ok(claude_stream) => Event::default().data(
                                        serde_json::to_string(&claude_stream).unwrap_or_default(),
                                    ),
                                    Err(e) => {
                                        error!("Failed to adapt stream chunk: {}", e);
                                        Event::default().data("error: failed to adapt response")
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse stream chunk: {}", e);
                                Event::default().data(json_str.to_string())
                            }
                        }
                    }
                } else {
                    Event::default().data(text.to_string())
                }
            }
            Err(e) => {
                error!("Failed to read stream chunk: {}", e);
                Event::default().data("error: failed to read chunk")
            }
        };
        Ok::<_, Infallible>(event)
    });

    let sse_response = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(1))
            .text("keep-alive"),
    );

    info!("Created SSE response for streaming");
    Ok(sse_response.into_response())
}

#[instrument(skip(_state, request))]
pub async fn handle_count_tokens(
    State(_state): State<AppState>,
    Json(request): Json<ClaudeCountTokensRequest>,
) -> Result<Json<ClaudeCountTokensResponse>, StatusCode> {
    info!("Received token counting request");
    let response = TokenCounter::count_tokens(request);
    info!("Completed token counting");
    Ok(Json(response))
}

pub async fn handle_fallback(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    // Log the full request information
    tracing::info!(
        "Fallback handler triggered for unhandled request: {} {}",
        method,
        uri
    );

    // Log headers
    let header_map: std::collections::HashMap<String, String> = headers
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value
                    .to_str()
                    .unwrap_or("<invalid header value>")
                    .to_string(),
            )
        })
        .collect();

    tracing::info!("Request headers: {:#?}", header_map);

    // Log body if available
    let body_str = String::from_utf8_lossy(&body);
    if !body_str.is_empty() {
        tracing::info!("Request body: {}", body_str);
    } else {
        tracing::info!("Request body: <empty>");
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": {
                "message": "Not Found",
                "type": "invalid_request_error",
                "param": null,
                "code": null
            }
        })),
    )
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/v1/messages", post(handle_claude_messages))
        .route("/v1/messages/count_tokens", post(handle_count_tokens))
        .fallback(handle_fallback)
        .layer(middleware::from_fn(log_request_response))
        .with_state(state)
}

async fn log_request_response(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // ====== 打印请求 ======
    let req_body_str = String::from_utf8_lossy(&body);
    info!("--> {} {} body: {}", method, uri, req_body_str);

    // 构建新的 Request 给下一个 handler
    let mut request = Request::new(Body::from(body));
    *request.method_mut() = method.clone();
    *request.uri_mut() = uri.clone();
    *request.headers_mut() = headers.clone();

    // ====== 执行下一个 handler ======
    let res = next.run(request).await;

    // ====== 打印响应 ======
    let (parts, body) = res.into_parts();
    let res_body_bytes = match to_bytes(body, 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let res_body_str = String::from_utf8_lossy(&res_body_bytes);

    info!("<-- {} {} response: {}", method, uri, res_body_str);

    // 重新构建 Response
    let res = Response::from_parts(parts, Body::from(res_body_bytes));

    Ok(res)
}
