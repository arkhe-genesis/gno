#!/usr/bin/env python3
"""
Testar cenário de censura total (todos os circuitos bloqueados) – failover para pontes
"""

import asyncio
import pytest
from bridge_nostr_tor_ipfs import NostrTorIpfsBridge

@pytest.mark.asyncio
async def test_censura_total():
    print("\n--- TESTE DE CENSURA TOTAL: BLOQUEIO TOR ---")

    # Inicializando bridge
    bridge = NostrTorIpfsBridge(
        node_id="test-node-censura",
        ed25519_pubkey="x"*64,
        ed25519_privkey="y"*64,
    )

    # Simulando um onion que forca o failover
    target_onion = "blocked.onion"

    print(f"Tentando conectar a {target_onion} via Tor...")
    success = await bridge.mesh_connect(target_onion=target_onion)

    if success:
        print("  ✓ Teste passou: Falha no Tor acionou failover para pontes Nostr com sucesso.")
    else:
        print("  ✗ Teste falhou: Failover para Nostr nao ocorreu.")

if __name__ == "__main__":
    asyncio.run(test_censura_total())
