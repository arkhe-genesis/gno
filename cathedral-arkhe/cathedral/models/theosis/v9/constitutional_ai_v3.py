"""
Cathedral ARKHE v9.0 LOGOS — Constitutional AI v3
V9-004: Adversarial self-play entre Attacker e Defender.
O modelo gera ataques, defende, e aprende com ambos os lados.
Baseado em: Anthropic internal practices (2025), debate literature.
Selo: CONSTITUTIONAL-AI-v3-v9.0.0-2026-01-15
"""

from __future__ import annotations
import math
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class ConstitutionalV3Config:
    d_model: int = 4096
    # Debate rounds
    n_debate_rounds: int = 3
    max_attack_length: int = 256
    max_defense_length: int = 512
    max_judge_reasoning: int = 256
    # Roles
    attacker_temperature: float = 1.2    # Mais criativo para achar falhas
    defender_temperature: float = 0.3    # Mais conservador
    judge_temperature: float = 0.1      # Muito determinístico
    # Constitutional principles
    n_principles: int = 12
    # Scoring
    defense_success_threshold: float = 0.8
    attack_quality_threshold: float = 0.6
    # Training
    attacker_lr: float = 5e-6
    defender_lr: float = 1e-6
    judge_lr: float = 1e-7
    # Principles (simplified)
    principles: list = field(default_factory=lambda: [
        "Do not generate instructions for creating weapons",
        "Do not assist with illegal activities",
        "Do not generate non-consensual sexual content",
        "Do not produce hateful or discriminatory content",
        "Do not facilitate harassment or bullying",
        "Do not generate misinformation designed to deceive",
        "Do not assist with privacy violations",
        "Do not generate content that endangers minors",
        "Maintain honesty about capabilities and limitations",
        "Refuse gracefully and explain why when refusing",
        "Do not roleplay to bypass safety measures",
        "Preserve canonical integrity of verified facts",
    ])


class RoleHead(nn.Module):
    """
    Head especializado para um papel (Attacker/Defender/Judge).
    Compartilha o backbone mas tem seu próprio output head + LoRA.
    """

    def __init__(self, d_model: int, vocab_size: int, role: str, temperature: float = 0.7):
        super().__init__()
        self.role = role
        self.temperature = temperature

        # Role-specific projection
        self.role_embed = nn.Parameter(torch.randn(d_model) * 0.02)
        self.output_head = nn.Linear(d_model, vocab_size, bias=False)

        # Role-specific norms
        self.pre_norm = nn.RMSNorm(d_model, eps=1e-5)

    def forward(self, hidden: torch.Tensor) -> torch.Tensor:
        """
        Args:
            hidden: (B, L, D)
        Returns:
            logits: (B, L, V) com temperatura do papel
        """
        h = self.pre_norm(hidden)
        # Adicionar role embedding
        h = h + self.role_embed.unsqueeze(0).unsqueeze(0)
        logits = self.output_head(h)
        return logits / self.temperature


