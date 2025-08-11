use crate::adapters::Adaptor;
use crate::models::claude_messages::{ClaudeMessagesStreamResponse, StreamDelta, Usage};
use crate::models::openai::OpenAIStreamResponse;
use anyhow::Result;

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
                    delta: None,
                    usage: Some(Usage {
                        input_tokens: usage.prompt_tokens,
                        output_tokens: usage.completion_tokens,
                        cache_creation_input_tokens: None,
                        cache_read_input_tokens: None,
                    }),
                    other: serde_json::Value::Null,
                });
            }
        }

        let delta = &choice.delta;
        
        // Handle content delta
        if let Some(content) = &delta.content {
            return Ok(ClaudeMessagesStreamResponse {
                delta: Some(StreamDelta {
                    content: content.clone(),
                }),
                usage: None,
                other: serde_json::Value::Null,
            });
        }

        // Default empty response
        Ok(ClaudeMessagesStreamResponse {
            delta: None,
            usage: None,
            other: serde_json::Value::Null,
        })
    }
}