use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fs;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid macaroon: {0}")]
    InvalidMacaroon(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Macaroon de autenticação para tapd.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Macaroon {
    bytes: Vec<u8>,
    // Permissões escopadas (read, write, courier)
    permissions: Vec<String>,
}

impl Macaroon {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, AuthError> {
        if bytes.is_empty() {
            return Err(AuthError::InvalidMacaroon("Empty macaroon".to_string()));
        }
        // Parse das permissões (simplificado)
        let permissions = vec!["read".to_string(), "write".to_string()];
        Ok(Self { bytes, permissions })
    }

    pub fn from_file(path: &str) -> Result<Self, AuthError> {
        let bytes = fs::read(path)?;
        Self::from_bytes(bytes)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }
}
