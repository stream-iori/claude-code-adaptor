use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesRequest {
    pub model: String,

    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,

    //mcp_servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    //service_tier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemContentEnum>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    //thinking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemContentEnum {
    String(String),
    Array(Vec<SystemContent>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ContentBlock {
    Text {
        #[serde(rename = "type")]
        content_type: String,
        text: String,
    },
    ToolUse {
        #[serde(rename = "type")]
        content_type: String,
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        #[serde(rename = "type")]
        content_type: String,
        tool_use_id: String,
        content: String,
        is_error: Option<bool>,
    },
}

impl ContentBlock {
    pub fn text(text: String) -> Self {
        ContentBlock::Text {
            content_type: "text".to_string(),
            text,
        }
    }

    pub fn tool_use(id: String, name: String, input: serde_json::Value) -> Self {
        ContentBlock::ToolUse {
            content_type: "tool_use".to_string(),
            id,
            name,
            input,
        }
    }

    pub fn tool_result(tool_use_id: String, content: String, is_error: Option<bool>) -> Self {
        ContentBlock::ToolResult {
            content_type: "tool_result".to_string(),
            tool_use_id,
            content,
            is_error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesResponse {
    pub id: String,

    #[serde(rename = "type")]
    pub response_type: String,

    pub role: String,
    pub content: Vec<ResponseContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ResponseContainer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContainer {
    pub id: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesStreamResponse {
    pub response_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<Delta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<StreamMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_block: Option<StreamContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<ResponseContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Auto,
    Any,
    Tool {
        #[serde(rename = "type")]
        choice_type: String,
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_message() {
        let claude_request = r#"
            {
                "model": "claude-3-5-haiku-20241022",
                "max_tokens": 512,
                "messages": [
                    {
                        "role": "user",
                        "content": "who are you"
                    }
                ],
                "system": [{
                    "type": "text",
                    "text": "Analyze if this message indicates a new conversation topic. If it does, extract a 2-3 word title that captures the new topic. Format your response as a JSON object with two fields: 'isNewTopic' (boolean) and 'title' (string, or null if isNewTopic is false). Only include these fields, no other text."
                }],
                "temperature": 0,
                "metadata": {
                    "user_id": "user_8b8886105677a603d22a9d4b562314eac9258ce75f8c387d16fcd9b80475d6ec_account__session_d4a424e2-ff1a-4110-901e-1e6ba2e7af4c"
                },
                "stream": true
            }
        "#;

        let claude_messages_request: ClaudeMessagesRequest =
            serde_json::from_str(claude_request).expect("Failed to parse ClaudeMessagesRequest");
        assert_eq!(claude_messages_request.model, "claude-3-5-haiku-20241022");
        assert_eq!(claude_messages_request.max_tokens, Some(512));
        assert_eq!(claude_messages_request.messages.len(), 1);
        assert_eq!(claude_messages_request.messages[0].role, MessageRole::User);
        assert_eq!(claude_messages_request.messages[0].content, "who are you");
        match claude_messages_request.system.unwrap() {
            crate::models::claude_messages::SystemContentEnum::Array(contents) => {
                assert_eq!(contents.len(), 1);
                assert_eq!(contents[0].text, "Analyze if this message indicates a new conversation topic. If it does, extract a 2-3 word title that captures the new topic. Format your response as a JSON object with two fields: 'isNewTopic' (boolean) and 'title' (string, or null if isNewTopic is false). Only include these fields, no other text.");
            }
            _ => panic!("Expected array format"),
        }
    }
}
