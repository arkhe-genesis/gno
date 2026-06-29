//! Safe-Core Parallax Bridge — Cliente gRPC para inferência distribuída.
//!
//! Este crate fornece um cliente thread-safe para comunicação com o scheduler
//! do Parallax, suportando:
//!
//! - Inferência síncrona (chat e completion)
//! - Verificação de saúde do cluster
//! - Listagem de modelos disponíveis
//! - Geração de embeddings
//!
//! # Exemplo
//!
//! ```no_run
//! use safe_core_parallax_bridge::ParallaxClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ParallaxClient::connect("http://localhost:50051").await?;
//!     let health = client.health().await?;
//!     println!("Cluster ready: {}", health.ready);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::ParallaxClient;
pub use error::ParallaxError;
pub use types::*;
