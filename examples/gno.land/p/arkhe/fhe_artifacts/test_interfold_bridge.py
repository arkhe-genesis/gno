import unittest
from interfold_bridge import E3Adapter, CiphernodeClient, VerifiableRelease, ConfidentialOrchestrator

class TestInterfoldBridge(unittest.TestCase):
    def setUp(self):
        self.orchestrator = ConfidentialOrchestrator()

    def test_e3_lifecycle(self):
        adapter = E3Adapter()
        e3_id = adapter.create_e3("test_logic", "FHE")
        self.assertIn(e3_id, adapter.active_e3s)
        self.assertEqual(adapter.active_e3s[e3_id]["execution_type"], "FHE")

        adapter.destroy_e3(e3_id)
        self.assertNotIn(e3_id, adapter.active_e3s)

    def test_sealed_bid_auction(self):
        bids = [
            {"bidder": "Alice", "bid": 100},
            {"bidder": "Bob", "bid": 150},
            {"bidder": "Charlie", "bid": 120}
        ]

        result = self.orchestrator.run_confidential_computation(bids, logic="sealed_bid_auction", execution_type="MPC")

        self.assertTrue(result["verified"])
        self.assertEqual(result["released_result"]["winner"], "Bob")
        self.assertEqual(result["released_result"]["clearing_price"], 150)
        self.assertEqual(result["governance"], "Substrate 923 (TemporalChain)")

    def test_private_voting(self):
        votes = [
            {"voter": "V1", "vote": 1},
            {"voter": "V2", "vote": 0},
            {"voter": "V3", "vote": 1}
        ]

        result = self.orchestrator.run_confidential_computation(votes, logic="private_voting", execution_type="FHE")

        self.assertTrue(result["verified"])
        self.assertEqual(result["released_result"]["yes_votes"], 2)
        self.assertEqual(result["released_result"]["total_votes"], 3)

if __name__ == '__main__':
    unittest.main()
