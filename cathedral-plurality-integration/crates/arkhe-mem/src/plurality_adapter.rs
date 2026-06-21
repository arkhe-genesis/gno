//! # Plurality Memory Adapter
//!
//! Adapter que mapeia Memory Buckets da Plurality para os níveis M0-M4
//! do sistema de memória hierárquica da Cathedral ARKHE.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use arkhe_mcp::{
    HashtreeClient, MerkleProof, McpError, Result
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLevel {
    M0, M1, M2, M3, M4
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub id: String,
    pub level: MemoryLevel,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct PluralityMemoryAdapter {
    pub cache: Arc<RwLock<LruCache<String, MemoryBlock>>>,
}

impl PluralityMemoryAdapter {
    // ========================================================================
    // HASHTREE VERIFICATION ON READ
    // ========================================================================

    pub async fn read(&self, key: &str, level: MemoryLevel) -> Result<Option<MemoryBlock>> {
        let cache = self.cache.read().await;
        Ok(cache.get(key).cloned())
    }

    pub async fn write(&self, key: &str, content: &[u8], level: MemoryLevel, _tags: Option<Vec<String>>) -> Result<MemoryItem> {
        let mut cache = self.cache.write().await;
        cache.put(key.to_string(), MemoryBlock { id: key.to_string(), level, content: content.to_vec() });
        Ok(MemoryItem { id: key.to_string() })
    }

    /// Read from memory with Hashtree integrity verification
    #[instrument(skip(self, hashtree_client), fields(key = %key, level = ?level))]
    pub async fn read_with_hashtree_verification(
        &self,
        key: &str,
        level: MemoryLevel,
        hashtree_client: &HashtreeClient,
    ) -> Result<Option<MemoryBlock>> {
        let block = self.read(key, level).await?;

        if let Some(ref block) = block {
            // Compute hash of the content
            let content_hash = HashtreeClient::sha256(&block.content);

            // Fetch the tree for this level from Nostr
            let tree_name = format!("memory_{:?}", level);
            match hashtree_client.fetch_from_nostr(&tree_name).await {
                Ok(tree) => {
                    // Verify the file integrity
                    if !hashtree_client.verify_file(&tree, &content_hash)? {
                        return Err(McpError::Validation {
                            field: "integrity".into(),
                            reason: format!("Hash mismatch for block '{}'", key),
                        }.into());
                    }
                    debug!("Hashtree verification passed for block: {}", key);
                }
                Err(e) => {
                    // Tree not found — maybe not published yet
                    warn!("No Hashtree found for {}: {}", tree_name, e);
                }
            }
        }

        Ok(block)
    }

    /// Write to memory and optionally publish to Hashtree
    #[instrument(skip(self, hashtree_client, content), fields(key = %key, level = ?level))]
    pub async fn write_with_hashtree_publish(
        &self,
        key: &str,
        content: &[u8],
        level: MemoryLevel,
        hashtree_client: &HashtreeClient,
        tags: Option<Vec<String>>,
    ) -> Result<MemoryItem> {
        // First, write to Plurality
        let item = self.write(key, content, level, tags).await?;

        // Then, publish to Hashtree for integrity verification
        let tree_name = format!("memory_{:?}", level);
        let files = vec![(key.to_string(), content.to_vec())];

        match hashtree_client.build_tree(&tree_name, files) {
            Ok(tree) => {
                if let Err(e) = hashtree_client.publish_to_nostr(&tree).await {
                    warn!("Failed to publish to Hashtree: {}", e);
                } else {
                    info!("Published memory block to Hashtree: {} → {}", key, tree.root_hash);
                }
            }
            Err(e) => {
                warn!("Failed to build Hashtree: {}", e);
            }
        }

        Ok(item)
    }
}
