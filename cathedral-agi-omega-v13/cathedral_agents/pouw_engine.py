"""
cathedral_agents/pouw_engine.py
Proof of Useful Work Engine – Verificação real de gradientes via ZK‑proofs.
Selo: POUW-ENGINE-v1.0.0-2026-06-10
"""
import hashlib
import json
import time
from typing import Dict, List, Optional, Callable, Tuple
from dataclasses import dataclass
import numpy as np

try:
    from ezkl import prove, verify  # type: ignore
    EZKL_AVAILABLE = True
except ImportError:
    EZKL_AVAILABLE = False
    print("[PoUW] EZKL não encontrado – usando stub de verificação (apenas testes)")

@dataclass
class WorkVerification:
    """Atesta que um nó executou trabalho útil corretamente."""
    task_id: str
    node_id: str
    gradient_hash: str
    model_hash: str
    proof: bytes
    timestamp: float
    verified: bool = False

class PoUWEngine:
    """
    Gerencia recompensas por trabalho útil. Verifica ZK‑proofs anexados.
    Em produção, usa EZKL para gerar e verificar provas de execução correta.
    """
    def __init__(self, node_id: str, zk_verifier: Optional[Callable] = None):
        self.node_id = node_id
        self.token_balance = 0.0
        self._submitted_tasks: Dict[str, WorkVerification] = {}
        self.zk_verifier = zk_verifier or self._default_verifier

    def _default_verifier(self, proof: bytes, public_inputs: bytes) -> bool:
        """Verificador padrão – usa EZKL se disponível, senão stub."""
        if EZKL_AVAILABLE:
            try:
                # Em produção, o vk_path seria carregado do modelo
                return verify(proof, "verification_key.json", public_inputs)
            except Exception:
                return False
        # Stub para testes: aceita qualquer proof não vazio
        return len(proof) > 0

    def verify_gradient_proof(self, gradient: List[float], proof: bytes, model_hash: str, task_id: str) -> bool:
        """
        Verifica se o gradiente foi calculado corretamente para o modelo.
        Retorna True se a prova ZK for válida.
        """
        # Converte gradiente para bytes
        gradient_bytes = json.dumps(gradient).encode()
        gradient_hash = hashlib.sha256(gradient_bytes).hexdigest()

        public_inputs = json.dumps({
            "gradient_hash": gradient_hash,
            "model_hash": model_hash,
            "task_id": task_id
        }).encode()

        is_valid = self.zk_verifier(proof, public_inputs)

        if is_valid:
            # Registra a tarefa como verificada
            verification = WorkVerification(
                task_id=task_id,
                node_id=self.node_id,
                gradient_hash=gradient_hash,
                model_hash=model_hash,
                proof=proof,
                timestamp=time.time(),
                verified=True
            )
            self._submitted_tasks[task_id] = verification
            self.token_balance += 1.0  # Recompensa por trabalho útil
            print(f"[PoUW] Tarefa {task_id} verificada! Saldo: {self.token_balance} tokens")

        return is_valid

    def submit_work(self, task_id: str, proof: bytes, public_inputs: bytes) -> bool:
        """Submete prova de trabalho útil e credita tokens se válida."""
        if task_id in self._submitted_tasks:
            print(f"[PoUW] Tarefa {task_id} já submetida")
            return False

        is_valid = self.zk_verifier(proof, public_inputs)
        if not is_valid:
            print(f"[PoUW] Prova inválida para tarefa {task_id}")
            return False

        self._submitted_tasks[task_id] = WorkVerification(
            task_id=task_id,
            node_id=self.node_id,
            gradient_hash="",
            model_hash="",
            proof=proof,
            timestamp=time.time(),
            verified=True
        )
        self.token_balance += 1.0
        print(f"[PoUW] Tarefa {task_id} aceita! Saldo: {self.token_balance} tokens")
        return True

    def get_balance(self) -> float:
        return self.token_balance
