use crate::adapters::Adaptor;
use crate::models::claude::*;
use crate::models::qwen3::QwenResponse;
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct QwenToClaudeAdapter;

impl Adaptor for QwenToClaudeAdapter {
    type From = QwenResponse;
    type To = ClaudeResponse;

    fn before_adapt(&self, from: &Self::From) {
        tracing::debug!("Before response adaptation: {:?}", from);
    }

    fn after_adapt(&self, to: Result<&Self::To, &anyhow::Error>) {
        match to {
            Ok(result) => tracing::debug!("After response adaptation: {:?}", result),
            Err(e) => tracing::debug!("Response adaptation failed {:?}", e),
        }
    }

    fn do_adapt(&self, qwen_response: Self::From) -> Result<Self::To> {
        let choice = &qwen_response.output.choices[0];

        // Handle tool calls if present
        let tool_calls = choice.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|call| ToolCall {
                    id: call.id.clone(),
                    call_type: call.call_type.clone(),
                    function: ToolCallFunction {
                        name: call.function.name.clone(),
                        arguments: call.function.arguments.to_string(),
                    },
                })
                .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::qwen3::{QwenChoice, QwenMessage, QwenOutput, QwenRole, QwenUsage};

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
        assert_eq!(
            claude_response.choices[0].message.content,
            Some("Hello! How can I help you?".to_string())
        );
        assert_eq!(
            claude_response.choices[0].message.role,
            ClaudeRole::Assistant
        );
    }
}
