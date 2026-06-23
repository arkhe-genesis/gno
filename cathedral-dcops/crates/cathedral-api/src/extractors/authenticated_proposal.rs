//! Extrator que valida DID + assinatura e recupera a proposta do banco
//! Suporta: Path, Query, e DELETE sem Path (usando ID do corpo)

use axum::{
    async_trait,
    extract::{FromRequestParts, Path, Query, State, FromRef},
    http::request::Parts,
    response::Response,
};
use crate::{AppState, ApiError, auth::verify_auth, ProposalQueryParams};
use cathedral_wormgraph::ImprovementProposal;
use std::sync::Arc;

/// Parâmetros para identificação da proposta
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum ProposalIdentifier {
    Path { id: String },
    Query { id: String },
    Body { id: String },
}

/// Extrator que valida autenticação e retorna a proposta
pub struct AuthenticatedProposal {
    pub proposal: ImprovementProposal,
    pub did: String,
    pub signature: Vec<u8>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedProposal
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extrai DID e assinatura dos headers
        let did = parts
            .headers
            .get("X-DID")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::MissingDid)
            .map_err(|_| Response::default())?
            .to_string();

        let signature = parts
            .headers
            .get("X-Signature")
            .and_then(|v| hex::decode(v.as_bytes()).ok())
            .ok_or_else(|| ApiError::MissingSignature)
            .map_err(|_| Response::default())?;

        // 2. Extrai o ID da proposta de diferentes fontes
        let id = if let Some(path_id) = parts.extensions.get::<Path<String>>() {
            path_id.0.clone()
        } else if let Some(query_params) = parts.extensions.get::<Query<ProposalQueryParams>>() {
            query_params.id.clone().ok_or(ApiError::MissingId).map_err(|_| Response::default())?
        } else {
            // Fallback: tenta extrair do corpo da requisição (para DELETE sem Path)
            // Nota: isso requer que o corpo já tenha sido lido antes
            // Na prática, você pode usar um middleware que armazena o corpo em extensions
            let body_bytes = parts
                .extensions
                .get::<Vec<u8>>()
                .ok_or(ApiError::MissingId)
                .map_err(|_| Response::default())?;
            let body_str = String::from_utf8_lossy(body_bytes);
            let body_json: serde_json::Value = serde_json::from_str(&body_str)
                .map_err(|_| ApiError::MissingId)
                .map_err(|_| Response::default())?;
            body_json["id"].as_str().ok_or(ApiError::MissingId).map_err(|_| Response::default())?.to_string()
        };

        // 3. Obtém o state da aplicação
        let app_state = Arc::from_ref(state);

        // 4. Busca a proposta no banco
        let proposal = app_state
            .wormgraph
            .get_proposal(&id)
            .await
            .map_err(|_| ApiError::NotFound)
            .map_err(|_| Response::default())?
            .ok_or(ApiError::NotFound)
            .map_err(|_| Response::default())?;

        // 5. Verifica se o DID é o autor (para operações de escrita)
        if proposal.author_did != did {
            return Err(Response::default());
        }

        // 6. Verifica a assinatura (payload = JSON da proposta)
        let payload = serde_json::to_vec(&proposal).map_err(|_| ApiError::InvalidPayload).map_err(|_| Response::default())?;
        verify_auth(&did, &signature, &payload).await.map_err(|_| Response::default())?;

        Ok(AuthenticatedProposal { proposal, did, signature })
    }
}

/// Extrator apenas para DID + assinatura (sem proposta)
pub struct AuthenticatedDid {
    pub did: String,
    pub signature: Vec<u8>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedDid
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let did = parts
            .headers
            .get("X-DID")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::MissingDid)
            .map_err(|_| Response::default())?
            .to_string();

        let signature = parts
            .headers
            .get("X-Signature")
            .and_then(|v| hex::decode(v.as_bytes()).ok())
            .ok_or_else(|| ApiError::MissingSignature)
            .map_err(|_| Response::default())?;

        Ok(AuthenticatedDid { did, signature })
    }
}
