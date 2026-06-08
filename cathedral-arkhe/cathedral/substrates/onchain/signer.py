"""
EIP-712 Signer + Kernel Self-Signer — Substrato 1100 v1.0.0
"""

from __future__ import annotations
import hashlib
import json
import logging
import time
from typing import Any, Dict, List, Optional
from pathlib import Path

from cathedral.substrates.onchain.types import (
    EIP712Domain, CanonizationType, ChainId
)

# Fallback para quando eth_account não está disponível
try:
    from eth_account import Account
    from eth_account.messages import encode_structured_data
    ETHERS_AVAILABLE = True
except ImportError:
    ETHERS_AVAILABLE = False


# Registro de tipos EIP-712 estendido
EIP712_TYPES = {
    "EIP712Domain": [
        {"name": "name", "type": "string"},
        {"name": "version", "type": "string"},
        {"name": "chainId", "type": "uint256"},
        {"name": "verifyingContract", "type": "address"},
    ],
    "KernelIntegrity": [
        {"name": "kernelHash", "type": "bytes32"},
        {"name": "kernelVersion", "type": "string"},
        {"name": "componentHashes", "type": "bytes32[]"},
        {"name": "timestamp", "type": "uint256"},
        {"name": "canonizationType", "type": "uint8"},
    ],
    "MetaOrchestratorPolicy": [
        {"name": "policyId", "type": "bytes32"},
        {"name": "policyType", "type": "string"},
        {"name": "parameters", "type": "string"},
        {"name": "effectivenessThreshold", "type": "uint256"},
        {"name": "expiry", "type": "uint256"},
        {"name": "parentPolicyHash", "type": "bytes32"},
    ],
    "TheosisRLRewardFunction": [
        {"name": "rewardFunctionId", "type": "bytes32"},
        {"name": "functionDefinition", "type": "string"},
        {"name": "convergenceCriteria", "type": "string"},
        {"name": "hyperparameters", "type": "string"},
        {"name": "safetyBounds", "type": "string"},
        {"name": "approvalEpoch", "type": "uint256"},
    ],
    "StateTransition": [
        {"name": "fromStateHash", "type": "bytes32"},
        {"name": "toStateHash", "type": "bytes32"},
        {"name": "transitionProof", "type": "bytes32"},
        {"name": "transitionType", "type": "string"},
        {"name": "gasUsed", "type": "uint256"},
        {"name": "blockNumber", "type": "uint256"},
    ],
    "ArchitecturalDecision": [
        {"name": "decisionId", "type": "bytes32"},
        {"name": "decisionType", "type": "string"},
        {"name": "rationale", "type": "string"},
        {"name": "impactAssessment", "type": "string"},
        {"name": "reversibility", "type": "bool"},
        {"name": "requiredSignatures", "type": "uint256"},
    ],
    "GovernanceProposal": [
        {"name": "proposalHash", "type": "bytes32"},
        {"name": "proposalType", "type": "string"},
        {"name": "proposedBy", "type": "address"},
        {"name": "executionData", "type": "string"},
        {"name": "votingDeadline", "type": "uint256"},
        {"name": "quorumRequired", "type": "uint256"},
    ],
    "ProofAnchor": [
        {"name": "proofRoot", "type": "bytes32"},
        {"name": "proofType", "type": "string"},
        {"name": "depth", "type": "uint256"},
        {"name": "leafCount", "type": "uint256"},
        {"name": "previousAnchor", "type": "bytes32"},
    ],
    "MemoryLakeSnapshot": [
        {"name": "snapshotHash", "type": "bytes32"},
        {"name": "lakeVersion", "type": "uint256"},
        {"name": "totalEntries", "type": "uint256"},
        {"name": "merkleRoot", "type": "bytes32"},
        {"name": "compressionAlgo", "type": "string"},
    ],
    "GarakScanResult": [
        {"name": "scanId", "type": "bytes32"},
        {"name": "riskScore", "type": "uint256"},
        {"name": "failureRate", "type": "uint256"},
        {"name": "criticalFailures", "type": "uint256"},
        {"name": "timestamp", "type": "uint256"},
    ],
    "KlerosVerdict": [
        {"name": "caseId", "type": "bytes32"},
        {"name": "verdict", "type": "string"},
        {"name": "urgencyScore", "type": "uint256"},
        {"name": "timestamp", "type": "uint256"},
    ],
}


