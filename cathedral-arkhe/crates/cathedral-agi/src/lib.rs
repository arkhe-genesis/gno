//! AGI Core v3.0.0 — com inferência real via Ollama e memória episódica
//! Selo: CATHEDRAL-ARKHE-AGI-CORE-v3.0.0-2026-06-19

pub mod world_model;
pub mod mcts;
pub mod meta_cognitive;
pub mod wormhole;
pub mod ethics;
pub mod agi_core;
pub mod llm_client;

pub use agi_core::AGICore;
pub use world_model::{WorldModel, WorldState, Intent};
pub use mcts::{MCTSEngine, MCTSNode, MCTSResult};
pub use meta_cognitive::{MetaCognitiveLoop, MetaState};
pub use wormhole::HierarchicalWormhole;
pub use ethics::{EthicsVerifier, EthicsResult};
pub use llm_client::OllamaClient;
