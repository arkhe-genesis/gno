#!/bin/bash
# scripts/run-integration-tests.sh

echo "🏛️  Executando testes de integração Taproot Assets"

# Executa os testes
cargo test -p cathedral-taproot-bridge --test integration -- --nocapture --test-threads=1

echo "✅ Testes de integração concluídos"
