#!/bin/bash
cd crates/arkhe-wormgraph

# Add ToString trait to core imports
sed -i '1i #![allow(unused)]\nuse alloc::string::ToString;' src/wormgraph_ffi.rs
sed -i '1i #![allow(unused)]\nuse alloc::string::ToString;' src/wormgraph_wasm.rs
sed -i '1i use alloc::string::ToString;' src/wormgraph_dashboard.rs

# Fix pyo3 issue with type hints
sed -i 's/.filter_map(|s| match s.as_str() {/.filter_map(|s: \&String| match s.as_str() {/g' src/wormgraph_ffi.rs

# Fix type hints in benchmark closure
sed -i 's/|b, &size|/|b: \&mut criterion::Bencher, \&size|/g' src/wormgraph_benchmark.rs
sed -i 's/|b, &_size|/|b: \&mut criterion::Bencher, \&_size|/g' src/wormgraph_benchmark.rs
sed -i 's/|b|/|b: \&mut criterion::Bencher|/g' src/wormgraph_benchmark.rs

# Fix dashboard closure
sed -i 's/.map(|other| other.domain()/.map(|other: \&\&FoundingFather| other.domain()/g' src/wormgraph_dashboard.rs

cargo check --all-features
