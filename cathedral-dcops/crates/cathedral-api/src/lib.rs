pub mod extractors;
pub mod middleware;
pub mod ws;

pub struct AppState {
    pub wormgraph: std::sync::Arc<cathedral_wormgraph::WormGraphClient>,
    pub notifier: std::sync::Arc<cathedral_self_improve::BroadcastNotifier>,
}

pub enum ApiError {
    MissingDid,
    MissingSignature,
    MissingId,
    NotFound,
    Forbidden,
    InvalidPayload,
    AuthFailed,
}

pub mod auth {
    pub async fn verify_auth(_did: &str, _signature: &[u8], _payload: &[u8]) -> Result<bool, super::ApiError> {
        Ok(true)
    }
}

pub struct ProposalQueryParams {
    pub id: Option<String>,
}
