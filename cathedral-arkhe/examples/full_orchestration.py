from cathedral.orchestrator.v5 import CathedralOrchestratorV5
import tempfile
from pathlib import Path
import numpy as np
from cathedral.types import GGUFHeader
from cathedral.constants import GGUF_MAGIC
from cathedral.substrates.theosis.core import VectorTheosis1092
from cathedral.substrates.stethoscope.core import Stethoscope1081
from cathedral.substrates.kleros.trigger import KlerosTrigger1085
from cathedral.substrates.zkml.bridge import ZKMLBridge1095
from cathedral.substrates.agentic.loop import AgenticLoop1096

def demo_orchestrator_v5():
    print("=" * 80)
    print("  CATHEDRAL ARKHE — ORQUESTRADOR v5.0.0")
    print("  Era Autônoma ZK-Agentica")
    print("  PLAN -> INFER -> ZKML -> STETH -> THEOSIS -> KLEROS -> ANCHOR -> LEARN")
    print("=" * 80)

    test_paths = ["./llama-2-7b.Q4_K_M.gguf", "./tinyllama-1.1b.Q4_K_M.gguf", "./model.gguf"]
    model_path = None
    for p in test_paths:
        if Path(p).exists(): model_path = p; break

    dash_path = tempfile.mktemp(suffix=".jsonl", prefix="cathedral_v5_dash_")
    orch = CathedralOrchestratorV5(model_path=model_path, n_ctx=2048, dashboard_path=dash_path)

    if model_path:
        orch.load_model(model_path)
        prompts = [
            "The horse raced past the barn fell.",
            "Attention is all you need.",
            "Quantum entanglement violates local realism.",
        ]
        for prompt in prompts:
            orch.infer(prompt, max_tokens=15, use_agentic=(prompt == prompts[-1]))
        orch.end_cycle()
    else:
        print("\n  Demonstração simulada completa:")
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
        np.random.seed(42)
        prompts = [
            "The horse raced past the barn",
            "fell",
            ".",
            "The horse raced past the barn and fell down.",
            "Attention is all you need for transformer models.",
            "Quantum entanglement violates local realism.",
        ]
        for prompt in prompts:
            orch.infer(prompt, max_tokens=8, use_agentic=(prompt == prompts[-1]))
        orch.end_cycle()

    # Telemetria final
    print(f"\n{'-' * 76}")
    print(f"  TELEMETRIA FINAL V5")
    print(f"{'-' * 76}")
    telem = orch.get_telemetry()
    print(f"  Orchestrator: {telem['module']} v{telem['version']}")
    print(f"  Ciclos: {telem['cycles']}")
    print(f"  Quarantinado: {telem['quarantined']}")
    if telem['vector_theosis']:
        vt = telem['vector_theosis']
        print(f"  VectorTheosis: dim={vt['dim']}, leituras={vt['n_readings']}")
        print(f"    Stats: {vt.get('stats', {})}")
    if telem['stethoscope']:
        st = telem['stethoscope']
        print(f"  Stethoscope: steps={st['steps']}, anomalias={st['anomalies_total']}")
    if telem['kleros']:
        kl = telem['kleros']
        print(f"  Kleros: casos={kl['total_cases']}, nao resolvidos={kl['unresolved']}")
        print(f"    Distribuicao: {kl['verdict_distribution']}")
    if telem['zkml']:
        zk = telem['zkml']
        print(f"  ZKML: provas={zk['total_proofs']}, verificadas={zk['verified']}")
    if telem['temporal']:
        tc = telem['temporal']
        print(f"  TemporalChain: ancoras={tc['total_anchors']}, batch={tc['pending_batch']}")
    if telem['agentic']:
        ag = telem['agentic']
        print(f"  AgenticLoop: steps={ag['total_steps']}, lessons={ag['lessons_learned']}")

    print(f"\n{'-' * 76}")
    print(f"  SELLOS V5")
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
        "ORCHESTRATOR-v5.0.0-2026-06-07",
    ]
    for seal in seals:
        print(f"  {seal}")
    print(f"{'-' * 76}")

    if Path(dash_path).exists():
        lines = Path(dash_path).read_text(encoding="utf-8").strip().split("\n")
        print(f"\n  Dashboard: {len(lines)} registros em {dash_path}")

if __name__ == "__main__":
    demo_orchestrator_v5()
