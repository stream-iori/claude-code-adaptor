use crate::adapters::Adaptor;
use crate::models::claude::*;
use crate::models::qwen3::{
    FunctionChoice as QwenFunctionChoice, FunctionDefinition as QwenFunctionDefinition, QwenInput,
    QwenMessage, QwenParameters, QwenRequest, QwenRole, ToolChoice as QwenToolChoice,
    ToolDefinition as QwenToolDefinition,
};
use anyhow::Result;

#[derive(Clone)]
pub struct ClaudeToQwenAdapter;

impl Adaptor for ClaudeToQwenAdapter {
    type From = ClaudeRequest;
    type To = QwenRequest;

    fn before_adapt(&self, from: &Self::From) {
        tracing::debug!("Before request adaptation: {:?}", from);
    }

    fn after_adapt(&self, to: Result<&Self::To, &anyhow::Error>) {
        match to {
            Ok(result) => tracing::debug!("After request adaptation: {:?}", result),
            Err(e) => tracing::debug!("Request adaptation failed {:?}", e),
        }
    }

    fn do_adapt(&self, claude_request: Self::From) -> Result<Self::To> {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system_content) = claude_request.system {
            messages.push(QwenMessage {
                role: QwenRole::System,
                content: system_content,
            });
        }

        // Convert Claude messages to Qwen messages
        for claude_msg in claude_request.messages {
            let qwen_role = match claude_msg.role {
                ClaudeRole::User => QwenRole::User,
                ClaudeRole::Assistant => QwenRole::Assistant,
                ClaudeRole::System => QwenRole::System,
            };

            messages.push(QwenMessage {
                role: qwen_role,
                content: claude_msg.content,
            });
        }

        // Transform tools if provided
        let tools = claude_request.tools.map(|claude_tools| {
            claude_tools
                .into_iter()
                .map(|tool| QwenToolDefinition {
                    tool_type: tool.tool_type,
                    function: QwenFunctionDefinition {
                        name: tool.function.name,
                        description: tool.function.description,
                        parameters: tool.function.parameters,
                    },
                })
                .collect()
        });

        // Transform tool_choice if provided
        let tool_choice = claude_request.tool_choice.map(|choice| QwenToolChoice {
            choice_type: choice.choice_type,
            function: choice
                .function
                .map(|func| QwenFunctionChoice { name: func.name }),
        });

        Ok(QwenRequest {
            model: claude_request.model,
            input: QwenInput { messages },
            parameters: QwenParameters {
                max_tokens: claude_request.max_tokens,
                temperature: claude_request.temperature,
                incremental_output: claude_request.stream,
                result_format: Some("message".to_string()),
                tools,
                tool_choice,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapt_claude_to_qwen() {
        let adapter = ClaudeToQwenAdapter;
        let claude_request = ClaudeRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![ClaudeMessage {
                role: ClaudeRole::User,
                content: "Hello, world!".to_string(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
            system: Some("You are a helpful assistant".to_string()),
            tools: None,
            tool_choice: None,
        };

        let qwen_request = adapter.adapt(claude_request).unwrap();

        assert_eq!(qwen_request.model, "qwen3-coder");
        assert_eq!(qwen_request.input.messages.len(), 2);
        assert_eq!(qwen_request.input.messages[0].role, QwenRole::System);
        assert_eq!(qwen_request.input.messages[1].role, QwenRole::User);
        assert_eq!(qwen_request.parameters.max_tokens, Some(100));
        assert_eq!(qwen_request.parameters.temperature, Some(0.7));
    }
}
