use crate::error::RuntimeError;
use crate::types::{InferenceRequest, InferenceResponse, ChatMessage, Tensor, ModelConfig};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait ModelRuntime: Send + Sync {
    async fn x_infer(&self, request: InferenceRequest) -> Result<InferenceResponse, RuntimeError>;

    async fn x_infer_chat(&self, messages: Vec<ChatMessage>) -> Result<InferenceResponse, RuntimeError> {
        let request = InferenceRequest::chat(messages);
        self.x_infer(request).await
    }

    async fn x_embed(&self, _texts: Vec<String>) -> Result<Vec<Tensor>, RuntimeError> {
        Err(RuntimeError::NotSupported(
            "Embedding not supported by this backend".into(),
        ))
    }

    fn model_name(&self) -> &str;

    async fn is_ready(&self) -> bool;
}

pub struct RuntimeRegistry {
    backends: tokio::sync::RwLock<HashMap<String, Arc<dyn ModelRuntime>>>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            backends: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, backend: Arc<dyn ModelRuntime>) -> Result<(), RuntimeError> {
        let name = backend.model_name().to_string();
        let mut backends = self.backends.write().await;
        backends.insert(name.clone(), backend);
        tracing::info!("Registered backend: {}", name);
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Result<Arc<dyn ModelRuntime>, RuntimeError> {
        let backends = self.backends.read().await;
        backends
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("Backend '{}' not registered", name)))
    }

    pub async fn list(&self) -> Vec<String> {
        let backends = self.backends.read().await;
        backends.keys().cloned().collect()
    }
}

impl Default for RuntimeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn register_parallax(
    registry: &RuntimeRegistry,
    addr: &str,
    model: &str,
    config: ModelConfig,
    guard: safe_core_policy::ConsensusGuard,
) -> Result<(), RuntimeError> {
    let backend = crate::backends::parallax::ParallaxBackend::new(addr, model, config, guard).await?;
    registry.register(std::sync::Arc::new(backend)).await?;
    Ok(())
}
