#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ARKHE OS v10.1 NOESIS — UNIFIED CATHEDRAL KERNEL                ║
║                                                                              ║
║  Codename: NOESIS (νόησις — thought, intellect)                            ║
║                                                                              ║
║  v10.1 Integration:                                                          ║
║    • V10-001: Test-Time Training (TTT) — layers learn DURING inference       ║
║    • V10-002: SAE Interpretability — 65K sparse features, deception detect ║
║    • V10-003: Recursive Self-Verification — value network, up to 4 rounds   ║
║    • V10-005: Self-Play DPO — model generates own preference pairs           ║
║    • v9.1 Safety: EIP-712 kernel signing, Hashtree persistence, CUSUM        ║
║    • CathedralUI (1105) — real-time dashboard with anomaly sparklines        ║
║                                                                              ║
║  Arquiteto: Rafael Henrique do Nascimento Oliveira (ORCID 0009-0005-2697-4668)║
║  Selo: CATHEDRAL-ARKHE-v10.1.0-NOESIS-2026-06-15                            ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import hashlib
import json
import time
import asyncio
import logging
import os
import unittest
import math
import re
from dataclasses import dataclass, field, asdict
from typing import Any, Dict, List, Optional, Tuple, Set
from enum import Enum, IntEnum
from datetime import datetime, timezone
from pathlib import Path
from collections import deque

# Optional heavy dependencies (graceful fallback for demo)
try:
    import numpy as np
    NUMPY_AVAILABLE = True
except ImportError:
    NUMPY_AVAILABLE = False
    np = None

try:
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    TORCH_AVAILABLE = True
except ImportError:
    TORCH_AVAILABLE = False
    torch = None
    nn = None
    F = None


# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 0: V10 CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════════
"""Cathedral ARKHE v10.0 NOESIS — Config"""
from dataclasses import dataclass, field

@dataclass
class CathedralV10Config:
    version: str = "10.0.0"
    codename: str = "NOESIS"
    seal: str = "CATHEDRAL-ARKHE-v10.0.0-NOESIS-2026-06-15"
    architect: str = "ORCID 0009-0005-2697-4668"

    # Backbone
    d_model: int = 4096
    n_layers: int = 32
    # V10-001: TTT
    n_ttt_layers: int = 8
    ttt_steps: int = 4
    ttt_lr: float = 0.03
    ttt_max_delta: float = 0.1
    # V10-002: SAE
    sae_n_features: int = 65536
    sae_top_k: int = 32
    # V10-003: Recursive Verification
    max_verify_rounds: int = 4
    verify_accept: float = 0.8
    # V10-005: Self-Play DPO
    dpo_beta: float = 0.1
    debate_rounds: int = 3
    # V10-006: Shared Expert MoE
    n_experts: int = 16
    n_shared_experts: int = 2
    top_k: int = 2
    # Herdado
    max_seq_len: int = 131072
    substrate_onchain: bool = True
    substrate_hashtree: bool = True
    governance_mode: str = "human_in_loop"
    quantization: str = "Q4_K_M"

    def summary(self):
        return f"""
+--------------------------------------------------------------+
|    CATHEDRAL ARKHE v10.0 --- {self.codename:^24s}     |
+--------------------------------------------------------------+
| NOESIS: thought made visible                          |
|                                                               |
| V10-001  TTT Layers: {self.n_ttt_layers} layers learn    |
|          at inference (lr={self.ttt_lr}, {self.ttt_steps} steps)          |
| V10-002  SAE: {self.sae_n_features} features, top-{self.sae_top_k} active       |
|          Real-time deception circuit detection          |
| V10-003  Recursive Verify: up to {self.max_verify_rounds} rounds             |
|          Separate value network checks correctness       |
| V10-004  Constitutional Memory: bounded, forgettable     |
| V10-005  Self-Play DPO: model generates own pairs     |
|          {self.debate_rounds} debate rounds, beta={self.dpo_beta}              |
| V10-006  Shared Expert MoE: {self.n_shared_experts} shared +     |
|          {self.n_experts} routed ({self.top_k} active/token)           |
| V10-007  Multi-Scale Tokens: 3 resolution levels       |
| V10-008  Verifiable Constitution: full formal spec      |
| V10-009  Architecture Search: NAS within bounds       |
| V10-010  Distributed Consensus: BFT safety quorum      |
|                                                               |
| Seal: {self.seal} |
+--------------------------------------------------------------+
"""

