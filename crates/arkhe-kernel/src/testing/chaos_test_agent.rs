use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn, instrument};
use serde_json::json;
use rand::{Rng, thread_rng};

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::deps::SubagentSpawner;

pub struct ChaosTestAgent {
    name: String,
    spawner: Arc<SubagentSpawner>,
    failure_rate: f64,
    kill_percentage: f32,
}

impl ChaosTestAgent {
    pub fn new(
        spawner: Arc<SubagentSpawner>,
        failure_rate: f64,
        kill_percentage: f32,
    ) -> Self {
        Self {
            name: "ChaosTestAgent".to_string(),
            spawner,
            failure_rate: failure_rate.clamp(0.0, 1.0),
            kill_percentage: kill_percentage.clamp(0.0, 100.0),
        }
    }
}

#[async_trait]
impl TestAgent for ChaosTestAgent {
    fn test_name(&self) -> &str { &self.name }
    fn test_type(&self) -> TestType { TestType::Chaos }

    #[instrument(name = "chaos_test.run", skip(self))]
    async fn run_test(&self, _context: &TestContext) -> Result<TestResult, String> {
        info!("💀 Executando teste de caos...");

        let active = self.spawner.list_active().await;
        let total = active.len();

        if total == 0 {
            return Err("Nenhum subagente ativo para testar".to_string());
        }

        let mut killed = 0;
        let mut errors = 0;

        let kill_count = ((total as f32) * self.kill_percentage / 100.0) as usize;
        let to_kill: Vec<String> = active.iter()
            .take(kill_count)
            .map(|s| s.identity.id.clone())
            .collect();

        for id in &to_kill {
            // Need to drop rng before the await point
            let simulate_failure = {
                let mut rng = thread_rng();
                rng.gen_bool(self.failure_rate)
            };
            if simulate_failure {
                warn!("💥 Falha simulada ao terminar {}", id);
                errors += 1;
            } else {
                if let Err(e) = self.spawner.terminate(id).await {
                    warn!("⚠️ Erro ao terminar {}: {}", id, e);
                    errors += 1;
                } else {
                    info!("💀 Subagente {} morto (caos)", id);
                    killed += 1;
                }
            }
        }

        let after_kill = self.spawner.list_active().await;
        let recovered = after_kill.len().saturating_sub(total - killed);

        let details = json!({
            "total_agents_before": total,
            "attempted_kills": kill_count,
            "successful_kills": killed,
            "simulated_failures": errors,
            "agents_recovered": recovered,
            "failure_rate": self.failure_rate,
            "kill_percentage": self.kill_percentage,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Chaos,
            passed: killed >= (kill_count / 2).max(1),
            duration_ms: 0,
            details,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> bool { true }

    fn config(&self) -> serde_json::Value {
        json!({
            "failure_rate": self.failure_rate,
            "kill_percentage": self.kill_percentage,
            "agent_name": self.name,
        })
    }
}
