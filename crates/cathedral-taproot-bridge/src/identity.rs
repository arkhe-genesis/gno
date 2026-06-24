// src/identity.rs
// Stub for did implementation as we don't have cathedral_identity crate available directly
// in our standalone test environment right now
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Did(String);

impl Did {
    pub fn from_string(s: &str) -> Self {
        Did(s.to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// Representação de um AssetRef do Taproot Assets.
/// Pode ser um asset_id (UUID) ou group_key (hex).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AssetRef {
    AssetId(String),    // UUID
    GroupKey(String),   // Hex string
}

impl AssetRef {
    pub fn from_string(s: &str) -> Self {
        if s.starts_with("group_key_") {
            AssetRef::GroupKey(s.trim_start_matches("group_key_").to_string())
        } else {
            AssetRef::AssetId(s.to_string())
        }
    }

    pub fn to_did(&self) -> Did {
        match self {
            AssetRef::AssetId(id) => Did::from_string(&format!("did:cathedral:asset:{}", id)),
            AssetRef::GroupKey(key) => Did::from_string(&format!("did:cathedral:group:{}", key)),
        }
    }

    pub fn from_did(did: &Did) -> Option<Self> {
        let s = did.to_string();
        if let Some(id) = s.strip_prefix("did:cathedral:asset:") {
            Some(AssetRef::AssetId(id.to_string()))
        } else if let Some(key) = s.strip_prefix("did:cathedral:group:") {
            Some(AssetRef::GroupKey(key.to_string()))
        } else {
            None
        }
    }
}
