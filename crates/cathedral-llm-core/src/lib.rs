#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelTier {
    Pro,
    Plus,
    Standard,
    Lite,
}

pub mod model;
pub use model::{LlamaEngine, ModelConfig};
