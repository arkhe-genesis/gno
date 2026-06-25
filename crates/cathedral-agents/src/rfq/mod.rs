use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod pricing;
pub mod order_book;
pub mod handler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfqRequest {
    pub id: String,
    pub asset_ref: String,
    pub amount: u64,
    pub side: OrderSide, // Buy ou Sell
    pub requested_price: Option<f64>,
    pub expiry: DateTime<Utc>,
    pub peer_did: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfqResponse {
    pub request_id: String,
    pub price: f64,
    pub max_fill: u64,
    pub expiry: DateTime<Utc>,
    pub quote_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPolicy {
    /// Fill or Kill: preenche tudo ou cancela tudo
    FOK,
    /// Immediate or Cancel: preenche o que for possível, cancela o resto
    IOC,
    /// Good 'Til Cancelled: ordem ativa até cancelamento
    GTC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub asset_ref: String,
    pub side: OrderSide,
    pub amount: u64,
    pub filled: u64,
    pub price: f64,
    pub policy: ExecutionPolicy,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Expired,
}

/// Resultado da execução de uma ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub order_id: String,
    pub filled_amount: u64,
    pub price: f64,
    pub status: OrderStatus,
    pub tx_id: Option<String>, // ID da transação Lightning
    pub timestamp: DateTime<Utc>,
}
