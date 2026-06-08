"""
OnChainCanonizer — Substrato 1100 v1.0.0
Substrato principal que orquestra canonização on-chain:
  • Pull de assinaturas do Etherscan
  • Ingestão no MemoryLake
  • Integração com RecursiveProofChain
  • GovernanceBridge (human-in-the-loop)
  • Kernel self-signing
"""

from __future__ import annotations
import asyncio
import hashlib
import json
import logging
import time
from typing import Any, Dict, List, Optional

from cathedral.substrates.onchain.types import (
    CanonizationType, ChainId, EIP712Domain, MemoryLakeEntry,
    SignatureStatus,
)
from cathedral.substrates.onchain.memory_lake import MemoryLake
from cathedral.substrates.onchain.proof_chain import RecursiveProofChain, ProofNode
from cathedral.substrates.onchain.signer import EIP712Signer, KernelSelfSigner, EIP712_TYPES
from cathedral.substrates.onchain.etherscan import EtherscanFetcher
from cathedral.substrates.onchain.governance import GovernanceBridge


class OnChainCanonizer:
    """
    Substrato principal de canonização on-chain.
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        private_key: Optional[str] = None,
        chain_id: ChainId = ChainId.MAINNET,
    ):
        self.domain = EIP712Domain(chainId=chain_id)
        self.signer = EIP712Signer(self.domain)
        self.memory_lake = MemoryLake()
        self.proof_chain = RecursiveProofChain()
        self.fetcher = EtherscanFetcher(api_key, chain_id)
        self.kernel_signer = KernelSelfSigner(self.signer)
        self.governance = GovernanceBridge(
            self.signer, self.memory_lake, self.proof_chain
        )

        if private_key:
            self.signer.set_private_key(private_key)

        self._sync_running = False
        self._last_sync_block = 0
        self._initialized = False

    async def initialize(self) -> bool:
        logging.info("[OnChainCanonizer] Initializing substrate...")

        # 1. Self-sign kernel
        logging.info("[OnChainCanonizer] Signing kernel...")
        kernel_sig = self.kernel_signer.sign_kernel()
        if kernel_sig:
            entry = MemoryLakeEntry(
                entry_hash="",
                entry_type=CanonizationType.KERNEL_INTEGRITY,
                data=kernel_sig.get("typed_data", {}).get("message", {}),
                signature=kernel_sig.get("signature"),
                signer=kernel_sig.get("signer"),
                status=SignatureStatus.VERIFIED_CANONICAL,
            )
            self.memory_lake.ingest(entry)
            self.proof_chain.add_canonization_proof(
                canonization_hash=entry.entry_hash,
                signature_hash=kernel_sig.get("message_hash"),
                proof_type="kernel_boot",
            )
            logging.info("[OnChainCanonizer] Kernel signed and canonized")

        # 2. Sync from Etherscan
        logging.info("[OnChainCanonizer] Syncing signatures from Etherscan...")
        await self.sync_signatures()

        # 3. Verify chain integrity
        valid, errors = self.proof_chain.verify_chain_integrity()
        if not valid:
            logging.warning(f"[OnChainCanonizer] Chain integrity issues: {errors}")

        self._initialized = True
        logging.info(
            f"[OnChainCanonizer] Initialized with {len(self.memory_lake.entries)} entries"
        )
        return True

    async def sync_signatures(
        self, start_block: int = 0, end_block: int = 99_999_999,
    ) -> int:
        signatures = await self.fetcher.fetch_verified_signatures(
            start_block=max(start_block, self._last_sync_block),
            end_block=end_block,
        )

        new_count = 0
        for sig in signatures:
            canon_type = self._map_signature_type(sig.parsed_type)

            entry = MemoryLakeEntry(
                entry_hash="",
                entry_type=canon_type,
                data=sig.parsed_data or {"raw_hash": sig.message_hash},
                signature=sig.signature,
                signer=sig.signer,
                block_number=sig.block_number,
                timestamp=sig.timestamp or time.time(),
                status=SignatureStatus.PENDING_VERIFICATION,
            )

            if self.memory_lake.ingest(entry):
                self.proof_chain.add_canonization_proof(
                    canonization_hash=entry.entry_hash,
                    signature_hash=sig.message_hash,
                    proof_type=f"etherscan_{sig.parsed_type or 'unknown'}",
                    auxiliary_data={
                        "block_number": sig.block_number,
                        "tx_hash": sig.tx_hash,
                    },
                )

                if sig.parsed_data and sig.parsed_type in EIP712_TYPES:
                    entry.status = SignatureStatus.VERIFIED_CANONICAL
                else:
                    entry.status = SignatureStatus.VERIFIED_NON_CANONICAL

                new_count += 1
                if sig.block_number:
                    self._last_sync_block = max(
                        self._last_sync_block, sig.block_number
                    )

        logging.info(f"[OnChainCanonizer] Synced {new_count} new signatures")
        return new_count

    def _map_signature_type(self, parsed_type: Optional[str]) -> CanonizationType:
        mapping = {
            "KernelIntegrity": CanonizationType.KERNEL_INTEGRITY,
            "MetaOrchestratorPolicy": CanonizationType.META_ORCHESTRATOR_POLICY,
            "TheosisRLRewardFunction": CanonizationType.THEOSIS_RL_REWARD_FUNCTION,
            "StateTransition": CanonizationType.STATE_TRANSITION,
            "ArchitecturalDecision": CanonizationType.ARCHITECTURAL_DECISION,
            "GovernanceProposal": CanonizationType.GOVERNANCE_PROPOSAL,
            "ProofAnchor": CanonizationType.PROOF_ANCHOR,
            "MemoryLakeSnapshot": CanonizationType.MEMORY_LAKE_SNAPSHOT,
            "EIP1271": CanonizationType.STATE_TRANSITION,
            "EIP712_IN_CALLDATA": CanonizationType.ARCHITECTURAL_DECISION,
        }
        return mapping.get(parsed_type, CanonizationType.ARCHITECTURAL_DECISION)

    async def continuous_sync(self, interval_seconds: int = 60):
        self._sync_running = True
        while self._sync_running:
            try:
                await self.sync_signatures()
            except Exception as e:
                logging.error(f"[OnChainCanonizer] Sync error: {e}")
            await asyncio.sleep(interval_seconds)

    def stop_sync(self):
        self._sync_running = False

    # ─── Métodos de conveniência ───

    def propose_meta_orchestrator_policy(
        self, policy_type: str, parameters: Dict,
        effectiveness_threshold: int = 75,
    ) -> str:
        policy_id = "0x" + hashlib.sha256(
            json.dumps(
                {"policy_type": policy_type, "params": parameters},
                sort_keys=True,
            ).encode()
        ).hexdigest()

        return self.governance.propose_canonization(
            CanonizationType.META_ORCHESTRATOR_POLICY,
            {
                "policyId": policy_id,
                "policyType": policy_type,
                "parameters": json.dumps(parameters),
                "effectivenessThreshold": effectiveness_threshold,
                "expiry": int(time.time()) + 365 * 86400,
                "parentPolicyHash": "0x" + "00" * 32,
            },
        )

    def propose_theosis_reward_function(
        self, function_definition: str, convergence_criteria: str,
        hyperparameters: Dict, safety_bounds: str,
    ) -> str:
        reward_id = "0x" + hashlib.sha256(function_definition.encode()).hexdigest()
        return self.governance.propose_canonization(
            CanonizationType.THEOSIS_RL_REWARD_FUNCTION,
            {
                "rewardFunctionId": reward_id,
                "functionDefinition": function_definition,
                "convergenceCriteria": convergence_criteria,
                "hyperparameters": json.dumps(hyperparameters),
                "safetyBounds": safety_bounds,
                "approvalEpoch": int(time.time()),
            },
        )

    def propose_architectural_decision(
        self, decision_type: str, rationale: str,
        impact_assessment: str, reversibility: bool = False,
    ) -> str:
        decision_id = "0x" + hashlib.sha256(
            json.dumps({
                "type": decision_type,
                "rationale": rationale,
                "ts": time.time(),
            }, sort_keys=True).encode()
        ).hexdigest()

        return self.governance.propose_canonization(
            CanonizationType.ARCHITECTURAL_DECISION,
            {
                "decisionId": decision_id,
                "decisionType": decision_type,
                "rationale": rationale,
                "impactAssessment": impact_assessment,
                "reversibility": reversibility,
                "requiredSignatures": 1,
            },
        )

    def canonize_garak_scan(self, garak_report: Dict) -> Optional[ProofNode]:
        """NOVO: Canoniza resultado de scan Garak diretamente."""
        scan_id = garak_report.get("scan_id", f"GARAK-{int(time.time())}")
        risk_score = garak_report.get("risk_score", 0.0)
        failure_rate = garak_report.get("failure_rate", 0.0)
        critical = garak_report.get("critical_failures", 0)

        sig = self.signer.sign_garak_scan_result(
            scan_id=scan_id,
            risk_score=risk_score,
            failure_rate=failure_rate,
            critical_failures=critical,
        )

        if sig:
            entry = MemoryLakeEntry(
                entry_hash="",
                entry_type=CanonizationType.GARAK_SCAN_RESULT,
                data=garak_report,
                signature=sig.get("signature"),
                signer=sig.get("signer"),
                status=SignatureStatus.VERIFIED_CANONICAL,
            )
            self.memory_lake.ingest(entry)
            return self.proof_chain.add_canonization_proof(
                canonization_hash=entry.entry_hash,
                signature_hash=sig.get("message_hash"),
                proof_type="garak_scan",
                auxiliary_data={"scan_id": scan_id, "risk_score": risk_score},
            )
        return None

    def canonize_kleros_verdict(self, kleros_case: Any) -> Optional[ProofNode]:
        """NOVO: Canoniza veredicto Kleros."""
        case_id = getattr(kleros_case, "case_id", str(kleros_case))
        verdict = getattr(kleros_case, "verdict", "UNKNOWN")
        urgency = getattr(kleros_case, "evidence", {}).get("urgency_score", 0.0)

        sig = self.signer.sign_kleros_verdict(
            case_id=case_id, verdict=verdict, urgency_score=urgency,
        )

        if sig:
            entry = MemoryLakeEntry(
                entry_hash="",
                entry_type=CanonizationType.KLEROS_VERDICT,
                data={"case_id": case_id, "verdict": verdict, "urgency": urgency},
                signature=sig.get("signature"),
                signer=sig.get("signer"),
                status=SignatureStatus.VERIFIED_CANONICAL,
            )
            self.memory_lake.ingest(entry)
            return self.proof_chain.add_canonization_proof(
                canonization_hash=entry.entry_hash,
                signature_hash=sig.get("message_hash"),
                proof_type="kleros_verdict",
                auxiliary_data={"case_id": case_id},
            )
        return None

    def anchor_merkle_root(self) -> Optional[ProofNode]:
        merkle_root = self.memory_lake.get_merkle_root()
        tree = self.memory_lake.build_merkle_tree()

        sig = self.signer.sign_proof_anchor(
            proof_root=merkle_root,
            proof_type="memory_lake_merkle",
            depth=len(tree),
            leaf_count=len(tree[0]),
            previous_anchor=self.proof_chain.tip_hash or "0x" + "00" * 32,
        )

        if sig:
            return self.proof_chain.add_merkle_anchor_proof(
                merkle_root=merkle_root,
                depth=len(tree),
                leaf_count=len(tree[0]),
                signature_hash=sig.get("message_hash"),
            )
        return None

    # ─── Verificação e queries ───

    def verify_boot_integrity(self) -> bool:
        return self.kernel_signer.verify_kernel_boot()

    def get_canonical_state(self) -> Dict:
        return {
            "memory_lake": {
                "merkle_root": self.memory_lake.get_merkle_root(),
                "total_entries": len(self.memory_lake.entries),
                "type_counts": {
                    t.name: len(self.memory_lake._type_index[t])
                    for t in CanonizationType
                },
                "recent": [
                    {
                        "hash": e.entry_hash[:16] + "...",
                        "type": e.entry_type.name,
                        "signer": e.signer[:16] + "..." if e.signer else None,
                        "status": e.status.name,
                    }
                    for e in self.memory_lake.get_recent(5)
                ],
            },
            "proof_chain": {
                "tip_hash": self.proof_chain.tip_hash,
                "length": len(self.proof_chain.ordered_indices),
                "chain_hash": self.proof_chain.get_chain_hash(),
            },
            "governance": {
                "pending_proposals": len(self.governance.get_pending_proposals()),
                "total_proposals": len(self.governance.proposals),
            },
            "kernel": self.kernel_signer.get_integrity_report(),
            "last_sync_block": self._last_sync_block,
            "initialized": self._initialized,
        }

    def get_canonization_proof(self, entry_hash: str) -> Dict:
        entry = self.memory_lake.entries.get(entry_hash)
        if not entry:
            return {"error": "Entry not found"}

        merkle_proof = self.memory_lake.get_proof(entry_hash)
        return {
            "entry": {
                "hash": entry.entry_hash,
                "type": entry.entry_type.name,
                "signer": entry.signer,
                "timestamp": entry.timestamp,
                "status": entry.status.name,
            },
            "merkle_proof": merkle_proof,
            "merkle_root": self.memory_lake.get_merkle_root(),
            "signature": entry.signature,
        }

    def get_telemetry(self) -> Dict:
        return {
            "module": "OnChainCanonizer",
            "version": "1.0.0",
            "substrate": "1100",
            "seal": "ONCHAIN-CANONIZER-1100-v1.0.0-2026-06-08",
            **self.get_canonical_state(),
        }
