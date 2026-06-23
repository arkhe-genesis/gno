pub mod wormgraph;
pub mod fixtures {}
pub mod auth {}

use cathedral_wormgraph::{ImprovementProposal, RiskLevel, ValidationStatus};
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;
use hex;
use std::convert::TryInto;

pub struct TestKeys {
    pub did: String,
    pub signing_key: SigningKey,
}

impl TestKeys {
    pub fn new(did: &str) -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self {
            did: did.to_string(),
            signing_key,
        }
    }

    pub fn sign(&self, payload: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(payload);
        signature.to_bytes().to_vec()
    }

    pub fn auth_header(&self, payload: &[u8]) -> String {
        let sig_hex = hex::encode(self.sign(payload));
        format!("DID {}; Signature {}", self.did, sig_hex)
    }
}

pub fn create_test_proposal(author_did: &str, title: &str) -> ImprovementProposal {
    ImprovementProposal {
        id: uuid::Uuid::new_v4().to_string(),
        title: title.to_string(),
        description: format!("Descrição de {}", title),
        code_diff: None,
        expected_impact: "Impacto médio".to_string(),
        risk_level: RiskLevel::Medium,
        validation_status: ValidationStatus::Pending,
        author_did: author_did.to_string(),
        signature: vec![],
        created_at: chrono::Utc::now(),
    }
}

pub fn populate_test_wormgraph(wg: &wormgraph::TestWormGraph, count: usize, author_did: &str) -> Result<(), String> {
    wg.populate_with_proposals(count, author_did)
        .map_err(|e| "error".to_string())
}
