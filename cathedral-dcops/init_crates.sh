#!/bin/bash
CRATES=(
    "cathedral-dcops-core"
    "cathedral-agent-monitoring"
    "cathedral-agent-capacity"
    "cathedral-agent-energy"
    "cathedral-agent-security"
    "cathedral-agent-compliance"
    "cathedral-dcops-runtime"
    "cathedral-identity"
    "cathedral-wormgraph"
    "cathedral-zk"
    "cathedral-arkheobex"
    "cathedral-observer"
    "cathedral-resilience"
    "cathedral-api"
    "cathedral-cli"
    "cathedral-self-improve"
    "cathedral-test-support"
    "cathedral-xtask"
)

for crate in "${CRATES[@]}"; do
    cat << TOML > "crates/$crate/Cargo.toml"
[package]
name = "$crate"
version = "1.0.0"
edition = "2021"

[dependencies]
TOML

    # Se for cli, xtask, ou api, teremos um main.rs e/ou lib.rs
    if [[ "$crate" == "cathedral-cli" || "$crate" == "cathedral-xtask" ]]; then
        cat << RUST > "crates/$crate/src/main.rs"
fn main() {
    println!("Hello from $crate!");
}
RUST
        if [[ "$crate" == "cathedral-cli" ]]; then
             cat << RUST > "crates/$crate/src/lib.rs"
pub mod commands;
RUST
        fi
    elif [[ "$crate" == "cathedral-api" ]]; then
        cat << RUST > "crates/$crate/src/lib.rs"
pub mod extractors;
pub mod middleware;
pub mod ws;

pub struct AppState {
    pub wormgraph: std::sync::Arc<cathedral_wormgraph::WormGraphClient>,
    pub notifier: std::sync::Arc<cathedral_self_improve::BroadcastNotifier>,
}

pub enum ApiError {
    MissingDid,
    MissingSignature,
    MissingId,
    NotFound,
    Forbidden,
    InvalidPayload,
    AuthFailed,
}

pub mod auth {
    pub async fn verify_auth(_did: &str, _signature: &[u8], _payload: &[u8]) -> Result<bool, super::ApiError> {
        Ok(true)
    }
}

pub struct ProposalQueryParams {
    pub id: Option<String>,
}
RUST
        cat << RUST > "crates/$crate/src/main.rs"
fn main() {
    println!("Cathedral API");
}
RUST

        # Adicionar dependencias basicas pro cathedral-api compilar o codigo injetado
        cat << TOML >> "crates/$crate/Cargo.toml"
axum.workspace = true
serde.workspace = true
hex.workspace = true
cathedral-wormgraph.workspace = true
cathedral-self-improve.workspace = true
TOML

    elif [[ "$crate" == "cathedral-self-improve" ]]; then
        cat << RUST > "crates/$crate/src/lib.rs"
pub mod architect;
pub mod orchestrator;

pub struct BroadcastNotifier;
impl BroadcastNotifier {
    pub async fn broadcast(&self, _proposal: cathedral_wormgraph::ImprovementProposal) {}
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<String> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
}
RUST
        cat << TOML >> "crates/$crate/Cargo.toml"
cathedral-wormgraph.workspace = true
syn.workspace = true
serde_json.workspace = true
walkdir.workspace = true
tokio.workspace = true
TOML
    elif [[ "$crate" == "cathedral-wormgraph" ]]; then
         cat << RUST > "crates/$crate/src/lib.rs"
pub mod backends;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImprovementProposal {
    pub id: String,
    pub author_did: String,
    pub title: String,
    pub description: String,
    pub code_diff: Option<String>,
    pub expected_impact: String,
    pub risk_level: RiskLevel,
    pub validation_status: ValidationStatus,
    pub signature: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    // outros campos
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel { Low, Medium, High, Critical }

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ValidationStatus { Pending, Validating, Approved, Rejected, Implemented, Reverted }

#[derive(Clone, Debug)]
pub struct ProposalFilter {
    pub risk_level: Option<RiskLevel>,
    pub status: Option<ValidationStatus>,
    pub author_did: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct LedgerEntry {
    pub id: String,
    pub timestamp: i64,
    pub agent_id: String,
}

#[derive(Clone, Debug)]
pub struct MemoryFilter {
    pub agent_id: Option<String>,
    pub decision_type: Option<String>,
    pub since_timestamp: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub enum WormGraphError {
    Forbidden,
    NotFound,
    DbError,
}

pub type Result<T> = std::result::Result<T, WormGraphError>;

impl ImprovementProposal {
    pub fn new(title: String, description: String) -> Self {
        Self {
            id: String::new(),
            author_did: String::new(),
            title,
            description,
            code_diff: None,
            expected_impact: String::new(),
            risk_level: RiskLevel::Low,
            validation_status: ValidationStatus::Pending,
            signature: vec![],
            created_at: chrono::Utc::now(),
        }
    }
    pub fn with_risk(mut self, risk: RiskLevel) -> Self { self.risk_level = risk; self }
    pub fn with_code_diff(mut self, diff: String) -> Self { self.code_diff = Some(diff); self }
    pub fn with_impact(mut self, impact: String) -> Self { self.expected_impact = impact; self }
}

#[async_trait::async_trait]
pub trait WormGraphBackend: Send + Sync {
    async fn list_memories(&self, filter: MemoryFilter) -> Result<Vec<LedgerEntry>>;
    async fn list_proposals(&self, filter: ProposalFilter) -> Result<Vec<ImprovementProposal>>;
}

pub struct WormGraphClient;
impl WormGraphClient {
    pub async fn get_proposal(&self, _id: &str) -> Result<Option<ImprovementProposal>> { Ok(None) }
}
RUST
         cat << TOML >> "crates/$crate/Cargo.toml"
sqlx.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
dashmap.workspace = true
async-trait.workspace = true
TOML
    elif [[ "$crate" == "cathedral-test-support" ]]; then
         cat << RUST > "crates/$crate/src/lib.rs"
pub mod wormgraph;
RUST
         cat << TOML >> "crates/$crate/Cargo.toml"
cathedral-wormgraph.workspace = true
dashmap.workspace = true
uuid.workspace = true
chrono.workspace = true
ed25519-dalek.workspace = true
hex.workspace = true
rand.workspace = true
async-trait.workspace = true
TOML
    elif [[ "$crate" == "cathedral-identity" ]]; then
         cat << RUST > "crates/$crate/src/lib.rs"
pub mod pqc;
RUST
         cat << TOML >> "crates/$crate/Cargo.toml"
ed25519-dalek.workspace = true
rand.workspace = true
serde.workspace = true
thiserror.workspace = true
pqcrypto-dilithium = { workspace = true, optional = true }

[features]
mldsa = ["dep:pqcrypto-dilithium"]
TOML
    elif [[ "$crate" == "cathedral-cli" ]]; then
         cat << TOML >> "crates/$crate/Cargo.toml"
clap.workspace = true
hex.workspace = true
base64.workspace = true
ed25519-dalek.workspace = true
rand.workspace = true
pqcrypto-dilithium = { workspace = true, optional = true }

[features]
mldsa = ["dep:pqcrypto-dilithium"]
TOML
    else
        cat << RUST > "crates/$crate/src/lib.rs"
// Stub for $crate
RUST
    fi
done
