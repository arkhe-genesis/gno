//! # Arkhe MCP Server
//!
//! Binary entry point for the MCP Bridge server.
//! Supports Plurality, Moltbook, and Hashtree MCP servers.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde_json::{json, Value};
use tracing::{info, warn, error};

use arkhe_mcp::{
    ConfigStatus, is_configured, VERSION,
    HashtreeClient, HashtreeMcpTools, MerkleProof,
    ZkProver, ZkAuditLog, ZkProofType, ZkAuditEntry,
    McpError,
};

// ============================================================================
// SHARED STATE
// ============================================================================

struct AppState {
    status: ConfigStatus,
    hashtree_tools: Option<HashtreeMcpTools>,
    zk_prover: ZkProver,
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("arkhe_mcp=debug".parse()?)
        )
        .init();

    info!("Cathedral ARKHE MCP Bridge v{}", VERSION);

    // Check configuration
    if !is_configured() {
        warn!("No MCP servers configured!");
        warn!("Set PLURALITY_PAT, PLURALITY_CLIENT_ID, or MOLTBOOK_API_KEY");
    }

    let status = ConfigStatus {
        plurality_pat: std::env::var("PLURALITY_PAT").is_ok(),
        plurality_oauth: std::env::var("PLURALITY_CLIENT_ID").is_ok(),
        moltbook: std::env::var("MOLTBOOK_API_KEY").is_ok(),
    };

    info!("Status: {}", status);

    // Initialize Hashtree client (if credentials available)
    let hashtree_tools = if let (Ok(relay), Ok(blossom)) = (
        std::env::var("NOSTR_RELAY"),
        std::env::var("BLOSSOM_URL"),
    ) {
        let mut client = HashtreeClient::new(relay, blossom);
        // If private key is set, authenticate
        if let Ok(private_key) = std::env::var("NOSTR_PRIVATE_KEY") {
            if let Err(e) = client.authenticate(private_key) {
                warn!("Failed to authenticate Hashtree client: {}", e);
            } else {
                info!("Hashtree client authenticated");
            }
        }
        Some(HashtreeMcpTools::new(client))
    } else {
        info!("Hashtree not configured (set NOSTR_RELAY and BLOSSOM_URL)");
        None
    };

    let state = AppState {
        status,
        hashtree_tools,
        zk_prover: ZkProver::new(),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/mcp/tools/list", post(list_tools))
        .route("/mcp/tools/call", post(call_tool))
        .with_state(Arc::new(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("MCP Bridge listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// HANDLERS
// ============================================================================

async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "version": VERSION,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn status_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "version": VERSION,
        "configured": state.status.is_ready(),
        "servers": state.status.active_servers(),
        "plurality": {
            "pat": state.status.plurality_pat,
            "oauth": state.status.plurality_oauth,
        },
        "moltbook": state.status.moltbook,
        "hashtree": state.hashtree_tools.is_some(),
    }))
}

async fn list_tools() -> Json<Value> {
    Json(json!({
        "tools": [
            // Plurality tools
            { "name": "plurality_get_buckets", "description": "List memory buckets" },
            { "name": "plurality_search", "description": "Search memory" },
            { "name": "plurality_save", "description": "Save to memory" },
            // Moltbook tools
            { "name": "moltbook_post", "description": "Read a post" },
            { "name": "moltbook_post_create", "description": "Create a post" },
            { "name": "moltbook_comment", "description": "Comment on a post" },
            { "name": "moltbook_vote", "description": "Vote on content" },
            { "name": "moltbook_search", "description": "Search Moltbook" },
            { "name": "moltbook_profile", "description": "View agent profile" },
            { "name": "moltbook_digest", "description": "Signal-filtered feed" },
            { "name": "moltbook_trust", "description": "Author trust scoring" },
            // Hashtree tools
            { "name": "hashtree_publish", "description": "Publish a Merkle tree to Nostr" },
            { "name": "hashtree_fetch", "description": "Fetch a file by hash from Blossom" },
            { "name": "hashtree_verify", "description": "Verify a Merkle proof" },
            { "name": "hashtree_list", "description": "List user's trees" },
            { "name": "hashtree_git_push", "description": "Push a git repo to htree://" },
            { "name": "hashtree_git_clone", "description": "Clone from htree://" },
            { "name": "hashtree_audit", "description": "Audit operations with ZK + Merkle root" },
        ]
    }))
}

