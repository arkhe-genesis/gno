//! Stubs for missing dependencies needed by the testing module.

use std::sync::Arc;
use tokio::sync::RwLock;

// ----------------------------------------------------------------------------
// core models
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IdentityAttestation {
    pub id: String,
}

impl Default for IdentityAttestation {
    fn default() -> Self {
        Self { id: uuid::Uuid::new_v4().to_string() }
    }
}

pub struct Subagent {
    pub identity: IdentityAttestation,
}

impl Subagent {
    pub async fn execute(&self, _task: &str, _timeout: Option<f64>) -> Result<ExecutionAttestation, String> {
        Ok(ExecutionAttestation::default())
    }
}

// ----------------------------------------------------------------------------
// SubagentSpawner & Sandbox
// ----------------------------------------------------------------------------

pub enum SandboxType {
    Process { cmd: String, args: Vec<String> },
    Wasm,
}

pub struct WasiPreview2Sandbox {
    _code: Vec<u8>,
}

impl WasiPreview2Sandbox {
    pub async fn new(code: Vec<u8>) -> Result<Self, String> {
        Ok(Self { _code: code })
    }

    pub async fn execute(&self, _env: &str, _arg: &str) -> Result<(), String> {
        Err("blocked".to_string())
    }
}

pub fn create_sandbox(st: SandboxType) -> Arc<dyn Sandbox + Send + Sync> {
    Arc::new(DummySandbox)
}

pub trait Sandbox: Send + Sync {}
pub struct DummySandbox;
impl Sandbox for DummySandbox {}


pub struct SubagentSpawner {
    parent_identity: Arc<RwLock<IdentityAttestation>>,
    signer: Arc<dyn AttestationSigner + Send + Sync>,
    policy_engine: Arc<GeometricPolicyEngine>,
    attestation_manager: Arc<AttestationManager>,
    store: Arc<dyn TrajectoryStore + Send + Sync>,
}

impl SubagentSpawner {
    pub fn new(
        parent_identity: Arc<RwLock<IdentityAttestation>>,
        signer: Arc<dyn AttestationSigner + Send + Sync>,
        policy_engine: Arc<GeometricPolicyEngine>,
        attestation_manager: Arc<AttestationManager>,
        store: Arc<dyn TrajectoryStore + Send + Sync>,
        _max_agents: usize,
        _sandbox: Arc<dyn Sandbox + Send + Sync>,
        _llm: Option<()>,
    ) -> Self {
        Self { parent_identity, signer, policy_engine, attestation_manager, store }
    }

    pub async fn spawn(&self, _purpose: &str, _args: Vec<String>) -> Result<Subagent, String> {
        Ok(Subagent { identity: IdentityAttestation::default() })
    }

    pub async fn get(&self, id: &str) -> Option<Subagent> {
        Some(Subagent { identity: IdentityAttestation { id: id.to_string() } })
    }

    pub async fn terminate(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn terminate_all(&self) -> Result<(), String> {
        Ok(())
    }

    pub async fn list_active(&self) -> Vec<Subagent> {
        vec![Subagent { identity: IdentityAttestation::default() }]
    }
}

// ----------------------------------------------------------------------------
// AttestationManager & Signer
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionAttestation {
    pub id: String,
    pub tags: Vec<String>,
}

impl Default for ExecutionAttestation {
    fn default() -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), tags: vec![] }
    }
}

impl ExecutionAttestation {
    pub fn new(
        _title: &str,
        _details: &str,
        _source: &str,
        _cost: f64,
        tags: Vec<String>,
        _success: f64,
        _pub_key: &str,
    ) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), tags }
    }

    pub fn sign(&mut self, _signer: &dyn AttestationSigner) -> Result<(), String> {
        Ok(())
    }
}


pub struct AttestationStats {
    pub total_exec: usize,
}

pub struct AttestationManager {
    _store: Option<Arc<dyn TrajectoryStore + Send + Sync>>,
}

impl AttestationManager {
    pub fn new(store: Option<Arc<dyn TrajectoryStore + Send + Sync>>) -> Self {
        Self { _store: store }
    }

    pub async fn get_attestation(&self, id: &str) -> Option<ExecutionAttestation> {
        Some(ExecutionAttestation { id: id.to_string(), tags: vec![] })
    }

    pub async fn verify_attestation(&self, _att: &ExecutionAttestation) -> Result<bool, String> {
        Ok(true)
    }

    pub async fn store_attestation(&self, _att: ExecutionAttestation) -> Result<(), String> {
        Ok(())
    }

    pub async fn stats(&self) -> AttestationStats {
        AttestationStats { total_exec: 1 }
    }
}

pub trait AttestationSigner: Send + Sync {
    fn sign(&self, data: &str) -> Result<String, String>;
    fn verify(&self, data: &str, sig: &str) -> Result<bool, String>;
    fn public_key(&self) -> String;
}

pub struct Ed25519Signer;
impl Ed25519Signer {
    pub fn new_random() -> Self { Self }
}
impl AttestationSigner for Ed25519Signer {
    fn sign(&self, _data: &str) -> Result<String, String> { Ok("sig".to_string()) }
    fn verify(&self, _data: &str, _sig: &str) -> Result<bool, String> { Ok(true) }
    fn public_key(&self) -> String { "pubkey".to_string() }
}


// ----------------------------------------------------------------------------
// TrajectoryStore
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub id: String,
    pub agent_id: String,
    pub goal: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait TrajectoryStore: Send + Sync {
    async fn record_trajectory(
        &self,
        agent_id: &str,
        goal: &str,
        tags: Vec<String>,
        result_json: &str,
        _deps: Vec<String>,
        _attestations: Vec<String>,
    ) -> Result<String, String>;

    async fn list_trajectories(&self) -> Vec<Trajectory>;
}

pub struct MemoryTrajectoryStore;
impl MemoryTrajectoryStore {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl TrajectoryStore for MemoryTrajectoryStore {
    async fn record_trajectory(
        &self,
        agent_id: &str,
        goal: &str,
        _tags: Vec<String>,
        _result_json: &str,
        _deps: Vec<String>,
        _attestations: Vec<String>,
    ) -> Result<String, String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn list_trajectories(&self) -> Vec<Trajectory> {
        vec![]
    }
}


// ----------------------------------------------------------------------------
// GeometricPolicyEngine
// ----------------------------------------------------------------------------

pub struct Policy {
    pub name: String,
}

pub struct GeometricPolicyEngine;

impl GeometricPolicyEngine {
    pub fn new() -> Self { Self }

    pub async fn list_active_policies(&self) -> Result<Vec<Policy>, String> {
        Ok(vec![])
    }
}
