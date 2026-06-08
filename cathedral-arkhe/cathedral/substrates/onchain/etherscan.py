"""
Etherscan Fetcher — Substrato 1100 v1.0.0
Busca assinaturas verificadas do Etherscan para endereços alvo.
"""

from __future__ import annotations
import asyncio
import hashlib
import json
import logging
import time
from dataclasses import dataclass
from typing import Dict, List, Optional

from cathedral.substrates.onchain.types import ChainId, EtherscanSignature

# Fallback para quando aiohttp não está disponível
try:
    import aiohttp
    AIOHTTP_AVAILABLE = True
except ImportError:
    AIOHTTP_AVAILABLE = False


class EtherscanFetcher:
    """
    Busca assinaturas verificadas do Etherscan.
    Suporta chamadas reais de API e modo simulação.
    """

    BASE_URLS = {
        ChainId.MAINNET: "https://api.etherscan.io/api",
        ChainId.SEPOLIA: "https://api-sepolia.etherscan.io/api",
        ChainId.GOERLI: "https://api-goerli.etherscan.io/api",
        ChainId.ARBITRUM_ONE: "https://api.arbiscan.io/api",
        ChainId.OPTIMISM: "https://api-optimistic.etherscan.io/api",
    }

    # FIX BUG-2: usar lista de endereços
    TARGET_ADDRESSES = [
        "0xbF7Da1f568684889A69A5BED9F1311F703985590",
        "0x716aD3C33A9B9a0A18967357969b94EE7d2ABC10",
    ]

    def __init__(
        self,
        api_key: Optional[str] = None,
        chain_id: ChainId = ChainId.MAINNET,
    ):
        self.api_key = api_key
        self.chain_id = chain_id
        self.base_url = self.BASE_URLS.get(
            chain_id, self.BASE_URLS[ChainId.MAINNET]
        )
        self.session: Optional[aiohttp.ClientSession] = None
        self._cache: List[EtherscanSignature] = []
        self._last_fetch = 0
        self._rate_limit_delay = 0.2  # 5 calls/sec para tier gratuito

    async def _get_session(self):
        if self.session is None and AIOHTTP_AVAILABLE:
            self.session = aiohttp.ClientSession()
        return self.session

    async def fetch_verified_signatures(
        self,
        start_block: int = 0,
        end_block: int = 99_999_999,
        page: int = 1,
        offset: int = 100,
    ) -> List[EtherscanSignature]:
        """Busca assinaturas verificadas do Etherscan."""
        if not AIOHTTP_AVAILABLE:
            return self._simulate_signatures()

        all_signatures: List[EtherscanSignature] = []

        for address in self.TARGET_ADDRESSES:
            sigs = await self._fetch_for_address(
                address, start_block, end_block, page, offset
            )
            all_signatures.extend(sigs)

        self._cache = all_signatures
        self._last_fetch = time.time()
        return all_signatures

    async def _fetch_for_address(
        self,
        address: str,
        start_block: int,
        end_block: int,
        page: int,
        offset: int,
    ) -> List[EtherscanSignature]:
        await asyncio.sleep(self._rate_limit_delay)

        params = {
            "module": "account",
            "action": "txlist",
            "address": address,
            "startblock": start_block,
            "endblock": end_block,
            "page": page,
            "offset": offset,
            "sort": "desc",
            "apikey": self.api_key or "YourApiKeyToken",
        }

        try:
            session = await self._get_session()
            async with session.get(self.base_url, params=params) as resp:
                data = await resp.json()

            if data.get("status") != "1":
                return []

            signatures = []
            for tx in data.get("result", []):
                sig = self._parse_tx_as_signature(tx, address)
                if sig:
                    signatures.append(sig)
            return signatures

        except Exception as e:
            logging.warning(f"[EtherscanFetcher] API error: {e}")
            return []

    def _parse_tx_as_signature(
        self, tx: Dict, signer: str
    ) -> Optional[EtherscanSignature]:
        input_data = tx.get("input", "0x")
        if len(input_data) < 130:
            return None

        method_id = input_data[:10]
        signature_methods = {
            "0x8208a634",  # isValidSignature(bytes32,bytes)
            "0x1626ba7e",  # isValidSignature(bytes,bytes)
            "0xa3b22fc4",  # verify
            "0x304e6ade",  # isValidSignatureWithResolver
        }

        if method_id in signature_methods:
            try:
                calldata = input_data[10:]
                message_hash = "0x" + calldata[0:64] if len(calldata) >= 64 else None
                signature = "0x" + calldata[128:192] if len(calldata) >= 192 else None

                if message_hash and signature:
                    return EtherscanSignature(
                        signature=signature,
                        message_hash=message_hash,
                        signer=signer,
                        block_number=int(tx.get("blockNumber", 0)),
                        timestamp=int(tx.get("timeStamp", 0)),
                        tx_hash=tx.get("hash"),
                        parsed_type="EIP1271",
                    )
            except Exception:
                pass

        if "cathedralarkhe" in input_data.lower() or "arkhe" in input_data.lower():
            return EtherscanSignature(
                signature=input_data[-130:] if len(input_data) >= 130 else input_data,
                message_hash="0x" + hashlib.sha256(input_data.encode()).hexdigest(),
                signer=signer,
                block_number=int(tx.get("blockNumber", 0)),
                timestamp=int(tx.get("timeStamp", 0)),
                tx_hash=tx.get("hash"),
                parsed_type="EIP712_IN_CALLDATA",
            )

        return None

    def _simulate_signatures(self) -> List[EtherscanSignature]:
        """Gera assinaturas simuladas para desenvolvimento/testes."""
        now = int(time.time())

        simulations = [
            EtherscanSignature(
                signature="0x" + "a1b2c3d4" * 16,
                message_hash="0x" + hashlib.sha256(b"kernel_integrity_v6").hexdigest(),
                signer=self.TARGET_ADDRESSES[0],
                block_number=19_500_000,
                timestamp=now - 86400,
                tx_hash="0x" + hashlib.sha256(b"tx1").hexdigest(),
                parsed_type="KernelIntegrity",
                parsed_data={
                    "kernelHash": "0x" + hashlib.sha256(b"arkhe_os_v6.py").hexdigest(),
                    "kernelVersion": "6.0.0",
                    "canonizationType": 1,
                },
            ),
            EtherscanSignature(
                signature="0x" + "e5f6a7b8" * 16,
                message_hash="0x" + hashlib.sha256(b"meta_orch_policy").hexdigest(),
                signer=self.TARGET_ADDRESSES[0],
                block_number=19_500_050,
                timestamp=now - 43200,
                tx_hash="0x" + hashlib.sha256(b"tx2").hexdigest(),
                parsed_type="MetaOrchestratorPolicy",
                parsed_data={
                    "policyId": "0x" + hashlib.sha256(b"policy_001").hexdigest(),
                    "policyType": "resource_allocation",
                    "effectivenessThreshold": 75,
                },
            ),
            EtherscanSignature(
                signature="0x" + "c9d0e1f2" * 16,
                message_hash="0x" + hashlib.sha256(b"theosis_reward").hexdigest(),
                signer=self.TARGET_ADDRESSES[1],
                block_number=19_500_100,
                timestamp=now - 21600,
                tx_hash="0x" + hashlib.sha256(b"tx3").hexdigest(),
                parsed_type="TheosisRLRewardFunction",
                parsed_data={
                    "rewardFunctionId": "0x" + hashlib.sha256(b"reward_fn_v2").hexdigest(),
                    "convergenceCriteria": "epsilon < 0.001 over 1000 episodes",
                },
            ),
            EtherscanSignature(
                signature="0x" + "1a2b3c4d" * 16,
                message_hash="0x" + hashlib.sha256(b"state_transition").hexdigest(),
                signer=self.TARGET_ADDRESSES[0],
                block_number=19_500_150,
                timestamp=now - 3600,
                tx_hash="0x" + hashlib.sha256(b"tx4").hexdigest(),
                parsed_type="StateTransition",
                parsed_data={
                    "fromStateHash": "0x" + "00" * 32,
                    "toStateHash": "0x" + "ff" * 32,
                    "transitionType": "epoch_rollover",
                },
            ),
            EtherscanSignature(
                signature="0x" + "5e6f7a8b" * 16,
                message_hash="0x" + hashlib.sha256(b"arch_decision").hexdigest(),
                signer=self.TARGET_ADDRESSES[1],
                block_number=19_500_200,
                timestamp=now,
                tx_hash="0x" + hashlib.sha256(b"tx5").hexdigest(),
                parsed_type="ArchitecturalDecision",
                parsed_data={
                    "decisionId": "0x" + hashlib.sha256(b"decision_zk_integration").hexdigest(),
                    "decisionType": "proof_system_upgrade",
                    "rationale": "Integrate Groth16 for recursive proof composition",
                },
            ),
        ]

        self._cache = simulations
        return simulations

    def get_telemetry(self) -> Dict:
        return {
            "module": "EtherscanFetcher",
            "version": "1.0.0",
            "substrate": "1100",
            "seal": "ETHERSCAN-FETCHER-1100-v1.0.0-2026-06-08",
            "chain_id": self.chain_id.name,
            "cached_signatures": len(self._cache),
            "target_addresses": len(self.TARGET_ADDRESSES),
            "aiohttp_available": AIOHTTP_AVAILABLE,
        }
