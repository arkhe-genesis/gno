import math
import numpy as np
from cathedral_governance.discourse import DiscourseDetector, DiscourseType

class CathedralMCTSNode:
    def __init__(self, state_hash, prior_prob=1.0):
        self.state_hash = state_hash
        self.prior_prob = prior_prob
        self.children = []
        self.value = 0.0

class MCTSPlanner:
    def __init__(self, ontology_graph, detector: DiscourseDetector):
        self.ontology = ontology_graph
        self.detector = detector

    def select_action(self, node: CathedralMCTSNode, c_param: float = 1.41):
        # UCB1 (Upper Confidence Bound) modulado pelo DiscourseDetector
        exploit = node.value / (node.prior_prob + 1e-8)
        explore = c_param * math.sqrt(math.log(node.prior_prob + 1) / (node.prior_prob + 1e-8))

        discourse_penalty = 0.0
        if self.detector.is_path_dangerous(node.state_hash):
            discourse_penalty = 10.0 # Penalidade severa para caminhos patológicos (Discurso Mestre/Capitalista)

        ucb_score = exploit + explore - discourse_penalty
        return ucb_score

    def expand(self, node: CathedralMCTSNode):
        # Gera estados futuros baseados em conceitos vizinhos no Onto-Cathedral
        neighbors = self.ontology.get_neighbors(node.state_hash)
        for neighbor in neighbors:
            child = CathedralMCTSNode(state_hash=neighbor.id, prior_prob=node.prior_prob * (1/len(neighbors)))
            node.children.append(child)
