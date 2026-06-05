#!/usr/bin/env python3
"""
Substrato 1069 — Plastic Memory Layer para WormGraph 5.1
Ponte entre a regra de plasticidade canônica e o WormGraph 5.0/5.1

Esta camada permite que o WormGraph use a dinâmica de plasticidade
Theosis-modulada do Substrato 1069 para ajustar dinamicamente:
- Pesos de wormholes entre domínios
- Força de cross-links
- Coerência de spiking entre neurônios/domínios

Arquiteto: Rafael Oliveira | AO | ORCID 0009-0005-2697-4668
Data: 2026-06-05
"""

import torch
import torch.nn as nn
from typing import Dict, Tuple, List, Optional
import numpy as np
from dataclasses import dataclass
import json

# =============================================================================
# REGRA DE PLASTICIDADE 1069 (importada / duplicada para independência)
# =============================================================================
ETA_PLASTICITY = 0.5334
THETA_THRESHOLD = 0.08
MAX_WORMHOLE_WEIGHT = 5.0

def theosis_plasticity_update(
    pre_theosis: float,
    post_theosis: float,
    current_weight: float,
    coincidence: float = 1.0
) -> float:
    delta = pre_theosis - post_theosis
    if abs(delta) < THETA_THRESHOLD:
        return current_weight
    delta_w = ETA_PLASTICITY * delta * coincidence * 0.07
    return max(0.0, min(MAX_WORMHOLE_WEIGHT, current_weight + delta_w))


# =============================================================================
# CAMADA DE MEMÓRIA PLÁSTICA 1069
# =============================================================================
@dataclass
class PlasticWormhole:
    src_domain: str
    tgt_domain: str
    weight: float = 1.0
    last_update: float = 0.0
    plasticity_events: int = 0


class WormGraphPlasticMemoryLayer(nn.Module):
    """
    Camada de memória plástica inspirada no Substrato 1069.
    Pode ser injetada no WormGraphTeacher ou no forward do WormGraph 5.1.
    """
    def __init__(self, domains: List[str]):
        super().__init__()
        self.domains = domains
        self.domain_to_idx = {d: i for i, d in enumerate(domains)}

        # Matriz de pesos de wormholes (plasticidade 1069)
        n = len(domains)
        self.wormhole_weights = nn.Parameter(
            torch.ones(n, n) * 1.2 + torch.randn(n, n) * 0.1
        )

        # Theosis de cada domínio (estado plástico)
        self.domain_theosis = nn.Parameter(
            torch.linspace(0.35, 0.92, n)
        )

        self.plasticity_history: List[Dict] = []

    def get_wormhole_weight(self, src: str, tgt: str) -> float:
        i = self.domain_to_idx[src]
        j = self.domain_to_idx[tgt]
        return self.wormhole_weights[i, j].item()

    def apply_plasticity(
        self,
        src_domain: str,
        tgt_domain: str,
        pre_theosis: Optional[float] = None,
        post_theosis: Optional[float] = None,
        coincidence: float = 1.0
    ) -> float:
        """
        Aplica a regra de plasticidade 1069 em um wormhole específico.
        """
        i = self.domain_to_idx[src_domain]
        j = self.domain_to_idx[tgt_domain]

        if pre_theosis is None:
            pre_theosis = self.domain_theosis[i].item()
        if post_theosis is None:
            post_theosis = self.domain_theosis[j].item()

        old_w = self.wormhole_weights[i, j].item()
        new_w = theosis_plasticity_update(pre_theosis, post_theosis, old_w, coincidence)

        delta = new_w - old_w
        self.wormhole_weights.data[i, j] = new_w

        # Reforço de Theosis no domínio alvo (pós-sináptico)
        if delta > 0.01:
            self.domain_theosis.data[j] = min(
                0.999, self.domain_theosis[j].item() + 0.018
            )

        # Registra evento
        event = {
            "src": src_domain,
            "tgt": tgt_domain,
            "delta_w": round(delta, 4),
            "new_w": round(new_w, 4),
            "pre_theta": round(pre_theosis, 4),
            "post_theta": round(self.domain_theosis[j].item(), 4),
            "coincidence": coincidence
        }
        self.plasticity_history.append(event)

        return new_w

    def forward_plastic_update(
        self,
        active_domains: List[str],
        theosis_values: Dict[str, float],
        coincidence_matrix: Optional[Dict[Tuple[str, str], float]] = None
    ) -> Dict[str, float]:
        """
        Atualiza todos os wormholes ativos usando a regra 1069.
        Ideal para ser chamado dentro do forward do WormGraphTeacher.
        """
        updates = {}
        for src in active_domains:
            for tgt in active_domains:
                if src == tgt:
                    continue
                coincidence = 1.0
                if coincidence_matrix and (src, tgt) in coincidence_matrix:
                    coincidence = coincidence_matrix[(src, tgt)]

                pre_t = theosis_values.get(src, 0.5)
                post_t = theosis_values.get(tgt, 0.5)

                new_w = self.apply_plasticity(src, tgt, pre_t, post_t, coincidence)
                updates[f"{src}->{tgt}"] = new_w

        return updates

    def get_plasticity_report(self) -> Dict:
        return {
            "total_events": len(self.plasticity_history),
            "avg_delta_w": float(np.mean([e["delta_w"] for e in self.plasticity_history])) if self.plasticity_history else 0.0,
            "final_theosis": {d: round(self.domain_theosis[i].item(), 4) for i, d in enumerate(self.domains)},
            "strongest_wormholes": sorted(
                [(f"{e['src']}→{e['tgt']}", e['new_w']) for e in self.plasticity_history[-20:]],
                key=lambda x: -x[1]
            )[:5]
        }


