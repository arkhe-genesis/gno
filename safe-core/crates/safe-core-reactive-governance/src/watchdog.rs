//! Autonomous watchdog that monitors metrics and proposes governance actions.

use crate::reactive_log::ReactiveLog;
use crate::governance::{GovernanceAction, GovernanceEntry, GovernanceError};
// use crate::crypto::Hasher;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, warn, info};
use metrics::{counter, gauge};
use crypto::DynPublicKey;
use crypto::DynSignature;
use crate::hsm_backend::HsmBackend;

/// Watchdog configuration.
#[derive(Clone)]
pub struct WatchdogConfig {
    pub check_interval_secs: u64,
    pub consecutive_failures_threshold: u32,
    pub governance_key_id: String,
    pub governance_hsm: Arc<dyn HsmBackend>, // assumed to be in scope
}

/// Autonomous watchdog that proposes governance actions based on metrics.
pub struct GovernanceWatchdog {
    log: Arc<ReactiveLog>,
    config: WatchdogConfig,
    consecutive_attestation_failures: u32,
    last_metrics: MetricsSnapshot,
}

#[derive(Default)]
struct MetricsSnapshot {
    attestation_trusted: f64,
    tool_call_error_rate: f64,
    ued_teacher_failure_rate: f64,
}

impl GovernanceWatchdog {
    pub fn new(log: Arc<ReactiveLog>, config: WatchdogConfig) -> Self {
        Self {
            log,
            config,
            consecutive_attestation_failures: 0,
            last_metrics: MetricsSnapshot::default(),
        }
    }

    /// Start the watchdog loop.
    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.check_interval_secs));
        loop {
            interval.tick().await;
            self.check_and_act().await;
        }
    }

    async fn check_and_act(&mut self) {
        // 1. Collect metrics (simulated; in production use Prometheus API)
        let metrics = self.collect_metrics().await;

        // 2. Check attestation failures
        if metrics.attestation_trusted == 0.0 {
            self.consecutive_attestation_failures += 1;
        } else {
            self.consecutive_attestation_failures = 0;
        }

        // 3. If threshold exceeded, propose EmergencyFreeze
        if self.consecutive_attestation_failures >= self.config.consecutive_failures_threshold {
            let action = GovernanceAction::EmergencyFreeze {
                reason: format!(
                    "Attestation failure for {} consecutive checks",
                    self.consecutive_attestation_failures
                ),
                duration_seconds: 300,
            };
            if let Err(e) = self.propose_governance(action).await {
                error!("Failed to propose governance action: {}", e);
            }
            // Reset counter to avoid spamming
            self.consecutive_attestation_failures = 0;
        }

        // 4. Check teacher failure rate (if available)
        if metrics.ued_teacher_failure_rate > 0.5 {
            // Penalize the teacher
            let action = GovernanceAction::AdjustTeacherReward {
                teacher_id: "default-teacher".to_string(),
                environment_hash: "".to_string(), // could be derived from last failure
                reward_delta: -0.2,
                reason: "High failure rate detected".to_string(),
            };
            if let Err(e) = self.propose_governance(action).await {
                error!("Failed to propose teacher reward adjustment: {}", e);
            }
        }

        // 5. Update metrics gauge
        gauge!("watchdog_attestation_failures").set(self.consecutive_attestation_failures as f64);
    }

    async fn collect_metrics(&self) -> MetricsSnapshot {
        // In production: query Prometheus or use metrics crate directly.
        // For simulation, return values from gauges.
        let attestation = 1.0; // stub
        let error_rate = 0.0;
        let teacher_failure = 0.0;

        MetricsSnapshot {
            attestation_trusted: attestation,
            tool_call_error_rate: error_rate,
            ued_teacher_failure_rate: teacher_failure,
        }
    }

    async fn propose_governance(&self, action: GovernanceAction) -> Result<(), GovernanceError> {
        // Sign the action using the governance HSM
        let payload = serde_json::to_vec(&action)
            .map_err(|e| GovernanceError::Serialization(e.to_string()))?;
        let signature = self.config.governance_hsm
            .sign(&self.config.governance_key_id, &payload)
            .map_err(|e| GovernanceError::InvalidSignature(e.to_string()))?;
        let verifying_key = self.config.governance_hsm
            .export_public_key(&self.config.governance_key_id)
            .map_err(|e| GovernanceError::InvalidSignature(e.to_string()))?;

        let entry = GovernanceEntry {
            action,
            issued_by: "watchdog".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            signature,
            verifying_key,
        };

        info!("Watchdog proposing action: {:?}", entry);
        // Actually apply it:
        self.log.apply_governance_entry(entry).await?;
        Ok(())
    }
}
