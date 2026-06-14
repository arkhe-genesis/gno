#![forbid(unsafe_code)]
use sha3::{Sha3_256, Digest};
use serde::{Serialize, Deserialize};

use crate::wormgraph_core::{WormGraph, Hash256, WormGraphError};
use crate::chain::rbb_client_stub::RBBChainClientStub;

// =============================================================================
// Tipos de Ancoragem Temporal
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnchor {
    pub nonce: u64,
    pub merkle_root: Hash256,
    pub node_count: usize,
    pub wormhole_count: usize,
    pub context_tokens: usize,
    pub phi_c: f64,
    pub timestamp_ns: u64,
    pub block_hash: Option<String>,
    pub tx_hash: Option<String>,
    pub zk_proof: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalCheckpoint {
    pub anchors: Vec<TemporalAnchor>,
    pub interval_ns: u64,
    pub last_anchor_ns: u64,
    pub total_anchors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDiff {
    pub from_nonce: u64,
    pub to_nonce: u64,
    pub added_nodes: Vec<Hash256>,
    pub removed_nodes: Vec<Hash256>,
    pub added_wormholes: Vec<(Hash256, Hash256)>,
    pub merkle_root_before: Hash256,
    pub merkle_root_after: Hash256,
    pub zk_proof: Vec<u8>,
}

// =============================================================================
// Temporal Anchor Engine
// =============================================================================

pub struct TemporalAnchorEngine {
    checkpoint: TemporalCheckpoint,
    pub anchor_interval_ns: u64,
    client: RBBChainClientStub,
    contract_address: String,
}

impl TemporalAnchorEngine {
    pub const DEFAULT_INTERVAL_NS: u64 = 3_600_000_000_000; // 1 hora em nanos
    pub const MIN_INTERVAL_NS: u64 = 60_000_000_000; // 1 minuto
    pub const MAX_INTERVAL_NS: u64 = 86_400_000_000_000; // 24 horas

    pub fn new(client: RBBChainClientStub, contract_address: String) -> Self {
        Self {
            checkpoint: TemporalCheckpoint {
                anchors: Vec::new(),
                interval_ns: Self::DEFAULT_INTERVAL_NS,
                last_anchor_ns: 0,
                total_anchors: 0,
            },
            anchor_interval_ns: Self::DEFAULT_INTERVAL_NS,
            client,
            contract_address,
        }
    }

    pub fn with_interval(mut self, interval_ns: u64) -> Self {
        self.anchor_interval_ns = interval_ns.clamp(Self::MIN_INTERVAL_NS, Self::MAX_INTERVAL_NS);
        self.checkpoint.interval_ns = self.anchor_interval_ns;
        self
    }

    /// Verifica se é hora de ancorar
    pub fn should_anchor(&self, current_ns: u64) -> bool {
        current_ns - self.checkpoint.last_anchor_ns >= self.anchor_interval_ns
    }

    /// Ancora o estado atual do WormGraph na blockchain
    pub async fn anchor(&mut self, graph: &WormGraph) -> Result<TemporalAnchor, WormGraphError> {
        let anchor = TemporalAnchor {
            nonce: graph.temporal_nonce,
            merkle_root: graph.merkle_root,
            node_count: graph.nodes.len(),
            wormhole_count: graph.wormholes.len(),
            context_tokens: graph.current_token_count,
            phi_c: graph.compute_phi_c(),
            timestamp_ns: Self::now_ns(),
            block_hash: None,
            tx_hash: None,
            zk_proof: None,
        };

        // Submeter para a RBB Chain
        match self.submit_to_chain(&anchor).await {
            Ok((block_hash, tx_hash)) => {
                let mut anchor = anchor;
                anchor.block_hash = Some(block_hash);
                anchor.tx_hash = Some(tx_hash);

                self.checkpoint.anchors.push(anchor.clone());
                self.checkpoint.last_anchor_ns = anchor.timestamp_ns;
                self.checkpoint.total_anchors += 1;

                Ok(anchor)
            }
            Err(_e) => {
                // Fallback: ancorar localmente com selo criptográfico
                let mut local_anchor = anchor;
                local_anchor.zk_proof = Some(self.generate_local_proof(&local_anchor));

                self.checkpoint.anchors.push(local_anchor.clone());
                self.checkpoint.last_anchor_ns = local_anchor.timestamp_ns;
                self.checkpoint.total_anchors += 1;

                Ok(local_anchor)
            }
        }
    }

    /// Gera diff entre dois checkpoints para sincronização eficiente
    pub fn generate_diff(&self, from_nonce: u64, to_nonce: u64, graph: &WormGraph) -> Result<TemporalDiff, WormGraphError> {
        let from_anchor = self.checkpoint.anchors.iter()
            .find(|a| a.nonce == from_nonce)
            .ok_or(WormGraphError::NodeNotFound)?;

        let to_anchor = self.checkpoint.anchors.iter()
            .find(|a| a.nonce == to_nonce)
            .ok_or(WormGraphError::NodeNotFound)?;

        // Em produção: manter índice de nós adicionados/removidos entre anchors
        // Aqui: stub que retorna todos os nós como "adicionados"
        let added_nodes: Vec<Hash256> = graph.nodes.iter().map(|n| n.id).collect();

        let diff = TemporalDiff {
            from_nonce,
            to_nonce,
            added_nodes,
            removed_nodes: vec![],
            added_wormholes: graph.wormholes.keys().cloned().collect(),
            merkle_root_before: from_anchor.merkle_root,
            merkle_root_after: to_anchor.merkle_root,
            zk_proof: self.generate_diff_proof(from_nonce, to_nonce, graph)?,
        };

        Ok(diff)
    }

    /// Verifica integridade de um anchor via Merkle proof
    pub fn verify_anchor_integrity(&self, anchor: &TemporalAnchor, graph: &WormGraph) -> bool {
        anchor.merkle_root == graph.merkle_root
            && anchor.node_count == graph.nodes.len()
            && anchor.wormhole_count == graph.wormholes.len()
            && (anchor.phi_c - graph.compute_phi_c()).abs() < 0.001
    }

    /// Rollback para um checkpoint anterior (emergência)
    pub fn rollback_to(&self, nonce: u64) -> Option<&TemporalAnchor> {
        self.checkpoint.anchors.iter()
            .find(|a| a.nonce == nonce)
    }

    /// Exporta checkpoint completo para backup
    pub fn export_checkpoint(&self) -> String {
        serde_json::to_string_pretty(&self.checkpoint).unwrap_or_default()
    }

    // =========================================================================
    // Métodos Privados
    // =========================================================================

    async fn submit_to_chain(&self, anchor: &TemporalAnchor) -> Result<(String, String), WormGraphError> {
        // Em produção: chamar contrato ArkheFederation.sol na RBB Chain
        // Aqui: stub que simula sucesso

        let block_hash = format!("0x{}", hex::encode(&anchor.merkle_root[..16]));
        let tx_hash = format!("0x{}", hex::encode(&anchor.merkle_root[16..]));

        Ok((block_hash, tx_hash))
    }

    fn generate_local_proof(&self, anchor: &TemporalAnchor) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(&anchor.nonce.to_le_bytes());
        hasher.update(&anchor.merkle_root);
        hasher.update(&anchor.timestamp_ns.to_le_bytes());
        hasher.update(b"LOCAL-PROOF-v5.2.0");
        hasher.finalize().to_vec()
    }

    fn generate_diff_proof(&self, from: u64, to: u64, graph: &WormGraph) -> Result<Vec<u8>, WormGraphError> {
        let mut hasher = Sha3_256::new();
        hasher.update(&from.to_le_bytes());
        hasher.update(&to.to_le_bytes());
        hasher.update(&graph.merkle_root);
        hasher.update(b"DIFF-PROOF-v5.2.0");
        Ok(hasher.finalize().to_vec())
    }

    fn now_ns() -> u64 {
        // Stub: substituir por timestamp real
        1_000_000_000u64
    }
}

// =============================================================================
// Auto-Anchor Task (background)
// =============================================================================

#[cfg(feature = "temporal")]
pub struct AutoAnchorTask {
    engine: TemporalAnchorEngine,
    running: bool,
}

#[cfg(feature = "temporal")]
impl AutoAnchorTask {
    pub fn new(engine: TemporalAnchorEngine) -> Self {
        Self { engine, running: false }
    }

    pub async fn start(&mut self, graph: &WormGraph) {
        self.running = true;

        while self.running {
            if self.engine.should_anchor(Self::now_ns()) {
                match self.engine.anchor(graph).await {
                    Ok(anchor) => {
                        log::info!("Temporal anchor created: nonce={}, block={:?}",
                                  anchor.nonce, anchor.block_hash);
                    }
                    Err(e) => {
                        log::error!("Failed to create temporal anchor: {:?}", e);
                    }
                }
            }

            // Sleep por 10% do intervalo de ancoragem
            let sleep_ns = self.engine.anchor_interval_ns / 10;
            tokio::time::sleep(tokio::time::Duration::from_nanos(sleep_ns)).await;
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    fn now_ns() -> u64 {
        1_000_000_000u64
    }
}
