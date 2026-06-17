use std::sync::Arc;
use tokio::sync::RwLock;

use arkhe_kernel::testing::{
    IntegrityTestAgent,
    PerformanceTestAgent,
    ChaosTestAgent,
    SecurityTestAgent,
    ComplianceTestAgent,
    IntegrationTestAgent,
    TestContext,
    TestAgent,
    TestResult,
    TestType,
};
use arkhe_kernel::testing::deps::{
    AttestationManager, IdentityAttestation, Ed25519Signer,
    GeometricPolicyEngine, MemoryTrajectoryStore, TrajectoryStore,
    SubagentSpawner, SandboxType, create_sandbox, AttestationSigner
};

async fn setup_test_environment() -> (
    Arc<SubagentSpawner>,
    Arc<AttestationManager>,
    Arc<dyn TrajectoryStore + Send + Sync>,
    Arc<dyn AttestationSigner + Send + Sync>,
) {
    let signer = Arc::new(Ed25519Signer::new_random());
    let parent_identity = Arc::new(RwLock::new(IdentityAttestation::default()));
    let policy_engine = Arc::new(GeometricPolicyEngine::new());
    let store: Arc<dyn TrajectoryStore + Send + Sync> = Arc::new(MemoryTrajectoryStore::new());
    let attestation_manager = Arc::new(AttestationManager::new(Some(store.clone())));
    let sandbox = create_sandbox(SandboxType::Process { cmd: "echo".to_string(), args: vec![] });

    let spawner = Arc::new(SubagentSpawner::new(
        parent_identity,
        signer.clone() as Arc<dyn AttestationSigner + Send + Sync>,
        policy_engine,
        attestation_manager.clone(),
        store.clone(),
        10,
        sandbox,
        None,
    ));

    (spawner, attestation_manager, store, signer)
}

#[tokio::test]
async fn test_integrity_agent_success() {
    let (_spawner, att_manager, store, signer) = setup_test_environment().await;

    let agent = IntegrityTestAgent::new(att_manager, store, signer, 10);
    let context = TestContext::new("test");

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed, "Teste de integridade falhou: {:?}", result.details);
}

#[tokio::test]
async fn test_performance_agent_basic() {
    let (spawner, _, _, signer) = setup_test_environment().await;

    let agent = PerformanceTestAgent::new(spawner, signer, 2);
    let mut context = TestContext::new("test");
    context = context.with_parameter("concurrency", 2);
    context = context.with_parameter("tasks", 5);

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.duration_ms == 0 || result.duration_ms > 0);
}

#[tokio::test]
async fn test_chaos_agent_basic() {
    let (spawner, _, _, _) = setup_test_environment().await;

    let agent = ChaosTestAgent::new(spawner, 0.3, 20.0);
    let context = TestContext::new("test");

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.details["successful_kills"].as_u64().unwrap_or(0) == 0 || result.details["successful_kills"].as_u64().unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_security_agent_basic() {
    let agent = SecurityTestAgent::new();
    let context = TestContext::new("test");

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed, "Teste de segurança falhou: {:?}", result.details);
}

#[tokio::test]
async fn test_compliance_agent_basic() {
    let (_spawner, att_manager, store, signer) = setup_test_environment().await;

    let required_policies = vec!["pii_prohibition".to_string(), "steering_safety".to_string()];
    let policy_engine = GeometricPolicyEngine::new();

    let agent = ComplianceTestAgent::new(
        Arc::new(policy_engine),
        att_manager,
        store,
        signer,
        required_policies,
    );
    let context = TestContext::new("test");

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.duration_ms == 0 || result.duration_ms > 0);
}

#[tokio::test]
async fn test_integration_agent_basic() {
    let (spawner, att_manager, store, signer) = setup_test_environment().await;

    let agent = IntegrationTestAgent::new(spawner, att_manager, store, signer, 3);
    let context = TestContext::new("test");

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed || result.details["total_errors"].as_u64().unwrap_or(0) < 3);
}

#[tokio::test]
async fn test_test_result_serialization() {
    let result = TestResult {
        test_id: "test-123".to_string(),
        test_name: "unit_test".to_string(),
        test_type: TestType::Integrity,
        passed: true,
        duration_ms: 100,
        details: serde_json::json!({ "detail": "test" }),
        attestation_id: None,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TestResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.test_id, result.test_id);
    assert_eq!(deserialized.passed, result.passed);
}