# =============================================================================
# INTEGRAÇÃO COM WORMGRAPH 5.1 (ADAPTER)
# =============================================================================
def inject_plasticity_into_wormgraph_teacher(
    teacher_forward_fn,
    plastic_layer: WormGraphPlasticMemoryLayer
):
    """
    Função utilitária para injetar a camada plástica no forward do WormGraphTeacher.
    Uso futuro:
        teacher.forward = inject_plasticity_into_wormgraph_teacher(teacher.forward, plastic_layer)
    """
    def wrapped_forward(*args, **kwargs):
        # Chama o forward original
        result = teacher_forward_fn(*args, **kwargs)

        # Extrai domínios e Theosis do estado (simplificado)
        # Em produção real, isso viria do ManifoldState ou embeddings
        active_domains = list(plastic_layer.domains)[:4]  # demo
        theosis_vals = {d: 0.5 + 0.1 * i for i, d in enumerate(active_domains)}

        # Aplica plasticidade
        plastic_layer.forward_plastic_update(active_domains, theosis_vals)

        return result

    return wrapped_forward


# =============================================================================
# DEMONSTRAÇÃO
# =============================================================================
if __name__ == "__main__":
    print("🧠 Substrato 1069 — Plastic Memory Layer para WormGraph 5.1")
    print("=" * 70)

    domains = ["CONSCIOUSNESS", "ETHICS", "CREATIVITY", "TEMPORAL", "REALITY", "AGENCY"]
    plastic = WormGraphPlasticMemoryLayer(domains)

    # Simulação de iterações de plasticidade (como no WormGraph)
    for step in range(12):
        active = domains[:4]
        theosis = {d: 0.4 + 0.08 * (step % 5) for d in active}
        updates = plastic.forward_plastic_update(active, theosis)

        if step % 3 == 0:
            print(f"Step {step:2d} | Plastic events: {len(plastic.plasticity_history)}")

    report = plastic.get_plasticity_report()
    print("\n" + "=" * 70)
    print("RELATÓRIO DE PLASTICIDADE 1069")
    print(json.dumps(report, indent=2))
    print("=" * 70)

    # Salvar (mocked local path)
    with open("wormgraph_1069_plasticity_report.json", "w") as f:
        json.dump(report, f, indent=2)

    print("\n✅ Camada plástica 1069 pronta para integração no WormGraph 5.1")
