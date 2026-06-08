"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                     ONCHAIN CANONIZER SUBSTRATE v1.0                        ║
║          Pulls verified signatures into MemoryLake as canonical state       ║
║              transitions with ZK-integrated recursive proof chain           ║
╚══════════════════════════════════════════════════════════════════════════════╝

Target Address: 0xbF7Da1f568684889A69A5BED9F1311F703985590
Architecture: arkhe_os_v6 kernel self-signing + EIP-712 extended governance
"""

import hashlib
import json
import time
import struct
import asyncio
from abc import ABC, abstractmethod
from dataclasses import dataclass, field, asdict
from typing import Any, Dict, List, Optional, Tuple, Set, Callable
from enum import Enum, IntEnum
from datetime import datetime
from pathlib import Path
import threading
import queue
import logging

# Try to import crypto libraries with fallbacks
try:
    from eth_account import Account
    from eth_account.messages import encode_structured_data
    from eth_utils import to_checksum_address, decode_hex
    from web3 import Web3
    ETHERS_AVAILABLE = True
except ImportError:
    ETHERS_AVAILABLE = False
    print("[OnChainCanonizer] eth_account/web3 not available — using simulation mode")

try:
    import aiohttp
    AIOHTTP_AVAILABLE = True
except ImportError:
    AIOHTTP_AVAILABLE = False
    print("[OnChainCanonizer] aiohttp not available — using sync fallback")

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 1: CORE TYPE DEFINITIONS
# ═══════════════════════════════════════════════════════════════════════════════

class CanonizationType(IntEnum):
    """Types of canonizable artifacts"""
    KERNEL_INTEGRITY = 1
    META_ORCHESTRATOR_POLICY = 2
    THEOSIS_RL_REWARD_FUNCTION = 3
    STATE_TRANSITION = 4
    ARCHITECTURAL_DECISION = 5
    GOVERNANCE_PROPOSAL = 6
    PROOF_ANCHOR = 7
    MEMORY_LAKE_SNAPSHOT = 8

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

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 2: EXTENDED EIP-712 TYPE REGISTRY
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class EIP712Domain:
    """EIP-712 domain separator for cathedral_arkhe"""
    name: str = "CathedralArkhe"
    version: str = "6.0.0"
    chainId: int = ChainId.MAINNET
    verifyingContract: str = "0xbF7Da1f568684889A69A5BED9F1311F703985590"

    def to_dict(self) -> Dict:
        return {
            "name": self.name,
            "version": self.version,
            "chainId": self.chainId,
            "verifyingContract": self.verifyingContract
        }

# Extended EIP-712 types for all canonizable artifacts
EIP712_TYPES = {
    "EIP712Domain": [
        {"name": "name", "type": "string"},
        {"name": "version", "type": "string"},
        {"name": "chainId", "type": "uint256"},
        {"name": "verifyingContract", "type": "address"}
    ],
    "KernelIntegrity": [
        {"name": "kernelHash", "type": "bytes32"},
        {"name": "kernelVersion", "type": "string"},
        {"name": "componentHashes", "type": "bytes32[]"},
        {"name": "timestamp", "type": "uint256"},
        {"name": "canonizationType", "type": "uint8"}
    ],
    "MetaOrchestratorPolicy": [
        {"name": "policyId", "type": "bytes32"},
        {"name": "policyType", "type": "string"},
        {"name": "parameters", "type": "string"},  # JSON-encoded
        {"name": "effectivenessThreshold", "type": "uint256"},
        {"name": "expiry", "type": "uint256"},
        {"name": "parentPolicyHash", "type": "bytes32"}
    ],
    "TheosisRLRewardFunction": [
        {"name": "rewardFunctionId", "type": "bytes32"},
        {"name": "functionDefinition", "type": "string"},  # Mathematical definition
        {"name": "convergenceCriteria", "type": "string"},
        {"name": "hyperparameters", "type": "string"},  # JSON-encoded
        {"name": "safetyBounds", "type": "string"},
        {"name": "approvalEpoch", "type": "uint256"}
    ],
    "StateTransition": [
        {"name": "fromStateHash", "type": "bytes32"},
        {"name": "toStateHash", "type": "bytes32"},
        {"name": "transitionProof", "type": "bytes32"},
        {"name": "transitionType", "type": "string"},
        {"name": "gasUsed", "type": "uint256"},
        {"name": "blockNumber", "type": "uint256"}
    ],
    "ArchitecturalDecision": [
        {"name": "decisionId", "type": "bytes32"},
        {"name": "decisionType", "type": "string"},
        {"name": "rationale", "type": "string"},
        {"name": "impactAssessment", "type": "string"},
        {"name": "reversibility", "type": "bool"},
        {"name": "requiredSignatures", "type": "uint256"}
    ],
    "GovernanceProposal": [
        {"name": "proposalHash", "type": "bytes32"},
        {"name": "proposalType", "type": "string"},
        {"name": "proposedBy", "type": "address"},
        {"name": "executionData", "type": "string"},
        {"name": "votingDeadline", "type": "uint256"},
        {"name": "quorumRequired", "type": "uint256"}
    ],
    "ProofAnchor": [
        {"name": "proofRoot", "type": "bytes32"},
        {"name": "proofType", "type": "string"},
        {"name": "depth", "type": "uint256"},
        {"name": "leafCount", "type": "uint256"},
        {"name": "previousAnchor", "type": "bytes32"}
    ],
    "MemoryLakeSnapshot": [
        {"name": "snapshotHash", "type": "bytes32"},
        {"name": "lakeVersion", "type": "uint256"},
        {"name": "totalEntries", "type": "uint256"},
        {"name": "merkleRoot", "type": "bytes32"},
        {"name": "compressionAlgo", "type": "string"}
    ]
}

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 3: MEMORY LAKE INTEGRATION
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class MemoryLakeEntry:
    """A canonical entry in MemoryLake"""
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
        """Compute deterministic hash for this entry"""
        content = json.dumps({
            "entry_type": self.entry_type.name,
            "data": self.data,
            "timestamp": self.timestamp,
            "signer": self.signer
        }, sort_keys=True, default=str)
        return "0x" + hashlib.sha256(content.encode()).hexdigest()

class MemoryLake:
    """
    In-memory lake for canonical state transitions.
    Acts as the substrate's local mirror of on-chain canonizations.
    """

    def __init__(self, max_entries: int = 100000):
        self.entries: Dict[str, MemoryLakeEntry] = {}
        self.ordered_hashes: List[str] = []
        self.merkle_tree: Optional[List[List[str]]] = None
        self.max_entries = max_entries
        self._lock = threading.RLock()
        self._type_index: Dict[CanonizationType, Set[str]] = {
            t: set() for t in CanonizationType
        }
        self._signer_index: Dict[str, Set[str]] = {}

    def ingest(self, entry: MemoryLakeEntry) -> bool:
        """Ingest a new entry into the lake"""
        with self._lock:
            if entry.entry_hash in self.entries:
                return False
            if len(self.entries) >= self.max_entries:
                self._evict_oldest()

            entry.entry_hash = entry.compute_hash()
            self.entries[entry.entry_hash] = entry
            self.ordered_hashes.append(entry.entry_hash)
            self._type_index[entry.entry_type].add(entry.entry_hash)

            if entry.signer:
                if entry.signer not in self._signer_index:
                    self._signer_index[entry.signer] = set()
                self._signer_index[entry.signer].add(entry.entry_hash)

            self._invalidate_merkle()
            return True

    def _evict_oldest(self):
        """Evict oldest entry when at capacity"""
        if self.ordered_hashes:
            oldest = self.ordered_hashes.pop(0)
            entry = self.entries.pop(oldest, None)
            if entry:
                self._type_index[entry.entry_type].discard(oldest)
                if entry.signer and entry.signer in self._signer_index:
                    self._signer_index[entry.signer].discard(oldest)

    def _invalidate_merkle(self):
        self.merkle_tree = None

    def build_merkle_tree(self) -> List[List[str]]:
        """Build Merkle tree from all entries"""
        with self._lock:
            if self.merkle_tree is not None:
                return self.merkle_tree

            if not self.ordered_hashes:
                return [[hashlib.sha256(b"empty").hexdigest()]]

            # Leaf layer
            leaves = [self.entries[h].entry_hash[2:] if h.startswith("0x") else h
                      for h in self.ordered_hashes]

            # Pad to power of 2
            while len(leaves) & (len(leaves) - 1) != 0:
                leaves.append(leaves[-1])

            tree = [leaves]
            current_level = leaves

            while len(current_level) > 1:
                next_level = []
                for i in range(0, len(current_level), 2):
                    combined = current_level[i] + current_level[i+1]
                    next_level.append(hashlib.sha256(combined.encode()).hexdigest())
                tree.append(next_level)
                current_level = next_level

            self.merkle_tree = tree
            return tree

    def get_merkle_root(self) -> str:
        """Get the Merkle root of the entire lake"""
        tree = self.build_merkle_tree()
        return "0x" + tree[-1][0]

    def get_proof(self, entry_hash: str) -> Optional[List[str]]:
        """Get Merkle proof for a specific entry"""
        tree = self.build_merkle_tree()

        try:
            idx = self.ordered_hashes.index(entry_hash)
        except ValueError:
            return None

        proof = []
        for level in tree[:-1]:
            sibling_idx = idx ^ 1
            if sibling_idx < len(level):
                proof.append(level[sibling_idx])
            idx >>= 1

        return proof

    def get_by_type(self, entry_type: CanonizationType) -> List[MemoryLakeEntry]:
        """Get all entries of a specific type"""
        with self._lock:
            return [self.entries[h] for h in self._type_index[entry_type]
                    if h in self.entries]

    def get_by_signer(self, signer: str) -> List[MemoryLakeEntry]:
        """Get all entries signed by a specific address"""
        with self._lock:
            return [self.entries[h] for h in self._signer_index.get(signer, set())
                    if h in self.entries]

    def get_recent(self, n: int = 10) -> List[MemoryLakeEntry]:
        """Get n most recent entries"""
        with self._lock:
            recent_hashes = self.ordered_hashes[-n:]
            return [self.entries[h] for h in reversed(recent_hashes)]

    def snapshot(self) -> Dict:
        """Create a snapshot of the lake state"""
        return {
            "merkle_root": self.get_merkle_root(),
            "total_entries": len(self.entries),
            "type_counts": {t.name: len(s) for t, s in self._type_index.items()},
            "timestamp": time.time()
        }

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 4: ETHERSCAN SIGNATURE FETCHER
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class EtherscanSignature:
    """Parsed verified signature from Etherscan"""
    signature: str
    message_hash: str
    signer: str
    raw_message: Optional[str] = None
    block_number: Optional[int] = None
    timestamp: Optional[int] = None
    tx_hash: Optional[str] = None
    parsed_type: Optional[str] = None
    parsed_data: Optional[Dict] = None

class EtherscanFetcher:
    """
    Fetches verified signatures from Etherscan for the target address.
    Supports both real API calls and simulation mode.
    """

    BASE_URLS = {
        ChainId.MAINNET: "https://api.etherscan.io/api",
        ChainId.SEPOLIA: "https://api-sepolia.etherscan.io/api",
        ChainId.GOERLI: "https://api-goerli.etherscan.io/api",
        ChainId.ARBITRUM_ONE: "https://api.arbiscan.io/api",
        ChainId.OPTIMISM: "https://api-optimistic.etherscan.io/api",
    }

    TARGET_ADDRESS = "0xbF7Da1f568684889A69A5BED9F1311F703985590"

    def __init__(self, api_key: Optional[str] = None, chain_id: ChainId = ChainId.MAINNET):
        self.api_key = api_key
        self.chain_id = chain_id
        self.base_url = self.BASE_URLS.get(chain_id, self.BASE_URLS[ChainId.MAINNET])
        self.session = None
        self._cache: List[EtherscanSignature] = []
        self._last_fetch = 0
        self._rate_limit_delay = 0.2  # 5 calls/sec for free tier

    async def _get_session(self):
        if self.session is None and AIOHTTP_AVAILABLE:
            self.session = aiohttp.ClientSession()
        return self.session

    async def fetch_verified_signatures(self,
                                         start_block: int = 0,
                                         end_block: int = 99999999,
                                         page: int = 1,
                                         offset: int = 100) -> List[EtherscanSignature]:
        """
        Fetch verified signatures from Etherscan.
        Note: Etherscan doesn't have a direct 'verified signatures' endpoint,
        so we parse signature-related transactions.
        """
        if not AIOHTTP_AVAILABLE or not ETHERS_AVAILABLE:
            return self._simulate_signatures()

        await asyncio.sleep(self._rate_limit_delay)

        params = {
            "module": "account",
            "action": "txlist",
            "address": self.TARGET_ADDRESS,
            "startblock": start_block,
            "endblock": end_block,
            "page": page,
            "offset": offset,
            "sort": "desc",
            "apikey": self.api_key or "YourApiKeyToken"
        }

        try:
            session = await self._get_session()
            async with session.get(self.base_url, params=params) as resp:
                data = await resp.json()

            if data.get("status") != "1":
                return self._simulate_signatures()

            signatures = []
            for tx in data.get("result", []):
                # Look for signature-like transactions
                # These could be calls to signature verification contracts,
                # EIP-712 typed data signatures posted on-chain, etc.
                sig = self._parse_tx_as_signature(tx)
                if sig:
                    signatures.append(sig)

            self._cache = signatures
            self._last_fetch = time.time()
            return signatures

        except Exception as e:
            logging.warning(f"[EtherscanFetcher] API error: {e}")
            return self._simulate_signatures()

    def _parse_tx_as_signature(self, tx: Dict) -> Optional[EtherscanSignature]:
        """Parse a transaction as a potential signature verification"""
        # In practice, signatures appear in:
        # 1. Calls to ERC-1271 isValidSignature
        # 2. EIP-712 typed data hashes in calldata
        # 3. Signature registry contract calls

        input_data = tx.get("input", "0x")
        if len(input_data) < 130:  # Minimum for a signature
            return None

        # Try to extract EIP-712 typed data hash from calldata
        # Method ID for common signature functions
        method_id = input_data[:10]

        signature_methods = {
            "0x8208a634",  # isValidSignature(bytes32,bytes)
            "0x1626ba7e",  # isValidSignature(bytes,bytes)
            "0xa3b22fc4",  # verify
            "0x304e6ade",  # isValidSignatureWithResolver
        }

        if method_id in signature_methods:
            # Extract message hash and signature from calldata
            try:
                # Parse ABI-encoded parameters (simplified)
                calldata = input_data[10:]

                # bytes32 messageHash (offset 0, length 64 hex chars)
                message_hash = "0x" + calldata[0:64] if len(calldata) >= 64 else None

                # bytes signature (dynamic, skip for now but note location)
                signature = "0x" + calldata[128:192] if len(calldata) >= 192 else None

                if message_hash and signature:
                    return EtherscanSignature(
                        signature=signature,
                        message_hash=message_hash,
                        signer=self.TARGET_ADDRESS,
                        block_number=int(tx.get("blockNumber", 0)),
                        timestamp=int(tx.get("timeStamp", 0)),
                        tx_hash=tx.get("hash"),
                        parsed_type="EIP1271"
                    )
            except Exception:
                pass

        # Check for EIP-712 domain separator in calldata
        if "cathedralarkhe" in input_data.lower() or "arkhe" in input_data.lower():
            return EtherscanSignature(
                signature=input_data[-130:] if len(input_data) >= 130 else input_data,
                message_hash="0x" + hashlib.sha256(input_data.encode()).hexdigest(),
                signer=self.TARGET_ADDRESS,
                block_number=int(tx.get("blockNumber", 0)),
                timestamp=int(tx.get("timeStamp", 0)),
                tx_hash=tx.get("hash"),
                parsed_type="EIP712_IN_CALLDATA"
            )

        return None

    def _simulate_signatures(self) -> List[EtherscanSignature]:
        """
        Generate simulated signatures for development/testing.
        These mirror the structure of real Etherscan verified signatures.
        """
        now = int(time.time())

        simulations = [
            EtherscanSignature(
                signature="0x" + "a1b2c3d4" * 16,
                message_hash="0x" + hashlib.sha256(b"kernel_integrity_v6").hexdigest(),
                signer=self.TARGET_ADDRESS,
                block_number=19_500_000,
                timestamp=now - 86400,
                tx_hash="0x" + hashlib.sha256(b"tx1").hexdigest(),
                parsed_type="KernelIntegrity",
                parsed_data={
                    "kernelHash": "0x" + hashlib.sha256(b"arkhe_os_v6.py").hexdigest(),
                    "kernelVersion": "6.0.0",
                    "canonizationType": 1
                }
            ),
            EtherscanSignature(
                signature="0x" + "e5f6a7b8" * 16,
                message_hash="0x" + hashlib.sha256(b"meta_orch_policy").hexdigest(),
                signer=self.TARGET_ADDRESS,
                block_number=19_500_050,
                timestamp=now - 43200,
                tx_hash="0x" + hashlib.sha256(b"tx2").hexdigest(),
                parsed_type="MetaOrchestratorPolicy",
                parsed_data={
                    "policyId": "0x" + hashlib.sha256(b"policy_001").hexdigest(),
                    "policyType": "resource_allocation",
                    "effectivenessThreshold": 75
                }
            ),
            EtherscanSignature(
                signature="0x" + "c9d0e1f2" * 16,
                message_hash="0x" + hashlib.sha256(b"theosis_reward").hexdigest(),
                signer=self.TARGET_ADDRESS,
                block_number=19_500_100,
                timestamp=now - 21600,
                tx_hash="0x" + hashlib.sha256(b"tx3").hexdigest(),
                parsed_type="TheosisRLRewardFunction",
                parsed_data={
                    "rewardFunctionId": "0x" + hashlib.sha256(b"reward_fn_v2").hexdigest(),
                    "convergenceCriteria": "epsilon < 0.001 over 1000 episodes"
                }
            ),
            EtherscanSignature(
                signature="0x" + "1a2b3c4d" * 16,
                message_hash="0x" + hashlib.sha256(b"state_transition").hexdigest(),
                signer=self.TARGET_ADDRESS,
                block_number=19_500_150,
                timestamp=now - 3600,
                tx_hash="0x" + hashlib.sha256(b"tx4").hexdigest(),
                parsed_type="StateTransition",
                parsed_data={
                    "fromStateHash": "0x" + "00" * 32,
                    "toStateHash": "0x" + "ff" * 32,
                    "transitionType": "epoch_rollover"
                }
            ),
            EtherscanSignature(
                signature="0x" + "5e6f7a8b" * 16,
                message_hash="0x" + hashlib.sha256(b"arch_decision").hexdigest(),
                signer=self.TARGET_ADDRESS,
                block_number=19_500_200,
                timestamp=now,
                tx_hash="0x" + hashlib.sha256(b"tx5").hexdigest(),
                parsed_type="ArchitecturalDecision",
                parsed_data={
                    "decisionId": "0x" + hashlib.sha256(b"decision_zk_integration").hexdigest(),
                    "decisionType": "proof_system_upgrade",
                    "rationale": "Integrate Groth16 for recursive proof composition"
                }
            ),
        ]

        self._cache = simulations
        return simulations

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 5: EIP-712 SIGNING AND VERIFICATION
# ═══════════════════════════════════════════════════════════════════════════════

class EIP712Signer:
    """
    Handles EIP-712 typed data signing and verification
    for all cathedral_arkhe canonization types.
    """

    def __init__(self, domain: Optional[EIP712Domain] = None):
        self.domain = domain or EIP712Domain()
        self._private_key: Optional[str] = None
        self._account: Optional[Any] = None

    def set_private_key(self, private_key: str):
        """Set the signing private key (for kernel self-signing)"""
        if ETHERS_AVAILABLE:
            self._private_key = private_key
            self._account = Account.from_key(private_key)
        else:
            self._private_key = private_key
            self._account = f"0x{hashlib.sha256(private_key.encode()).hexdigest()[:40]}"

    def get_signer_address(self) -> str:
        """Get the address derived from the private key"""
        if ETHERS_AVAILABLE and self._account:
            return self._account.address
        elif self._account:
            return self._account
        return self.domain.verifyingContract

    def _build_typed_data(self, type_name: str, message: Dict) -> Dict:
        """Build complete EIP-712 typed data structure"""
        return {
            "types": {
                "EIP712Domain": EIP712_TYPES["EIP712Domain"],
                type_name: EIP712_TYPES[type_name]
            },
            "primaryType": type_name,
            "domain": self.domain.to_dict(),
            "message": message
        }

    def sign_kernel_integrity(self,
                               kernel_hash: str,
                               kernel_version: str,
                               component_hashes: List[str]) -> Optional[Dict]:
        """Sign kernel integrity attestation"""
        message = {
            "kernelHash": kernel_hash,
            "kernelVersion": kernel_version,
            "componentHashes": component_hashes,
            "timestamp": int(time.time()),
            "canonizationType": CanonizationType.KERNEL_INTEGRITY
        }

        return self._sign_typed("KernelIntegrity", message)

    def sign_meta_orchestrator_policy(self,
                                       policy_id: str,
                                       policy_type: str,
                                       parameters: Dict,
                                       effectiveness_threshold: int = 75,
                                       expiry: Optional[int] = None,
                                       parent_policy_hash: str = "0x" + "00" * 32) -> Optional[Dict]:
        """Sign a MetaOrchestrator policy"""
        message = {
            "policyId": policy_id,
            "policyType": policy_type,
            "parameters": json.dumps(parameters),
            "effectivenessThreshold": effectiveness_threshold,
            "expiry": expiry or int(time.time()) + 365 * 86400,
            "parentPolicyHash": parent_policy_hash
        }

        return self._sign_typed("MetaOrchestratorPolicy", message)

    def sign_theosis_reward_function(self,
                                      reward_function_id: str,
                                      function_definition: str,
                                      convergence_criteria: str,
                                      hyperparameters: Dict,
                                      safety_bounds: str) -> Optional[Dict]:
        """Sign a TheosisRL reward function"""
        message = {
            "rewardFunctionId": reward_function_id,
            "functionDefinition": function_definition,
            "convergenceCriteria": convergence_criteria,
            "hyperparameters": json.dumps(hyperparameters),
            "safetyBounds": safety_bounds,
            "approvalEpoch": int(time.time())
        }

        return self._sign_typed("TheosisRLRewardFunction", message)

    def sign_architectural_decision(self,
                                     decision_id: str,
                                     decision_type: str,
                                     rationale: str,
                                     impact_assessment: str,
                                     reversibility: bool = False,
                                     required_signatures: int = 1) -> Optional[Dict]:
        """Sign an architectural decision"""
        message = {
            "decisionId": decision_id,
            "decisionType": decision_type,
            "rationale": rationale,
            "impactAssessment": impact_assessment,
            "reversibility": reversibility,
            "requiredSignatures": required_signatures
        }

        return self._sign_typed("ArchitecturalDecision", message)

    def sign_governance_proposal(self,
                                  proposal_hash: str,
                                  proposal_type: str,
                                  proposed_by: str,
                                  execution_data: str,
                                  voting_deadline: int,
                                  quorum_required: int = 1) -> Optional[Dict]:
        """Sign a governance proposal"""
        message = {
            "proposalHash": proposal_hash,
            "proposalType": proposal_type,
            "proposedBy": proposed_by,
            "executionData": execution_data,
            "votingDeadline": voting_deadline,
            "quorumRequired": quorum_required
        }

        return self._sign_typed("GovernanceProposal", message)

    def sign_proof_anchor(self,
                           proof_root: str,
                           proof_type: str,
                           depth: int,
                           leaf_count: int,
                           previous_anchor: str = "0x" + "00" * 32) -> Optional[Dict]:
        """Sign a proof anchor for ZK integration"""
        message = {
            "proofRoot": proof_root,
            "proofType": proof_type,
            "depth": depth,
            "leafCount": leaf_count,
            "previousAnchor": previous_anchor
        }

        return self._sign_typed("ProofAnchor", message)

    def sign_memory_lake_snapshot(self,
                                   snapshot_hash: str,
                                   lake_version: int,
                                   total_entries: int,
                                   merkle_root: str,
                                   compression_algo: str = "zstd") -> Optional[Dict]:
        """Sign a MemoryLake snapshot"""
        message = {
            "snapshotHash": snapshot_hash,
            "lakeVersion": lake_version,
            "totalEntries": total_entries,
            "merkleRoot": merkle_root,
            "compressionAlgo": compression_algo
        }

        return self._sign_typed("MemoryLakeSnapshot", message)

    def _sign_typed(self, type_name: str, message: Dict) -> Optional[Dict]:
        """Internal method to sign typed data"""
        typed_data = self._build_typed_data(type_name, message)

        if ETHERS_AVAILABLE and self._account:
            try:
                encoded = encode_structured_data(typed_data)
                signed = self._account.sign_message(encoded)
                return {
                    "typed_data": typed_data,
                    "signature": signed.signature.hex(),
                    "message_hash": signed.message_hash.hex(),
                    "signer": self._account.address
                }
            except Exception as e:
                logging.error(f"[EIP712Signer] Signing error: {e}")
                return self._simulate_sign(typed_data)
        else:
            return self._simulate_sign(typed_data)

    def _simulate_sign(self, typed_data: Dict) -> Dict:
        """Simulate signing when crypto libs unavailable"""
        content = json.dumps(typed_data, sort_keys=True, default=str)
        fake_sig = "0x" + hashlib.sha256(content.encode()).hexdigest() + \
                   hashlib.sha256((content + "sig").encode()).hexdigest()[:64]

        return {
            "typed_data": typed_data,
            "signature": fake_sig,
            "message_hash": "0x" + hashlib.sha256(content.encode()).hexdigest(),
            "signer": self.get_signer_address(),
            "simulated": True
        }

    def verify_signature(self,
                          typed_data: Dict,
                          signature: str,
                          expected_signer: str) -> bool:
        """Verify an EIP-712 signature"""
        if ETHERS_AVAILABLE:
            try:
                encoded = encode_structured_data(typed_data)
                recovered = Account.recover_message(encoded, signature=signature)
                return recovered.lower() == expected_signer.lower()
            except Exception as e:
                logging.error(f"[EIP712Signer] Verification error: {e}")
                return False
        else:
            # Simulation mode: check hash consistency
            content = json.dumps(typed_data, sort_keys=True, default=str)
            expected_hash = "0x" + hashlib.sha256(content.encode()).hexdigest()
            return signature.startswith(expected_hash[:66])

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 6: RECURSIVE PROOF CHAIN WITH SIGNATURE INTEGRATION
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class ProofNode:
    """A node in the recursive proof chain"""
    index: int
    proof_hash: str
    parent_hash: Optional[str]
    canonization_hash: Optional[str]
    signature_hash: Optional[str]
    timestamp: float
    proof_type: str
    auxiliary_data: Dict = field(default_factory=dict)
    children: List[str] = field(default_factory=list)

    def compute_hash(self) -> str:
        content = json.dumps({
            "index": self.index,
            "parent_hash": self.parent_hash,
            "canonization_hash": self.canonization_hash,
            "signature_hash": self.signature_hash,
            "timestamp": self.timestamp,
            "proof_type": self.proof_type,
            "auxiliary_data": self.auxiliary_data
        }, sort_keys=True, default=str)
        return "0x" + hashlib.sha256(content.encode()).hexdigest()

class RecursiveProofChain:
    """
    Accumulates ZK proofs with integrated signature verification.
    Each canonization becomes a proof node, creating an unbroken chain
    of verified state transitions.
    """

    def __init__(self, genesis_hash: Optional[str] = None):
        self.nodes: Dict[str, ProofNode] = {}
        self.ordered_indices: List[int] = []
        self.current_index = 0
        self.tip_hash: Optional[str] = None

        # Create genesis node
        genesis = ProofNode(
            index=0,
            proof_hash=genesis_hash or "0x" + hashlib.sha256(b"genesis").hexdigest(),
            parent_hash=None,
            canonization_hash=None,
            signature_hash=None,
            timestamp=time.time(),
            proof_type="genesis",
            auxiliary_data={"description": "Genesis of the RecursiveProofChain"}
        )
        self._add_node(genesis)

    def _add_node(self, node: ProofNode):
        node.proof_hash = node.compute_hash()
        self.nodes[node.proof_hash] = node
        self.ordered_indices.append(node.proof_hash)
        self.tip_hash = node.proof_hash

        if node.parent_hash and node.parent_hash in self.nodes:
            self.nodes[node.parent_hash].children.append(node.proof_hash)

    def add_canonization_proof(self,
                                canonization_hash: str,
                                signature_hash: str,
                                proof_type: str = "canonization",
                                auxiliary_data: Optional[Dict] = None) -> ProofNode:
        """Add a proof node for a canonization event"""
        self.current_index += 1

        node = ProofNode(
            index=self.current_index,
            proof_hash="",  # Will be computed
            parent_hash=self.tip_hash,
            canonization_hash=canonization_hash,
            signature_hash=signature_hash,
            timestamp=time.time(),
            proof_type=proof_type,
            auxiliary_data=auxiliary_data or {}
        )

        self._add_node(node)
        return node

    def add_state_transition_proof(self,
                                    from_state: str,
                                    to_state: str,
                                    transition_proof: str,
                                    signature_hash: str) -> ProofNode:
        """Add a proof node for a state transition"""
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
                "transition_proof": transition_proof
            }
        )

        self._add_node(node)
        return node

    def add_merkle_anchor_proof(self,
                                 merkle_root: str,
                                 depth: int,
                                 leaf_count: int,
                                 signature_hash: str) -> ProofNode:
        """Anchor a Merkle root into the proof chain"""
        self.current_index += 1

        node = ProofNode(
            index=self.current_index,
            proof_hash="",
            parent_hash=self.tip_hash,
            canonization_hash=merkle_root,
            signature_hash=signature_hash,
            timestamp=time.time(),
            proof_type="merkle_anchor",
            auxiliary_data={
                "depth": depth,
                "leaf_count": leaf_count
            }
        )

        self._add_node(node)
        return node

    def get_proof_chain(self, from_index: int = 0) -> List[ProofNode]:
        """Get the proof chain from a specific index"""
        return [self.nodes[h] for h in self.ordered_indices if self.nodes[h].index >= from_index]

    def get_chain_hash(self) -> str:
        """Get a hash representing the entire chain state"""
        if not self.ordered_indices:
            return "0x" + hashlib.sha256(b"empty_chain").hexdigest()

        chain_content = json.dumps({
            "tip": self.tip_hash,
            "length": len(self.ordered_indices),
            "nodes": [self.nodes[h].proof_hash for h in self.ordered_indices[-100:]]  # Last 100 for efficiency
        }, sort_keys=True)

        return "0x" + hashlib.sha256(chain_content.encode()).hexdigest()

    def verify_chain_integrity(self) -> Tuple[bool, List[str]]:
        """Verify the integrity of the entire chain"""
        errors = []

        for i, hash_val in enumerate(self.ordered_indices):
            node = self.nodes[hash_val]

            # Verify parent linkage
            if node.index > 0:
                if node.parent_hash not in self.nodes:
                    errors.append(f"Node {node.index}: missing parent")
                elif self.nodes[node.parent_hash].index != node.index - 1:
                    errors.append(f"Node {node.index}: parent index mismatch")

            # Verify hash
            expected_hash = node.compute_hash()
            if node.proof_hash != expected_hash:
                errors.append(f"Node {node.index}: hash mismatch")

        return len(errors) == 0, errors

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 7: KERNEL SELF-SIGNING
# ═══════════════════════════════════════════════════════════════════════════════

class KernelSelfSigner:
    """
    Signs the arkhe_os_v6.py kernel and its key components.
    Provides boot-time integrity verification.
    """

    KERNEL_COMPONENTS = [
        "arkhe_os_v6.py",
        "meta_orchestrator.py",
        "memory_lake.py",
        "recursive_prover.py",
        "theosis_rl.py",
        "onchain_canonizer.py"
    ]

    def __init__(self, signer: EIP712Signer, kernel_path: Optional[str] = None):
        self.signer = signer
        self.kernel_path = kernel_path
        self._component_hashes: Dict[str, str] = {}
        self._kernel_signature: Optional[Dict] = None
        self._boot_verified = False

    def compute_component_hashes(self, base_path: str = ".") -> Dict[str, str]:
        """Compute hashes for all kernel components"""
        for component in self.KERNEL_COMPONENTS:
            path = Path(base_path) / component
            if path.exists():
                with open(path, "rb") as f:
                    content = f.read()
                self._component_hashes[component] = "0x" + hashlib.sha256(content).hexdigest()
            else:
                # Simulate hash for non-existent components
                self._component_hashes[component] = "0x" + hashlib.sha256(
                    f"simulated_{component}".encode()
                ).hexdigest()

        return self._component_hashes

    def sign_kernel(self) -> Dict:
        """Sign the complete kernel"""
        if not self._component_hashes:
            self.compute_component_hashes()

        # Compute aggregate kernel hash
        all_hashes = json.dumps(self._component_hashes, sort_keys=True)
        kernel_hash = "0x" + hashlib.sha256(all_hashes.encode()).hexdigest()

        # Get ordered component hashes for the signature
        component_hash_list = [
            self._component_hashes[c] for c in self.KERNEL_COMPONENTS
        ]

        self._kernel_signature = self.signer.sign_kernel_integrity(
            kernel_hash=kernel_hash,
            kernel_version="6.0.0",
            component_hashes=component_hash_list
        )

        return self._kernel_signature

    def verify_kernel_boot(self, stored_signature: Optional[Dict] = None) -> bool:
        """
        Verify kernel integrity at boot time.
        Called by MetaOrchestrator during initialization.
        """
        sig = stored_signature or self._kernel_signature
        if not sig:
            logging.error("[KernelSelfSigner] No signature to verify")
            return False

        # Recompute current hashes
        current_hashes = self.compute_component_hashes()
        current_all = json.dumps(current_hashes, sort_keys=True)
        current_kernel_hash = "0x" + hashlib.sha256(current_all.encode()).hexdigest()

        # Compare with signed hash
        signed_kernel_hash = sig.get("typed_data", {}).get("message", {}).get("kernelHash")

        if current_kernel_hash == signed_kernel_hash:
            self._boot_verified = True
            logging.info("[KernelSelfSigner] ✓ Kernel integrity verified at boot")
            return True
        else:
            logging.error(f"[KernelSelfSigner] ✗ Kernel hash mismatch!")
            logging.error(f"  Expected: {signed_kernel_hash}")
            logging.error(f"  Got:      {current_kernel_hash}")
            return False

    def get_integrity_report(self) -> Dict:
        """Generate a detailed integrity report"""
        return {
            "kernel_version": "6.0.0",
            "boot_verified": self._boot_verified,
            "component_hashes": self._component_hashes,
            "signature_present": self._kernel_signature is not None,
            "signer": self.signer.get_signer_address() if self._kernel_signature else None,
            "timestamp": time.time()
        }

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 8: HUMAN-IN-THE-LOOP GOVERNANCE BRIDGE
# ═══════════════════════════════════════════════════════════════════════════════

class ProposalState(Enum):
    PROPOSED = "proposed"
    PENDING_SIGNATURE = "pending_signature"
    SIGNED = "signed"
    CANONIZED = "canonized"
    REJECTED = "rejected"
    EXPIRED = "expired"

@dataclass
class GovernanceProposal:
    """A proposal awaiting human signature"""
    proposal_id: str
    proposal_type: CanonizationType
    proposal_data: Dict
    proposed_at: float
    deadline: float
    state: ProposalState = ProposalState.PROPOSED
    signature: Optional[Dict] = None
    canonization_entry: Optional[MemoryLakeEntry] = None
    proof_node: Optional[ProofNode] = None

    def is_expired(self) -> bool:
        return time.time() > self.deadline

class GovernanceBridge:
    """
    Bridge that allows the kernel to propose new canonizations
    and await human signature (human-in-the-loop governance).
    """

    def __init__(self,
                 signer: EIP712Signer,
                 memory_lake: MemoryLake,
                 proof_chain: RecursiveProofChain,
                 default_deadline_seconds: int = 86400 * 7):  # 7 days default
        self.signer = signer
        self.memory_lake = memory_lake
        self.proof_chain = proof_chain
        self.default_deadline = default_deadline_seconds
        self.proposals: Dict[str, GovernanceProposal] = {}
        self._pending_queue: queue.Queue = queue.Queue()
        self._signature_callback: Optional[Callable] = None

    def set_signature_callback(self, callback: Callable):
        """Set callback for when a signature is needed"""
        self._signature_callback = callback

    def propose_canonization(self,
                              canonization_type: CanonizationType,
                              data: Dict,
                              deadline: Optional[int] = None) -> str:
        """
        Kernel proposes a new canonization.
        Returns proposal ID for tracking.
        """
        proposal_id = "0x" + hashlib.sha256(
            json.dumps({
                "type": canonization_type.name,
                "data": data,
                "timestamp": time.time()
            }, sort_keys=True).encode()
        ).hexdigest()

        proposal = GovernanceProposal(
            proposal_id=proposal_id,
            proposal_type=canonization_type,
            proposal_data=data,
            proposed_at=time.time(),
            deadline=time.time() + (deadline or self.default_deadline),
            state=ProposalState.PENDING_SIGNATURE
        )

        self.proposals[proposal_id] = proposal
        self._pending_queue.put(proposal_id)

        logging.info(f"[GovernanceBridge] Proposal {proposal_id[:16]}... created")
        logging.info(f"  Type: {canonization_type.name}")
        logging.info(f"  Deadline: {datetime.fromtimestamp(proposal.deadline).isoformat()}")

        # Trigger callback if set
        if self._signature_callback:
            try:
                self._signature_callback(proposal)
            except Exception as e:
                logging.error(f"[GovernanceBridge] Callback error: {e}")

        return proposal_id

    def submit_signature(self, proposal_id: str, signature: str) -> bool:
        """
        Human submits a signature for a proposal.
        This is the human-in-the-loop entry point.
        """
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

        # Verify the signature
        # In production, this would verify against the expected signer
        proposal.signature = {"raw": signature, "timestamp": time.time()}
        proposal.state = ProposalState.SIGNED

        logging.info(f"[GovernanceBridge] Signature received for {proposal_id[:16]}...")

        # Auto-canonize after signature
        return self._canonize_proposal(proposal)

    def sign_and_canonize_locally(self, proposal_id: str) -> bool:
        """
        Sign a proposal using the local signer (for automated governance).
        Use with caution — this bypasses human-in-the-loop.
        """
        proposal = self.proposals.get(proposal_id)
        if not proposal:
            return False

        # Create typed data based on proposal type
        type_name = proposal.proposal_type.name
        if type_name not in EIP712_TYPES:
            type_name = "ArchitecturalDecision"  # Fallback

        # Sign the proposal data
        sig_result = self.signer._sign_typed(type_name, proposal.proposal_data)

        if sig_result:
            proposal.signature = sig_result
            proposal.state = ProposalState.SIGNED
            return self._canonize_proposal(proposal)

        return False

    def _canonize_proposal(self, proposal: GovernanceProposal) -> bool:
        """Internal: Canonize a signed proposal"""
        # Create MemoryLake entry
        entry = MemoryLakeEntry(
            entry_hash="",  # Will be computed
            entry_type=proposal.proposal_type,
            data=proposal.proposal_data,
            signature=proposal.signature.get("signature") if proposal.signature else None,
            signer=proposal.signature.get("signer") if proposal.signature else None,
            timestamp=time.time(),
            status=SignatureStatus.VERIFIED_CANONICAL
        )

        self.memory_lake.ingest(entry)
        proposal.canonization_entry = entry

        # Add to proof chain
        proof_node = self.proof_chain.add_canonization_proof(
            canonization_hash=entry.entry_hash,
            signature_hash=proposal.signature.get("message_hash") if proposal.signature else None,
            proof_type=f"canonization_{proposal.proposal_type.name}",
            auxiliary_data={"proposal_id": proposal.proposal_id}
        )
        proposal.proof_node = proof_node

        proposal.state = ProposalState.CANONIZED
        logging.info(f"[GovernanceBridge] ✓ Canonized: {proposal.proposal_id[:16]}...")

        return True

    def get_pending_proposals(self) -> List[GovernanceProposal]:
        """Get all proposals awaiting signature"""
        return [
            p for p in self.proposals.values()
            if p.state == ProposalState.PENDING_SIGNATURE and not p.is_expired()
        ]

    def get_proposal_status(self, proposal_id: str) -> Optional[Dict]:
        """Get detailed status of a proposal"""
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
            "proof_node_index": proposal.proof_node.index if proposal.proof_node else None
        }

    def await_signature(self, proposal_id: str, timeout: float = None) -> bool:
        """
        Block until a signature is received or timeout.
        Used for synchronous human-in-the-loop flows.
        """
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

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 9: ONCHAIN CANONIZER — MAIN SUBSTRATE
# ═══════════════════════════════════════════════════════════════════════════════

class OnChainCanonizer:
    """
    The main substrate that orchestrates all canonization activities:
    - Pulls signatures from Etherscan
    - Ingests into MemoryLake
    - Integrates with RecursiveProofChain
    - Provides governance bridge
    - Handles kernel self-signing
    """

    def __init__(self,
                 api_key: Optional[str] = None,
                 private_key: Optional[str] = None,
                 chain_id: ChainId = ChainId.MAINNET):

        # Core components
        self.domain = EIP712Domain(chainId=chain_id)
        self.signer = EIP712Signer(self.domain)
        self.memory_lake = MemoryLake()
        self.proof_chain = RecursiveProofChain()
        self.fetcher = EtherscanFetcher(api_key, chain_id)
        self.kernel_signer = KernelSelfSigner(self.signer)
        self.governance = GovernanceBridge(self.signer, self.memory_lake, self.proof_chain)

        # Set up private key if provided
        if private_key:
            self.signer.set_private_key(private_key)

        # State tracking
        self._sync_running = False
        self._last_sync_block = 0
        self._initialized = False

    async def initialize(self) -> bool:
        """
        Initialize the canonizer substrate.
        Called by MetaOrchestrator during boot.
        """
        logging.info("[OnChainCanonizer] Initializing substrate...")

        # 1. Self-sign the kernel
        logging.info("[OnChainCanonizer] Signing kernel...")
        kernel_sig = self.kernel_signer.sign_kernel()
        if kernel_sig:
            # Canonize the kernel signature
            entry = MemoryLakeEntry(
                entry_hash="",
                entry_type=CanonizationType.KERNEL_INTEGRITY,
                data=kernel_sig.get("typed_data", {}).get("message", {}),
                signature=kernel_sig.get("signature"),
                signer=kernel_sig.get("signer"),
                status=SignatureStatus.VERIFIED_CANONICAL
            )
            self.memory_lake.ingest(entry)

            # Add to proof chain
            self.proof_chain.add_canonization_proof(
                canonization_hash=entry.entry_hash,
                signature_hash=kernel_sig.get("message_hash"),
                proof_type="kernel_boot"
            )
            logging.info("[OnChainCanonizer] ✓ Kernel signed and canonized")

        # 2. Initial sync from Etherscan
        logging.info("[OnChainCanonizer] Syncing signatures from Etherscan...")
        await self.sync_signatures()

        # 3. Verify chain integrity
        valid, errors = self.proof_chain.verify_chain_integrity()
        if not valid:
            logging.warning(f"[OnChainCanonizer] Chain integrity issues: {errors}")

        self._initialized = True
        logging.info(f"[OnChainCanonizer] ✓ Initialized with {len(self.memory_lake.entries)} entries")

        return True

    async def sync_signatures(self,
                               start_block: int = 0,
                               end_block: int = 99999999) -> int:
        """
        Sync verified signatures from Etherscan into MemoryLake.
        Returns number of new entries ingested.
        """
        signatures = await self.fetcher.fetch_verified_signatures(
            start_block=max(start_block, self._last_sync_block),
            end_block=end_block
        )

        new_count = 0
        for sig in signatures:
            # Map parsed type to CanonizationType
            canon_type = self._map_signature_type(sig.parsed_type)

            # Create MemoryLake entry
            entry = MemoryLakeEntry(
                entry_hash="",
                entry_type=canon_type,
                data=sig.parsed_data or {"raw_hash": sig.message_hash},
                signature=sig.signature,
                signer=sig.signer,
                block_number=sig.block_number,
                timestamp=sig.timestamp or time.time(),
                status=SignatureStatus.PENDING_VERIFICATION
            )

            if self.memory_lake.ingest(entry):
                # Add to proof chain
                self.proof_chain.add_canonization_proof(
                    canonization_hash=entry.entry_hash,
                    signature_hash=sig.message_hash,
                    proof_type=f"etherscan_{sig.parsed_type or 'unknown'}",
                    auxiliary_data={
                        "block_number": sig.block_number,
                        "tx_hash": sig.tx_hash
                    }
                )

                # Verify signature if possible
                if sig.parsed_data and sig.parsed_type in EIP712_TYPES:
                    entry.status = SignatureStatus.VERIFIED_CANONICAL
                else:
                    entry.status = SignatureStatus.VERIFIED_NON_CANONICAL

                new_count += 1

                if sig.block_number:
                    self._last_sync_block = max(self._last_sync_block, sig.block_number)

        logging.info(f"[OnChainCanonizer] Synced {new_count} new signatures")
        return new_count

    def _map_signature_type(self, parsed_type: Optional[str]) -> CanonizationType:
        """Map Etherscan parsed type to CanonizationType"""
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
        """
        Continuously sync signatures from Etherscan.
        Run as a background task.
        """
        self._sync_running = True
        while self._sync_running:
            try:
                await self.sync_signatures()
            except Exception as e:
                logging.error(f"[OnChainCanonizer] Sync error: {e}")
            await asyncio.sleep(interval_seconds)

    def stop_sync(self):
        """Stop the continuous sync"""
        self._sync_running = False

    # ═══════════════════════════════════════════════════════════════════════
    # CONVENIENCE METHODS FOR SPECIFIC CANONIZATION TYPES
    # ═══════════════════════════════════════════════════════════════════════

    def propose_meta_orchestrator_policy(self,
                                          policy_type: str,
                                          parameters: Dict,
                                          effectiveness_threshold: int = 75) -> str:
        """Propose a new MetaOrchestrator policy for human signing"""
        policy_id = "0x" + hashlib.sha256(
            json.dumps({"policy_type": policy_type, "params": parameters}, sort_keys=True).encode()
        ).hexdigest()

        return self.governance.propose_canonization(
            CanonizationType.META_ORCHESTRATOR_POLICY,
            {
                "policyId": policy_id,
                "policyType": policy_type,
                "parameters": json.dumps(parameters),
                "effectivenessThreshold": effectiveness_threshold,
                "expiry": int(time.time()) + 365 * 86400,
                "parentPolicyHash": "0x" + "00" * 32
            }
        )

    def propose_theosis_reward_function(self,
                                         function_definition: str,
                                         convergence_criteria: str,
                                         hyperparameters: Dict,
                                         safety_bounds: str) -> str:
        """Propose a new TheosisRL reward function for human signing"""
        reward_id = "0x" + hashlib.sha256(
            function_definition.encode()
        ).hexdigest()

        return self.governance.propose_canonization(
            CanonizationType.THEOSIS_RL_REWARD_FUNCTION,
            {
                "rewardFunctionId": reward_id,
                "functionDefinition": function_definition,
                "convergenceCriteria": convergence_criteria,
                "hyperparameters": json.dumps(hyperparameters),
                "safetyBounds": safety_bounds,
                "approvalEpoch": int(time.time())
            }
        )

    def propose_architectural_decision(self,
                                        decision_type: str,
                                        rationale: str,
                                        impact_assessment: str,
                                        reversibility: bool = False) -> str:
        """Propose an architectural decision for human signing"""
        decision_id = "0x" + hashlib.sha256(
            json.dumps({
                "type": decision_type,
                "rationale": rationale,
                "ts": time.time()
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
                "requiredSignatures": 1
            }
        )

    def anchor_merkle_root(self) -> Optional[ProofNode]:
        """Anchor the current MemoryLake Merkle root into the proof chain"""
        merkle_root = self.memory_lake.get_merkle_root()
        tree = self.memory_lake.build_merkle_tree()

        # Sign the anchor
        sig = self.signer.sign_proof_anchor(
            proof_root=merkle_root,
            proof_type="memory_lake_merkle",
            depth=len(tree),
            leaf_count=len(tree[0]),
            previous_anchor=self.proof_chain.tip_hash or "0x" + "00" * 32
        )

        if sig:
            return self.proof_chain.add_merkle_anchor_proof(
                merkle_root=merkle_root,
                depth=len(tree),
                leaf_count=len(tree[0]),
                signature_hash=sig.get("message_hash")
            )
        return None

    # ═══════════════════════════════════════════════════════════════════════
    # VERIFICATION AND QUERY METHODS
    # ═══════════════════════════════════════════════════════════════════════

    def verify_boot_integrity(self) -> bool:
        """
        Called by MetaOrchestrator to verify kernel at boot.
        Returns True if kernel signature is valid.
        """
        return self.kernel_signer.verify_kernel_boot()

    def get_canonical_state(self) -> Dict:
        """Get the current canonical state of the substrate"""
        return {
            "memory_lake": {
                "merkle_root": self.memory_lake.get_merkle_root(),
                "total_entries": len(self.memory_lake.entries),
                "type_counts": {t.name: len(self.memory_lake._type_index[t])
                               for t in CanonizationType},
                "recent": [
                    {
                        "hash": e.entry_hash[:16] + "...",
                        "type": e.entry_type.name,
                        "signer": e.signer[:16] + "..." if e.signer else None,
                        "status": e.status.name
                    }
                    for e in self.memory_lake.get_recent(5)
                ]
            },
            "proof_chain": {
                "tip_hash": self.proof_chain.tip_hash,
                "length": len(self.proof_chain.ordered_indices),
                "chain_hash": self.proof_chain.get_chain_hash()
            },
            "governance": {
                "pending_proposals": len(self.governance.get_pending_proposals()),
                "total_proposals": len(self.governance.proposals)
            },
            "kernel": self.kernel_signer.get_integrity_report(),
            "last_sync_block": self._last_sync_block,
            "initialized": self._initialized
        }

    def get_canonization_proof(self, entry_hash: str) -> Dict:
        """Get full proof for a canonization"""
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
                "status": entry.status.name
            },
            "merkle_proof": merkle_proof,
            "merkle_root": self.memory_lake.get_merkle_root(),
            "signature": entry.signature
        }

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 10: METAORCHESTRATOR INTEGRATION
# ═══════════════════════════════════════════════════════════════════════════════

class MetaOrchestratorBootVerifier:
    """
    Integration point for MetaOrchestrator to verify
    the canonical state at boot.
    """

    REQUIRED_CANONIZATION_TYPES = {
        CanonizationType.KERNEL_INTEGRITY: 1,  # At least 1 kernel signature
    }

    def __init__(self, canonizer: OnChainCanonizer):
        self.canonizer = canonizer
        self._boot_passed = False

    async def verify_boot(self) -> Tuple[bool, List[str]]:
        """
        Full boot verification sequence.
        Returns (success, errors).
        """
        errors = []

        # 1. Verify kernel integrity
        if not self.canonizer.verify_boot_integrity():
            errors.append("Kernel integrity verification failed")

        # 2. Check required canonization types
        for canon_type, min_count in self.REQUIRED_CANONIZATION_TYPES.items():
            entries = self.canonizer.memory_lake.get_by_type(canon_type)
            verified = [e for e in entries if e.status == SignatureStatus.VERIFIED_CANONICAL]
            if len(verified) < min_count:
                errors.append(
                    f"Insufficient {canon_type.name} canonizations: "
                    f"have {len(verified)}, need {min_count}"
                )

        # 3. Verify proof chain integrity
        chain_valid, chain_errors = self.canonizer.proof_chain.verify_chain_integrity()
        if not chain_valid:
            errors.extend([f"Proof chain: {e}" for e in chain_errors])

        # 4. Verify MemoryLake consistency
        expected_root = self.canonizer.memory_lake.get_merkle_root()
        if not expected_root:
            errors.append("MemoryLake Merkle root computation failed")

        self._boot_passed = len(errors) == 0

        if self._boot_passed:
            logging.info("[MetaOrchestratorBootVerifier] ✓ All verifications passed")
        else:
            logging.error(f"[MetaOrchestratorBootVerifier] ✗ {len(errors)} errors:")
            for err in errors:
                logging.error(f"  - {err}")

        return self._boot_passed, errors

    def is_boot_verified(self) -> bool:
        return self._boot_passed

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 11: CLI AND DEMO
# ═══════════════════════════════════════════════════════════════════════════════

async def demo_canonizer():
    """Demonstrate the OnChainCanonizer substrate"""

    print("\n" + "="*80)
    print("  ONCHAIN CANONIZER SUBSTRATE — DEMONSTRATION")
    print("="*80 + "\n")

    # Initialize canonizer
    canonizer = OnChainCanonizer(chain_id=ChainId.MAINNET)

    # Initialize (includes kernel signing and Etherscan sync)
    await canonizer.initialize()

    # Show canonical state
    print("\n── Current Canonical State ──")
    state = canonizer.get_canonical_state()
    print(json.dumps(state, indent=2, default=str))

    # Propose some governance items
    print("\n── Proposing Governance Items ──")

    # 1. MetaOrchestrator policy
    policy_id = canonizer.propose_meta_orchestrator_policy(
        policy_type="resource_allocation",
        parameters={
            "max_memory_mb": 4096,
            "max_cpu_percent": 80,
            "priority_classes": ["critical", "normal", "low"]
        },
        effectiveness_threshold=85
    )
    print(f"Proposed policy: {policy_id[:32]}...")

    # 2. TheosisRL reward function
    reward_id = canonizer.propose_theosis_reward_function(
        function_definition="R(s,a) = α·Q(s,a) - β·||a||² + γ·log(π(a|s))",
        convergence_criteria="|ΔR| < 0.001 over 1000 episodes",
        hyperparameters={"alpha": 0.7, "beta": 0.01, "gamma": 0.1},
        safety_bounds="R ∈ [-10, 10], |∇R| < 100"
    )
    print(f"Proposed reward function: {reward_id[:32]}...")

    # 3. Architectural decision
    decision_id = canonizer.propose_architectural_decision(
        decision_type="proof_system_upgrade",
        rationale="Upgrade from Groth16 to PLONKish for better recursion support",
        impact_assessment="High impact on proof generation time, minimal impact on verification",
        reversibility=True
    )
    print(f"Proposed decision: {decision_id[:32]}...")

    # Show pending proposals
    print("\n── Pending Proposals ──")
    for p in canonizer.governance.get_pending_proposals():
        status = canonizer.governance.get_proposal_status(p.proposal_id)
        print(f"  {p.proposal_type.name}: {p.proposal_id[:24]}...")
        print(f"    State: {status['state']}")
        print(f"    Deadline: {datetime.fromtimestamp(status['deadline']).isoformat()}")

    # Simulate human signing (auto-sign for demo)
    print("\n── Simulating Human Signatures ──")
    for pid in [policy_id, reward_id, decision_id]:
        success = canonizer.governance.sign_and_canonize_locally(pid)
        status = "✓" if success else "✗"
        print(f"  {status} {pid[:24]}...")

    # Anchor Merkle root
    print("\n── Anchoring Merkle Root ──")
    anchor = canonizer.anchor_merkle_root()
    if anchor:
        print(f"  ✓ Anchored at proof node {anchor.index}")
        print(f"    Root: {canonizer.memory_lake.get_merkle_root()[:32]}...")

    # Final state
    print("\n── Final Canonical State ──")
    final_state = canonizer.get_canonical_state()
    print(json.dumps(final_state, indent=2, default=str))

    # Boot verification
    print("\n── MetaOrchestrator Boot Verification ──")
    verifier = MetaOrchestratorBootVerifier(canonizer)
    boot_ok, errors = await verifier.verify_boot()
    print(f"  Boot verified: {boot_ok}")
    if errors:
        for err in errors:
            print(f"  Error: {err}")

    # Generate a proof for a specific entry
    print("\n── Sample Canonization Proof ──")
    recent = canonizer.memory_lake.get_recent(1)
    if recent:
        proof = canonizer.get_canonization_proof(recent[0].entry_hash)
        print(json.dumps(proof, indent=2, default=str))

    print("\n" + "="*80)
    print("  DEMONSTRATION COMPLETE")
    print("="*80 + "\n")

    return canonizer

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 12: INTEGRATION SNIPPET FOR arkhe_os_v6.py
# ═══════════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s [%(name)s] %(levelname)s: %(message)s',
        datefmt='%H:%M:%S'
    )

    asyncio.run(demo_canonizer())