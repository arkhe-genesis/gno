use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BridgeState {
    pub verification_keys: Arc<RwLock<HashMap<String, VerificationKey>>>,
    pub wormgraph: Arc<cathedral_wormgraph::WormgraphClient>,
    pub nostr_replicator: Option<Arc<cathedral_nostr::NostrReplicator>>,
}

#[derive(Clone)]
pub struct VerificationKey {
    pub hash: Vec<u8>,
    pub elf: Vec<u8>,
}
