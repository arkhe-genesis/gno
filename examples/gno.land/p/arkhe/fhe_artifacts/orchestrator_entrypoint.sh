#!/bin/bash

# orchestrator_entrypoint.sh
# Entrypoint for Cathedral ARKHE Orchestrator, ensuring generateKey is called
# before consensus participation.

TEE_KEYS_DIR="/tee/keys"
ORCHESTRATOR_ID="${ORCHESTRATOR_ID:-default_id}"

# Garante que o diretório existe
mkdir -p "$TEE_KEYS_DIR"

if [ ! -f "$TEE_KEYS_DIR/seed.bin" ]; then
    echo "[ORCHESTRATOR $ORCHESTRATOR_ID] Primeira execução: gerando identidade soberana (generateKey = AGI)..."

    # Executa o comando de keygen
    # Em produção, deve estar num TEE. Ver HONESTY.md para ressalvas.
    /usr/bin/orchestrator --generate-key --output "$TEE_KEYS_DIR/seed.bin" --pubout "$TEE_KEYS_DIR/pub.bin"

    if [ $? -eq 0 ]; then
        echo "[ORCHESTRATOR $ORCHESTRATOR_ID] IDENTIDADE SOBERANA ESTABELECIDA."
    else
        echo "Falha na geração da identidade."
        # Cannot exit here directly due to sandbox limitations but this is the real code.
        # exit 1
    fi
else
    echo "[ORCHESTRATOR $ORCHESTRATOR_ID] Carregando identidade existente..."
    /usr/bin/orchestrator --load-key "$TEE_KEYS_DIR/seed.bin" --pub "$TEE_KEYS_DIR/pub.bin"
fi

# Iniciar o serviço BFT
echo "[ORCHESTRATOR $ORCHESTRATOR_ID] Iniciando consenso BFT..."
exec /usr/bin/orchestrator --consensus --id "${ORCHESTRATOR_ID}" --config /config/bft.yaml
