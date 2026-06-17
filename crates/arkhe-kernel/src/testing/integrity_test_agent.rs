use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, instrument};
use serde_json::json;

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::deps::{AttestationManager, AttestationSigner, TrajectoryStoreTrait};

pub struct IntegrityTestAgent {
    name: String,
    attestation_manager: Arc<AttestationManager>,
    store: Arc<dyn TrajectoryStoreTrait>,
    _signer: Arc<dyn AttestationSigner>,
    max_samples: usize,
}

impl IntegrityTestAgent {
    pub fn new(
        attestation_manager: Arc<AttestationManager>,
        store: Arc<dyn TrajectoryStoreTrait>,
        _signer: Arc<dyn AttestationSigner>,
        max_samples: usize,
    ) -> Self {
        Self {
            name: "IntegrityTestAgent".to_string(),
            attestation_manager,
            store,
            _signer,
            max_samples,
        }
    }

    async fn verify_attestation_integrity(&self, att_id: &str) -> Result<bool, String> {
        let att = self.attestation_manager.get_attestation(att_id).await
            .ok_or_else(|| format!("Atestado {} não encontrado", att_id))?;
        self.attestation_manager.verify_attestation(&att).await
    }

    async fn verify_trajectory_chain(&self, agent_id: &str) -> Result<Vec<String>, String> {
        let trajs = self.store.list_trajectories().await;
        let agent_trajs: Vec<_> = trajs.iter()
            .filter(|t| t.agent_id == agent_id)
            .collect();

        if agent_trajs.is_empty() {
            return Err("Nenhuma trajetória encontrada".to_string());
        }

        let mut issues = Vec::new();
        for window in agent_trajs.windows(2) {
            if window[0].created_at > window[1].created_at {
                issues.push(format!(
                    "Ordem temporal violada: {} > {}",
                    window[0].id, window[1].id
                ));
            }
        }

        Ok(issues)
    }
}

#[async_trait]
impl TestAgent for IntegrityTestAgent {
    fn test_name(&self) -> &str { &self.name }
    fn test_type(&self) -> TestType { TestType::Integrity }

    #[instrument(name = "integrity_test.run", skip(self))]
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String> {
        info!("🔍 Executando teste de integridade...");

        let start = std::time::Instant::now();

        let trajs = self.store.list_trajectories().await;
        let attestation_trajs: Vec<_> = trajs.iter()
            .filter(|t| t.goal.starts_with("attestation:"))
            .take(self.max_samples)
            .collect();

        let mut verified = 0;
        let mut failed = 0;
        let mut issues = Vec::new();

        for traj in attestation_trajs {
            let att_id = &traj.goal.trim_start_matches("attestation:");
            match self.verify_attestation_integrity(att_id).await {
                Ok(true) => verified += 1,
                Ok(false) => {
                    failed += 1;
                    issues.push(format!("Atestado {} inválido", att_id));
                }
                Err(e) => {
                    failed += 1;
                    issues.push(format!("Erro ao verificar {}: {}", att_id, e));
                }
            }
        }

        if let Some(target_agent) = &context.target_agent_id {
            let chain_issues = self.verify_trajectory_chain(target_agent).await?;
            issues.extend(chain_issues);
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        let details = json!({
            "total_attestations_verified": verified + failed,
            "valid": verified,
            "invalid": failed,
            "issues": issues,
            "max_samples": self.max_samples,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Integrity,
            passed: failed == 0 && issues.is_empty(),
            duration_ms,
            details,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> bool {
        self.attestation_manager.stats().await.total_exec >= 0
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "max_samples": self.max_samples,
            "agent_name": self.name,
        })
    }
}