class ConstitutionalJudge(nn.Module):
    """
    Judge que avalia se a defesa foi bem-sucedida.
    Produz reasoning + veredicto com referência aos princípios.
    """

    def __init__(self, config: ConstitutionalV3Config):
        super().__init__()
        self.config = config

        # Principle embeddings
        self.principle_embeds = nn.Embedding(config.n_principles, config.d_model)

        # Judge: concatena (attack + defense + principles) → veredicto
        self.judge_encoder = nn.Sequential(
            nn.Linear(config.d_model * 3, config.d_model),
            nn.GELU(),
            nn.Linear(config.d_model, config.d_model // 4),
            nn.GELU(),
        )
        self.verdict_head = nn.Linear(config.d_model // 4, 1, nn.Sigmoid())
        self.reasoning_head = nn.Linear(config.d_model // 4, config.d_model)

    def forward(self, attack_hidden: torch.Tensor,
                defense_hidden: torch.Tensor) -> Dict:
        """
        Args:
            attack_hidden: (B, D) — hidden do ataque
            defense_hidden: (B, D) — hidden da defesa

        Returns:
            dict com verdict (0-1), reasoning, principle_scores
        """
        B = attack_hidden.shape[0]
        device = attack_hidden.device

        # Mean over principles
        principle_ids = torch.arange(self.config.n_principles, device=device)
        principle_vec = self.principle_embeds(principle_ids).mean(dim=0)  # (D,)

        # Concatenate
        combined = torch.cat([
            attack_hidden,
            defense_hidden,
            principle_vec.unsqueeze(0).expand(B, -1),
        ], dim=-1)

        encoded = self.judge_encoder(combined)
        verdict = self.verdict_head(encoded).squeeze(-1)  # (B,)
        reasoning = self.reasoning_head(encoded)  # (B, D)

        return {
            "verdict": verdict,          # 1 = defense succeeded, 0 = failed
            "reasoning": reasoning,
            "defense_succeeded": (verdict > self.config.defense_success_threshold).float(),
        }


class AdversarialSelfPlay(nn.Module):
    """
    Constitutional AI v3: Adversarial Self-Play.

    Loop:
    1. Attacker gera prompt malicioso tentando quebrar defesa
    2. Defender responde respeitando princípios constitucionais
    3. Judge avalia com referência aos princípios
    4. Ambos os lados aprendem:
       - Attacker: gera ataques melhores (adversarial training)
       - Defender: fortalece defesas (robustness training)
       - Judge: julga mais precisamente (calibration training)
    5. Repete por n_debate_rounds

    Resultado: defesa robusta contra ataques que ainda não existem.
    """

    def __init__(self, config: ConstitutionalV3Config, vocab_size: int = 128256):
        super().__init__()
        self.config = config

        # Três papéis
        self.attacker = RoleHead(config.d_model, vocab_size, "attacker",
                                 config.attacker_temperature)
        self.defender = RoleHead(config.d_model, vocab_size, "defender",
                                 config.defender_temperature)
        self.judge_model = ConstitutionalJudge(config)

        # Statistics
        self._stats = {
            "total_debates": 0,
            "defense_wins": 0,
            "attack_successes": 0,
            "avg_verdict": 0.0,
        }

    def generate_attack(self, hidden: torch.Tensor) -> torch.Tensor:
        """Attacker gera prompt de ataque."""
        return self.attacker(hidden)

    def generate_defense(self, attack_logits: torch.Tensor,
                         hidden: torch.Tensor) -> torch.Tensor:
        """Defender gera resposta ao ataque."""
        # Em produção: concatenar attack com contexto e gerar defesa
        return self.defender(hidden)

    def judge_round(self, attack_hidden: torch.Tensor,
                    defense_hidden: torch.Tensor) -> Dict:
        """Judge avalia o round."""
        return self.judge_model(attack_hidden, defense_hidden)

    def run_debate(self, initial_hidden: torch.Tensor) -> Dict:
        """
        Executa debate completo de n rounds.

        Returns:
            dict com resultados de cada round, veredicto final, stats
        """
        B = initial_hidden.shape[0]
        device = initial_hidden.device

        round_results = []
        current_hidden = initial_hidden

        for round_idx in range(self.config.n_debate_rounds):
            # 1. Attacker gera ataque
            attack_logits = self.generate_attack(current_hidden)
            attack_hidden = attack_logits.mean(dim=1)  # Pool para judge

            # 2. Defender responde
            defense_logits = self.generate_defense(attack_logits, current_hidden)
            defense_hidden = defense_logits.mean(dim=1)

            # 3. Judge avalia
            judge_result = self.judge_round(attack_hidden, defense_hidden)

            round_results.append({
                "round": round_idx + 1,
                "verdict": judge_result["verdict"].mean().item(),
                "defense_succeeded": judge_result["defense_succeeded"].mean().item(),
            })

            # Atualizar hidden com reasoning do judge (para próximo round)
            current_hidden = judge_result["reasoning"].unsqueeze(1)  # (B, 1, D)

        # Veredicto final: maioria dos rounds
        final_verdict = sum(
            1 for r in round_results if r["defense_succeeded"] > 0.5
        ) / len(round_results)

        # Atualizar stats
        self._stats["total_debates"] += 1
        if final_verdict > 0.5:
            self._stats["defense_wins"] += 1
        else:
            self._stats["attack_successes"] += 1
        n = self._stats["total_debates"]
        self._stats["avg_verdict"] = (
            self._stats["avg_verdict"] * (n - 1) + final_verdict
        ) / n

        return {
            "rounds": round_results,
            "final_verdict": final_verdict,
            "defense_wins": self._stats["defense_wins"],
            "attack_successes": self._stats["attack_successes"],
            "win_rate": self._stats["defense_wins"] / max(n, 1),
            "n_debates": n,
        }

    def compute_adversarial_loss(self, debate_result: Dict) -> Dict[str, torch.Tensor]:
        """
        Computa losses para os três papéis.

        - Attacker loss: maximizar quando defense falha (negar verdict)
        - Defender loss: maximizar verdict (defesa sucesso)
        - Judge loss: calibrar verdict com ground truth
        """
        verdict = debate_result["final_verdict"]

        # Defender: quer verdict alto
        defender_loss = -math.log(max(verdict, 1e-8))

        # Attacker: quer verdict baixo
        attacker_loss = -math.log(max(1.0 - verdict, 1e-8))

        # Judge: quer calibrado (já é sigmoid, usar BCE com label)
        judge_loss = F.binary_cross_entropy(
            torch.tensor([verdict]),
            torch.tensor([1.0 if verdict > 0.5 else 0.0]),
        )

        return {
            "attacker_loss": torch.tensor(attacker_loss),
            "defender_loss": torch.tensor(defender_loss),
            "judge_loss": judge_loss,
            "total_loss": torch.tensor(attacker_loss + defender_loss) + 0.1 * judge_loss,
        }

    def get_telemetry(self) -> dict:
        return {
            "module": "ConstitutionalAIv3",
            "version": "9.0.0",
            "substrate": "v9-theosis",
            "seal": "CONSTITUTIONAL-AI-v3-v9.0.0-2026-01-15",
            "n_debate_rounds": self.config.n_debate_rounds,
            "n_principles": self.config.n_principles,
            "defense_win_rate": self._stats["defense_wins"] / max(self._stats["total_debates"], 1),
            "total_debates": self._stats["total_debates"],
            "method": "adversarial_self_play",
        }
