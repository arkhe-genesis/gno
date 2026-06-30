//! Benchmarks do Safe-Core AGI
//!
//! Para executar: cargo bench --workspace

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use safe_core_crypto::{Blake3Hasher, Sha256Hasher};
use safe_core_merkle::MerkleTree;
use safe_core_sandbox_bridge::DockerBackend;
use tokio::runtime::Runtime;

fn benchmark_hashes(c: &mut Criterion) {
    let data = b"The quick brown fox jumps over the lazy dog. " * 1000;

    let mut group = c.benchmark_group("Hashing");

    group.bench_function("blake3", |b| {
        b.iter(|| {
            Blake3Hasher::hash(data)
        })
    });

    group.bench_function("sha256", |b| {
        b.iter(|| {
            Sha256Hasher::hash(data)
        })
    });

    group.finish();
}

fn benchmark_merkle(c: &mut Criterion) {
    let mut group = c.benchmark_group("Merkle Tree");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
            let mut tree = MerkleTree::new();
            b.iter(|| {
                for i in 0..size {
                    tree.insert([i as u8; 32]);
                }
                tree.commit();
            })
        });
    }

    group.finish();
}

fn benchmark_docker_exec(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let backend = rt.block_on(DockerBackend::new("alpine:latest")).unwrap();

    let mut group = c.benchmark_group("Sandbox Execution");

    group.bench_function("docker_echo", |b| {
        b.to_async(&rt).iter(|| async {
            let ws = backend.x_create_workspace(Default::default()).await.unwrap();
            let _ = backend.x_execute(&ws.id, "echo hello").await.unwrap();
            backend.x_destroy(&ws.id).await.unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_hashes,
    benchmark_merkle,
    benchmark_docker_exec
);
criterion_main!(benches);
