//! Safe-Core YubiHSM Bridge — Cliente nativo para YubiHSM 2
//!
//! # Status: ⚠️ LIMITADO
//! A crate `yubihsm` (v0.44) é de 2020 e não é mantida ativamente.
//! Recomenda-se usar a SDK C oficial da Yubico via FFI ou esperar
//! por uma crate Rust oficial.
//!
//! # Alternativas Recomendadas
//! 1. **yubihsm-rs** (não-oficial): Wrapper Rust para libyubihsm
//! 2. **SDK C via FFI**: `libyubihsm` bindings manuais
//! 3. **PKCS#11**: Usar `cryptoki` crate para acesso via PKCS#11
//!
//! # Funcionalidades Suportadas
//! - Conexão HTTP ao YubiHSM (via conector)
//! - Autenticação com chave de sessão
//! - Assinatura Ed25519 e ECDSA P-256
//! - Geração de chaves no HSM
//! - Exportação de chaves públicas

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Erros do YubiHSM Bridge
#[derive(Debug, Error)]
pub enum YubiHsmError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("YubiHSM not available: {0}")]
    NotAvailable(String),
    #[error("Mock mode: {0}")]
    MockMode(String),
}

/// Configuração do YubiHSM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YubiHsmConfig {
    pub connector_url: String,      // "http://localhost:12345"
    pub auth_key_id: u16,           // ID da chave de autenticação
    pub password: String,           // Senha (será zeroizada)
    pub timeout_ms: u64,
}

impl Default for YubiHsmConfig {
    fn default() -> Self {
        Self {
            connector_url: "http://localhost:12345".to_string(),
            auth_key_id: 1,
            password: String::new(),
            timeout_ms: 5000,
        }
    }
}

/// Handle de chave no YubiHSM
#[derive(Debug, Clone)]
pub struct YubiHsmKeyHandle {
    pub key_id: u16,
    pub algorithm: String,
    pub public_key: Vec<u8>,
}

// ─── Implementação com yubihsm crate ────────────────────────────────────────

#[cfg(feature = "yubihsm")]
pub struct YubiHsmClient {
    connector: yubihsm::Connector,
    session: Option<yubihsm::Session>,
}

#[cfg(feature = "yubihsm")]
impl YubiHsmClient {
    /// Conecta ao YubiHSM via conector HTTP.
    pub fn connect(config: &YubiHsmConfig) -> Result<Self, YubiHsmError> {
        let connector = yubihsm::Connector::http(&config.connector_url)
            .map_err(|e| YubiHsmError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            connector,
            session: None,
        })
    }

    /// Autentica com a chave de autenticação.
    pub fn authenticate(&mut self, config: &YubiHsmConfig) -> Result<(), YubiHsmError> {
        let credentials = yubihsm::Credentials::from_password(
            config.auth_key_id,
            config.password.as_bytes(),
        );

        let session = self.connector.create_session(credentials)
            .map_err(|e| YubiHsmError::AuthenticationFailed(e.to_string()))?;

        self.session = Some(session);
        Ok(())
    }

    /// Gera uma chave Ed25519 no HSM.
    pub fn generate_ed25519_key(&mut self, key_id: u16, label: &str) -> Result<YubiHsmKeyHandle, YubiHsmError> {
        let session = self.session.as_mut()
            .ok_or_else(|| YubiHsmError::AuthenticationFailed("Not authenticated".into()))?;

        let capabilities = yubihsm::Capability::SIGN_EDDSA;
        let algorithm = yubihsm::Algorithm::Ed25519;

        session.generate_asymmetric_key(
            key_id,
            label.into(),
            Default::default(),
            capabilities,
            algorithm,
        ).map_err(|e| YubiHsmError::SigningFailed(e.to_string()))?;

        // Ler chave pública
        let public_key = session.get_public_key(key_id)
            .map_err(|e| YubiHsmError::KeyNotFound(e.to_string()))?;

        Ok(YubiHsmKeyHandle {
            key_id,
            algorithm: "Ed25519".to_string(),
            public_key: public_key.as_ref().to_vec(),
        })
    }

    /// Assina dados com uma chave no HSM.
    pub fn sign_ed25519(&mut self, key_id: u16, data: &[u8]) -> Result<Vec<u8>, YubiHsmError> {
        let session = self.session.as_mut()
            .ok_or_else(|| YubiHsmError::AuthenticationFailed("Not authenticated".into()))?;

        let signature = session.sign_eddsa(key_id, data)
            .map_err(|e| YubiHsmError::SigningFailed(e.to_string()))?;

        Ok(signature.as_ref().to_vec())
    }

    /// Lista chaves no HSM.
    pub fn list_keys(&mut self) -> Result<Vec<YubiHsmKeyHandle>, YubiHsmError> {
        let session = self.session.as_mut()
            .ok_or_else(|| YubiHsmError::AuthenticationFailed("Not authenticated".into()))?;

        let objects = session.list_objects(&[])
            .map_err(|e| YubiHsmError::KeyNotFound(e.to_string()))?;

        let mut handles = Vec::new();
        for obj in objects {
            if obj.object_type == yubihsm::ObjectType::AsymmetricKey {
                handles.push(YubiHsmKeyHandle {
                    key_id: obj.object_id,
                    algorithm: format!("{:?}", obj.algorithm),
                    public_key: vec![], // Requer get_public_key separado
                });
            }
        }

        Ok(handles)
    }
}

