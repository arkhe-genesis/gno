use crypto::{DynPublicKey, DynSignature};

pub trait HsmBackend: Send + Sync {
    fn sign(&self, key_id: &str, data: &[u8]) -> Result<DynSignature, String>;
    fn export_public_key(&self, key_id: &str) -> Result<DynPublicKey, String>;
}
