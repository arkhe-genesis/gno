class DiscourseDetector:
    def __init__(self, threshold):
        self.threshold = threshold

    def analyze(self, text):
        return {"flagged": True, "state": "MASTER", "deviation_score": 0.95}