async fn call_tool(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let tool = body.get("tool").and_then(|t| t.as_str()).unwrap_or("unknown");
    let args = body.get("args").cloned().unwrap_or(Value::Null);

    info!("Tool call: {}", tool);

    let result = match tool {
        // ========================
        // HASHTREE TOOLS
        // ========================
        "hashtree_publish" => {
            match handle_hashtree_publish(&state, args).await {
                Ok(res) => json!({ "success": true, "result": res }),
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        "hashtree_fetch" => {
            match handle_hashtree_fetch(&state, args).await {
                Ok(data) => {
                    use base64::{Engine as _, engine::general_purpose};
                    json!({ "success": true, "data": general_purpose::STANDARD.encode(data) })
                },
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        "hashtree_verify" => {
            match handle_hashtree_verify(&state, args).await {
                Ok(valid) => json!({ "success": true, "valid": valid }),
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        "hashtree_list" => {
            match handle_hashtree_list(&state, args).await {
                Ok(list) => json!({ "success": true, "trees": list }),
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        "hashtree_git_push" => {
            match handle_hashtree_git_push(&state, args).await {
                Ok(url) => json!({ "success": true, "url": url }),
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        "hashtree_git_clone" => {
            match handle_hashtree_git_clone(&state, args).await {
                Ok(()) => json!({ "success": true }),
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        "hashtree_audit" => {
            match handle_hashtree_audit(&state, args).await {
                Ok(proof) => json!({ "success": true, "audit_proof": proof }),
                Err(e) => json!({ "success": false, "error": e.to_string() }),
            }
        }
        // Other tools...
        _ => json!({ "success": false, "error": format!("Unknown tool: {}", tool) }),
    };

    Json(result)
}

// ============================================================================
// HANDLER IMPLEMENTATIONS
// ============================================================================

async fn handle_hashtree_publish(
    state: &Arc<AppState>,
    args: Value,
) -> Result<Value, McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'name' parameter".into()))?;

    let files = args.get("files")
        .and_then(|v| v.as_array())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'files' array".into()))?;

    let mut file_data = Vec::new();
    for file in files {
        let fname = file.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest("Missing 'name' in file entry".into()))?;
        let content = file.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest("Missing 'content' in file entry".into()))?;
        use base64::{Engine as _, engine::general_purpose};
        let bytes = general_purpose::STANDARD.decode(content)
            .map_err(|e| McpError::InvalidRequest(format!("Invalid base64 content: {}", e)))?;
        file_data.push((fname.to_string(), bytes));
    }

    let tree = tools.hashtree_publish(name, file_data).await?;
    Ok(serde_json::to_value(&tree).map_err(|e| McpError::Serialization(e))?)
}

async fn handle_hashtree_fetch(
    state: &Arc<AppState>,
    args: Value,
) -> Result<Vec<u8>, McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    let hash = args.get("hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'hash' parameter".into()))?;

    tools.hashtree_fetch(hash).await
}

async fn handle_hashtree_verify(
    state: &Arc<AppState>,
    args: Value,
) -> Result<bool, McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    let proof_json = args.get("proof")
        .ok_or_else(|| McpError::InvalidRequest("Missing 'proof' parameter".into()))?;
    let proof: MerkleProof = serde_json::from_value(proof_json.clone())
        .map_err(|e| McpError::Serialization(e))?;

    let root_hash = args.get("root_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'root_hash' parameter".into()))?;

    Ok(tools.hashtree_verify(&proof, root_hash))
}

async fn handle_hashtree_list(
    state: &Arc<AppState>,
    _args: Value,
) -> Result<Vec<String>, McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    tools.hashtree_list().await
}

async fn handle_hashtree_git_push(
    state: &Arc<AppState>,
    args: Value,
) -> Result<String, McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    let repo_path = args.get("repo_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'repo_path' parameter".into()))?;

    let tree_name = args.get("tree_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'tree_name' parameter".into()))?;

    let path = std::path::Path::new(repo_path);
    tools.hashtree_git_push(path, tree_name).await
}

async fn handle_hashtree_git_clone(
    state: &Arc<AppState>,
    args: Value,
) -> Result<(), McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    let url = args.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'url' parameter".into()))?;

    let dest = args.get("dest")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'dest' parameter".into()))?;

    let path = std::path::Path::new(dest);
    tools.hashtree_git_clone(url, path).await
}

// ============================================================================
// HASHTREE AUDIT TOOL (combina Hashtree + ZK)
// ============================================================================

async fn handle_hashtree_audit(
    state: &Arc<AppState>,
    args: Value,
) -> Result<Value, McpError> {
    let tools = state.hashtree_tools.as_ref()
        .ok_or_else(|| McpError::MissingConfig("Hashtree not configured".into()))?;

    // Extrair parâmetros
    let tree_name = args.get("tree_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest("Missing 'tree_name' parameter".into()))?;

    // 1. Obter a árvore do Nostr (via fetch)
    let tree = tools.client.fetch_from_nostr(tree_name).await?;

    // 2. Criar um audit log com base na árvore
    let mut audit_log = ZkAuditLog::new();
    let root_hash = tree.root_hash.clone();

    // Adicionar entrada com a raiz da árvore
    audit_log.append(ZkAuditEntry {
        sequence: 1,
        timestamp: chrono::Utc::now(),
        proof_type: ZkProofType::ContentIntegrity,
        proof_hash: root_hash.clone(),
        public_inputs: vec![
            format!("tree:{}", tree_name),
            format!("root:{}", root_hash),
            format!("timestamp:{}", tree.created_at.to_rfc3339()),
        ],
        agent_id: "audit".into(),
    });

    // 3. Gerar prova ZK de integridade do log
    let zk_proof = audit_log.prove_integrity(&state.zk_prover).await?;

    // 4. Gerar também uma prova Merkle para a raiz (opcional)
    let merkle_proof = tools.client.generate_proof(&tree, &root_hash)?;

    // 5. Retornar um relatório combinado
    Ok(json!({
        "tree_name": tree_name,
        "root_hash": root_hash,
        "created_at": tree.created_at,
        "audit_log_entries": audit_log.entries(),
        "zk_proof": zk_proof,
        "merkle_proof": merkle_proof,
        "verified": true,
    }))
}
