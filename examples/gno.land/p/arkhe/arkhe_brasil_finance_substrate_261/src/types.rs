//! Tipos fundamentais
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictKey {
    pub key_type: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixTransaction {
    pub end_to_end_id: String,
    pub amount: f64,
    pub receiver_key: DictKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrOrder {
    pub ispb_debited: String,
    pub ispb_credited: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingTitle {
    pub isin: String,
    pub nominal_value: f64,
    pub registry_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMessage {
    pub payload: String,
    pub signature: String,
}
