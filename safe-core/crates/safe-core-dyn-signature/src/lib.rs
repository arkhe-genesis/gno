//! Safe-Core DynSignature — Assinaturas dinâmicas (P-256 + Ed25519)
//!
//! Suporta múltiplos algoritmos de assinatura via enum dinâmico,
//! permitindo que o Safe-Core negocie o algoritmo com o peer.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Erros de assinatura
#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Algorithm mismatch: expected {expected}, got {actual}")]
    AlgorithmMismatch { expected: String, actual: String },
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

/// Algoritmos de assinatura suportados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// NIST P-256 com ECDSA
    #[serde(rename = "P256")]
    P256,
    /// Ed25519 (Curve25519)
    #[serde(rename = "Ed25519")]
    Ed25519,
}

impl SignatureAlgorithm {
    pub fn as_str(&self) -> &str {
        match self {
            SignatureAlgorithm::P256 => "P256",
            SignatureAlgorithm::Ed25519 => "Ed25519",
        }
    }
}

/// Chave privada dinâmica (zeroized on drop).
#[derive(Zeroize, ZeroizeOnDrop)]
pub enum DynPrivateKey {
    #[cfg(feature = "p256")]
    #[zeroize(skip)]
    P256(p256::ecdsa::SigningKey),
    #[cfg(feature = "ed25519")]
    #[zeroize(skip)]
    Ed25519(ed25519_dalek::SigningKey),
}

/// Chave pública dinâmica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynPublicKey {
    #[cfg(feature = "p256")]
    P256(p256::ecdsa::VerifyingKey),
    #[cfg(feature = "ed25519")]
    Ed25519(ed25519_dalek::VerifyingKey),
}

impl PartialEq for DynPublicKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            #[cfg(feature = "p256")]
            (Self::P256(l0), Self::P256(r0)) => l0 == r0,
            #[cfg(feature = "ed25519")]
            (Self::Ed25519(l0), Self::Ed25519(r0)) => l0 == r0,
            _ => false,
        }
    }
}

/// Assinatura dinâmica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynSignature {
    #[cfg(feature = "p256")]
    P256(p256::ecdsa::Signature),
    #[cfg(feature = "ed25519")]
    Ed25519(ed25519_dalek::Signature),
}

impl DynSignature {
    /// Serializa a assinatura para bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            #[cfg(feature = "p256")]
            DynSignature::P256(sig) => sig.to_vec(),
            #[cfg(feature = "ed25519")]
            DynSignature::Ed25519(sig) => sig.to_bytes().to_vec(),
        }
    }

    /// Desserializa a assinatura a partir de bytes.
    pub fn from_bytes(alg: SignatureAlgorithm, bytes: &[u8]) -> Result<Self, SignatureError> {
        match alg {
            #[cfg(feature = "p256")]
            SignatureAlgorithm::P256 => {
                let sig = p256::ecdsa::Signature::from_slice(bytes)
                    .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;
                Ok(DynSignature::P256(sig))
            }
            #[cfg(feature = "ed25519")]
            SignatureAlgorithm::Ed25519 => {
                let bytes_arr: [u8; 64] = bytes.try_into()
                    .map_err(|_| SignatureError::InvalidKey("Ed25519 sig must be 64 bytes".into()))?;
                Ok(DynSignature::Ed25519(ed25519_dalek::Signature::from_bytes(&bytes_arr)))
            }
            #[allow(unreachable_patterns)]
            _ => Err(SignatureError::FeatureNotEnabled(alg.as_str().to_string())),
        }
    }
}

/// Trait para operações de assinatura.
pub trait Signer: Send + Sync {
    fn algorithm(&self) -> SignatureAlgorithm;
    fn sign(&self, message: &[u8]) -> Result<DynSignature, SignatureError>;
    fn public_key(&self) -> DynPublicKey;
}

/// Trait para verificação.
pub trait Verifier: Send + Sync {
    fn algorithm(&self) -> SignatureAlgorithm;
    fn verify(&self, message: &[u8], signature: &DynSignature) -> Result<(), SignatureError>;
}

// ─── P-256 Implementation ───────────────────────────────────────────────────

#[cfg(feature = "p256")]
pub struct P256Signer {
    signing_key: p256::ecdsa::SigningKey,
}

#[cfg(feature = "p256")]
impl P256Signer {
    pub fn generate<R: rand::CryptoRng + rand::RngCore>(rng: &mut R) -> Self {
        Self { signing_key: p256::ecdsa::SigningKey::random(rng) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        let key = p256::ecdsa::SigningKey::from_slice(bytes)
            .map_err(|e| SignatureError::InvalidKey(e.to_string()))?;
        Ok(Self { signing_key: key })
    }
}

#[cfg(feature = "p256")]
impl Signer for P256Signer {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::P256
    }

    fn sign(&self, message: &[u8]) -> Result<DynSignature, SignatureError> {
        use ecdsa::signature::Signer as _;
        let sig = self.signing_key.try_sign(message)
            .map_err(|e| SignatureError::SigningFailed(e.to_string()))?;
        Ok(DynSignature::P256(sig))
    }

    fn public_key(&self) -> DynPublicKey {
        DynPublicKey::P256(self.signing_key.verifying_key().clone())
    }
}

#[cfg(feature = "p256")]
pub struct P256Verifier {
    verifying_key: p256::ecdsa::VerifyingKey,
}

