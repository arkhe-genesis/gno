//! # Hashtree Integration for MCP Arkhe
//!
//! Provides Merkle tree (Hashtree) capabilities for:
//! - File integrity verification
//! - Content addressing via SHA-256
//! - Nostr publication (kind 30078, NIP-78)
//! - Blossom blob storage
//! - Git remote (htree://) integration
//! - NIP-44 encryption for private trees
//!
//! ## Architecture
//!
//! ```text
//! +-------------------------------------------------------------+
//! |                      MCP Arkhe                              |
//! |  +-------------+  +-------------+  +---------------------+ |
//! |  | arkhe-core  |  |  arkhe-mem  |  |   arkhe-mcp         | |
//! |  |  (tensores) |  |  (M0-M4)    |  |  (MCP Bridge)       | |
//! |  +------+------+  +------+------+  +----------+----------+ |
//! |         |                |                     |            |
//! |         +----------------+---------------------+            |
//! |                          |                                  |
//! |                   +------v------+                           |
//! |                   |  hashtree.rs |                           |
//! |                   |  (Merkle)    |                           |
//! |                   +------+------+                           |
//! +--------------------------+----------------------------------+
//!                            |
//!            +---------------+---------------+
//!            |               |               |
//!     +------v------+ +------v------+ +-----v------+
//!     |   Nostr     | |  Blossom    | |    Git     |
//!     |   (30078)   | |  (storage)  | |   (remote) |
//!     +-------------+ +-------------+ +------------+
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;
use nip44::{encrypt, decrypt, validate_key};



use crate::error::{McpError, Result};

// ============================================================================
// HASHTREE TYPES
// ============================================================================

/// A node in the Merkle tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashtreeNode {
    /// SHA-256 hash of this node
    pub hash: String,
    /// Left child (if internal node)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<Box<HashtreeNode>>,
    /// Right child (if internal node)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<Box<HashtreeNode>>,
    /// File content (if leaf node)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
    /// File name (if leaf)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A Hashtree (root of a Merkle tree)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashtree {
    /// Tree name
    pub name: String,
    /// Merkle root hash
    pub root_hash: String,
    /// Root node
    pub root: HashtreeNode,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Encryption status
    pub encrypted: bool,
    /// Nostr event ID (kind 30078)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nostr_event_id: Option<String>,
    /// Decryption key (for private trees)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decryption_key: Option<String>,
}

/// Nostr event for Hashtree (kind 30078, NIP-78)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashtreeEvent {
    /// Event kind (30078)
    pub kind: u64,
    /// Content (encrypted JSON for private trees)
    pub content: String,
    /// Tags
    pub tags: Vec<Vec<String>>,
    /// Created at
    pub created_at: u64,
    /// Pubkey
    pub pubkey: String,
    /// ID
    pub id: String,
    /// Sig
    pub sig: String,
}

/// Blossom blob descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlossomBlob {
    /// SHA-256 hash
    pub sha256: String,
    /// Size in bytes
    pub size: u64,
    /// URL
    pub url: String,
    /// Type
    #[serde(rename = "type")]
    pub mime_type: String,
}

/// Content addressing info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAddress {
    /// SHA-256 hash
    pub hash: String,
    /// Size
    pub size: u64,
    /// Merkle root
    pub merkle_root: String,
    /// Blossom URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blossom_url: Option<String>,
}

// ============================================================================
// HASHTREE CLIENT
// ============================================================================

/// Client for Hashtree protocol operations
#[derive(Debug, Clone)]
pub struct HashtreeClient {
    /// Nostr relay URL
    relay_url: String,
    /// Blossom server URL
    blossom_url: String,
    /// Private key (hex) for Nostr signing
    private_key: Option<String>,
    /// Public key (hex)
    pubkey: Option<String>,
    /// HTTP client
    http: reqwest::Client,
}

impl HashtreeClient {
    /// Create a new Hashtree client
    pub fn new(relay_url: impl Into<String>, blossom_url: impl Into<String>) -> Self {
        Self {
            relay_url: relay_url.into(),
            blossom_url: blossom_url.into(),
            private_key: None,
            pubkey: None,
            http: reqwest::Client::new(),
        }
    }

    /// Authenticate with Nostr private key
    pub fn authenticate(&mut self, private_key_hex: impl Into<String>) -> Result<()> {
        let key = private_key_hex.into();
        // Derive pubkey from private key (simplified)
        self.pubkey = Some(Self::derive_pubkey(&key)?);
        self.private_key = Some(key);
        info!("Hashtree client authenticated");
        Ok(())
    }

