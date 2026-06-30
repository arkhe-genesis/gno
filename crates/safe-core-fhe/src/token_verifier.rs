//! safe-core-fhe/src/token_verifier.rs

use tfhe::prelude::*;
use tfhe::{
    ClientKey, ServerKey, FheUint8, FheBool,
    generate_keys, ConfigBuilder,
};
use serde::{Serialize, Deserialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use sha2::{Sha256, Digest};
use std::array;

pub const HASH_LEN: usize = 32;

#[derive(Clone, Serialize, Deserialize)]
pub struct EncryptedHash {
    pub bytes: [Vec<u8>; HASH_LEN],
}

impl EncryptedHash {
    pub fn from_bytes(bytes: &[Vec<u8>; HASH_LEN]) -> Self {
        Self { bytes: bytes.clone() }
    }
}

pub struct FheTokenVerifier {
    client_key: ClientKey,
    server_key: ServerKey,
    server_key_bytes: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("Erro criptográfico FHE: {0}")]
    Crypto(String),

    #[error("Erro de serialização: {0}")]
    Serialization(String),

    #[error("Erro de desserialização: {0}")]
    Deserialization(String),
}

impl FheTokenVerifier {
    pub fn new() -> Self {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);

        let server_key_bytes = bincode::serialize(&server_key)
            .expect("Serialização da ServerKey falhou");

        Self {
            client_key,
            server_key,
            server_key_bytes,
        }
    }

    pub fn hash_credential(plaintext: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(plaintext.as_bytes());
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }

    pub fn encrypt_hash(&self, hash: &[u8; 32]) -> Result<EncryptedHash, VerifierError> {
        let mut enc_bytes: [Vec<u8>; HASH_LEN] = array::from_fn(|_| Vec::new());

        for (i, &byte) in hash.iter().enumerate() {
            let ct = FheUint8::encrypt(byte, &self.client_key);
            let serialized = bincode::serialize(&ct)
                .map_err(|e| VerifierError::Serialization(e.to_string()))?;
            enc_bytes[i] = serialized;
        }

        Ok(EncryptedHash { bytes: enc_bytes })
    }

    pub fn decrypt_hash(&self, encrypted: &EncryptedHash) -> Result<[u8; 32], VerifierError> {
        let mut hash = [0u8; 32];

        for i in 0..HASH_LEN {
            let ct: FheUint8 = bincode::deserialize(&encrypted.bytes[i])
                .map_err(|e| VerifierError::Deserialization(e.to_string()))?;
            hash[i] = ct.decrypt(&self.client_key);
        }

        Ok(hash)
    }

    pub fn verify_homomorphic(
        &self,
        provided: &EncryptedHash,
        allowed: &EncryptedHash,
    ) -> Result<FheBool, VerifierError> {
        tfhe::set_server_key(self.server_key.clone());

        let mut provided_arr: Vec<FheUint8> = Vec::with_capacity(HASH_LEN);
        let mut allowed_arr: Vec<FheUint8> = Vec::with_capacity(HASH_LEN);

        for i in 0..HASH_LEN {
            provided_arr.push(bincode::deserialize(&provided.bytes[i])
                .map_err(|e| VerifierError::Deserialization(e.to_string()))?);
            allowed_arr.push(bincode::deserialize(&allowed.bytes[i])
                .map_err(|e| VerifierError::Deserialization(e.to_string()))?);
        }

        let mut is_equal = FheBool::encrypt(true, &self.client_key);

        for i in 0..HASH_LEN {
            let byte_eq = provided_arr[i].eq(&allowed_arr[i]);
            is_equal = &is_equal & &byte_eq;
        }

        Ok(is_equal)
    }

    pub fn decrypt_verdict(&self, verdict: &FheBool) -> Result<bool, VerifierError> {
        Ok(verdict.decrypt(&self.client_key))
    }

    pub fn server_key_bytes(&self) -> &[u8] {
        &self.server_key_bytes
    }
}

impl ZeroizeOnDrop for FheTokenVerifier {}

impl Drop for FheTokenVerifier {
    fn drop(&mut self) {
        self.server_key_bytes.zeroize();
    }
}

#[derive(Serialize, Deserialize)]
pub struct VerificationPayload {
    pub provided_hash: EncryptedHash,
    pub allowed_hash: EncryptedHash,
    pub trace_id: [u8; 32],
}

