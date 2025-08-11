use crate::adapters::Adaptor;
use crate::models::claude_messages::{
    ClaudeMessagesRequest, InputMessageContent, InputMessageContentPart, MessageRole, SystemMessage,
    ToolChoice as ClaudeMessagesToolChoice,
};
use crate::models::openai::{
    OpenAIFunctionChoice, OpenAIFunctionDefinition, OpenAIMessage, OpenAIRequest, OpenAIRole,
    OpenAIToolChoice, OpenAIToolDefinition,
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
        if let Some(system_msges) = claude_request.system {
            for system_msg in system_msges {
                let system_text = match system_msg {
                    SystemMessage { text, .. } => text,
                };
                messages.push(OpenAIMessage {
                    role: OpenAIRole::System,
                    content: Some(system_text),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
        }

        // Convert Claude messages to OpenAI messages
        for claude_msg in claude_request.messages {
            let openai_role = match claude_msg.role {
                MessageRole::User => OpenAIRole::User,
                MessageRole::Assistant => OpenAIRole::Assistant,
            };

            let content_text = match claude_msg.content {
                InputMessageContent::Text(text) => text,
                InputMessageContent::Parts(parts) => {
                    // For now, extract text content from parts
                    parts
                        .into_iter()
                        .filter_map(|part| match part {
                            InputMessageContentPart::Text { text, .. } => Some(text),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };

            messages.push(OpenAIMessage {
                role: openai_role,
                content: Some(content_text),
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
            ClaudeMessagesToolChoice::Auto { .. } => OpenAIToolChoice::String("auto".to_string()),
            ClaudeMessagesToolChoice::Any { .. } => OpenAIToolChoice::String("any".to_string()),
            ClaudeMessagesToolChoice::Tool { name, .. } => OpenAIToolChoice::Object {
                choice_type: "function".to_string(),
                function: OpenAIFunctionChoice { name },
            },
            ClaudeMessagesToolChoice::None => OpenAIToolChoice::String("none".to_string()),
        });

        Ok(OpenAIRequest {
            model: claude_request.model,
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
    use super::*;
    use crate::models::claude_messages::{
        ClaudeMessagesRequest, InputMessage, MessageRole, ToolChoice, ToolDefinition,
    };

    #[test]
    fn test_adapt_claude_messages_to_qwen() {
        let adapter = ClaudeMessagesToOpenAIAdapter;
        let claude_request = ClaudeMessagesRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![InputMessage {
                role: MessageRole::User,
                content: InputMessageContent::Text("Hello, world!".to_string()),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(1.0),
            stream: Some(false),
            system: Some(vec![SystemMessage {
                content_type: "text".to_string(),
                text: "You are a helpful assistant".to_string(),
                cache_control: None,
                citations: None,
            }]),
            tools: None,
            tool_choice: None,
            container: None,
            metadata: None,
            service_tier: None,
            stop_sequences: None,
            thinking: None,
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
            messages: vec![InputMessage {
                role: MessageRole::User,
                content: InputMessageContent::Text("Hello, world!".to_string()),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: Some(1.0),
            stream: Some(true),
            system: None,
            tools: None,
            tool_choice: None,
            container: None,
            metadata: None,
            service_tier: None,
            stop_sequences: None,
            thinking: None,
        };

        let openai_request = adapter.adapt(claude_request).unwrap();

        assert_eq!(openai_request.model, "qwen3-coder");
        assert_eq!(openai_request.messages.len(), 1);
        assert_eq!(openai_request.messages[0].role, OpenAIRole::User);
        assert_eq!(openai_request.max_tokens, Some(100));
        assert_eq!(openai_request.temperature, Some(0.7));
        assert_eq!(openai_request.stream, Some(true));
    }

    #[test]
    fn test_adapt_claude_messages_with_system_and_tools() {
        let adapter = ClaudeMessagesToOpenAIAdapter;
        let claude_request = ClaudeMessagesRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![InputMessage {
                role: MessageRole::User,
                content: InputMessageContent::Text("What's the weather?".to_string()),
            }],
            max_tokens: Some(200),
            temperature: Some(0.5),
            top_p: Some(1.0),
            stream: Some(false),
            system: Some(vec![SystemMessage {
                content_type: "text".to_string(),
                text: "You are a weather assistant".to_string(),
                cache_control: None,
                citations: None,
            }]),
            tools: Some(vec![ToolDefinition {
                tool_type: None,
                name: "get_weather".to_string(),
                description: "Get weather information".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                }),
                cache_control: None,
                other_values: None,
            }]),
            tool_choice: Some(ToolChoice::Auto {
                disable_parallel_tool_use: false,
            }),
            container: None,
            metadata: None,
            service_tier: None,
            stop_sequences: None,
            thinking: None,
        };

        let openai_request = adapter.adapt(claude_request).unwrap();

        assert_eq!(openai_request.model, "qwen3-coder");
        assert_eq!(openai_request.messages.len(), 2);
        assert_eq!(openai_request.messages[0].role, OpenAIRole::System);
        assert_eq!(
            openai_request.messages[0].content,
            Some("You are a weather assistant".to_string())
        );
        assert_eq!(openai_request.messages[1].role, OpenAIRole::User);
        assert_eq!(
            openai_request.messages[1].content,
            Some("What's the weather?".to_string())
        );
        assert_eq!(openai_request.max_tokens, Some(200));
        assert_eq!(openai_request.temperature, Some(0.5));
        assert!(openai_request.tools.is_some());
        assert_eq!(openai_request.tools.as_ref().unwrap().len(), 1);
        assert_eq!(
            openai_request.tools.as_ref().unwrap()[0].function.name,
            "get_weather"
        );
    }

    #[test]
    fn test_adapt_claude_messages_no_system() {
        let adapter = ClaudeMessagesToOpenAIAdapter;
        let claude_request = ClaudeMessagesRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![
                InputMessage {
                    role: MessageRole::User,
                    content: InputMessageContent::Text("Hi".to_string()),
                },
                InputMessage {
                    role: MessageRole::Assistant,
                    content: InputMessageContent::Text("Hello!".to_string()),
                },
            ],
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: None,
            system: None,
            tools: None,
            tool_choice: None,
            container: None,
            metadata: None,
            service_tier: None,
            stop_sequences: None,
            thinking: None,
        };

        let openai_request = adapter.adapt(claude_request).unwrap();

        assert_eq!(openai_request.model, "qwen3-coder");
        assert_eq!(openai_request.messages.len(), 2);
        assert_eq!(openai_request.messages[0].role, OpenAIRole::User);
        assert_eq!(openai_request.messages[0].content, Some("Hi".to_string()));
        assert_eq!(openai_request.messages[1].role, OpenAIRole::Assistant);
        assert_eq!(
            openai_request.messages[1].content,
            Some("Hello!".to_string())
        );
    }
}