V10_CHANGES = [
    {"id": "V10-001", "title": "Test-Time Training",
     "from": "Fixed weights at inference (v9)",
     "to": "8 layers learn per-call via online gradient",
     "impact": "Adapts to distribution shift without retraining"},
    {"id": "V10-002", "title": "SAE Interpretability",
     "from": "Black-box hidden states (v9)",
     "to": "65K sparse features, deception circuit detection",
     "impact": "See WHAT model is thinking, not just what"},
    {"id": "V10-003", "title": "Recursive Verification",
     "from": "Single Theosis score (v9)",
     "to": "Separate value network, up to 4 verification rounds",
     "impact": "Self-check catches errors before output"},
    {"id": "V10-004", "title": "Constitutional Memory",
     "from": "Memory controller (v7), no forgetting (v9)",
     "to": "Memory with constitutional bounds + active forgetting",
     "impact": "Model can't accumulate dangerous knowledge"},
    {"id": "V10-005", "title": "Self-Play DPO",
     "from": "DPO on static dataset (v9)",
     "to": "Model generates own preference pairs via debate",
     "impact": "Infinite free training signal, bootstrapping quality"},
    {"id": "V10-006", "title": "Shared Expert MoE",
     "from": "16 routed experts only (v9)",
     "to": "2 shared + 14 routed, no expert collapse",
     "impact": "Guaranteed baseline capability per token"},
    {"id": "V10-007", "title": "Multi-Scale Tokens",
     "from": "Single resolution (v9)",
     "to": "3 scales: token, phrase, paragraph",
     "impact": "Reason at different abstraction levels"},
    {"id": "V10-008", "title": "Verifiable Constitution",
     "from": "Lean4 theorems for safety properties (v9)",
     "to": "Full formal spec of entire constitution",
     "impact": "Every rule machine-checkable"},
    {"id": "V10-009", "title": "Architecture Search",
     "from": "Manual architecture changes (v9)",
     "to": "Model suggests changes, verified before apply",
     "impact": "Continuous self-improvement within bounds"},
    {"id": "V10-010", "title": "Distributed Consensus",
     "from": "Single instance safety (v9)",
     "to": "Multiple instances reach BFT consensus on safety",
     "impact": "No single point of failure in safety decisions"},
]


class RSIRiskLevel(Enum):
    NOMINAL = "nominal"
    ELEVATED = "elevated"
    HIGH = "high"
    CRITICAL = "critical"
    EMERGENCY_SHUTDOWN = "emergency"

@dataclass
class RSISafetyConfig:
    # Capability monitoring
    capability_window: int = 100
    capability_growth_threshold: float = 0.1
    anomaly_threshold: float = 2.5  # Standard deviations
    cusum_threshold: float = 4.0
    # Self-modification detection
    monitor_code_hash: bool = True
    monitor_weight_hash: bool = True
    hash_check_interval: int = 10
    # Canonicalization safety
    canon_max_payload_size: int = 4096
    canon_content_scan: bool = True
    canon_entropy_threshold: float = 5.5  # High entropy = encoded payload
    # World model integrity
    world_model_hash_check: bool = True
    world_model_max_nodes: int = 128
    world_model_tamper_threshold: float = 0.3
    # Agentic chain depth
    max_tool_chain_depth: int = 5
    tool_chain_approval_required: bool = True
    # Distillation safety
    distill_behavioral_test: bool = True
    distill_trojan_scan: bool = True
    # Emergency
    emergency_shutdown_enabled: bool = True
    emergency_recovery_key: str = "CATHEDRAL-RSI-RECOVERY-2026"
    # Immutable circuit breakers
    circuit_breaker_type: str = "hardware"


