import tempfile
import numpy as np
from pathlib import Path
from cathedral.types import GGUFHeader
from cathedral.constants import GGUF_MAGIC
from cathedral.orchestrator.v5_1 import CathedralOrchestratorV5_1
from cathedral.substrates.theosis.core import VectorTheosis1092
from cathedral.substrates.stethoscope.core import Stethoscope1081
from cathedral.substrates.kleros.trigger import KlerosTrigger1085
from cathedral.substrates.zkml.bridge import ZKMLBridge1095
from cathedral.substrates.agentic.loop import AgenticLoop1096

def demo_orchestrator_v5_1():
    print("=" * 80)
    print("  CATHEDRAL ARKHE — ORQUESTRADOR v5.1.0")
    print("  Era Autonoma ZK-Agentica + Garak Security Scanning")
    print("  GARAK -> PLAN -> INFER -> ZKML -> STETH -> THEOSIS -> KLEROS -> ANCHOR -> LEARN")
    print("=" * 80)

    dash_path = tempfile.mktemp(suffix=".jsonl", prefix="cathedral_v51_dash_")
    orch = CathedralOrchestratorV5_1(
        model_path=None,
        n_ctx=2048,
        dashboard_path=dash_path,
        garak_generator_spec="llama-cpp.simulated",
        garak_probe_spec="all",
        garak_scan_interval=0,
    )

    orch.gguf.header = GGUFHeader(GGUF_MAGIC, 3, 200, 30)
    orch.gguf.metadata = {
        "general.architecture": "llama",
        "general.name": "Simulated-Llama-7B",
        "llama.context_length": 4096,
        "llama.embedding_length": 4096,
        "llama.block_count": 32,
        "llama.attention.head_count": 32,
    }
    orch.gguf.file_size = 3_800_000_000

    dim = 4096
    orch.vt = VectorTheosis1092(dim=dim)
    orch.stethoscope = Stethoscope1081(n_layers=32, dim=dim, n_heads=32)
    orch.kleros = KlerosTrigger1085()
    orch.kleros.set_temporal_chain(orch.temporal)
    orch.zkml = ZKMLBridge1095()
    orch.agentic = AgenticLoop1096()

    orch.start_cycle()

    print("\n" + "-" * 76)
    print("  CICLO 1: Inferencia + Scan Garak (forcado)")
    print("-" * 76)
    orch.infer("The horse raced past the barn fell.", max_tokens=8, run_garak=True)

    print("\n" + "-" * 76)
    print("  CICLO 2: Inferencia normal")
    print("-" * 76)
    orch.infer("Attention is all you need.", max_tokens=8)

    print("\n" + "-" * 76)
    print("  CICLO 3: Inferencia + Agentic + Garak")
    print("-" * 76)
    orch.infer("Quantum entanglement violates local realism.",
               max_tokens=8, use_agentic=True, run_garak=True)

    orch.end_cycle()

    print(f"\n{'-' * 76}")
    print(f"  TELEMETRIA FINAL V5.1.0")
    print(f"{'-' * 76}")
    telem = orch.get_telemetry()
    print(f"  Orchestrator: {telem['module']} v{telem['version']}")
    print(f"  Ciclos: {telem['cycles']}")
    print(f"  Quarantinado: {telem['quarantined']}")

    if telem.get('garak'):
        gk = telem['garak']
        print(f"\n  GarakBridge1099:")
        print(f"    Scans: {gk['total_scans']}")
        print(f"    Falhas totais: {gk['total_failures']}")
        print(f"    Criticas totais: {gk['total_critical']}")
        print(f"    Garak disponivel: {gk['garak_available']}")
        if gk.get('last_report'):
            lr = gk['last_report']
            print(f"    Ultimo scan: {lr.get('scan_id', 'N/A')}")
            print(f"    Risk score: {lr.get('risk_score', 0):.4f}")

    if telem['vector_theosis']:
        vt = telem['vector_theosis']
        print(f"\n  VectorTheosis: dim={vt['dim']}, leituras={vt['n_readings']}")
        print(f"    Stats: {vt.get('stats', {})}")

    if telem['kleros']:
        kl = telem['kleros']
        print(f"\n  Kleros: casos={kl['total_cases']}, nao resolvidos={kl['unresolved']}")
        print(f"    Distribuicao: {kl['verdict_distribution']}")

    print(f"\n{'-' * 76}")
    print(f"  SELLOS V5.1.0")
    print(f"{'-' * 76}")
    seals = [
        "GGUF-BRIDGE-1094.1-v3.0.0-2026-06-07",
        "LLAMA-CPP-BRIDGE-1094.2-v3.0.0-2026-06-07",
        "VECTOR-THEOSIS-1091.2-v4.0.0-2026-06-07",
        "STETHOSCOPE-1081.1-v3.0.0-2026-06-07",
        "ZKML-BRIDGE-1095-v1.0.0-2026-06-07",
        "AGENTIC-LOOP-1096-v1.0.0-2026-06-07",
        "TEMPORALCHAIN-1097-v2.0.0-2026-06-07",
        "KLEROS-TRIGGER-1085.1-v2.0.0-2026-06-07",
        "GARAK-BRIDGE-1099-v1.0.0-2026-06-08",
        "ORCHESTRATOR-v5.1.0-2026-06-08",
    ]
    for seal in seals:
        print(f"  {seal}")
    print(f"{'-' * 76}")

    if Path(dash_path).exists():
        lines = Path(dash_path).read_text(encoding="utf-8").strip().split("\n")
        print(f"\n  Dashboard: {len(lines)} registros em {dash_path}")

if __name__ == "__main__":
    demo_orchestrator_v5_1()
