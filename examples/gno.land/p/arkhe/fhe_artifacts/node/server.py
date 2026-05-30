from passport_gateway import PassportGateway
from api_gateway import APIGateway
import time

class ArkheNode:
    def __init__(self, config_path: str = "config.yaml"):
        self.node_id = "node-1"
        self.config = {"passport_enabled": True}
        self.passport = PassportGateway()
        self.api = APIGateway(node_id=self.node_id, passport=self.passport)

    def start(self):
        print(f"Starting ArkheNode {self.node_id}...")
        if self.config.get("passport_enabled", True):
            self.passport.start()
        self.api.start_http_server()
        print("ArkheNode started.")
        while True:
            time.sleep(1)

if __name__ == "__main__":
    node = ArkheNode()
    node.start()
