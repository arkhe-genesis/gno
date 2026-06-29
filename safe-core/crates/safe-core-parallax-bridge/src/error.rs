use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ParallaxError {
    #[error("gRPC error: {0}")]
    Grpc(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Node unavailable")]
    NodeUnavailable,
    #[error("Not supported: {0}")]
    NotSupported(String),
}

impl From<tonic::Status> for ParallaxError {
    fn from(status: tonic::Status) -> Self {
        ParallaxError::Grpc(status.message().to_string())
    }
}

impl From<serde_json::Error> for ParallaxError {
    fn from(err: serde_json::Error) -> Self {
        ParallaxError::Serialization(err.to_string())
    }
}
