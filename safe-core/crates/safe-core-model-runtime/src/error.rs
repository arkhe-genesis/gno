use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    #[error("Policy denied: {0}")]
    Policy(String),
    #[error("Backend error: {0}")]
    Backend(String),
    #[error("Model not found: {0}")]
    NotFound(String),
    #[error("Model not ready")]
    NotReady,
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("Not supported: {0}")]
    NotSupported(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<safe_core_parallax_bridge::ParallaxError> for RuntimeError {
    fn from(err: safe_core_parallax_bridge::ParallaxError) -> Self {
        match err {
            safe_core_parallax_bridge::ParallaxError::ModelNotFound(msg) => {
                RuntimeError::NotFound(msg)
            }
            safe_core_parallax_bridge::ParallaxError::NotSupported(msg) => {
                RuntimeError::NotSupported(msg)
            }
            other => RuntimeError::Backend(other.to_string()),
        }
    }
}
