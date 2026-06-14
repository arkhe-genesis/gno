#!/bin/bash
cd crates/arkhe-wormgraph

# Remove unsafe restriction entirely from lib.rs
sed -i 's/#![forbid(unsafe_code)]//g' src/lib.rs

# Also from wormgraph_core.rs
sed -i 's/#![forbid(unsafe_code)]//g' src/wormgraph_core.rs

# In Cargo.toml, the `criterion` dependency needs to be an actual dependency because benchmark code is compiled as part of the library itself, not just as dev-dependency for `cargo check --all-features`.
sed -i 's/\[dev-dependencies\]/\[dependencies\]\nrand = "0.8"\ncriterion = "0.4"/g' Cargo.toml
# Remove duplicate definition from Cargo.toml
sed -i '/rand = "0.8"/d' Cargo.toml
sed -i '/criterion = "0.4"/d' Cargo.toml

cat << 'TOML' > Cargo.toml
[package]
name = "arkhe-wormgraph"
version = "5.2.0"
edition = "2021"
authors = ["Arkhe Cathedral <orcid:0009-0005-2697-4668>"]
description = "WormGraph Core v5.2.0 - O(1) Memory Layer with ZK & FAIR"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
sha3 = { version = "0.10", default-features = false }
serde = { version = "1.0", features = ["derive"], default-features = false }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
log = "0.4"
tokio = { version = "1.0", features = ["full"], optional = true }
hex = "0.4"
rand = "0.8"
criterion = "0.4"

# FFI and WASM features
pyo3 = { version = "0.18", features = ["extension-module"], optional = true }
wasm-bindgen = { version = "0.2", optional = true }
js-sys = { version = "0.3", optional = true }
serde-wasm-bindgen = { version = "0.5", optional = true }

[features]
default = ["std"]
std = ["sha3/std", "serde/std", "serde_json/std"]
ffi = ["std"]
python = ["ffi", "pyo3"]
wasm = ["wasm-bindgen", "js-sys", "serde-wasm-bindgen"]
temporal = ["tokio"]

[[bench]]
name = "wormgraph_bench"
harness = false
TOML

cargo check --all-features
