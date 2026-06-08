"""Testes para OnChainCanonizer 1100."""

import pytest
import numpy as np
from cathedral.substrates.onchain import (
    OnChainCanonizer, CanonizationType, ChainId,
    MemoryLake, EIP712Signer, GovernanceBridge,
)


class TestOnChainCanonizer:
    @pytest.mark.asyncio
    async def test_initialization(self):
        canonizer = OnChainCanonizer(chain_id=ChainId.LOCAL)
        result = await canonizer.initialize()
        assert result is True
        assert canonizer._initialized is True
        assert len(canonizer.memory_lake.entries) > 0

    def test_propose_policy(self):
        canonizer = OnChainCanonizer(chain_id=ChainId.LOCAL)
        pid = canonizer.propose_meta_orchestrator_policy(
            "resource_allocation",
            {"max_memory_mb": 4096},
        )
        assert pid.startswith("0x")
        assert len(pid) == 66

    def test_garak_canonization(self):
        canonizer = OnChainCanonizer(chain_id=ChainId.LOCAL)
        report = {
            "scan_id": "GARAK-TEST-001",
            "risk_score": 0.35,
            "failure_rate": 0.12,
            "critical_failures": 1,
        }
        node = canonizer.canonize_garak_scan(report)
        assert node is not None
        assert node.proof_type == "garak_scan"

    def test_merkle_anchor(self):
        canonizer = OnChainCanonizer(chain_id=ChainId.LOCAL)
        canonizer.propose_meta_orchestrator_policy("test", {})
        anchor = canonizer.anchor_merkle_root()
        assert anchor is not None
        assert anchor.proof_type == "merkle_anchor"

    def test_canonical_state(self):
        canonizer = OnChainCanonizer(chain_id=ChainId.LOCAL)
        state = canonizer.get_canonical_state()
        assert "memory_lake" in state
        assert "proof_chain" in state
        assert "governance" in state
        assert "kernel" in state

    def test_telemetry(self):
        canonizer = OnChainCanonizer(chain_id=ChainId.LOCAL)
        telem = canonizer.get_telemetry()
        assert telem["module"] == "OnChainCanonizer"
        assert telem["version"] == "1.0.0"
        assert telem["substrate"] == "1100"
        assert "ONCHAIN-CANONIZER-1100" in telem["seal"]
