use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdaptorError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    
    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid configuration: {0}")]
    Configuration(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
}