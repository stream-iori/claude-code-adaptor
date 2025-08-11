use serde::{Deserialize, Serialize};
use crate::models::claude_messages::{InputMessage, ToolDefinition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCountTokensRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<InputMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<SystemPrompt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPrompt {
    #[serde(rename = "type")]
    pub prompt_type: String,
    pub cache_control: Option<CacheControl>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCountTokensResponse {
    pub input_tokens: u32,
}