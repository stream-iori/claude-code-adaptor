pub mod adapters;
pub mod config;
pub mod models;
pub mod services;
pub mod cli;

#[cfg(test)]
mod tests {
    use crate::adapters::request_adapter::ClaudeMessagesToOpenAIAdapter;
    use crate::adapters::Adaptor;
    use crate::models::{
        claude_messages::{ClaudeMessagesRequest, Message, MessageRole, SystemContentEnum},
        openai::OpenAIRole,
    };

    #[test]
    fn test_full_integration() {
        let adapter = ClaudeMessagesToOpenAIAdapter;
        
        let claude_request = ClaudeMessagesRequest {
            model: "qwen3-coder".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Write a Rust function".to_string(),
                },
                Message {
                    role: MessageRole::Assistant,
                    content: "I'll write a simple Rust function for you.".to_string(),
                },
            ],
            max_tokens: Some(500),
            temperature: Some(0.8),
            stream: Some(false),
            system: Some(SystemContentEnum::String("You are a Rust expert".to_string())),
            tools: None,
            tool_choice: None,
            metadata: None,
            stop_sequences: None,
            top_k: None,
            top_p: Some(0.9),
            container: None,
        };

        let openai_request = adapter.adapt(claude_request).unwrap();
        
        // Verify model
        assert_eq!(openai_request.model, "qwen3-coder");
        
        // Verify messages structure and content
        assert_eq!(openai_request.messages.len(), 3); // system + user + assistant
        
        // Verify system message
        assert_eq!(openai_request.messages[0].role, OpenAIRole::System);
        assert_eq!(openai_request.messages[0].content, Some("You are a Rust expert".to_string()));
        assert!(openai_request.messages[0].tool_calls.is_none());
        assert!(openai_request.messages[0].tool_call_id.is_none());
        
        // Verify user message
        assert_eq!(openai_request.messages[1].role, OpenAIRole::User);
        assert_eq!(openai_request.messages[1].content, Some("Write a Rust function".to_string()));
        
        // Verify assistant message
        assert_eq!(openai_request.messages[2].role, OpenAIRole::Assistant);
        assert_eq!(openai_request.messages[2].content, Some("I'll write a simple Rust function for you.".to_string()));
        
        // Verify optional parameters
        assert_eq!(openai_request.max_tokens, Some(500));
        assert_eq!(openai_request.temperature, Some(0.8));
        assert_eq!(openai_request.stream, Some(false));
        assert_eq!(openai_request.top_p, Some(0.9));
        
        // Verify tools and tool_choice are None
        assert!(openai_request.tools.is_none());
        assert!(openai_request.tool_choice.is_none());
        
        // Verify stream options
        assert!(openai_request.stream_options.is_some());
        assert_eq!(openai_request.stream_options.unwrap().include_usage, Some(true));
    }
}