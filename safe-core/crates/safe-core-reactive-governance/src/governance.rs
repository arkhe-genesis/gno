//! Governance action types and signed entries.

use crypto::{DynSignature, DynPublicKey, verify_dyn_signature};
use serde::{Deserialize, Serialize};

/// Signed governance action that can be applied to the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceAction {
    /// Roll back the UED curriculum to a previous Signed Tree Head.
    RollbackCurriculum {
        target_sth: Vec<u8>,        // serialized SignedTreeHead
        reason: String,
    },
    /// Adjust the reward function of a UED Teacher.
    AdjustTeacherReward {
        teacher_id: String,
        environment_hash: String,   // hash of the problematic environment
        reward_delta: f64,          // negative to penalize
        reason: String,
    },
    /// Ban a specific routing path in the Sparse-Dense router.
    BanRoutingPath {
        router_id: String,
        from_module: String,
        to_module: String,
        reason: String,
    },
    /// Emergency freeze of the entire system.
    EmergencyFreeze {
        reason: String,
        duration_seconds: u64,
    },
    /// Unfreeze the system (signed by a higher authority).
    Unfreeze {
        reason: String,
    },
}

/// A signed governance entry recorded in the reactive log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEntry {
    pub action: GovernanceAction,
    pub issued_by: String,          // e.g., "watchdog", "admin", "dao"
    pub timestamp: i64,
    pub signature: DynSignature,
    pub verifying_key: DynPublicKey,
}

impl GovernanceEntry {
    /// Verify the signature of this governance entry.
    pub fn verify(&self) -> GovernanceResult<()> {
        let payload = serde_json::to_vec(&self.action)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;
        verify_dyn_signature(&self.signature, &self.verifying_key, &payload)
            .map_err(|e| GovernanceError::InvalidSignature(e.to_string()))
    }
}

/// Error types for governance operations.
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Unauthorized issuer: {0}")]
    Unauthorized(String),
    #[error("Governance action not supported: {0}")]
    UnsupportedAction(String),
    #[error("Transparency Log Error: {0}")]
    LogAppend(String),
}

pub type GovernanceResult<T> = Result<T, GovernanceError>;
