use crate::adapters::Adaptor;
use crate::models::claude_messages::{ClaudeMessagesStreamResponse, Delta, StreamMessage, Usage, StreamContentBlock};
use crate::models::openai::OpenAIStreamResponse;
use anyhow::Result;
use serde_json::Value;

#[derive(Clone)]
pub struct OpenAIStreamToClaudeStreamAdapter;

impl Adaptor for OpenAIStreamToClaudeStreamAdapter {
    type From = OpenAIStreamResponse;
    type To = ClaudeMessagesStreamResponse;

    fn before_adapt(&self, from: &Self::From) {
        tracing::debug!("Before OpenAI stream response adaptation: {:?}", from);
    }

    fn after_adapt(&self, to: Result<&Self::To, &anyhow::Error>) {
        match to {
            Ok(result) => tracing::debug!("After OpenAI stream response adaptation: {:?}", result),
            Err(e) => tracing::debug!("OpenAI stream response adaptation failed {:?}", e),
        }
    }

    fn do_adapt(&self, openai_stream_response: Self::From) -> Result<Self::To> {
        let choice = &openai_stream_response.choices[0];
        
        // Handle final usage chunk
        if openai_stream_response.choices.is_empty() {
            if let Some(usage) = &openai_stream_response.usage {
                return Ok(ClaudeMessagesStreamResponse {
                    response_type: "message_stop".to_string(),
                    delta: None,
                    message: None,
                    content_block: None,
                    index: None,
                    usage: Some(Usage {
                        input_tokens: usage.prompt_tokens,
                        output_tokens: usage.completion_tokens,
                        cache_creation_input_tokens: None,
                        cache_read_input_tokens: None,
                    }),
                });
            }
        }

        let delta = &choice.delta;
        let mut claude_response = ClaudeMessagesStreamResponse {
            response_type: "content_block_delta".to_string(),
            delta: None,
            message: None,
            content_block: None,
            index: Some(0),
            usage: None,
        };

        // Handle content delta
        if let Some(content) = &delta.content {
            claude_response.content_block = Some(StreamContentBlock {
                content_type: "text".to_string(),
                text: Some(content.clone()),
                id: None,
                name: None,
                input: None,
            });
        }

        // Handle tool calls delta
        if let Some(tool_calls) = &delta.tool_calls {
            for tool_call in tool_calls {
                if let Some(function) = &tool_call.function {
                    claude_response.content_block = Some(StreamContentBlock {
                        content_type: "tool_use".to_string(),
                        text: None,
                        id: tool_call.id.clone(),
                        name: function.name.clone(),
                        input: function.arguments.as_ref().map(|args| 
                            serde_json::from_str(args).unwrap_or(Value::Null)
                        ),
                    });
                }
            }
        }

        // Handle role change (message start)
        if let Some(role) = &delta.role {
            claude_response.response_type = "message_start".to_string();
            claude_response.message = Some(StreamMessage {
                id: openai_stream_response.id.clone(),
                message_type: "message".to_string(),
                role: role.to_string().to_lowercase(),
                content: Vec::new(),
            });
        }

        // Handle finish reason (message stop)
        if let Some(finish_reason) = &choice.finish_reason {
            claude_response.response_type = "message_stop".to_string();
            claude_response.delta = Some(Delta {
                stop_reason: Some(finish_reason.clone()),
                stop_sequence: None,
            });
        }

        Ok(claude_response)
    }
}