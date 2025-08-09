use crate::config::OpenAIConfig;
use crate::models::openai::*;
use anyhow::{Context, Result};
use reqwest::{Client, header};
use tracing::{error, info, instrument};

pub trait OpenAIClient {
    async fn send_request(&self, request: OpenAIRequest) -> Result<reqwest::Response>;
}

#[derive(Clone)]
pub struct OpenAIService {
    client: Client,
    config: OpenAIConfig,
}

impl OpenAIService {
    pub fn new(config: OpenAIConfig) -> Self {
        let client = Client::new();
        info!(
            "Initializing OpenAI service with base_url: {}",
            config.base_url
        );
        Self { client, config }
    }
}

impl OpenAIClient for OpenAIService {
    #[instrument(skip(self, request), fields(model = %request.model, stream = %request.stream.unwrap_or(false)))]
    async fn send_request(&self, request: OpenAIRequest) -> Result<reqwest::Response> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);

        info!(
            "Sending request to OpenAI API with model: {}",
            request.model
        );

        let response = self
            .client
            .post(&url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.config.api_key),
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI API request failed with status: {}", status);
            return Err(anyhow::anyhow!("OpenAI API request failed: {}", error_text));
        }

        info!("OpenAI API request successful");
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OpenAIConfig;

    #[tokio::test]
    async fn test_qwen_service_creation() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            base_url: "https://test.com".to_string(),
            model: "qwen3-coder".to_string(),
        };

        let service = OpenAIService::new(config);

        assert!(!service.config.api_key.is_empty());
        assert_eq!(service.config.base_url, "https://test.com");
    }
}
