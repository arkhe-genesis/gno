use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);
impl AgentId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn generate() -> Self { Self(uuid::Uuid::new_v4().to_string()) }
}
impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
