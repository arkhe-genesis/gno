#![allow(unsafe_code)]
#![allow(unused)]
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::collections::BTreeSet;
use rand::prelude::*;
use rand::distributions::Standard;
use rand::thread_rng;

use crate::wormgraph_core::{WormGraph, NodeMetadata, FoundingFather};

fn random_embedding(dim: usize, rng: &mut ThreadRng) -> Vec<f32> {
    rng.sample_iter(Standard).take(dim).map(|x: f32| x * 2.0 - 1.0).collect()
}

pub fn benchmark_add_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_node");

    for size in [100, 1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b: &mut criterion::Bencher, &size| {
            let mut wg = WormGraph::new(2_097_152);
            let mut rng = thread_rng();

            b.iter(|| {
                for i in 0..size {
                    let metadata = NodeMetadata {
                        substrate_id: format!("bench-{}", i),
                        phi_c: 0.99,
                        theosis: 0.98,
                        tags: vec![format!("tag-{}", i % 10)],
                        cross_links: vec![],
                        version: String::from("5.2.0"),
                        seal: [0u8; 32],
                        timestamp_ns: i as u64 * 1_000_000,
                    };
                    let dna = BTreeSet::from([FoundingFather::ALL[i % 12]]);
                    let embedding = random_embedding(768, &mut rng);
                    let _ = wg.add_node(&format!("content-{}", i), metadata, dna, embedding);
                }
                black_box(&wg);
            });
        });
    }
    group.finish();
}

pub fn benchmark_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_o1");

    for size in [1000, 10000, 100000, 1000000].iter() {
        let mut wg = WormGraph::new(2_097_152);
        let mut rng = thread_rng();
        let mut node_ids = Vec::with_capacity(*size);

        // Pre-populate
        for i in 0..*size {
            let metadata = NodeMetadata {
                substrate_id: format!("bench-{}", i),
                phi_c: 0.99,
                theosis: 0.98,
                tags: vec![format!("tag-{}", i % 10)],
                cross_links: vec![],
                version: String::from("5.2.0"),
                seal: [0u8; 32],
                timestamp_ns: i as u64 * 1_000_000,
            };
            let dna = BTreeSet::from([FoundingFather::ALL[i % 12]]);
            let embedding = random_embedding(768, &mut rng);
            if let Ok(id) = wg.add_node(&format!("content-{}", i), metadata, dna, embedding) {
                node_ids.push(id);
            }
        }

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b: &mut criterion::Bencher, &_size| {
            b.iter(|| {
                for id in &node_ids {
                    black_box(wg.get_node(id));
                }
            });
        });
    }
    group.finish();
}

pub fn benchmark_semantic_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_query");

    for size in [1000, 10000, 100000].iter() {
        let mut wg = WormGraph::new(2_097_152);
        let mut rng = thread_rng();

        // Pre-populate with similar embeddings for wormhole creation
        for i in 0..*size {
            let mut embedding = vec![0.0f32; 768];
            embedding[i % 768] = 0.9;
            embedding[(i + 1) % 768] = 0.7;

            let metadata = NodeMetadata {
                substrate_id: format!("bench-{}", i),
                phi_c: 0.99,
                theosis: 0.98,
                tags: vec![format!("tag-{}", i % 10)],
                cross_links: vec![],
                version: String::from("5.2.0"),
                seal: [0u8; 32],
                timestamp_ns: i as u64 * 1_000_000,
            };
            let dna = BTreeSet::from([FoundingFather::ALL[i % 12]]);
            let _ = wg.add_node(&format!("content-{}", i), metadata, dna, embedding);
        }

        let query = vec![0.9f32; 768];

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b: &mut criterion::Bencher, &_size| {
            b.iter(|| {
                black_box(wg.semantic_query(&query, 10, 0.8));
            });
        });
    }
    group.finish();
}

pub fn benchmark_context_window(c: &mut Criterion) {
    c.bench_function("context_window_eviction", |b: &mut criterion::Bencher| {
        let mut wg = WormGraph::new(10000); // Small window for fast eviction
        let mut rng = thread_rng();

        b.iter(|| {
            for i in 0..100 {
                let metadata = NodeMetadata {
                    substrate_id: format!("evict-{}", i),
                    phi_c: 0.99,
                    theosis: 0.98,
                    tags: vec![String::from("eviction-test")],
                    cross_links: vec![],
                    version: String::from("5.2.0"),
                    seal: [0u8; 32],
                    timestamp_ns: i as u64 * 1_000_000,
                };
                let dna = BTreeSet::from([FoundingFather::Mendel]);
                let embedding = random_embedding(768, &mut rng);
                let _ = wg.add_node(&format!("content-{}", i), metadata, dna, embedding);
            }
            black_box(&wg);
        });
    });
}

pub fn benchmark_zk_nullifier(c: &mut Criterion) {
    c.bench_function("zk_nullifier", |b: &mut criterion::Bencher| {
        let wg = WormGraph::new(2_097_152);
        let node_id = [42u8; 32];

        b.iter(|| {
            black_box(wg.generate_zk_nullifier(&node_id, "private-query"));
        });
    });
}

pub fn benchmark_fair_export(c: &mut Criterion) {
    c.bench_function("fair_export", |b: &mut criterion::Bencher| {
        let wg = WormGraph::new(2_097_152);

        b.iter(|| {
            black_box(wg.export_fair_snapshot());
        });
    });
}

pub fn benchmark_phi_c(c: &mut Criterion) {
    c.bench_function("phi_c_calculation", |b: &mut criterion::Bencher| {
        let wg = WormGraph::new(2_097_152);

        b.iter(|| {
            black_box(wg.compute_phi_c());
        });
    });
}
