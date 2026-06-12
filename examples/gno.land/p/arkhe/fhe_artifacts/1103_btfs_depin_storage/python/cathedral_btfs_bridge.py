import json
import subprocess
import base64
from typing import Dict, Any, Optional

class BTFSBridge:
    """Cliente qrexec para o serviço BTFS no qube btfs-gateway."""

    def __init__(self, target_qube: str = "btfs-gateway"):
        self.target_qube = target_qube

    def _call(self, service: str, payload: Dict) -> Dict:
        cmd = ["qrexec-client-vm", self.target_qube, service]
        proc = subprocess.run(
            cmd,
            input=json.dumps(payload).encode(),
            capture_output=True,
            text=True
        )
        if proc.returncode != 0:
            return {"error": proc.stderr}
        try:
            return json.loads(proc.stdout)
        except json.JSONDecodeError:
            return {"raw": proc.stdout}

    def store(self, content: bytes, encrypt: bool = False) -> Optional[str]:
        payload = {
            "content_base64": base64.b64encode(content).decode(),
            "encrypt": encrypt
        }
        result = self._call("cathedral.BTFSStore", payload)
        return result.get("cid") if "cid" in result else None

    def retrieve(self, cid: str) -> Optional[bytes]:
        payload = {"cid": cid}
        result = self._call("cathedral.BTFSRetrieve", payload)
        if "error" in result:
            return None
        return base64.b64decode(result.get("raw", ""))

    def list_providers(self, cid: str) -> list:
        payload = {"cid": cid}
        result = self._call("cathedral.BTFSProviderList", payload)
        return result.get("providers", [])
