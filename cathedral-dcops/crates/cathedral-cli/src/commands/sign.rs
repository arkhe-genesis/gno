use clap::Args;
use ed25519_dalek::{SigningKey, Signer};
use std::fs;
use std::path::PathBuf;
use base64::Engine;

#[cfg(feature = "mldsa")]
use pqcrypto_dilithium::dilithium5::*;

#[derive(Args)]
pub struct SignArgs {
    /// Arquivo com o payload (JSON)
    #[arg(short, long)]
    payload: PathBuf,

    /// Arquivo com a chave privada (hex)
    #[arg(short, long)]
    private_key: PathBuf,

    /// Algoritmo: ed25519 (padrão) ou mldsa
    #[arg(long, default_value = "ed25519")]
    algorithm: String,

    /// Saída: hex (padrão) ou base64
    #[arg(long, default_value = "hex")]
    output_format: String,
}

pub async fn run_sign(args: SignArgs) -> Result<(), String> {
    // Lê payload
    let payload_bytes = fs::read(&args.payload)
        .map_err(|e| format!("Erro ao ler payload: {}", e))?;

    // Lê chave privada (hex)
    let priv_key_hex = fs::read_to_string(&args.private_key)
        .map_err(|e| format!("Erro ao ler chave: {}", e))?;
    let priv_key_bytes = hex::decode(priv_key_hex.trim())
        .map_err(|e| format!("Chave inválida: {}", e))?;

    let signature_hex = match args.algorithm.as_str() {
        "ed25519" => {
            // Ed25519: chave deve ter 32 bytes
            if priv_key_bytes.len() != 32 {
                return Err("Chave Ed25519 deve ter 32 bytes".to_string());
            }
            let signing_key = SigningKey::from_bytes(&priv_key_bytes.try_into().unwrap());
            let signature = signing_key.sign(&payload_bytes);
            hex::encode(signature.to_bytes())
        }
        #[cfg(feature = "mldsa")]
        "mldsa" => {
            // ML-DSA-65: chave deve ter 4000 bytes
            if priv_key_bytes.len() != 4000 {
                return Err("Chave ML-DSA-65 deve ter 4000 bytes".to_string());
            }
            let sk = SecretKey::from_bytes(&priv_key_bytes)
                .map_err(|_| "Invalid key for mldsa".to_string())?;
            let sig = sign(&payload_bytes, &sk);
            hex::encode(sig.as_bytes())
        }
        #[cfg(not(feature = "mldsa"))]
        "mldsa" => return Err("ML-DSA não habilitado. Compile com --features mldsa".to_string()),
        _ => return Err("Algoritmo não suportado".to_string()),
    };

    // Saída
    match args.output_format.as_str() {
        "hex" => println!("{}", signature_hex),
        "base64" => {
            let sig_bytes = hex::decode(signature_hex).unwrap();
            println!("{}", base64::engine::general_purpose::STANDARD.encode(&sig_bytes));
        }
        _ => return Err("Formato de saída inválido".to_string()),
    }

    Ok(())
}
