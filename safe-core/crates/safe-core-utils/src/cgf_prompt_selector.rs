use crate::cgf_metrics::EpistemicLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PromptDepth {
    Shallow, // α̂ < 0.25
    Medium,  // 0.25 ≤ α̂ < 0.50
    Deep,    // 0.50 ≤ α̂ < 0.75
    Expert,  // α̂ ≥ 0.75
}

impl From<f64> for PromptDepth {
    fn from(alpha: f64) -> Self {
        match alpha {
            a if a < 0.25 => Self::Shallow,
            a if a < 0.50 => Self::Medium,
            a if a < 0.75 => Self::Deep,
            _ => Self::Expert,
        }
    }
}

pub struct PromptSelector;

impl PromptSelector {
    pub fn new() -> Self {
        Self
    }
}
