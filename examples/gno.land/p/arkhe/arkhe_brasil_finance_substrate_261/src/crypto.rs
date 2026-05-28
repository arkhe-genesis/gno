//! Criptografia (HMAC-SHA3-256)
use sha3::{Sha3_256, Digest};
use hmac::{Hmac, Mac};

type HmacSha3_256 = Hmac<Sha3_256>;

pub fn generate_hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha3_256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}
