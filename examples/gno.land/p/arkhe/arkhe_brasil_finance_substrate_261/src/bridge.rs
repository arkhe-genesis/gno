//! ARKHE-OS Bridge com TemporalChain e ZK
use arkhe_jax_zk::ComputationProof;
use crate::types::PixTransaction;
use sha3::{Sha3_256, Digest};

pub struct ArkheBrasilBridge;

impl ArkheBrasilBridge {
    pub fn new() -> Self { Self }
    pub fn verify_zk_pix(&self, tx: &PixTransaction, proof: &ComputationProof) -> bool {
        let mut hasher = Sha3_256::new();
        hasher.update(tx.end_to_end_id.as_bytes());
        hasher.update(&tx.amount.to_le_bytes());
        hasher.update(tx.receiver_key.key_type.as_bytes());
        hasher.update(tx.receiver_key.key.as_bytes());
        let tx_hash: [u8; 32] = hasher.finalize().into();

        proof.verify(&tx_hash)
    }
}
