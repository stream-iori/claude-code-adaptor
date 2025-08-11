use crate::adapters::Adaptor;
use crate::models::claude_messages::{ClaudeMessagesResponse, Usage};
use crate::models::openai::OpenAIResponse;
use anyhow::Result;

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
        let content = choice.message.content.clone().unwrap_or_default();

        let usage = openai_response.usage.unwrap_or_default();

        Ok(ClaudeMessagesResponse {
            id: openai_response.id,
            content: vec![],
            role: choice.message.role.to_string().to_lowercase(),
            model: openai_response.model,
            stop_reason: choice.finish_reason.clone(),
            stop_sequence: None,
            usage: Usage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
            response_type: "".to_string(),
            container: None,
        })
    }
}