    // ========================================================================
    // NIP-44 REAL ENCRYPTION
    // ========================================================================

    /// Encrypt data with NIP-44 (real implementation)
    pub fn nip44_encrypt(data: &str, key: &str) -> Result<String> {
        // Validate the key format (NIP-44 requires 32-byte hex or base64)
        if !validate_key(key) {
            return Err(McpError::Validation {
                field: "nip44_key".into(),
                reason: "Invalid NIP-44 key format".into(),
            });
        }

        // Encrypt with NIP-44
        encrypt(key, data)
            .map_err(|e| McpError::Validation {
                field: "nip44_encrypt".into(),
                reason: e.to_string(),
            })
    }

    /// Decrypt data with NIP-44 (real implementation)
    pub fn nip44_decrypt(data: &str, key: &str) -> Result<String> {
        if !validate_key(key) {
            return Err(McpError::Validation {
                field: "nip44_key".into(),
                reason: "Invalid NIP-44 key format".into(),
            });
        }

        decrypt(key, data)
            .map_err(|e| McpError::Validation {
                field: "nip44_decrypt".into(),
                reason: e.to_string(),
            })
    }

    /// Generate a random NIP-44 key (32 bytes hex)
    pub fn generate_nip44_key() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        hex::encode(bytes)
    }

    // ========================================================================
    // MERKLE TREE OPERATIONS
    // ========================================================================

    /// Build a Merkle tree from file contents
    #[instrument(skip(self, data))]
    pub fn build_tree(&self, name: &str, data: Vec<(String, Vec<u8>)>) -> Result<Hashtree> {
        let mut leaves: Vec<HashtreeNode> = data.into_iter()
            .map(|(filename, bytes)| {
                let hash = Self::sha256(&bytes);
                HashtreeNode {
                    hash: hash.clone(),
                    left: None,
                    right: None,
                    data: Some(bytes),
                    name: Some(filename),
                }
            })
            .collect();

        // Build tree bottom-up
        while leaves.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in leaves.chunks(2) {
                let left = chunk[0].clone();
                let right = chunk.get(1).cloned().unwrap_or_else(|| left.clone());

                let combined_hash = Self::sha256(
                    format!("{}{}", left.hash, right.hash).as_bytes()
                );

                next_level.push(HashtreeNode {
                    hash: combined_hash,
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                    data: None,
                    name: None,
                });
            }

            leaves = next_level;
        }

        let root = leaves.into_iter().next().ok_or_else(||
            McpError::InvalidRequest("Cannot build tree from empty data".into())
        )?;

        Ok(Hashtree {
            name: name.to_string(),
            root_hash: root.hash.clone(),
            root,
            created_at: chrono::Utc::now(),
            encrypted: false,
            nostr_event_id: None,
            decryption_key: None,
        })
    }

    /// Verify a Merkle proof for a specific file
    #[instrument(skip(self, tree, file_hash))]
    pub fn verify_file(&self, tree: &Hashtree, file_hash: &str) -> Result<bool> {
        // Find the leaf node
        let leaf = Self::find_leaf(&tree.root, file_hash)
            .ok_or_else(|| McpError::NotFound(
                format!("File hash {} not found in tree", file_hash)
            ))?;

        // Recompute path to root
        let computed_root = Self::compute_merkle_root(&tree.root, file_hash)?;

        Ok(computed_root == tree.root_hash)
    }

    /// Generate Merkle proof for a file
    pub fn generate_proof(&self, tree: &Hashtree, file_hash: &str) -> Result<MerkleProof> {
        let mut proof = Vec::new();
        Self::collect_proof_path(&tree.root, file_hash, &mut proof)?;

        Ok(MerkleProof {
            root: tree.root_hash.clone(),
            target: file_hash.to_string(),
            path: proof,
        })
    }

    // ========================================================================
    // NOSTR PUBLICATION
    // ========================================================================

    /// Publish tree to Nostr (kind 30078)
    #[instrument(skip(self, tree))]
    pub async fn publish_to_nostr(&self, tree: &Hashtree) -> Result<String> {
        let pubkey = self.pubkey.as_ref().ok_or_else(||
            McpError::Auth("Not authenticated with Nostr key".into())
        )?;

        let event = HashtreeEvent {
            kind: 30078,
            content: if tree.encrypted {
                // Encrypt tree metadata with NIP-44
                Self::nip44_encrypt(&tree.root_hash, tree.decryption_key.as_ref().unwrap())?
            } else {
                serde_json::to_string(&tree).map_err(|e| McpError::Serialization(e))?
            },
            tags: vec![
                vec!["d".to_string(), tree.name.clone()],
                vec!["l".to_string(), "hashtree".to_string()],
                vec!["hash".to_string(), tree.root_hash.clone()],
                vec!["key".to_string(), tree.decryption_key.clone().unwrap_or_default()],
            ],
            created_at: chrono::Utc::now().timestamp() as u64,
            pubkey: pubkey.clone(),
            id: uuid::Uuid::new_v4().to_string(),
            sig: "mock_sig".to_string(), // In production: sign with secp256k1
        };

        // Publish to relay
        let response = self.http
            .post(&self.relay_url)
            .json(&event)
            .send()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!("Failed to publish: {}", response.status())));
        }

        info!("Published tree '{}' to Nostr (kind 30078)", tree.name);
        Ok(event.id)
    }

    /// Fetch tree from Nostr by name
    #[instrument(skip(self), fields(name = %name))]
    pub async fn fetch_from_nostr(&self, name: &str) -> Result<Hashtree> {
        let filter = serde_json::json!({
            "kinds": [30078],
            "authors": [self.pubkey.as_ref().unwrap_or(&"".to_string())],
            "#d": [name],
        });

        let response = self.http
            .post(&format!("{}/req", self.relay_url))
            .json(&filter)
            .send()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        let events: Vec<HashtreeEvent> = response.json().await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        let event = events.into_iter().next().ok_or_else(||
            McpError::NotFound(format!("Tree '{}' not found on Nostr", name))
        )?;

        // Parse tree from event
        let tree: Hashtree = if event.content.starts_with('{') {
            serde_json::from_str(&event.content).map_err(|e| McpError::Serialization(e))?
        } else {
            // Decrypt NIP-44
            let decrypted = Self::nip44_decrypt(&event.content, "key")?;
            serde_json::from_str(&decrypted).map_err(|e| McpError::Serialization(e))?
        };

        Ok(tree)
    }

    // ========================================================================
    // BLOSSOM STORAGE
    // ========================================================================

    /// Upload blob to Blossom server
    #[instrument(skip(self, data))]
    pub async fn upload_blossom(&self, data: Vec<u8>) -> Result<BlossomBlob> {
        let hash = Self::sha256(&data);

        let response = self.http
            .post(&format!("{}/upload", self.blossom_url))
            .body(data)
            .send()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        let blob: BlossomBlob = response.json().await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        info!("Uploaded blob to Blossom: {} ({} bytes)", hash, blob.size);
        Ok(blob)
    }

    /// Download blob from Blossom by hash
    #[instrument(skip(self), fields(hash = %hash))]
    pub async fn download_blossom(&self, hash: &str) -> Result<Vec<u8>> {
        let response = self.http
            .get(&format!("{}/{}", self.blossom_url, hash))
            .send()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        let data = response.bytes().await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        // Verify hash
        let computed_hash = Self::sha256(&data);
        if computed_hash != hash {
            return Err(McpError::Validation {
                field: "hash".into(),
                reason: format!("Hash mismatch: expected {}, got {}", hash, computed_hash),
            });
        }

        Ok(data.to_vec())
    }

    // ========================================================================
    // GIT INTEGRATION
    // ========================================================================

    /// Push git repository to htree:// remote
    #[instrument(skip(self, repo_path))]
    pub async fn git_push_htree(&self, repo_path: &Path, tree_name: &str) -> Result<String> {
        // Read all files in repo
        let mut files = Vec::new();
        Self::collect_files(repo_path, repo_path, &mut files)?;

        // Build tree
        let tree = self.build_tree(tree_name, files)?;

        // Publish to Nostr
        let event_id = self.publish_to_nostr(&tree).await?;

        info!("Pushed git repo to htree://{}/{} (event: {})",
            self.pubkey.as_ref().unwrap_or(&"".to_string()),
            tree_name,
            event_id
        );

        Ok(format!("htree://{}/{}",
            self.pubkey.as_ref().unwrap_or(&"".to_string()),
            tree_name
        ))
    }

    /// Clone from htree:// URL
    #[instrument(skip(self, url))]
    pub async fn git_clone_htree(&self, url: &str, dest: &Path) -> Result<()> {
        // Parse htree://pubkey/tree-name
        let parts: Vec<&str> = url.trim_start_matches("htree://").split('/').collect();
        if parts.len() != 2 {
            return Err(McpError::InvalidRequest(
                format!("Invalid htree URL: {}", url)
            ));
        }

        let tree_name = parts[1];

        // Fetch tree from Nostr
        let tree = self.fetch_from_nostr(tree_name).await?;

        // Write files to disk
        Self::write_tree(&tree.root, dest)?;

        info!("Cloned {} to {}", url, dest.display());
        Ok(())
    }

    // ========================================================================
    // ENCRYPTION (NIP-44)
    // ========================================================================

    /// Encrypt tree with NIP-44
    pub fn encrypt_tree(&self, tree: &mut Hashtree, key: &str) -> Result<()> {
        tree.encrypted = true;
        tree.decryption_key = Some(key.to_string());

        // Encrypt leaf data
        Self::encrypt_node(&mut tree.root, key)?;

        info!("Tree '{}' encrypted with NIP-44", tree.name);
        Ok(())
    }

    /// Decrypt tree
    pub fn decrypt_tree(&self, tree: &mut Hashtree, key: &str) -> Result<()> {
        if !tree.encrypted {
            return Ok(());
        }

        Self::decrypt_node(&mut tree.root, key)?;
        tree.encrypted = false;
        tree.decryption_key = None;

        info!("Tree '{}' decrypted", tree.name);
        Ok(())
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    pub fn sha256(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn derive_pubkey(_private_key: &str) -> Result<String> {
        // In production: secp256k1 public key derivation
        Ok("mock_pubkey".to_string())
    }

    fn find_leaf<'a>(node: &'a HashtreeNode, hash: &str) -> Option<&'a HashtreeNode> {
        if node.hash == hash {
            return Some(node);
        }

        if let Some(ref left) = node.left {
            if let Some(found) = Self::find_leaf(left, hash) {
                return Some(found);
            }
        }

        if let Some(ref right) = node.right {
            if let Some(found) = Self::find_leaf(right, hash) {
                return Some(found);
            }
        }

        None
    }

    fn compute_merkle_root(node: &HashtreeNode, target_hash: &str) -> Result<String> {
        if node.hash == target_hash {
            return Ok(node.hash.clone());
        }

        if let Some(ref left) = node.left {
            if Self::find_leaf(left, target_hash).is_some() {
                let left_hash = Self::compute_merkle_root(left, target_hash)?;
                let right_hash = node.right.as_ref().map(|r| r.hash.clone())
                    .unwrap_or_else(|| left_hash.clone());
                return Ok(Self::sha256(format!("{}{}", left_hash, right_hash).as_bytes()));
            }
        }

        if let Some(ref right) = node.right {
            if Self::find_leaf(right, target_hash).is_some() {
                let left_hash = node.left.as_ref().map(|l| l.hash.clone())
                    .unwrap_or_else(|| right.hash.clone());
                let right_hash = Self::compute_merkle_root(right, target_hash)?;
                return Ok(Self::sha256(format!("{}{}", left_hash, right_hash).as_bytes()));
            }
        }

        Err(McpError::NotFound("Hash not in tree".into()))
    }

    fn collect_proof_path(
        node: &HashtreeNode,
        target_hash: &str,
        proof: &mut Vec<ProofNode>,
    ) -> Result<bool> {
        if node.hash == target_hash {
            return Ok(true);
        }

        if let Some(ref left) = node.left {
            if Self::collect_proof_path(left, target_hash, proof)? {
                proof.push(ProofNode {
                    hash: node.right.as_ref().map(|r| r.hash.clone())
                        .unwrap_or_else(|| left.hash.clone()),
                    side: ProofSide::Right,
                });
                return Ok(true);
            }
        }

        if let Some(ref right) = node.right {
            if Self::collect_proof_path(right, target_hash, proof)? {
                proof.push(ProofNode {
                    hash: node.left.as_ref().map(|l| l.hash.clone())
                        .unwrap_or_else(|| right.hash.clone()),
                    side: ProofSide::Left,
                });
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn collect_files(base: &Path, current: &Path, files: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
        for entry in std::fs::read_dir(current).map_err(|e| McpError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| McpError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                let rel_path = path.strip_prefix(base)
                    .map_err(|e| McpError::IoError(e.to_string()))?;
                let content = std::fs::read(&path).map_err(|e| McpError::IoError(e.to_string()))?;
                files.push((rel_path.to_string_lossy().to_string(), content));
            } else if path.is_dir() {
                Self::collect_files(base, &path, files)?;
            }
        }

        Ok(())
    }

    fn write_tree(node: &HashtreeNode, dest: &Path) -> Result<()> {
        if let Some(ref data) = node.data {
            if let Some(ref name) = node.name {
                let path = dest.join(name);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| McpError::IoError(e.to_string()))?;
                }
                std::fs::write(&path, data).map_err(|e| McpError::IoError(e.to_string()))?;
            }
        }

        if let Some(ref left) = node.left {
            Self::write_tree(left, dest)?;
        }
        if let Some(ref right) = node.right {
            Self::write_tree(right, dest)?;
        }

        Ok(())
    }

    fn encrypt_node(node: &mut HashtreeNode, key: &str) -> Result<()> {
        if let Some(ref mut data) = node.data {
            use base64::{Engine as _, engine::general_purpose};
            let b64 = general_purpose::STANDARD.encode(&*data);
            *data = Self::nip44_encrypt(&b64, key)?.into_bytes();
        }

        if let Some(ref mut left) = node.left {
            Self::encrypt_node(left, key)?;
        }
        if let Some(ref mut right) = node.right {
            Self::encrypt_node(right, key)?;
        }

        Ok(())
    }

    fn decrypt_node(node: &mut HashtreeNode, key: &str) -> Result<()> {
        if let Some(ref mut data) = node.data {
            let decrypted = Self::nip44_decrypt(&String::from_utf8_lossy(data), key)?;
            use base64::{Engine as _, engine::general_purpose};
            *data = general_purpose::STANDARD.decode(decrypted).map_err(|e| McpError::Validation {
                field: "base64".into(),
                reason: e.to_string(),
            })?;
        }

        if let Some(ref mut left) = node.left {
            Self::decrypt_node(left, key)?;
        }
        if let Some(ref mut right) = node.right {
            Self::decrypt_node(right, key)?;
        }

        Ok(())
    }
}

