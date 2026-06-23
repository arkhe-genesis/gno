//! Cathedral-OS — Integração com Plurality Network
//! Selo: CATHEDRAL-ARKHE-PLURALITY-v1.0.0-2026-06-21

pub mod plurality_client;
pub mod plurality_auth;
pub mod plurality_types;
pub mod memory_adapter;
pub mod smart_profile;

pub use plurality_client::PluralityClient;
pub use plurality_auth::{PluralityAuth, AuthMethod};
pub use plurality_types::*;
pub use memory_adapter::MemoryAdapter;


pub type Result<T> = std::result::Result<T, PluralityError>;

#[derive(Debug, thiserror::Error)]
pub enum PluralityError {
    #[error("Erro de autenticação: {0}")]
    Auth(String),
    #[error("Erro de rede: {0}")]
    Network(String),
    #[error("Erro de serialização: {0}")]
    Serialization(String),
    #[error("Bucket inválido: {0}")]
    InvalidBucket(String),
    #[error("Timeout")]
    Timeout,
    #[error("Item não encontrado: {0}")]
    NotFound(String),
    #[error("Rate limit excedido")]
    RateLimit,
    #[error("Outro erro: {0}")]
    Other(String),
}
