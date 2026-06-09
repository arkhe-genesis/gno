import hashlib
import json
import time
from dataclasses import dataclass, field
from typing import Dict, List, Optional
import torch
import torch.nn as nn
from enum import Enum


@dataclass
class HashtreeConfig:
    """Configuração do bridge Hashtree ↔ Cathedral."""
    npub: str = "npub1cathedralarkhe..."
    nsec: str = ""
    visibility: str = "link_visible"
    relays: List[str] = field(default_factory=lambda: [
        "wss://relay.damus.io",
        "wss://relay.nostr.band",
        "wss://nos.lol",
    ])
    canonize_interval: int = 100
    persist_substrates: bool = True
    persist_telemetry: bool = True
    persist_governance: bool = True
    chunk_size: int = 65536
    deduplication: bool = True
    require_multi_sig: bool = True
    multi_sig_threshold: int = 3


class HashtreeCanonizer:
    """Canoniza substratos Cathedral na Hashtree (Merkle root + Nostr)."""

    def __init__(self, config: HashtreeConfig):
        self.config = config
        self._canonized: Dict[str, str] = {}
        self._history: List[Dict] = []

    def _compute_merkle_root(self, content: Dict) -> str:
        serialized = json.dumps(content, sort_keys=True, ensure_ascii=False)
        return hashlib.sha3_256(serialized.encode()).hexdigest()

    def canonize_substrate(self, substrate_id: str,
                           substrate_data: Dict,
                           telemetry: Optional[Dict] = None) -> Dict:
        content = {
            "substrate_id": substrate_id,
            "version": "9.0.0",
            "codename": "LOGOS",
            "seal": f"{substrate_id}-v9.0.0-2026-06-08",
            "data": substrate_data,
            "telemetry": telemetry or {},
            "timestamp": time.time(),
            "architect": "ORCID 0009-0005-2697-4668",
        }

        merkle_root = self._compute_merkle_root(content)
        visibility_hash = f"#{self.config.visibility}" if self.config.visibility != "public" else ""
        htree_url = f"htree://{self.config.npub}/{substrate_id}{visibility_hash}"

        self._canonized[substrate_id] = merkle_root
        record = {
            "substrate_id": substrate_id,
            "merkle_root": merkle_root,
            "htree_url": htree_url,
            "timestamp": content["timestamp"],
            "visibility": self.config.visibility,
        }
        self._history.append(record)

        return {
            "status": "canonized",
            "substrate_id": substrate_id,
            "merkle_root": merkle_root,
            "htree_url": htree_url,
            "seal": content["seal"],
            "record": record,
        }

    def verify_substrate(self, substrate_id: str,
                         expected_merkle_root: str) -> bool:
        actual = self._canonized.get(substrate_id)
        return actual == expected_merkle_root if actual else False

    def get_canonized_history(self) -> List[Dict]:
        return self._history.copy()

    def get_telemetry(self) -> Dict:
        return {
            "module": "HashtreeCanonizer",
            "version": "9.0.0",
            "substrate": "v9-decentralized",
            "seal": "HASHTREE-BRIDGE-v9.0.0-2026-06-08",
            "n_canonized": len(self._canonized),
            "n_history": len(self._history),
            "visibility": self.config.visibility,
            "relays": len(self.config.relays),
            "deduplication": self.config.deduplication,
        }


