use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// Stubs to make the testing module compile

#[derive(Default, Clone)]
pub struct IdentityAttestation {
    pub id: String,
}

pub trait AttestationSigner: Send + Sync {
    fn public_key(&self) -> String;
    fn sign(&self, data: &str) -> Result<String, String>;
    fn verify(&self, data: &str, sig: &str) -> Result<bool, String>;
}

#[derive(Clone)]
pub struct Ed25519Signer;
impl Ed25519Signer {
    pub fn new_random() -> Self { Self }
}
impl AttestationSigner for Ed25519Signer {
    fn public_key(&self) -> String { "pub_key".to_string() }
    fn sign(&self, _data: &str) -> Result<String, String> { Ok("sig".to_string()) }
    fn verify(&self, _data: &str, _sig: &str) -> Result<bool, String> { Ok(true) }
}

#[derive(Default)]
pub struct AttestationManager;
impl AttestationManager {
    pub fn new(_store: Option<Arc<TrajectoryStore>>) -> Self { Self }
    pub async fn get_attestation(&self, _id: &str) -> Option<ExecutionAttestation> { None }
    pub async fn verify_attestation(&self, _att: &ExecutionAttestation) -> Result<bool, String> { Ok(true) }
    pub async fn stats(&self) -> AttestationStats { AttestationStats { total_exec: 0 } }
    pub async fn store_attestation(&self, _att: ExecutionAttestation) {}
}

pub struct AttestationStats {
    pub total_exec: usize,
}

#[derive(Clone, Default)]
pub struct ExecutionAttestation {
    pub id: String,
    pub tags: Vec<String>,
}
impl ExecutionAttestation {
    pub fn new(_a: &str, _b: &str, _c: &str, _d: f64, _e: Vec<String>, _f: f64, _g: &str) -> Self { Self::default() }
    pub fn sign(&mut self, _signer: &dyn AttestationSigner) -> Result<(), String> { Ok(()) }
}

pub struct PolicyDescriptor {
    pub name: String,
}

#[derive(Default)]
pub struct GeometricPolicyEngine;
impl GeometricPolicyEngine {
    pub fn new() -> Self { Self }
    pub async fn list_active_policies(&self) -> Result<Vec<PolicyDescriptor>, String> { Ok(vec![]) }
}

pub struct Trajectory {
    pub id: String,
    pub goal: String,
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait TrajectoryStoreTrait: Send + Sync {
    async fn list_trajectories(&self) -> Vec<Trajectory>;
    async fn record_trajectory(&self, _a: &str, _b: &str, _c: Vec<String>, _d: &str, _e: Vec<String>, _f: Vec<String>) -> Result<String, String>;
}

#[derive(Default)]
pub struct TrajectoryStore;
impl TrajectoryStore {
    pub fn new() -> Self { Self }
}
#[async_trait]
impl TrajectoryStoreTrait for TrajectoryStore {
    async fn list_trajectories(&self) -> Vec<Trajectory> { vec![] }
    async fn record_trajectory(&self, _a: &str, _b: &str, _c: Vec<String>, _d: &str, _e: Vec<String>, _f: Vec<String>) -> Result<String, String> { Ok("id".to_string()) }
}

#[derive(Clone)]
pub struct SubagentIdentity {
    pub id: String,
}

#[derive(Clone)]
pub struct Subagent {
    pub identity: SubagentIdentity,
}
impl Subagent {
    pub async fn execute(&self, _task: &str, _cost: Option<f64>) -> Result<ExecutionAttestation, String> { Ok(ExecutionAttestation::default()) }
}

pub struct SubagentSpawner;
impl SubagentSpawner {
    pub fn new(_a: Arc<tokio::sync::RwLock<IdentityAttestation>>, _b: Arc<dyn AttestationSigner>, _c: Arc<GeometricPolicyEngine>, _d: Arc<AttestationManager>, _e: Arc<TrajectoryStore>, _f: usize, _g: Arc<dyn Sandbox>, _h: Option<()>) -> Self { Self }
    pub async fn spawn(&self, _purpose: &str, _tools: Vec<String>) -> Result<Subagent, String> { Ok(Subagent { identity: SubagentIdentity { id: "id".to_string() } }) }
    pub async fn terminate(&self, _id: &str) -> Result<(), String> { Ok(()) }
    pub async fn terminate_all(&self) -> Result<(), String> { Ok(()) }
    pub async fn get(&self, _id: &str) -> Option<Subagent> { Some(Subagent { identity: SubagentIdentity { id: "id".to_string() } }) }
    pub async fn list_active(&self) -> Vec<Subagent> { vec![] }
}

#[async_trait]
pub trait Sandbox: Send + Sync {
    async fn execute(&self, _a: &str, _b: &str) -> Result<(), String>;
}

pub enum SandboxType {
    Process { cmd: String, args: Vec<String> }
}

pub struct DummySandbox;
#[async_trait]
impl Sandbox for DummySandbox {
    async fn execute(&self, _a: &str, _b: &str) -> Result<(), String> { Ok(()) }
}

pub fn create_sandbox(_type: SandboxType) -> Arc<dyn Sandbox> {
    Arc::new(DummySandbox)
}

pub struct WasiPreview2Sandbox;
impl WasiPreview2Sandbox {
    pub async fn new(_wasm: Vec<u8>) -> Result<Self, String> { Ok(Self) }
    pub async fn execute(&self, _a: &str, _b: &str) -> Result<(), String> { Ok(()) }
}
