use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, instrument};
use serde_json::json;

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::deps::{SubagentSpawner, AttestationManager, AttestationSigner, TrajectoryStore};

pub struct IntegrationTestAgent {
    name: String,
    spawner: Arc<SubagentSpawner>,
    attestation_manager: Arc<AttestationManager>,
    store: Arc<dyn TrajectoryStore + Send + Sync>,
    signer: Arc<dyn AttestationSigner + Send + Sync>,
    test_count: usize,
}

impl IntegrationTestAgent {
    pub fn new(
        spawner: Arc<SubagentSpawner>,
        attestation_manager: Arc<AttestationManager>,
        store: Arc<dyn TrajectoryStore + Send + Sync>,
        signer: Arc<dyn AttestationSigner + Send + Sync>,
        test_count: usize,
    ) -> Self {
        Self {
            name: "IntegrationTestAgent".to_string(),
            spawner,
            attestation_manager,
            store,
            signer,
            test_count,
        }
    }

    async fn test_full_cycle(&self, count: usize) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        for i in 0..count {
            let purpose = format!("integration_test_{}", i);
            let sub = self.spawner.spawn(&purpose, vec!["echo".to_string()]).await
                .map_err(|e| format!("Falha ao spawnar: {}", e))?;

            let task = format!("echo 'test_{}'", i);
            let att = sub.execute(&task, Some(0.01)).await
                .map_err(|e| format!("Falha ao executar: {}", e))?;

            let retrieved = self.attestation_manager.get_attestation(&att.id).await
                .ok_or_else(|| format!("Atestado {} não recuperado", att.id))?;

            if retrieved.id != att.id {
                errors.push(format!("ID do atestado não corresponde: {} vs {}", att.id, retrieved.id));
            }

            self.spawner.terminate(&sub.identity.id).await
                .map_err(|e| format!("Falha ao terminar: {}", e))?;
        }

        Ok(errors)
    }

    async fn test_concurrency(&self, count: usize) -> Result<Vec<String>, String> {
        let mut handles = Vec::new();
        let spawner = self.spawner.clone();

        for i in 0..count {
            let spawner_clone = spawner.clone();
            handles.push(tokio::spawn(async move {
                let purpose = format!("concurrent_{}", i);
                spawner_clone.spawn(&purpose, vec!["echo".to_string()]).await
            }));
        }

        let results = futures::future::join_all(handles).await;
        let mut errors = Vec::new();

        for (idx, result) in results.iter().enumerate() {
            match result {
                Ok(Ok(_sub)) => {}
                Ok(Err(e)) => errors.push(format!("Falha no spawn concorrente {}: {}", idx, e)),
                Err(e) => errors.push(format!("Panic no spawn concorrente {}: {}", idx, e)),
            }
        }

        Ok(errors)
    }
}

#[async_trait]
impl TestAgent for IntegrationTestAgent {
    fn test_name(&self) -> &str { &self.name }
    fn test_type(&self) -> TestType { TestType::Integration }

    #[instrument(name = "integration_test.run", skip(self))]
    async fn run_test(&self, _context: &TestContext) -> Result<TestResult, String> {
        info!("🔗 Executando teste de integração...");

        let start = std::time::Instant::now();

        let cycle_errors = self.test_full_cycle(self.test_count).await?;
        let concurrent_errors = self.test_concurrency(self.test_count).await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let all_errors = [cycle_errors, concurrent_errors].concat();
        let passed = all_errors.is_empty();

        let details = json!({
            "test_count": self.test_count,
            "cycle_errors": all_errors.len(),
            "concurrent_errors": all_errors.len(),
            "total_errors": all_errors.len(),
            "errors": all_errors,
            "integration_status": if passed { "healthy" } else { "degraded" },
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Integration,
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
            "test_count": self.test_count,
            "agent_name": self.name,
        })
    }
}
