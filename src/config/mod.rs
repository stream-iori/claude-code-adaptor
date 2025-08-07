use serde::{Deserialize, Serialize};
use std::env;
use crate::models::error::AdaptorError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub qwen: QwenConfig,
    pub claude: ClaudeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            qwen: QwenConfig {
                api_key: env::var("QWEN_API_KEY").unwrap_or_default(),
                base_url: "https://dashscope.aliyuncs.com/api/v1".to_string(),
                model: "qwen3-coder".to_string(),
            },
            claude: ClaudeConfig {
                api_key: env::var("CLAUDE_API_KEY").unwrap_or_default(),
                base_url: "https://api.anthropic.com".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> std::result::Result<Self, AdaptorError> {
        dotenv::dotenv().ok();
        
        let config = Config::default();

        if config.qwen.api_key.is_empty() {
            return Err(AdaptorError::Configuration(
                "QWEN_API_KEY environment variable is required".to_string(),
            ));
        }
        
        Ok(config)
    }
}