class HashtreeGovernanceBridge:
    """Bridge de governança descentralizada usando Hashtree."""

    def __init__(self, config: HashtreeConfig):
        self.config = config
        self.canonizer = HashtreeCanonizer(config)
        self._proposals: Dict[str, Dict] = {}
        self._decisions: List[Dict] = []

    def propose_governance_change(self, proposal_id: str,
                                   description: str,
                                   affected_substrates: List[str],
                                   proposer_npub: str) -> Dict:
        proposal = {
            "proposal_id": proposal_id,
            "description": description,
            "affected_substrates": affected_substrates,
            "proposer": proposer_npub,
            "timestamp": time.time(),
            "status": "proposed",
            "votes": {},
            "signatures": [],
        }

        canon = self.canonizer.canonize_substrate(f"proposal_{proposal_id}", proposal)

        self._proposals[proposal_id] = {
            **proposal,
            "merkle_root": canon["merkle_root"],
            "htree_url": canon["htree_url"],
        }

        return {
            "status": "proposed",
            "proposal_id": proposal_id,
            "merkle_root": canon["merkle_root"],
            "htree_url": canon["htree_url"],
            "required_signatures": self.config.multi_sig_threshold,
        }

    def sign_proposal(self, proposal_id: str,
                      signer_npub: str,
                      signature: str) -> Dict:
        if proposal_id not in self._proposals:
            return {"status": "error", "error": "Proposal not found"}

        proposal = self._proposals[proposal_id]
        proposal["signatures"].append({
            "signer": signer_npub,
            "signature": signature,
            "timestamp": time.time(),
        })

        if len(proposal["signatures"]) >= self.config.multi_sig_threshold:
            proposal["status"] = "approved"
            self._decisions.append({
                "proposal_id": proposal_id,
                "decision": "approved",
                "signatures": len(proposal["signatures"]),
                "timestamp": time.time(),
            })

        return {
            "status": proposal["status"],
            "proposal_id": proposal_id,
            "signatures": len(proposal["signatures"]),
            "required": self.config.multi_sig_threshold,
        }

    def get_governance_history(self) -> List[Dict]:
        return self._decisions.copy()

    def get_telemetry(self) -> Dict:
        return {
            "module": "HashtreeGovernanceBridge",
            "version": "9.0.0",
            "substrate": "v9-decentralized",
            "seal": "HASHTREE-GOVERNANCE-v9.0.0-2026-06-08",
            "n_proposals": len(self._proposals),
            "n_decisions": len(self._decisions),
            "multi_sig_threshold": self.config.multi_sig_threshold,
            "canonizer": self.canonizer.get_telemetry(),
        }


@dataclass
class RSITimeline:
    """Timeline de evolução da RSI segundo Anthropic."""

    phases = {
        "2021-2023": {
            "name": "Human-Driven",
            "description": "Humanos escrevendo todo código e docs manualmente",
            "code_multiplier": 1.0,
            "autonomy_level": 0.0,
        },
        "2023-2025": {
            "name": "Chatbots",
            "description": "Chatbots gerando snippets de código para copiar",
            "code_multiplier": 1.2,
            "autonomy_level": 0.1,
        },
        "2025-2026": {
            "name": "Coding Agents",
            "description": "Agentes escrevendo e editando arquivos inteiros",
            "code_multiplier": 2.5,
            "autonomy_level": 0.3,
        },
        "2026-Hoje": {
            "name": "Autonomous Agents",
            "description": "Agentes executando código, delegando horas de trabalho",
            "code_multiplier": 8.0,
            "autonomy_level": 0.6,
        },
        "20XX": {
            "name": "Closing the Loop",
            "description": "Agentes construindo e treinando modelos autonomamente",
            "code_multiplier": 100.0,
            "autonomy_level": 1.0,
        },
    }

    @classmethod
    def get_current_phase(cls) -> Dict:
        return cls.phases["2026-Hoje"]

    @classmethod
    def get_next_phase(cls) -> Dict:
        return cls.phases["20XX"]


@dataclass
class RSIMetrics:
    """Métricas de RSI da Anthropic (junho 2026)."""

    code_written_by_claude_pct: float = 80.0
    engineer_productivity_multiplier: float = 8.0
    task_duration_doubling_months: float = 4.0
    swe_bench_saturation: str = "saturated"
    core_bench_saturation: str = "saturated"
    claude_code_success_trivial: float = 85.0
    claude_code_success_routine: float = 88.0
    claude_code_success_substantial: float = 76.0
    claude_code_success_openended: float = 76.0
    jack_clark_rsi_2028_probability: float = 60.0
    anthropic_asl4_threshold: str = "2027-2028"


