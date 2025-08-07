use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenRequest {
    pub model: String,
    pub input: QwenInput,
    pub parameters: QwenParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenInput {
    pub messages: Vec<QwenMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenMessage {
    pub role: QwenRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QwenRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoice {
    #[serde(rename = "type")]
    pub choice_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenResponse {
    pub output: QwenOutput,
    pub usage: QwenUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenOutput {
    pub choices: Vec<QwenChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenChoice {
    pub message: QwenMessage,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenStreamResponse {
    pub output: QwenStreamOutput,
    pub usage: Option<QwenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenStreamOutput {
    pub choices: Vec<QwenStreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenStreamChoice {
    pub message: QwenMessage,
    pub finish_reason: Option<String>,
}