#[cfg(feature = "p256")]
impl P256Verifier {
    pub fn from_public_key(key: &DynPublicKey) -> Result<Self, SignatureError> {
        match key {
            DynPublicKey::P256(vk) => Ok(Self { verifying_key: *vk }),
            _ => Err(SignatureError::AlgorithmMismatch {
                expected: "P256".into(),
                actual: "other".into(),
            }),
        }
    }
}

#[cfg(feature = "p256")]
impl Verifier for P256Verifier {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::P256
    }

    fn verify(&self, message: &[u8], signature: &DynSignature) -> Result<(), SignatureError> {
        use ecdsa::signature::Verifier as _;
        match signature {
            DynSignature::P256(sig) => {
                self.verifying_key.verify(message, sig)
                    .map_err(|e| SignatureError::VerificationFailed(e.to_string()))
            }
            _ => Err(SignatureError::AlgorithmMismatch {
                expected: "P256".into(),
                actual: "other".into(),
            }),
        }
    }
}

// ─── Ed25519 Implementation ─────────────────────────────────────────────────

#[cfg(feature = "ed25519")]
pub struct Ed25519Signer {
    signing_key: ed25519_dalek::SigningKey,
}

#[cfg(feature = "ed25519")]
impl Ed25519Signer {
    pub fn generate<R: rand::CryptoRng + rand::RngCore>(rng: &mut R) -> Self {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self { signing_key: ed25519_dalek::SigningKey::from_bytes(&bytes) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        let bytes_arr: [u8; 32] = bytes.try_into()
            .map_err(|_| SignatureError::InvalidKey("Ed25519 key must be 32 bytes".into()))?;
        Ok(Self { signing_key: ed25519_dalek::SigningKey::from_bytes(&bytes_arr) })
    }
}

#[cfg(feature = "ed25519")]
impl Signer for Ed25519Signer {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::Ed25519
    }

    fn sign(&self, message: &[u8]) -> Result<DynSignature, SignatureError> {
        use ed25519_dalek::Signer as _;
        let sig = self.signing_key.sign(message);
        Ok(DynSignature::Ed25519(sig))
    }

    fn public_key(&self) -> DynPublicKey {
        DynPublicKey::Ed25519(self.signing_key.verifying_key())
    }
}

#[cfg(feature = "ed25519")]
pub struct Ed25519Verifier {
    verifying_key: ed25519_dalek::VerifyingKey,
}

#[cfg(feature = "ed25519")]
impl Ed25519Verifier {
    pub fn from_public_key(key: &DynPublicKey) -> Result<Self, SignatureError> {
        match key {
            DynPublicKey::Ed25519(vk) => Ok(Self { verifying_key: *vk }),
            _ => Err(SignatureError::AlgorithmMismatch {
                expected: "Ed25519".into(),
                actual: "other".into(),
            }),
        }
    }
}

#[cfg(feature = "ed25519")]
impl Verifier for Ed25519Verifier {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::Ed25519
    }

    fn verify(&self, message: &[u8], signature: &DynSignature) -> Result<(), SignatureError> {
        use ed25519_dalek::Verifier as _;
        match signature {
            DynSignature::Ed25519(sig) => {
                self.verifying_key.verify(message, sig)
                    .map_err(|e| SignatureError::VerificationFailed(e.to_string()))
            }
            _ => Err(SignatureError::AlgorithmMismatch {
                expected: "Ed25519".into(),
                actual: "other".into(),
            }),
        }
    }
}

pub fn verify_dyn_signature(sig: &DynSignature, key: &DynPublicKey, message: &[u8]) -> Result<(), SignatureError> {
    match key {
        #[cfg(feature = "p256")]
        DynPublicKey::P256(vk) => {
            let verifier = P256Verifier::from_public_key(key)?;
            verifier.verify(message, sig)
        }
        #[cfg(feature = "ed25519")]
        DynPublicKey::Ed25519(vk) => {
            let verifier = Ed25519Verifier::from_public_key(key)?;
            verifier.verify(message, sig)
        }
        _ => Err(SignatureError::AlgorithmMismatch {
            expected: "Supported algorithm".into(),
            actual: "Unsupported".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    #[cfg(feature = "p256")]
    fn test_p256_sign_verify() {
        let mut rng = OsRng;
        let signer = P256Signer::generate(&mut rng);
        let message = b"test message";
        let sig = signer.sign(message).unwrap();
        let verifier = P256Verifier::from_public_key(&signer.public_key()).unwrap();
        assert!(verifier.verify(message, &sig).is_ok());
    }

    #[test]
    #[cfg(feature = "ed25519")]
    fn test_ed25519_sign_verify() {
        let mut rng = OsRng;
        let signer = Ed25519Signer::generate(&mut rng);
        let message = b"test message";
        let sig = signer.sign(message).unwrap();
        let verifier = Ed25519Verifier::from_public_key(&signer.public_key()).unwrap();
        assert!(verifier.verify(message, &sig).is_ok());
    }

    #[test]
    #[cfg(all(feature = "p256", feature = "ed25519"))]
    fn test_algorithm_mismatch() {
        let mut rng = OsRng;
        let p256_signer = P256Signer::generate(&mut rng);
        let ed_signer = Ed25519Signer::generate(&mut rng);
        let message = b"test";
        let p256_sig = p256_signer.sign(message).unwrap();
        let ed_verifier = Ed25519Verifier::from_public_key(&ed_signer.public_key()).unwrap();
        assert!(ed_verifier.verify(message, &p256_sig).is_err());
    }
}