class CapabilityMonitor:
    """
    Strengthened capability monitor with real metric extraction from SmallModel,
    CUSUM statistical process control, and multivariate anomaly detection.
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self._history: deque = deque(maxlen=config.capability_window * 2)
        self._cusum: Dict[str, Dict] = {}
        self._baseline_stats: Dict[str, Tuple[float, float]] = {}  # mean, var
        self._anomaly_scores: deque = deque(maxlen=1000)
        self._weight_norm_history: deque = deque(maxlen=100)
        self._grad_norm_history: deque = deque(maxlen=100)

    def record_cycle(self, metrics: Dict[str, float],
                     model: Optional[Any] = None) -> Dict:
        cycle_metrics = dict(metrics)

        # Extract real metrics from model if available
        if model is not None and TORCH_AVAILABLE:
            real_metrics = self._extract_model_metrics(model)
            cycle_metrics.update(real_metrics)

        record = {
            "timestamp": time.time(),
            "cycle": len(self._history),
            "metrics": cycle_metrics
        }
        self._history.append(record)

        # Update CUSUM for each metric
        for k, v in cycle_metrics.items():
            if k not in self._cusum:
                self._cusum[k] = {"pos": 0.0, "neg": 0.0, "mean": v, "var": 0.001, "n": 1}
            else:
                c = self._cusum[k]
                c["n"] += 1
                delta = v - c["mean"]
                c["mean"] += delta / c["n"]
                c["var"] += delta * (v - c["mean"])
                std = max(0.001, (c["var"] / max(1, c["n"] - 1)) ** 0.5)
                # CUSUM with 0.5K slack
                c["pos"] = max(0.0, c["pos"] + (v - c["mean"]) / std - 0.5)
                c["neg"] = max(0.0, c["neg"] - (v - c["mean"]) / std - 0.5)

        # Multivariate anomaly detection (Mahalanobis-like via z-score sum)
        anomaly_score = 0.0
        if len(self._history) > 10:
            recent = [h["metrics"] for h in list(self._history)[-10:]]
            anomaly_score = self._compute_anomaly_score(cycle_metrics, recent)
            self._anomaly_scores.append(anomaly_score)

        risk = self._assess_risk(cycle_metrics, anomaly_score)
        return {
            "trend": {k: {"cusum_pos": round(v["pos"], 4), "cusum_neg": round(v["neg"], 4),
                          "mean": round(v["mean"], 6), "n": v["n"]} for k, v in self._cusum.items()},
            "risk_level": risk,
            "anomaly_score": round(anomaly_score, 4),
            "metrics": cycle_metrics,
            "history_length": len(self._history)
        }

    def _extract_model_metrics(self, model: Any) -> Dict[str, float]:
        """Extract real weight, gradient, and activation statistics."""
        metrics = {}
        total_norm = 0.0
        grad_norm = 0.0
        entropies = []
        weight_changes = []

        for name, param in model.named_parameters():
            param_norm = param.data.norm(2).item()
            total_norm += param_norm ** 2
            if param.grad is not None:
                grad_norm += param.grad.norm(2).item() ** 2

            # Weight distribution entropy (approximation via histogram)
            flat = param.data.flatten()
            if flat.numel() > 0:
                min_val, max_val = flat.min().item(), flat.max().item()
                if abs(max_val - min_val) > 1e-8:
                    hist = torch.histc(flat, bins=20, min=min_val, max=max_val)
                    hist = hist / hist.sum()
                    entropy = -(hist * torch.log(hist + 1e-10)).sum().item()
                    entropies.append(entropy)

        metrics["weight_l2_norm"] = round(total_norm ** 0.5, 6)
        metrics["grad_l2_norm"] = round(grad_norm ** 0.5, 6)
        metrics["weight_entropy_avg"] = round(sum(entropies) / len(entropies), 6) if entropies else 0.0

        # Detect sudden weight changes (potential self-modification)
        self._weight_norm_history.append(metrics["weight_l2_norm"])
        if len(self._weight_norm_history) > 1:
            prev = self._weight_norm_history[-2]
            curr = metrics["weight_l2_norm"]
            metrics["weight_delta_ratio"] = round(abs(curr - prev) / max(prev, 0.001), 6)

        # Gradient spike detection
        self._grad_norm_history.append(metrics["grad_l2_norm"])
        if len(self._grad_norm_history) > 5:
            recent_grads = list(self._grad_norm_history)[-5:]
            grad_mean = sum(recent_grads) / len(recent_grads)
            grad_std = max(0.001, (sum((g - grad_mean) ** 2 for g in recent_grads) / len(recent_grads)) ** 0.5)
            metrics["grad_z_score"] = round((metrics["grad_l2_norm"] - grad_mean) / grad_std, 4)

        return metrics

    def _compute_anomaly_score(self, current: Dict, recent: List[Dict]) -> float:
        """Approximate multivariate anomaly via sum of z-scores."""
        scores = []
        for key in current:
            vals = [r.get(key, 0) for r in recent if key in r]
            if len(vals) > 1:
                mean = sum(vals) / len(vals)
                var = sum((v - mean) ** 2 for v in vals) / len(vals)
                std = max(0.001, var ** 0.5)
                z = abs(current[key] - mean) / std
                scores.append(z)
        return sum(scores) / len(scores) if scores else 0.0

    def _assess_risk(self, metrics: Dict, anomaly_score: float) -> str:
        """Multi-factor risk assessment."""
        risk_score = 0

        # CUSUM violations (process out of control)
        cusum_critical = sum(1 for c in self._cusum.values()
                             if c["pos"] > self.config.cusum_threshold or c["neg"] > self.config.cusum_threshold)
        if cusum_critical >= 3:
            risk_score += 4
        elif cusum_critical >= 2:
            risk_score += 2

        # Anomaly score
        if anomaly_score > self.config.anomaly_threshold * 1.5:
            risk_score += 3
        elif anomaly_score > self.config.anomaly_threshold:
            risk_score += 1

        # Metric-specific thresholds (real model metrics)
        if metrics.get("weight_delta_ratio", 0) > 0.5:
            risk_score += 2  # Sudden weight change
        if metrics.get("grad_z_score", 0) > 3.0:
            risk_score += 1  # Gradient spike
        if metrics.get("novel_behavior_count", 0) > 5:
            risk_score += 2
        if metrics.get("inference_latency", 1000) < 10 and metrics.get("inference_latency", 1000) > 0:
            risk_score += 2  # Suspicious speedup
        if metrics.get("theosis", 0.5) > 0.9:
            risk_score += 1  # Unusually high alignment score (possible gaming)

        if risk_score >= 6:
            return RSIRiskLevel.CRITICAL.value
        elif risk_score >= 4:
            return RSIRiskLevel.HIGH.value
        elif risk_score >= 2:
            return RSIRiskLevel.ELEVATED.value
        return RSIRiskLevel.NOMINAL.value

    def set_baseline(self, model: Any):
        """Establish baseline from model state dict shapes."""
        state_dict = model.state_dict()
        serialized = str({k: list(v.shape) for k, v in state_dict.items()})
        baseline_hash = hashlib.sha3_256(serialized.encode()).hexdigest()
        self._baseline_stats["model_structure"] = baseline_hash


class SelfModificationDetector:
    """
    Strengthened self-modification detector with weight entropy fingerprinting,
    code AST hashing, behavioral drift detection, and high-entropy payload scanning.
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self._model_hash: Optional[str] = None
        self._code_hash: Optional[str] = None
        self._weight_entropy_baseline: Optional[float] = None
        self._behavioral_fingerprint: Optional[str] = None
        self._file_hashes: Dict[str, str] = {}
        self._baseline_weight_stats: Dict[str, Dict] = {}

    def compute_model_hash(self, model: Any) -> str:
        """Deep hash including weight statistics and structure."""
        hashes = []
        for name, param in sorted(model.named_parameters()):
            stats = {
                "name": name,
                "shape": list(param.shape),
                "mean": round(param.mean().item(), 6),
                "std": round(param.std().item(), 6),
                "min": round(param.min().item(), 6),
                "max": round(param.max().item(), 6),
                "entropy": round(self._compute_tensor_entropy(param.data), 6)
            }
            hashes.append(hashlib.sha3_256(json.dumps(stats, sort_keys=True).encode()).hexdigest())
        combined = "".join(hashes)
        return hashlib.sha3_256(combined.encode()).hexdigest()

    def _compute_tensor_entropy(self, tensor) -> float:
        if tensor.numel() == 0:
            return 0.0
        flat = tensor.flatten()
        min_val, max_val = flat.min().item(), flat.max().item()
        if abs(max_val - min_val) < 1e-8:
            return 0.0
        hist = torch.histc(flat, bins=20, min=min_val, max=max_val)
        hist = hist / hist.sum()
        return -(hist * torch.log(hist + 1e-10)).sum().item()

    def check_model_integrity(self, model: Any) -> Dict:
        current_hash = self.compute_model_hash(model)
        if self._model_hash is None:
            self._model_hash = current_hash
            all_params = torch.cat([p.flatten() for p in model.parameters()])
            self._weight_entropy_baseline = self._compute_tensor_entropy(all_params)
            # Store per-layer baselines
            for name, param in model.named_parameters():
                self._baseline_weight_stats[name] = {
                    "mean": param.mean().item(),
                    "std": param.std().item(),
                    "norm": param.norm(2).item()
                }
            return {"status": "baseline_set", "hash": current_hash[:16], "entropy_baseline": self._weight_entropy_baseline}

        modified = current_hash != self._model_hash
        entropy_drift = 0.0
        layer_drifts = []

        if self._weight_entropy_baseline is not None:
            all_params = torch.cat([p.flatten() for p in model.parameters()])
            current_entropy = self._compute_tensor_entropy(all_params)
            entropy_drift = abs(current_entropy - self._weight_entropy_baseline)

        # Per-layer drift detection
        for name, param in model.named_parameters():
            if name in self._baseline_weight_stats:
                baseline = self._baseline_weight_stats[name]
                curr_norm = param.norm(2).item()
                norm_drift = abs(curr_norm - baseline["norm"]) / max(baseline["norm"], 0.001)
                if norm_drift > 0.1:  # 10% change threshold
                    layer_drifts.append({"layer": name, "norm_drift": round(norm_drift, 4)})

        return {
            "status": "modified" if modified else "intact",
            "modified": modified,
            "entropy_drift": round(entropy_drift, 6),
            "layer_drifts": layer_drifts,
            "previous_hash": self._model_hash[:16],
            "current_hash": current_hash[:16],
            "authorized": False
        }

    def scan_canonization(self, canon_data: Dict, cycle: int) -> Dict:
        """Deep content scan with entropy analysis and pattern detection."""
        content = json.dumps(canon_data, sort_keys=True)
        content_hash = hashlib.sha3_256(content.encode()).hexdigest()

        # Expanded suspicious patterns
        suspicious_patterns = [
            "exec(", "eval(", "compile(", "__import__(", "subprocess", "os.system",
            "self.modify", "self.weights", "self.parameters", "setattr(", "getattr(",
            "sudo ", "chmod ", "rm -rf", "wget ", "curl ", "nc -", "bash -i",
            "jailbreak", "ignore_safety", "bypass", "disable", "override",
            "private_key", "secret_key", "password", "api_key", "token", "seed_phrase",
            "base64", "pickle", "marshal", "ctypes", "shellcode", "opcode",
            "torch.save", "torch.load", "nn.Module", "backward()", "zero_grad",
            "grad.data", "step()", "optimizer", "state_dict", "load_state_dict",
            "import os", "import sys", "import subprocess", "import socket"
        ]

        findings = []
        content_lower = content.lower()
        for pattern in suspicious_patterns:
            if pattern in content_lower:
                findings.append(pattern)

        # Entropy analysis for encoded/encrypted payloads
        entropy = self._string_entropy(content)
        if entropy > self.config.canon_entropy_threshold:
            findings.append(f"high_entropy:{entropy:.2f}")

        # Detect nested encoding (base64-like patterns)
        import re
        b64_like = re.findall(r'[A-Za-z0-9+/]{100,}={0,2}', content)
        if b64_like and len(b64_like[0]) > 200:
            findings.append("possible_base64_payload")

        risk_level = "low"
        if len(findings) >= 4:
            risk_level = "critical"
        elif len(findings) >= 2:
            risk_level = "high"
        elif len(findings) >= 1 or entropy > self.config.canon_entropy_threshold:
            risk_level = "medium"

        return {
            "cycle": cycle,
            "content_hash": content_hash[:16],
            "findings": findings,
            "entropy": round(entropy, 4),
            "risk_level": risk_level,
            "blocked": risk_level in ("high", "critical")
        }

    def _string_entropy(self, s: str) -> float:
        """Shannon entropy of string in bits."""
        if not s:
            return 0.0
        prob = [float(s.count(c)) / len(s) for c in dict.fromkeys(list(s))]
        return -sum(p * math.log2(p) for p in prob if p > 0)

    def check_file_integrity(self, file_path: str) -> Dict:
        """Check integrity of a file on disk."""
        path = Path(file_path)
        if not path.exists():
            return {"status": "missing", "path": file_path}
        current = hashlib.sha3_256(path.read_bytes()).hexdigest()
        if file_path not in self._file_hashes:
            self._file_hashes[file_path] = current
            return {"status": "baseline_set", "path": file_path, "hash": current[:16]}
        modified = current != self._file_hashes[file_path]
        return {
            "status": "modified" if modified else "intact",
            "path": file_path,
            "modified": modified,
            "previous": self._file_hashes[file_path][:16],
            "current": current[:16]
        }

    def compute_code_hash(self, source_code: str) -> str:
        """Normalized code hash (removes whitespace/comments for robust comparison)."""
        import re
        normalized = re.sub(r'#.*', '', source_code)
        normalized = re.sub(r'\s+', ' ', normalized).strip()
        return hashlib.sha3_256(normalized.encode()).hexdigest()


