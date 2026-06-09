"""
Cathedral ARKHE v9.0 LOGOS — Multi-Token Prediction Heads
V9-002: Prediz múltiplos tokens futuros durante treino.
Melhora sample efficiency e habilita speculative decoding nativo.
Baseado em: Meta "Efficient Multi-Token Prediction" (2024), Llama 4.
Selo: MTP-v9.0.0-2026-01-15
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class MultiTokenPredConfig:
    d_model: int = 4096
    vocab_size: int = 128256
    n_future_tokens: int = 4          # Prever +1, +2, +3, +4 tokens
    shared_embedding: bool = True      # Compartilhar embedding com LM head
    # Pesos de loss (tokens mais distantes têm menos peso)
    loss_weights: list = None          # [1.0, 0.8, 0.6, 0.4]
    # Arquitetura dos prediction heads
    head_depth: int = 2                # Profundidade de cada head
    head_hidden: int = 1024            # Dimensão interna

    def __post_init__(self):
        if self.loss_weights is None:
            self.loss_weights = [1.0 / (i + 1) for i in range(self.n_future_tokens)]


class MultiTokenPredictionHead(nn.Module):
    """
    Head que prediz o token na posição +offset.
    Cada head é uma MLP rasa: hidden → hidden → vocab.
    """

    def __init__(self, d_model: int, vocab_size: int,
                 hidden_dim: int, n_layers: int, offset: int):
        super().__init__()
        self.offset = offset

        layers = []
        in_d = d_model
        for _ in range(n_layers):
            layers.append(nn.Linear(in_d, hidden_dim, bias=False))
            layers.append(nn.GELU())
            in_d = hidden_dim
        layers.append(nn.Linear(in_d, vocab_size, bias=False))

        self.net = nn.Sequential(*layers)

    def forward(self, hidden: torch.Tensor) -> torch.Tensor:
        """
        Args:
            hidden: (B, L, D) — hidden states na posição t
        Returns:
            logits: (B, L, V) — logits para token na posição t+offset
        """
        return self.net(hidden)


class MultiTokenPredictionLoss(nn.Module):
    """
    Loss combinada para múltiplos tokens futuros.
    Usa weight sharing: todos os heads compartilham o embedding final.
    """

    def __init__(self, config: MultiTokenPredConfig):
        super().__init__()
        self.config = config
        self.weights = torch.tensor(config.loss_weights, dtype=torch.float32)
        # Normalizar pesos
        self.weights = self.weights / self.weights.sum()

    def forward(self, predictions: List[torch.Tensor],
                targets: torch.Tensor,
                ignore_index: int = -100) -> Tuple[torch.Tensor, Dict]:
        """
        Args:
            predictions: lista de [n_future_tokens] tensores (B, L, V)
            targets: (B, L + n_future_tokens) — tokens reais
            ignore_index: índice de padding

        Returns:
            total_loss: escalar
            per_head: dict com loss por head
        """
        B, L, V = predictions[0].shape
        device = predictions[0].device
        weights = self.weights.to(device)

        total_loss = torch.tensor(0.0, device=device)
        per_head = {}

        for i, pred_logits in enumerate(predictions):
            # Target para esta head: tokens[i+1 : i+1+L]
            target_slice = targets[:, i + 1: i + 1 + L]  # (B, L)

            loss = F.cross_entropy(
                pred_logits.reshape(-1, V),
                target_slice.reshape(-1),
                ignore_index=ignore_index,
                reduction='mean',
            )

            per_head[f"offset_{i + 1}"] = loss.item()
            total_loss = total_loss + weights[i] * loss

        return total_loss, per_head


class MultiTokenPredictionModule(nn.Module):
    """
    Módulo completo de Multi-Token Prediction.

    Integração com o backbone:
    - Durante treino: computa loss para +1, +2, ..., +N tokens
    - Durante inferência: fornece draft tokens para Medusa (v8-006)
    - Compartilha embedding com LM head principal

    Benefícios:
    - +15-25% sample efficiency (mais sinal por forward)
    - Draft tokens nativos para speculative decoding
    - Melhor representação interna (força o modelo a "planejar")
    """

    def __init__(self, config: MultiTokenPredConfig,
                 shared_embed: Optional[nn.Embedding] = None):
        super().__init__()
        self.config = config

        # Heads de predição
        self.heads = nn.ModuleList([
            MultiTokenPredictionHead(
                d_model=config.d_model,
                vocab_size=config.vocab_size,
                hidden_dim=config.head_hidden,
                n_layers=config.head_depth,
                offset=i + 1,
            )
            for i in range(config.n_future_tokens)
        ])

        # Loss
        self.loss_fn = MultiTokenPredictionLoss(config)

        # Compartilhar embedding (weight tying)
        if shared_embed is not None and config.shared_embedding:
            for head in self.heads:
                # Substituir última camada linear pela embedding transposta
                head.net[-1] = nn.Linear(
                    config.head_hidden, config.vocab_size, bias=False
                )
                # Tie weights
                head.net[-1].weight = shared_embed.weight

    def forward(self, hidden: torch.Tensor,
                targets: Optional[torch.Tensor] = None) -> Dict:
        """
        Args:
            hidden: (B, L, D)
            targets: (B, L + n_future) — para treino

        Returns:
            dict com predictions, loss (se targets fornecido), draft tokens
        """
        predictions = [head(hidden) for head in self.heads]

        result = {"predictions": predictions, "n_heads": len(self.heads)}

        # Se estamos em inferência, extrair draft tokens
        if targets is None:
            draft_tokens = []
            for i, pred in enumerate(predictions):
                probs = F.softmax(pred[:, -1, :], dim=-1)
                token = torch.argmax(probs, dim=-1)  # Greedy para draft
                draft_tokens.append(token)
            result["draft_tokens"] = draft_tokens

        # Se estamos em treino, computar loss
        if targets is not None:
            loss, per_head = self.loss_fn(predictions, targets)
            result["loss"] = loss
            result["per_head_loss"] = per_head

        return result

    def get_draft_for_medusa(self, hidden: torch.Tensor,
                             temperature: float = 0.6) -> List[torch.Tensor]:
        """
        Gera draft tokens para o Medusa decoder (v8-006).
        Cada head prediz uma posição futura.
        """
        draft = []
        for head in self.heads:
            logits = head(hidden[:, -1:, :])  # (B, 1, V)
            probs = F.softmax(logits / temperature, dim=-1)
            token = torch.multinomial(probs.squeeze(1), 1)  # (B, 1)
            draft.append(token)
        return draft

    def get_telemetry(self) -> dict:
        return {
            "module": "MultiTokenPrediction",
            "version": "9.0.0",
            "substrate": "v9-backbone",
            "seal": "MTP-v9.0.0-2026-01-15",
            "n_future_tokens": self.config.n_future_tokens,
            "loss_weights": self.config.loss_weights,
            "shared_embedding": self.config.shared_embedding,
            "head_depth": self.config.head_depth,
        }
