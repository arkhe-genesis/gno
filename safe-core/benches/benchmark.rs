use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use safe_core_hash_blake3::blake3::Hasher;

fn benchmark_hashes(c: &mut Criterion) {
    let data = b"The quick brown fox jumps over the lazy dog. " * 1000;

    let mut group = c.benchmark_group("Hashing");

    group.bench_function("blake3", |b| {
        b.iter(|| {
            let mut hasher = Hasher::new();
            hasher.update(&data);
            hasher.finalize()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_hashes,
);
criterion_main!(benches);
