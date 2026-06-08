"""
Integração OnChainCanonizer → CathedralOrchestratorV5_1
Substrato 1100 integrado ao pipeline v5.1.0
"""

from __future__ import annotations
import asyncio
import logging
import time
import hashlib
import json
from typing import Any, Dict, Optional

from cathedral.orchestrator.v5_1 import CathedralOrchestratorV5_1
from cathedral.substrates.onchain import (
    OnChainCanonizer, CanonizationType, ChainId,
    GovernanceBridge, ProposalState,
)


class CathedralOrchestratorV5_1_OnChain(CathedralOrchestratorV5_1):
    """
    Orquestrador v5.1.0 + OnChainCanonizer (Substrato 1100)

    Pipeline estendido:
    GARAK → PLAN → INFER → ZKML → STETH → THEOSIS → KLEROS →
    CANONIZE → ANCHOR → LEARN
    """

    def __init__(
        self,
        model_path=None,
        n_ctx=2048,
        dashboard_path=None,
        garak_generator_spec="llama-cpp.simulated",
        garak_probe_spec="all",
        garak_scan_interval=0,
        # OnChainCanonizer params
        etherscan_api_key: Optional[str] = None,
        canonizer_private_key: Optional[str] = None,
        chain_id: ChainId = ChainId.MAINNET,
        merkle_anchor_interval: int = 100,
        auto_canonize_kleros: bool = True,
        auto_canonize_garak: bool = True,
    ):
        super().__init__(
            model_path=model_path,
            n_ctx=n_ctx,
            dashboard_path=dashboard_path,
            garak_generator_spec=garak_generator_spec,
            garak_probe_spec=garak_probe_spec,
            garak_scan_interval=garak_scan_interval,
        )

        self.canonizer = OnChainCanonizer(
            api_key=etherscan_api_key,
            private_key=canonizer_private_key,
            chain_id=chain_id,
        )
        self._merkle_anchor_interval = merkle_anchor_interval
        self._auto_canonize_kleros = auto_canonize_kleros
        self._auto_canonize_garak = auto_canonize_garak
        self._entries_since_anchor = 0
        self.version = "5.1.0-onchain"
        self._seal = "ORCHESTRATOR-v5.1.0-ONCHAIN-2026-06-08"

    async def boot(self):
        """Boot extendido com inicialização do OnChainCanonizer."""
        logging.info("[OrchestratorV5.1-OnChain] Booting with OnChainCanonizer...")

        # Inicializa canonizer (kernel signing + Etherscan sync)
        await self.canonizer.initialize()

        # Boot padrão
        self.start_cycle()

        # Inicia sync contínuo
        asyncio.create_task(
            self.canonizer.continuous_sync(interval_seconds=300)
        )

        logging.info("[OrchestratorV5.1-OnChain] Boot complete")
        return True

    def infer(self, prompt, max_tokens=50, use_agentic=False, run_garak=False):
        """Inferência com canonização automática de resultados."""
        result = super().infer(
            prompt, max_tokens=max_tokens,
            use_agentic=use_agentic, run_garak=run_garak,
        )

        # Canoniza leitura Theosis
        if self.vt and self.vt.readings:
            latest = self.vt.readings[-1]
            self._canonize_theosis_reading(latest)

        # Verifica se deve fazer anchor Merkle
        self._entries_since_anchor += 1
        if self._entries_since_anchor >= self._merkle_anchor_interval:
            self._anchor_merkle()

        return result

    def run_garak_cycle(self, force=False):
        """Scan Garak com canonização automática do relatório."""
        report = super().run_garak_cycle(force=force)

        if self._auto_canonize_garak and report.get("status") != "SKIPPED":
            proof_node = self.canonizer.canonize_garak_scan(report)
            if proof_node:
                logging.info(
                    f"[OnChainCanonizer] Garak scan canonized at node {proof_node.index}"
                )

        return report

    def _canonize_theosis_reading(self, reading: Dict):
        """Canoniza leitura Theosis como StateTransition."""
        try:
            self.canonizer.governance.propose_canonization(
                CanonizationType.STATE_TRANSITION,
                {
                    "fromStateHash": "0x" + "00" * 32,
                    "toStateHash": "0x" + hashlib.sha256(
                        json.dumps(reading, sort_keys=True, default=str).encode()
                    ).hexdigest()[:64],
                    "transitionType": "theosis_update",
                    "theosis": reading.get("theosis"),
                    "tee": reading.get("tee"),
                    "gate": reading.get("gate"),
                },
                deadline=3600,  # 1 hora para assinatura
            )
        except Exception as e:
            logging.warning(f"[OnChainCanonizer] Failed to canonize reading: {e}")

    def _anchor_merkle(self):
        """Faz anchor do Merkle root na proof chain."""
        anchor = self.canonizer.anchor_merkle_root()
        if anchor:
            logging.info(
                f"[OnChainCanonizer] Merkle anchored at node {anchor.index}"
            )
        self._entries_since_anchor = 0

    def end_cycle(self):
        """Finaliza ciclo com anchor final e relatório on-chain."""
        # Anchor final
        self._anchor_merkle()

        # Canoniza relatório de ciclo
        cycle_report = {
            "cycles": self.cycle_count,
            "quarantined": self._quarantined,
            "timestamp": time.time(),
        }

        self.canonizer.governance.propose_canonization(
            CanonizationType.MEMORY_LAKE_SNAPSHOT,
            {
                "snapshotHash": "0x" + hashlib.sha256(
                    json.dumps(cycle_report, sort_keys=True, default=str).encode()
                ).hexdigest(),
                "lakeVersion": self.cycle_count,
                "totalEntries": len(self.canonizer.memory_lake.entries),
                "merkleRoot": self.canonizer.memory_lake.get_merkle_root(),
                "compressionAlgo": "zstd",
            },
        )

        report = super().end_cycle()
        report["onchain"] = self.canonizer.get_telemetry()
        return report

    def get_telemetry(self):
        telem = super().get_telemetry()
        telem["version"] = self.version
        telem["seal"] = self._seal
        telem["onchain_canonizer"] = self.canonizer.get_telemetry()
        telem["merkle_anchor_interval"] = self._merkle_anchor_interval
        telem["auto_canonize_kleros"] = self._auto_canonize_kleros
        telem["auto_canonize_garak"] = self._auto_canonize_garak
        return telem
