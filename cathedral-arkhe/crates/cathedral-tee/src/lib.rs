//! TEEBridge — Atestação unificada SGX + IoNet
//! Selo: CATHEDRAL-ARKHE-TEE-v1.0.0-2026-06-19

pub mod bridge;
pub mod types;
pub mod verifier;
pub mod secure_vm;

pub use bridge::TEEBridge;
pub use types::{TeeType, AttestationReport, AttestationResult};
pub use secure_vm::SecureVmExecutor;