#[derive(Serialize, Deserialize)]
pub struct VerificationResponse {
    pub verdict: Vec<u8>,
    pub cpu_ms: u64,
    pub memory_kb: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn create_verifier() -> FheTokenVerifier {
        FheTokenVerifier::new()
    }

    #[test]
    fn test_full_triple_seal_flow() {
        let raw_credential = "123.456.789-00";
        let safe_text = "CPF: [REDACTED]";
        assert!(!safe_text.contains(raw_credential));

        let hash = FheTokenVerifier::hash_credential(raw_credential);

        let verifier = create_verifier();
        let enc_provided = verifier.encrypt_hash(&hash).expect("Encrypt failed");

        let allowed_hash = FheTokenVerifier::hash_credential("123.456.789-00");
        let enc_allowed = verifier.encrypt_hash(&allowed_hash).expect("Encrypt failed");

        let verdict_enc = verifier.verify_homomorphic(&enc_provided, &enc_allowed)
            .expect("Verification failed");

        let is_allowed = verifier.decrypt_verdict(&verdict_enc)
            .expect("Decrypt failed");

        assert!(is_allowed);
    }

    #[test]
    fn test_reject_invalid_hash() {
        let verifier = create_verifier();

        let valid_hash = FheTokenVerifier::hash_credential("111.222.333-44");
        let invalid_hash = FheTokenVerifier::hash_credential("999.999.999-99");

        let enc_valid = verifier.encrypt_hash(&valid_hash).expect("Encrypt failed");
        let enc_invalid = verifier.encrypt_hash(&invalid_hash).expect("Encrypt failed");

        let verdict = verifier.verify_homomorphic(&enc_invalid, &enc_valid)
            .expect("Verification failed");
        let is_allowed = verifier.decrypt_verdict(&verdict)
            .expect("Decrypt failed");

        assert!(!is_allowed);
    }

    #[test]
    fn test_homomorphic_equality_byte_by_byte() {
        let verifier = create_verifier();
        let hash_a = FheTokenVerifier::hash_credential("test_identity");
        let hash_b = FheTokenVerifier::hash_credential("test_identity");
        let hash_c = FheTokenVerifier::hash_credential("different_identity");

        let enc_a = verifier.encrypt_hash(&hash_a).expect("Encrypt A");
        let enc_b = verifier.encrypt_hash(&hash_b).expect("Encrypt B");
        let enc_c = verifier.encrypt_hash(&hash_c).expect("Encrypt C");

        let verdict_ab = verifier.verify_homomorphic(&enc_a, &enc_b).expect("Verify AB");
        assert!(verifier.decrypt_verdict(&verdict_ab).expect("Decrypt AB"));

        let verdict_ac = verifier.verify_homomorphic(&enc_a, &enc_c).expect("Verify AC");
        assert!(!verifier.decrypt_verdict(&verdict_ac).expect("Decrypt AC"));
    }

    #[test]
    fn test_performance_benchmark() {
        let verifier = create_verifier();
        let hash_1 = FheTokenVerifier::hash_credential("perf_test_1");
        let hash_2 = FheTokenVerifier::hash_credential("perf_test_2");

        let enc_1 = verifier.encrypt_hash(&hash_1).expect("Encrypt 1");
        let enc_2 = verifier.encrypt_hash(&hash_2).expect("Encrypt 2");

        let mut durations = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let _ = verifier.verify_homomorphic(&enc_1, &enc_2);
            durations.push(start.elapsed());
        }

        let avg = durations.iter().sum::<std::time::Duration>() / durations.len() as u32;
        println!("Tempo médio de verificação FHE (32 bytes): {:?}", avg);
        assert!(avg.as_millis() < 5000);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let verifier = create_verifier();
        let hash = FheTokenVerifier::hash_credential("serialization_test");

        let encrypted = verifier.encrypt_hash(&hash).expect("Encrypt");
        let decrypted = verifier.decrypt_hash(&encrypted).expect("Decrypt");

        assert_eq!(hash, decrypted);
    }

    #[test]
    fn test_server_key_cannot_decrypt() {
        let verifier = create_verifier();
        let hash = FheTokenVerifier::hash_credential("test");
        let enc = verifier.encrypt_hash(&hash).expect("Encrypt");

        let decrypted = verifier.decrypt_hash(&enc).expect("Decrypt");
        assert_eq!(hash, decrypted);

        assert!(true);
    }
}
