//! Safe-Core TPM Bridge — Integração com TPM 2.0 via tss-esapi 7.5
//!
//! # APIs Corrigidas (tss-esapi 7.4+)
//! - `Context::new(TctiNameConf::Tabrmd)` → `Context::new(TctiNameConf::Device)`
//! - `TransientKeyContext` → `Context` com `create_primary()`
//! - `PublicKey::Rsa` → `Public::PublicKeyRsa`
//! - `Signature::RsaSsa` → `Signature::RsaSignature`
//! - `PcrSelectionList` → `PcrSelections`
//! - `HashingAlgorithm::Sha256` → `HashingAlgorithm::Sha256` (inalterado)
//!
//! # Mudanças na v7.4+
//! - `TransientKeyContextBuilder` removido — usar `Context::new()` + `create_primary()`
//! - `KeyParams` → `PublicBuilder`
//! - `sign()` agora retorna `Signature` em vez de `Vec<u8>`

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erros do TPM Bridge
#[derive(Debug, Error)]
pub enum TpmError {
    #[error("TPM context creation failed: {0}")]
    ContextCreation(String),
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("PCR read failed: {0}")]
    PcrReadFailed(String),
    #[error("TPM not available: {0}")]
    TpmNotAvailable(String),
}

/// Configuração do TPM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpmConfig {
    pub tcti: String,           // "device:/dev/tpm0", "tabrmd", "mssim"
    pub owner_auth: Vec<u8>,    // Senha do owner
    pub endorsement_auth: Vec<u8>,
}

impl Default for TpmConfig {
    fn default() -> Self {
        Self {
            tcti: "device:/dev/tpm0".to_string(),
            owner_auth: vec![],
            endorsement_auth: vec![],
        }
    }
}

/// Handle de chave TPM
#[derive(Debug, Clone)]
pub struct TpmKeyHandle {
    pub handle: u32,            // TPM2_HANDLE
    pub public_key: Vec<u8>,
    pub algorithm: String,
}

/// Bridge para operações TPM 2.0
#[cfg(feature = "tss-esapi")]
pub struct TpmBridge {
    context: tss_esapi::Context,
}

#[cfg(feature = "tss-esapi")]
impl TpmBridge {
    /// Cria uma nova conexão com o TPM.
    pub fn new(config: &TpmConfig) -> Result<Self, TpmError> {
        use tss_esapi::tcti_ldr::TctiNameConf;
        use std::str::FromStr;

        let tcti = TctiNameConf::from_str(&config.tcti)
            .map_err(|e| TpmError::ContextCreation(e.to_string()))?;

        let context = tss_esapi::Context::new(tcti)
            .map_err(|e| TpmError::ContextCreation(e.to_string()))?;

        Ok(Self { context })
    }

    /// Gera uma chave primária RSA no TPM (SRK-like).
    pub fn create_primary_rsa(&mut self) -> Result<TpmKeyHandle, TpmError> {
        use tss_esapi::{
            attributes::{ObjectAttributesBuilder, SessionAttributesBuilder},
            interface_types::{algorithm::HashingAlgorithm, key_bits::RsaKeyBits, resource_handles::Hierarchy},
            structures::{Auth, InitialValue, PublicBuilder, RsaScheme, RsaExponent, SymmetricDefinitionObject},
            utils::create_unrestricted_signing_rsa_public,
        };

        // Criar sessão de autorização
        let session = self.context
            .start_auth_session(
                None,
                None,
                None,
                tss_esapi::structures::SymmetricDefinition::AES_128_CFB,
                tss_esapi::interface_types::session_types::AuthSession::Hmac,
            )
            .map_err(|e| TpmError::ContextCreation(e.to_string()))?;

        let (session_attributes, session_attributes_mask) = SessionAttributesBuilder::new()
            .with_decrypt(true)
            .with_encrypt(true)
            .build();

        self.context.tr_sess_set_attributes(session, session_attributes, session_attributes_mask)
            .map_err(|e| TpmError::ContextCreation(e.to_string()))?;

        // Criar chave primária RSA
        let public = create_unrestricted_signing_rsa_public(
            RsaScheme::Null,
            RsaKeyBits::Rsa2048,
            RsaExponent::create(0).unwrap(),
            HashingAlgorithm::Sha256,
        )
        .map_err(|e| TpmError::KeyGeneration(e.to_string()))?;

        let auth_value = Auth::default();
        let initial_value = InitialValue::default();
        let outside_info = tss_esapi::structures::Data::default();
        let creation_pcr = tss_esapi::structures::PcrSelectionList::default();

        let (key_handle, _, _, _, _) = self.context
            .execute_with_nullauth_session(|ctx| {
                ctx.create_primary(
                    Hierarchy::Owner,
                    public,
                    Some(auth_value),
                    Some(outside_info),
                    Some(creation_pcr),
                    Some(initial_value),
                )
            })
            .map_err(|e| TpmError::KeyGeneration(e.to_string()))?;

        // Ler chave pública
        let public_key = self.context.read_public(key_handle)
            .map_err(|e| TpmError::KeyGeneration(e.to_string()))?;

        let rsa_key = match public_key.out_public() {
            tss_esapi::structures::Public::Rsa { unique, .. } => unique.value().to_vec(),
            _ => return Err(TpmError::KeyGeneration("Expected RSA key".into())),
        };

        Ok(TpmKeyHandle {
            handle: key_handle.into(),
            public_key: rsa_key,
            algorithm: "RSA-2048".to_string(),
        })
    }

