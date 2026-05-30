#!/usr/bin/env python3
"""
ARKHE Global Mesh — Hamiltonian Consensus
Substrato 972 — ARKHE-GLOBAL-MESH

Algoritmo de consenso baseado em Theosis (965).
Nos votam com peso proporcional a sua Theosis.
"""

import numpy as np
from typing import Dict, List
from dataclasses import dataclass

@dataclass
class Vote:
    node_id: str
    proposal_id: str
    vote: float  # 0-1
    theosis_weight: float
    seal: str

class HamiltonianConsensus:
    """Consenso por Theosis integrando reputacao Nostr."""

    def __init__(self, threshold: float = 0.67):
        self.threshold = threshold
        self.votes: Dict[str, List[Vote]] = {}
        # Nostr reputation scores mapped by node_id (0.0 to 1.0)
        self.nostr_reputation: Dict[str, float] = {}
        # Reputacao padrao se o no nao tem score registrado
        self.default_reputation = 0.5

    def update_nostr_reputation(self, node_id: str, reputation: float):
        """Atualiza a reputacao de um relay via Nostr (Axiarchy)."""
        self.nostr_reputation[node_id] = max(0.0, min(1.0, reputation))

    def get_effective_theosis(self, vote: Vote) -> float:
        """Aplica o peso de reputacao Nostr ao Theosis original do no."""
        reputation = self.nostr_reputation.get(vote.node_id, self.default_reputation)
        # O peso final e modulado pela reputacao do relay
        return vote.theosis_weight * (0.5 + 0.5 * reputation)

    def propose(self, proposal_id: str, description: str) -> bool:
        """Cria nova proposta."""
        self.votes[proposal_id] = []
        return True

    def vote(self, proposal_id: str, vote: Vote) -> bool:
        """Registra voto."""
        if proposal_id not in self.votes:
            return False
        self.votes[proposal_id].append(vote)
        return True

    def tally(self, proposal_id: str) -> Dict:
        """Conta votos ponderados por Theosis e reputacao Nostr."""
        if proposal_id not in self.votes:
            return {"approved": False, "reason": "No votes"}

        votes = self.votes[proposal_id]
        total_weight = sum(self.get_effective_theosis(v) for v in votes)

        if total_weight == 0:
            return {"approved": False, "reason": "Zero weight"}

        weighted_vote = sum(v.vote * self.get_effective_theosis(v) for v in votes) / total_weight

        return {
            "approved": weighted_vote >= self.threshold,
            "weighted_vote": weighted_vote,
            "total_votes": len(votes),
            "total_weight": total_weight,
        }
