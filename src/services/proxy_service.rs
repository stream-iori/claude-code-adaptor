use crate::adapters::{
    request_adapter::ClaudeToQwenAdapter, 
    response_adapter::QwenToClaudeAdapter,
    Adaptor
};
use crate::config::Config;
use crate::models::{claude::*, claude_count_tokens::*, claude_messages::*};
use crate::services::qwen_service::{QwenClient, QwenService};
use crate::services::token_counter::TokenCounter;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

#[derive(Clone)]
pub struct AppState {
    pub qwen_service: QwenService,
    pub request_adapter: ClaudeToQwenAdapter,
    pub response_adapter: QwenToClaudeAdapter,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let qwen_service = QwenService::new(config.qwen);
        Self {
            qwen_service,
            request_adapter: ClaudeToQwenAdapter,
            response_adapter: QwenToClaudeAdapter,
        }
    }
}

pub async fn health_check() -> Result<&'static str, StatusCode> {
    Ok("OK")
}

pub async fn handle_chat_completion(
    State(state): State<AppState>,
    Json(request): Json<ClaudeRequest>,
) -> Result<Json<ClaudeResponse>, StatusCode> {
    // Transform Claude request to Qwen request
    let qwen_request = state
        .request_adapter
        .adapt(request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Send request to Qwen API
    let qwen_response = state
        .qwen_service
        .send_request(qwen_request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Transform Qwen response to Claude response
    let claude_response = state
        .response_adapter
        .adapt(qwen_response)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(claude_response))
}

pub async fn handle_claude_messages(
    State(state): State<AppState>,
    Json(request): Json<ClaudeMessagesRequest>,
) -> Result<Json<ClaudeMessagesResponse>, StatusCode> {
    // Convert Claude Messages format to Claude Chat format for compatibility
    let claude_request = ClaudeRequest {
        model: request.model,
        messages: request
            .messages
            .into_iter()
            .map(|msg| {
                let content = msg
                    .content
                    .into_iter()
                    .find_map(|block| match block {
                        ContentBlock::Text { text, .. } => Some(text),
                        _ => None,
                    })
                    .unwrap_or_default();

                ClaudeMessage {
                    role: match msg.role {
                        MessageRole::User => ClaudeRole::User,
                        MessageRole::Assistant => ClaudeRole::Assistant,
                    },
                    content,
                }
            })
            .collect(),
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        stream: request.stream,
        system: request.system,
        tools: request.tools.map(|tools| {
            tools
                .into_iter()
                .map(|tool| crate::models::claude::ToolDefinition {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: tool.name,
                        description: tool.description,
                        parameters: tool.input_schema,
                    },
                })
                .collect()
        }),
        tool_choice: request.tool_choice.map(|choice| match choice {
            crate::models::claude_messages::ToolChoice::Auto => crate::models::claude::ToolChoice {
                choice_type: "auto".to_string(),
                function: None,
            },
            crate::models::claude_messages::ToolChoice::Any => crate::models::claude::ToolChoice {
                choice_type: "any".to_string(),
                function: None,
            },
            crate::models::claude_messages::ToolChoice::Tool { choice_type, name } => {
                crate::models::claude::ToolChoice {
                    choice_type,
                    function: Some(FunctionChoice { name }),
                }
            }
        }),
    };

    // Transform Claude request to Qwen request
    let qwen_request = state
        .request_adapter
        .adapt(claude_request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Send request to Qwen API
    let qwen_response = state
        .qwen_service
        .send_request(qwen_request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Transform Qwen response to Claude Messages format
    let choice = &qwen_response.output.choices[0];
    let mut content = Vec::new();

    // Add text content if present
    if !choice.message.content.is_empty() {
        content.push(ResponseContentBlock::Text {
            text: choice.message.content.clone(),
        });
    }

    // Add tool calls if present
    if let Some(tool_calls) = &choice.tool_calls {
        for call in tool_calls {
            content.push(ResponseContentBlock::ToolUse {
                id: call.id.clone(),
                name: call.function.name.clone(),
                input: call.function.arguments.clone(),
            });
        }
    }

    let claude_response = ClaudeMessagesResponse {
        id: uuid::Uuid::new_v4().to_string(),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content,
        model: "qwen3-coder".to_string(),
        stop_reason: choice.finish_reason.clone(),
        stop_sequence: None,
        usage: Usage {
            input_tokens: qwen_response.usage.input_tokens,
            output_tokens: qwen_response.usage.output_tokens,
        },
    };

    Ok(Json(claude_response))
}

pub async fn handle_count_tokens(
    State(_state): State<AppState>,
    Json(request): Json<ClaudeCountTokensRequest>,
) -> Result<Json<ClaudeCountTokensResponse>, StatusCode> {
    let response = TokenCounter::count_tokens(request);
    Ok(Json(response))
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/v1/chat/completions", post(handle_chat_completion))
        .route("/v1/messages", post(handle_claude_messages))
        .route("/v1/messages/count_tokens", post(handle_count_tokens))
        .with_state(state)
}
