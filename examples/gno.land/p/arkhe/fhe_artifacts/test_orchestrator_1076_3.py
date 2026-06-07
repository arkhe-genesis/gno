import pytest
import numpy as np
from orchestrator_1076_3 import OrchestratorRSI

def test_orchestrator_run_cycle():
    orch = OrchestratorRSI()
    X = np.random.rand(10, 2)
    dX = np.random.rand(10, 2)

    result = orch.run_cycle(X, dX, "dx_dt")

    assert result.iteration == 1
    assert result.discovery['label'] == "dx_dt"
    assert result.discovery['equation'] == "1.0000·x1"
    assert result.proof['verified'] == True
    assert "dx_dt" in result.patch['patch_code']
    assert result.verification['all_passed'] == True
    assert result.deployed == True
    assert len(orch.system_state['substrates']) == 1

def test_orchestrator_metrics():
    orch = OrchestratorRSI()
    X = np.random.rand(10, 2)
    dX = np.random.rand(10, 2)

    orch.run_cycle(X, dX, "dx_dt")
    metrics = orch.export_metrics()

    assert metrics['substrate'] == '1076.3'
    assert metrics['total_cycles'] == 1
    assert metrics['successful_deploys'] == 1

def test_orchestrator_low_theosis():
    orch = OrchestratorRSI(theosis_threshold=0.99)
    X = np.random.rand(10, 2)
    dX = np.random.rand(10, 2)

    # "dx_dt" has theosis 0.9571
    result = orch.run_cycle(X, dX, "dx_dt")

    # Should not deploy because theosis 0.9571 < 0.99
    assert result.deployed == False
    assert len(orch.system_state['substrates']) == 0
