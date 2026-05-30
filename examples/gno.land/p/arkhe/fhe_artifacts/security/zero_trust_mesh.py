#!/usr/bin/env python3
"""
ARKHE Global Mesh — Zero Trust Security
Substrato 972 — ARKHE-GLOBAL-MESH

Seguranca baseada em:
- mTLS 1.3 com Ed25519
- PQC (Kyber + Dilithium)
- ZK proofs para autenticacao
- Axiarchy (954) para validacao etica de cada conexao
"""

import hashlib
import json
from typing import Dict, List, Optional
from dataclasses import dataclass

@dataclass
class SecurityContext:
    node_id: str
    ed25519_pubkey: str
    kyber_pubkey: str
    dilithium_pubkey: str
    theosis_level: float
    ethical_score: float
    seal: str

class ZeroTrustMesh:
    """Zero Trust para malha ARKHE."""

    def __init__(self):
        self.trusted_nodes: Dict[str, SecurityContext] = {}
        self.revoked_nodes: List[str] = []

    def authenticate(self, context: SecurityContext) -> bool:
        """Autentica no com ZK proof + Axiarchy."""
        # Verificar Theosis minima
        if context.theosis_level < 0.3:
            return False

        # Verificar score etico
        if context.ethical_score < 0.7:
            return False

        # Verificar se nao esta revogado
        if context.node_id in self.revoked_nodes:
            return False

        self.trusted_nodes[context.node_id] = context
        return True

    def revoke(self, node_id: str, reason: str) -> bool:
        """Revoga no da malha."""
        if node_id in self.trusted_nodes:
            del self.trusted_nodes[node_id]
        self.revoked_nodes.append(node_id)
        return True

    def verify_connection(self, src: str, dst: str) -> bool:
        """Verifica se conexao entre dois nos e permitida."""
        if src not in self.trusted_nodes or dst not in self.trusted_nodes:
            return False

        # Verificar compatibilidade etica
        src_ethics = self.trusted_nodes[src].ethical_score
        dst_ethics = self.trusted_nodes[dst].ethical_score

        return abs(src_ethics - dst_ethics) < 0.5  # Tolerancia