// ============================================================================
// MERKLE PROOF
// ============================================================================

/// A Merkle proof path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub root: String,
    pub target: String,
    pub path: Vec<ProofNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofNode {
    pub hash: String,
    pub side: ProofSide,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofSide {
    Left,
    Right,
}

impl MerkleProof {
    /// Verify this proof against a root hash
    pub fn verify(&self, root_hash: &str) -> bool {
        let mut current = self.target.clone();

        for node in &self.path {
            current = match node.side {
                ProofSide::Left => {
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(format!("{}{}", node.hash, current));
                    format!("{:x}", hasher.finalize())
                }
                ProofSide::Right => {
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(format!("{}{}", current, node.hash));
                    format!("{:x}", hasher.finalize())
                }
            };
        }

        current == root_hash
    }
}

// ============================================================================
// MCP TOOL WRAPPERS
// ============================================================================

/// MCP tool wrappers for Hashtree operations
#[derive(Debug, Clone)]
pub struct HashtreeMcpTools {
    pub client: HashtreeClient,
}

impl HashtreeMcpTools {
    pub fn new(client: HashtreeClient) -> Self {
        Self { client }
    }

    /// Tool: Publish file to Hashtree
    pub async fn hashtree_publish(
        &self,
        name: &str,
        files: Vec<(String, Vec<u8>)>,
    ) -> Result<Hashtree> {
        let tree = self.client.build_tree(name, files)?;
        self.client.publish_to_nostr(&tree).await?;
        Ok(tree)
    }

    /// Tool: Fetch file by hash
    pub async fn hashtree_fetch(&self, hash: &str) -> Result<Vec<u8>> {
        self.client.download_blossom(hash).await
    }

    /// Tool: List user's trees
    pub async fn hashtree_list(&self) -> Result<Vec<String>> {
        // Query Nostr for kind 30078 events
        Ok(vec![]) // Placeholder
    }

    /// Tool: Verify Merkle proof
    pub fn hashtree_verify(&self, proof: &MerkleProof, root_hash: &str) -> bool {
        proof.verify(root_hash)
    }

    /// Tool: Push git repo to htree
    pub async fn hashtree_git_push(&self, repo_path: &Path, tree_name: &str) -> Result<String> {
        self.client.git_push_htree(repo_path, tree_name).await
    }

    /// Tool: Clone from htree
    pub async fn hashtree_git_clone(&self, url: &str, dest: &Path) -> Result<()> {
        self.client.git_clone_htree(url, dest).await
    }
}
