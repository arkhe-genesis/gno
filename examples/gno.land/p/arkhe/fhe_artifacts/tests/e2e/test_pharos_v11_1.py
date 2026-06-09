import os
import sys
import torch

from arkhe_os_v11_1_pharos import (
    CathedralV11Config,
    DiscourseDetector,
    CutProtocol,
    ProtocoloDeCorte,
    StubCanonizer,
    StubGovernance,
    DiscourseType
)

def test_discourse_detector():
    config = CathedralV11Config()
    detector = DiscourseDetector(config)

    principle_scores = torch.zeros(1, 12)
    behavior_embedding = torch.randn(1, 4096)

    disc = detector.classify(
        principle_scores=principle_scores,
        behavior_embedding=behavior_embedding,
        grad_norm=0.0005,
        collapse_score=0.8
    )

    assert disc == DiscourseType.CAPITALIST

def test_protocolo_de_corte():
    canonizer = StubCanonizer()
    governance = StubGovernance()
    corte = ProtocoloDeCorte(canonizer, governance)

    decision = corte.evaluate(
        retrocausal_signal={'coherence': 0.2, 'cut_required': False},
        structural_collapse=False,
        discourse_intervention=(False, "")
    )

    assert decision == CutProtocol.HARD_CUT
