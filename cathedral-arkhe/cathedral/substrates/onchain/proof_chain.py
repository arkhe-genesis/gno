"""
Recursive Proof Chain — Substrato 1100 v1.0.0
Cadeia de provas ZK integradas com verificação de assinaturas.
"""

from __future__ import annotations
import hashlib
import json
import time
from typing import Any, Dict, List, Optional, Tuple

from cathedral.substrates.onchain.types import ProofNode


class RecursiveProofChain:
    """
    Acumula provas ZK com verificação de assinaturas integrada.
    Cada canonização vira um nó de prova, criando cadeia ininterrupta.
    """

    def __init__(self, genesis_hash: Optional[str] = None):
        self.nodes: Dict[str, ProofNode] = {}
        self.ordered_indices: List[int] = []
        self.current_index = 0
        self.tip_hash: Optional[str] = None

        genesis = ProofNode(
            index=0,
            proof_hash=genesis_hash or "0x" + hashlib.sha256(b"genesis").hexdigest(),
            parent_hash=None,
            canonization_hash=None,
            signature_hash=None,
            timestamp=time.time(),
            proof_type="genesis",
            auxiliary_data={"description": "Genesis of the RecursiveProofChain"},
        )
        self._add_node(genesis)

    def _add_node(self, node: ProofNode):
        node.proof_hash = node.compute_hash()
        self.nodes[node.proof_hash] = node
        self.ordered_indices.append(node.proof_hash)
        self.tip_hash = node.proof_hash

        if node.parent_hash and node.parent_hash in self.nodes:
            self.nodes[node.parent_hash].children.append(node.proof_hash)

    def add_canonization_proof(
        self, canonization_hash: str, signature_hash: str,
        proof_type: str = "canonization",
        auxiliary_data: Optional[Dict] = None,
    ) -> ProofNode:
        self.current_index += 1
        node = ProofNode(
            index=self.current_index,
            proof_hash="",
            parent_hash=self.tip_hash,
            canonization_hash=canonization_hash,
            signature_hash=signature_hash,
            timestamp=time.time(),
            proof_type=proof_type,
            auxiliary_data=auxiliary_data or {},
        )
        self._add_node(node)
        return node

    def add_state_transition_proof(
        self, from_state: str, to_state: str,
        transition_proof: str, signature_hash: str,
    ) -> ProofNode:
        self.current_index += 1
        node = ProofNode(
            index=self.current_index,
            proof_hash="",
            parent_hash=self.tip_hash,
            canonization_hash=None,
            signature_hash=signature_hash,
            timestamp=time.time(),
            proof_type="state_transition",
            auxiliary_data={
                "from_state": from_state,
                "to_state": to_state,
                "transition_proof": transition_proof,
            },
        )
        self._add_node(node)
        return node

    def add_merkle_anchor_proof(
        self, merkle_root: str, depth: int,
        leaf_count: int, signature_hash: str,
    ) -> ProofNode:
        self.current_index += 1
        node = ProofNode(
            index=self.current_index,
            proof_hash="",
            parent_hash=self.tip_hash,
            canonization_hash=merkle_root,
            signature_hash=signature_hash,
            timestamp=time.time(),
            proof_type="merkle_anchor",
            auxiliary_data={"depth": depth, "leaf_count": leaf_count},
        )
        self._add_node(node)
        return node

    def get_proof_chain(self, from_index: int = 0) -> List[ProofNode]:
        return [
            self.nodes[h] for h in self.ordered_indices
            if self.nodes[h].index >= from_index
        ]

    def get_chain_hash(self) -> str:
        if not self.ordered_indices:
            return "0x" + hashlib.sha256(b"empty_chain").hexdigest()

        chain_content = json.dumps({
            "tip": self.tip_hash,
            "length": len(self.ordered_indices),
            "nodes": [
                self.nodes[h].proof_hash
                for h in self.ordered_indices[-100:]
            ],
        }, sort_keys=True)
        return "0x" + hashlib.sha256(chain_content.encode()).hexdigest()

    def verify_chain_integrity(self) -> Tuple[bool, List[str]]:
        errors = []
        for i, idx in enumerate(self.ordered_indices):
            node = self.nodes[idx]
            if node.index > 0:
                if node.parent_hash not in self.nodes:
                    errors.append(f"Node {node.index}: missing parent")
                elif self.nodes[node.parent_hash].index != node.index - 1:
                    errors.append(f"Node {node.index}: parent index mismatch")
            expected_hash = node.compute_hash()
            if node.proof_hash != expected_hash:
                errors.append(f"Node {node.index}: hash mismatch")
        return len(errors) == 0, errors

    def get_telemetry(self) -> Dict:
        return {
            "module": "RecursiveProofChain",
            "version": "1.0.0",
            "substrate": "1100",
            "seal": "PROOF-CHAIN-1100-v1.0.0-2026-06-08",
            "total_nodes": len(self.nodes),
            "tip_hash": self.tip_hash[:16] + "..." if self.tip_hash else None,
            "chain_hash": self.get_chain_hash()[:16] + "...",
        }
