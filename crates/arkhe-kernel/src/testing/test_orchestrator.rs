use std::sync::Arc;
use tracing::{info, error, instrument};
use serde_json::json;
use futures::future::join_all;

use crate::testing::test_agent::{TestAgent, TestResult};
use crate::testing::test_attestation::TestAttestationExt;
use crate::testing::deps::{SubagentSpawner, AttestationManager, AttestationSigner, TrajectoryStore, ExecutionAttestation};

pub struct TestOrchestrator {
    spawner: Arc<SubagentSpawner>,
    attestation_manager: Arc<AttestationManager>,
    store: Arc<dyn TrajectoryStore + Send + Sync>,
    signer: Arc<dyn AttestationSigner + Send + Sync>,
    test_agents: Vec<Arc<dyn TestAgent>>,
}

impl TestOrchestrator {
    pub fn new(
        spawner: Arc<SubagentSpawner>,
        attestation_manager: Arc<AttestationManager>,
        store: Arc<dyn TrajectoryStore + Send + Sync>,
        signer: Arc<dyn AttestationSigner + Send + Sync>,
    ) -> Self {
        Self {
            spawner,
            attestation_manager,
            store,
            signer,
            test_agents: Vec::new(),
        }
    }

    pub fn register_test_agent(&mut self, agent: Arc<dyn TestAgent>) {
        info!("📋 Agente de teste registado: {}", agent.test_name());
        self.test_agents.push(agent);
    }

    #[instrument(name = "test_orchestrator.run_all", skip(self))]
    pub async fn run_all_tests(&self) -> Vec<TestResult> {
        info!("🚀 Executando todos os {} testes...", self.test_agents.len());

        let context = crate::testing::test_agent::TestContext::new("orchestrator");

        let handles: Vec<_> = self.test_agents.iter()
            .map(|agent| {
                let ctx = context.clone();
                let agent_clone = agent.clone();
                tokio::spawn(async move {
                    agent_clone.run_test(&ctx).await
                })
            })
            .collect();

        let results = join_all(handles).await;
        let mut test_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(test_result)) => {
                    let json = serde_json::to_string(&test_result).unwrap_or_default();
                    let _ = self.store.record_trajectory(
                        "test_orchestrator",
                        &format!("test_result:{}", test_result.test_name),
                        vec![format!("{:?}", test_result.test_type)],
                        &json,
                        vec![],
                        vec![],
                    ).await;
                    test_results.push(test_result);
                }
                Ok(Err(e)) => error!("Erro no teste: {}", e),
                Err(e) => error!("Panic no teste: {}", e),
            }
        }

        self.generate_report(&test_results).await;
        info!("✅ Testes concluídos: {} resultados", test_results.len());
        test_results
    }

    #[instrument(name = "test_orchestrator.run_all_with_tracing", skip(self))]
    pub async fn run_all_tests_with_tracing(&self) -> Vec<TestResult> {
        info!("🚀 Executando todos os testes com OpenTelemetry...");

        let context = crate::testing::test_agent::TestContext::new("orchestrator");

        let handles: Vec<_> = self.test_agents.iter()
            .map(|agent| {
                let ctx = context.clone();
                let agent_clone = agent.clone();
                tokio::spawn(async move {
                    agent_clone.run_test(&ctx).await
                })
            })
            .collect();

        let results = futures::future::join_all(handles).await;
        let mut test_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(test_result)) => {
                    if let Err(e) = test_result.store_test_result_as_attestation(
                        self.signer.as_ref(),
                        self.store.as_ref(),
                    ).await {
                        error!("Falha ao persistir atestado de teste: {}", e);
                    }
                    test_results.push(test_result);
                }
                Ok(Err(e)) => error!("Erro no teste: {}", e),
                Err(e) => error!("Panic no teste: {}", e),
            }
        }

        self.generate_report(&test_results).await;
        info!("✅ Testes concluídos com tracing: {} resultados", test_results.len());
        test_results
    }

    async fn generate_report(&self, results: &[TestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let report = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_tests": total,
            "passed": passed,
            "failed": failed,
            "success_rate": if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            "results": results.iter().map(|r| json!({
                "name": r.test_name,
                "type": format!("{:?}", r.test_type),
                "passed": r.passed,
                "duration_ms": r.duration_ms,
                "details": r.details,
            })).collect::<Vec<_>>(),
        });

        let report_json = serde_json::to_string_pretty(&report).unwrap_or_default();
        info!("📊 Relatório de testes:\n{}", report_json);

        let _ = self.store.record_trajectory(
            "test_orchestrator",
            "test_report",
            vec![],
            &report_json,
            vec![],
            vec![],
        ).await;

        let mut attestation = ExecutionAttestation::new(
            "test_report",
            &report_json,
            "test_orchestrator",
            0.0,
            vec!["test".to_string()],
            1.0,
            &self.signer.public_key(),
        );
        let _ = attestation.sign(self.signer.as_ref());
        let _ = self.attestation_manager.store_attestation(attestation).await;
    }

    pub async fn stats(&self) -> serde_json::Value {
        let trajs = self.store.list_trajectories().await;
        let test_results: Vec<_> = trajs.iter()
            .filter(|t| t.goal.starts_with("test_result:"))
            .collect();

        json!({
            "total_test_results": test_results.len(),
            "registered_test_agents": self.test_agents.len(),
        })
    }
}
