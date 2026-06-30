//! Reactive Governance Module for Dark Bio + AGISAFE.
//!
//! This crate extends the transparency log with signed governance actions,
//! enabling autonomous corrective feedback loops for UED and Sparse-Dense systems.

pub mod governance;
pub mod reactive_log;
pub mod watchdog;
pub mod integration;
pub mod transparency_log;
pub mod hsm_backend;

pub use governance::{GovernanceAction, GovernanceEntry};
pub use reactive_log::ReactiveLog;
pub use watchdog::GovernanceWatchdog;
pub use integration::{UedGovernance, SparseRouterGovernance};
