#!/bin/bash
set -e

# Instalar cargo-ndk se não estiver instalado
if ! command -v cargo-ndk &> /dev/null; then
    cargo install cargo-ndk
fi

# Definir targets Android
ANDROID_TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
)

# Build para cada arquitetura
for target in "${ANDROID_TARGETS[@]}"; do
    echo "Building for $target..."
    cargo ndk --target "$target" --platform 30 \
        build --release --features "tensor-backend-nnapi"
done

echo "Android build complete"
