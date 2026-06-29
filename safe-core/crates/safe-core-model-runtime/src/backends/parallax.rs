use crate::error::RuntimeError;
use crate::runtime::ModelRuntime;
use crate::types::*;
use async_trait::async_trait;
use safe_core_parallax_bridge::ParallaxClient;
use safe_core_policy::{ConsensusGuard, Proposal};
use tracing::{info, warn};

pub struct ParallaxBackend {
    client: ParallaxClient,
    guard: ConsensusGuard,
    model_name: String,
    config: ModelConfig,
}

impl ParallaxBackend {
    pub async fn new(
        addr: &str,
        model_name: &str,
        config: ModelConfig,
        guard: ConsensusGuard,
    ) -> Result<Self, RuntimeError> {
        let client = ParallaxClient::connect(addr)
            .await
            .map_err(|e| RuntimeError::Backend(format!("Connection failed: {}", e)))?;

        let health = client
            .health()
            .await
            .map_err(|e| RuntimeError::Backend(format!("Health check failed: {}", e)))?;

        if !health.ready {
            return Err(RuntimeError::NotReady);
        }

        info!(
            "Parallax cluster ready (version: {}), checking model availability...",
            health.version
        );

        let available_models = client
            .list_models()
            .await
            .map_err(|e| RuntimeError::Backend(format!("Failed to list models: {}", e)))?;

        if !available_models.contains(&model_name.to_string()) {
            return Err(RuntimeError::NotFound(format!(
                "Model '{}' not found. Available models: {:?}",
                model_name, available_models
            )));
        }

        info!("Model '{}' confirmed available", model_name);

        Ok(Self {
            client,
            guard,
            model_name: model_name.to_string(),
            config,
        })
    }
}

#[async_trait]
impl ModelRuntime for ParallaxBackend {
    async fn x_infer(&self, request: InferenceRequest) -> Result<InferenceResponse, RuntimeError> {
        if request.prompt.is_empty() && request.messages.is_empty() {
            return Err(RuntimeError::InvalidRequest(
                "Either prompt or messages must be provided".into(),
            ));
        }

        let proposal = Proposal {
            tool: "infer".to_string(),
            payload: serde_json::json!({
                "model": self.model_name,
                "prompt_length": request.prompt.len(),
                "messages_count": request.messages.len(),
            }),
        };

        self.guard
            .evaluate(&proposal)
            .map_err(|e| RuntimeError::Policy(e.to_string()))?;

        let params = safe_core_parallax_bridge::SamplingParams {
            temperature: request.params.temperature,
            top_p: request.params.top_p,
            top_k: request.params.top_k,
            max_tokens: request.params.max_tokens,
            stop_sequences: request.params.stop_sequences,
            seed: request.params.seed,
        };

        let bridge_request = safe_core_parallax_bridge::InferRequest {
            model_name: self.model_name.clone(),
            prompt: request.prompt,
            system_prompt: request.system_prompt,
            messages: request
                .messages
                .into_iter()
                .map(|m| safe_core_parallax_bridge::ChatMessage {
                    role: m.role,
                    content: m.content,
                })
                .collect(),
            params,
            tools: request
                .tools
                .into_iter()
                .map(|t| safe_core_parallax_bridge::ToolDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                })
                .collect(),
            metadata: request.metadata,
        };

        let resp = self.client.infer(bridge_request).await?;

        Ok(InferenceResponse {
            id: resp.id,
            content: resp.content,
            tool_calls: resp
                .tool_calls
                .into_iter()
                .map(|tc| ToolCall {
                    id: tc.id,
                    name: tc.name,
                    arguments: tc.arguments,
                })
                .collect(),
            usage: TokenUsage {
                prompt_tokens: resp.usage.prompt_tokens,
                completion_tokens: resp.usage.completion_tokens,
                total_tokens: resp.usage.total_tokens,
            },
            finish_reason: FinishReason::from(resp.finish_reason.as_str()),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn x_embed(&self, texts: Vec<String>) -> Result<Vec<Tensor>, RuntimeError> {
        if texts.is_empty() {
            return Err(RuntimeError::InvalidRequest(
                "texts must not be empty".into(),
            ));
        }

        let embeddings = self
            .client
            .embed(&self.model_name, texts)
            .await
            .map_err(|e| RuntimeError::Backend(format!("Embedding failed: {}", e)))?;

        Ok(embeddings
            .into_iter()
            .map(|emb| Tensor::new(emb.values.clone(), vec![emb.values.len()]))
            .collect())
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn is_ready(&self) -> bool {
        match self.client.health().await {
            Ok(health) => health.ready,
            Err(e) => {
                warn!("Health check failed: {}", e);
                false
            }
        }
    }
}
