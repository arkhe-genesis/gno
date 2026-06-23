use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use crate::AppState;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthInfo {
    pub did: String,
    pub signature: Vec<u8>,
}

pub struct AuthMiddleware;

pub async fn auth_middleware(
    State(_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Valida DID e assinatura, injeta no extensions
    let did = req.headers()
        .get("X-DID")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let signature = req.headers()
        .get("X-Signature")
        .and_then(|v| hex::decode(v.as_bytes()).ok());

    if let (Some(did), Some(sig)) = (did, signature) {
        // Verificação real (pode ser deferida para o handler)
        req.extensions_mut().insert(AuthInfo { did, signature: sig });
    }
    next.run(req).await
}
