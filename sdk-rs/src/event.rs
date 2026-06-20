use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusEvent {
    pub event_id: String,
    pub timestamp_ns: u64,
    pub event_type: EventType,
    pub project_id: String,
    pub agent_id: String,
    pub design_hash: String,
    pub parent_hashes: Vec<String>,
    pub metadata: EventMetadata,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    DesignProposed,
    SimulationCompleted,
    DesignOptimized,
    AgentMutation,
    FabricationPlanned,
    TestResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub domain: String,
    pub confidence: f64,
    pub compute_cost_usd: f64,
    pub tags: Vec<String>,
}
