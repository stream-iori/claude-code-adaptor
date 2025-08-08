use crate::models::error::AdaptorError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub qwen: QwenConfig,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            qwen: QwenConfig {
                api_key: "test".to_string(),
                base_url: "https://dashscope.aliyuncs.com/api/v1".to_string(),
                model: "qwen3-coder".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(config_path: &str) -> Result<Self, AdaptorError> {
        use std::fs;
        use std::path::Path;

        if !Path::new(config_path).exists() {
            let config = Config::default();
            return Ok(config);
        }

        let config_content = fs::read_to_string(config_path).map_err(|e| {
            AdaptorError::Configuration(format!("Failed to read config file: {}", e))
        })?;

        let mut config: Config = serde_json::from_str(&config_content).map_err(|e| {
            AdaptorError::Configuration(format!("Failed to parse config JSON: {}", e))
        })?;

        if config.qwen.api_key.is_empty() {
            return Err(AdaptorError::Configuration("API_KEY is empty".to_string()));
        }

        if config.qwen.api_key.is_empty() {
            return Err(AdaptorError::Configuration(
                "API_KEY is required".to_string(),
            ));
        }
        
        Ok(config)
    }
}
