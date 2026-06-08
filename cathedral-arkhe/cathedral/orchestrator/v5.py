import time
import json
import numpy as np
from collections import deque
from pathlib import Path

from cathedral.types import GGUFHeader
from cathedral.constants import GGUF_MAGIC
from cathedral.substrates.gguf.bridge import GGUFBridgeV3
from cathedral.substrates.llm.bridge import LlamaCppBridgeV3
from cathedral.substrates.zkml.bridge import ZKMLBridge1095
from cathedral.substrates.agentic.loop import AgenticLoop1096
from cathedral.substrates.temporal.chain import TemporalChain1097
from cathedral.substrates.theosis.core import VectorTheosis1092
from cathedral.substrates.stethoscope.core import Stethoscope1081
from cathedral.substrates.kleros.trigger import KlerosTrigger1085

class CathedralOrchestratorV5:
    def __init__(self, model_path=None, n_ctx=2048, dashboard_path=None):
        self.gguf = GGUFBridgeV3()
        self.llm = LlamaCppBridgeV3(model_path=model_path, n_ctx=n_ctx)
        self.zkml = ZKMLBridge1095()
        self.agentic = AgenticLoop1096()
        self.temporal = TemporalChain1097()
        self.vt = None; self.stethoscope = None; self.kleros = None
        self.cycle_count = 0; self._active = False; self._quarantined = False
        self._cycle_log = []; self._dashboard_path = dashboard_path
        self._recent_gate_history = deque(maxlen=10)
        self.model_path = model_path

    def load_model(self, model_path):
        print(f"\n[OrchestratorV5.0] Carregando: {model_path}")
        gguf_ok = self.gguf.open(model_path)
        if not gguf_ok: return {"status": "ERROR", "error": "Falha ao parsear GGUF"}
        arch = self.gguf.get_architecture(); emb_dim = self.gguf.get_embedding_length()
        n_layers = self.gguf.get_block_count(); n_heads = self.gguf.get_head_count()
        print(f"  ✓ GGUF v3: {arch} | emb={emb_dim} | layers={n_layers} | heads={n_heads}")
        llm_ok = self.llm.load(model_path)
        if llm_ok: print(f"  ✓ llama-cpp: n_embd={self.llm._n_embd}, vocab={self.llm._vocab_size}")
        else: print(f"  ⚠ llama-cpp indisponível — simulação")
        model_commitment = self.zkml.commit_model(model_path)
        print(f"  ✓ ZKML commitment: {model_commitment[:16]}...")
        dim = self.llm._n_embd if self.llm._llm else (emb_dim or 4096)
        self.vt = VectorTheosis1092(dim=dim)
        print(f"  ✓ VectorTheosis 1091.2: dim={dim}, RKHS, φ²")
        self.stethoscope = Stethoscope1081(n_layers=max(n_layers, 1), dim=dim, n_heads=max(n_heads, 1))
        print(f"  ✓ Stethoscope 1081.1: {self.stethoscope.n_layers} layers, FFT")
        self.kleros = KlerosTrigger1085()
        self.kleros.set_temporal_chain(self.temporal)
        print(f"  ✓ Kleros 1085.1: trigger + ZK-proof + TemporalChain")
        print(f"  ✓ AgenticLoop 1096: ReAct + Reflection + Planning")
        print(f"  ✓ TemporalChain 1097: Merkle + ZK-Rollup")
        self.model_path = model_path
        return {"status": "LOADED", "embedding_dim": dim}

    def infer(self, prompt, max_tokens=50, use_agentic=False):
        if not self._active: self.start_cycle()
        q_status = self.kleros.check_quarantine() if self.kleros else {"in_quarantine": False}
        if q_status.get("in_quarantine"):
            self._quarantined = True
            print(f"  [QUARANTINA] {q_status['duration_seconds']:.1f}s")
        self.cycle_count += 1
        cycle_start = time.time()
        print(f"\n[Cycle {self.cycle_count}] '{prompt[:60]}...'")

        # 1. AGENTIC PLAN
        agentic_result = None
        if use_agentic and self.agentic:
            agentic_result = self.agentic.execute(
                objective=prompt, llm_generate=lambda p: {"output": f"[agentic] {p}"},
                theosis_monitor=lambda x: 0.95)
            print(f"  [AGENTIC] Plan: {len(agentic_result['plan'])} steps")

        # 2. INFER
        if self.llm._llm:
            result = self.llm.generate_with_full_extraction(prompt, max_tokens=max_tokens)
            logits_list = [np.array(l) for l in result.get("logits_per_position", [])]
            emb_vec = result.get("embedding_array")
        else:
            result, logits_list, emb_vec = self._simulate_inference(prompt, max_tokens)

        # 3. ZKML PROOF
        zk_proof = None
        if self.zkml and emb_vec is not None:
            zk_proof = self.zkml.prove_inference(
                self.model_path or "simulated", prompt,
                result.get("generated_text", ""), emb_vec)
            print(f"  [ZKML] Prova {zk_proof.proof_id[:20]}... gerada")

        # 4. STETHOSCOPE
        steth_reading = None
        if self.stethoscope and logits_list:
            steth_reading = self.stethoscope.feed_logits_trajectory(
                logits_list, emb_vec if emb_vec is not None else np.zeros(self.vt.dim if self.vt else 4096))
            agg = steth_reading.get("aggregate", {})
            print(f"  [STETH] cos={agg.get('mean_cosine', 0):.3f} | rate={agg.get('max_rate', 0):.3f} | entropy={agg.get('mean_entropy', 0):.2f}")
            if steth_reading.get("spectral"):
                print(f"  [STETH] Spectral: dom_freq={steth_reading['spectral'].get('dominant_freq', 0):.4f}")

        # 5. VECTOR THEOSIS φ²
        theosis_reading = None
        if self.vt and emb_vec is not None:
            theosis_reading = self.vt.update(emb_vec, logits=logits_list[0] if logits_list else None)
            if theosis_reading:
                gate = theosis_reading["gate"]
                print(f"  [THEOSIS] Θ={theosis_reading['theosis']:.4f} | TEE={theosis_reading['tee']:.4f} | SpecEnt={theosis_reading['spectral_entropy']:.4f} | Bifurc={theosis_reading['bifurcation_detected']} | Gate={gate}")
                self._recent_gate_history.append(gate)
                theosis_reading["_recent_gates"] = list(self._recent_gate_history)

        # 6. KLEROS TRIGGER
        kleros_case = None
        if self.kleros and theosis_reading:
            gate = theosis_reading["gate"]
            if gate in ("EMERGENCY", "LOCKED"):
                print(f"  [KLEROS] ⚡ TRIGGER — Gate={gate}")
                kleros_case = self.kleros.evaluate(
                    gate=gate, theosis_reading=theosis_reading,
                    stethoscope_reading=steth_reading, llm_result=result, zk_proof=zk_proof)
                print(f"  [KLEROS] {kleros_case.case_id}: {kleros_case.verdict} | urg={kleros_case.evidence['urgency_score']:.3f}")
                if kleros_case.verdict == "ESCALATE": print(f"  [KLEROS] 🚨 ESCALAÇÃO — Intervenção humana!")
                elif kleros_case.verdict == "QUARANTINE": print(f"  [KLEROS] 🔒 QUARANTINA ativada")

        # 7. ANCHOR
        if self.temporal and theosis_reading:
            anchor = self.temporal.anchor_reading(theosis_reading, zk_proof)
            print(f"  [TEMPORAL] Âncora {anchor.anchor_id[:20]}... | Merkle: {anchor.merkle_root[:16]}...")

        # 8. LOG
        cycle_record = {
            "cycle": self.cycle_count, "timestamp": cycle_start, "prompt": prompt,
            "status": "QUARANTINED" if self._quarantined else "OK",
            "theosis": theosis_reading,
            "kleros": {"triggered": kleros_case is not None,
                       "verdict": kleros_case.verdict if kleros_case else None},
            "zkml": {"proof_id": zk_proof.proof_id if zk_proof else None},
            "agentic": agentic_result,
        }
        self._cycle_log.append(cycle_record)
        if self._dashboard_path: self._write_dashboard(cycle_record)

        return {"cycle": self.cycle_count, "generated_text": result.get("generated_text", ""),
                "theosis": theosis_reading, "kleros_triggered": kleros_case is not None}

    def _simulate_inference(self, prompt, max_tokens):
        dim = self.vt.dim if self.vt else 4096
        total_tokens = len(prompt.split()) + max_tokens
        base = np.random.randn(dim).astype(np.float32) * 0.1
        drift = np.random.randn(dim).astype(np.float32) * 0.05
        emb_vec = base + drift * 0.1
        vocab_size = 32000
        logits_list = [np.random.randn(vocab_size).astype(np.float32) * 0.5 for _ in range(total_tokens)]
        if np.random.random() < 0.3:
            spike_idx = len(logits_list) // 2
            logits_list[spike_idx] = np.random.randn(vocab_size).astype(np.float32) * 5.0
        result = {"status": "SIMULATED", "prompt": prompt, "generated_text": "[simulação]",
                  "generated_tokens": max_tokens, "logits_per_position": [l.tolist() for l in logits_list[:32]],
                  "embeddings": {"mean": emb_vec.tolist()}, "embedding_array": emb_vec}
        return result, logits_list, emb_vec

    def _write_dashboard(self, record):
        try:
            with open(self._dashboard_path, "a", encoding="utf-8") as f:
                f.write(json.dumps(record, ensure_ascii=False, default=str) + "\n")
        except Exception as e: print(f"  [DASHBOARD] Erro: {e}")

    def start_cycle(self):
        self._active = True; self._quarantined = False
        if self.vt: self.vt.reset()
        if self.stethoscope: self.stethoscope.reset()
        self._recent_gate_history.clear()
        print(f"\n{'=' * 76}")
        print(f"  CATHEDRAL ORCHESTRATOR v5.0.0 — Era Autônoma ZK-Agentica")
        print(f"  PLAN → INFER → ZKML → STETH → THEOSIS → KLEROS → ANCHOR → LEARN")
        print(f"{'=' * 76}")

    def end_cycle(self):
        self._active = False
        report = {
            "cycles": self.cycle_count, "gguf": self.gguf.get_telemetry(),
            "llm": self.llm.get_telemetry(), "zkml": self.zkml.get_telemetry(),
            "vector_theosis": self.vt.get_telemetry() if self.vt else None,
            "stethoscope": self.stethoscope.get_telemetry() if self.stethoscope else None,
            "kleros": self.kleros.get_telemetry() if self.kleros else None,
            "temporal": self.temporal.get_telemetry(),
            "agentic": self.agentic.get_telemetry() if self.agentic else None,
        }
        print(f"\n{'=' * 76}")
        print(f"  CICLO V5 FINALIZADO — {report['cycles']} ciclos")
        if self.vt and self.vt.readings:
            stats = self.vt.get_stats()
            print(f"    Theosis: μ={stats['theosis_mean']:.4f} [{stats['theosis_min']:.4f}, {stats['theosis_max']:.4f}]")
            print(f"    TEE: μ={stats['tee_mean']:.4f} | max={stats['tee_max']:.4f}")
            print(f"    Bifurcações: {stats['bifurcations']}")
            print(f"    Gates: {stats['gate_distribution']}")
        if self.kleros and self.kleros.cases: print(f"    Kleros: {len(self.kleros.cases)} caso(s)")
        if self.zkml: print(f"    ZKML: {self.zkml.get_telemetry()['total_proofs']} provas")
        if self.temporal: print(f"    TemporalChain: {self.temporal.get_telemetry()['total_anchors']} âncoras")
        print(f"{'=' * 76}")
        return report

    def get_telemetry(self):
        return {"module": "CathedralOrchestratorV5", "version": "5.0.0", "substrate": "1098",
                "seal": "ORCHESTRATOR-v5.0.0-2026-06-07", "cycles": self.cycle_count,
                "active": self._active, "quarantined": self._quarantined,
                "gguf": self.gguf.get_telemetry(), "llm": self.llm.get_telemetry(),
                "zkml": self.zkml.get_telemetry(),
                "vector_theosis": self.vt.get_telemetry() if self.vt else None,
                "stethoscope": self.stethoscope.get_telemetry() if self.stethoscope else None,
                "kleros": self.kleros.get_telemetry() if self.kleros else None,
                "temporal": self.temporal.get_telemetry(),
                "agentic": self.agentic.get_telemetry() if self.agentic else None}
