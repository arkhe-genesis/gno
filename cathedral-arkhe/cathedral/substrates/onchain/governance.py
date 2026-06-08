"""
Governance Bridge — Substrato 1100 v1.0.0
Governança human-in-the-loop para canonizações.
"""

from __future__ import annotations
import hashlib
import json
import logging
import queue
import time
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Callable, Dict, List, Optional

from cathedral.substrates.onchain.types import (
    CanonizationType, MemoryLakeEntry, SignatureStatus,
)
from cathedral.substrates.onchain.memory_lake import MemoryLake
from cathedral.substrates.onchain.proof_chain import RecursiveProofChain
from cathedral.substrates.onchain.signer import EIP712Signer


class ProposalState(Enum):
    PROPOSED = "proposed"
    PENDING_SIGNATURE = "pending_signature"
    SIGNED = "signed"
    CANONIZED = "canonized"
    REJECTED = "rejected"
    EXPIRED = "expired"


@dataclass
class GovernanceProposal:
    proposal_id: str
    proposal_type: CanonizationType
    proposal_data: Dict[str, Any]
    proposed_at: float
    deadline: float
    state: ProposalState = ProposalState.PROPOSED
    signature: Optional[Dict] = None
    canonization_entry: Optional[MemoryLakeEntry] = None
    proof_node: Optional[Any] = None

    def is_expired(self) -> bool:
        return time.time() > self.deadline


class GovernanceBridge:
    """
    Bridge que permite ao kernel propor novas canonizações
    e aguardar assinatura humana (human-in-the-loop).
    """

    def __init__(
        self,
        signer: EIP712Signer,
        memory_lake: MemoryLake,
        proof_chain: RecursiveProofChain,
        default_deadline_seconds: int = 86400 * 7,
    ):
        self.signer = signer
        self.memory_lake = memory_lake
        self.proof_chain = proof_chain
        self.default_deadline = default_deadline_seconds
        self.proposals: Dict[str, GovernanceProposal] = {}
        self._pending_queue: queue.Queue = queue.Queue()
        self._signature_callback: Optional[Callable] = None

    def set_signature_callback(self, callback: Callable):
        self._signature_callback = callback

    def propose_canonization(
        self, canonization_type: CanonizationType,
        data: Dict, deadline: Optional[int] = None,
    ) -> str:
        proposal_id = "0x" + hashlib.sha256(
            json.dumps({
                "type": canonization_type.name,
                "data": data,
                "timestamp": time.time(),
            }, sort_keys=True).encode()
        ).hexdigest()

        proposal = GovernanceProposal(
            proposal_id=proposal_id,
            proposal_type=canonization_type,
            proposal_data=data,
            proposed_at=time.time(),
            deadline=time.time() + (deadline or self.default_deadline),
            state=ProposalState.PENDING_SIGNATURE,
        )

        self.proposals[proposal_id] = proposal
        self._pending_queue.put(proposal_id)

        logging.info(
            f"[GovernanceBridge] Proposal {proposal_id[:16]}... created | "
            f"Type: {canonization_type.name} | "
            f"Deadline: {datetime.fromtimestamp(proposal.deadline).isoformat()}"
        )

        if self._signature_callback:
            try:
                self._signature_callback(proposal)
            except Exception as e:
                logging.error(f"[GovernanceBridge] Callback error: {e}")

        return proposal_id

    def submit_signature(self, proposal_id: str, signature: str) -> bool:
        proposal = self.proposals.get(proposal_id)
        if not proposal:
            logging.error(f"[GovernanceBridge] Unknown proposal: {proposal_id}")
            return False

        if proposal.state != ProposalState.PENDING_SIGNATURE:
            logging.error(f"[GovernanceBridge] Proposal not pending: {proposal.state}")
            return False

        if proposal.is_expired():
            proposal.state = ProposalState.EXPIRED
            logging.warning(f"[GovernanceBridge] Proposal expired: {proposal_id[:16]}...")
            return False

        proposal.signature = {"raw": signature, "timestamp": time.time()}
        proposal.state = ProposalState.SIGNED
        logging.info(f"[GovernanceBridge] Signature received for {proposal_id[:16]}...")

        return self._canonize_proposal(proposal)

    def sign_and_canonize_locally(self, proposal_id: str) -> bool:
        """Assina proposta localmente (para governança automatizada). CUIDADO."""
        proposal = self.proposals.get(proposal_id)
        if not proposal:
            return False

        type_name = proposal.proposal_type.name
        if type_name not in EIP712_TYPES:
            type_name = "ArchitecturalDecision"

        sig_result = self.signer._sign_typed(type_name, proposal.proposal_data)

        if sig_result:
            proposal.signature = sig_result
            proposal.state = ProposalState.SIGNED
            return self._canonize_proposal(proposal)
        return False

    def _canonize_proposal(self, proposal: GovernanceProposal) -> bool:
        entry = MemoryLakeEntry(
            entry_hash="",
            entry_type=proposal.proposal_type,
            data=proposal.proposal_data,
            signature=proposal.signature.get("signature") if proposal.signature else None,
            signer=proposal.signature.get("signer") if proposal.signature else None,
            timestamp=time.time(),
            status=SignatureStatus.VERIFIED_CANONICAL,
        )

        self.memory_lake.ingest(entry)
        proposal.canonization_entry = entry

        proof_node = self.proof_chain.add_canonization_proof(
            canonization_hash=entry.entry_hash,
            signature_hash=proposal.signature.get("message_hash") if proposal.signature else None,
            proof_type=f"canonization_{proposal.proposal_type.name}",
            auxiliary_data={"proposal_id": proposal.proposal_id},
        )
        proposal.proof_node = proof_node

        proposal.state = ProposalState.CANONIZED
        logging.info(f"[GovernanceBridge] Canonized: {proposal.proposal_id[:16]}...")
        return True

    def get_pending_proposals(self) -> List[GovernanceProposal]:
        return [
            p for p in self.proposals.values()
            if p.state == ProposalState.PENDING_SIGNATURE and not p.is_expired()
        ]

    def get_proposal_status(self, proposal_id: str) -> Optional[Dict]:
        proposal = self.proposals.get(proposal_id)
        if not proposal:
            return None
        return {
            "proposal_id": proposal.proposal_id,
            "type": proposal.proposal_type.name,
            "state": proposal.state.value,
            "proposed_at": proposal.proposed_at,
            "deadline": proposal.deadline,
            "expired": proposal.is_expired(),
            "has_signature": proposal.signature is not None,
            "canonized": proposal.state == ProposalState.CANONIZED,
            "lake_entry_hash": proposal.canonization_entry.entry_hash if proposal.canonization_entry else None,
            "proof_node_index": proposal.proof_node.index if proposal.proof_node else None,
        }

    def await_signature(self, proposal_id: str, timeout: float = None) -> bool:
        start = time.time()
        while True:
            proposal = self.proposals.get(proposal_id)
            if not proposal:
                return False
            if proposal.state in (ProposalState.SIGNED, ProposalState.CANONIZED):
                return True
            if proposal.is_expired():
                return False
            if timeout and (time.time() - start) > timeout:
                return False
            time.sleep(0.1)

    def get_telemetry(self) -> Dict:
        from collections import Counter
        states = [p.state.value for p in self.proposals.values()]
        return {
            "module": "GovernanceBridge",
            "version": "1.0.0",
            "substrate": "1100",
            "seal": "GOVERNANCE-BRIDGE-1100-v1.0.0-2026-06-08",
            "total_proposals": len(self.proposals),
            "pending": len(self.get_pending_proposals()),
            "state_distribution": dict(Counter(states)) if states else {},
        }
