use crate::models::{qwen3::*, error::AdaptorError};
type Result<T> = std::result::Result<T, AdaptorError>;
use crate::config::QwenConfig;
use reqwest::{Client, header};

pub trait QwenClient {
    async fn send_request(&self, request: QwenRequest) -> Result<QwenResponse>;
    async fn send_stream_request(&self, request: QwenRequest) -> Result<reqwest::Response>;
}

#[derive(Clone)]
pub struct QwenService {
    client: Client,
    config: QwenConfig,
}

impl QwenService {
    pub fn new(config: QwenConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }
}

impl QwenClient for QwenService {
    async fn send_request(&self, request: QwenRequest) -> Result<QwenResponse> {
        let url = format!("{}/services/aigc/text-generation/generation", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.config.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::models::error::AdaptorError::ApiError(format!(
                "Qwen API request failed: {}", error_text
            )));
        }

        let qwen_response = response.json::<QwenResponse>().await?;
        Ok(qwen_response)
    }

    async fn send_stream_request(&self, request: QwenRequest) -> Result<reqwest::Response> {
        let url = format!("{}/services/aigc/text-generation/generation", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.config.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::models::error::AdaptorError::ApiError(format!(
                "Qwen API stream request failed: {}", error_text
            )));
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::QwenConfig;

    #[tokio::test]
    async fn test_qwen_service_creation() {
        let config = QwenConfig {
            api_key: "test-key".to_string(),
            base_url: "https://test.com".to_string(),
            model: "qwen3-coder".to_string(),
        };
        
        let service = QwenService::new(config);
        
        assert!(!service.config.api_key.is_empty());
        assert_eq!(service.config.base_url, "https://test.com");
    }
}