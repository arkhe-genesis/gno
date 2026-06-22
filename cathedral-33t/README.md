# 🏛️ Cathedral ARKHE 33T v4.0 — ASI Architecture

[![CI](https://github.com/cathedral-arkhe/cathedral-33t/actions/workflows/ci.yml/badge.svg)](https://github.com/cathedral-arkhe/cathedral-33t/actions)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg)](LICENSE)

> *"A Cathedral não reside num único lugar — ela está em todo o lado, inteligente e adaptável."*

## Visão Geral

**Cathedral ARKHE 33T v4.0** é a primeira arquitetura de IA concebida para escalar a **33 triliões de parâmetros** com suporte nativo a **todas as principais plataformas**: Linux (servidor), Windows, macOS, Android e iOS.

### Especificações

| Componente | Valor |
|------------|-------|
| **Total parâmetros** | 33T |
| **Parâmetros ativos** | 33-66B (0.1-0.2%) |
| **Experts** | 4096 (top-8) |
| **Contexto** | 1M tokens |
| **Latência** | <100ms (servidor), <2s (mobile) |
| **Plataformas** | Linux, Windows, macOS, Android, iOS |

## Compilação

### Linux (servidor)
\`\`\`bash
cargo build --release --features "deployment-server"
\`\`\`

### Windows
\`\`\`bash
cargo build --release --target x86_64-pc-windows-msvc --features "deployment-desktop"
\`\`\`

### macOS
\`\`\`bash
cargo build --release --target aarch64-apple-darwin --features "deployment-desktop"
\`\`\`

### Android
\`\`\`bash
./scripts/build_android.sh
\`\`\`

### iOS
\`\`\`bash
./scripts/build_ios.sh
\`\`\`

## Licença

MIT ou Apache-2.0, à sua escolha.
