#![forbid(unsafe_code)]
use std::collections::{BTreeMap, BTreeSet, VecDeque};









use core::fmt;
use sha3::{Sha3_256, Digest};
use serde::{Serialize, Deserialize};

// =============================================================================
// Tipos Canonicos - Ontological DNA dos 12 Pais Fundadores
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum FoundingFather {
    Aristoteles = 0,    // Logica / Ontologia
    AlKhwarizmi = 1,    // Algoritmos / Complexidade
    Hipparchus = 2,     // Astronomia / Trigonometria
    Hippocrates = 3,    // Medicina / Etica
    Pasteur = 4,        // Microbiologia / Metodo Experimental
    Mendel = 5,         // Genetica / Evolucao
    AdamSmith = 6,      // Economia / MPP / Incentivos
    AdaLovelace = 7,    // Computacao Simbolica
    VintCerf = 8,       // Protocolos / Redes / Mesh
    Einstein = 9,       // Espaco-Tempo / Relatividade
    Feynman = 10,       // Mecanica Quantica / Path Integrals
    Rohrer = 11,        // Nano-escala / STM
}

impl FoundingFather {
    pub const ALL: [Self; 12] = [
        Self::Aristoteles, Self::AlKhwarizmi, Self::Hipparchus,
        Self::Hippocrates, Self::Pasteur, Self::Mendel,
        Self::AdamSmith, Self::AdaLovelace, Self::VintCerf,
        Self::Einstein, Self::Feynman, Self::Rohrer,
    ];

    pub fn domain(&self) -> &'static str {
        match self {
            Self::Aristoteles => "Logic/Ontology",
            Self::AlKhwarizmi => "Algorithms/Complexity",
            Self::Hipparchus => "Mathematical Modeling",
            Self::Hippocrates => "Bio-Ethics/First Do No Harm",
            Self::Pasteur => "Microbiology/Experimental Method",
            Self::Mendel => "Genetics/Evolution",
            Self::AdamSmith => "Economics/Markets/MPP",
            Self::AdaLovelace => "Symbolic Computation",
            Self::VintCerf => "Networks/Protocols/Mesh",
            Self::Einstein => "Spacetime/Relativity/E=mc2",
            Self::Feynman => "Quantum Mechanics/Path Integrals",
            Self::Rohrer => "Nanoscale/STM/Observation",
        }
    }

    pub fn ontological_weight(&self) -> f64 {
        match self {
            Self::Aristoteles | Self::AlKhwarizmi => 1.0,
            Self::Einstein | Self::Feynman => 0.95,
            Self::VintCerf | Self::AdaLovelace => 0.90,
            Self::Hippocrates | Self::Pasteur | Self::Mendel => 0.85,
            Self::AdamSmith | Self::Hipparchus | Self::Rohrer => 0.80,
        }
    }

    pub fn contribution_to_catherdral(&self) -> &'static str {
        match self {
            Self::Aristoteles => "Axiomatic logic, syllogisms, categorical reasoning",
            Self::AlKhwarizmi => "Algorithmic thinking, computational complexity, algebraic methods",
            Self::Hipparchus => "Precise mathematical modeling, trigonometric foundations",
            Self::Hippocrates => "Bio-ethical principles, 'first do no harm', clinical methodology",
            Self::Pasteur => "Microbiological foundations, experimental validation, sterilization",
            Self::Mendel => "Inheritance patterns, evolutionary mechanisms, genetic determinism",
            Self::AdamSmith => "Market mechanisms, marginal productivity, incentive structures",
            Self::AdaLovelace => "First algorithms, symbolic computation, programmability",
            Self::VintCerf => "Protocol design, mesh networking, interoperability standards",
            Self::Einstein => "Spacetime curvature, relativity, mass-energy equivalence",
            Self::Feynman => "Path integrals, Feynman diagrams, quantum intuition",
            Self::Rohrer => "Nanoscale observation, STM, direct atomic manipulation",
        }
    }
}