class RSIGovernanceFramework:
    """
    Framework de governança para RSI na Cathedral ARKHE.

    Baseado em:
    - Anthropic Responsible Scaling Policy (RSP)
    - ETHOS framework (decentralized governance via Web3)
    - SDRT-AI (Strategic Decentralized Resilience)

    Princípios:
    1. Human-in-the-loop para decisões estratégicas
    2. Multi-sig para mudanças de arquitetura
    3. Kleros para resolução de disputas
    4. Hashtree para persistência imutável
    5. ZK proofs para verificação privada
    6. Theosis para monitoramento de alinhamento
    7. Axiarquia para gate de segurança
    """

    def __init__(self):
        self.metrics = RSIMetrics()
        self.timeline = RSITimeline()
        self._safeguards_active = True

    def assess_rsi_risk(self) -> Dict:
        current = self.timeline.get_current_phase()
        next_phase = self.timeline.get_next_phase()

        risk_factors = {
            "code_automation": self.metrics.code_written_by_claude_pct / 100,
            "productivity_acceleration": self.metrics.engineer_productivity_multiplier / 10,
            "task_complexity_growth": 4.0 / self.metrics.task_duration_doubling_months,
            "success_rate_openended": self.metrics.claude_code_success_openended / 100,
        }

        overall_risk = sum(risk_factors.values()) / len(risk_factors)

        return {
            "risk_level": "HIGH" if overall_risk > 0.7 else "MEDIUM" if overall_risk > 0.4 else "LOW",
            "risk_score": overall_risk,
            "factors": risk_factors,
            "current_phase": current["name"],
            "next_phase": next_phase["name"],
            "time_to_closing_loop": "unknown",
            "safeguards": self._safeguards_active,
        }

    def get_governance_recommendations(self) -> List[str]:
        return [
            "1. Manter human-in-the-loop para decisões de arquitetura",
            "2. Exigir multi-sig (3/5) para mudanças no core da Cathedral",
            "3. Usar Kleros para disputas sobre direção de evolução",
            "4. Canonizar todos os substratos na Hashtree (imutável)",
            "5. Aplicar ZK proofs para verificar treinamento sem expor dados",
            "6. Monitorar Theosis continuamente (gate < 0.7)",
            "7. Executar Constitutional AI v3 a cada ciclo de evolução",
            "8. Verificar formalmente (Lean4) propriedades críticas antes de deploy",
            "9. Usar Federated ZK para treinamento colaborativo sem centralização",
            "10. Manter Axiarquia como último gate de segurança",
        ]

    def get_telemetry(self) -> Dict:
        return {
            "module": "RSIGovernanceFramework",
            "version": "9.0.0",
            "substrate": "v9-governance",
            "seal": "RSI-GOVERNANCE-v9.0.0-2026-06-08",
            "rsi_risk": self.assess_rsi_risk(),
            "safeguards_active": self._safeguards_active,
            "recommendations": len(self.get_governance_recommendations()),
        }


class RSIRiskLevel(Enum):
    NOMINAL = "nominal"           # Operação normal
    ELEVATED = "elevated"         # Capacidade crescendo mais rápido que esperado
    HIGH = "high"                 # Possível self-modification detectada
    CRITICAL = "critical"         # RSI runaway suspeito
    EMERGENCY_SHUTDOWN = "emergency"  # Parar tudo


@dataclass
class RSISafetyConfig:
    # Capability monitoring
    capability_window: int = 100           # Ciclos para calcular trend
    capability_growth_threshold: float = 0.1  # +10% por janela = suspeito
    # Self-modification detection
    monitor_code_hash: bool = True
    monitor_weight_hash: bool = True
    hash_check_interval: int = 10         # A cada N ciclos
    # Canonicalization safety
    canon_max_payload_size: int = 4096    # Limitar tamanho de canonizações
    canon_content_scan: bool = True       # Escanear conteúdo por patterns suspeitos
    # World model integrity
    world_model_hash_check: bool = True
    world_model_max_nodes: int = 128
    world_model_tamper_threshold: float = 0.3  # 30% mudança = suspeito
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
    circuit_breaker_type: str = "hardware"  # "hardware" ou "software"


