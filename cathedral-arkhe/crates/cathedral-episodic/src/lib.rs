//! EpisodicSync — CRDT-lite com vector clock
//! Selo: CATHEDRAL-ARKHE-EPISODIC-v1.0.0-2026-06-19

pub mod sync;
pub mod sqlite_storage;
pub mod types;

pub use sync::EpisodicSync;
pub use sqlite_storage::SqliteStorage;
pub use types::{EpisodicEntry, VectorClock, Ordering};
