//! Tipos unificados usando alloy::primitives
//! Selo: CATHEDRAL-RELAY-TYPES-v2.0.0-2026-06-22

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

pub type ChainId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetAddress(pub Address);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub address: Address,
    pub asset: AssetAddress,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: U256,
    pub total_assets: U256,
    pub apy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub total_assets: U256,
    pub yield_earned: U256,
    pub pending_rewards: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub amount: U256,
    pub recipient: Address,
    pub l1_asset: AssetAddress,
    pub target_chain: ChainId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub tx_hash: String,
    pub amount_out: U256,
    pub fee: U256,
}
