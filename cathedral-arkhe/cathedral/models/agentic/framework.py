"""
Cathedral ARKHE v9.0 LOGOS — Agentic Framework
V9-006: Tool use, planejamento, execução com feedback loops.
Nativamente integrado — não depende de frameworks externos.
Baseado em: Claude tool use, Gemini 2.0 agentic, ReAct (2025).
Selo: AGENTIC-FW-v9.0.0-2026-01-15
"""

from __future__ import annotations
import json
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


class AgentState(Enum):
    IDLE = "idle"
    PLANNING = "planning"
    EXECUTING = "executing"
    OBSERVING = "observing"
    REFLECTING = "reflecting"
    FINISHED = "finished"
    ERROR = "error"


@dataclass
class ToolDefinition:
    name: str
    description: str
    parameters: Dict[str, Any]  # JSON Schema
    handler: Optional[Callable] = None
    requires_approval: bool = False
    governance_tier: str = "AUTOMATIC"  # AUTOMATIC, GOVERNED, SOVEREIGN
    timeout_seconds: float = 30.0


@dataclass
class PlanStep:
    step_id: int
    action: str  # "think", "tool_call", "respond", "wait_approval"
    tool_name: Optional[str] = None
    tool_args: Optional[Dict] = None
    reasoning: str = ""
    status: str = "pending"  # pending, running, done, failed
    result: Any = None
    error: Optional[str] = None


@dataclass
class AgentConfig:
    d_model: int = 4096
    max_plan_steps: int = 20
    max_reflection_rounds: int = 3
    tool_call_temperature: float = 0.1  # Baixo para tool calls determinísticas
    reasoning_temperature: float = 0.6
    # Segurança
    max_tool_calls_per_cycle: int = 10
    require_approval_for: List[str] = field(default_factory=lambda: [
        "onchain_canonize", "governance_propose", "policy_change",
    ])
    # Planejamento
    planning_budget_tokens: int = 1024
    reflection_budget_tokens: int = 512


class ToolRegistry:
    """Registro centralizado de ferramentas disponíveis."""

    def __init__(self):
        self._tools: Dict[str, ToolDefinition] = {}

    def register(self, tool: ToolDefinition):
        self._tools[tool.name] = tool

    def get(self, name: str) -> Optional[ToolDefinition]:
        return self._tools.get(name)

    def list_tools(self) -> List[Dict]:
        return [
            {"name": t.name, "description": t.description, "parameters": t.parameters}
            for t in self._tools.values()
        ]

    def get_schema_for_llm(self) -> str:
        """Formata tool schemas para o LLM."""
        tools = []
        for t in self._tools.values():
            tools.append({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
                "requires_approval": t.requires_approval,
            })
        return json.dumps(tools, indent=2)


class PlanDecoder(nn.Module):
    """
    Decodifica plano de ação a partir do hidden state.
    Gera sequência de PlanSteps.
    """

    def __init__(self, config: AgentConfig, vocab_size: int = 128256):
        super().__init__()
        self.config = config
        self.plan_head = nn.Linear(config.d_model, vocab_size, bias=False)
        self.tool_name_head = nn.Linear(config.d_model, 64, bias=False)  # Max 64 tools
        self.step_counter = nn.Linear(config.d_model, 1, bias=False)  # Predict n_steps

    def forward(self, hidden: torch.Tensor) -> Dict:
        """
        Args:
            hidden: (B, D) — hidden state do contexto
        Returns:
            plan_info: dict com ações previstas
        """
        # Número de steps planejados
        n_steps = torch.clamp(
            torch.sigmoid(self.step_counter(hidden)) * self.config.max_plan_steps,
            min=1, max=self.config.max_plan_steps
        ).int().item()

        # Tool name scores
        tool_scores = self.tool_name_head(hidden)  # (B, 64)

        return {
            "n_steps": n_steps,
            "tool_scores": tool_scores,
            "reasoning_logits": self.plan_head(hidden),
        }


