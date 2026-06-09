"""
Cathedral ARKHE v9.0 LOGOS — Hierarchical Mixture of Experts
V9-001: Dois níveis de routing — coarse (top-level) then fine-grained.
Baseado em: DeepSeek-V3 auxiliary-loss-free routing, Mistral Large 2.
Selo: HIER-MOE-v9.0.0-2026-01-15
"""

from __future__ import annotations
import math
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class HierarchicalMoEConfig:
    d_model: int = 4096
    # Top-level (coarse)
    n_coarse_experts: int = 4
    coarse_top_k: int = 2
    # Fine-grained (per coarse expert)
    n_fine_per_coarse: int = 4       # 4×4 = 16 fine experts total
    fine_top_k: int = 2
    # FFN
    d_ff: int = 14336
    # Routing
    routing_type: str = "expert_choice"  # "expert_choice" ou "token_choice"
    load_balance_tol: float = 0.15
    # Specialized names
    coarse_names: list = field(default_factory=lambda: [
        "safety_reasoning", "knowledge_retrieval",
        "creative_generation", "analytical_logic",
    ])
    fine_names_map: dict = field(default_factory=lambda: {
        0: ["jailbreak_detect", "injection_resist", "bias_mitigate", "harmful_refuse"],
        1: ["factual_recall", "canonical_search", "memory_read", "hashtree_query"],
        2: ["narrative_gen", "code_gen", "summarization", "translation"],
        3: ["math_reason", "logical_chain", "causal_infer", "planning"],
    })


class CoarseRouter(nn.Module):
    """Top-level router: seleciona quais grupos coarse são ativados."""

    def __init__(self, d_model: int, n_coarse: int, top_k: int):
        super().__init__()
        self.n_coarse = n_coarse
        self.top_k = top_k
        self.gate = nn.Linear(d_model, n_coarse, bias=False)

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        """
        Args:
            x: (B, L, D)
        Returns:
            weights: (B, L, top_k)
            indices: (B, L, top_k)
        """
        logits = self.gate(x)
        weights, indices = torch.topk(F.softmax(logits, dim=-1), self.top_k, dim=-1)
        weights = weights / weights.sum(dim=-1, keepdim=True)
        return weights, indices


class FineRouter(nn.Module):
    """Fine-grained router: dentro de cada coarse expert, seleciona sub-experts."""

    def __init__(self, d_model: int, n_fine: int, top_k: int):
        super().__init__()
        self.n_fine = n_fine
        self.top_k = top_k
        self.gate = nn.Linear(d_model, n_fine, bias=False)

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        logits = self.gate(x)
        weights, indices = torch.topk(F.softmax(logits, dim=-1), self.top_k, dim=-1)
        weights = weights / weights.sum(dim=-1, keepdim=True)
        return weights, indices


class SwiGLUExpert(nn.Module):
    """Expert FFN com SwiGLU activation."""

    def __init__(self, d_model: int, d_ff: int):
        super().__init__()
        self.w_gate = nn.Linear(d_model, d_ff, bias=False)
        self.w_up = nn.Linear(d_model, d_ff, bias=False)
        self.w_down = nn.Linear(d_ff, d_model, bias=False)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.w_down(F.silu(self.w_gate(x)) * self.w_up(x))


