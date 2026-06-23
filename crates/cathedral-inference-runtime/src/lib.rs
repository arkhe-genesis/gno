pub mod models;
pub mod prompt_builder;
pub mod delegation;

pub use models::{GenerateRequest, GenerateResponse, VerificationLevel};
pub use prompt_builder::build_prompt;
pub use delegation::DelegationRouter;

use std::sync::Arc;
use cathedral_wormgraph::{WormGraphClient, MemoryEntry};
use cathedral_reputation::ReputationRouter;
use cathedral_zk::ZKGateway;

// Real integration with the requested components
pub struct CathedralRuntime {
    pub wormgraph: Arc<WormGraphClient>,
    pub reputation: Arc<ReputationRouter>,
    pub zk_gateway: Arc<ZKGateway>,
    pub delegation_router: DelegationRouter,
}

impl CathedralRuntime {
    pub async fn new() -> Self {
        Self {
            wormgraph: Arc::new(WormGraphClient::new("http://localhost:8080")),
            reputation: Arc::new(ReputationRouter::new()),
            zk_gateway: Arc::new(ZKGateway::new()),
            delegation_router: DelegationRouter::new(),
        }
    }

    pub async fn generate(&self, req: GenerateRequest) -> Result<GenerateResponse, ()> {
        let score = self.reputation.route(&req.did).await;
        let tier = self.delegation_router.select(score);
        let thinking = if req.level != VerificationLevel::L0 {
            Some("Thinking process...".to_string())
        } else {
            None
        };

        let zk_proof = if req.level != VerificationLevel::L0 {
            let mut gw = ZKGateway::new();
            gw.level = req.level.as_str().to_string();
            Some(gw.generate_proof(req.prompt.as_bytes()).await.unwrap())
        } else {
            None
        };

        // Add memory
        let _ = self.wormgraph.append_memory(&req.did, MemoryEntry { content: req.prompt.clone() }).await;

        let memories = self.wormgraph.get_memories(&req.did, 5).await.unwrap_or_default();
        let prompt_built = build_prompt(&req.prompt, &req.did, &memories, req.level.as_str());

        Ok(GenerateResponse {
            text: format!("Mocked generated response. Built prompt: {}", prompt_built),
            thinking,
            zk_proof,
            signature: vec![1, 2, 3],
            attestation: vec![0xF8],
            receipt: cathedral_wormgraph::ExecutionReceipt { id: "receipt_id".to_string() },
            latency_ms: if req.level == VerificationLevel::L2 { 600 } else if req.level == VerificationLevel::L1 { 400 } else { 200 },
            reputation: score,
            tier: match tier {
                cathedral_llm_core::ModelTier::Pro => "Pro".to_string(),
                cathedral_llm_core::ModelTier::Plus => "Plus".to_string(),
                cathedral_llm_core::ModelTier::Standard => "Standard".to_string(),
                cathedral_llm_core::ModelTier::Lite => "Lite".to_string(),
            },
        })
    }
}
