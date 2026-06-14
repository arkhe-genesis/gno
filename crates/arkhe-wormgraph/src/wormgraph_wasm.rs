#[allow(unused)]

#[allow(unused)]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use js_sys::{Array, Uint8Array, Float32Array, Object, Reflect};
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen::{to_value, from_value};

use crate::wormgraph_core::{WormGraph, WormNode, NodeMetadata, FoundingFather, WormGraphError};

// =============================================================================
// WASM-Exposed Types
// =============================================================================

#[wasm_bindgen]
pub struct WasmWormGraph {
    graph: WormGraph,
}

#[wasm_bindgen]
pub struct WasmQueryResult {
    node_id: Vec<u8>,
    similarity: f32,
    substrate_id: String,
    access_count: u64,
}

#[wasm_bindgen]
pub struct WasmFairSnapshot {
    json: String,
}

#[wasm_bindgen]
pub struct WasmNodeId {
    bytes: Vec<u8>,
}

// =============================================================================
// WASM API
// =============================================================================

#[wasm_bindgen]
impl WasmWormGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(max_tokens: usize, embedding_dim: Option<usize>) -> Self {
        let mut graph = WormGraph::new(max_tokens);
        if let Some(dim) = embedding_dim {
            graph = graph.with_embedding_dim(dim);
        }
        Self { graph }
    }

    /// Adiciona um no ao WormGraph
    /// @param content: conteudo textual
    /// @param substrate_id: ID do substrato
    /// @param tags: array de strings
    /// @param dna: array de strings (nomes dos FoundingFathers)
    /// @param embedding: Float32Array de embedding_dim elementos
    /// @return Uint8Array de 32 bytes (NodeId)
    pub fn add_node(
        &mut self,
        content: String,
        substrate_id: String,
        tags: Array,
        dna: Array,
        embedding: Float32Array,
    ) -> Result<Uint8Array, JsValue> {
        let tags_vec: Vec<String> = tags.iter()
            .filter_map(|v| v.as_string())
            .collect();

        let dna_set: std::collections::BTreeSet<FoundingFather> = dna.iter()
            .filter_map(|v| v.as_string())
            .filter_map(|s| match s.as_str() {
                "aristoteles" => Some(FoundingFather::Aristoteles),
                "al_khwarizmi" => Some(FoundingFather::AlKhwarizmi),
                "hipparchus" => Some(FoundingFather::Hipparchus),
                "hippocrates" => Some(FoundingFather::Hippocrates),
                "pasteur" => Some(FoundingFather::Pasteur),
                "mendel" => Some(FoundingFather::Mendel),
                "adam_smith" => Some(FoundingFather::AdamSmith),
                "ada_lovelace" => Some(FoundingFather::AdaLovelace),
                "vint_cerf" => Some(FoundingFather::VintCerf),
                "einstein" => Some(FoundingFather::Einstein),
                "feynman" => Some(FoundingFather::Feynman),
                "rohrer" => Some(FoundingFather::Rohrer),
                _ => None,
            })
            .collect();

        let embedding_vec: Vec<f32> = embedding.to_vec();

        let metadata = NodeMetadata {
            substrate_id,
            phi_c: 0.99,
            theosis: 0.98,
            tags: tags_vec,
            cross_links: vec![],
            version: String::from("5.2.0"),
            seal: [0u8; 32],
            timestamp_ns: 1_000_000_000,
        };

        match self.graph.add_node(&content, metadata, dna_set, embedding_vec) {
            Ok(node_id) => {
                let array = Uint8Array::new_with_length(32);
                array.copy_from(&node_id);
                Ok(array)
            }
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    }

    /// Query semantica
    /// @param query_embedding: Float32Array
    /// @param top_k: numero maximo de resultados
    /// @param min_similarity: threshold de similaridade (0.0-1.0)
    /// @return Array de objetos {nodeId, similarity, substrateId, accessCount}
    pub fn semantic_query(
        &self,
        query_embedding: Float32Array,
        top_k: usize,
        min_similarity: f32,
    ) -> Result<Array, JsValue> {
        let query_vec: Vec<f32> = query_embedding.to_vec();

        match self.graph.semantic_query(&query_vec, top_k, min_similarity) {
            Ok(results) => {
                let array = Array::new();
                for (node, sim) in results {
                    let obj = Object::new();

                    let node_id_arr = Uint8Array::new_with_length(32);
                    node_id_arr.copy_from(&node.id);
                    Reflect::set(&obj, &"nodeId".into(), &node_id_arr)?;

                    Reflect::set(&obj, &"similarity".into(), &JsValue::from_f64(sim as f64))?;
                    Reflect::set(&obj, &"substrateId".into(), &JsValue::from_str(&node.metadata.substrate_id))?;
                    Reflect::set(&obj, &"accessCount".into(), &JsValue::from_f64(node.access_count as f64))?;

                    array.push(&obj);
                }
                Ok(array)
            }
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    }

    /// Gera ZK nullifier
    /// @param node_id: Uint8Array de 32 bytes
    /// @param query_intent: string
    /// @return Uint8Array de 32 bytes (nullifier)
    pub fn generate_nullifier(
        &self,
        node_id: Uint8Array,
        query_intent: String,
    ) -> Result<Uint8Array, JsValue> {
        if node_id.length() != 32 {
            return Err(JsValue::from_str("node_id must be 32 bytes"));
        }

        let mut node_id_arr = [0u8; 32];
        node_id.copy_to(&mut node_id_arr);

        match self.graph.generate_zk_nullifier(&node_id_arr, &query_intent) {
            Ok(nullifier) => {
                let array = Uint8Array::new_with_length(32);
                array.copy_from(&nullifier);
                Ok(array)
            }
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    }

    /// Exporta snapshot FAIR
    /// @return objeto JSON
    pub fn export_fair(&self) -> Result<JsValue, JsValue> {
        let snapshot = self.graph.export_fair_snapshot();
        to_value(&snapshot).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Calcula Phi_C
    /// @return f64
    pub fn compute_phi_c(&self) -> f64 {
        self.graph.compute_phi_c()
    }

    /// Retorna estatisticas do grafo
    /// @return objeto {nodeCount, wormholeCount, contextTokens, maxTokens, phiC}
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let obj = Object::new();
        Reflect::set(&obj, &"nodeCount".into(), &JsValue::from_f64(self.graph.nodes.len() as f64))?;
        Reflect::set(&obj, &"wormholeCount".into(), &JsValue::from_f64(self.graph.wormholes.len() as f64))?;
        Reflect::set(&obj, &"contextTokens".into(), &JsValue::from_f64(self.graph.current_token_count as f64))?;
        Reflect::set(&obj, &"maxTokens".into(), &JsValue::from_f64(self.graph.max_context_tokens as f64))?;
        Reflect::set(&obj, &"phiC".into(), &JsValue::from_f64(self.graph.compute_phi_c()))?;
        Ok(JsValue::from(obj))
    }
}

// =============================================================================
// WASM-Optimized Functions
// =============================================================================

/// Funcao otimizada para batch insert (usada em ingestao de dados)
#[wasm_bindgen]
pub fn wormgraph_batch_insert(
    graph: &mut WasmWormGraph,
    contents: Array,
    substrate_ids: Array,
    tags_list: Array,
    dna_list: Array,
    embeddings_list: Array,
) -> Result<Array, JsValue> {
    let node_ids = Array::new();

    for i in 0..contents.length() {
        let content = contents.get(i).as_string().ok_or("Invalid content")?;
        let substrate_id = substrate_ids.get(i).as_string().ok_or("Invalid substrate_id")?;
        let tags = tags_list.get(i).dyn_into::<Array>()?;
        let dna = dna_list.get(i).dyn_into::<Array>()?;
        let embedding = embeddings_list.get(i).dyn_into::<Float32Array>()?;

        let node_id = graph.add_node(content, substrate_id, tags, dna, embedding)?;
        node_ids.push(&node_id);
    }

    Ok(node_ids)
}

/// Pre-computa embeddings usando WebGL/WebGPU (stub para futura implementacao)
#[wasm_bindgen]
pub fn wormgraph_compute_embeddings_webgl(
    _texts: Array,
    _model_url: String,
) -> Result<Array, JsValue> {
    // Stub: em producao, usar ONNX Runtime Web ou TensorFlow.js
    // para computar embeddings no GPU do browser
    Err(JsValue::from_str("WebGL embedding computation not yet implemented"))
}