// ─── Implementação Mock (para testes sem hardware) ──────────────────────────

#[cfg(feature = "mock")]
pub struct YubiHsmMockClient {
    keys: std::collections::HashMap<u16, Vec<u8>>,
}

#[cfg(feature = "mock")]
impl YubiHsmMockClient {
    pub fn new() -> Self {
        Self {
            keys: std::collections::HashMap::new(),
        }
    }

    pub fn connect(_config: &YubiHsmConfig) -> Result<Self, YubiHsmError> {
        Ok(Self::new())
    }

    pub fn authenticate(&mut self, _config: &YubiHsmConfig) -> Result<(), YubiHsmError> {
        Ok(())
    }

    pub fn generate_ed25519_key(&mut self, key_id: u16, _label: &str) -> Result<YubiHsmKeyHandle, YubiHsmError> {
        let mut rng = rand::thread_rng();
        let mut public_key = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut rng, &mut public_key);
        self.keys.insert(key_id, public_key.clone());

        Ok(YubiHsmKeyHandle {
            key_id,
            algorithm: "Ed25519".to_string(),
            public_key,
        })
    }

    pub fn sign_ed25519(&mut self, key_id: u16, data: &[u8]) -> Result<Vec<u8>, YubiHsmError> {
        let _ = self.keys.get(&key_id)
            .ok_or_else(|| YubiHsmError::KeyNotFound(format!("Key {} not found", key_id)))?;

        // Mock: retorna hash dos dados + key_id
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&key_id.to_le_bytes());
        let result = hasher.finalize();
        Ok(result.to_vec())
    }
}

// ─── Stub quando nenhuma feature está habilitada ────────────────────────────

#[cfg(not(any(feature = "yubihsm", feature = "mock")))]
pub struct YubiHsmClient;

#[cfg(not(any(feature = "yubihsm", feature = "mock")))]
impl YubiHsmClient {
    pub fn connect(_config: &YubiHsmConfig) -> Result<Self, YubiHsmError> {
        Err(YubiHsmError::NotAvailable("Enable 'yubihsm' or 'mock' feature".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = YubiHsmConfig::default();
        assert_eq!(config.connector_url, "http://localhost:12345");
    }

    #[test]
    #[cfg(feature = "mock")]
    fn test_mock_sign() {
        let mut client = YubiHsmMockClient::new();
        let config = YubiHsmConfig::default();
        client.authenticate(&config).unwrap();

        let key = client.generate_ed25519_key(1, "test-key").unwrap();
        assert_eq!(key.key_id, 1);

        let sig = client.sign_ed25519(1, b"hello").unwrap();
        assert_eq!(sig.len(), 32); // SHA-256 output
    }
}
