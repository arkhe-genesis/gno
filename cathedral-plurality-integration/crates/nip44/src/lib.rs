use chacha20poly1305::{aead::{Aead, KeyInit, generic_array::GenericArray}, ChaCha20Poly1305};
use rand::RngCore;

pub fn validate_key(key: &str) -> bool {
    hex::decode(key).map(|v| v.len() == 32).unwrap_or(false)
}

pub fn encrypt(key: &str, data: &str) -> Result<String, String> {
    let key_bytes = hex::decode(key).map_err(|e| e.to_string())?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;

    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);

    let ciphertext = cipher.encrypt(&nonce.into(), data.as_bytes()).map_err(|e| e.to_string())?;

    let mut result = nonce.to_vec();
    result.extend(ciphertext);

    Ok(base64::encode(result))
}

pub fn decrypt(key: &str, data: &str) -> Result<String, String> {
    let key_bytes = hex::decode(key).map_err(|e| e.to_string())?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;

    let data_bytes = base64::decode(data).map_err(|e| e.to_string())?;
    if data_bytes.len() < 12 {
        return Err("Invalid data".to_string());
    }

    let (nonce, ciphertext) = data_bytes.split_at(12);
    let plaintext = cipher.decrypt(nonce.into(), ciphertext).map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
