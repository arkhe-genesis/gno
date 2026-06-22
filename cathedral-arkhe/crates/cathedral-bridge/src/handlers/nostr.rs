//! Handler para PublishNostr RPC
//! Selo: CATHEDRAL-ARKHE-BRIDGE-HANDLER-NOSTR-v1.0.0-2026-06-21

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::proto::{NostrPublishRequest, NostrPublishResponse};
use crate::server::BridgeState;

pub struct NostrHandler;

impl NostrHandler {
    pub async fn handle(
        state: Arc<BridgeState>,
        request: Request<NostrPublishRequest>,
    ) -> Result<Response<NostrPublishResponse>, Status> {
        let req = request.into_inner();
        info!("📡 PublishNostr: project={}, hash={}", req.project_id, req.design_hash);

        // 1. Verifica se o replicator está ativo
        let replicator = state.nostr_replicator.as_ref()
            .ok_or_else(|| Status::unavailable("Nostr replicator não configurado"))?;

        // 2. Constrói o evento Nostr (kind 30078)
        let tags = if req.tags.is_empty() {
            vec![
                vec!["project".to_string(), req.project_id.clone()],
                vec!["design_hash".to_string(), req.design_hash.clone()],
            ]
        } else {
            req.tags
        };

        let event = nostr_sdk::EventBuilder::new(
            nostr_sdk::Kind::Custom(30078),
            req.wormgraph_json,
            tags.into_iter().filter_map(|t| nostr_sdk::Tag::parse(&t).ok()).collect::<Vec<_>>(),
        )
        .to_event(&nostr_sdk::Keys::generate())
        .map_err(|e| Status::internal(format!("Falha na assinatura: {}", e)))?;

        // 3. Publica nos relays
        let relays = if req.relay_urls.is_empty() {
            replicator.default_relays().to_vec()
        } else {
            req.relay_urls
        };

        let published = replicator.publish_to_relays(&event, &relays).await
            .map_err(|e| Status::internal(format!("Falha na publicação: {}", e)))?;

        info!("✅ Evento publicado: {}", published.event_id_hex);

        Ok(Response::new(NostrPublishResponse {
            success: true,
            event_id_hex: published.event_id_hex,
            relay_urls: published.relay_urls,
            error: None,
            published_at: chrono::Utc::now().timestamp() as u64,
        }))
    }
}
