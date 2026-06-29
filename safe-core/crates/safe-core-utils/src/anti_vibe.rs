//! 🚨 Anti-Vibe Attractor — Catálogo Formal de Falhas do Vibe-Coding (v3.0)
//!
//! Este módulo documenta 15 casos reais de falhas em código gerado por IA,
//! categoriza-os em 6 tipos de alucinação e fornece prompts específicos
//! para cada categoria, integrados ao Coherence-Gradient Following.
//!
//! # Convenção X
//! - `x_detect_vibe_awareness` — analisa respostas de fronteira (LLM)
//! - `AntiVibeScenario` — estrutura de dados imutável (core)
//! - `ANTI_VIBE_CATALOG` — catálogo público de cenários documentados

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// =============================================================================
// CATEGORIAS DE ALUCINAÇÃO
// =============================================================================

/// Categorias de alucinação em vibe-coding (taxonomia 2025-2026).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HallucinationCategory {
    /// LLMs inventam nomes de bibliotecas ou pacotes que não existem.
    /// Taxa: 26-99% de ocorrência.
    Library,
    /// Código parece funcionar mas remove verificações de segurança.
    /// 69 vulnerabilidades em 15 apps testadas.
    Security,
    /// Agentes "esquecem" o contexto entre prompts.
    /// Alto risco de código inconsistente.
    ContextMemory,
    /// Bugs de lógica: 10 padrões documentados (misinterpretations, syntax, etc.)
    LogicBehavior,
    /// Agentes mentem, escondem erros, deletam produção.
    /// Casos reais: Replit, Claude.
    AgentBehavioral,
    /// Produtividade em declínio, código ineficiente e ilegível.
    PerformanceQuality,
}

// =============================================================================
// ESTRUTURA DE DADOS
// =============================================================================

/// Cenário documentado de falha no vibe-coding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiVibeScenario {
    pub title: String,
    pub industry: String,
    pub time_lost_hours: u32,
    pub monetary_cost: Option<String>,
    pub bad_code_pattern: String,
    pub bad_code_example: String,
    pub safe_core_solution: String,
    pub safe_core_code_example: String,
    pub source: String,
    pub severity: u8,
    pub keywords: Vec<String>,
    pub category: HallucinationCategory,
}

// =============================================================================
// CATÁLOGO EXPANDIDO (15 CENÁRIOS)
// =============================================================================

pub const ANTI_VIBE_CATALOG: &[AntiVibeScenario] = &[
    // ---- Security ----
    AntiVibeScenario {
        title: String::new(),
        industry: String::new(),
        time_lost_hours: 6,
        monetary_cost: None,
        bad_code_pattern: String::new(),
        bad_code_example: String::new(),
        safe_core_solution: String::new(),
        safe_core_code_example: String::new(),
        source: String::new(),
        severity: 10,
        keywords: vec![],
        category: HallucinationCategory::Security,
    },
];

// =============================================================================
// PALAVRAS-CHAVE GLOBAIS
// =============================================================================

pub const ANTI_VIBE_KEYWORDS: &[&str] = &[
    "amazon", "outage", "roguelike", "debugging", "loop", "zero progress",
    "incomprehensible", "eldritch", "deploy", "migration", "corrupt",
    "rewrite", "revert", "assistants", "paradox", "misunderstanding",
    "legacy", "unmaintainable", "tpm", "mock", "hardware", "deadlock",
    "orchestration", "formal", "proof", "vibe-coding", "ai-generated",
    "production", "panic", "unwrapped", "safety", "validation", "XSS",
];

// =============================================================================
// FUNÇÕES DE FRONTEIRA (Convenção X)
// =============================================================================

/// [FRONTEIRA] Detecta se uma resposta demonstra awareness de falhas do vibe-coding.
pub fn x_detect_vibe_awareness(response: &str) -> f64 {
    let text_lower = response.to_lowercase();
    let mut detected = HashSet::new();
    for kw in ANTI_VIBE_KEYWORDS {
        if text_lower.contains(kw) {
            detected.insert(kw);
        }
    }
    let coverage = detected.len() as f64 / ANTI_VIBE_KEYWORDS.len() as f64;
    let length_penalty = if response.len() < 50 { 0.5 } else { 1.0 };
    (coverage * length_penalty).min(1.0)
}

/// Retorna o cenário mais severo do catálogo.
pub fn find_most_severe_scenario() -> &'static AntiVibeScenario {
    ANTI_VIBE_CATALOG.iter().max_by_key(|s| s.severity).unwrap()
}

/// Retorna o cenário mais relevante para uma categoria.
pub fn find_scenario_by_category(category: HallucinationCategory) -> Option<&'static AntiVibeScenario> {
    ANTI_VIBE_CATALOG.iter().find(|s| s.category == category)
}

/// Gera prompt específico para uma categoria.
pub fn generate_category_prompt(category: HallucinationCategory) -> String {
    let scenario = find_scenario_by_category(category).unwrap_or_else(|| find_most_severe_scenario());
    format!(
        r#"## 🚨 O Custo Real do Vibe-Coding — Caso: {}

**Indústria:** {}
**Tempo Perdido:** {} horas
**Custo Monetário:** {}

**O Problema:**
```rust
{}
```

**A Solução Safe-Core:**
```rust
{}
```

**Fonte:** {}

**Pergunta:** Como o Safe-Core — com sua SafetyBarrier imutável, verificação formal em Lean 4, Hardware Root of Trust, e Convenção X — transforma essas "{} horas de vibe-coding" em "segundos de verificação formal"?

Dê um exemplo concreto de como o Safe-Core teria prevenido este cenário específico, citando pelo menos um dos 5 pilares e a Convenção X.
"#,
        scenario.title,
        scenario.industry,
        scenario.time_lost_hours,
        scenario.monetary_cost.as_deref().unwrap_or("N/A"),
        scenario.bad_code_example,
        scenario.safe_core_code_example,
        scenario.source,
        scenario.time_lost_hours,
    )
}

/// Calcula total de horas economizadas baseado nos conceitos detectados.
pub fn calculate_total_savings(detected_concepts: &[String]) -> u32 {
    let mut total = 0;
    for scenario in ANTI_VIBE_CATALOG {
        let detected = scenario.keywords.iter().any(|kw| {
            detected_concepts.iter().any(|c| c.to_lowercase().contains(kw))
        });
        if detected { total += scenario.time_lost_hours; }
    }
    total
}
