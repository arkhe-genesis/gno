use crate::cgf_metrics::{CgfEngine, CgfReportX, EpistemicLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CgfOrchestratorError {
    #[error("Nenhuma resposta recebida dos modelos")]
    NoResponses,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmModel {
    Claude4,
}

#[derive(Debug, Clone)]
pub struct CgfOrchestratorConfig {
    pub max_iterations: usize,
    pub convergence_threshold: f64,
}

impl Default for CgfOrchestratorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            convergence_threshold: 0.75,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgfRoundResult {
    pub round: usize,
    pub global_alpha: f64,
    pub dominant_level: EpistemicLevel,
}

pub struct CgfOrchestrator {
    pub config: CgfOrchestratorConfig,
}

impl CgfOrchestrator {
    pub fn new(config: CgfOrchestratorConfig) -> Self {
        Self {
            config,
        }
    }
}
