#!/bin/bash
# scripts/setup-precommit.sh — Instala pre-commit hooks para Safe-Core AGI

set -e

echo "🔧 Instalando pre-commit hooks para Safe-Core AGI..."

# Verificar se Python está instalado
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 não encontrado. Instalando..."
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo apt-get install -y python3 python3-pip
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        brew install python3
    else
        echo "❌ Sistema operacional não suportado. Instale Python 3 manualmente."
        exit 1
    fi
fi

# Instalar pre-commit
echo "📦 Instalando pre-commit..."
pip3 install --user pre-commit

# Instalar hooks
echo "🪝 Instalando hooks..."
pre-commit install

# Instalar ferramentas Rust necessárias
echo "🦀 Instalando ferramentas Rust..."
cargo install cargo-audit cargo-deny cargo-tarpaulin

# Verificar instalação
echo "✅ Verificando instalação..."
pre-commit --version
cargo audit --version
cargo deny --version

echo ""
echo "✅ Pre-commit hooks instalados com sucesso!"
echo ""
echo "📋 Hooks ativos:"
echo "  - cargo fmt (formatação)"
echo "  - cargo clippy (linting)"
echo "  - cargo audit (segurança)"
echo "  - cargo deny (dependências)"
echo "  - cargo test (testes unitários)"
echo "  - yamllint (YAML)"
echo "  - markdownlint (Markdown)"
echo ""
echo "💡 Para executar manualmente:"
echo "  pre-commit run --all-files"
echo ""
echo "🚀 Para pular hooks (emergência):"
echo "  git commit --no-verify"
