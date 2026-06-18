use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub action_type: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

impl Action {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerViolation {
    pub layer: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterVerdict {
    Passed,
    Failed(Vec<LayerViolation>),
}

pub struct EthicalFilter;
impl EthicalFilter {
    pub async fn evaluate(&self, _action: &Action) -> FilterVerdict {
        FilterVerdict::Passed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorEvent {
    ComplianceChecked { action_id: String, verdict: crate::substrato_4004::compliance_engine::ComplianceVerdict, timestamp: i64 },
    B20BatchSettled { batch_id: String, receipt: crate::substrato_4004::settlement_engine::SettlementReceipt, timestamp: i64 },
    B20Memo { tx_hash: String, log_index: u64, caller: String, memo: String, timestamp: i64 },
    ActionProposed { action: Action, timestamp: i64 },
    B20ToXrplBridge { b20_tx_hash: String, xrpl_escrow_id: String, amount: String, token: String, timestamp: i64 },
    XrplToB20Release { xrpl_escrow_id: String, b20_tx_hash: String, recipient: String, timestamp: i64 },
}

pub struct EventStore {
    events: tokio::sync::Mutex<Vec<OrchestratorEvent>>,
}
impl EventStore {
    pub fn new() -> Self {
        Self { events: tokio::sync::Mutex::new(Vec::new()) }
    }
    pub async fn emit(&self, event: OrchestratorEvent) -> Result<(), String> {
        self.events.lock().await.push(event);
        Ok(())
    }
    pub async fn store(&self, event: OrchestratorEvent) -> Result<(), String> {
        self.events.lock().await.push(event);
        Ok(())
    }
    pub async fn query_by_memo(&self, _memo_hex: &str) -> Result<Vec<OrchestratorEvent>, String> {
        Ok(vec![])
    }
}

pub struct CrossChainEmitterV2;
impl CrossChainEmitterV2 {
    pub async fn emit_cross_chain(&self, _event: OrchestratorEvent) -> Result<(), String> {
        Ok(())
    }
}

pub struct HybridZkVerifier;
impl HybridZkVerifier {
    pub async fn prove_settlement(&self, _tx_hashes: &[String]) -> Result<String, String> {
        Ok("mock_proof".to_string())
    }
}

pub struct BatchSettlementEngine;

#[derive(Debug, Clone)]
pub struct B20Payment {
    pub id: String,
    pub token: Address,
    pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub memo: Option<[u8; 32]>,
}

impl B20Payment {
    pub fn to_action(&self) -> Action {
        Action {
            id: self.id.clone(),
            action_type: "payment_b20".to_string(),
            payload: serde_json::json!({
                "token": format!("{:?}", self.token),
                "from": format!("{:?}", self.from),
                "to": format!("{:?}", self.to),
                "amount": self.amount.to_string(),
            }),
            metadata: HashMap::new(),
        }
    }
    pub fn to_x402_payment(&self) -> String {
        "x402_payment_data".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct B20PaymentBatch {
    pub id: String,
    pub payments: Vec<B20Payment>,
}

pub struct X402XrplBridge {
    pub escrow_manager: EscrowManager,
}
impl X402XrplBridge {
    pub async fn create_settlement_escrow(&self, _payment: &str) -> Result<String, String> {
        Ok("mock_escrow_id".to_string())
    }
}

pub struct EscrowManager;
impl EscrowManager {
    pub async fn get_state(&self, _id: &str) -> Result<EscrowState, String> {
        Ok(EscrowState {
            released: true,
            token: Address::zero(),
            amount: U256::zero(),
        })
    }
}

pub struct EscrowState {
    pub released: bool,
    pub token: Address,
    pub amount: U256,
}

pub struct IB20;
impl IB20 {
    pub fn new(_token: Address, _provider: std::sync::Arc<ethers::providers::Provider<ethers::providers::Http>>) -> Self {
        Self
    }
}

pub struct B20Constants;
impl B20Constants {
    pub const MINT_ROLE: [u8; 32] = [1; 32];
    pub const BURN_ROLE: [u8; 32] = [2; 32];
    pub const BURN_BLOCKED_ROLE: [u8; 32] = [3; 32];
    pub const PAUSE_ROLE: [u8; 32] = [4; 32];
    pub const UNPAUSE_ROLE: [u8; 32] = [5; 32];
    pub const OPERATOR_ROLE: [u8; 32] = [6; 32];
}
