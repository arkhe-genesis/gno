//! Reactive log that interprets signed governance entries.

// use crate::crypto::Hasher;
// use crate::merkle::MerkleTree;
// use crate::transparency::TransparencyLog;
use crate::governance::{GovernanceAction, GovernanceEntry, GovernanceResult, GovernanceError};
use crate::transparency_log::TransparencyLog;
use crypto::DynPublicKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Active governance state maintained by the reactive log.
#[derive(Debug, Default)]
pub struct GovernanceState {
    pub frozen: bool,
    pub banned_routes: HashMap<String, Vec<String>>, // router_id -> ["from_module->to_module"]
    pub reward_adjustments: HashMap<String, f64>,    // teacher_id -> delta
    pub last_rollback_sth: Option<Vec<u8>>,
}

/// Reactive log that applies governance actions.
pub struct ReactiveLog {
    inner: TransparencyLog,
    state: Arc<RwLock<GovernanceState>>,
    authorized_keys: Vec<DynPublicKey>,
}

impl ReactiveLog {
    pub fn new(
        log: TransparencyLog,
        authorized_keys: Vec<DynPublicKey>,
    ) -> Self {
        Self {
            inner: log,
            state: Arc::new(RwLock::new(GovernanceState::default())),
            authorized_keys,
        }
    }

    /// Apply a signed governance entry.
    pub async fn apply_governance_entry(&self, entry: GovernanceEntry) -> GovernanceResult<()> {
        // 1. Verify signature
        entry.verify()?;

        // 2. Check authorization
        if !self.authorized_keys.contains(&entry.verifying_key) {
            return Err(GovernanceError::Unauthorized(entry.issued_by));
        }

        // 3. Record the governance action in the underlying log (append-only)
        let entry_data = serde_json::to_vec(&entry)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;
        let _ = self.inner.append(
            &entry.issued_by,
            "governance/action",
            entry.timestamp,
            &hex::encode(entry_data),
            &entry.signature.to_bytes(),
        )?;

        // 4. Apply the action to the active state
        let mut state = self.state.write().await;
        match entry.action {
            GovernanceAction::RollbackCurriculum { target_sth, reason } => {
                state.last_rollback_sth = Some(target_sth);
                warn!(reason, "Rollback curriculum to STH");
                // In production: reload the UED state from the target STH.
            }
            GovernanceAction::AdjustTeacherReward { teacher_id, environment_hash, reward_delta, reason } => {
                let current = state.reward_adjustments.entry(teacher_id.clone()).or_insert(0.0);
                *current += reward_delta;
                warn!(teacher_id, environment_hash, reward_delta, reason, "Teacher reward adjusted");
            }
            GovernanceAction::BanRoutingPath { router_id, from_module, to_module, reason } => {
                let path = format!("{}->{}", from_module, to_module);
                state.banned_routes.entry(router_id.clone()).or_default().push(path);
                warn!(router_id, from_module, to_module, reason, "Routing path banned");
            }
            GovernanceAction::EmergencyFreeze { reason, duration_seconds } => {
                state.frozen = true;
                error!(reason, duration_seconds, "🚨 SYSTEM FROZEN");
                // Start a timer to unfreeze after duration_seconds.
                let state_clone = self.state.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(duration_seconds)).await;
                    let mut state = state_clone.write().await;
                    state.frozen = false;
                    info!("System unfrozen automatically after {} seconds", duration_seconds);
                });
            }
            GovernanceAction::Unfreeze { reason } => {
                state.frozen = false;
                info!(reason, "System unfrozen by governance action");
            }
        }
        Ok(())
    }

    // --- Query methods for integration ---

    pub async fn is_frozen(&self) -> bool {
        self.state.read().await.frozen
    }

    pub async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool {
        let state = self.state.read().await;
        if let Some(banned) = state.banned_routes.get(router_id) {
            banned.contains(&format!("{}->{}", from_module, to_module))
        } else {
            false
        }
    }

    pub async fn get_teacher_reward_delta(&self, teacher_id: &str) -> f64 {
        self.state.read().await
            .reward_adjustments.get(teacher_id)
            .copied()
            .unwrap_or(0.0)
    }

    pub async fn get_last_rollback_sth(&self) -> Option<Vec<u8>> {
        self.state.read().await.last_rollback_sth.clone()
    }

    /// Expose the underlying log for STH and proof operations.
    pub fn inner(&self) -> &TransparencyLog {
        &self.inner
    }
}
