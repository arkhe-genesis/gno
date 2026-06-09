"""
Cathedral ARKHE v9.0 LOGOS — Federated Learning with ZK Proofs
V9-010: Treinamento descentralizado sem compartilhar dados brutos.
Cada nodo prova (via ZK) que treinou corretamente.
Baseado em: ZK-ML proofs (emerging 2025), FL with DP (maturing).
Selo: FEDERATED-ZK-v9.0.0-2026-01-15
"""

from __future__ import annotations
import hashlib
import json
import time
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class FederatedConfig:
    # Federated learning
    n_nodes: int = 8
    min_nodes_per_round: int = 4
    max_rounds: int = 100
    local_epochs: int = 2
    local_lr: float = 1e-5
    # Aggregation
    aggregation_method: str = "fedavg"  # "fedavg", "fedprox", "scaffold"
    fedprox_mu: float = 0.01
    # Privacy
    differential_privacy: bool = True
    dp_epsilon: float = 10.0
    dp_delta: float = 1e-5
    dp_noise_multiplier: float = 0.8
    clip_norm: float = 1.0
    # ZK proofs
    zk_enabled: bool = True
    zk_circuit_depth: int = 10
    zk_proof_timeout: float = 60.0
    # Security
    byzantine_threshold: float = 0.25  # Tolerância a nodos byzantinos
    min_stake: float = 100.0           # Stake mínimo para participar


class ZKProofGenerator:
    """
    Gera provas zero-knowledge de que o treinamento local foi correto.

    Em produção: usa circuitos zk-SNARKs (Groth16/PLONK).
    Aqui: simulação com hash commitments.
    """

    def __init__(self, config: FederatedConfig):
        self.config = config

    def generate_training_proof(self, node_id: int,
                                 model_hash_before: str,
                                 model_hash_after: str,
                                 loss_before: float,
                                 loss_after: float,
                                 n_samples: int,
                                 clip_norm: float) -> Dict:
        """
        Gera prova de que:
        1. O modelo foi atualizado a partir do estado correto
        2. O loss diminuiu (ou ficou estável)
        3. O gradient foi clipped ao norm máximo
        4. O número de samples está dentro do permitido

        Retorna commitment + proof hash (em produção: proof serializado).
        """
        if not self.config.zk_enabled:
            return {"zk_enabled": False, "status": "skipped"}

        # Commitment: hash dos inputs
        commitment_input = json.dumps({
            "node": node_id,
            "model_before": model_hash_before,
            "model_after": model_hash_after,
            "loss_before": loss_before,
            "loss_after": loss_after,
            "n_samples": n_samples,
            "clip": clip_norm,
            "timestamp": time.time(),
        }, sort_keys=True)
        commitment = hashlib.sha256(commitment_input.encode()).hexdigest()

        # Proof: em produção, seria um zk-SNARK real
        # Aqui: simulamos com hash estendido
        proof_input = f"{commitment}:{node_id}:{loss_before}:{loss_after}"
        proof_hash = hashlib.sha256(proof_input.encode()).hexdigest()

        # Verificações simuladas
        loss_decreased = loss_after <= loss_before * 1.1  # Tolerância de 10%
        clip_ok = clip_norm <= self.config.clip_norm * 1.05
        samples_ok = n_samples > 0

        valid = loss_decreased and clip_ok and samples_ok

        return {
            "zk_enabled": True,
            "status": "valid" if valid else "invalid",
            "commitment": commitment,
            "proof_hash": proof_hash,
            "checks": {
                "loss_decreased": loss_decreased,
                "clip_ok": clip_ok,
                "samples_ok": samples_ok,
            },
            "node_id": node_id,
        }


class ZKProofVerifier:
    """Verifica provas ZK de outros nodos."""

    def __init__(self, config: FederatedConfig):
        self.config = config

    def verify_proof(self, proof: Dict) -> bool:
        """Verifica uma prova ZK."""
        if not self.config.zk_enabled or not proof.get("zk_enabled"):
            return True  # Se ZK desabilitado, confia

        if proof.get("status") != "valid":
            return False

        # Verificar que checks passaram
        checks = proof.get("checks", {})
        return all(checks.values())


class FederatedNode:
    """Representa um nodo federado."""

    def __init__(self, node_id: int, model: nn.Module,
                 config: FederatedConfig, data_size: int = 1000):
        self.node_id = node_id
        self.model = model
        self.config = config
        self.data_size = data_size
        self.stake = config.min_stake
        self.is_honest = True  # Para simulação de byzantine

    def get_model_hash(self) -> str:
        """Hash do estado atual do modelo."""
        state = {k: v.shape for k, v in self.model.state_dict().items()}
        return hashlib.sha256(json.dumps(state, sort_keys=True).encode()).hexdigest()

    def local_train(self, n_epochs: int = 2) -> Dict:
        """
        Treinamento local (simulado).
        Em produção: usa dados reais do nodo.
        """
        self.model.train()
        loss_before = 1.0  # Placeholder

        # Simular atualização
        for name, param in self.model.named_parameters():
            if param.requires_grad:
                noise = torch.randn_like(param) * 0.001
                if self.is_honest:
                    param.data += noise * self.config.local_lr
                else:
                    # Byzantine: atualização adversarial
                    param.data += noise * self.config.local_lr * 10

        loss_after = 0.95  # Placeholder: loss diminuiu

        # Differential privacy: clip gradients
        if self.config.differential_privacy:
            for param in self.model.parameters():
                param.data = param.data.clamp(-self.config.clip_norm, self.config.clip_norm)

        return {
            "node_id": self.node_id,
            "loss_before": loss_before,
            "loss_after": loss_after,
            "n_samples": self.data_size,
            "clip_norm": self.config.clip_norm,
            "is_honest": self.is_honest,
        }

    def get_update(self) -> Dict[str, torch.Tensor]:
        """Retorna delta do modelo (update para agregar)."""
        return {k: v.clone() for k, v in self.model.state_dict().items()}


