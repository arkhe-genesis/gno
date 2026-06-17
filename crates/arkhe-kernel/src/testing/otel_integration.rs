use tracing::{info, error, span, Level, instrument};
use async_trait::async_trait;

use crate::testing::test_agent::{TestAgent, TestResult, TestContext};

#[async_trait]
pub trait TraceableTestAgent: TestAgent {
    async fn run_test_with_tracing(&self, context: &TestContext) -> Result<TestResult, String> {
        let span = span!(
            Level::INFO,
            "test.agent",
            test_name = %self.test_name(),
            test_type = ?self.test_type(),
            agent_id = %context.agent_id,
        );
        let _enter = span.enter();

        info!("🔄 Executando teste com tracing: {}", self.test_name());

        let result = self.run_test(context).await;

        match &result {
            Ok(test_result) => {
                span.record("passed", &test_result.passed);
                span.record("duration_ms", &test_result.duration_ms);
                info!("✅ Teste concluído: {} (passou: {})", test_result.test_name, test_result.passed);
            }
            Err(e) => {
                span.record("error", &e);
                error!("❌ Teste falhou: {} - {}", self.test_name(), e);
            }
        }

        result
    }
}

impl<T: TestAgent + ?Sized> TraceableTestAgent for T {}
