#!/bin/bash
# scripts/setup-regtest.sh

set -e

echo "🏛️  Configurando ambiente Regtest para Taproot Assets"

# 1. Iniciar containers
docker-compose up -d

# 2. Aguardar bitcoind
echo "⏳ Aguardando bitcoind..."
sleep 10

# 3. Gerar blocos iniciais
docker exec -it cathedral-bitcoind-1 bitcoin-cli -regtest -rpcuser=devuser -rpcpassword=devpass generate 101

# 4. Obter endereços LND
ALICE_ADDR=$(docker exec -it cathedral-lnd-alice-1 lncli --network=regtest newaddress p2wkh | jq -r '.address')
BOB_ADDR=$(docker exec -it cathedral-lnd-bob-1 lncli --network=regtest newaddress p2wkh | jq -r '.address')

# 5. Enviar BTC para os nós
docker exec -it cathedral-bitcoind-1 bitcoin-cli -regtest -rpcuser=devuser -rpcpassword=devpass sendtoaddress $ALICE_ADDR 10
docker exec -it cathedral-bitcoind-1 bitcoin-cli -regtest -rpcuser=devuser -rpcpassword=devpass sendtoaddress $BOB_ADDR 10

# 6. Confirmar transações
docker exec -it cathedral-bitcoind-1 bitcoin-cli -regtest -rpcuser=devuser -rpcpassword=devpass generate 6

# 7. Conectar LND Alice e Bob
ALICE_PUBKEY=$(docker exec -it cathedral-lnd-alice-1 lncli --network=regtest getinfo | jq -r '.identity_pubkey')
docker exec -it cathedral-lnd-bob-1 lncli --network=regtest connect $ALICE_PUBKEY@lnd-alice:9735

# 8. Abrir canal entre Alice e Bob
docker exec -it cathedral-lnd-alice-1 lncli --network=regtest openchannel --node_key=$ALICE_PUBKEY --local_amt=5000000

# 9. Confirmar canal
docker exec -it cathedral-bitcoind-1 bitcoin-cli -regtest -rpcuser=devuser -rpcpassword=devpass generate 6

echo "✅ Ambiente Regtest configurado com sucesso!"
echo "📍 Alice: localhost:10029 (gRPC) / 8089 (REST)"
echo "📍 Bob: localhost:10030 (gRPC) / 8090 (REST)"
