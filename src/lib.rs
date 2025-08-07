pub mod adapters;
pub mod config;
pub mod models;
pub mod services;
pub mod cli;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        claude::{ClaudeRequest, ClaudeMessage, ClaudeRole},
        qwen3::{QwenRequest, QwenMessage, QwenRole},
    };
    use crate::adapters::request_adapter::{RequestAdapter, ClaudeToQwenAdapter};

    #[test]
    fn test_full_integration() {
        let adapter = ClaudeToQwenAdapter;
        
        let claude_request = ClaudeRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![
                ClaudeMessage {
                    role: ClaudeRole::User,
                    content: "Write a Rust function".to_string(),
                },
            ],
            max_tokens: Some(500),
            temperature: Some(0.8),
            stream: Some(false),
            system: Some("You are a Rust expert".to_string()),
            tools: None,
            tool_choice: None,
        };

        let qwen_request = adapter.adapt(claude_request).unwrap();
        
        assert_eq!(qwen_request.model, "qwen3-coder");
        assert_eq!(qwen_request.input.messages.len(), 2);
        assert_eq!(qwen_request.input.messages[0].role, QwenRole::System);
        assert_eq!(qwen_request.input.messages[1].role, QwenRole::User);
        assert_eq!(qwen_request.parameters.max_tokens, Some(500));
        assert_eq!(qwen_request.parameters.temperature, Some(0.8));
    }
}