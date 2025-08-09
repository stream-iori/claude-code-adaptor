use crate::adapters::Adaptor;
use crate::models::claude_messages::{ClaudeMessagesResponse, ResponseContentBlock, Usage};
use crate::models::openai::OpenAIResponse;
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct OpenAIToClaudeMessagesAdapter;

impl Adaptor for OpenAIToClaudeMessagesAdapter {
    type From = OpenAIResponse;
    type To = ClaudeMessagesResponse;

    fn before_adapt(&self, from: &Self::From) {
        tracing::debug!("Before OpenAI response adaptation: {:?}", from);
    }

    fn after_adapt(&self, to: Result<&Self::To, &anyhow::Error>) {
        match to {
            Ok(result) => tracing::debug!("After OpenAI response adaptation: {:?}", result),
            Err(e) => tracing::debug!("OpenAI response adaptation failed {:?}", e),
        }
    }

    fn do_adapt(&self, openai_response: Self::From) -> Result<Self::To> {
        let choice = &openai_response.choices[0];
        let mut content = Vec::new();

        // Add text content if present
        if let Some(text) = &choice.message.content {
            content.push(ResponseContentBlock::Text {
                text: text.clone(),
            });
        }

        // Add tool calls if present
        if let Some(tool_calls) = &choice.message.tool_calls {
            for call in tool_calls {
                content.push(ResponseContentBlock::ToolUse {
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    input: serde_json::from_str(&call.function.arguments)
                        .unwrap_or(serde_json::Value::Null),
                });
            }
        }

        let usage = openai_response.usage.unwrap_or_default();

        Ok(ClaudeMessagesResponse {
            id: openai_response.id,
            response_type: "message".to_string(),
            role: choice.message.role.to_string().to_lowercase(),
            content,
            model: openai_response.model,
            stop_reason: choice.finish_reason.clone(),
            stop_sequence: None,
            usage: Usage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            container: None,
        })
    }
}