import pytest
from hashtree_governance_rsi import HashtreeConfig, HashtreeGovernanceBridge, RSISafetyConfig, RSISafetyLayer, RSIRiskLevel
import torch.nn as nn

def test_hashtree_governance_propose():
    config = HashtreeConfig()
    bridge = HashtreeGovernanceBridge(config)

    proposal = bridge.propose_governance_change(
        proposal_id="prop_01",
        description="Test proposal",
        affected_substrates=["sub_1"],
        proposer_npub="npub_tester"
    )

    assert proposal["status"] == "proposed"
    assert proposal["proposal_id"] == "prop_01"

def test_rsi_safety_layer():
    config = RSISafetyConfig(capability_window=2)
    layer = RSISafetyLayer(config)

    # Mock model
    class DummyModel(nn.Module):
        def __init__(self):
            super().__init__()
            self.linear = nn.Linear(10, 10)

    model = DummyModel()

    res = layer.pre_inference_check(model, {"theosis_avg": 0.5}, [])
    assert res["allowed"] == True

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
