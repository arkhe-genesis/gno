use serde::{Deserialize, Serialize};

pub struct WormgraphClient {}

impl WormgraphClient {
    pub async fn append(&self, entry: ProvenanceEntry) -> Result<(), String> {
        Ok(()) // Mock implementation
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub id: String,
    pub version: u64,
    pub decision_type: String,
    pub before_state: String,
    pub after_state: String,
    pub rationale: Option<String>,
    pub timestamp: i64,
    pub agent_id: String,
    pub entry_hash: Vec<u8>,
    pub nostr_event_id: Option<String>,
    pub tree_id: Option<String>,
    pub parent_event_id: Option<String>,
    pub agent_identity: Option<String>,
}
