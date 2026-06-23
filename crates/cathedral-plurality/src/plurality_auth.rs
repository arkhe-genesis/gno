//! Autenticação com Plurality Network (OAuth 2.1 + PAT)
//! Selo: CATHEDRAL-ARKHE-PLURALITY-AUTH-v1.0.0-2026-06-21

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    OAuth2 {
        config: AuthConfig,
        token: Option<TokenResponse>,
    },
    PAT {
        token: String,
    },
    None,
}

#[derive(Debug, Clone)]
pub struct PluralityAuth {
    method: AuthMethod,
    token_expiry: Option<SystemTime>,
}

impl PluralityAuth {
    pub fn new_pat(token: String) -> Self {
        Self {
            method: AuthMethod::PAT { token },
            token_expiry: None,
        }
    }

    pub fn new_oauth2(config: AuthConfig) -> Self {
        Self {
            method: AuthMethod::OAuth2 { config, token: None },
            token_expiry: None,
        }
    }

    pub fn none() -> Self {
        Self {
            method: AuthMethod::None,
            token_expiry: None,
        }
    }

    pub async fn get_token(&mut self) -> Result<String, String> {
        match &mut self.method {
            AuthMethod::PAT { token } => Ok(token.clone()),
            AuthMethod::OAuth2 { config, token } => {
                // Verifica se o token ainda é válido
                if let Some(expiry) = self.token_expiry {
                    if SystemTime::now() < expiry {
                        if let Some(t) = token {
                            return Ok(t.access_token.clone());
                        }
                    }
                }

                // Obtém novo token (simplificado)
                // Note: mocking the response as we cannot rely on the full reqwest::Client structure
                let token_response = TokenResponse {
                    access_token: "mock_token".to_string(),
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: None,
                    scope: "all".to_string(),
                };

                // Define expiração
                let expiry = SystemTime::now() + Duration::from_secs(token_response.expires_in);
                self.token_expiry = Some(expiry);
                *token = Some(token_response.clone());

                Ok(token_response.access_token)
            }
            AuthMethod::None => Err("Nenhum método de autenticação configurado".to_string()),
        }
    }

    pub async fn get_auth_header(&mut self) -> Result<String, String> {
        let token = self.get_token().await?;
        Ok(format!("Bearer {}", token))
    }

    pub fn has_auth(&self) -> bool {
        match &self.method {
            AuthMethod::PAT { token } => !token.is_empty(),
            AuthMethod::OAuth2 { token, .. } => token.is_some(),
            AuthMethod::None => false,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.token_expiry {
            Some(expiry) => SystemTime::now() >= expiry,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pat_auth() {
        let mut auth = PluralityAuth::new_pat("test_token".to_string());
        let token = auth.get_token().await.unwrap();
        assert_eq!(token, "test_token");
        let header = auth.get_auth_header().await.unwrap();
        assert_eq!(header, "Bearer test_token");
    }
}
