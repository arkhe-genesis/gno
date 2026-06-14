#!/bin/bash
cd crates/arkhe-wormgraph

# Fix Dashboard closure map argument
sed -i 's/other: &&FoundingFather/other: \&FoundingFather/g' src/wormgraph_dashboard.rs

# Fix core format alloc
sed -i 's/alloc::format!/format!/g' src/wormgraph_core.rs

# Remove #![allow(unused)] and #![allow(unsafe_code)] from the first lines of files, because we use outer attributes like #[allow(unused)] or it fails in standard rust.
sed -i 's/#\!\[allow(unused)\]/#[allow(unused)]/g' src/wormgraph_ffi.rs
sed -i 's/#\!\[allow(unsafe_code)\]/#[allow(unsafe_code)]/g' src/wormgraph_ffi.rs
sed -i 's/#\!\[allow(non_snake_case)\]/#[allow(non_snake_case)]/g' src/wormgraph_ffi.rs

sed -i 's/#\!\[allow(unused)\]/#[allow(unused)]/g' src/wormgraph_wasm.rs
sed -i 's/#\!\[allow(unsafe_code)\]/#[allow(unsafe_code)]/g' src/wormgraph_wasm.rs

sed -i 's/use alloc::string::ToString;//g' src/wormgraph_ffi.rs
sed -i 's/use alloc::string::ToString;//g' src/wormgraph_wasm.rs
sed -i 's/use alloc::string::ToString;//g' src/wormgraph_dashboard.rs

cargo check --all-features