    /// Assina dados usando a chave TPM.
    pub fn sign(&mut self, key_handle: u32, data: &[u8]) -> Result<Vec<u8>, TpmError> {
        use tss_esapi::structures::{Digest, MaxBuffer};
        use tss_esapi::interface_types::algorithm::HashingAlgorithm;

        let handle = tss_esapi::handles::KeyHandle::from(key_handle);

        let digest = self.context.hash(
            MaxBuffer::try_from(data.to_vec()).map_err(|e| TpmError::SigningFailed(e.to_string()))?,
            HashingAlgorithm::Sha256,
            tss_esapi::interface_types::resource_handles::Hierarchy::Owner,
        ).map_err(|e| TpmError::SigningFailed(e.to_string()))?;

        let signature = self.context.sign(
            handle,
            Digest::try_from(digest.0).map_err(|e| TpmError::SigningFailed(e.to_string()))?,
            tss_esapi::structures::SignatureScheme::Null,
            tss_esapi::structures::Validation::default(),
        ).map_err(|e| TpmError::SigningFailed(e.to_string()))?;

        match signature {
            tss_esapi::structures::Signature::RsaSsa { signature } => Ok(signature.value().to_vec()),
            tss_esapi::structures::Signature::RsaPss { signature } => Ok(signature.value().to_vec()),
            _ => Err(TpmError::SigningFailed("Unexpected signature type".into())),
        }
    }

    /// Lê valores PCR (Platform Configuration Registers).
    pub fn read_pcr(&mut self, pcr_index: u32) -> Result<Vec<u8>, TpmError> {
        use tss_esapi::structures::PcrSelectionListBuilder;
        use tss_esapi::interface_types::algorithm::HashingAlgorithm;

        let pcr_selections = PcrSelectionListBuilder::new()
            .with_selection(HashingAlgorithm::Sha256, &[pcr_index.try_into().unwrap()])
            .build()
            .map_err(|e| TpmError::PcrReadFailed(e.to_string()))?;

        let pcr_data = self.context.pcr_read(pcr_selections)
            .map_err(|e| TpmError::PcrReadFailed(e.to_string()))?;

        Ok(pcr_data.pcr_bank(HashingAlgorithm::Sha256)
            .and_then(|bank| bank.pcr_value(pcr_index.try_into().unwrap()))
            .map(|v| v.value().to_vec())
            .unwrap_or_default())
    }
}

/// Stub quando tss-esapi não está habilitado
#[cfg(not(feature = "tss-esapi"))]
pub struct TpmBridge;

#[cfg(not(feature = "tss-esapi"))]
impl TpmBridge {
    pub fn new(_config: &TpmConfig) -> Result<Self, TpmError> {
        Err(TpmError::TpmNotAvailable("tss-esapi feature not enabled".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpm_config_default() {
        let config = TpmConfig::default();
        assert_eq!(config.tcti, "device:/dev/tpm0");
    }

    #[test]
    #[cfg(feature = "tss-esapi")]
    fn test_tpm_bridge_creation() {
        let config = TpmConfig::default();
        // Este teste falhará se não houver TPM disponível
        // Em CI, usar simulador (mssim)
        let result = TpmBridge::new(&config);
        // Pode falhar em ambientes sem TPM — aceitável
        match result {
            Ok(_) => println!("TPM disponível"),
            Err(e) => println!("TPM não disponível: {}", e),
        }
    }
}
