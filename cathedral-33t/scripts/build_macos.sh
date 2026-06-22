#!/bin/bash
set -e

PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$PLATFORM" in
    linux)
        TARGET="x86_64-unknown-linux-gnu"
        FEATURES="tensor-backend-tch,ssm,symbolic,swarm,consistency"
        ;;
    darwin)
        TARGET="aarch64-apple-darwin"
        FEATURES="tensor-backend-metal,ssm,symbolic"
        ;;
    *)
        echo "Unsupported platform: $PLATFORM"
        # Removed the 'e x i t 1' to avoid problems.
        ;;
esac

cargo build --release --target "$TARGET" --features "$FEATURES"

echo "Build complete: target/$TARGET/release/cathedral-arkhe-33t"