class FederatedAggregator:
    """Agrega updates de múltiplos nodos."""

    def __init__(self, config: FederatedConfig, global_model: nn.Module):
        self.config = config
        self.global_model = global_model

    def aggregate(self, updates: List[Dict[str, torch.Tensor]],
                  weights: List[float]) -> Dict[str, torch.Tensor]:
        """FedAvg: média ponderada dos updates."""
        total_weight = sum(weights)
        aggregated = {}

        for key in updates[0].keys():
            weighted_sum = torch.zeros_like(updates[0][key], dtype=torch.float32)
            for update, w in zip(updates, weights):
                weighted_sum += update[key].float() * w
            aggregated[key] = (weighted_sum / total_weight).to(updates[0][key].dtype)

        return aggregated

    def apply_update(self, aggregated: Dict[str, torch.Tensor]):
        """Aplica update agregado ao modelo global."""
        self.global_model.load_state_dict(aggregated)


class FederatedZKTrainer:
    """
    Trainer federado com provas ZK.

    Ciclo por round:
    1. Broadcast modelo global para nodos selecionados
    2. Cada nodo treina localmente
    3. Cada nodo gera prova ZK do treinamento
    4. Agregador verifica provas ZK
    5. Apenas updates com prova válida são agregados
    6. Novo modelo global é broadcast
    """

    def __init__(self, config: FederatedConfig, global_model: nn.Module):
        self.config = config
        self.global_model = global_model
        self.nodes: List[FederatedNode] = []
        self.zk_gen = ZKProofGenerator(config)
        self.zk_verifier = ZKProofVerifier(config)
        self.aggregator = FederatedAggregator(config, global_model)
        self._round_history: List[Dict] = []

    def register_node(self, node: FederatedNode):
        if node.stake >= self.config.min_stake:
            self.nodes.append(node)

    def run_round(self, round_id: int) -> Dict:
        """Executa um round de treinamento federado."""
        # Selecionar nodos para este round
        selected = self._select_nodes()
        if len(selected) < self.config.min_nodes_per_round:
            return {"status": "error", "error": "Not enough nodes"}

        valid_updates = []
        valid_weights = []
        proof_results = []

        for node in selected:
            # Treino local
            model_hash_before = node.get_model_hash()
            train_result = node.local_train(self.config.local_epochs)
            model_hash_after = node.get_model_hash()

            # Gerar prova ZK
            proof = self.zk_gen.generate_training_proof(
                node_id=node.node_id,
                model_hash_before=model_hash_before,
                model_hash_after=model_hash_after,
                **train_result,
            )

            # Verificar prova
            proof_valid = self.zk_verifier.verify_proof(proof)
            proof_results.append({
                "node": node.node_id,
                "valid": proof_valid,
                "proof": proof.get("proof_hash", "none")[:16],
            })

            if proof_valid:
                update = node.get_update()
                valid_updates.append(update)
                valid_weights.append(node.data_size)

        # Agregar
        if valid_updates:
            aggregated = self.aggregator.aggregate(valid_updates, valid_weights)
            self.aggregator.apply_update(aggregated)

        round_result = {
            "round": round_id,
            "selected_nodes": len(selected),
            "valid_proofs": sum(1 for p in proof_results if p["valid"]),
            "rejected_proofs": sum(1 for p in proof_results if not p["valid"]),
            "proof_details": proof_results,
            "aggregated": len(valid_updates) > 0,
        }
        self._round_history.append(round_result)
        return round_result

    def _select_nodes(self) -> List[FederatedNode]:
        """Seleciona nodos para o round (por stake)."""
        sorted_nodes = sorted(self.nodes, key=lambda n: n.stake, reverse=True)
        return sorted_nodes[:self.config.n_nodes]

    def get_telemetry(self) -> dict:
        n_rounds = len(self._round_history)
        return {
            "module": "FederatedZKTrainer",
            "version": "9.0.0",
            "substrate": "v9-decentralized",
            "seal": "FEDERATED-ZK-v9.0.0-2026-01-15",
            "n_registered_nodes": len(self.nodes),
            "n_rounds_completed": n_rounds,
            "zk_enabled": self.config.zk_enabled,
            "dp_enabled": self.config.differential_privacy,
            "dp_epsilon": self.config.dp_epsilon if self.config.differential_privacy else None,
            "aggregation": self.config.aggregation_method,
            "byzantine_tolerance": self.config.byzantine_threshold,
        }
