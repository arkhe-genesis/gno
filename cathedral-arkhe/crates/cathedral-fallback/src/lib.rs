//! FallbackChain + CostOptimizer
//! Selo: CATHEDRAL-ARKHE-FALLBACK-v1.0.0-2026-06-19

pub mod fallback;
pub mod cost;
pub mod lc3_executor;

pub use fallback::{FallbackChain, TaskExecutor, WorkerExecutor, WorkerTier};
pub use cost::{CostOptimizer, CostRecord, OptimizationStats};
pub use lc3_executor::Lc3VmExecutor;
