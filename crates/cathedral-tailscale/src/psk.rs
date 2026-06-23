use blake3::Hasher;
use rand::RngCore;

/// Pre-Shared Key (256-bit) para resistência pós-quântica
pub struct PreSharedKey([u8; 32]);

impl PreSharedKey {
    /// Gera PSK aleatória (CRNG)
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self(key)
    }

    /// Deriva PSK a partir de DID + segredo compartilhado
    pub fn derive(did: &str, secret: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(did.as_bytes());
        hasher.update(secret);
        hasher.update(b"cathedral-psk-v1");

        let mut key = [0u8; 32];
        key.copy_from_slice(hasher.finalize().as_bytes());
        Self(key)
    }

    /// Converte para hex (para config WireGuard)
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}
