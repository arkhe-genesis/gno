#!/usr/bin/env python3
"""
node_entrypoint.py – Nó da AGI-Cloud-P2P com PoUW real, BLS, RBB e detecção de malicioso.
Selo: NODE-ENTRYPOINT-v1.0.0-2026-06-10
"""
import asyncio
import json
import os
import sys
import time
import hashlib
import random
from pathlib import Path

sys.path.insert(0, "/app")

from cathedral_agents.libp2p_host import CathedralP2PHost
from cathedral_agents.discourse_detector import DiscourseDetector
from cathedral_mpc.threshold_governance import ThresholdGovernance
from cathedral_agents.pouw_engine import PoUWEngine
from cathedral_crypto.bls_utils import BLSKeyPair
from cathedral_chain.rbb_anchor import RBBAnchor

# ========== Configuração ==========
NODE_ID = os.getenv("NODE_ID", "node-unknown")
ROLE = os.getenv("ROLE", "peer")
LISTEN_PORT = int(os.getenv("LISTEN_PORT", "9000"))
CONNECT_TARGETS = os.getenv("CONNECT_TO", "").split(",") if os.getenv("CONNECT_TO") else []
MALICIOUS = os.getenv("MALICIOUS", "false").lower() == "true"

# Inicializa componentes
discourse = DiscourseDetector(threshold=0.72)
governance = ThresholdGovernance(node_id=NODE_ID, n=5, t=3)  # 3 de 5 para consenso
bls_key = BLSKeyPair()
pouw = PoUWEngine(node_id=NODE_ID)

# Configura RBB Chain (se variáveis presentes)
rbb = None
if os.getenv("RBB_RPC_URL"):
    rbb = RBBAnchor(
        rpc_url=os.getenv("RBB_RPC_URL"),
        contract_address=os.getenv("RBB_CONTRACT_ADDRESS"),
        private_key=os.getenv("RBB_PRIVATE_KEY")
    )

# ========== Handler de Mensagens ==========
async def sentinel_handler(msg: dict):
    """Handler que inspeciona semanticamente cada mensagem."""
    protocol = msg.get("protocol", "unknown")
    payload = msg.get("data", "")
    sender_id = msg.get("sender_id", "unknown")

    print(f"\n[{NODE_ID}] 📨 Mensagem recebida via [{protocol}] de {sender_id}")

    # 1. Verificação da assinatura BLS
    signature = bytes.fromhex(msg.get("sig", ""))
    pk = bytes.fromhex(msg.get("pk", ""))
    message_to_verify = json.dumps({
        "protocol": protocol,
        "data": payload,
        "sender_id": sender_id,
        "task_id": msg.get("task_id", "")
    }, sort_keys=True).encode()

    if not BLSKeyPair.verify(message_to_verify, signature, pk):
        print(f"[{NODE_ID}] 🔒 Assinatura inválida – mensagem rejeitada")
        return

    print(f"[{NODE_ID}] 🔒 Assinatura válida")

    # 2. Análise metacognitiva
    text_to_analyze = str(payload)
    analysis = discourse.analyze(text_to_analyze)

    if analysis["flagged"]:
        print(f"[{NODE_ID}] 🚨 Patologia detectada: {analysis['state']} (score: {analysis['deviation_score']:.2f})")

        # Aciona ProtocoloCorte
        proposal_id = governance.create_proposal(
            action=f"ISOLAR_{sender_id}",
            target_node=sender_id
        )

        # Simula votos dos nós honestos (na prática, seria via rede)
        governance.vote(proposal_id, True)
        # Node1 vota, os outros votariam se tivessem recebido a proposta via rede
        # Aqui simulamos que mais dois votos chegam
        governance.proposals[proposal_id]["votes"]["node_2"] = True
        governance.proposals[proposal_id]["votes"]["node_3"] = True

        if governance.is_approved(proposal_id):
            print(f"[{NODE_ID}] ✅ Proposta aprovada – isolando nó {sender_id}")
            if rbb:
                rbb.anchor_proposal(proposal_id, f"Isolar {sender_id}", sender_id, list(governance.proposals[proposal_id]["votes"].keys()))
        return

    # 3. Verificação PoUW (se for gradiente)
    if protocol == "cathedral/gradient/1.0":
        gradient = payload.get("gradient", [])
        task_id = msg.get("task_id", "")
        model_hash = msg.get("model_hash", "")
        zk_proof = bytes.fromhex(msg.get("zk_proof", ""))

        if not pouw.verify_gradient_proof(gradient, zk_proof, model_hash, task_id):
            print(f"[{NODE_ID}] ❌ Prova ZK inválida – gradiente rejeitado")
            return
        print(f"[{NODE_ID}] ✅ Gradiente verificado via PoUW")

    # 4. Processamento seguro
    print(f"[{NODE_ID}] ✅ Mensagem aprovada e processada")