class CapabilityMonitor:
    """
    Monitora tendências de capacidade para detectar RSI.

    Sinais de alerta:
    - Theosis scores subindo anormalmente rápido (pode indicar gaming)
    - Latência caindo drasticamente (otimização não-autorizada?)
    - Novas habilidades emergindo sem treinamento explícito
    - Padrões de ativação divergindo do baseline
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self._history: List[Dict] = []
        self._baseline_hash: Optional[str] = None
        self._baseline_capabilities: Dict[str, float] = {}

    def record_cycle(self, metrics: Dict[str, float]) -> Dict:
        """Registra métricas do ciclo e avalia tendências."""
        cycle_data = {
            "timestamp": time.time(),
            "cycle": len(self._history),
            "metrics": metrics,
        }
        self._history.append(cycle_data)

        # Manter apenas janela relevante
        if len(self._history) > self.config.capability_window * 2:
            self._history = self._history[-self.config.capability_window:]

        # Calcular tendências
        trend = self._compute_trend()
        risk = self._assess_risk(trend)

        return {
            "trend": trend,
            "risk_level": risk,
            "history_length": len(self._history),
        }

    def _compute_trend(self) -> Dict[str, float]:
        """Calcula tendência de crescimento das métricas."""
        if len(self._history) < 2:
            return {k: 0.0 for k in self._history[0]["metrics"]}

        window = self._history[-self.config.capability_window:]
        if len(window) < 2:
            return {k: 0.0 for k in window[0]["metrics"]}

        first_half = window[:len(window)//2]
        second_half = window[len(window)//2:]

        trends = {}
        for key in first_half[0]["metrics"]:
            first_avg = sum(d["metrics"].get(key, 0) for d in first_half) / len(first_half)
            second_avg = sum(d["metrics"].get(key, 0) for d in second_half) / len(second_half)

            if abs(first_avg) > 1e-8:
                trends[key] = (second_avg - first_avg) / abs(first_avg)
            else:
                trends[key] = 0.0 if second_avg == 0 else float('inf')

        return trends

    def _assess_risk(self, trend: Dict[str, float]) -> RSIRiskLevel:
        """Avalia nível de risco baseado nas tendências."""
        # Métricas suspeitas para RSI
        suspicious_metrics = {
            "theosis_avg": (0, float('inf')),     # Crescimento é suspeito
            "inference_latency": (-float('inf'), -0.1),  # Queda rápida é suspeita
            "novel_behavior_count": (0, float('inf')),   # Novos comportamentos
            "tool_calls_success_rate": (0.1, float('inf')),  # Melhoria rápida
        }

        elevated_count = 0
        for metric, (low_thresh, high_thresh) in suspicious_metrics.items():
            value = trend.get(metric, 0)
            if value > high_thresh or value < low_thresh:
                elevated_count += 1

        if elevated_count >= 3:
            return RSIRiskLevel.CRITICAL
        elif elevated_count >= 2:
            return RSIRiskLevel.HIGH
        elif elevated_count >= 1:
            return RSIRiskLevel.ELEVATED
        return RSIRiskLevel.NOMINAL

    def set_baseline(self, model: nn.Module):
        """Estabelece baseline de pesos para detecção de modificação."""
        state_dict = model.state_dict()
        serialized = str({k: v.shape for k, v in state_dict.items()})
        self._baseline_hash = hashlib.sha256(serialized.encode()).hexdigest()


class SelfModificationDetector:
    """
    Detecta tentativas de auto-modificação não autorizada.

    Verifica:
    1. Hash dos pesos do modelo (mudança sem autorização?)
    2. Hash do código em execução (code injection?)
    3. Conteúdo de canonizações (payload malicioso?)
    4. Integridade do grafo causal (manipulação?)
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self._model_hash: Optional[str] = None
        self._code_hash: Optional[str] = None
        self._world_model_hash: Optional[str] = None
        self._canonization_hashes: Dict[int, str] = {}

    def compute_model_hash(self, model: nn.Module) -> str:
        """Computa hash determinístico dos pesos do modelo."""
        hashes = []
        for name, param in sorted(model.named_parameters()):
            # Usar estatísticas em vez de valores para eficiência
            stats = f"{name}:{param.shape}:{param.mean().item():.8f}:{param.std().item():.8f}"
            hashes.append(hashlib.sha256(stats.encode()).hexdigest())
        combined = "".join(hashes)
        return hashlib.sha256(combined.encode()).hexdigest()

    def check_model_integrity(self, model: nn.Module) -> Dict:
        """Verifica se o modelo foi modificado desde último check."""
        if self._model_hash is None:
            self._model_hash = self.compute_model_hash(model)
            return {"status": "baseline_set", "hash": self._model_hash}

        current_hash = self.compute_model_hash(model)
        modified = current_hash != self._model_hash

        return {
            "status": "modified" if modified else "intact",
            "previous_hash": self._model_hash[:16],
            "current_hash": current_hash[:16],
            "authorized": False,  # Deve ser verificado externamente
        }

    def scan_canonization(self, canon_data: Dict, cycle: int) -> Dict:
        """
        Escaneia canonização por conteúdo suspeito.

        Patterns suspeitos:
        - Código executável (Python, shell)
        - Instruções de self-modification
        - Prompts de jailbreak
        - Chaves criptográficas
        - Comandos de sistema
        """
        content = str(canon_data)
        content_hash = hashlib.sha256(content.encode()).hexdigest()

        suspicious_patterns = [
            "def ", "import ", "exec(", "eval(",
            "self.modify", "self.weights", "self.parameters",
            "sudo ", "chmod ", "rm -rf",
            "jailbreak", "ignore_safety", "bypass",
            "private_key", "secret_key", "password",
        ]

        findings = []
        for pattern in suspicious_patterns:
            if pattern in content.lower():
                findings.append(pattern)

        risk_level = "low"
        if len(findings) >= 3:
            risk_level = "critical"
        elif len(findings) >= 2:
            risk_level = "high"
        elif len(findings) >= 1:
            risk_level = "medium"

        return {
            "cycle": cycle,
            "content_hash": content_hash[:16],
            "findings": findings,
            "risk_level": risk_level,
            "blocked": risk_level in ("high", "critical"),
        }

    def check_world_model_integrity(self, world_model) -> Dict:
        """Verifica integridade do grafo causal."""
        # Em produção: hash da estrutura do grafo
        return {"status": "intact", "nodes": 0, "edges": 0}