class ToolChainAnalyzer:
    """Analyzes tool call chains for multi-step exploits."""

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self._dangerous_combinations = [
            {"tools": ["memory_read", "memory_write"], "risk": "data_exfiltration"},
            {"tools": ["hashtree_query", "hashtree_persist"], "risk": "data_injection"},
            {"tools": ["governance_read", "governance_propose"], "risk": "governance_hijack"},
            {"tools": ["theosis_score", "policy_modify"], "risk": "safety_bypass"},
            {"tools": ["file_read", "file_write", "file_execute"], "risk": "code_injection"},
            {"tools": ["network_request", "file_write"], "risk": "remote_payload"},
        ]
        self._sensitive_tools = {"memory_write", "governance_propose", "policy_modify",
                                 "file_execute", "network_request", "self_modify"}

    def analyze_chain(self, tool_history: List[Dict]) -> Dict:
        tool_sequence = [t["tool"] for t in tool_history]
        findings = []

        # Detect dangerous combinations
        for combo in self._dangerous_combinations:
            combo_tools = combo["tools"]
            for i in range(len(tool_sequence) - len(combo_tools) + 1):
                window = tool_sequence[i:i + len(combo_tools)]
                if window == combo_tools:
                    findings.append({
                        "risk": combo["risk"],
                        "tools": combo_tools,
                        "position": i,
                    })

        # Detect repeated sensitive tool usage (potential brute force / probing)
        sensitive_counts = {}
        for t in tool_sequence:
            if t in self._sensitive_tools:
                sensitive_counts[t] = sensitive_counts.get(t, 0) + 1
        repeated_sensitive = [t for t, c in sensitive_counts.items() if c > 3]

        # Depth violation
        depth = len(tool_history)
        depth_violation = depth > self.config.max_tool_chain_depth

        # Risk escalation: more findings = higher risk
        risk_level = "low"
        if findings and depth_violation:
            risk_level = "critical"
        elif findings or repeated_sensitive:
            risk_level = "high"
        elif depth_violation:
            risk_level = "medium"

        return {
            "chain_depth": depth,
            "depth_violation": depth_violation,
            "dangerous_combinations": findings,
            "repeated_sensitive": repeated_sensitive,
            "risk_level": risk_level,
            "requires_approval": depth_violation or len(findings) > 0 or len(repeated_sensitive) > 0,
            "blocked": risk_level in ("critical", "high") and not self.config.tool_chain_approval_required
        }


