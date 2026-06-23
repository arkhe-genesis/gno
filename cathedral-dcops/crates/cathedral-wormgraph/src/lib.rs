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

#[derive(Debug)]
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
