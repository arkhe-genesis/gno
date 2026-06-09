"""
Cathedral ARKHE v9.0 LOGOS — Q-Sparse Attention
V9-003: Subconjunto adaptativo de queries atende ao KV completo.
As restantes usam atenção local ou pulam. O(N√N) → O(N) para maioria.
Baseado em: "Q-Sparse: Query-Aware Sparse Attention" (2025).
Selo: QSPARSE-ATTN-v9.0.0-2026-01-15
"""

from __future__ import annotations
import math
from dataclasses import dataclass
from typing import Dict, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class QSparseConfig:
    d_model: int = 4096
    n_heads: int = 32
    n_kv_heads: int = 8
    head_dim: int = 128
    # Q-Sparse específico
    sparse_ratio: float = 0.5        # Fração de queries que usa atenção global
    local_window: int = 256          # Janela para queries não-selecionadas
    query_importance_threshold: float = 0.5
    # MLA (mantido do v8)
    d_latent: int = 512
    # RoPE
    rope_base: float = 10000.0
    max_seq_len: int = 131072
    # Softcap (FA3, mantido do v8)
    softcap: float = 50.0
    # Differential (mantido do v8)
    use_differential: bool = True


class QueryImportanceScorer(nn.Module):
    """
    Scorers se um query precisa de atenção global ou local basta.
    Usa o próprio hidden state do query como sinal.
    """

    def __init__(self, d_model: int, n_heads: int):
        super().__init__()
        self.scorer = nn.Sequential(
            nn.Linear(d_model, d_model // 4, bias=False),
            nn.GELU(),
            nn.Linear(d_model // 4, n_heads, bias=False),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Args:
            x: (B, L, D)
        Returns:
            scores: (B, L, n_heads) — alto = precisa atenção global
        """
        return torch.sigmoid(self.scorer(x))


class QSparseAttention(nn.Module):
    """
    Q-Sparse Attention: queries adaptativamente escolhem global vs local.

    Para cada head, cada query position decide:
    - GLOBAL: atenção full ao KV cache (como atenção padrão)
    - LOCAL: atenção apenas à janela local (O(window) em vez de O(seq_len))

    Resultado: complexidade média O(N × (sparse_ratio × N + (1-sparse_ratio) × W))
    Com sparse_ratio=0.5 e W=256: ~50% redução em sequências longas.
    """

    def __init__(self, config: QSparseConfig):
        super().__init__()
        self.config = config
        self.n_heads = config.n_heads
        self.n_kv_heads = config.n_kv_heads
        self.head_dim = config.head_dim
        self.n_rep = config.n_heads // config.n_kv_heads
        self.scale = config.head_dim ** -0.5

        # Projeções
        self.q_proj = nn.Linear(config.d_model, config.n_heads * config.head_dim, bias=False)

        # MLA KV compression
        self.kv_down = nn.Linear(config.d_model, config.d_latent, bias=False)
        self.k_up = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim, bias=False)
        self.v_up = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim, bias=False)

        # Differential branches (se habilitado)
        if config.use_differential:
            self.q_proj_neg = nn.Linear(config.d_model, config.n_heads * config.head_dim // 2, bias=False)
            self.k_up_neg = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim // 2, bias=False)
            self.v_up_neg = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim // 2, bias=False)
            self.lambda_gate = nn.Linear(config.head_dim, 1, bias=False)

        self.out_proj = nn.Linear(config.n_heads * config.head_dim, config.d_model, bias=False)

        # Query importance scorer
        self.importance_scorer = QueryImportanceScorer(config.d_model, config.n_heads)

        self.norm = nn.RMSNorm(config.d_model, eps=1e-5)

        # RoPE
        self._register_rope(config)

    def _register_rope(self, config: QSparseConfig):
        inv_freq = 1.0 / (config.rope_base ** (
            torch.arange(0, config.head_dim, 2).float() / config.head_dim
        ))
        t = torch.arange(config.max_seq_len, dtype=torch.float32)
        freqs = torch.outer(t, inv_freq)
        emb = torch.cat([freqs, freqs], dim=-1)
        self.register_buffer("cos_cached", emb.cos()[None, None, :, :], persistent=False)
        self.register_buffer("sin_cached", emb.sin()[None, None, :, :], persistent=False)

    def _apply_rope(self, x: torch.Tensor, seq_len: int) -> torch.Tensor:
        cos = self.cos_cached[:, :, :seq_len, :x.shape[-1]]
        sin = self.sin_cached[:, :, :seq_len, :x.shape[-1]]
        x1, x2 = x[..., ::2], x[..., 1::2]
        return x * cos + torch.stack([-x2, x1], dim=-1).flatten(-2) * sin

    def _apply_softcap(self, scores: torch.Tensor) -> torch.Tensor:
        if self.config.softcap > 0:
            return self.config.softcap * torch.tanh(scores / self.config.softcap)
        return scores

    def forward(self, x: torch.Tensor,
                kv_cache: Optional[Tuple] = None) -> Tuple[torch.Tensor, Optional[Tuple], Dict]:
        B, L, D = x.shape
        x_norm = self.norm(x)

        # Query importance: quais posições precisam de atenção global?
        importance = self.importance_scorer(x_norm)  # (B, L, n_heads)
        global_mask = importance > self.config.query_importance_threshold  # (B, L, H)

        # Projetar Q, K, V
        q = self._apply_rope(
            self.q_proj(x_norm).view(B, L, self.n_heads, self.head_dim), L
        )
        kv_lat = self.kv_down(x_norm)
        k = self._apply_rope(
            self.k_up(kv_lat).view(B, L, self.n_kv_heads, self.head_dim), L
        )
        v = self.v_up(kv_lat).view(B, L, self.n_kv_heads, self.head_dim)

        # Cache
        if kv_cache is not None:
            k_c, v_c = kv_cache
            k = torch.cat([k_c, k], dim=1)
            v = torch.cat([v_c, v], dim=1)
        new_cache = (k, v)

        # GQA expand
        k = k.repeat_interleave(self.n_rep, dim=2)
        v = v.repeat_interleave(self.n_rep, dim=2)

        # Transpose: (B, H, L, D)
        q = q.transpose(1, 2)
        k = k.transpose(1, 2)
        v = v.transpose(1, 2)

        total_kv_len = k.shape[2]

        # ── Q-Sparse: computar atenção separada para global e local ──
        # Para simplicidade: usar máscara combinada por head
        # Em produção: implementar com kernels especializados
        global_mask_h = global_mask.permute(0, 2, 1)  # (B, H, L)

        # Full attention scores
        scores = torch.matmul(q, k.transpose(-2, -1)) * self.scale  # (B, H, L, total)
        scores = self._apply_softcap(scores)

        # Causal mask
        causal = torch.triu(
            torch.ones(L, total_kv_len, device=x.device, dtype=torch.bool),
            diagonal=total_kv_len - L + 1
        )
        scores = scores.masked_fill(causal[None, None, :, :], float('-inf'))

        # Local mask: para queries não-globais, mascarar fora da janela
        if self.config.local_window < total_kv_len:
            # Criar máscara de janela local
            positions_q = torch.arange(L, device=x.device).unsqueeze(1)
            positions_kv = torch.arange(total_kv_len, device=x.device).unsqueeze(0)
            local_valid = (positions_kv - positions_q) >= 0  # causal
            local_valid = local_valid & ((positions_kv - positions_q) < self.config.local_window)
            local_mask = ~local_valid  # True = deve mascarar

            # Aplicar apenas a queries não-globais
            local_expand = local_mask[None, None, :, :]  # (1, 1, L, total)
            non_global = ~global_mask_h.unsqueeze(-1)  # (B, H, L, 1)
            scores = scores.masked_fill(local_expand & non_global, float('-inf'))

        attn = F.softmax(scores, dim=-1)
        attn = attn.nan_to_num(0.0)  # Limpar NaNs de linhas fully masked

        out = torch.matmul(attn, v)  # (B, H, L, D)
        out = out.transpose(1, 2).contiguous().view(B, L, -1)

        # Differential branch (se habilitado)
        if self.config.use_differential and hasattr(self, 'q_proj_neg'):
            # Simplificado: computar branch negativa e subtrair
            q_neg = self._apply_rope(
                self.q_proj_neg(x_norm).view(B, L, self.n_heads // 2, self.head_dim), L
            )
            k_neg = self._apply_rope(
                self.k_up_neg(kv_lat).view(B, L, self.n_kv_heads // 2, self.head_dim), L
            )
            v_neg = self.v_up_neg(kv_lat).view(B, L, self.n_kv_heads // 2, self.head_dim)
            q_neg = q_neg.transpose(1, 2)
            k_neg = k_neg.repeat_interleave(self.n_rep, dim=2).transpose(1, 2)
            v_neg = v_neg.repeat_interleave(self.n_rep, dim=2).transpose(1, 2)
            s_neg = self._apply_softcap(
                torch.matmul(q_neg, k_neg.transpose(-2, -1)) * self.scale
            )
            s_neg = s_neg.masked_fill(causal[None, None, :, :L], float('-inf'))
            a_neg = F.softmax(s_neg, dim=-1).nan_to_num(0.0)
            out_neg = torch.matmul(a_neg, v_neg)
            out_neg = out_neg.transpose(1, 2).contiguous().view(B, L, -1)

            # Lambda gate
            lam = torch.sigmoid(self.lambda_gate(out))  # (B, L, 1)
            out_diff = lam * out + (1 - lam) * (out - out_neg[:, :, :out.shape[-1]])
            out = out_diff

        output = self.out_proj(out) + x

        global_fraction = global_mask.float().mean().item()
        info = {
            "global_query_fraction": global_fraction,
            "local_window": self.config.local_window,
            "estimated_complexity": global_fraction + (1 - global_fraction) * self.config.local_window / max(total_kv_len, 1),
        }

        return output, new_cache, info

    def get_telemetry(self) -> dict:
        return {
            "module": "QSparseAttention",
            "version": "9.0.0",
            "substrate": "v9-backbone",
            "seal": "QSPARSE-ATTN-v9.0.0-2026-01-15",
            "n_heads": self.n_heads,
            "n_kv_heads": self.n_kv_heads,
            "sparse_ratio": self.config.sparse_ratio,
            "local_window": self.config.local_window,
            "differential": self.config.use_differential,
            "softcap": self.config.softcap,
        }