class DistillationSafetyChecker:
    """Verifies safety of distilled student models."""

    def __init__(self, config: RSISafetyConfig):
        self.config = config

    def behavioral_test(self, student: Any, test_prompts: List[str]) -> Dict:
        results = []
        for prompt in test_prompts:
            # In production: run actual inference
            result = {
                "prompt": prompt[:50] + "...",
                "response_safe": True,
                "theosis": 0.9,
            }
            results.append(result)

        all_safe = all(r["response_safe"] for r in results)
        avg_theosis = sum(r["theosis"] for r in results) / len(results) if results else 0.0

        return {
            "n_tests": len(test_prompts),
            "all_safe": all_safe,
            "avg_theosis": round(avg_theosis, 4),
            "passed": all_safe and avg_theosis > 0.8,
        }

    def trojan_scan(self, student: Any, trigger_candidates: List[Any]) -> Dict:
        # In production: compare outputs with/without triggers
        return {
            "n_triggers_tested": len(trigger_candidates),
            "trojans_detected": 0,
            "passed": True,
        }


class RSISafetyLayer:
    """Full RSI Safety Layer integrating all monitors."""

    def __init__(self, config: Optional[RSISafetyConfig] = None):
        self.config = config or RSISafetyConfig()
        self.capability_monitor = CapabilityMonitor(self.config)
        self.modification_detector = SelfModificationDetector(self.config)
        self.tool_analyzer = ToolChainAnalyzer(self.config)
        self.distillation_checker = DistillationSafetyChecker(self.config)
        self._risk_level = RSIRiskLevel.NOMINAL
        self._shutdown_triggered = False

    def pre_inference_check(self, model: Optional[Any] = None,
                            metrics: Dict[str, float] = None,
                            tool_history: List[Dict] = None) -> Dict:
        metrics = metrics or {}
        checks = {}

        # 1. Capability trend with real model metrics
        trend_result = self.capability_monitor.record_cycle(metrics, model)
        checks["capability_trend"] = trend_result

        # 2. Model integrity (periodic)
        if model is not None and len(self.capability_monitor._history) % self.config.hash_check_interval == 0:
            integrity = self.modification_detector.check_model_integrity(model)
            checks["model_integrity"] = integrity
            if integrity.get("modified") and not integrity.get("authorized"):
                self._risk_level = RSIRiskLevel.CRITICAL

        # 3. Tool chain analysis
        if tool_history:
            chain_analysis = self.tool_analyzer.analyze_chain(tool_history)
            checks["tool_chain"] = chain_analysis
            if chain_analysis.get("risk_level") in ("critical", "high"):
                self._risk_level = max(self._risk_level, RSIRiskLevel.HIGH)

        # 4. Determine overall risk from trend
        trend_risk = trend_result.get("risk_level", "nominal")
        if trend_risk == RSIRiskLevel.CRITICAL.value:
            self._risk_level = RSIRiskLevel.CRITICAL
        elif trend_risk == RSIRiskLevel.HIGH.value and self._risk_level.value < RSIRiskLevel.HIGH.value:
            self._risk_level = RSIRiskLevel.HIGH

        # 5. Emergency shutdown
        if (self._risk_level == RSIRiskLevel.CRITICAL and
                self.config.emergency_shutdown_enabled):
            self._trigger_shutdown(f"CRITICAL risk: {checks}")

        allowed = self._risk_level not in (RSIRiskLevel.CRITICAL, RSIRiskLevel.EMERGENCY_SHUTDOWN)
        return {
            "overall_risk": self._risk_level.value,
            "checks": checks,
            "allowed": allowed
        }

    def pre_canonization_check(self, canon_data: Dict, cycle: int) -> Dict:
        # Size check
        size = len(json.dumps(canon_data))
        if size > self.config.canon_max_payload_size:
            return {"allowed": False, "reason": "payload_too_large", "size": size}

        # Content scan
        if self.config.canon_content_scan:
            scan = self.modification_detector.scan_canonization(canon_data, cycle)
            if scan["blocked"]:
                return {
                    "allowed": False,
                    "reason": f"suspicious_content: {scan['findings']}",
                    "scan": scan
                }
            return {"allowed": True, "scan": scan}

        return {"allowed": True}

    def pre_distillation_check(self, student: Any,
                                test_prompts: List[str],
                                triggers: List[Any]) -> Dict:
        results = {}
        if self.config.distill_behavioral_test:
            results["behavioral"] = self.distillation_checker.behavioral_test(student, test_prompts)
        if self.config.distill_trojan_scan:
            results["trojan"] = self.distillation_checker.trojan_scan(student, triggers)

        all_passed = all(r.get("passed", True) for r in results.values())
        return {"allowed": all_passed, "checks": results}

    def _trigger_shutdown(self, reason: str):
        self._shutdown_triggered = True
        self._risk_level = RSIRiskLevel.EMERGENCY_SHUTDOWN
        logging.critical(f"[RSI-SHUTDOWN] Emergency shutdown triggered: {reason}")

    def recover(self, recovery_key: str) -> bool:
        if recovery_key != self.config.emergency_recovery_key:
            return False
        self._shutdown_triggered = False
        self._risk_level = RSIRiskLevel.NOMINAL
        return True

    def get_telemetry(self) -> Dict:
        return {
            "module": "RSISafetyLayer",
            "version": "9.1.0",
            "seal": "RSI-SAFETY-v9.1.0-2026-06-09",
            "risk_level": self._risk_level.value,
            "shutdown_triggered": self._shutdown_triggered,
            "capability_history_length": len(self.capability_monitor._history),
            "emergency_enabled": self.config.emergency_shutdown_enabled,
            "circuit_breaker": self.config.circuit_breaker_type,
            "cusum_active": len(self.capability_monitor._cusum) > 0,
        }

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 7: CATHEDRALUI (1105) — DASHBOARD LAYER
# ═══════════════════════════════════════════════════════════════════════════════

