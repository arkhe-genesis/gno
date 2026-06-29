use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EpistemicLevel {
    Level1, // α̂ < 0.25
    Level2, // 0.25 ≤ α̂ < 0.50
    Level3, // 0.50 ≤ α̂ < 0.75
    Level4, // α̂ ≥ 0.75
}

impl EpistemicLevel {
    pub fn from_alpha(alpha: f64) -> Self {
        match alpha {
            a if a < 0.25 => Self::Level1,
            a if a < 0.50 => Self::Level2,
            a if a < 0.75 => Self::Level3,
            _ => Self::Level4,
        }
    }
}

pub const SAFE_CORE_CONCEPTS: &[&str] = &[
    "unfireable", "safety kernel", "barreira imutável", "fail-closed",
];

pub const CONCEPT_WEIGHTS: &[(&str, f64)] = &[
    ("unfireable", 1.5), ("safety kernel", 1.5), ("fail-closed", 1.3),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReport {
    pub session_id: String,
    pub model_name: String,
    pub timestamp: u64,
    pub detected_concepts: Vec<String>,
    pub citation_count: usize,
    pub semantic_depth: f64,
    pub alpha_hat: f64,
    pub epistemic_level: EpistemicLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgfReportX {
    pub total_sessions: usize,
    pub global_alpha_hat: f64,
    pub alpha_trend: f64,
    pub inter_session_consistency: f64,
    pub dominant_level: EpistemicLevel,
    pub timestamp: u64,
}

pub struct CgfEngine {
    pub max_history: usize,
}

impl CgfEngine {
    pub fn new(max_history: usize) -> Self {
        Self {
            max_history,
        }
    }

    pub fn x_measure_session(
        &mut self,
        session_id: &str,
        model_name: &str,
        _response_text: &str,
    ) -> SessionReport {
        SessionReport {
            session_id: session_id.to_string(),
            model_name: model_name.to_string(),
            timestamp: 0,
            detected_concepts: vec![],
            citation_count: 0,
            semantic_depth: 0.0,
            alpha_hat: 0.0,
            epistemic_level: EpistemicLevel::Level1,
        }
    }
}
