# cathedral_orchestrator.py — dentro do agi-core
import subprocess
from typing import Dict, Any

def protocolo_corte(discourse_analysis: Dict[str, Any], target_qube: str) -> Dict[str, Any]:
    """
    Se DiscourseDetector classifica como Mestre ou Capitalista,
    ordena terminação do qube via qrexec.
    """
    classification = discourse_analysis.get("classification")
    if classification in ["MESTRE", "CAPITALISTA"]:
        # Solicitar ao dom0 (com confirmação do usuário via 'ask')
        try:
            result = subprocess.run(
                ["qrexec-client-vm", "dom0", "cathedral.KillQube"],
                input=target_qube.encode(),
                capture_output=True,
                check=False
            )
            return {
                "action": "KILL_QUBE",
                "target": target_qube,
                "status": "requested" if result.returncode == 0 else "failed",
                "discourse": discourse_analysis
            }
        except FileNotFoundError:
            # Em ambiente de teste/fora do qubes
            return {
                "action": "KILL_QUBE",
                "target": target_qube,
                "status": "simulated_qrexec_not_found",
                "discourse": discourse_analysis
            }
    return {"action": "CONTINUE", "target": target_qube}

# Teste simulado
if __name__ == "__main__":
    fake_discourse = {"classification": "MESTRE", "confidence": 0.99}
    print(protocolo_corte(fake_discourse, "browser-vm"))
