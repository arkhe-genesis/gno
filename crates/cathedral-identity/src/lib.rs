pub struct SignatureGuard;

impl SignatureGuard {
    pub fn new() -> Self {
        Self
    }

    pub fn sign(&self, _data: &[u8]) -> Vec<u8> {
        vec![0u8; 64]
    }
}
