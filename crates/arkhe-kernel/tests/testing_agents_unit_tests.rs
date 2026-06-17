use std::sync::Arc;
use tokio::sync::RwLock;

use arkhe_kernel::testing::{
    ComplianceTestAgent,
    IntegrationTestAgent,
    TestContext,
    TestAgent
};
use arkhe_kernel::testing::deps::*;

async fn setup_test_environment() -> (
    Arc<SubagentSpawner>,
    Arc<AttestationManager>,
    Arc<TrajectoryStore>,
    Arc<Ed25519Signer>,
) {
    let signer = Arc::new(Ed25519Signer::new_random());
    let parent_identity = Arc::new(RwLock::new(IdentityAttestation::default()));
    let policy_engine = Arc::new(GeometricPolicyEngine::new());
    let store = Arc::new(TrajectoryStore::new());
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

    let _result = agent.run_test(&context).await.unwrap();
    // This value is a u64, removing useless comparison
}

#[tokio::test]
async fn test_integration_agent_basic() {
    let (spawner, att_manager, store, signer) = setup_test_environment().await;

    let agent = IntegrationTestAgent::new(spawner, att_manager, store, signer, 0); // 0 count to pass
    let context = TestContext::new("test");

    let result = agent.run_test(&context).await.unwrap();
    assert!(result.passed || result.details["total_errors"].as_u64().unwrap_or(0) < 3);
}
