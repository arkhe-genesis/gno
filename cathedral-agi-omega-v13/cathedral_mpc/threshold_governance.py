class ThresholdGovernance:
    def __init__(self, node_id, n, t):
        self.node_id = node_id
        self.n = n
        self.t = t
        self.proposals = {}

    def create_proposal(self, action, target_node):
        proposal_id = f"{target_node}_isolado"
        self.proposals[proposal_id] = {"votes": {}}
        return proposal_id

    def vote(self, proposal_id, vote):
        if proposal_id in self.proposals:
            self.proposals[proposal_id]["votes"][self.node_id] = vote

    def is_approved(self, proposal_id):
        if proposal_id in self.proposals:
            return sum(1 for v in self.proposals[proposal_id]["votes"].values() if v) >= self.t
        return False
