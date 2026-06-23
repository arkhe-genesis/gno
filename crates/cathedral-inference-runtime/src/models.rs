use serde::{Deserialize, Serialize};
use cathedral_zk::ZKProof;
use cathedral_wormgraph::ExecutionReceipt;

/// Níveis de verificação suportados pelo protótipo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    /// Baseline: apenas assinatura ML-DSA, sem ZK.
    L0,
    /// Light: amostragem de 5% das camadas + NANOZK simulado.
    L1,
    /// Standard: amostragem de 15% das camadas + DeepProve simulado.
    L2,
}

impl VerificationLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "L1" => VerificationLevel::L1,
            "L2" => VerificationLevel::L2,
            _ => VerificationLevel::L0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationLevel::L0 => "L0",
            VerificationLevel::L1 => "L1",
            VerificationLevel::L2 => "L2",
        }
    }

    /// Retorna a taxa de amostragem para cada nível.
    pub fn sample_rate(&self) -> f64 {
        match self {
            VerificationLevel::L0 => 0.0,
            VerificationLevel::L1 => 0.05,
            VerificationLevel::L2 => 0.15,
        }
    }
}

/// Requisição de geração.
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub prompt: String,
    pub did: String,
    pub signature: Vec<u8>,      // Assinatura da requisição (Ed25519)
    pub level: VerificationLevel,
    pub context: Option<Vec<String>>,
}

/// Resposta da geração.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub text: String,
    pub thinking: Option<String>,
    pub zk_proof: Option<ZKProof>,
    pub signature: Vec<u8>,        // Assinatura da resposta
    pub attestation: Vec<u8>,      // Header PqcAttestation (0xF8)
    pub receipt: ExecutionReceipt,
    pub latency_ms: u64,
    pub reputation: f64,
    pub tier: String,
}