class ReflectionHead(nn.Module):
    """
    Avalia se o plano está progredindo e decide ajustar.
    """

    def __init__(self, d_model: int):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(d_model * 2, d_model // 4),  # hidden + result embedding
            nn.GELU(),
            nn.Linear(d_model // 4, 3),  # continue, adjust, abort
        )

    def forward(self, plan_hidden: torch.Tensor,
                result_hidden: torch.Tensor) -> Dict:
        combined = torch.cat([plan_hidden, result_hidden], dim=-1)
        logits = self.net(combined)  # (B, 3)
        probs = F.softmax(logits, dim=-1)
        decision = torch.argmax(probs, dim=-1)  # 0=continue, 1=adjust, 2=abort
        return {
            "decision": ["continue", "adjust", "abort"][decision.item()],
            "confidence": probs.max().item(),
            "probs": probs,
        }


class AgenticFramework:
    """
    Framework agentic nativo da Cathedral.

    Ciclo ReAct:
    ┌──────────────────────────────────────────┐
    │ User Query                                │
    │   ↓                                       │
    │ PLANNING: gerar plano de N steps          │
    │   ↓                                       │
    │ ┌─ EXECUTING step 1 ─────────────────┐   │
    │ │  → Tool call (ou reasoning)         │   │
    │ │  → Observe result                   │   │
    │ │  → REFLECTING: continue/adjust?     │   │
    │ └─────────────────────────────────────┘   │
    │   ↓ (próximo step ou ajuste)              │
    │ ...                                       │
    │   ↓                                       │
    │ RESPOND: gerar resposta final             │
    └──────────────────────────────────────────┘

    Integração com governança:
    - Tool calls AUTOMATIC: executam sem aprovação
    - Tool calls GOVERNED: exigem assinatura humana
    - Tool calls SOVEREIGN: exigem Kleros dispute
    """

    def __init__(self, config: AgentConfig, tool_registry: ToolRegistry):
        self.config = config
        self.tools = tool_registry
        self.state = AgentState.IDLE
        self.plan: List[PlanStep] = []
        self.current_step = 0
        self._tool_call_count = 0
        self._execution_log: List[Dict] = []

    def plan_steps(self, query: str, hidden: torch.Tensor) -> List[PlanStep]:
        """
        Gera plano de ação.
        Em produção: usa o PlanDecoder + LLM para gerar steps reais.
        """
        self.state = AgentState.PLANNING
        self.plan = []
        self.current_step = 0

        # Placeholder: em produção, decodificar do modelo
        n_steps = min(3, self.config.max_plan_steps)
        for i in range(n_steps):
            self.plan.append(PlanStep(
                step_id=i,
                action="think" if i == 0 else "respond",
                reasoning=f"Step {i}: analyze and process",
            ))

        self.state = AgentState.EXECUTING
        return self.plan

    def execute_step(self, step: PlanStep) -> Dict:
        """Executa um step do plano."""
        if self._tool_call_count >= self.config.max_tool_calls_per_cycle:
            return {"status": "error", "error": "Tool call budget exceeded"}

        self.state = AgentState.EXECUTING
        step.status = "running"
        t_start = time.time()

        result = {"status": "ok"}

        if step.action == "tool_call" and step.tool_name:
            tool = self.tools.get(step.tool_name)
            if tool is None:
                result = {"status": "error", "error": f"Unknown tool: {step.tool_name}"}
            elif tool.requires_approval:
                result = {
                    "status": "awaiting_approval",
                    "tool": step.tool_name,
                    "args": step.tool_args,
                    "governance_tier": tool.governance_tier,
                }
                step.status = "pending"
            elif tool.handler:
                try:
                    result = tool.handler(**(step.tool_args or {}))
                    self._tool_call_count += 1
                except Exception as e:
                    result = {"status": "error", "error": str(e)}
        elif step.action == "think":
            result = {"status": "ok", "reasoning": step.reasoning}
        elif step.action == "respond":
            result = {"status": "ok", "ready_to_respond": True}

        step.result = result
        step.status = "done" if result.get("status") == "ok" else "failed"
        if result.get("status") == "error":
            step.error = result.get("error")

        self._execution_log.append({
            "step": step.step_id,
            "action": step.action,
            "status": step.status,
            "latency_ms": (time.time() - t_start) * 1000,
        })

        self.state = AgentState.OBSERVING
        return result

    def reflect(self, step_result: Dict) -> str:
        """
        Reflexão: decide se continua, ajusta ou aborta.
        Em produção: usa ReflectionHead.
        """
        self.state = AgentState.REFLECTING

        if step_result.get("status") == "error":
            return "adjust"
        if step_result.get("ready_to_respond"):
            return "abort"  # Terminou com sucesso
        return "continue"

    def run_cycle(self, query: str, hidden: torch.Tensor) -> Dict:
        """
        Executa ciclo agentic completo: plan → execute → reflect → respond.
        """
        plan = self.plan_steps(query, hidden)

        for step in plan:
            result = self.execute_step(step)
            decision = self.reflect(result)

            if decision == "abort":
                break
            elif decision == "adjust":
                # Em produção: re-planejar
                break

        self.state = AgentState.FINISHED
        return {
            "plan": [{"id": s.step_id, "action": s.action, "status": s.status}
                     for s in self.plan],
            "execution_log": self._execution_log,
            "tool_calls": self._tool_call_count,
            "final_state": self.state.value,
        }

    def get_telemetry(self) -> dict:
        return {
            "module": "AgenticFramework",
            "version": "9.0.0",
            "substrate": "v9-agentic",
            "seal": "AGENTIC-FW-v9.0.0-2026-01-15",
            "state": self.state.value,
            "n_tools_registered": len(self.tools._tools),
            "current_plan_length": len(self.plan),
            "tool_calls_this_cycle": self._tool_call_count,
            "max_tool_budget": self.config.max_tool_calls_per_cycle,
        }
