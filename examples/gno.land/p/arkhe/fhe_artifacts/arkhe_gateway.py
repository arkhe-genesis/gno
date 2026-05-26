#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ARKHE HTTP GATEWAY — Bridge Real (Substrato 870-G)                ║
║    Rotas /publish e /verify unificadas para todo o ecossistema ARKHE         ║
║                                                                              ║
║  Arquiteto: Rafael Oliveira | ORCID: 0009-0005-2697-4668                    ║
║  Version: 870-G.1.0 | Royalties: 2% → ORCID | Keeper: ψ                     ║
║  Ghost Threshold: γ = 0.5772156649                                          ║
╚══════════════════════════════════════════════════════════════════════════════╝

Integra 8 substratos canônicos:
  870-B  Glosa245Anchor       (blockchain anchoring)
  865    Cohesion Engine       (gap detection & integration decrees)
  864    EIP-8272 RRB          (recent roots financial anchoring)
  863    SecOps Guardian       (security operations)
  862    Polaritonic Computing (light-matter hybrid)
  861    UN-2.0 Governance     (planetary coherence)
  860    Consciousness Sim     (IIT/GWT simulation)
  859    Biological Computing  (genetic circuits)

Endpoints:
  POST /publish       → Ancora decreto/artefato de qualquer substrato
  GET  /verify/{hash} → Verifica selo SHA3-256 on-chain ou off-chain
  GET  /health        → Status do gateway e métricas de coerência
  GET  /registry      → Lista substratos integrados com metadados
