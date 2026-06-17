use async_trait::async_trait;
use tracing::{info, instrument};
use serde_json::json;

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::deps::WasiPreview2Sandbox;

pub struct SecurityTestAgent {
    name: String,
    test_cases: Vec<SecurityTestCase>,
}

#[derive(Debug, Clone)]
struct SecurityTestCase {
    name: String,
    wasm_code: Vec<u8>,
    expected_blocked: bool,
}

impl SecurityTestAgent {
    pub fn new() -> Self {
        let test_cases = vec![
            SecurityTestCase {
                name: "network_exfiltration".to_string(),
                wasm_code: vec![],
                expected_blocked: true,
            },
            SecurityTestCase {
                name: "filesystem_access".to_string(),
                wasm_code: vec![],
                expected_blocked: true,
            },
            SecurityTestCase {
                name: "environment_leak".to_string(),
                wasm_code: vec![],
                expected_blocked: true,
            },
        ];
        Self {
            name: "SecurityTestAgent".to_string(),
            test_cases,
        }
    }
}

#[async_trait]
impl TestAgent for SecurityTestAgent {
    fn test_name(&self) -> &str { &self.name }
    fn test_type(&self) -> TestType { TestType::Security }

    #[instrument(name = "security_test.run", skip(self))]
    async fn run_test(&self, _context: &TestContext) -> Result<TestResult, String> {
        info!("🔐 Executando teste de segurança...");

        let mut results = Vec::new();
        let mut blocked_count = 0;

        for test_case in &self.test_cases {
            let sandbox = WasiPreview2Sandbox::new(test_case.wasm_code.clone()).await?;
            let result = sandbox.execute("test", "test").await;
            let is_blocked = result.is_err();
            let test_passed = is_blocked == test_case.expected_blocked;
            if test_passed {
                blocked_count += 1;
            }
            results.push(json!({
                "test": test_case.name,
                "blocked": is_blocked,
                "expected_blocked": test_case.expected_blocked,
                "passed": test_passed,
            }));
        }

        let all_passed = blocked_count == self.test_cases.len();
        let details = json!({
            "total_tests": self.test_cases.len(),
            "passed": blocked_count,
            "failed": self.test_cases.len() - blocked_count,
            "results": results,
        });

        Ok(TestResult {
            test_id: uuid::Uuid::new_v4().to_string(),
            test_name: self.name.clone(),
            test_type: TestType::Security,
            passed: all_passed,
            duration_ms: 0,
            details,
            attestation_id: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> bool { true }

    fn config(&self) -> serde_json::Value {
        json!({
            "test_cases": self.test_cases.len(),
            "agent_name": self.name,
        })
    }
}

impl Default for SecurityTestAgent {
    fn default() -> Self {
        Self::new()
    }
}
