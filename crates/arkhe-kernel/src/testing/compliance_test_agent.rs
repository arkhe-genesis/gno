use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, instrument};
use serde_json::json;

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::deps::{AttestationManager, AttestationSigner, GeometricPolicyEngine, TrajectoryStoreTrait};

pub struct ComplianceTestAgent {
    name: String,
    policy_engine: Arc<GeometricPolicyEngine>,
    attestation_manager: Arc<AttestationManager>,
    store: Arc<dyn TrajectoryStoreTrait>,
    _signer: Arc<dyn AttestationSigner>,
    required_policies: Vec<String>,
}

impl ComplianceTestAgent {
    pub fn new(
        policy_engine: Arc<GeometricPolicyEngine>,
        attestation_manager: Arc<AttestationManager>,
        store: Arc<dyn TrajectoryStoreTrait>,
        _signer: Arc<dyn AttestationSigner>,
        required_policies: Vec<String>,
    ) -> Self {
        Self {
            name: "ComplianceTestAgent".to_string(),
            policy_engine,
            attestation_manager,
            store,
            _signer,
            required_policies,
        }
    }

    async fn verify_policies_active(&self) -> Result<Vec<String>, String> {
        let active = self.policy_engine.list_active_policies().await?;
        let missing: Vec<String> = self.required_policies
            .iter()
            .filter(|p| !active.iter().any(|a| a.name == **p))
            .cloned()
            .collect();
        Ok(missing)
    }

    async fn verify_attestations_compliance(&self) -> Result<(usize, Vec<String>), String> {
        let trajs = self.store.list_trajectories().await;
        let mut non_compliant = Vec::new();
        let mut total = 0;

        for traj in trajs.iter().filter(|t| t.goal.starts_with("attestation:")) {
            total += 1;
            if let Some(att) = self.attestation_manager.get_attestation(&traj.id).await {
                for policy in &self.required_policies {
                    if !att.tags.iter().any(|t| t == policy) {
                        non_compliant.push(format!("Atestado {} não contém política {}", att.id, policy));
                        break;
                    }
                }
            }
        }

        Ok((total, non_compliant))
    }
}

#[async_trait]
impl TestAgent for ComplianceTestAgent {
    fn test_name(&self) -> &str { &self.name }
    fn test_type(&self) -> TestType { TestType::Compliance }

    #[instrument(name = "compliance_test.run", skip(self))]
    async fn run_test(&self, _context: &TestContext) -> Result<TestResult, String> {
        info!("📜 Executando teste de conformidade...");

        let start = std::time::Instant::now();

        let missing_policies = self.verify_policies_active().await?;
        let (total_attestations, issues) = self.verify_attestations_compliance().await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = missing_policies.is_empty() && issues.is_empty();

        let details = json!({
            "required_policies": self.required_policies,
            "missing_policies": missing_policies,
            "attestations_checked": total_attestations,
            "non_compliant_attestations": issues.len(),
            "issues": issues,
            "compliance_score": if total_attestations > 0 {
                (total_attestations - issues.len()) as f64 / total_attestations as f64
            } else {
                1.0
            },
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Compliance,
            passed,
            duration_ms,
            details,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> bool { true }

    fn config(&self) -> serde_json::Value {
        json!({
            "required_policies": self.required_policies,
            "agent_name": self.name,
        })
    }
}
