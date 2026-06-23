//! Erros unificados para o Relay Adapter
//! Selo: CATHEDRAL-RELAY-ERROR-v2.0.0-2026-06-22

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("Erro de RPC: {0}")]
    Rpc(String),

    #[error("Erro no contrato: {0}")]
    Contract(String),

    #[error("Erro na transação: {0}")]
    Transaction(String),

    #[error("Erro no indexador: {0}")]
    Indexer(String),

    #[error("Erro de rede: {0}")]
    Network(String),

    #[error("Erro de signer: {0}")]
    Signer(String),

    #[error("Erro de serialização: {0}")]
    Serialization(String),
}
