//! # Moltbook MCP Client
//!
//! Cliente Streamable HTTP para o servidor MCP do Moltbook.

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MoltbookClient {
    http: Client,
}

impl MoltbookClient {
    pub async fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}
