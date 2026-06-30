# 🚀 Safe-Core AGI — Guia de Onboarding para Novos Contribuidores

**Bem-vindo ao Safe-Core AGI!** Este guia vai te ajudar a configurar seu ambiente, entender a arquitetura e começar a contribuir em menos de 30 minutos.

---

## 📋 Pré-requisitos

Antes de começar, certifique-se de ter instalado:

| Ferramenta | Versão Mínima | Comando para verificar |
|------------|---------------|------------------------|
| **Rust** | 1.85+ | `rustc --version` |
| **Cargo** | 1.85+ | `cargo --version` |
| **Git** | 2.40+ | `git --version` |
| **Docker** | 24.0+ | `docker --version` |
| **protoc** | 3.20+ | `protoc --version` |

### Instalação Rápida (Linux/macOS)

```bash
# Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Protobuf
# Ubuntu/Debian:
sudo apt install protobuf-compiler
# macOS:
brew install protobuf

# Docker
# https://docs.docker.com/engine/install/
```

---

## 📦 Clonando e Compilando

```bash
# Clonar o repositório
git clone https://github.com/arkhe-research/safe-core.git
cd safe-core

# Compilar workspace
cargo build --workspace

# Executar testes
cargo test --workspace

# Executar CLI
cargo run -p safe-core-cli -- status
```

---

## 🏛️ Entendendo a Arquitetura

O Safe-Core AGI é um **workspace Rust** com **12+ crates** organizados em camadas:

```
┌─────────────────────────────────────────────────────┐
│  CAMADA 9 — Interfaces (CLI, TUI, Web GUI)         │
├─────────────────────────────────────────────────────┤
│  CAMADA 8 — Kernel (Orquestração Multiagente)      │
├─────────────────────────────────────────────────────┤
│  CAMADA 7 — Watchdog (Monitoramento SI-Level)      │
├─────────────────────────────────────────────────────┤
│  CAMADA 6 — MLFM Oracle (Oráculo Restrito PAE)     │
├─────────────────────────────────────────────────────┤
│  CAMADA 5 — Governança (Máquina de Estado CAIS)    │
├─────────────────────────────────────────────────────┤
│  CAMADA 4 — Transparência (Ledger Imutável)        │
├─────────────────────────────────────────────────────┤
│  CAMADA 3 — Persistência (RocksDB)                 │
├─────────────────────────────────────────────────────┤
│  CAMADA 2 — HSM Backends (YubiKey, TPM, Software)  │
├─────────────────────────────────────────────────────┤
│  CAMADA 1 — Merkle Tree (Integridade Criptográfica)│
├─────────────────────────────────────────────────────┤
│  CAMADA 0 — Crypto Multi-Algo (Primitivas)         │
└─────────────────────────────────────────────────────┘
```

### Crates Principais

| Crate | Responsabilidade |
|-------|------------------|
| `crypto-multi-algo` | Assinaturas, hashing, canonicalização |
| `merkle-tree` | Árvore de Merkle com proofs O(log N) |
| `hw-backends` | YubiHSM, TPM 2.0, SoftHSM |
| `persistence` | RocksDB Column Families |
| `transparency` | Ledger imutável com STH |
| `governance` | Ações MultiSig e ReactiveLog |
| `mlfm-oracle` | LLM Oracle com sanitização PAE |
| `watchdog` | Monitoramento Prometheus |
| `cognitive-core` | Planejamento Hierárquico |
| `action-executor` | Sandbox com Landlock + seccomp |
| `memory-system` | Qdrant + Merkle sealing |
| `kernel` | Orquestrador Multiagente |

---

## 🧪 Executando Testes

```bash
# Todos os testes (unitários + integração)
cargo test --workspace

# Apenas unitários (rápidos)
cargo test --workspace --lib

# Apenas testes de integração (exige Qdrant)
cargo test --workspace --test '*' -- --ignored
```

---

## 🔧 Configurando o Ambiente de Desenvolvimento

### Configuração do `rust-analyzer` (VS Code)

```json
// .vscode/settings.json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.cargo.target": "x86_64-unknown-linux-gnu",
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.diagnostics.enable": true
}
```

### Variáveis de Ambiente

```bash
# Crie um arquivo .env na raiz
SAFE_CORE_DATA_DIR=./data
SAFE_CORE_LOG_LEVEL=debug
SAFE_CORE_QDRANT_URL=http://localhost:6334
SAFE_CORE_PROMETHEUS_URL=http://localhost:9090
```

---

## 📝 Como Contribuir

### 1. Escolha uma Issue

- **Good First Issues**: [Link para issues com label]
- **Help Wanted**: [Link para issues com label]

### 2. Branch e Commits

```bash
# Criar branch a partir de main
git checkout -b feature/minha-feature

# Seguir Conventional Commits
git commit -m "feat(crypto): add Ed25519 signature verification"
```

### 3. Antes do PR

```bash
# Executar pre-commit hooks
pre-commit run --all-files

# Ou manualmente:
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
```

### 4. Código de Conduta

Este projeto segue o [Contributor Covenant](https://www.contributor-covenant.org/).
Seja respeitoso e colaborativo.

---

## 🐛 Troubleshooting Comum

| Problema | Solução |
|----------|---------|
| `protoc` não encontrado | Instalar `protobuf-compiler` |
| `libclang` não encontrado | Instalar `libclang-dev` (Linux) ou `llvm` (macOS) |
| Qdrant não responde | Verificar se Docker está rodando e porta 6334 livre |
| `cargo test` falha | Executar `cargo clean` e `cargo build` primeiro |
| `cargo audit` falha | Verificar se há vulnerabilidades conhecidas; ignorar com `cargo audit --ignore RUSTSEC-...` |

---

## 📚 Recursos Adicionais

- **Documentação Técnica**: [docs/architecture.md](architecture.md)
- **Guia de Segurança**: [docs/security.md](security.md)
- **API Reference**: [docs/api.md](api.md)
- **CAIS Model**: [docs/cais.md](cais.md)

---

## 💬 Comunicação

- **Discord**: [Link para Discord]
- **Slack**: [Link para Slack]
- **Issues**: [GitHub Issues]

---

**Última atualização:** 2026-06-29
**Versão do guia:** v3.0

---

*"O Safe-Core AGI é um sistema complexo, mas com ferramentas certas
e colaboração, podemos construir uma AGI segura juntos."*
