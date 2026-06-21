//! src/main.rs
//! Exemplo de inicialização do CCA 2.0 com todos os componentes.

use std::sync::Arc;
use cathedral_cse::{
    CCAgentV2, CCAConfig, SessionManager,
    agent::{TrinityCore, ToolContext},
    llm::LlmClient,
};

// Implementação concreta do LlmClient (exemplo)
struct OpenAiClient;

#[async_trait::async_trait]
impl LlmClient for OpenAiClient {
    async fn chat_completion(&self, _messages: &[cathedral_cse::agent::AgentMessage], _tools: Option<serde_json::Value>) -> Result<String, String> {
        Ok("Resposta simulada".to_string())
    }

    async fn chat_completion_stream(
        &self,
        _messages: &[cathedral_cse::agent::AgentMessage],
        _tools: Option<serde_json::Value>,
    ) -> Result<Box<dyn futures::Stream<Item = Result<String, String>> + Send + Unpin>, String> {
        Err("Not implemented".to_string())
    }

    fn clone_arc(&self) -> Arc<dyn LlmClient + Send + Sync> {
        Arc::new(OpenAiClient)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm_client = Arc::new(OpenAiClient);
    let trinity = Arc::new(TrinityCore::new());
    let session_manager = Arc::new(SessionManager::new(100));

    let config = CCAConfig::default();
    let mut agent = CCAgentV2::new(config, llm_client, trinity, session_manager.clone()).await;

    let session_id = "test-session";
    agent.session_manager.create_session(session_id, Arc::new(ToolContext::new("./workspace".into()))).await;

    let response = agent.process("Cria uma função em Rust que calcula o factorial", session_id).await.unwrap_or_default();
    println!("Resposta:\n{}", response);

    Ok(())
}