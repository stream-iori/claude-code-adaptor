use crate::adapters::Adaptor;
use crate::models::claude_messages::{ClaudeMessagesRequest, MessageRole, ToolChoice as ClaudeMessagesToolChoice};
use crate::models::openai::{
    OpenAIRequest, OpenAIMessage, OpenAIRole, OpenAIToolDefinition, OpenAIFunctionDefinition,
    OpenAIToolChoice, OpenAIFunctionChoice,
};
use anyhow::Result;


#[derive(Clone)]
pub struct ClaudeMessagesToOpenAIAdapter;

impl Adaptor for ClaudeMessagesToOpenAIAdapter {
    type From = ClaudeMessagesRequest;
    type To = OpenAIRequest;

    fn before_adapt(&self, from: &Self::From) {
        tracing::debug!("Before messages request adaptation: {:?}", from);
    }

    fn after_adapt(&self, to: Result<&Self::To, &anyhow::Error>) {
        match to {
            Ok(result) => tracing::debug!("After messages request adaptation: {:?}", result),
            Err(e) => tracing::debug!("Messages request adaptation failed {:?}", e),
        }
    }

    fn do_adapt(&self, claude_request: Self::From) -> Result<Self::To> {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system_content) = claude_request.system {
            let system_text = match system_content {
                crate::models::claude_messages::SystemContentEnum::String(text) => text,
                crate::models::claude_messages::SystemContentEnum::Array(contents) => {
                    contents
                        .into_iter()
                        .filter(|c| c.content_type == "text")
                        .map(|c| c.text)
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };
            messages.push(OpenAIMessage {
                role: OpenAIRole::System,
                content: Some(system_text),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        // Convert Claude messages to OpenAI messages
        for claude_msg in claude_request.messages {
            let openai_role = match claude_msg.role {
                MessageRole::User => OpenAIRole::User,
                MessageRole::Assistant => OpenAIRole::Assistant,
            };

            messages.push(OpenAIMessage {
                role: openai_role,
                content: Some(claude_msg.content),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        // Transform tools if provided
        let tools = claude_request.tools.map(|claude_tools| {
            claude_tools
                .into_iter()
                .map(|tool| OpenAIToolDefinition {
                    tool_type: "function".to_string(),
                    function: OpenAIFunctionDefinition {
                        name: tool.name,
                        description: Some(tool.description),
                        parameters: tool.input_schema,
                    },
                })
                .collect()
        });

        // Transform tool_choice if provided
        let tool_choice = claude_request.tool_choice.map(|choice| match choice {
            ClaudeMessagesToolChoice::Auto => OpenAIToolChoice::String("auto".to_string()),
            ClaudeMessagesToolChoice::Any => OpenAIToolChoice::String("any".to_string()),
            ClaudeMessagesToolChoice::Tool { name, .. } => OpenAIToolChoice::Object {
                choice_type: "function".to_string(),
                function: OpenAIFunctionChoice { name },
            },
        });

        Ok(OpenAIRequest {
            model: "qwen3-coder-flash".to_string(),
            messages,
            max_tokens: claude_request.max_tokens,
            temperature: claude_request.temperature,
            top_p: claude_request.top_p,
            stream: claude_request.stream,
            tools,
            tool_choice,
            response_format: None,
            stream_options: Some(crate::models::openai::StreamOptions {
                include_usage: Some(true),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::models::claude_messages::{Message, SystemContentEnum};
    use super::*;


    #[test]
    fn test_adapt_claude_messages_to_qwen() {
        let adapter = ClaudeMessagesToOpenAIAdapter;
        let claude_request = ClaudeMessagesRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello, world!".to_string(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
            system: Some(SystemContentEnum::String("You are a helpful assistant".to_string())),
            tools: None,
            tool_choice: None,
            metadata: None,
            stop_sequences: None,
            top_k: None,
            top_p: None,
            container: None,
        };

        let openai_request = adapter.adapt(claude_request).unwrap();

        assert_eq!(openai_request.model, "qwen3-coder");
        assert_eq!(openai_request.messages.len(), 2);
        assert_eq!(openai_request.messages[0].role, OpenAIRole::System);
        assert_eq!(openai_request.messages[1].role, OpenAIRole::User);
        assert_eq!(openai_request.max_tokens, Some(100));
        assert_eq!(openai_request.temperature, Some(0.7));
        assert_eq!(openai_request.stream, Some(false));
    }

    #[test]
    fn test_adapt_claude_messages_to_qwen_with_streaming() {
        let adapter = ClaudeMessagesToOpenAIAdapter;
        let claude_request = ClaudeMessagesRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello, world!".to_string(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(true),
            system: None,
            tools: None,
            tool_choice: None,
            metadata: None,
            stop_sequences: None,
            top_k: None,
            top_p: None,
            container: None,
        };

        let openai_request = adapter.adapt(claude_request).unwrap();

        assert_eq!(openai_request.model, "qwen3-coder");
        assert_eq!(openai_request.messages.len(), 1);
        assert_eq!(openai_request.messages[0].role, OpenAIRole::User);
        assert_eq!(openai_request.max_tokens, Some(100));
        assert_eq!(openai_request.temperature, Some(0.7));
        assert_eq!(openai_request.stream, Some(true));
    }
}
