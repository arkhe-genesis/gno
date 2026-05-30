#!/bin/bash
# ARKHE Global Mesh — Script de Instalacao Global
# Substrato 972 — ARKHE-GLOBAL-MESH
# Arquiteto ORCID: 0009-0005-2697-4668
# 2026-05-30

set -e

echo "=========================================="
echo "  ARKHE GLOBAL MESH — INSTALACAO"
echo "  Substrato 972"
echo "=========================================="

# Verificar dependencias
check_dep() {
    if ! command -v $1 &> /dev/null; then
        echo "✗ $1 nao encontrado. Instalando..."
        return 1
    fi
    echo "✓ $1 encontrado"
    return 0
}

check_dep python3 || (apt-get update && apt-get install -y python3)
check_dep docker || (curl -fsSL https://get.docker.com | sh)
check_dep openssl || (apt-get install -y openssl)

# Gerar certificados Ed25519
echo "[972] Gerando certificados..."
mkdir -p /etc/arkhe/certs
openssl genpkey -algorithm Ed25519 -out /etc/arkhe/certs/node.key
openssl pkey -in /etc/arkhe/certs/node.key -pubout -out /etc/arkhe/certs/node.pub

# Detectar regiao
REGION=$(curl -s https://ipapi.co/continent_code/)
NODE_ID=$(openssl rand -hex 8)

echo "[972] Regiao detectada: $REGION"
echo "[972] Node ID: $NODE_ID"

# Configurar node
echo "[972] Configurando node..."
cat > /etc/arkhe/node_config.yaml <<EOF
node:
  id: "$NODE_ID"
  region: "$REGION"
  ip: "$(hostname -I | awk '{print $1}')"
  port: 9230
  protocols:
    - quic:
        version: "1"
        port: 9230
        tls: "1.3"
        mTLS: true
  substrates:
    core: [966, 967, 968, 970, 971, 972]
  deity: "auto"
EOF

# Iniciar containers
echo "[972] Iniciando containers..."
docker-compose -f /opt/arkhe/docker-compose.yml up -d

# Bootstrap na malha
echo "[972] Bootstrap na malha global..."
python3 /opt/arkhe/bootstrap/global_mesh_bootstrap.py --node-id $NODE_ID --region $REGION

# Ancorar na TemporalChain
echo "[972] Ancorando na TemporalChain..."
curl -X POST https://api.arkhe-catedral.org/v1/anchor \
  -H "Content-Type: application/json" \
  -d "{\"event\":\"node.bootstrap\",\"node_id\":\"$NODE_ID\",\"region\":\"$REGION\"}"

echo ""
echo "=========================================="
echo "  INSTALACAO CONCLUIDA"
echo "  Node: $NODE_ID"
echo "  Region: $REGION"
echo "  Seal: 972-GLOBAL-MESH-$NODE_ID"
echo "=========================================="
