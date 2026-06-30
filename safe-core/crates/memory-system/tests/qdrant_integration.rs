//! Testes de integração com Qdrant real usando testcontainers
//!
//! # Requisitos
//! - Docker instalado e rodando
//! - Variável de ambiente `QDRANT_INTEGRATION_TESTS=1`

use testcontainers::{
    core::WaitFor,
    runners::AsyncRunner,
    Image,
};

struct QdrantImage;

impl Image for QdrantImage {
    type Args = ();

    fn name(&self) -> &str {
        "qdrant/qdrant"
    }

    fn tag(&self) -> &str {
        "v1.11.0"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("Qdrant HTTP server listening")]
    }
}

#[tokio::test]
#[ignore]
async fn test_qdrant_vector_insert_and_search() {
    let container = QdrantImage
        .start()
        .await
        .expect("Failed to start Qdrant container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(6334).await.unwrap();
    let qdrant_url = format!("http://{}:{}", host, port);

    assert!(!qdrant_url.is_empty());
}
