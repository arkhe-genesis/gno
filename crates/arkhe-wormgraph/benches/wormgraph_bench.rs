use criterion::criterion_main;
use criterion::criterion_group;
use arkhe_wormgraph::wormgraph_benchmark::*;

criterion_group!(
    benches,
    benchmark_add_node,
    benchmark_lookup,
    benchmark_semantic_query,
    benchmark_context_window,
    benchmark_zk_nullifier,
    benchmark_fair_export,
    benchmark_phi_c
);

criterion_main!(benches);
