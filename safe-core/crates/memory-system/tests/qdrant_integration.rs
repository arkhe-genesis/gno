//! Testes de integração com Qdrant real usando testcontainers
//!
//! # Requisitos
//! - Docker instalado e rodando
//! - Variável de ambiente `QDRANT_INTEGRATION_TESTS=1`


use safe_core_memory_system::MemorySystem;
use testcontainers::{
    core::WaitFor,
    clients,
    Image,
};

struct QdrantImage;

impl Image for QdrantImage {
    type Args = ();

    fn name(&self) -> String {
        "qdrant/qdrant".to_string()
    }

    fn tag(&self) -> String {
        "v1.11.0".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("Qdrant HTTP server listening")]
    }
}

#[tokio::test]
#[ignore] // Executar apenas com `cargo test -- --ignored`
async fn test_qdrant_vector_insert_and_search() {
    let docker = clients::Cli::default();
    // 1. Iniciar container Qdrant
    let container = docker.run(QdrantImage);

    let host = "localhost";
    let port = container.get_host_port_ipv4(6334);
    let qdrant_url = format!("http://{}:{}", host, port);

    // 2. Inicializar MemorySystem
    let hsm = safe_core_hw_backends::SoftwareHsm::new_p256();
    let mut memory = MemorySystem::new(
        std::sync::Arc::new(hsm),
        qdrant_url,
        "test_collection".to_string(),
    )
    .await
    .expect("Failed to initialize MemorySystem");

    // 3. Inserir vetores
    let vectors = vec![
        ("vec1", vec![0.1, 0.2, 0.3], "Test vector 1"),
        ("vec2", vec![0.4, 0.5, 0.6], "Test vector 2"),
        ("vec3", vec![0.7, 0.8, 0.9], "Test vector 3"),
    ];

    for (id, vector, text) in vectors {
        memory
            .insert_vector(id, vector, text)
            .await
            .expect("Failed to insert vector");
    }

    // 4. Buscar por similaridade
    let query = vec![0.1, 0.2, 0.3];
    let results = memory.search(&query, 2).await.expect("Search failed");

    assert_eq!(results.len(), 2, "Should return 2 results");
    assert!(
        results[0].score > 0.99,
        "First result should be nearly identical"
    );

    // 5. Verificar selagem Merkle
    let snapshot = memory.seal().await.expect("Seal failed");
    assert_eq!(snapshot.total_entries, 3, "Should have 3 entries");
    assert!(
        memory.verify_integrity(&snapshot.id).await.unwrap(),
        "Integrity check should pass"
    );
}

#[tokio::test]
#[ignore]
async fn test_qdrant_collection_management() {
    let docker = clients::Cli::default();
    let container = docker.run(QdrantImage);
    let host = "localhost";
    let port = container.get_host_port_ipv4(6334);
    let qdrant_url = format!("http://{}:{}", host, port);

    let hsm = safe_core_hw_backends::SoftwareHsm::new_p256();
    let memory = MemorySystem::new(
        std::sync::Arc::new(hsm),
        qdrant_url,
        "test_collection_mgmt".to_string(),
    )
    .await
    .unwrap();

    // Testar criação de coleção
    let collections = memory.list_collections().await.unwrap();
    assert!(
        collections.contains(&"test_collection_mgmt".to_string()),
        "Collection should be created"
    );

    // Testar deleção
    memory.delete_collection().await.unwrap();
    let collections_after = memory.list_collections().await.unwrap();
    assert!(
        !collections_after.contains(&"test_collection_mgmt".to_string()),
        "Collection should be deleted"
    );
}
