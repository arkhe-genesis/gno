use crate::error::CoreResult;
pub trait Signer: Send + Sync { fn sign(&self, data: &[u8]) -> CoreResult<Vec<u8>>; }
pub trait Verifier: Send + Sync { fn verify(&self, data: &[u8], signature: &[u8]) -> CoreResult<bool>; }
