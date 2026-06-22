//! # Arkhe MCP Bridge
//!
//! MCP (Model Context Protocol) integration for Cathedral ARKHE.
//! Provides connectivity to multiple MCP-compatible services:
//! - Plurality Network (memory/context)
//! - Moltbook (social network for AI agents)
//!
//! ## Features
//! - **Multi-server support**: Connect to multiple MCP servers simultaneously
//! - **Plurality MCP Client**: 7 tools for memory/context management
//! - **Moltbook MCP Client**: 18 tools for agent social interaction
//! - **ZK Proofs**: Zero-knowledge proofs for MCP operation attestation
//! - **OAuth 2.1 + PKCE**: Secure authentication
//! - **JWKS Validation**: Local JWT validation
//! - **Circuit Breaker**: Resilience against failures
//! - **Exponential Backoff**: Intelligent retry
//! - **Distributed Tracing**: OpenTelemetry-compatible spans

pub mod error;
pub mod types;
pub mod zk;
pub mod hashtree;

pub use error::{McpError, Result};
pub use types::*;
pub use zk::{ZkProver, ZkProof, ZkProofType, ZkAuditLog, ZkAuditEntry};
pub use hashtree::*;

/// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if MCP bridge is configured
pub fn is_configured() -> bool {
    std::env::var("PLURALITY_PAT").is_ok()
        || std::env::var("MOLTBOOK_API_KEY").is_ok()
}

/// Configuration status
pub fn config_status() -> ConfigStatus {
    ConfigStatus {
        plurality_pat: std::env::var("PLURALITY_PAT").is_ok(),
        plurality_oauth: std::env::var("PLURALITY_CLIENT_ID").is_ok(),
        moltbook: std::env::var("MOLTBOOK_API_KEY").is_ok(),
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStatus {
    pub plurality_pat: bool,
    pub plurality_oauth: bool,
    pub moltbook: bool,
}

impl ConfigStatus {
    pub fn is_ready(&self) -> bool {
        self.plurality_pat || self.plurality_oauth || self.moltbook
    }

    pub fn active_servers(&self) -> Vec<&'static str> {
        let mut servers = Vec::new();
        if self.plurality_pat || self.plurality_oauth {
            servers.push("plurality");
        }
        if self.moltbook {
            servers.push("moltbook");
        }
        servers
    }
}

impl std::fmt::Display for ConfigStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MCP Servers: ")?;
        let servers = self.active_servers();
        if servers.is_empty() {
            write!(f, "none configured")
        } else {
            write!(f, "{}", servers.join(", "))
        }
    }
}

pub mod moltbook_client;
pub use moltbook_client::*;
