//! Safe-Core Hash — Blake3 com fallback SHA-2
//!
//! # Features
//! - `blake3` (default): Usa BLAKE3 (SIMD acelerado, 3x mais rápido que SHA-256)
//! - `sha2-fallback`: Usa SHA-256 quando BLAKE3 não está disponível
//!
//! # Performance
//! ```text
//! BLAKE3: ~1 GB/s (single-thread), ~3 GB/s (multi-thread com rayon's blake3)
//! SHA-256: ~200 MB/s (software), ~500 MB/s (SHA-NI)
//! ```

use thiserror::Error;

/// Erros de hashing
#[derive(Debug, Error)]
pub enum HashError {
    #[error("Hashing failed: {0}")]
    Internal(String),
}

/// Trait unificado para algoritmos de hash.
pub trait Hasher: Send + Sync {
    /// Atualiza o hasher com dados.
    fn update(&mut self, data: &[u8]);
    /// Finaliza e retorna o hash de 32 bytes.
    fn finalize(self) -> [u8; 32];
    /// Hash one-shot.
    fn hash(data: &[u8]) -> [u8; 32] where Self: Sized;
}

// ─── Blake3 Implementation ──────────────────────────────────────────────────

#[cfg(feature = "blake3")]
pub struct Blake3Hasher {
    state: blake3::Hasher,
}

#[cfg(feature = "blake3")]
impl Blake3Hasher {
    pub fn new() -> Self {
        Self { state: blake3::Hasher::new() }
    }
}

#[cfg(feature = "blake3")]
impl Hasher for Blake3Hasher {
    fn update(&mut self, data: &[u8]) {
        self.state.update(data);
    }

    fn finalize(self) -> [u8; 32] {
        self.state.finalize().into()
    }

    fn hash(data: &[u8]) -> [u8; 32] {
        blake3::hash(data).into()
    }
}

// ─── SHA-256 Fallback Implementation ────────────────────────────────────────

#[cfg(feature = "sha2-fallback")]
pub struct Sha256Hasher {
    state: sha2::Sha256,
}

#[cfg(feature = "sha2-fallback")]
impl Sha256Hasher {
    pub fn new() -> Self {
        use sha2::Digest;
        Self { state: sha2::Sha256::new() }
    }
}

#[cfg(feature = "sha2-fallback")]
impl Hasher for Sha256Hasher {
    fn update(&mut self, data: &[u8]) {
        use sha2::Digest;
        self.state.update(data);
    }

    fn finalize(self) -> [u8; 32] {
        use sha2::Digest;
        let result = self.state.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        bytes
    }

    fn hash(data: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let result = Sha256::digest(data);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        bytes
    }
}

// ─── Factory ────────────────────────────────────────────────────────────────

/// Cria o hasher padrão (Blake3 se disponível, senão SHA-256).
pub fn default_hasher() -> Box<dyn Hasher> {
    #[cfg(feature = "blake3")]
    { Box::new(Blake3Hasher::new()) }

    #[cfg(not(feature = "blake3"))]
    {
        #[cfg(feature = "sha2-fallback")]
        { Box::new(Sha256Hasher::new()) }

        #[cfg(not(feature = "sha2-fallback"))]
        { panic!("No hasher backend enabled. Enable 'blake3' or 'sha2-fallback'.") }
    }
}

/// Hash one-shot usando o algoritmo padrão.
pub fn hash(data: &[u8]) -> [u8; 32] {
    #[cfg(feature = "blake3")]
    { Blake3Hasher::hash(data) }

    #[cfg(not(feature = "blake3"))]
    {
        #[cfg(feature = "sha2-fallback")]
        { Sha256Hasher::hash(data) }

        #[cfg(not(feature = "sha2-fallback"))]
        { panic!("No hasher backend enabled.") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "blake3")]
    fn test_blake3_hash() {
        let data = b"hello world";
        let hash = Blake3Hasher::hash(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    #[cfg(feature = "sha2-fallback")]
    fn test_sha256_fallback() {
        let data = b"hello world";
        let hash = Sha256Hasher::hash(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_factory() {
        let hash = hash(b"test");
        assert_eq!(hash.len(), 32);
    }
}
