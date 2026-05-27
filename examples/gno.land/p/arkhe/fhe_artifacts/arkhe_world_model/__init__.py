#!/usr/bin/env python3
# ╔══════════════════════════════════════════════════════════════════╗
# ║  ARKHE WORLD MODEL — Substrato 890                              ║
# ║  Embryonic World Model: Token Grounding → Physics Priors       ║
# ║  → Multimodal Fusion → Embodied Simulation → Causal Reasoning  ║
# ║  → Self-Modeling                                                ║
# ╚══════════════════════════════════════════════════════════════════╝

"""
Arkhe World Model — Package Principal

Este package implementa o Substrato 890 (WORLD-MODEL-EMBRYO),
conforme formalizado na Glosa 252. O modelo de mundo embrionário
contém 6 estágios de maturidade, desde grounding de tokens até
auto-modelagem, com 3 níveis de desenvolvimento (embryo/infant/adult).

Uso:
    from arkhe_world_model import WorldModelEmbryo

    model = WorldModelEmbryo(stage=1, maturity="embryo")
    model.train(data_loader)
    prediction = model.predict(scene_description)

Módulos:
    llm_engine       — Token Grounding (Stage 1)
    physics_priors   — Physics Priors (Stage 2)
    multimodal_fusion— Multimodal Fusion (Stage 3)
    brax_simulator   — Embodied Simulation (Stage 4)
    causal_reasoning — Causal Reasoning (Stage 5)
    self_model       — Self-Modeling (Stage 6)
    losses           — Loss Híbrida (Training Infrastructure)
    rl_policy        — PPO/DreamerV3 (Policy Training)
"""

__version__ = "890.0.0"
__substrate__ = "890"
__status__ = "CANONIZED_SPECULATIVE"
__uncertainty__ = "H=2.0"
__seal__ = "8d4e2f1a9c3b7e5d"
__architect__ = "ORCID 0009-0005-2697-4668"

from .world_model import WorldModelEmbryo
from .llm_engine import ArkheLLMEngine
from .physics_priors import PhysicsPriorsModule
from .multimodal_fusion import MultimodalFusionModule
from .brax_simulator import ArkheBraxSimulator, SimulationConfig
from .causal_reasoning import ArkheCausalReasoner, DifferentiableSCM
from .losses import ArkheHybridLoss, PhysicsConsistencyLoss, ContrastiveWorldLoss
from .rl_policy import ArkheRLPolicy, WorldModelEnv, PPOPolicy

__all__ = [
    "WorldModelEmbryo",
    "ArkheLLMEngine",
    "PhysicsPriorsModule",
    "MultimodalFusionModule",
    "ArkheBraxSimulator",
    "SimulationConfig",
    "ArkheCausalReasoner",
    "DifferentiableSCM",
    "ArkheHybridLoss",
    "PhysicsConsistencyLoss",
    "ContrastiveWorldLoss",
    "ArkheRLPolicy",
    "WorldModelEnv",
    "PPOPolicy",
]