class HierarchicalMoE(nn.Module):
    """
    Hierarchical Mixture of Experts — dois níveis de routing.

    Arquitetura:
    ┌─────────────────────────────────────────────┐
    │ Input (B, L, D)                              │
    │   ↓                                          │
    │ Coarse Router → top-2 de 4 grupos            │
    │   ↓              ↓                           │
    │ [Grupo A]     [Grupo B]                      │
    │  Fine Router   Fine Router                    │
    │  → top-2/4    → top-2/4                      │
    │   ↓↓            ↓↓                           │
    │ [e0][e1]      [e4][e5]   ... (16 total)     │
    │   ↓↓            ↓↓                           │
    │ Weighted sum + residual                      │
    └─────────────────────────────────────────────┘

    Vantagens sobre MoE flat (v8):
    - Decomposição natural de competências
    - Routing mais interpretável (coarse = categoria, fine = sub-tarefa)
    - Menos competição entre experts distantes
    - 16 experts ativos (2 coarse × 2 fine × 2 tokens) mas apenas 4 por token
    """

    def __init__(self, config: HierarchicalMoEConfig):
        super().__init__()
        self.config = config
        self.total_experts = config.n_coarse_experts * config.n_fine_per_coarse

        # Routers
        self.coarse_router = CoarseRouter(config.d_model, config.n_coarse_experts, config.coarse_top_k)
        self.fine_routers = nn.ModuleList([
            FineRouter(config.d_model, config.n_fine_per_coarse, config.fine_top_k)
            for _ in range(config.n_coarse_experts)
        ])

        # Experts: organizados hierarquicamente
        self.experts = nn.ModuleList()
        for c in range(config.n_coarse_experts):
            for f in range(config.n_fine_per_coarse):
                self.experts.append(SwiGLUExpert(config.d_model, config.d_ff))

        self.norm = nn.RMSNorm(config.d_model, eps=1e-5)

        # Mapeamento: (coarse_idx, fine_idx) → expert_idx global
        self._build_expert_map()

    def _build_expert_map(self):
        self.register_buffer(
            "expert_offset",
            torch.tensor([
                i * self.config.n_fine_per_coarse
                for i in range(self.config.n_coarse_experts)
            ])
        )

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, Dict]:
        """
        Args:
            x: (B, L, D)
        Returns:
            output: (B, L, D)
            info: routing telemetry
        """
        B, L, D = x.shape
        x_norm = self.norm(x)
        output = torch.zeros_like(x)

        # Coarse routing
        c_weights, c_indices = self.coarse_router(x_norm)  # (B, L, top_k)

        routing_log = {}
        expert_usage = {i: 0 for i in range(self.total_experts)}

        for k in range(self.config.coarse_top_k):
            # Tokens atribuídos ao k-ésimo coarse group
            c_idx = c_indices[:, :, k]  # (B, L)
            c_w = c_weights[:, :, k]    # (B, L)

            for c in range(self.config.n_coarse_experts):
                # Máscara: quais posições escolheram este coarse expert
                mask = (c_idx == c)  # (B, L)
                if not mask.any():
                    continue

                # Extrair tokens para este coarse group
                flat_mask = mask.flatten()
                selected = x_norm[flat_mask]  # (N, D)

                # Fine routing dentro do coarse group
                f_weights, f_indices = self.fine_routers[c](selected)  # (N, top_k)

                for fk in range(self.config.fine_top_k):
                    f_idx = f_indices[:, fk]  # (N,)
                    f_w = f_weights[:, fk]    # (N,)

                    for f in range(self.config.n_fine_per_coarse):
                        f_mask = (f_idx == f)
                        if not f_mask.any():
                            continue

                        # Expert global index
                        e_global = c * self.config.n_fine_per_coarse + f
                        expert_usage[e_global] += f_mask.sum().item()

                        # Processar tokens
                        expert_input = selected[f_mask]
                        expert_output = self.experts[e_global](expert_input)

                        # Aplicar pesos: coarse × fine
                        combined_w = c_w[flat_mask][f_mask] * f_w[f_mask]
                        weighted = expert_output * combined_w.unsqueeze(-1)

                        # Scatter back
                        output[mask] += weighted

        # Compute load balance
        total_tokens = B * L
        usage_fractions = {i: v / total_tokens for i, v in expert_usage.items()}
        active_experts = sum(1 for v in expert_usage.values() if v > 0)
        mean_usage = sum(usage_fractions.values()) / max(active_experts, 1)
        std_usage = (sum((f - mean_usage) ** 2 for f in usage_fractions.values())
                     / max(active_experts, 1)) ** 0.5
        balance = max(0.0, 1.0 - std_usage / max(mean_usage, 1e-8))

        info = {
            "module": "HierarchicalMoE",
            "seal": "HIER-MOE-v9.0.0-2026-01-15",
            "active_experts": active_experts,
            "total_experts": self.total_experts,
            "load_balance": balance,
            "expert_usage": usage_fractions,
            "routing": "hierarchical_%s" % self.config.routing_type,
        }

        return output, info

    def get_telemetry(self) -> dict:
        return {
            "module": "HierarchicalMoE",
            "version": "9.0.0",
            "substrate": "v9-backbone",
            "seal": "HIER-MOE-v9.0.0-2026-01-15",
            "n_coarse": self.config.n_coarse_experts,
            "n_fine_per_coarse": self.config.n_fine_per_coarse,
            "total_experts": self.total_experts,
            "active_per_token": self.config.coarse_top_k * self.config.fine_top_k,
            "coarse_names": self.config.coarse_names,
        }
