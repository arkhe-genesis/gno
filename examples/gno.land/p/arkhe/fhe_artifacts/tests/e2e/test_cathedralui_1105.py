import os
import sys

from cathedralui_1105 import CathedralUI

class MockOrchestrator:
    def __init__(self):
        self.onchain = type('obj', (object,), {
            'memory_lake': type('obj', (object,), {
                'get_merkle_root': lambda *args, **kwargs: 'mocked_merkle_root',
                'entries': []
            })(),
            'proof_chain': type('obj', (object,), {
                'tip_hash': 'mocked_tip_hash',
                'nodes': []
            })(),
            'kernel_payload': None
        })()

        self.hashtree_gov = type('obj', (object,), {
            'canonizer': type('obj', (object,), {
                'node_client': type('obj', (object,), {
                    'get_canonical_state': lambda *args, **kwargs: {"status": "mocked_canonical_state"}
                })()
            })(),
            '_proposals': {},
            '_decisions': [],
            'config': type('obj', (object,), {
                'multi_sig_threshold': 3
            })()
        })()

        self.safety = type('obj', (object,), {
            'get_telemetry': lambda *args, **kwargs: {"risk_level": "low"},
            'capability_monitor': type('obj', (object,), {
                '_cusum': {},
                '_anomaly_scores': []
            })()
        })()

        self.theosis_rl = type('obj', (object,), {
            'policy': {
                'gate_sensitivity_multiplier': 1.0,
                'roleplay_resistance': 0.0,
                'refusal_bias': 0.3,
                'memory_weight': 0.4
            },
            'rewards': [],
            'step': 0
        })()

        self.eco_health = 0.95
        self.containment_mode = False
        self.cycle_count = 1

def test_cathedralui_render():
    orchestrator = MockOrchestrator()
    ui = CathedralUI(orchestrator)
    html = ui.render_html()
    assert html is not None
    assert "Cathedral ARKHE v10.1 — LOGOS Dashboard" in html
    assert "mocked_merkle_root" in html

def test_cathedralui_save_dashboard():
    orchestrator = MockOrchestrator()
    ui = CathedralUI(orchestrator)
    path = "test_cathedralui_1105.html"
    res = ui.save_dashboard(path)
    assert res == path
    assert os.path.exists(path)
    os.remove(path)
