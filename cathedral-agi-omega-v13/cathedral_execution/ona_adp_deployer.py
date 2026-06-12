"""
cathedral_execution/ona_adp_deployer.py
Integração real com a API do Management Plane do ONA.
Lê o ADP e provisiona o ambiente de execução seguro.
Selo: ONA-ADP-INTEGRATION-v1.0.0-2026-06-11
"""
import requests
import json
import yaml
import time
import polling # pip install polling
from pathlib import Path

class ONAADPDeployer:
    def __init__(self, management_api_url: str, api_key: str):
        self.url = management_api_url.rstrip("/")
        self.headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}

    def deploy_cathedral(self, adp_path: str, github_repo_url: str) -> dict:
        """Faz o deploy do manifesto Cathedral no ONA."""
        with open(adp_path, 'r') as f:
            manifest_yaml = f.read()

        payload = {
            "name": f"Cathedral-{int(time.time())}",
            "description": "Agente Autônomo Soberano com verificação formal (Lean 4) e governança K-de-N.",
            "adp_content": manifest_yaml,
            "source_repo": github_repo_url,
            "runner_config": {
                "image": "ubuntu:22.04",
                "resources": {"cpu": "4", "memory": "16Gi"},
                "security": {
                    "veto": {
                        "network_control": {"default_action": "DENY", "allowed_endpoints": ["localhost:8545"]},
                        "executable_control": {"mode": "allow_list", "hashes": ["sha256:PLACEHOLDER_CATHEDRAL_HASH"]}
                    }
                }
            }
        }

        print(f"[ONA] Enviando manifesto Cathedral para o Management Plane...")
        response = requests.post(f"{self.url}/api/v1/workspaces", headers=self.headers, json=payload)
        response.raise_for_status()
        workspace_id = response.json().get("id")

        print(f"[ONA] Workspace criado: {workspace_id}. Aguardando provisionamento...")

        # Polling até o runner estar pronto
        status_url = f"{self.url}/api/v1/workspaces/{workspace_id}/status"
        for _ in range(60): # Timeout de 5 minutos
            time.sleep(5)
            status_res = requests.get(status_url, headers=self.headers).json()
            if status_res.get("status") == "ready":
                print(f"[ONA] Runner pronto! ID: {workspace_id}")
                return status_res

        raise TimeoutError("Timeout aguardando provisionamento do runner ONA.")

    def trigger_task(self, workspace_id: str, task_command: str):
        """Dispara uma tarefa no runner ONA já provisionado."""
        payload = {
            "action": "execute_command",
            "command": task_command
        }
        requests.post(f"{self.url}/api/v1/workspaces/{workspace_id}/execute",
                   headers=self.headers, json=payload)
