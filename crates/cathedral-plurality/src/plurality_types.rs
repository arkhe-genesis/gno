//! Tipos para Plurality Network
//! Selo: CATHEDRAL-ARKHE-PLURALITY-TYPES-v1.0.0-2026-06-21

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBucket {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub bucket_type: BucketType,
    pub ttl_seconds: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub size_bytes: u64,
    pub item_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BucketType {
    M0,  // Working Memory (RAM)
    M1,  // Short-term Cache
    M2,  // Long-term Memory
    M3,  // Shared Memory
    M4,  // Public Memory (Nostr)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: Vec<u8>,
    pub bucket: BucketType,
    pub ttl_seconds: u64,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub vector: Option<Vec<f32>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItemInput {
    pub key: String,
    pub value: Vec<u8>,
    pub bucket: BucketType,
    pub ttl_seconds: u64,
    pub vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub vector: Vec<f32>,
    pub bucket: BucketType,
    pub limit: u32,
    pub min_similarity: f32,
    pub filter: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub items: Vec<MemoryItem>,
    pub total: usize,
    pub took_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartProfile {
    pub id: String,
    pub agent_id: String,
    pub preferences: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub context: HashMap<String, serde_json::Value>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartProfileInput {
    pub agent_id: String,
    pub preferences: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub context: HashMap<String, serde_json::Value>,
}

impl BucketType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BucketType::M0 => "M0",
            BucketType::M1 => "M1",
            BucketType::M2 => "M2",
            BucketType::M3 => "M3",
            BucketType::M4 => "M4",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "M0" => Some(BucketType::M0),
            "M1" => Some(BucketType::M1),
            "M2" => Some(BucketType::M2),
            "M3" => Some(BucketType::M3),
            "M4" => Some(BucketType::M4),
            _ => None,
        }
    }
}

impl Default for BucketType {
    fn default() -> Self {
        BucketType::M2
    }
}
