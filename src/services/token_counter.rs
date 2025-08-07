use crate::models::claude_count_tokens::{ClaudeCountTokensRequest, ClaudeCountTokensResponse, SystemPrompt};
use crate::models::claude_messages::{ContentBlock, Message};

pub struct TokenCounter;

impl TokenCounter {
    /// Approximate token count based on word count and character count
    /// This is a simplified implementation for local token counting
    pub fn count_tokens(request: ClaudeCountTokensRequest) -> ClaudeCountTokensResponse {
        let mut total_tokens = 0;

        // Count tokens for messages
        if let Some(messages) = request.messages {
            for message in messages {
                for content_block in message.content {
                    total_tokens += Self::count_content_block_tokens(&content_block);
                }
            }
        }

        // Count tokens for system prompts
        if let Some(system) = request.system {
            for prompt in system {
                total_tokens += Self::estimate_tokens(&prompt.text);
            }
        }

        // Count tokens for tools
        if let Some(tools) = request.tools {
            for tool in tools {
                total_tokens += Self::estimate_tokens(&tool.name);
                total_tokens += Self::estimate_tokens(&tool.description);
                // Rough estimate for JSON schema
                total_tokens += 50; // Approximate for JSON schema
            }
        }

        ClaudeCountTokensResponse {
            input_tokens: total_tokens,
        }
    }

    fn count_content_block_tokens(content_block: &ContentBlock) -> u32 {
        match content_block {
            ContentBlock::Text { text, .. } => Self::estimate_tokens(text),
            ContentBlock::ToolUse { id, name, input, .. } => {
                let mut tokens = 0;
                tokens += Self::estimate_tokens(id);
                tokens += Self::estimate_tokens(name);
                tokens += Self::estimate_tokens(&input.to_string());
                tokens
            }
            ContentBlock::ToolResult { tool_use_id, content, .. } => {
                let mut tokens = 0;
                tokens += Self::estimate_tokens(tool_use_id);
                tokens += Self::estimate_tokens(content);
                tokens
            }
        }
    }

    fn estimate_tokens(text: &str) -> u32 {
        // Rough approximation: 1 token ≈ 4 characters on average
        // This is a simplification - real tokenization is more complex
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        
        // Use average of character-based and word-based estimates
        let char_estimate = (char_count as f32) / 4.0;
        let word_estimate = (word_count as f32) * 1.3; // Average 1.3 tokens per word
        
        ((char_estimate + word_estimate) / 2.0).ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::claude_messages::{ContentBlock, Message, MessageRole};
    use crate::models::claude_count_tokens::ClaudeCountTokensRequest;

    #[test]
    fn test_count_simple_text() {
        let request = ClaudeCountTokensRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: Some(vec![Message {
                role: MessageRole::User,
                content: vec![ContentBlock::text("Hello, world!".to_string())],
            }]),
            system: None,
            tools: None,
        };

        let response = TokenCounter::count_tokens(request);
        assert!(response.input_tokens > 0);
        assert!(response.input_tokens < 50); // Should be reasonable
    }

    #[test]
    fn test_count_with_system_prompt() {
        let request = ClaudeCountTokensRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: Some(vec![Message {
                role: MessageRole::User,
                content: vec![ContentBlock::text("Hello".to_string())],
            }]),
            system: Some(vec![SystemPrompt {
                prompt_type: "text".to_string(),
                cache_control: None,
                text: "You are a helpful assistant.".to_string(),
            }]),
            tools: None,
        };

        let response = TokenCounter::count_tokens(request);
        assert!(response.input_tokens > 0);
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(TokenCounter::estimate_tokens(""), 0);
        assert!(TokenCounter::estimate_tokens("hello world") > 0);
        assert!(TokenCounter::estimate_tokens("A longer sentence with more words") > 2);
    }
}