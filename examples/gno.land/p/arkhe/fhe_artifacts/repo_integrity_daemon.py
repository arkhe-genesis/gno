#!/usr/bin/env python3
# Monitora novos pacotes em PyPI, npm, Crates.io em busca de nomes suspeitos
import requests
import hashlib
import time
import json

SUSPICIOUS_PATTERNS = [
    "security", "wallet", "auditor", "defi", "risk", "scanner",
    "checker", "validator", "protector", "guard", "shield"
]

class RepoIntegrityDaemon:
    def __init__(self, webhook_url=None):
        self.webhook_url = webhook_url
        self.known_bad = set()

    def scan_pypi(self):
        """Consulta novos projetos PyPI (via RSS/JSON API) e analisa nomes."""
        # Exemplo: feed de novos projetos
        try:
            resp = requests.get("https://pypi.org/rss/packages.xml", timeout=10)
        except Exception:
            pass
        # ... parsing ...
        new_packages = ["wallet-security-checker", "eth-security-auditor"]  # simulado
        for pkg in new_packages:
            if any(pattern in pkg.lower() for pattern in SUSPICIOUS_PATTERNS):
                self.flag_package(pkg, "PyPI")

    def flag_package(self, name, registry):
        seal = hashlib.sha3_256(f"{name}:{registry}".encode()).hexdigest()[:16]
        alert = f"[ALERTA] Pacote suspeito detectado: {name} ({registry}). Selo: {seal}"
        print(alert)
        # Enviar para Telegraph
        if self.webhook_url:
            try:
                requests.post(self.webhook_url, json={"alert": alert, "seal": seal})
            except Exception:
                pass

# Execução
if __name__ == "__main__":
    daemon = RepoIntegrityDaemon()
    daemon.scan_pypi()
