//! Benchmarks do Safe-Core AGI
//!
//! Para executar: cargo bench --workspace

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_hashes(c: &mut Criterion) {
    let _data = b"The quick brown fox jumps over the lazy dog. " * 1000;

    let mut group = c.benchmark_group("Hashing");

    group.bench_function("blake3", |b| {
        b.iter(|| {
            // Mock Blake3Hasher::hash
        })
    });

    group.bench_function("sha256", |b| {
        b.iter(|| {
            // Mock Sha256Hasher::hash
        })
    });

    group.finish();
}

fn benchmark_merkle(c: &mut Criterion) {
    let mut group = c.benchmark_group("Merkle Tree");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &_size| {
            b.iter(|| {
                // Mock
            })
        });
    }

    group.finish();
}

fn benchmark_docker_exec(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sandbox Execution");

    group.bench_function("docker_echo", |b| {
        b.iter(|| {
            // Mock
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
