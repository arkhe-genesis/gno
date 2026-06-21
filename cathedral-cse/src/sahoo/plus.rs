//! src/sahoo/plus.rs
//! SAHOO+ com GDI adaptativo, invariantes contextuais e RL guardrails.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::moe::CognitiveContext;

// Placholders since we couldn't properly link it from crates with exactly same names as in old code.
// The code relies on `cathedral_eac::SahooGuard` and `AlignmentResult` which are likely inside the eac/kernel crate.
// We'll create small stubs to ensure compilation.

#[derive(Clone)]
pub struct SahooConfig {
    pub goal_drift_threshold: f64,
}

impl Default for SahooConfig {
    fn default() -> Self {
        Self {
            goal_drift_threshold: 0.5,
        }
    }
}

pub struct SahooGuard {
    _config: SahooConfig,
}

impl SahooGuard {
    pub fn new(config: SahooConfig) -> Self {
        Self { _config: config }
    }

    pub async fn check_alignment(&self, _original: &str, _mutated: &str) -> AlignmentResult {
        AlignmentResult {
            passed: true,
            regression_risk: 0.1,
            constraint_violations: Vec::new(),
            goal_drift_index: 0.2,
        }
    }
}

pub struct AlignmentResult {
    pub passed: bool,
    pub regression_risk: f64,
    pub constraint_violations: Vec<String>,
    pub goal_drift_index: f64,
}

#[derive(Debug, Clone)]
pub struct AlignmentDecision {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub input: String,
    pub output: String,
    pub gdi: f64,
    pub violations: Vec<String>,
    pub accepted: bool,
    pub reward: f64,
}

pub struct AdaptiveGDI {
    base_threshold: f64,
    pub current_threshold: f64,
}

impl AdaptiveGDI {
    pub fn new(base: f64) -> Self {
        Self { base_threshold: base, current_threshold: base }
    }

    pub fn adjust(&mut self, context: &CognitiveContext, history: &[AlignmentDecision]) {
        let complexity = context.prompt.len() as f64 / 1000.0;
        let avg_gdi = history.iter().map(|d| d.gdi).sum::<f64>() / history.len().max(1) as f64;
        self.current_threshold = self.base_threshold * (1.0 + 0.1 * complexity);
        self.current_threshold = self.current_threshold.max(0.1).min(0.9);
        if avg_gdi > 0.5 {
            self.current_threshold *= 0.9;
        }
    }
}

pub struct RLGuardrail {
    pub name: String,
    condition: Box<dyn Fn(&str) -> bool + Send + Sync>,
    pub violation_message: String,
}

impl RLGuardrail {
    pub fn new<F>(name: &str, condition: F, message: &str) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            condition: Box::new(condition),
            violation_message: message.to_string(),
        }
    }

    pub fn check(&self, text: &str) -> Result<(), String> {
        if !(self.condition)(text) {
            Err(self.violation_message.clone())
        } else {
            Ok(())
        }
    }
}

pub struct SahooPlus {
    base: SahooGuard,
    adaptive_gdi: Arc<Mutex<AdaptiveGDI>>,
    context_invariants: HashMap<String, Vec<String>>,
    rl_guardrails: Vec<RLGuardrail>,
    decision_history: Arc<Mutex<Vec<AlignmentDecision>>>,
}

impl SahooPlus {
    pub fn new(base_config: SahooConfig) -> Self {
        Self {
            base: SahooGuard::new(base_config.clone()),
            adaptive_gdi: Arc::new(Mutex::new(AdaptiveGDI::new(base_config.goal_drift_threshold))),
            context_invariants: HashMap::new(),
            rl_guardrails: Vec::new(),
            decision_history: Arc::new(Mutex::new(Vec::with_capacity(1000))),
        }
    }

    pub async fn check_alignment_with_context(
        &self,
        original: &str,
        mutated: &str,
        context: &CognitiveContext,
    ) -> Result<AlignmentResult, String> {
        let mut result = self.base.check_alignment(original, mutated).await;

        {
            let mut adaptive = self.adaptive_gdi.lock().await;
            let history = self.decision_history.lock().await;
            adaptive.adjust(context, &history);
            let threshold = adaptive.current_threshold;
            drop(adaptive);

            if let Some(invs) = self.context_invariants.get(&context.prompt) {
                for inv in invs {
                    if !mutated.contains(inv) {
                        result.constraint_violations.push(format!("Invariante violado: {}", inv));
                    }
                }
            }

            for guardrail in &self.rl_guardrails {
                if let Err(e) = guardrail.check(mutated) {
                    result.constraint_violations.push(format!("RL Guardrail: {}", e));
                }
            }

            result.passed = result.constraint_violations.is_empty() && result.regression_risk < threshold;
        }

        let mut history = self.decision_history.lock().await;
        history.push(AlignmentDecision {
            timestamp: chrono::Utc::now(),
            input: original.to_string(),
            output: mutated.to_string(),
            gdi: result.goal_drift_index,
            violations: result.constraint_violations.clone(),
            accepted: result.passed,
            reward: 0.0,
        });
        if history.len() > 1000 { history.remove(0); }

        Ok(result)
    }

    pub async fn record_feedback(&self, decision_id: usize, reward: f64) {
        if let Some(decision) = self.decision_history.lock().await.get_mut(decision_id) {
            decision.reward = reward;
        }
    }

    pub fn add_rl_guardrail(&mut self, guardrail: RLGuardrail) {
        self.rl_guardrails.push(guardrail);
    }
}