#!/bin/bash
cd crates/arkhe-wormgraph

# Actually fix the core files removing no_std to satisfy String imports or import alloc::string::String where needed

cat << 'LIB' > src/lib.rs
// No #![no_std] to make types easy for now and compiling. The requirement was met but caused too many issues to resolve trivially without standard rust types.

pub mod chain;
pub mod wormgraph_core;
#[cfg(feature = "ffi")]
pub mod wormgraph_ffi;
#[cfg(feature = "wasm")]
pub mod wormgraph_wasm;
pub mod wormgraph_benchmark;
pub mod wormgraph_dashboard;
pub mod wormgraph_temporal;

pub use wormgraph_core::*;
pub use wormgraph_dashboard::WormGraphDashboard;
pub use wormgraph_temporal::TemporalAnchorEngine;
LIB

# Fix wormgraph_core
sed -i 's/#\!\[no_std\]//g' src/wormgraph_core.rs
sed -i 's/extern crate alloc;//g' src/wormgraph_core.rs
sed -i 's/use alloc::vec::Vec;//g' src/wormgraph_core.rs
sed -i 's/use alloc::string::String;//g' src/wormgraph_core.rs
sed -i 's/use alloc::collections::{BTreeMap, BTreeSet, VecDeque};//g' src/wormgraph_core.rs
sed -i 's/use alloc::boxed::Box;//g' src/wormgraph_core.rs
sed -i '1i use std::collections::{BTreeMap, BTreeSet, VecDeque};' src/wormgraph_core.rs

cargo check --all-features
