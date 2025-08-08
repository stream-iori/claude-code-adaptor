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

impl Config {
    pub fn load(config_path: &str) -> Result<Self, AdaptorError> {
        use std::fs;
        use std::path::Path;

        if !Path::new(config_path).exists() {
            return Err(AdaptorError::Configuration(
                "config file not exist.".to_string(),
            ));
        }

        let config_content = fs::read_to_string(config_path).map_err(|e| {
            AdaptorError::Configuration(format!("Failed to read config file: {}", e))
        })?;

        let config: Config = serde_json::from_str(&config_content).map_err(|e| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config() {
        let config = Config::load("custom_config.json").unwrap();
        assert_eq!(config.server.host, "0.0.0.0");

        let result = Config::load("unknow.json");
        assert!(matches!(result.unwrap_err(), AdaptorError::Configuration(_)));
    }
}