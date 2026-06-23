//! Cliente para Plurality Network API
//! Selo: CATHEDRAL-ARKHE-PLURALITY-CLIENT-v1.0.0-2026-06-21

use crate::{
    BucketType, MemoryItem, MemoryItemInput, PluralityAuth, PluralityError,
    Result, SearchQuery, SearchResult, SmartProfile, SmartProfileInput,
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[async_trait]
pub trait PluralityClientTrait {
    async fn store(&mut self, item: MemoryItemInput) -> Result<MemoryItem>;
    async fn retrieve(&mut self, key: &str, bucket: BucketType) -> Result<Option<MemoryItem>>;
    async fn search(&mut self, query: SearchQuery) -> Result<SearchResult>;
    async fn delete(&mut self, key: &str, bucket: BucketType) -> Result<()>;
    async fn get_profile(&mut self, agent_id: &str) -> Result<SmartProfile>;
    async fn update_profile(&mut self, profile: SmartProfileInput) -> Result<SmartProfile>;
}

pub struct PluralityClient {
    base_url: String,
    auth: PluralityAuth,
    client: Client,
    timeout: Duration,
}

impl PluralityClient {
    pub fn new(base_url: String, auth: PluralityAuth) -> Self {
        Self {
            base_url,
            auth,
            client: Client::new(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn request(&mut self, _method: reqwest::Method, _path: &str, _body: Option<serde_json::Value>) -> Result<reqwest::Response> {
        Err(PluralityError::Other("Not implemented in mock".to_string()))
    }
}

#[async_trait]
impl PluralityClientTrait for PluralityClient {
    async fn store(&mut self, item: MemoryItemInput) -> Result<MemoryItem> {
        let body = json!({
            "key": item.key,
            "value": BASE64.encode(&item.value),
            "bucket": item.bucket.as_str(),
            "ttl_seconds": item.ttl_seconds,
            "vector": item.vector,
            "metadata": item.metadata,
        });

        let response = self.request(reqwest::Method::POST, "/api/v1/memory", Some(body)).await?;
        let item: MemoryItem = response.json().await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(item)
    }

    async fn retrieve(&mut self, key: &str, bucket: BucketType) -> Result<Option<MemoryItem>> {
        let path = format!("/api/v1/memory/{}/{}", bucket.as_str(), key);
        let response = self.request(reqwest::Method::GET, &path, None).await;

        match response {
            Ok(resp) => {
                let item: MemoryItem = resp.json().await
                    .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
                Ok(Some(item))
            }
            Err(PluralityError::Other(msg)) if msg.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn search(&mut self, query: SearchQuery) -> Result<SearchResult> {
        let body = json!({
            "vector": query.vector,
            "bucket": query.bucket.as_str(),
            "limit": query.limit,
            "min_similarity": query.min_similarity,
            "filter": query.filter,
        });

        let response = self.request(reqwest::Method::POST, "/api/v1/memory/search", Some(body)).await?;
        let result: SearchResult = response.json().await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(result)
    }

    async fn delete(&mut self, key: &str, bucket: BucketType) -> Result<()> {
        let path = format!("/api/v1/memory/{}/{}", bucket.as_str(), key);
        self.request(reqwest::Method::DELETE, &path, None).await?;
        Ok(())
    }

    async fn get_profile(&mut self, agent_id: &str) -> Result<SmartProfile> {
        let path = format!("/api/v1/profiles/{}", agent_id);
        let response = self.request(reqwest::Method::GET, &path, None).await?;
        let profile: SmartProfile = response.json().await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(profile)
    }

    async fn update_profile(&mut self, profile: SmartProfileInput) -> Result<SmartProfile> {
        let body = json!({
            "agent_id": profile.agent_id,
            "preferences": profile.preferences,
            "capabilities": profile.capabilities,
            "context": profile.context,
        });

        let response = self.request(reqwest::Method::PUT, "/api/v1/profiles", Some(body)).await?;
        let profile: SmartProfile = response.json().await
            .map_err(|e| PluralityError::Serialization(format!("Erro ao parsear: {}", e)))?;
        Ok(profile)
    }
}
