use std::sync::Arc;
use ethers::types::Address;
use sha2::{Sha256, Digest};

use crate::substrato_4004::deps::{Action, EventStore, CrossChainEmitterV2, OrchestratorEvent};

#[derive(Debug, Clone)]
pub enum TracerError {
    Store(String),
    CrossChain(String),
}

impl std::fmt::Display for TracerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracerError::Store(s) => write!(f, "Store error: {}", s),
            TracerError::CrossChain(s) => write!(f, "Cross chain error: {}", s),
        }
    }
}

pub struct MemoTracer {
    pub event_store: Arc<EventStore>,
    pub cross_chain_emitter: Arc<CrossChainEmitterV2>,
}

impl MemoTracer {
    pub fn generate_memo(&self, action: &Action) -> [u8; 32] {
        let action_hash = Sha256::digest(action.canonical_bytes());
        let mut memo = [0u8; 32];
        memo.copy_from_slice(&action_hash[..32]);
        memo
    }

    pub async fn index_memo_event(
        &self,
        tx_hash: &str,
        log_index: u64,
        caller: Address,
        memo: [u8; 32],
    ) -> Result<(), TracerError> {
        let event = OrchestratorEvent::B20Memo {
            tx_hash: tx_hash.to_string(),
            log_index,
            caller: format!("{:?}", caller),
            memo: hex::encode(memo),
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.event_store.store(event.clone()).await.map_err(TracerError::Store)?;
        self.cross_chain_emitter.emit_cross_chain(event).await.map_err(TracerError::CrossChain)?;

        Ok(())
    }

    pub async fn resolve_memo(&self, memo: [u8; 32]) -> Result<Option<Action>, TracerError> {
        let events = self.event_store
            .query_by_memo(&hex::encode(memo))
            .await
            .map_err(TracerError::Store)?;

        if let Some(event) = events.first() {
            if let OrchestratorEvent::ActionProposed { action, .. } = event {
                return Ok(Some(action.clone()));
            }
        }

        Ok(None)
    }
}