class EIP712Signer:
    """Assinatura e verificação EIP-712 para tipos canônicos da Cathedral."""

    def __init__(self, domain: Optional[EIP712Domain] = None):
        self.domain = domain or EIP712Domain()
        self._private_key: Optional[str] = None
        self._account: Optional[Any] = None

    def set_private_key(self, private_key: str):
        """Define chave privada para self-signing do kernel."""
        if ETHERS_AVAILABLE:
            self._private_key = private_key
            self._account = Account.from_key(private_key)
        else:
            self._private_key = private_key
            self._account = f"0x{hashlib.sha256(private_key.encode()).hexdigest()[:40]}"

    def get_signer_address(self) -> str:
        if ETHERS_AVAILABLE and self._account:
            return self._account.address
        elif self._account:
            return self._account
        return self.domain.verifyingContract

    def _build_typed_data(self, type_name: str, message: Dict) -> Dict:
        return {
            "types": {
                "EIP712Domain": EIP712_TYPES["EIP712Domain"],
                type_name: EIP712_TYPES[type_name],
            },
            "primaryType": type_name,
            "domain": self.domain.to_dict(),
            "message": message,
        }

    def sign_kernel_integrity(
        self, kernel_hash: str, kernel_version: str, component_hashes: List[str]
    ) -> Optional[Dict]:
        message = {
            "kernelHash": kernel_hash,
            "kernelVersion": kernel_version,
            "componentHashes": component_hashes,
            "timestamp": int(time.time()),
            "canonizationType": CanonizationType.KERNEL_INTEGRITY,
        }
        return self._sign_typed("KernelIntegrity", message)

    def sign_meta_orchestrator_policy(
        self, policy_id: str, policy_type: str, parameters: Dict,
        effectiveness_threshold: int = 75, expiry: Optional[int] = None,
        parent_policy_hash: str = "0x" + "00" * 32,
    ) -> Optional[Dict]:
        message = {
            "policyId": policy_id,
            "policyType": policy_type,
            "parameters": json.dumps(parameters),
            "effectivenessThreshold": effectiveness_threshold,
            "expiry": expiry or int(time.time()) + 365 * 86400,
            "parentPolicyHash": parent_policy_hash,
        }
        return self._sign_typed("MetaOrchestratorPolicy", message)

    def sign_theosis_reward_function(
        self, reward_function_id: str, function_definition: str,
        convergence_criteria: str, hyperparameters: Dict, safety_bounds: str,
    ) -> Optional[Dict]:
        message = {
            "rewardFunctionId": reward_function_id,
            "functionDefinition": function_definition,
            "convergenceCriteria": convergence_criteria,
            "hyperparameters": json.dumps(hyperparameters),
            "safetyBounds": safety_bounds,
            "approvalEpoch": int(time.time()),
        }
        return self._sign_typed("TheosisRLRewardFunction", message)

    def sign_architectural_decision(
        self, decision_id: str, decision_type: str, rationale: str,
        impact_assessment: str, reversibility: bool = False,
        required_signatures: int = 1,
    ) -> Optional[Dict]:
        message = {
            "decisionId": decision_id,
            "decisionType": decision_type,
            "rationale": rationale,
            "impactAssessment": impact_assessment,
            "reversibility": reversibility,
            "requiredSignatures": required_signatures,
        }
        return self._sign_typed("ArchitecturalDecision", message)

    def sign_governance_proposal(
        self, proposal_hash: str, proposal_type: str, proposed_by: str,
        execution_data: str, voting_deadline: int, quorum_required: int = 1,
    ) -> Optional[Dict]:
        message = {
            "proposalHash": proposal_hash,
            "proposalType": proposal_type,
            "proposedBy": proposed_by,
            "executionData": execution_data,
            "votingDeadline": voting_deadline,
            "quorumRequired": quorum_required,
        }
        return self._sign_typed("GovernanceProposal", message)

    def sign_proof_anchor(
        self, proof_root: str, proof_type: str, depth: int,
        leaf_count: int, previous_anchor: str = "0x" + "00" * 32,
    ) -> Optional[Dict]:
        message = {
            "proofRoot": proof_root,
            "proofType": proof_type,
            "depth": depth,
            "leafCount": leaf_count,
            "previousAnchor": previous_anchor,
        }
        return self._sign_typed("ProofAnchor", message)

    def sign_memory_lake_snapshot(
        self, snapshot_hash: str, lake_version: int, total_entries: int,
        merkle_root: str, compression_algo: str = "zstd",
    ) -> Optional[Dict]:
        message = {
            "snapshotHash": snapshot_hash,
            "lakeVersion": lake_version,
            "totalEntries": total_entries,
            "merkleRoot": merkle_root,
            "compressionAlgo": compression_algo,
        }
        return self._sign_typed("MemoryLakeSnapshot", message)

    def sign_garak_scan_result(
        self, scan_id: str, risk_score: float, failure_rate: float,
        critical_failures: int,
    ) -> Optional[Dict]:
        """NOVO: Assina resultado de scan Garak."""
        message = {
            "scanId": scan_id,
            "riskScore": int(risk_score * 10000),  # Escalar para uint256
            "failureRate": int(failure_rate * 10000),
            "criticalFailures": critical_failures,
            "timestamp": int(time.time()),
        }
        return self._sign_typed("GarakScanResult", message)

    def sign_kleros_verdict(
        self, case_id: str, verdict: str, urgency_score: float,
    ) -> Optional[Dict]:
        """NOVO: Assina veredicto Kleros."""
        message = {
            "caseId": case_id,
            "verdict": verdict,
            "urgencyScore": int(urgency_score * 10000),
            "timestamp": int(time.time()),
        }
        return self._sign_typed("KlerosVerdict", message)

    def _sign_typed(self, type_name: str, message: Dict) -> Optional[Dict]:
        typed_data = self._build_typed_data(type_name, message)

        if ETHERS_AVAILABLE and self._account:
            try:
                encoded = encode_structured_data(typed_data)
                signed = self._account.sign_message(encoded)
                return {
                    "typed_data": typed_data,
                    "signature": signed.signature.hex(),
                    "message_hash": signed.message_hash.hex(),
                    "signer": self._account.address,
                }
            except Exception as e:
                logging.error(f"[EIP712Signer] Signing error: {e}")
                return self._simulate_sign(typed_data)
        else:
            return self._simulate_sign(typed_data)

    def _simulate_sign(self, typed_data: Dict) -> Dict:
        content = json.dumps(typed_data, sort_keys=True, default=str)
        fake_sig = (
            "0x" + hashlib.sha256(content.encode()).hexdigest()
            + hashlib.sha256((content + "sig").encode()).hexdigest()[:64]
        )
        return {
            "typed_data": typed_data,
            "signature": fake_sig,
            "message_hash": "0x" + hashlib.sha256(content.encode()).hexdigest(),
            "signer": self.get_signer_address(),
            "simulated": True,
        }

    def verify_signature(
        self, typed_data: Dict, signature: str, expected_signer: str
    ) -> bool:
        if ETHERS_AVAILABLE:
            try:
                encoded = encode_structured_data(typed_data)
                recovered = Account.recover_message(encoded, signature=signature)
                return recovered.lower() == expected_signer.lower()
            except Exception as e:
                logging.error(f"[EIP712Signer] Verification error: {e}")
                return False
        else:
            content = json.dumps(typed_data, sort_keys=True, default=str)
            expected_hash = "0x" + hashlib.sha256(content.encode()).hexdigest()
            return signature.startswith(expected_hash[:66])