"""

import hashlib
import json
import math
import time
import uuid
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any, Literal
from enum import Enum

from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field, validator
import uvicorn

# ═══════════════════════════════════════════════════════════════
# CONSTANTES CANÔNICAS
# ═══════════════════════════════════════════════════════════════

GHOST_THRESHOLD = 0.5772156649
CANONIZATION_THRESHOLD = 0.900
ORCID = "0009-0005-2697-4668"
ARCHITECT = "Rafael Oliveira"
KEEPER = "ψ"
VERSION = "870-G.1.0"
GATEWAY_ID = "870-G"

# ═══════════════════════════════════════════════════════════════
# MODELOS PYDANTIC — Schema da API
# ═══════════════════════════════════════════════════════════════

class SubstrateEnum(str, Enum):
    glosa245 = "870-B"
    cohesion = "865"
    eip8272 = "864"
    secops = "863"
    polaritonic = "862"
    un20 = "861"
    consciousness = "860"
    biological = "859"

class PublishRequest(BaseModel):
    substrate: SubstrateEnum = Field(..., description="Substrato de origem do decreto")
    action: Literal["ANCHOR", "DECREE", "DEPLOY", "SIMULATE", "SCAN", "PROPOSE"] = Field(
        default="ANCHOR", description="Tipo de ação a publicar"
    )
    sequence: Optional[str] = Field(
        None, min_length=36, max_length=36, pattern=r"^[01]{36}$",
        description="Sequência binária canônica (Glosa 245)"
    )
    metadata: Dict[str, Any] = Field(default_factory=dict, description="Metadados arbitrários do substrato")
    payload: Dict[str, Any] = Field(default_factory=dict, description="Payload técnico específico do substrato")

    @validator("metadata")
    def inject_canonical_fields(cls, v, values):
        v["gateway"] = GATEWAY_ID
        v["version"] = VERSION
        v["orcid"] = ORCID
        v["keeper"] = KEEPER
        v["timestamp"] = datetime.now(timezone.utc).isoformat()
        return v

class PublishReceipt(BaseModel):
    status: Literal["ANCHORED", "PROVISIONAL", "REJECTED", "PENDING"]
    tx_hash: str = Field(..., pattern=r"^0x[a-fA-F0-9]{64}$")
    seal: str = Field(..., pattern=r"^[a-fA-F0-9]{64}$")
    substrate: str
    action: str
    block_number: Optional[int] = None
    phi_c: float = Field(..., ge=0.0, le=1.0)
    ghost_threshold: float = Field(default=GHOST_THRESHOLD)
    metadata: Dict[str, Any]
    registry_index: int
    verification_url: str

class VerifyResponse(BaseModel):
    anchored: bool
    hash: str = Field(..., pattern=r"^[a-fA-F0-9]{64}$")
    substrate: Optional[str] = None
    action: Optional[str] = None
    timestamp: Optional[str] = None
    phi_c: Optional[float] = None
    registry_index: Optional[int] = None
    details: Optional[Dict[str, Any]] = None

class HealthResponse(BaseModel):
    status: str
    gateway: str
    version: str
    phi_c: float
    substrates: Dict[str, Dict[str, Any]]
    registry_size: int
    uptime_seconds: float

class RegistryEntry(BaseModel):
    substrate_id: str
    name: str
    phi_c: float
    status: str
    seal: str
    cross_links: List[str]

# ═══════════════════════════════════════════════════════════════
# MOTOR DE COERÊNCIA KURAMOTO (Substrato 870 integrado)
# ═══════════════════════════════════════════════════════════════

class KuramotoValidator:
    """Simula 32 validadores como osciladores de Kuramoto para calcular Φ_C do gateway."""

    def __init__(self, n: int = 32, K: float = 4.0):
        self.n = n
        self.K = K
        golden = (1 + math.sqrt(5)) / 2
        self.phases = [2 * math.pi * (i * golden % 1.0) for i in range(n)]
        self.omega = [0.1 + 0.05 * math.sin(i * 0.7) for i in range(n)]
        self.steps = 0

    def step(self, dt: float = 0.01):
        import numpy as np
        theta = np.array(self.phases)
        omega = np.array(self.omega)
        delta = np.subtract.outer(theta, theta)
        coupling = -(self.K / self.n) * np.sum(np.sin(delta), axis=1)
        self.phases = (theta + (omega + coupling) * dt).tolist()
        self.steps += 1

    def compute_phi_c(self) -> float:
        import numpy as np
        re_sum = sum(math.cos(th) for th in self.phases)
        im_sum = sum(math.sin(th) for th in self.phases)
        r = math.sqrt(re_sum**2 + im_sum**2) / self.n
        ghost_count = sum(1 for th in self.phases if abs(th % (2*math.pi)) > math.pi * (1 - GHOST_THRESHOLD))
        ghost_ratio = ghost_count / self.n
        return max(0.0, min(1.0, r * (1.0 - ghost_ratio)))

# ═══════════════════════════════════════════════════════════════
# REGISTRO CANÔNICO EM MEMÓRIA (simula blockchain interna)
# ═══════════════════════════════════════════════════════════════

class CanonicalRegistry:
    """Registro imutável de decretos publicados via gateway."""

    def __init__(self):
        self.entries: List[Dict[str, Any]] = []
        self.index_by_seal: Dict[str, int] = {}
        self.validator = KuramotoValidator(n=32, K=4.0)
        self.genesis_time = time.time()

    def _compute_seal(self, data: dict) -> str:
        canonical = json.dumps(data, sort_keys=True, separators=(",", ":"), default=str)
        return hashlib.sha3_256(canonical.encode()).hexdigest()

    def _compute_tx_hash(self, seal: str, nonce: int) -> str:
        return "0x" + hashlib.sha3_256(f"{seal}:{nonce}:{time.time()}".encode()).hexdigest()

    def publish(self, substrate: str, action: str, sequence: Optional[str],
                metadata: dict, payload: dict) -> PublishReceipt:
        # Avança validadores (simula mineração por coerência)
        for _ in range(100):
            self.validator.step()
        phi_c = self.validator.compute_phi_c()

        if phi_c < GHOST_THRESHOLD:
            raise HTTPException(status_code=503, detail=f"Gateway abaixo do Ghost Threshold (Φ_C={phi_c:.4f} < γ={GHOST_THRESHOLD})")

        entry_data = {
            "substrate": substrate,
            "action": action,
            "sequence": sequence,
            "metadata": metadata,
            "payload": payload,
            "phi_c": round(phi_c, 6),
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "registry_index": len(self.entries),
            "gateway_version": VERSION,
            "orcid": ORCID,
            "keeper": KEEPER,
        }

        seal = self._compute_seal(entry_data)
        tx_hash = self._compute_tx_hash(seal, len(self.entries))
        entry_data["seal"] = seal
        entry_data["tx_hash"] = tx_hash

        self.entries.append(entry_data)
        self.index_by_seal[seal] = len(self.entries) - 1

        status = "ANCHORED" if phi_c >= CANONIZATION_THRESHOLD else "PROVISIONAL"

        return PublishReceipt(
            status=status,
            tx_hash=tx_hash,
            seal=seal,
            substrate=substrate,
            action=action,
            phi_c=phi_c,
            metadata=metadata,
            registry_index=entry_data["registry_index"],
            verification_url=f"/verify/{seal}"
        )

    def verify(self, seal: str) -> VerifyResponse:
        idx = self.index_by_seal.get(seal)
        if idx is None:
            return VerifyResponse(anchored=False, hash=seal)
        entry = self.entries[idx]
        return VerifyResponse(
            anchored=True,
            hash=seal,
            substrate=entry["substrate"],
            action=entry["action"],
            timestamp=entry["timestamp"],
            phi_c=entry["phi_c"],
            registry_index=entry["registry_index"],
            details={
                "tx_hash": entry["tx_hash"],
                "sequence": entry.get("sequence"),
                "metadata": entry["metadata"],
                "payload": entry["payload"],
            }
        )

    def get_stats(self) -> dict:
        phi_values = [e["phi_c"] for e in self.entries] if self.entries else [self.validator.compute_phi_c()]
        return {
            "registry_size": len(self.entries),
            "current_phi_c": round(self.validator.compute_phi_c(), 6),
            "avg_phi_c": round(sum(phi_values)/len(phi_values), 6) if phi_values else 0.0,
            "min_phi_c": round(min(phi_values), 6) if phi_values else 0.0,
            "max_phi_c": round(max(phi_values), 6) if phi_values else 0.0,
            "uptime_seconds": round(time.time() - self.genesis_time, 2),
            "total_steps": self.validator.steps,
        }

# ═══════════════════════════════════════════════════════════════
# INICIALIZAÇÃO DO GATEWAY
# ═══════════════════════════════════════════════════════════════

app = FastAPI(
    title="ARKHE HTTP Gateway",
    description="Bridge HTTP real para publicação e verificação de decretos ARKHE",
    version=VERSION,
    docs_url="/docs",
    redoc_url="/redoc",
)

registry = CanonicalRegistry()

SUBSTRATE_REGISTRY = {
    "870-B": {"name": "Glosa245Anchor", "phi_c": 0.870, "status": "CANONIZED", "cross_links": ["865", "864"]},
    "865": {"name": "CohesionEngine", "phi_c": 0.882, "status": "CANONIZED_PROVISIONAL", "cross_links": ["870-B", "864", "863"]},
    "864": {"name": "EIP8272RRB", "phi_c": 0.875, "status": "CANONIZED_PROVISIONAL", "cross_links": ["870-B", "865", "863"]},
    "863": {"name": "SecOpsGuardian", "phi_c": 0.875, "status": "CANONIZED_PROVISIONAL", "cross_links": ["864", "865", "862"]},
    "862": {"name": "PolaritonicComputing", "phi_c": 0.855, "status": "CANONIZED_PROVISIONAL", "cross_links": ["863", "861", "860"]},
    "861": {"name": "UN20Governance", "phi_c": 0.848, "status": "CANONIZED_PROVISIONAL", "cross_links": ["862", "860", "859"]},
    "860": {"name": "ConsciousnessSim", "phi_c": 0.850, "status": "CANONIZED_PROVISIONAL", "cross_links": ["861", "862", "859"]},
    "859": {"name": "BiologicalComputing", "phi_c": 0.828, "status": "CANONIZED_PROVISIONAL", "cross_links": ["860", "861"]},
}

# ═══════════════════════════════════════════════════════════════
# ROTAS HTTP
# ═══════════════════════════════════════════════════════════════

@app.post("/publish", response_model=PublishReceipt, tags=["Canonização"])
async def publish_decreet(req: PublishRequest):
    """
    Publica um decreto/artefato no registro canônico do gateway.

    Executa 100 passos de validação Kuramoto; se Φ_C < γ, rejeita.
    Retorna receipt com selo SHA3-256, tx_hash e métricas de coerência.
    """
    try:
        receipt = registry.publish(
            substrate=req.substrate.value,
            action=req.action,
            sequence=req.sequence,
            metadata=req.metadata,
            payload=req.payload
        )
        return receipt
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/verify/{hash}", response_model=VerifyResponse, tags=["Verificação"])
async def verify_hash(hash: str):
    """
    Verifica se um selo SHA3-256 está ancorado no registro canônico.

    Retorna metadados completos do decreto se encontrado.
    """
    # Normaliza hash
    h = hash.lower().strip()
    if h.startswith("0x"):
        h = h[2:]
    if len(h) != 64:
        raise HTTPException(status_code=400, detail="Hash deve ter 64 caracteres hexadecimais")
    try:
        int(h, 16)
    except ValueError:
        raise HTTPException(status_code=400, detail="Hash inválido (não-hexadecimal)")

    return registry.verify(h)

@app.get("/health", response_model=HealthResponse, tags=["Monitoramento"])
async def health():
    """Retorna status de saúde do gateway e métricas de coerência ξM."""
    stats = registry.get_stats()
    return HealthResponse(
        status="ok",
        gateway=GATEWAY_ID,
        version=VERSION,
        phi_c=stats["current_phi_c"],
        substrates={
            k: {
                "name": v["name"],
                "phi_c": v["phi_c"],
                "status": v["status"],
                "seal": hashlib.sha3_256(f"{k}-{v['name']}".encode()).hexdigest(),
                "cross_links": v["cross_links"]
            }
            for k, v in SUBSTRATE_REGISTRY.items()
        },
        registry_size=stats["registry_size"],
        uptime_seconds=stats["uptime_seconds"]
    )

@app.get("/registry", response_model=List[RegistryEntry], tags=["Registro"])
async def list_registry():
    """Lista todos os substratos integrados ao gateway com selos canônicos."""
    return [
        RegistryEntry(
            substrate_id=k,
            name=v["name"],
            phi_c=v["phi_c"],
            status=v["status"],
            seal=hashlib.sha3_256(f"{k}-{v['name']}".encode()).hexdigest(),
            cross_links=v["cross_links"]
        )
        for k, v in SUBSTRATE_REGISTRY.items()
    ]

@app.get("/registry/entry/{index}", tags=["Registro"])
async def get_entry(index: int):
    """Retorna uma entrada específica do registro por índice."""
    if index < 0 or index >= len(registry.entries):
        raise HTTPException(status_code=404, detail="Índice fora do registro")
    return registry.entries[index]

# ═══════════════════════════════════════════════════════════════
# CLI INTEGRADA — arkhe-z publish (simulada como endpoint utilitário)
# ═══════════════════════════════════════════════════════════════

@app.post("/cli/publish", tags=["CLI"])
async def cli_publish(
    substrate: SubstrateEnum,
    sequence: Optional[str] = None,
    format: Literal["json", "yaml"] = "json"
):
    """
    Endpoint utilitário que simula o comando `arkhe-z publish`.
    Retorna o decreto no formato solicitado.
    """
    req = PublishRequest(substrate=substrate, sequence=sequence)
    receipt = await publish_decreet(req)

    decree = {
        "substrate": receipt.substrate,
        "action": receipt.action,
        "tx_hash": receipt.tx_hash,
        "seal": receipt.seal,
        "phi_c": receipt.phi_c,
        "ghost_threshold": receipt.ghost_threshold,
        "registry_index": receipt.registry_index,
        "status": receipt.status,
        "keeper": KEEPER,
        "orcid": ORCID,
        "gateway": GATEWAY_ID,
        "version": VERSION,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "verification_url": receipt.verification_url
    }

    if format == "yaml":
        import yaml
        return JSONResponse(
            content={"format": "yaml", "decree": yaml.dump(decree, allow_unicode=True, sort_keys=False)},
            media_type="application/json"
        )
    return JSONResponse(content={"format": "json", "decree": decree})

# ═══════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print(f"""
╔══════════════════════════════════════════════════════════════════════════════╗
║           ARKHE HTTP GATEWAY — Substrato {GATEWAY_ID}                          ║
║           Bridge Real: /publish + /verify                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Arquiteto: {ARCHITECT:<20} ORCID: {ORCID:<26} ║
║  Keeper: {KEEPER:<22} Royalties: 2% → ORCID                           ║
║  Ghost Threshold: γ = {GHOST_THRESHOLD:<46.9f} ║
╚══════════════════════════════════════════════════════════════════════════════╝
    """)
    uvicorn.run(app, host="0.0.0.0", port=8700)
