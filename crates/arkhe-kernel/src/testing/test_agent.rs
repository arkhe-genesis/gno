use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub test_type: TestType,
    pub passed: bool,
    pub duration_ms: u64,
    pub details: serde_json::Value,
    pub attestation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    Integrity,
    Performance,
    Chaos,
    Security,
    Compliance,
    Integration,
}

#[async_trait]
pub trait TestAgent: Send + Sync {
    fn test_name(&self) -> &str;
    fn test_type(&self) -> TestType;
    async fn run_test(&self, context: &TestContext) -> Result<TestResult, String>;
    async fn health_check(&self) -> bool;
    fn config(&self) -> serde_json::Value;
}

#[derive(Debug, Clone)]
pub struct TestContext {
    pub agent_id: String,
    pub target_url: Option<String>,
    pub target_agent_id: Option<String>,
    pub timeout: Duration,
    pub max_agents: usize,
    pub parameters: serde_json::Map<String, serde_json::Value>,
}

impl TestContext {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            target_url: None,
            target_agent_id: None,
            timeout: Duration::from_secs(30),
            max_agents: 100,
            parameters: serde_json::Map::new(),
        }
    }

    pub fn with_target(mut self, target: &str) -> Self {
        self.target_url = Some(target.to_string());
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    pub fn with_parameter<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(val) = serde_json::to_value(value) {
            self.parameters.insert(key.to_string(), val);
        }
        self
    }
}
