use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolicyError { #[error("Tool not allowed: {0}")] NotAllowed(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal { pub tool: String, pub payload: serde_json::Value }

pub struct ConsensusGuard { allowed_tools: HashSet<String> }
impl ConsensusGuard {
    pub fn new() -> Self { Self { allowed_tools: ["infer", "read"].iter().map(|s| s.to_string()).collect() } }
    pub fn evaluate(&self, proposal: &Proposal) -> Result<bool, PolicyError> {
        if !self.allowed_tools.contains(&proposal.tool) {
            return Err(PolicyError::NotAllowed(proposal.tool.clone()));
        }
        Ok(true)
    }
}
impl Default for ConsensusGuard { fn default() -> Self { Self::new() } }
