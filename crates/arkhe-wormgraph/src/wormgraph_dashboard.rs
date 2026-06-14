#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use serde::{Serialize, Deserialize};

use crate::wormgraph_core::{WormGraph, WormNode, FoundingFather, NodeId};

// =============================================================================
// Estruturas de Dados do Dashboard
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_nodes: usize,
    pub total_wormholes: usize,
    pub context_tokens: usize,
    pub max_tokens: usize,
    pub context_utilization: f64,
    pub phi_c: f64,
    pub merkle_root: String,
    pub temporal_nonce: u64,
    pub founding_father_distribution: BTreeMap<String, usize>,
    pub substrate_distribution: BTreeMap<String, usize>,
    pub tag_cloud: BTreeMap<String, usize>,
    pub wormhole_density: f64,
    pub avg_ontological_affinity: f64,
    pub top_accessed_nodes: Vec<NodeSummary>,
    pub recent_nodes: Vec<NodeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub id_hex: String,
    pub substrate_id: String,
    pub tags: Vec<String>,
    pub dna: Vec<String>,
    pub access_count: u64,
    pub last_accessed: String,
    pub wormhole_count: usize,
    pub phi_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormholeView {
    pub source_hex: String,
    pub target_hex: String,
    pub semantic_similarity: f32,
    pub ontological_affinity: f64,
    pub created_at: String,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologicalDNAView {
    pub father: String,
    pub domain: String,
    pub weight: f64,
    pub node_count: usize,
    pub avg_phi_c: f64,
    pub connections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalView {
    pub nonce: u64,
    pub merkle_root: String,
    pub node_count: usize,
    pub wormhole_count: usize,
    pub timestamp: String,
}

// =============================================================================
// Dashboard Engine
// =============================================================================

pub struct WormGraphDashboard<'a> {
    graph: &'a WormGraph,
}

impl<'a> WormGraphDashboard<'a> {
    pub fn new(graph: &'a WormGraph) -> Self {
        Self { graph }
    }

    pub fn compute_stats(&self) -> DashboardStats {
        let mut ff_dist: BTreeMap<String, usize> = BTreeMap::new();
        let mut substrate_dist: BTreeMap<String, usize> = BTreeMap::new();
        let mut tag_cloud: BTreeMap<String, usize> = BTreeMap::new();
        let mut node_summaries: Vec<NodeSummary> = Vec::new();

        for node in &self.graph.nodes {
            // Founding Father distribution
            for ff in &node.ontological_dna {
                *ff_dist.entry(ff.domain().to_string()).or_insert(0) += 1;
            }

            // Substrate distribution
            *substrate_dist.entry(node.metadata.substrate_id.clone()).or_insert(0) += 1;

            // Tag cloud
            for tag in &node.metadata.tags {
                *tag_cloud.entry(tag.clone()).or_insert(0) += 1;
            }

            // Node summary
            let wormhole_count = self.graph.wormholes.iter()
                .filter(|((s, t), _)| *s == node.id || *t == node.id)
                .count();

            node_summaries.push(NodeSummary {
                id_hex: hex::encode(&node.id[..8]),
                substrate_id: node.metadata.substrate_id.clone(),
                tags: node.metadata.tags.clone(),
                dna: node.ontological_dna.iter().map(|ff| ff.domain().to_string()).collect(),
                access_count: node.access_count,
                last_accessed: format!("{}", node.last_accessed_ns),
                wormhole_count,
                phi_c: node.metadata.phi_c,
            });
        }

        // Sort for top accessed and recent
        let mut top_accessed = node_summaries.clone();
        top_accessed.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        top_accessed.truncate(10);

        let mut recent = node_summaries.clone();
        recent.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        recent.truncate(10);

        let total_nodes = self.graph.nodes.len();
        let total_wormholes = self.graph.wormholes.len();
        let wormhole_density = if total_nodes > 0 {
            (total_wormholes as f64) / (total_nodes as f64)
        } else {
            0.0
        };

        let avg_affinity = if total_wormholes > 0 {
            self.graph.wormholes.values()
                .map(|w| w.ontological_affinity)
                .sum::<f64>() / (total_wormholes as f64)
        } else {
            0.0
        };

        DashboardStats {
            total_nodes,
            total_wormholes,
            context_tokens: self.graph.current_token_count,
            max_tokens: self.graph.max_context_tokens,
            context_utilization: (self.graph.current_token_count as f64) / (self.graph.max_context_tokens as f64),
            phi_c: self.graph.compute_phi_c(),
            merkle_root: hex::encode(&self.graph.merkle_root),
            temporal_nonce: self.graph.temporal_nonce,
            founding_father_distribution: ff_dist,
            substrate_distribution: substrate_dist,
            tag_cloud,
            wormhole_density,
            avg_ontological_affinity: avg_affinity,
            top_accessed_nodes: top_accessed,
            recent_nodes: recent,
        }
    }

    pub fn get_wormholes_for_node(&self, node_id: &NodeId) -> Vec<WormholeView> {
        self.graph.wormholes.iter()
            .filter(|((s, t), _)| s == node_id || t == node_id)
            .map(|(_, w)| WormholeView {
                source_hex: hex::encode(&w.source[..8]),
                target_hex: hex::encode(&w.target[..8]),
                semantic_similarity: w.semantic_similarity,
                ontological_affinity: w.ontological_affinity,
                created_at: format!("{}", w.created_at_ns),
                access_count: w.access_count,
            })
            .collect()
    }

    pub fn get_ontological_dna_view(&self) -> Vec<OntologicalDNAView> {
        FoundingFather::ALL.iter().map(|ff| {
            let nodes_with_ff: Vec<&WormNode> = self.graph.nodes.iter()
                .filter(|n| n.ontological_dna.contains(ff))
                .collect();

            let avg_phi = if !nodes_with_ff.is_empty() {
                nodes_with_ff.iter().map(|n| n.metadata.phi_c).sum::<f64>() / (nodes_with_ff.len() as f64)
            } else {
                0.0
            };

            let connections: Vec<String> = nodes_with_ff.iter()
                .flat_map(|n| n.ontological_dna.iter())
                .filter(|&other| other != ff)
                .map(|other: &FoundingFather| other.domain().to_string())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            OntologicalDNAView {
                father: ff.domain().to_string(),
                domain: ff.domain().to_string(),
                weight: ff.ontological_weight(),
                node_count: nodes_with_ff.len(),
                avg_phi_c: avg_phi,
                connections,
            }
        }).collect()
    }

    pub fn get_temporal_timeline(&self) -> Vec<TemporalView> {
        // Em produção: consultar TemporalChain para histórico completo
        vec![TemporalView {
            nonce: self.graph.temporal_nonce,
            merkle_root: hex::encode(&self.graph.merkle_root),
            node_count: self.graph.nodes.len(),
            wormhole_count: self.graph.wormholes.len(),
            timestamp: format!("{}", self.graph.temporal_nonce),
        }]
    }

    pub fn export_graph_json(&self) -> String {
        let stats = self.compute_stats();
        serde_json::to_string_pretty(&stats).unwrap_or_default()
    }

    pub fn export_graph_gexf(&self) -> String {
        // Formato GEXF para visualização em Gephi/Cytoscape
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<gexf xmlns="http://www.gexf.net/1.2draft" version="1.2">
  <graph mode="static" defaultedgetype="undirected">
    <nodes>
"#);

        for (i, node) in self.graph.nodes.iter().enumerate() {
            let dna_str = node.ontological_dna.iter()
                .map(|ff| ff.domain())
                .collect::<Vec<_>>()
                .join(",");
            xml.push_str(&format!(
                r#"      <node id="{}" label="{}">
        <attvalues>
          <attvalue for="substrate" value="{}"/>
          <attvalue for="dna" value="{}"/>
          <attvalue for="phi_c" value="{}"/>
          <attvalue for="access_count" value="{}"/>
        </attvalues>
      </node>
"#,
                i,
                hex::encode(&node.id[..8]),
                node.metadata.substrate_id,
                dna_str,
                node.metadata.phi_c,
                node.access_count
            ));
        }

        xml.push_str(r#"    </nodes>
    <edges>
"#);

        for (i, ((s, t), wormhole)) in self.graph.wormholes.iter().enumerate() {
            let source_idx = self.graph.nodes.iter().position(|n| n.id == *s).unwrap_or(0);
            let target_idx = self.graph.nodes.iter().position(|n| n.id == *t).unwrap_or(0);
            xml.push_str(&format!(
                r#"      <edge id="{}" source="{}" target="{}" weight="{}"/>
"#,
                i,
                source_idx,
                target_idx,
                wormhole.semantic_similarity
            ));
        }

        xml.push_str(r#"    </edges>
  </graph>
</gexf>"#);

        xml
    }
}