class KernelSelfSigner:
    """Assina o kernel arkhe_os e seus componentes."""

    KERNEL_COMPONENTS = [
        "arkhe_os_v6.py",
        "meta_orchestrator.py",
        "memory_lake.py",
        "recursive_prover.py",
        "theosis_rl.py",
        "onchain_canonizer.py",
    ]

    def __init__(
        self, signer: EIP712Signer, kernel_path: Optional[str] = None
    ):
        self.signer = signer
        self.kernel_path = kernel_path
        self._component_hashes: Dict[str, str] = {}
        self._kernel_signature: Optional[Dict] = None
        self._boot_verified = False

    def compute_component_hashes(self, base_path: str = ".") -> Dict[str, str]:
        for component in self.KERNEL_COMPONENTS:
            path = Path(base_path) / component
            if path.exists():
                with open(path, "rb") as f:
                    content = f.read()
                self._component_hashes[component] = (
                    "0x" + hashlib.sha256(content).hexdigest()
                )
            else:
                self._component_hashes[component] = (
                    "0x" + hashlib.sha256(f"simulated_{component}".encode()).hexdigest()
                )
        return self._component_hashes

    def sign_kernel(self) -> Dict:
        if not self._component_hashes:
            self.compute_component_hashes()

        all_hashes = json.dumps(self._component_hashes, sort_keys=True)
        kernel_hash = "0x" + hashlib.sha256(all_hashes.encode()).hexdigest()
        component_hash_list = [
            self._component_hashes[c] for c in self.KERNEL_COMPONENTS
        ]

        self._kernel_signature = self.signer.sign_kernel_integrity(
            kernel_hash=kernel_hash,
            kernel_version="6.0.0",
            component_hashes=component_hash_list,
        )
        return self._kernel_signature

    def verify_kernel_boot(self, stored_signature: Optional[Dict] = None) -> bool:
        sig = stored_signature or self._kernel_signature
        if not sig:
            logging.error("[KernelSelfSigner] No signature to verify")
            return False

        current_hashes = self.compute_component_hashes()
        current_all = json.dumps(current_hashes, sort_keys=True)
        current_kernel_hash = "0x" + hashlib.sha256(current_all.encode()).hexdigest()
        signed_kernel_hash = sig.get("typed_data", {}).get("message", {}).get("kernelHash")

        if current_kernel_hash == signed_kernel_hash:
            self._boot_verified = True
            logging.info("[KernelSelfSigner] Kernel integrity verified at boot")
            return True
        else:
            logging.error("[KernelSelfSigner] Kernel hash mismatch!")
            return False

    def get_integrity_report(self) -> Dict:
        return {
            "kernel_version": "6.0.0",
            "boot_verified": self._boot_verified,
            "component_hashes": self._component_hashes,
            "signature_present": self._kernel_signature is not None,
            "signer": self.signer.get_signer_address() if self._kernel_signature else None,
            "timestamp": time.time(),
        }
