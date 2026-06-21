use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Bucket not found: {0}")]
    BucketNotFound(String),
    #[error("Rate limited by Plurality, retry after {retry_after:?}")]
    RateLimited { retry_after: Option<chrono::Duration> },
    #[error("Consent revoked for bucket {bucket}")]
    ConsentRevoked { bucket: String },
    #[error("Invalid response from Plurality")]
    InvalidResponse,
    #[error("Invalid JWT: {0}")]
    JwtError(String),
    #[error("Validation error for field {field}: {reason}")]
    Validation { field: String, reason: String },
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Missing configuration: {0}")]
    MissingConfig(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub type Result<T> = std::result::Result<T, McpError>;
