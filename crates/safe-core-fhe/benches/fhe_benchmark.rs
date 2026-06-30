use criterion::{criterion_group, criterion_main, Criterion};
use safe_core_fhe::token_verifier::{FheTokenVerifier, HASH_LEN};
use std::time::Duration;

fn bench_fhe_verification(c: &mut Criterion) {
    let verifier = FheTokenVerifier::new();
    let hash_1 = FheTokenVerifier::hash_credential("benchmark_hash_1");
    let hash_2 = FheTokenVerifier::hash_credential("benchmark_hash_2");

    let enc_1 = verifier.encrypt_hash(&hash_1).unwrap();
    let enc_2 = verifier.encrypt_hash(&hash_2).unwrap();

    let mut group = c.benchmark_group("FHE Verification");
    group.sample_size(10);  // 10 amostras (cada verificação é cara)
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("verify_hash_32_bytes", |b| {
        b.iter(|| {
            let _ = verifier.verify_homomorphic(&enc_1, &enc_2);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_fhe_verification);
criterion_main!(benches);
