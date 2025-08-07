use crate::models::claude::*;
use crate::models::qwen3::{QwenResponse, QwenStreamResponse};
use uuid::Uuid;
type Result<T> = std::result::Result<T, crate::models::error::AdaptorError>;

pub trait ResponseAdapter {
    type From;
    type To;
    
    fn adapt(&self, from: Self::From) -> Result<Self::To>;
}

#[derive(Clone)]
pub struct QwenToClaudeAdapter;

impl ResponseAdapter for QwenToClaudeAdapter {
    type From = QwenResponse;
    type To = ClaudeResponse;
    
    fn adapt(&self, qwen_response: Self::From) -> Result<Self::To> {
        let choice = &qwen_response.output.choices[0];
        
        // Handle tool calls if present
        let tool_calls = choice.tool_calls.as_ref().map(|calls| {
            calls.iter().map(|call| {
                ToolCall {
                    id: call.id.clone(),
                    call_type: call.call_type.clone(),
                    function: ToolCallFunction {
                        name: call.function.name.clone(),
                        arguments: call.function.arguments.to_string(),
                    },
                }
            }).collect()
        });

        let message = ClaudeMessageWithTools {
            role: ClaudeRole::Assistant,
            content: Some(choice.message.content.clone()),
            tool_calls,
        };
        
        Ok(ClaudeResponse {
            id: Uuid::new_v4().to_string(),
            model: "qwen3-coder".to_string(), // Could be mapped from qwen_response if available
            usage: ClaudeUsage {
                input_tokens: qwen_response.usage.input_tokens,
                output_tokens: qwen_response.usage.output_tokens,
            },
            choices: vec![ClaudeChoice {
                message,
                finish_reason: choice.finish_reason.clone(),
                index: 0,
            }],
        })
    }
}

#[derive(Clone)]
pub struct QwenToClaudeStreamAdapter;

impl ResponseAdapter for QwenToClaudeStreamAdapter {
    type From = QwenStreamResponse;
    type To = ClaudeStreamResponse;
    
    fn adapt(&self, qwen_response: Self::From) -> Result<Self::To> {
        let choice = &qwen_response.output.choices[0];
        let delta = ClaudeDelta {
            content: Some(choice.message.content.clone()),
        };
        
        Ok(ClaudeStreamResponse {
            id: Uuid::new_v4().to_string(),
            model: "qwen3-coder".to_string(),
            choices: vec![ClaudeStreamChoice {
                delta,
                finish_reason: choice.finish_reason.clone(),
                index: 0,
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::qwen3::{QwenOutput, QwenChoice, QwenMessage, QwenRole, QwenUsage};
    
    #[test]
    fn test_adapt_qwen_to_claude() {
        let adapter = QwenToClaudeAdapter;
        let qwen_response = QwenResponse {
            output: QwenOutput {
                choices: vec![QwenChoice {
                    message: QwenMessage {
                        role: QwenRole::Assistant,
                        content: "Hello! How can I help you?".to_string(),
                    },
                    finish_reason: Some("stop".to_string()),
                    tool_calls: None,
                }],
            },
            usage: QwenUsage {
                input_tokens: 10,
                output_tokens: 15,
                total_tokens: 25,
            },
            request_id: Some("test-id".to_string()),
        };
        
        let claude_response = adapter.adapt(qwen_response).unwrap();
        
        assert!(!claude_response.id.is_empty());
        assert_eq!(claude_response.model, "qwen3-coder");
        assert_eq!(claude_response.usage.input_tokens, 10);
        assert_eq!(claude_response.usage.output_tokens, 15);
        assert_eq!(claude_response.choices.len(), 1);
        assert_eq!(claude_response.choices[0].message.content, Some("Hello! How can I help you?".to_string()));
        assert_eq!(claude_response.choices[0].message.role, ClaudeRole::Assistant);
    }
}