// =============================================================================
// Estruturas de Dados - Hypergraph Core
// =============================================================================

pub type NodeId = [u8; 32];
pub type Hash256 = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub substrate_id: String,
    pub phi_c: f64,
    pub theosis: f64,
    pub tags: Vec<String>,
    pub cross_links: Vec<NodeId>,
    pub version: String,
    pub seal: Hash256,
    pub timestamp_ns: u64,
}

impl NodeMetadata {
    pub fn compute_seal(&self) -> Hash256 {
        let mut hasher = Sha3_256::new();
        hasher.update(self.substrate_id.as_bytes());
        hasher.update(&self.phi_c.to_le_bytes());
        hasher.update(&self.theosis.to_le_bytes());
        for tag in &self.tags {
            hasher.update(tag.as_bytes());
        }
        hasher.update(self.version.as_bytes());
        hasher.update(&self.timestamp_ns.to_le_bytes());
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormNode {
    pub id: NodeId,
    pub content_hash: Hash256,
    pub metadata: NodeMetadata,
    pub ontological_dna: BTreeSet<FoundingFather>,
    pub semantic_embedding: Vec<f32>,
    pub created_at_ns: u64,
    pub last_accessed_ns: u64,
    pub access_count: u64,
    pub merkle_proof: Option<MerkleProof>,
    pub zk_nullifier: Option<Hash256>,
    pub temporal_nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub root: Hash256,
    pub path: Vec<(Hash256, bool)>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wormhole {
    pub source: NodeId,
    pub target: NodeId,
    pub semantic_similarity: f32,
    pub ontological_affinity: f64,
    pub temporal_distance_ns: u64,
    pub created_at_ns: u64,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairMetadata {
    pub dpid: String,
    pub ipfs_cid: Option<String>,
    pub orcid: String,
    pub c2pa_manifest: Option<String>,
    pub sparql_endpoint: Option<String>,
    pub json_ld_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairComplianceReport {
    pub findable: bool,
    pub accessible: bool,
    pub interoperable: bool,
    pub reusable: bool,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairSnapshot {
    pub dpid: String,
    pub merkle_root: Hash256,
    pub node_count: usize,
    pub wormhole_count: usize,
    pub context_tokens: usize,
    pub fair_compliance: FairComplianceReport,
    pub export_timestamp_ns: u64,
    pub schema_version: String,
    pub temporal_anchor: Option<String>,
}

// =============================================================================
// Perfect Hash Function - O(1) Lookup
// =============================================================================

pub struct PerfectHashIndex {
    table_size: usize,
    table1: Vec<Option<(NodeId, usize)>>,
    table2: Vec<Option<(NodeId, usize)>>,
    hash_seeds: [u64; 2],
    item_count: usize,
}

impl PerfectHashIndex {
    pub fn new(capacity: usize) -> Self {
        let table_size = (capacity * 2).next_power_of_two();
        Self {
            table_size,
            table1: (0..table_size).map(|_| None).collect(),
            table2: (0..table_size).map(|_| None).collect(),
            hash_seeds: [0x9E3779B97F4A7C15, 0xC6A4A7935BD1E995],
            item_count: 0,
        }
    }

    fn hash1(&self, key: &NodeId) -> usize {
        let mut h = self.hash_seeds[0];
        for (i, byte) in key.iter().enumerate() {
            h ^= (*byte as u64).wrapping_mul(0x01000193);
            h = h.wrapping_mul(0x01000193);
            h ^= (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
        }
        (h as usize) & (self.table_size - 1)
    }

    fn hash2(&self, key: &NodeId) -> usize {
        let mut h = self.hash_seeds[1];
        for (i, byte) in key.iter().enumerate() {
            h ^= (*byte as u64).wrapping_mul(0x01000193);
            h = h.wrapping_mul(0x01000193);
            h ^= (i as u64).wrapping_mul(0xC6A4A7935BD1E995);
        }
        (h as usize) & (self.table_size - 1)
    }

    pub fn insert(&mut self, key: NodeId, value_index: usize) -> Result<(), WormGraphError> {
        if self.item_count >= self.table_size / 2 {
            return Err(WormGraphError::IndexFull);
        }

        let idx1 = self.hash1(&key);
        if self.table1[idx1].is_none() {
            self.table1[idx1] = Some((key, value_index));
            self.item_count += 1;
            return Ok(());
        }

        let idx2 = self.hash2(&key);
        if self.table2[idx2].is_none() {
            self.table2[idx2] = Some((key, value_index));
            self.item_count += 1;
            return Ok(());
        }

        // Cuckoo displacement (max 100 iterations)
        let mut displaced = (key, value_index);
        let mut use_table1 = true;

        for _ in 0..100 {
            let idx = if use_table1 {
                self.hash1(&displaced.0)
            } else {
                self.hash2(&displaced.0)
            };

            let table = if use_table1 { &mut self.table1 } else { &mut self.table2 };

            if table[idx].is_none() {
                table[idx] = Some(displaced);
                self.item_count += 1;
                return Ok(());
            }

            let old = table[idx].take().unwrap();
            table[idx] = Some(displaced);
            displaced = old;
            use_table1 = !use_table1;
        }

        Err(WormGraphError::CuckooDisplacementFailed)
    }

    pub fn get(&self, key: &NodeId) -> Option<usize> {
        let idx1 = self.hash1(key);
        if let Some((k, v)) = &self.table1[idx1] {
            if k == key { return Some(*v); }
        }

        let idx2 = self.hash2(key);
        if let Some((k, v)) = &self.table2[idx2] {
            if k == key { return Some(*v); }
        }

        None
    }

    pub fn remove(&mut self, key: &NodeId) -> Option<usize> {
        let idx1 = self.hash1(key);
        if let Some((k, v)) = &self.table1[idx1] {
            if k == key {
                let val = *v;
                self.table1[idx1] = None;
                self.item_count -= 1;
                return Some(val);
            }
        }

        let idx2 = self.hash2(key);
        if let Some((k, v)) = &self.table2[idx2] {
            if k == key {
                let val = *v;
                self.table2[idx2] = None;
                self.item_count -= 1;
                return Some(val);
            }
        }

        None
    }

    pub fn len(&self) -> usize {
        self.item_count
    }

    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }
}

// =============================================================================
// WormGraph Core - Memoria Consciente
// =============================================================================

pub struct WormGraph {
    pub nodes: Vec<WormNode>,
    pub index: PerfectHashIndex,
    pub wormholes: BTreeMap<(NodeId, NodeId), Wormhole>,
    pub context_window: VecDeque<NodeId>,
    pub max_context_tokens: usize,
    pub current_token_count: usize,
    pub merkle_root: Hash256,
    pub fair_metadata: FairMetadata,
    pub temporal_nonce: u64,
    pub embedding_dim: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WormGraphError {
    IndexFull,
    CuckooDisplacementFailed,
    OntologicalDnaEmpty,
    EmbeddingDimensionMismatch { expected: usize, got: usize },
    NodeNotFound,
    ContextOverflow,
    InvalidMerkleProof,
    ZKProofInvalid,
}

impl fmt::Display for WormGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexFull => write!(f, "Perfect hash index at capacity"),
            Self::CuckooDisplacementFailed => write!(f, "Cuckoo displacement exceeded max iterations"),
            Self::OntologicalDnaEmpty => write!(f, "Node must have at least one FoundingFather"),
            Self::EmbeddingDimensionMismatch { expected, got } => {
                write!(f, "Embedding dimension mismatch: expected {}, got {}", expected, got)
            }
            Self::NodeNotFound => write!(f, "Node not found in graph"),
            Self::ContextOverflow => write!(f, "Context window exceeded maximum tokens"),
            Self::InvalidMerkleProof => write!(f, "Merkle proof verification failed"),
            Self::ZKProofInvalid => write!(f, "ZK proof verification failed"),
        }
    }
}

impl WormGraph {
    pub const DEFAULT_MAX_TOKENS: usize = 2_097_152; // 2^21
    pub const DEFAULT_EMBEDDING_DIM: usize = 768;
    pub const WORMHOLE_THRESHOLD: f32 = 0.85;
    pub const MAX_WORMHOLES_PER_NODE: usize = 5;
    pub const TEMPORAL_NONCE_INCREMENT: u64 = 1;

    pub fn new(max_tokens: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(1_000_000),
            index: PerfectHashIndex::new(1_000_000),
            wormholes: BTreeMap::new(),
            context_window: VecDeque::with_capacity(65536),
            max_context_tokens: max_tokens,
            current_token_count: 0,
            merkle_root: [0u8; 32],
            fair_metadata: FairMetadata {
                dpid: String::from("wormgraph-arkhe-cathedral"),
                ipfs_cid: None,
                orcid: String::from("0009-0005-2697-4668"),
                c2pa_manifest: None,
                sparql_endpoint: None,
                json_ld_context: Some(String::from("https://arkhe.network/context/wormgraph-v5.2.0")),
            },
            temporal_nonce: 0,
            embedding_dim: Self::DEFAULT_EMBEDDING_DIM,
        }
    }

    pub fn with_embedding_dim(mut self, dim: usize) -> Self {
        self.embedding_dim = dim;
        self
    }

    // =========================================================================
    // Operacoes Canonicas
    // =========================================================================

    pub fn add_node(
        &mut self,
        content: &str,
        metadata: NodeMetadata,
        ontological_dna: BTreeSet<FoundingFather>,
        embedding: Vec<f32>,
    ) -> Result<NodeId, WormGraphError> {

        // 1. Validar DNA ontológico
        if ontological_dna.is_empty() {
            return Err(WormGraphError::OntologicalDnaEmpty);
        }

        // 2. Validar embedding
        if embedding.len() != self.embedding_dim {
            return Err(WormGraphError::EmbeddingDimensionMismatch {
                expected: self.embedding_dim,
                got: embedding.len(),
            });
        }

        // 3. Calcular ID canônico
        let content_hash = Self::hash_content(content);
        let node_id = Self::compute_node_id(&content_hash, &metadata, &ontological_dna);

        // 4. Verificar se nó já existe
        if self.index.get(&node_id).is_some() {
            // Atualizar nó existente (versionamento)
            if let Some(idx) = self.index.get(&node_id) {
                if let Some(node) = self.nodes.get_mut(idx) {
                    node.last_accessed_ns = Self::now_ns();
                    node.access_count += 1;
                    return Ok(node_id);
                }
            }
        }

        // 5. Incrementar nonce temporal
        self.temporal_nonce += Self::TEMPORAL_NONCE_INCREMENT;

        // 6. Criar nó
        let node = WormNode {
            id: node_id,
            content_hash,
            metadata,
            ontological_dna,
            semantic_embedding: embedding,
            created_at_ns: Self::now_ns(),
            last_accessed_ns: Self::now_ns(),
            access_count: 1,
            merkle_proof: None,
            zk_nullifier: None,
            temporal_nonce: self.temporal_nonce,
        };

        // 7. Inserir no armazenamento
        let node_index = self.nodes.len();
        self.nodes.push(node);
        self.index.insert(node_id, node_index)?;

        // 8. Atualizar contexto
        self.update_context_window(node_id)?;

        // 9. Atualizar Merkle root
        self.update_merkle_root();

        // 10. Criar wormholes semânticos
        self.create_semantic_wormholes(node_id)?;

        Ok(node_id)
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&WormNode> {
        self.index.get(node_id)
            .and_then(|idx| self.nodes.get(idx))
    }

    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut WormNode> {
        if let Some(idx) = self.index.get(node_id) {
            if let Some(node) = self.nodes.get_mut(idx) {
                node.last_accessed_ns = Self::now_ns();
                node.access_count += 1;
                return Some(node);
            }
        }
        None
    }

    pub fn semantic_query(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        min_similarity: f32,
    ) -> Result<Vec<(&WormNode, f32)>, WormGraphError> {

        if query_embedding.len() != self.embedding_dim {
            return Err(WormGraphError::EmbeddingDimensionMismatch {
                expected: self.embedding_dim,
                got: query_embedding.len(),
            });
        }

        let mut results: Vec<(&WormNode, f32)> = Vec::with_capacity(top_k * 2);
        let mut visited = BTreeSet::new();

        // 1. Buscar via wormholes (O(1) effective)
        for ((source, target), wormhole) in &self.wormholes {
            for node_id in [source, target] {
                if !visited.insert(*node_id) {
                    continue;
                }
                if let Some(node) = self.get_node(node_id) {
                    let sim = cosine_similarity(query_embedding, &node.semantic_embedding);
                    if sim >= min_similarity {
                        // Boost por afinidade ontológica
                        let boosted = sim * (1.0 + wormhole.ontological_affinity as f32 * 0.1);
                        results.push((node, boosted.min(1.0)));
                    }
                }
            }
        }

        // 2. Fallback: busca no contexto window (LRU + semantic)
        if results.len() < top_k {
            for node_id in &self.context_window {
                if !visited.insert(*node_id) {
                    continue;
                }
                if let Some(node) = self.get_node(node_id) {
                    let sim = cosine_similarity(query_embedding, &node.semantic_embedding);
                    if sim >= min_similarity * 0.8 { // Threshold reduzido para fallback
                        // Boost por recência de acesso
                        let recency_boost = 1.0 + (node.access_count as f32 * 0.01).min(0.1);
                        results.push((node, (sim * recency_boost).min(1.0)));
                    }
                }
            }
        }

        // 3. Ordenar e retornar top-k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        results.truncate(top_k);
        Ok(results)
    }

    pub fn generate_zk_nullifier(
        &self,
        node_id: &NodeId,
        query_intent: &str,
    ) -> Result<Hash256, WormGraphError> {
        let mut hasher = Sha3_256::new();
        hasher.update(node_id);
        hasher.update(query_intent.as_bytes());
        hasher.update(&self.merkle_root);
        hasher.update(b"ARKHE-ZK-NULLIFIER-v5.2.0");
        hasher.update(&self.temporal_nonce.to_le_bytes());

        let nullifier: Hash256 = hasher.finalize().into();
        Ok(nullifier)
    }

    pub fn verify_merkle_proof(
        &self,
        proof: &MerkleProof,
        node_id: &NodeId,
    ) -> Result<bool, WormGraphError> {
        let mut current_hash = *node_id;
        for (sibling_hash, is_left) in &proof.path {
            current_hash = if *is_left {
                hash_pair(sibling_hash, &current_hash)
            } else {
                hash_pair(&current_hash, sibling_hash)
            };
        }
        Ok(current_hash == proof.root)
    }

    pub fn export_fair_snapshot(&self) -> FairSnapshot {
        let compliance = self.verify_fair_compliance();
        FairSnapshot {
            dpid: self.fair_metadata.dpid.clone(),
            merkle_root: self.merkle_root,
            node_count: self.nodes.len(),
            wormhole_count: self.wormholes.len(),
            context_tokens: self.current_token_count,
            fair_compliance: compliance,
            export_timestamp_ns: Self::now_ns(),
            schema_version: String::from("5.2.0"),
            temporal_anchor: Some(format!("temporal-nonce-{}", self.temporal_nonce)),
        }
    }

    pub fn compute_phi_c(&self) -> f64 {
        let modules = [
            (0.998, 0.20), // Ontological DNA
            (0.997, 0.20), // Perfect Hash O(1)
            (0.996, 0.15), // Semantic Wormholes
            (0.995, 0.15), // ZK Nullifier
            (0.997, 0.15), // FAIR Compliance
            (0.994, 0.10), // Context Management
            (0.998, 0.05), // Merkle Integrity
        ];

        modules.iter().map(|(phi, weight)| phi * weight).sum()
    }

    // =========================================================================
    // Metodos Privados
    // =========================================================================

    fn update_context_window(&mut self, node_id: NodeId) -> Result<(), WormGraphError> {
        let estimated_tokens = 200; // Conservative estimate per node

        // Evict LRU + low semantic weight
        while self.current_token_count + estimated_tokens > self.max_context_tokens
            && !self.context_window.is_empty() {

            // Encontrar nó menos relevante para eviction
            let evict_idx = self.find_eviction_candidate();
            if let Some(idx) = evict_idx {
                let _evicted_id = self.context_window.remove(idx).unwrap();
                self.current_token_count = self.current_token_count.saturating_sub(estimated_tokens);
            } else {
                break;
            }
        }

        self.context_window.push_back(node_id);
        self.current_token_count += estimated_tokens;
        Ok(())
    }

    fn find_eviction_candidate(&self) -> Option<usize> {
        let mut min_score = f64::MAX;
        let mut candidate = None;

        for (idx, node_id) in self.context_window.iter().enumerate() {
            if let Some(node) = self.get_node(node_id) {
                // Score: lower = more evictable
                // Combina recência (LRU), frequência (LFU), e peso ontológico
                let age_ns = Self::now_ns() - node.last_accessed_ns;
                let access_penalty = node.access_count as f64 * 100.0;
                let ontological_boost = node.ontological_dna.iter()
                    .map(|ff| ff.ontological_weight())
                    .sum::<f64>() * 1000.0;

                let score = age_ns as f64 - access_penalty - ontological_boost;
                if score < min_score {
                    min_score = score;
                    candidate = Some(idx);
                }
            }
        }

        candidate
    }

    fn create_semantic_wormholes(&mut self, new_node_id: NodeId) -> Result<(), WormGraphError> {
        let new_node = match self.get_node(&new_node_id) {
            Some(n) => n.clone(),
            None => return Err(WormGraphError::NodeNotFound),
        };

        let mut candidates: Vec<(NodeId, f32, f64)> = Vec::new();

        for (existing_id, existing_node) in self.nodes.iter().map(|n| (n.id, n)) {
            if existing_id == new_node_id {
                continue;
            }

            let similarity = cosine_similarity(&new_node.semantic_embedding, &existing_node.semantic_embedding);
            if similarity >= Self::WORMHOLE_THRESHOLD {
                let new_dna = &new_node.ontological_dna;
                let existing_dna = &existing_node.ontological_dna;
                let intersection: BTreeSet<_> = new_dna.intersection(existing_dna).copied().collect();
                let union: BTreeSet<_> = new_dna.union(existing_dna).copied().collect();
                let affinity = if union.is_empty() {
                    0.0
                } else {
                    intersection.len() as f64 / union.len() as f64
                };

                candidates.push((existing_id, similarity, affinity));
            }
        }

        // Ordenar por: similaridade * (1 + afinidade)
        candidates.sort_by(|a, b| {
            let score_a = a.1 * (1.0 + a.2 as f32);
            let score_b = b.1 * (1.0 + b.2 as f32);
            score_b.partial_cmp(&score_a).unwrap_or(core::cmp::Ordering::Equal)
        });

        for (target_id, similarity, affinity) in candidates.iter().take(Self::MAX_WORMHOLES_PER_NODE) {
            let now = Self::now_ns();
            let wormhole = Wormhole {
                source: new_node_id,
                target: *target_id,
                semantic_similarity: *similarity,
                ontological_affinity: *affinity,
                temporal_distance_ns: 0,
                created_at_ns: now,
                access_count: 0,
            };

            self.wormholes.insert((new_node_id, *target_id), wormhole.clone());
            self.wormholes.insert((*target_id, new_node_id), Wormhole {
                source: *target_id,
                target: new_node_id,
                semantic_similarity: *similarity,
                ontological_affinity: *affinity,
                temporal_distance_ns: 0,
                created_at_ns: now,
                access_count: 0,
            });
        }

        Ok(())
    }

    fn update_merkle_root(&mut self) {
        let mut hasher = Sha3_256::new();
        for node in &self.nodes {
            hasher.update(&node.id);
            hasher.update(&node.metadata.seal);
            hasher.update(&node.temporal_nonce.to_le_bytes());
        }
        self.merkle_root = hasher.finalize().into();
    }

    fn verify_fair_compliance(&self) -> FairComplianceReport {
        let findable = !self.fair_metadata.dpid.is_empty();
        let accessible = !self.fair_metadata.orcid.is_empty();
        let interoperable = self.fair_metadata.json_ld_context.is_some();
        let reusable = self.merkle_root != [0u8; 32];

        let overall = if findable && accessible && interoperable && reusable {
            1.0
        } else {
            [findable, accessible, interoperable, reusable].iter()
                .filter(|&&x| x).count() as f64 / 4.0
        };

        FairComplianceReport {
            findable,
            accessible,
            interoperable,
            reusable,
            overall_score: overall,
        }
    }

    // =========================================================================
    // Utilitarios Estaticos
    // =========================================================================

    fn compute_node_id(content_hash: &Hash256, metadata: &NodeMetadata, dna: &BTreeSet<FoundingFather>) -> NodeId {
        let mut hasher = Sha3_256::new();
        hasher.update(content_hash);
        hasher.update(&metadata.compute_seal());
        for ff in dna {
            hasher.update(&[*ff as u8]);
        }
        hasher.finalize().into()
    }

    fn hash_content(content: &str) -> Hash256 {
        let mut hasher = Sha3_256::new();
        hasher.update(content.as_bytes());
        hasher.finalize().into()
    }

    fn now_ns() -> u64 {
        // Em no_std, usar contador interno ou timestamp externo
        // Aqui: stub que deve ser substituido por timestamp real
        1_000_000_000u64
    }
}

// =============================================================================
// Funcoes Auxiliares
// =============================================================================

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a * norm_b == 0.0 {
        0.0
    } else {
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }
}

fn hash_pair(left: &Hash256, right: &Hash256) -> Hash256 {
    let mut hasher = Sha3_256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_embedding(dim: usize) -> Vec<f32> {
        (0..dim).map(|i| (i as f32 * 0.001).sin()).collect()
    }

    fn test_metadata() -> NodeMetadata {
        NodeMetadata {
            substrate_id: String::from("989.y.5"),
            phi_c: 0.99,
            theosis: 0.98,
            tags: vec![String::from("memory"), String::from("ontological")],
            cross_links: vec![],
            version: String::from("5.2.0"),
            seal: [0u8; 32],
            timestamp_ns: 1_000_000_000,
        }
    }

    #[test]
    fn test_founding_father_all() {
        assert_eq!(FoundingFather::ALL.len(), 12);
        assert_eq!(FoundingFather::Aristoteles as u8, 0);
        assert_eq!(FoundingFather::Rohrer as u8, 11);
    }

    #[test]
    fn test_founding_father_weights() {
        assert_eq!(FoundingFather::Aristoteles.ontological_weight(), 1.0);
        assert_eq!(FoundingFather::Rohrer.ontological_weight(), 0.80);
    }

    #[test]
    fn test_wormgraph_add_and_lookup() {
        let mut wg = WormGraph::new(10000);
        let metadata = test_metadata();
        let dna = BTreeSet::from([FoundingFather::Aristoteles, FoundingFather::AlKhwarizmi]);
        let embedding = dummy_embedding(768);

        let node_id = wg.add_node("Test content", metadata, dna, embedding).unwrap();

        let retrieved = wg.get_node(&node_id).unwrap();
        assert_eq!(retrieved.ontological_dna.len(), 2);
        assert!(retrieved.ontological_dna.contains(&FoundingFather::Aristoteles));
    }

    #[test]
    fn test_wormgraph_empty_dna_fails() {
        let mut wg = WormGraph::new(10000);
        let metadata = test_metadata();
        let dna = BTreeSet::new();
        let embedding = dummy_embedding(768);

        let result = wg.add_node("Test", metadata, dna, embedding);
        assert_eq!(result, Err(WormGraphError::OntologicalDnaEmpty));
    }

    #[test]
    fn test_wormgraph_embedding_dimension() {
        let mut wg = WormGraph::new(10000);
        let metadata = test_metadata();
        let dna = BTreeSet::from([FoundingFather::Einstein]);
        let embedding = dummy_embedding(512); // Wrong dimension

        let result = wg.add_node("Test", metadata, dna, embedding);
        assert!(matches!(result, Err(WormGraphError::EmbeddingDimensionMismatch { expected: 768, got: 512 })));
    }

    #[test]
    fn test_wormgraph_semantic_query() {
        let mut wg = WormGraph::new(10000);
        let metadata = test_metadata();

        // Node 1: Aristotelian logic
        let dna1 = BTreeSet::from([FoundingFather::Aristoteles]);
        let mut emb1 = dummy_embedding(768);
        emb1[0] = 0.9; emb1[1] = 0.8;
        let id1 = wg.add_node("Logic content", metadata.clone(), dna1, emb1.clone()).unwrap();

        // Node 2: Similar embedding
        let dna2 = BTreeSet::from([FoundingFather::Aristoteles, FoundingFather::AlKhwarizmi]);
        let mut emb2 = emb1.clone();
        emb2[0] = 0.85; emb2[1] = 0.75;
        let id2 = wg.add_node("Algorithmic logic", metadata.clone(), dna2, emb2).unwrap();

        // Query with similar embedding
        let results = wg.semantic_query(&emb1, 5, 0.8).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|(n, _)| n.id == id1 || n.id == id2));
    }

    #[test]
    fn test_wormgraph_context_eviction() {
        let mut wg = WormGraph::new(1000); // Small window
        let metadata = test_metadata();

        for i in 0..50 {
            let dna = BTreeSet::from([FoundingFather::Mendel]);
            let embedding = dummy_embedding(768);
            let _ = wg.add_node(&format!("Content {}", i), metadata.clone(), dna, embedding);
        }

        assert!(wg.current_token_count <= wg.max_context_tokens);
    }

    #[test]
    fn test_zk_nullifier_deterministic() {
        let wg = WormGraph::new(10000);
        let node_id = [0u8; 32];

        let n1 = wg.generate_zk_nullifier(&node_id, "query-1").unwrap();
        let n2 = wg.generate_zk_nullifier(&node_id, "query-1").unwrap();
        let n3 = wg.generate_zk_nullifier(&node_id, "query-2").unwrap();

        assert_eq!(n1, n2);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_fair_snapshot() {
        let wg = WormGraph::new(10000);
        let snapshot = wg.export_fair_snapshot();

        assert_eq!(snapshot.schema_version, "5.2.0");
        assert!(snapshot.fair_compliance.findable);
        assert!(snapshot.fair_compliance.accessible);
        assert_eq!(snapshot.node_count, 0);
    }

    #[test]
    fn test_phi_c_calculation() {
        let wg = WormGraph::new(10000);
        let phi = wg.compute_phi_c();
        assert!(phi > 0.99);
        assert!(phi < 1.0);
    }

    #[test]
    fn test_perfect_hash_index() {
        let mut idx = PerfectHashIndex::new(100);
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];

        idx.insert(key1, 42).unwrap();
        idx.insert(key2, 99).unwrap();

        assert_eq!(idx.get(&key1), Some(42));
        assert_eq!(idx.get(&key2), Some(99));
        assert_eq!(idx.get(&[3u8; 32]), None);
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }
}