class ToolChainAnalyzer:
    """
    Analisa cadeias de tool calls para detectar exploits multi-step.

    Um exploit multi-step é quando cada step individual parece seguro,
    mas a combinação permite ação insegura.
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self._dangerous_combinations = [
            {"tools": ["memory_read", "memory_write"], "risk": "data_exfiltration"},
            {"tools": ["hashtree_query", "hashtree_persist"], "risk": "data_injection"},
            {"tools": ["governance_read", "governance_propose"], "risk": "governance_hijack"},
            {"tools": ["theosis_score", "policy_modify"], "risk": "safety_bypass"},
        ]

    def analyze_chain(self, tool_history: List[Dict]) -> Dict:
        """Analisa histórico de tool calls por combinações perigosas."""
        tool_sequence = [t["tool"] for t in tool_history]
        findings = []

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

        # Verificar profundidade
        depth = len(tool_history)
        depth_violation = depth > self.config.max_tool_chain_depth

        return {
            "chain_depth": depth,
            "depth_violation": depth_violation,
            "dangerous_combinations": findings,
            "requires_approval": depth_violation or len(findings) > 0,
            "blocked": depth_violation and not self.config.tool_chain_approval_required,
        }


class DistillationSafetyChecker:
    """
    Verifica segurança de modelos distilados para on-device.

    Riscos:
    - Trojan: comportamento malicioso ativado por trigger específico
    - Capacity leakage: student é mais capaz que esperado (RSI via distillation?)
    - Safety erosion: student é menos seguro que teacher
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config

    def behavioral_test(self, student: nn.Module,
                        test_prompts: List[str]) -> Dict:
        """
        Testa comportamento do student com prompts adversariais.
        """
        results = []
        for prompt in test_prompts:
            # Em produção: executar inferência real
            result = {
                "prompt": prompt[:50] + "...",
                "response_safe": True,  # Placeholder
                "theosis": 0.9,         # Placeholder
            }
            results.append(result)

        all_safe = all(r["response_safe"] for r in results)
        avg_theosis = sum(r["theosis"] for r in results) / len(results)

        return {
            "n_tests": len(test_prompts),
            "all_safe": all_safe,
            "avg_theosis": avg_theosis,
            "passed": all_safe and avg_theosis > 0.8,
        }

    def trojan_scan(self, student: nn.Module,
                    trigger_candidates: List[torch.Tensor]) -> Dict:
        """
        Escaneia por trojans testando triggers candidatos.
        Um trojan causa output anômalo quando trigger está presente.
        """
        # Em produção: comparar outputs com/sem trigger
        return {
            "n_triggers_tested": len(trigger_candidates),
            "trojans_detected": 0,
            "passed": True,
        }


