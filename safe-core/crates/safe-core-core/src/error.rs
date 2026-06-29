use thiserror::Error;
#[derive(Debug, Error, Clone)]
pub enum CoreError {
    #[error("Invalid input: {0}")] InvalidInput(String),
    #[error("Unauthorized: {0}")] Unauthorized(String),
    #[error("Internal error: {0}")] Internal(String),
}
pub type CoreResult<T> = Result<T, CoreError>;
