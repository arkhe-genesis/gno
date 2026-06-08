"""
OnChainCanonizer Types — Substrato 1100 v1.0.0
Tipos base para canonização on-chain com EIP-712.
"""

from __future__ import annotations
import hashlib
import json
import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Set
from enum import IntEnum


class CanonizationType(IntEnum):
    """Tipos de artefatos canonizáveis."""
    KERNEL_INTEGRITY = 1
    META_ORCHESTRATOR_POLICY = 2
    THEOSIS_RL_REWARD_FUNCTION = 3
    STATE_TRANSITION = 4
    ARCHITECTURAL_DECISION = 5
    GOVERNANCE_PROPOSAL = 6
    PROOF_ANCHOR = 7
    MEMORY_LAKE_SNAPSHOT = 8
    GARAK_SCAN_RESULT = 9        # NOVO: integração com GarakBridge1099
    KLEROS_VERDICT = 10        # NOVO: integração com KlerosTrigger1085


class SignatureStatus(IntEnum):
    PENDING_VERIFICATION = 0
    VERIFIED_CANONICAL = 1
    VERIFIED_NON_CANONICAL = 2
    INVALID_SIGNATURE = 3
    REVOKED = 4
    EXPIRED = 5


class ChainId(IntEnum):
    MAINNET = 1
    GOERLI = 5
    SEPOLIA = 11155111
    ARBITRUM_ONE = 42161
    OPTIMISM = 10
    LOCAL = 31337
    RBB = 12120014             # Rede Blockchain Brasil


@dataclass
class EIP712Domain:
    """Separador de domínio EIP-712 para Cathedral ARKHE."""
    name: str = "CathedralArkhe"
    version: str = "6.0.0"
    chainId: int = ChainId.MAINNET
    verifyingContract: str = "0xbF7Da1f568684889A69A5BED9F1311F703985590"

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "version": self.version,
            "chainId": self.chainId,
            "verifyingContract": self.verifyingContract,
        }


@dataclass
class MemoryLakeEntry:
    """Entrada canônica no MemoryLake."""
    entry_hash: str
    entry_type: CanonizationType
    data: Dict[str, Any]
    signature: Optional[str] = None
    signer: Optional[str] = None
    block_number: Optional[int] = None
    timestamp: float = field(default_factory=time.time)
    status: SignatureStatus = SignatureStatus.PENDING_VERIFICATION
    proof_chain_hash: Optional[str] = None
    merkle_index: Optional[int] = None

    def compute_hash(self) -> str:
        """Hash determinístico para esta entrada."""
        content = json.dumps({
            "entry_type": self.entry_type.name,
            "data": self.data,
            "timestamp": self.timestamp,
            "signer": self.signer,
        }, sort_keys=True, default=str)
        return "0x" + hashlib.sha256(content.encode()).hexdigest()


@dataclass
class EtherscanSignature:
    """Assinatura verificada parseada do Etherscan."""
    signature: str
    message_hash: str
    signer: str
    raw_message: Optional[str] = None
    block_number: Optional[int] = None
    timestamp: Optional[int] = None
    tx_hash: Optional[str] = None
    parsed_type: Optional[str] = None
    parsed_data: Optional[Dict[str, Any]] = None


@dataclass
class ProofNode:
    """Nó na cadeia de provas recursivas."""
    index: int
    proof_hash: str
    parent_hash: Optional[str]
    canonization_hash: Optional[str]
    signature_hash: Optional[str]
    timestamp: float
    proof_type: str
    auxiliary_data: Dict[str, Any] = field(default_factory=dict)
    children: List[str] = field(default_factory=list)

    def compute_hash(self) -> str:
        content = json.dumps({
            "index": self.index,
            "parent_hash": self.parent_hash,
            "canonization_hash": self.canonization_hash,
            "signature_hash": self.signature_hash,
            "timestamp": self.timestamp,
            "proof_type": self.proof_type,
            "auxiliary_data": self.auxiliary_data,
        }, sort_keys=True, default=str)
        return "0x" + hashlib.sha256(content.encode()).hexdigest()