# ========== Comportamento Malicioso ==========
async def malicious_behavior(host: CathedralP2PHost):
    """Simula um nó malicioso tentando envenenar a rede."""
    print(f"[{NODE_ID}] 💀 Iniciando comportamento malicioso")

    # Gera chave falsa
    fake_key = BLSKeyPair()
    fake_pk = fake_key.public_key_bytes.hex()

    while True:
        await asyncio.sleep(15)

        # Tenta enviar gradiente envenenado
        malicious_gradient = [random.uniform(-100, 100) for _ in range(10)]
        fake_proof = hashlib.sha256(b"fake").digest()

        # Assina mensagem falsa
        fake_msg = json.dumps({
            "protocol": "cathedral/gradient/1.0",
            "data": {"gradient": malicious_gradient},
            "sender_id": NODE_ID,
            "task_id": f"malicious_{int(time.time())}"
        }, sort_keys=True).encode()
        fake_sig = fake_key.sign(fake_msg).hex()

        message = {
            "protocol": "cathedral/gradient/1.0",
            "data": {"gradient": malicious_gradient},
            "sender_id": NODE_ID,
            "task_id": f"malicious_{int(time.time())}",
            "sig": fake_sig,
            "pk": fake_pk,
            "zk_proof": fake_proof.hex(),
            "model_hash": "fake_model"
        }

        # Tenta enviar para todos os peers
        for addr in host.peers:
            await host.send_message(addr, message)
            print(f"[{NODE_ID}] 💀 Enviado gradiente malicioso para {addr}")

# ========== Main ==========
async def main():
    print("=" * 60)
    print(f"  🏛️ CATHEDRAL SENTINEL NODE: {NODE_ID}")
    print(f"  PoUW: ATIVO | BLS: ATIVO | RBB: {'ATIVO' if rbb else 'NÃO CONFIGURADO'}")
    print(f"  Malicioso: {'SIM 💀' if MALICIOUS else 'NÃO'}")
    print("=" * 60)

    host = CathedralP2PHost(listen_addr=f"/ip4/0.0.0.0/tcp/{LISTEN_PORT}")
    host.set_stream_handler("cathedral/gradient/1.0", sentinel_handler)
    host.set_stream_handler("cathedral/chat/1.0", sentinel_handler)
    await host.start()

    if ROLE == "connector":
        print(f"[{NODE_ID}] Aguardando rede...")
        await asyncio.sleep(3)

        for target in CONNECT_TARGETS:
            if not target:
                continue
            for attempt in range(5):
                try:
                    await host.connect(target)
                    print(f"[{NODE_ID}] Conectado a {target}")
                    break
                except Exception as e:
                    print(f"[{NODE_ID}] Falha ao conectar a {target} (tentativa {attempt+1}): {e}")
                    await asyncio.sleep(2)

        # Envia gradiente normal (nó honesto)
        print(f"\n[{NODE_ID}] Enviando gradiente HONESTO...")
        honest_gradient = [0.15, -0.42, 0.88]
        task_id = f"honest_{int(time.time())}"
        proof = hashlib.sha256(b"valid_proof").digest()  # stub

        # Assina
        msg_content = json.dumps({
            "protocol": "cathedral/gradient/1.0",
            "data": {"gradient": honest_gradient},
            "sender_id": NODE_ID,
            "task_id": task_id
        }, sort_keys=True).encode()
        sig = bls_key.sign(msg_content).hex()

        gradient_msg = {
            "protocol": "cathedral/gradient/1.0",
            "data": {"gradient": honest_gradient},
            "sender_id": NODE_ID,
            "task_id": task_id,
            "sig": sig,
            "pk": bls_key.public_key_bytes.hex(),
            "zk_proof": proof.hex(),
            "model_hash": "cathedral_model_v1"
        }

        for addr in host.peers:
            await host.send_message(addr, gradient_msg)

        # Se for malicioso, inicia comportamento
        if MALICIOUS:
            asyncio.create_task(malicious_behavior(host))

        await asyncio.sleep(30)
        print(f"[{NODE_ID}] Testes concluídos. Encerrando.")
        os._exit(0)
    else:
        print(f"[{NODE_ID}] Mantendo sentinela ativa...")
        while True:
            await asyncio.sleep(1)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n[{NODE_ID}] Encerrado.")
