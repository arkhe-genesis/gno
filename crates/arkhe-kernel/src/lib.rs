#![allow(unsafe_op_in_unsafe_fn, dead_code, unused_variables, unused_comparisons, non_local_definitions)]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod testing;
pub mod substrato_4004;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use std::sync::Arc;
#[cfg(feature = "python")]
use tokio::runtime::Runtime;
#[cfg(feature = "python")]
use tokio::sync::Mutex;
#[cfg(feature = "python")]


#[cfg(feature = "python")]
#[pyclass]
struct PyTestOrchestrator {
    inner: Arc<Mutex<crate::testing::TestOrchestrator>>,
    rt: Runtime,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyTestOrchestrator {
    #[new]
    fn new() -> PyResult<Self> {
        let parent_identity = Arc::new(tokio::sync::RwLock::new(crate::testing::deps::IdentityAttestation::default()));
        let signer: Arc<dyn crate::testing::deps::AttestationSigner + Send + Sync> = Arc::new(crate::testing::deps::Ed25519Signer::new_random());
        let policy_engine = Arc::new(crate::testing::deps::GeometricPolicyEngine::new());
        let store: Arc<dyn crate::testing::deps::TrajectoryStore + Send + Sync> = Arc::new(crate::testing::deps::MemoryTrajectoryStore::new());
        let attestation_manager = Arc::new(crate::testing::deps::AttestationManager::new(Some(store.clone())));
        let sandbox = crate::testing::deps::create_sandbox(crate::testing::deps::SandboxType::Process { cmd: "echo".to_string(), args: vec![] });

        let spawner = Arc::new(crate::testing::deps::SubagentSpawner::new(
            parent_identity,
            signer.clone(),
            policy_engine,
            attestation_manager.clone(),
            store.clone(),
            10,
            sandbox,
            None,
        ));

        let orchestrator = crate::testing::TestOrchestrator::new(
            spawner,
            attestation_manager,
            store,
            signer,
        );

        Ok(Self {
            inner: Arc::new(Mutex::new(orchestrator)),
            rt: Runtime::new().unwrap(),
        })
    }

    fn register_integrity_test(&self, max_samples: usize) -> PyResult<()> {
        let store: Arc<dyn crate::testing::deps::TrajectoryStore + Send + Sync> = Arc::new(crate::testing::deps::MemoryTrajectoryStore::new());
        let att_manager = Arc::new(crate::testing::deps::AttestationManager::new(Some(store.clone())));
        let signer: Arc<dyn crate::testing::deps::AttestationSigner + Send + Sync> = Arc::new(crate::testing::deps::Ed25519Signer::new_random());

        let agent = Arc::new(crate::testing::IntegrityTestAgent::new(
            att_manager,
            store,
            signer,
            max_samples,
        ));

        self.rt.block_on(async {
            self.inner.lock().await.register_test_agent(agent);
        });
        Ok(())
    }

    fn register_performance_test(&self, concurrency: usize) -> PyResult<()> {
        let parent_identity = Arc::new(tokio::sync::RwLock::new(crate::testing::deps::IdentityAttestation::default()));
        let signer: Arc<dyn crate::testing::deps::AttestationSigner + Send + Sync> = Arc::new(crate::testing::deps::Ed25519Signer::new_random());
        let policy_engine = Arc::new(crate::testing::deps::GeometricPolicyEngine::new());
        let store: Arc<dyn crate::testing::deps::TrajectoryStore + Send + Sync> = Arc::new(crate::testing::deps::MemoryTrajectoryStore::new());
        let attestation_manager = Arc::new(crate::testing::deps::AttestationManager::new(Some(store.clone())));
        let sandbox = crate::testing::deps::create_sandbox(crate::testing::deps::SandboxType::Process { cmd: "echo".to_string(), args: vec![] });

        let spawner = Arc::new(crate::testing::deps::SubagentSpawner::new(
            parent_identity,
            signer.clone(),
            policy_engine,
            attestation_manager,
            store,
            10,
            sandbox,
            None,
        ));

        let agent = Arc::new(crate::testing::PerformanceTestAgent::new(
            spawner,
            signer,
            concurrency,
        ));

        self.rt.block_on(async {
            self.inner.lock().await.register_test_agent(agent);
        });
        Ok(())
    }

    fn register_chaos_test(&self, failure_rate: f64, kill_percentage: f32) -> PyResult<()> {
        let parent_identity = Arc::new(tokio::sync::RwLock::new(crate::testing::deps::IdentityAttestation::default()));
        let signer: Arc<dyn crate::testing::deps::AttestationSigner + Send + Sync> = Arc::new(crate::testing::deps::Ed25519Signer::new_random());
        let policy_engine = Arc::new(crate::testing::deps::GeometricPolicyEngine::new());
        let store: Arc<dyn crate::testing::deps::TrajectoryStore + Send + Sync> = Arc::new(crate::testing::deps::MemoryTrajectoryStore::new());
        let attestation_manager = Arc::new(crate::testing::deps::AttestationManager::new(Some(store.clone())));
        let sandbox = crate::testing::deps::create_sandbox(crate::testing::deps::SandboxType::Process { cmd: "echo".to_string(), args: vec![] });

        let spawner = Arc::new(crate::testing::deps::SubagentSpawner::new(
            parent_identity,
            signer.clone(),
            policy_engine,
            attestation_manager,
            store,
            10,
            sandbox,
            None,
        ));

        let agent = Arc::new(crate::testing::ChaosTestAgent::new(
            spawner,
            failure_rate,
            kill_percentage,
        ));

        self.rt.block_on(async {
            self.inner.lock().await.register_test_agent(agent);
        });
        Ok(())
    }

    fn register_security_test(&self) -> PyResult<()> {
        let agent = Arc::new(crate::testing::SecurityTestAgent::new());
        self.rt.block_on(async {
            self.inner.lock().await.register_test_agent(agent);
        });
        Ok(())
    }

    fn register_compliance_test(&self, required_policies: Vec<String>) -> PyResult<()> {
        let policy_engine = Arc::new(crate::testing::deps::GeometricPolicyEngine::new());
        let store: Arc<dyn crate::testing::deps::TrajectoryStore + Send + Sync> = Arc::new(crate::testing::deps::MemoryTrajectoryStore::new());
        let att_manager = Arc::new(crate::testing::deps::AttestationManager::new(Some(store.clone())));
        let signer: Arc<dyn crate::testing::deps::AttestationSigner + Send + Sync> = Arc::new(crate::testing::deps::Ed25519Signer::new_random());

        let agent = Arc::new(crate::testing::ComplianceTestAgent::new(
            policy_engine,
            att_manager,
            store,
            signer,
            required_policies,
        ));

        self.rt.block_on(async {
            self.inner.lock().await.register_test_agent(agent);
        });
        Ok(())
    }

    fn register_integration_test(&self, test_count: usize) -> PyResult<()> {
        let parent_identity = Arc::new(tokio::sync::RwLock::new(crate::testing::deps::IdentityAttestation::default()));
        let signer: Arc<dyn crate::testing::deps::AttestationSigner + Send + Sync> = Arc::new(crate::testing::deps::Ed25519Signer::new_random());
        let policy_engine = Arc::new(crate::testing::deps::GeometricPolicyEngine::new());
        let store: Arc<dyn crate::testing::deps::TrajectoryStore + Send + Sync> = Arc::new(crate::testing::deps::MemoryTrajectoryStore::new());
        let attestation_manager = Arc::new(crate::testing::deps::AttestationManager::new(Some(store.clone())));
        let sandbox = crate::testing::deps::create_sandbox(crate::testing::deps::SandboxType::Process { cmd: "echo".to_string(), args: vec![] });

        let spawner = Arc::new(crate::testing::deps::SubagentSpawner::new(
            parent_identity,
            signer.clone(),
            policy_engine,
            attestation_manager.clone(),
            store.clone(),
            10,
            sandbox,
            None,
        ));

        let agent = Arc::new(crate::testing::IntegrationTestAgent::new(
            spawner,
            attestation_manager,
            store,
            signer,
            test_count,
        ));

        self.rt.block_on(async {
            self.inner.lock().await.register_test_agent(agent);
        });
        Ok(())
    }

    fn run_all_tests(&self) -> PyResult<String> {
        let results = self.rt.block_on(async {
            self.inner.lock().await.run_all_tests().await
        });
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Serialization error: {}", e)))?;
        Ok(json)
    }

    fn stats(&self, py: Python) -> PyResult<PyObject> {
        let stats = self.rt.block_on(async {
            self.inner.lock().await.stats().await
        });

        let json_str = serde_json::to_string(&stats).unwrap();
        let json_module = py.import("json")?;
        let result = json_module.getattr("loads")?.call1((json_str,))?;
        Ok(result.into())
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn cathedral_arkhe(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTestOrchestrator>()?;
    Ok(())
}