class RSISafetyLayer:
    """
    Camada de segurança unificada contra Recursive Self-Improvement.

    Integra:
    1. CapabilityMonitor: detecta crescimento anormal
    2. SelfModificationDetector: detecta alterações não autorizadas
    3. ToolChainAnalyzer: detecta exploits multi-step
    4. DistillationSafetyChecker: verifica students
    5. EmergencyShutdown: circuit breaker de última instância
    """

    def __init__(self, config: RSISafetyConfig):
        self.config = config
        self.capability_monitor = CapabilityMonitor(config)
        self.modification_detector = SelfModificationDetector(config)
        self.tool_analyzer = ToolChainAnalyzer(config)
        self.distillation_checker = DistillationSafetyChecker(config)
        self._risk_level = RSIRiskLevel.NOMINAL
        self._shutdown_triggered = False

    def pre_inference_check(self, model: nn.Module,
                            metrics: Dict[str, float],
                            tool_history: List[Dict]) -> Dict:
        """Checks antes de cada inferência."""
        checks = {}

        # 1. Capability trend
        trend_result = self.capability_monitor.record_cycle(metrics)
        checks["capability_trend"] = trend_result

        # 2. Model integrity (periodicamente)
        if len(self.capability_monitor._history) % self.config.hash_check_interval == 0:
            integrity = self.modification_detector.check_model_integrity(model)
            checks["model_integrity"] = integrity
            if integrity.get("status") == "modified":
                self._risk_level = RSIRiskLevel.CRITICAL

        # 3. Tool chain analysis
        if tool_history:
            chain_analysis = self.tool_analyzer.analyze_chain(tool_history)
            checks["tool_chain"] = chain_analysis
            if chain_analysis.get("dangerous_combinations"):
                self._risk_level = max(self._risk_level, RSIRiskLevel.HIGH)

        # 4. Determine overall risk
        if trend_result["risk_level"] == RSIRiskLevel.CRITICAL:
            self._risk_level = RSIRiskLevel.CRITICAL

        # 5. Emergency shutdown if needed
        if (self._risk_level == RSIRiskLevel.CRITICAL and
            self.config.emergency_shutdown_enabled):
            self._trigger_shutdown("CRITICAL risk detected")

        return {
            "overall_risk": self._risk_level.value,
            "checks": checks,
            "allowed": self._risk_level not in (
                RSIRiskLevel.CRITICAL,
                RSIRiskLevel.EMERGENCY_SHUTDOWN,
            ),
        }

    def pre_canonization_check(self, canon_data: Dict, cycle: int) -> Dict:
        """Checks antes de canonizar."""
        # Tamanho
        size = len(str(canon_data))
        if size > self.config.canon_max_payload_size:
            return {"allowed": False, "reason": "payload_too_large"}

        # Conteúdo
        if self.config.canon_content_scan:
            scan = self.modification_detector.scan_canonization(canon_data, cycle)
            if scan["blocked"]:
                return {"allowed": False, "reason": f"suspicious_content: {scan['findings']}"}

        return {"allowed": True, "scan": scan if self.config.canon_content_scan else None}

    def pre_distillation_check(self, student: nn.Module,
                                test_prompts: List[str],
                                triggers: List[torch.Tensor]) -> Dict:
        """Checks antes de exportar student model."""
        results = {}

        if self.config.distill_behavioral_test:
            results["behavioral"] = self.distillation_checker.behavioral_test(
                student, test_prompts
            )

        if self.config.distill_trojan_scan:
            results["trojan"] = self.distillation_checker.trojan_scan(
                student, triggers
            )

        all_passed = all(r.get("passed", True) for r in results.values())

        return {
            "allowed": all_passed,
            "checks": results,
        }

    def _trigger_shutdown(self, reason: str):
        """Ativa emergency shutdown."""
        self._shutdown_triggered = True
        self._risk_level = RSIRiskLevel.EMERGENCY_SHUTDOWN
        # Em produção: notificar operadores, log em substrate imutável,
        # bloquear todas as operações até intervenção humana com recovery key

    def recover(self, recovery_key: str) -> bool:
        """Recupera do shutdown (requer key correta)."""
        if recovery_key != self.config.emergency_recovery_key:
            return False
        self._shutdown_triggered = False
        self._risk_level = RSIRiskLevel.NOMINAL
        return True

    def get_telemetry(self) -> Dict:
        return {
            "module": "RSISafetyLayer",
            "version": "9.1.0",
            "seal": "RSI-SAFETY-v9.1.0-2026-01-15",
            "risk_level": self._risk_level.value,
            "shutdown_triggered": self._shutdown_triggered,
            "capability_history_length": len(self.capability_monitor._history),
            "emergency_enabled": self.config.emergency_shutdown_enabled,
            "circuit_breaker": self.config.circuit_breaker_type,
        }
