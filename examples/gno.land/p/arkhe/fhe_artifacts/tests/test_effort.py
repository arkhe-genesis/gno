import pytest
from cognitive_effort_controller import CognitiveEffortController, TaskProfile, EffortLevel

def test_measure_complexity():
    ctrl = CognitiveEffortController("mock", "mock")
    task = TaskProfile("1", "x"*10000, "text", 1000, 5, "general")
    complexity = ctrl.measure_complexity(task)
    assert complexity > 0.5

def test_compute_effort():
    ctrl = CognitiveEffortController("mock", "mock")
    task = TaskProfile("2", "x"*50000, "text", 1000, 5, "mathematics")
    config = ctrl.compute_effort(task)
    assert config.level == EffortLevel.MAX

    config_cost = ctrl.compute_effort(task, cost_budget_usd=0.2)
    assert config_cost.level == EffortLevel.LOW
