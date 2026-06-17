use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, instrument};
use serde_json::json;
use futures::future::join_all;

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::deps::{SubagentSpawner, AttestationSigner};

pub struct PerformanceTestAgent {
    name: String,
    spawner: Arc<SubagentSpawner>,
    signer: Arc<dyn AttestationSigner + Send + Sync>,
    default_concurrency: usize,
}

impl PerformanceTestAgent {
    pub fn new(
        spawner: Arc<SubagentSpawner>,
        signer: Arc<dyn AttestationSigner + Send + Sync>,
        default_concurrency: usize,
    ) -> Self {
        Self {
            name: "PerformanceTestAgent".to_string(),
            spawner,
            signer,
            default_concurrency,
        }
    }
}

#[async_trait]
impl TestAgent for PerformanceTestAgent {
    fn test_name(&self) -> &str { &self.name }
    fn test_type(&self) -> TestType { TestType::Performance }

    #[instrument(name = "performance_test.run", skip(self))]
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String> {
        info!("⚡ Executando teste de performance...");

        let concurrency = context.parameters
            .get("concurrency")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.default_concurrency as u64) as usize;

        let tasks_count = context.parameters
            .get("tasks")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let task_template = context.parameters
            .get("task_template")
            .and_then(|v| v.as_str())
            .unwrap_or("echo 'test'");

        let start = std::time::Instant::now();

        let mut agent_ids = Vec::new();
        for i in 0..concurrency.min(50) {
            let purpose = format!("perf_test_{}", i);
            let sub = self.spawner.spawn(&purpose, vec!["echo".to_string()]).await?;
            agent_ids.push(sub.identity.id.clone());
        }

        let mut handles = Vec::new();
        for _ in 0..tasks_count {
            let ids = agent_ids.clone();
            let spawner = self.spawner.clone();
            let task = task_template.to_string();
            handles.push(tokio::spawn(async move {
                let mut results = Vec::new();
                for id in ids {
                    if let Some(sub) = spawner.get(&id).await {
                        let _ = sub.execute(&task, Some(0.01)).await;
                        results.push(true);
                    }
                }
                results.len()
            }));
        }

        let results = join_all(handles).await;
        let total_executions: usize = results.iter()
            .filter_map(|r| r.as_ref().ok())
            .sum();

        let duration_ms = start.elapsed().as_millis() as u64;
        let throughput = if duration_ms > 0 {
            (total_executions as f64 / (duration_ms as f64 / 1000.0)) as u64
        } else {
            0
        };

        for id in &agent_ids {
            let _ = self.spawner.terminate(id).await;
        }

        let details = json!({
            "concurrency": concurrency,
            "tasks": tasks_count,
            "total_executions": total_executions,
            "duration_ms": duration_ms,
            "throughput_per_second": throughput,
            "temporary_agents_spawned": agent_ids.len(),
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Performance,
            passed: total_executions > 0 && duration_ms < context.timeout.as_millis() as u64,
            duration_ms,
            details,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> bool {
        self.spawner.list_active().await.len() >= 0
    }

    fn config(&self) -> serde_json::Value {
        json!({
            "default_concurrency": self.default_concurrency,
            "agent_name": self.name,
        })
    }
}
