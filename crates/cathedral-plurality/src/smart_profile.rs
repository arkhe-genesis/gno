//! Smart Profile para agentes (perfil inteligente)
//! Selo: CATHEDRAL-ARKHE-SMART-PROFILE-v1.0.0-2026-06-21

use crate::{PluralityClient, plurality_client::PluralityClientTrait, Result, SmartProfile, SmartProfileInput};
use serde_json::Value;
use std::collections::HashMap;

pub struct SmartProfileManager {
    client: PluralityClient,
    cache: HashMap<String, SmartProfile>,
}

impl SmartProfileManager {
    pub fn new(client: PluralityClient) -> Self {
        Self {
            client,
            cache: HashMap::new(),
        }
    }

    pub async fn get_profile(&mut self, agent_id: &str) -> Result<SmartProfile> {
        if let Some(cached) = self.cache.get(agent_id) {
            return Ok(cached.clone());
        }
        let profile = self.client.get_profile(agent_id).await?;
        self.cache.insert(agent_id.to_string(), profile.clone());
        Ok(profile)
    }

    pub async fn update_profile(&mut self, agent_id: &str, preferences: HashMap<String, String>) -> Result<SmartProfile> {
        let mut profile = self.get_profile(agent_id).await?;
        profile.preferences.extend(preferences);
        let input = SmartProfileInput {
            agent_id: agent_id.to_string(),
            preferences: profile.preferences.clone(),
            capabilities: profile.capabilities.clone(),
            context: profile.context.clone(),
        };
        let updated = self.client.update_profile(input).await?;
        self.cache.insert(agent_id.to_string(), updated.clone());
        Ok(updated)
    }

    pub async fn add_capability(&mut self, agent_id: &str, capability: String) -> Result<()> {
        let mut profile = self.get_profile(agent_id).await?;
        if !profile.capabilities.contains(&capability) {
            profile.capabilities.push(capability);
            let input = SmartProfileInput {
                agent_id: agent_id.to_string(),
                preferences: profile.preferences.clone(),
                capabilities: profile.capabilities.clone(),
                context: profile.context.clone(),
            };
            let updated = self.client.update_profile(input).await?;
            self.cache.insert(agent_id.to_string(), updated);
        }
        Ok(())
    }

    pub async fn set_context(&mut self, agent_id: &str, key: String, value: Value) -> Result<()> {
        let mut profile = self.get_profile(agent_id).await?;
        profile.context.insert(key, value);
        let input = SmartProfileInput {
            agent_id: agent_id.to_string(),
            preferences: profile.preferences.clone(),
            capabilities: profile.capabilities.clone(),
            context: profile.context.clone(),
        };
        let updated = self.client.update_profile(input).await?;
        self.cache.insert(agent_id.to_string(), updated);
        Ok(())
    }

    pub async fn get_context(&mut self, agent_id: &str, key: &str) -> Option<Value> {
        let profile = self.get_profile(agent_id).await.ok()?;
        profile.context.get(key).cloned()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn remove_from_cache(&mut self, agent_id: &str) {
        self.cache.remove(agent_id);
    }
}