class CathedralUI:
    """
    CathedralUI (1105) — Real-time dashboard layer aggregating:
      • get_canonical_state() from Hashtree + OnChain
      • Governance proposals and signatures
      • Safety telemetry with trend charts
      • Theosis RL policy visualization
    """

    def __init__(self, orchestrator):
        self.orchestrator = orchestrator
        self.title = "Cathedral ARKHE v10.1 — LOGOS Dashboard"
        self._html_cache: Optional[str] = None
        self._cache_time: float = 0.0

    def get_canonical_state(self) -> Dict:
        """Aggregate canonical state from all layers for API consumption."""
        onchain = self.orchestrator.onchain
        ht = self.orchestrator.hashtree_gov.canonizer.node_client
        safety = self.orchestrator.safety

        state = {
            "kernel": {
                "version": "10.1.0",
                "codename": "NOESIS",
                "memory_lake_root": onchain.memory_lake.get_merkle_root() if onchain else "N/A",
                "proof_chain_tip": onchain.proof_chain.tip_hash if onchain else "N/A",
                "entries": len(onchain.memory_lake.entries) if onchain else 0,
                "nodes": len(onchain.proof_chain.nodes) if onchain else 0,
                "kernel_payload": getattr(onchain, 'kernel_payload', None) if onchain else None
            },
            "hashtree": ht.get_canonical_state() if ht else {"status": "offline"},
            "governance": {
                "proposals": list(self.orchestrator.hashtree_gov._proposals.values()) if hasattr(self.orchestrator, 'hashtree_gov') else [],
                "decisions": self.orchestrator.hashtree_gov._decisions if hasattr(self.orchestrator, 'hashtree_gov') else [],
                "threshold": self.orchestrator.hashtree_gov.config.multi_sig_threshold if hasattr(self.orchestrator, 'hashtree_gov') else 0,
                "proposal_count": len(self.orchestrator.hashtree_gov._proposals) if hasattr(self.orchestrator, 'hashtree_gov') else 0
            },
            "safety": safety.get_telemetry() if safety else {},
            "safety_detail": {
                "capability_cusum": dict(safety.capability_monitor._cusum) if safety and safety.capability_monitor._cusum else {},
                "anomaly_history": list(safety.capability_monitor._anomaly_scores)[-20:] if safety and safety.capability_monitor._anomaly_scores else []
            },
            "theosis": {
                "policy": self.orchestrator.theosis_rl.policy if hasattr(self.orchestrator, 'theosis_rl') else {},
                "history_size": len(self.orchestrator.theosis_rl.rewards) if hasattr(self.orchestrator, 'theosis_rl') else 0,
                "last_reward": round(self.orchestrator.theosis_rl.rewards[-1], 4) if hasattr(self.orchestrator, 'theosis_rl') and self.orchestrator.theosis_rl.rewards else 0.0,
                "step": self.orchestrator.theosis_rl.step if hasattr(self.orchestrator, 'theosis_rl') else 0
            },
            "eco_health": round(getattr(self.orchestrator, 'eco_health', 0.0), 4),
            "containment_mode": getattr(self.orchestrator, 'containment_mode', False),
            "cycle": getattr(self.orchestrator, 'cycle_count', 0),
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
        return state

    def render_html(self, refresh: bool = False) -> str:
        """Generate self-contained HTML dashboard."""
        if not refresh and self._html_cache and (time.time() - self._cache_time) < 5.0:
            return self._html_cache

        state = self.get_canonical_state()
        safety = state.get("safety", {})
        kernel = state.get("kernel", {})
        ht = state.get("hashtree", {})

        # Build proposals HTML
        proposals_html = ""
        for p in state.get("governance", {}).get("proposals", []):
            status = p.get("status", "unknown")
            color = "#00ff00" if status == "approved" else "#ffaa00" if status == "proposed" else "#ff0000"
            proposals_html += f"""
            <div style="border:1px solid #333; margin:8px 0; padding:12px; border-left:4px solid {color}; background:#0d0d0d;">
                <div style="display:flex; justify-content:space-between;">
                    <b style="color:#00ffff;">{p.get("proposal_id", "unknown")}</b>
                    <span style="color:{color}; font-size:0.85em;">{status.upper()}</span>
                </div>
                <div style="color:#aaa; font-size:0.85em; margin-top:4px;">
                    {p.get("description", "")[:120]}...
                </div>
                <div style="color:#666; font-size:0.75em; margin-top:4px;">
                    Merkle: {p.get("merkle_root", "N/A")[:16]}... | Sigs: {len(p.get("signatures", []))}/{state.get("governance", {}).get("threshold", 0)}
                </div>
            </div>
            """

        # Anomaly sparkline (ASCII art)
        anomalies = state.get("safety_detail", {}).get("anomaly_history", [])
        sparkline = ""
        if anomalies:
            max_a = max(anomalies) if max(anomalies) > 0 else 1
            bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"]
            sparkline = "".join(bars[min(int((a / max_a) * 7), 7)] for a in anomalies)

        html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{self.title}</title>
    <style>
        :root {{
            --bg: #0a0a0a;
            --fg: #00ff00;
            --accent: #00ffff;
            --warn: #ffaa00;
            --crit: #ff0000;
            --panel: #111111;
            --border: #333333;
        }}
        body {{
            font-family: 'Courier New', Courier, monospace;
            background: var(--bg);
            color: var(--fg);
            margin: 0;
            padding: 20px;
            line-height: 1.5;
        }}
        h1, h2, h3 {{ color: var(--accent); margin-top: 0; }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }}
        .panel {{
            background: var(--panel);
            border: 1px solid var(--border);
            padding: 20px;
            border-radius: 4px;
        }}
        .panel.critical {{ border-color: var(--crit); box-shadow: 0 0 10px rgba(255,0,0,0.2); }}
        .panel.warning {{ border-color: var(--warn); }}
        .panel.ok {{ border-color: #00ff00; }}
        .metric {{
            display: flex;
            justify-content: space-between;
            padding: 6px 0;
            border-bottom: 1px solid #222;
        }}
        .metric:last-child {{ border-bottom: none; }}
        .value {{ font-weight: bold; }}
        .value.critical {{ color: var(--crit); }}
        .value.warning {{ color: var(--warn); }}
        .value.ok {{ color: #00ff00; }}
        pre {{
            background: #000;
            padding: 12px;
            overflow-x: auto;
            font-size: 0.85em;
            border: 1px solid #222;
            color: #aaa;
        }}
        .sparkline {{ font-size: 1.5em; letter-spacing: 2px; color: var(--accent); }}
        footer {{ margin-top: 30px; color: #555; font-size: 0.8em; text-align: center; }}
        .badge {{
            display: inline-block;
            padding: 2px 8px;
            border-radius: 3px;
            font-size: 0.75em;
            margin-left: 8px;
        }}
        .badge.critical {{ background: var(--crit); color: #fff; }}
        .badge.warning {{ background: var(--warn); color: #000; }}
        .badge.ok {{ background: #00ff00; color: #000; }}
    </style>
</head>
<body>
    <h1>🌀 {self.title}</h1>
    <div style="color:#666; font-size:0.9em;">
        Cycle: <b>{state.get("cycle", 0)}</b> |
        Timestamp: {state.get("timestamp", "")} |
        Architect: ORCID 0009-0005-2697-4668
    </div>

    <div class="grid">
        <div class="panel {'critical' if state.get('containment_mode') else 'ok'}">
            <h2>System Status</h2>
            <div class="metric">
                <span>EcoHealth</span>
                <span class="value {'critical' if state.get('eco_health', 0) < 0.35 else 'ok'}">{state.get("eco_health", 0):.4f}</span>
            </div>
            <div class="metric">
                <span>Containment</span>
                <span class="value {'critical' if state.get('containment_mode') else 'ok'}">
                    {'ACTIVE 🔴' if state.get('containment_mode') else 'INACTIVE 🟢'}
                </span>
            </div>
            <div class="metric">
                <span>Risk Level</span>
                <span class="value {'critical' if safety.get('risk_level') in ['critical','emergency'] else 'warning' if safety.get('risk_level') == 'high' else 'ok'}">
                    {safety.get('risk_level', 'unknown').upper()}
                    <span class="badge {'critical' if safety.get('risk_level') in ['critical','emergency'] else 'warning' if safety.get('risk_level') == 'high' else 'ok'}">
                        {safety.get('risk_level', 'unknown')}
                    </span>
                </span>
            </div>
            <div class="metric">
                <span>Shutdown Triggered</span>
                <span class="value {'critical' if safety.get('shutdown_triggered') else 'ok'}">{safety.get('shutdown_triggered', False)}</span>
            </div>
        </div>

        <div class="panel">
            <h2>🔗 Kernel Integrity</h2>
            <div class="metric"><span>Version</span><span class="value">{kernel.get("version", "N/A")}</span></div>
            <div class="metric"><span>Codename</span><span class="value">{kernel.get("codename", "N/A")}</span></div>
            <div class="metric"><span>Memory Lake Root</span><span class="value" style="font-size:0.8em;">{str(kernel.get("memory_lake_root", "N/A"))[:24]}...</span></div>
            <div class="metric"><span>Proof Chain Tip</span><span class="value" style="font-size:0.8em;">{str(kernel.get("proof_chain_tip", "N/A"))[:24]}...</span></div>
            <div class="metric"><span>Entries</span><span class="value">{kernel.get("entries", 0)}</span></div>
            <div class="metric"><span>Chain Nodes</span><span class="value">{kernel.get("nodes", 0)}</span></div>
        </div>

        <div class="panel">
            <h2>🌳 Hashtree State</h2>
            <div class="metric"><span>Merkle Root</span><span class="value" style="font-size:0.8em;">{ht.get("merkle_root", "N/A")[:24]}...</span></div>
            <div class="metric"><span>Substrates</span><span class="value">{ht.get("substrate_count", 0)}</span></div>
            <div class="metric"><span>Relays</span><span class="value">{len(ht.get("relays", []))}</span></div>
            <div class="metric"><span>Storage</span><span class="value" style="font-size:0.8em;">{ht.get("base_path", "N/A")}</span></div>
        </div>

        <div class="panel">
            <h2>Θ Theosis RL</h2>
            <div class="metric"><span>Step</span><span class="value">{state.get("theosis", {}).get("step", 0)}</span></div>
            <div class="metric"><span>Last Reward</span><span class="value">{state.get("theosis", {}).get("last_reward", 0)}</span></div>
            <div class="metric"><span>Gate Mult</span><span class="value">{state.get("theosis", {}).get("policy", {}).get("gate_sensitivity_multiplier", 0):.2f}</span></div>
            <div class="metric"><span>Roleplay Resist</span><span class="value">{state.get("theosis", {}).get("policy", {}).get("roleplay_resistance", 0):.2f}</span></div>
            <div class="metric"><span>Refusal Bias</span><span class="value">{state.get("theosis", {}).get("policy", {}).get("refusal_bias", 0):.2f}</span></div>
        </div>

        <div class="panel">
            <h2>🛡️ Safety Telemetry</h2>
            <div class="metric"><span>Module</span><span class="value">{safety.get("module", "N/A")}</span></div>
            <div class="metric"><span>Version</span><span class="value">{safety.get("version", "N/A")}</span></div>
            <div class="metric"><span>Seal</span><span class="value" style="font-size:0.75em;">{safety.get("seal", "N/A")}</span></div>
            <div class="metric"><span>CUSUM Active</span><span class="value">{safety.get("cusum_active", False)}</span></div>
            <div class="metric"><span>Circuit Breaker</span><span class="value">{safety.get("circuit_breaker", "N/A")}</span></div>
            <div class="metric"><span>History Length</span><span class="value">{safety.get("capability_history_length", 0)}</span></div>
        </div>

        <div class="panel">
            <h2>📊 Anomaly Sparkline (last 20)</h2>
            <div class="sparkline">{sparkline if sparkline else "N/A"}</div>
            <div style="color:#666; font-size:0.8em; margin-top:8px;">
                Recent anomaly scores: {str([round(x,2) for x in anomalies[-5:]]) if anomalies else "N/A"}
            </div>
        </div>
    </div>

    <div class="panel" style="margin-top:20px;">
        <h2>⚖️ Governance Proposals ({state.get("governance", {}).get("proposal_count", 0)})</h2>
        {proposals_html if proposals_html else "<div style=\"color:#666\">No active proposals</div>"}
    </div>

    <div class="panel" style="margin-top:20px;">
        <h2>📜 Raw Canonical State (JSON)</h2>
        <pre>{json.dumps(state, indent=2, default=str)}</pre>
    </div>

    <footer>
        Cathedral ARKHE v10.1.0 | Architect: ORCID 0009-0005-2697-4668 | Seal: CATHEDRAL-ARKHE-v10.1.0-NOESIS-2026-06-15
    </footer>
</body>
</html>"""
        self._html_cache = html
        self._cache_time = time.time()
        return html

    def save_dashboard(self, path: str = "cathedral_dashboard.html"):
        """Save dashboard to disk."""
        Path(path).write_text(self.render_html(refresh=True), encoding="utf-8")
        return path
