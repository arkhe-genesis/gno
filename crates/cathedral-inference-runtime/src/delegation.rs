use cathedral_llm_core::ModelTier;

/// Router de delegação que seleciona o tier do modelo baseado na reputação.
pub struct DelegationRouter {
    thresholds: Vec<f64>,          // [90, 70, 50] -> Pro, Plus, Standard, Lite
    tiers: Vec<ModelTier>,
}

impl DelegationRouter {
    pub fn new() -> Self {
        Self {
            thresholds: vec![90.0, 70.0, 50.0],
            tiers: vec![
                ModelTier::Pro,
                ModelTier::Plus,
                ModelTier::Standard,
                ModelTier::Lite,
            ],
        }
    }

    /// Seleciona o tier baseado na pontuação de reputação.
    pub fn select(&self, reputation: f64) -> ModelTier {
        for (i, &threshold) in self.thresholds.iter().enumerate() {
            if reputation >= threshold {
                return self.tiers[i].clone();
            }
        }
        ModelTier::Lite
    }

    /// Retorna os thresholds atuais (para debug).
    pub fn thresholds(&self) -> &[f64] {
        &self.thresholds
    }
}
