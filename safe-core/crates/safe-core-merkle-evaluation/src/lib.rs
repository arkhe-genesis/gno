//! Safe-Core Merkle Tree — Implementação otimizada
//!
//! # Design
//! - Árvore binária completa (completada com hashing de zeros)
//! - Hashing com BLAKE3 (mais rápido que SHA-256)
//! - Proofs compactos (apenas irmãos no caminho)
//! - Suporte a snapshots incrementais

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erros da Merkle Tree
#[derive(Debug, Error)]
pub enum MerkleError {
    #[error("Invalid leaf index: {0}")]
    InvalidIndex(usize),
    #[error("Tree is empty")]
    EmptyTree,
    #[error("Proof verification failed")]
    VerificationFailed,
}

/// Nó da Merkle Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

/// Merkle Tree otimizada para Safe-Core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeMerkleTree {
    root: Option<MerkleNode>,
    leaves: Vec<[u8; 32]>,
}

impl SafeMerkleTree {
    /// Cria uma nova árvore a partir de leaves.
    pub fn from_leaves(leaves: &[[u8; 32]]) -> Self {
        if leaves.is_empty() {
            return Self { root: None, leaves: vec![] };
        }

        let mut current_level: Vec<[u8; 32]> = leaves.to_vec();

        // Completar para potência de 2
        let next_pow2 = current_level.len().next_power_of_two();
        while current_level.len() < next_pow2 {
            current_level.push([0u8; 32]);
        }

        // Construir árvore de baixo para cima
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let hash = Self::hash_pair(&chunk[0], &chunk[1]);
                next_level.push(hash);
            }
            current_level = next_level;
        }

        Self {
            root: Some(MerkleNode {
                hash: current_level[0],
                left: None,
                right: None,
            }),
            leaves: leaves.to_vec(),
        }
    }

    /// Retorna a raiz da árvore.
    pub fn root(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|n| n.hash)
    }

    /// Gera proof para uma leaf.
    pub fn proof(&self, index: usize) -> Result<MerkleProof, MerkleError> {
        if index >= self.leaves.len() {
            return Err(MerkleError::InvalidIndex(index));
        }

        let mut proof = Vec::new();
        let mut current_idx = index;
        let mut level_size = self.leaves.len().next_power_of_two();
        let mut level: Vec<[u8; 32]> = self.leaves.clone();

        while level_size > 1 {
            let sibling_idx = if current_idx % 2 == 0 { current_idx + 1 } else { current_idx - 1 };
            if sibling_idx < level.len() {
                proof.push(level[sibling_idx]);
            } else {
                proof.push([0u8; 32]);
            }

            // Próximo nível
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
            }

            level = next_level;
            current_idx /= 2;
            level_size /= 2;
        }

        Ok(MerkleProof { siblings: proof, leaf_index: index })
    }

    /// Verifica uma proof.
    pub fn verify_proof(&self, leaf: &[u8; 32], proof: &MerkleProof) -> Result<bool, MerkleError> {
        let root = self.root().ok_or(MerkleError::EmptyTree)?;
        let mut current_hash = *leaf;
        let mut index = proof.leaf_index;

        for sibling in &proof.siblings {
            current_hash = if index % 2 == 0 {
                Self::hash_pair(&current_hash, sibling)
            } else {
                Self::hash_pair(sibling, &current_hash)
            };
            index /= 2;
        }

        Ok(current_hash == root)
    }

    /// Hash de um par de nós (ordem importa).
    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}

/// Proof de Merkle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub leaf_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut hash = [0u8; 32];
                hash[0] = i as u8;
                hash
            })
            .collect();

        let tree = SafeMerkleTree::from_leaves(&leaves);
        assert!(tree.root().is_some());

        let proof = tree.proof(2).unwrap();
        assert!(tree.verify_proof(&leaves[2], &proof).unwrap());
    }

    #[test]
    fn test_invalid_proof() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut hash = [0u8; 32];
                hash[0] = i as u8;
                hash
            })
            .collect();

        let tree = SafeMerkleTree::from_leaves(&leaves);
        let mut fake_leaf = [0u8; 32];
        fake_leaf[0] = 99;

        let proof = tree.proof(2).unwrap();
        assert!(!tree.verify_proof(&fake_leaf, &proof).unwrap());
    }
}
