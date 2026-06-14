#!/usr/bin/env python3
"""
Cathedral ARKHE v14.0.0 — Sidecar Server (Production-Ready)
FastAPI real, prometheus_client nativo, parse de body, auth Bearer.
"""
import asyncio
import logging
import os
import time
import uuid
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException, Header, Request
from fastapi.responses import Response
from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
import uvicorn

from .engine import GgufInferenceEngine

log = logging.getLogger("cathedral.v14.server")

# Prometheus metrics nativos
REQUEST_COUNT = Counter(
    'cathedral_v14_inferences_total',
    'Total de inferências processadas',
    ['status']
)
REQUEST_LATENCY = Histogram(
    'cathedral_v14_inference_duration_seconds',
    'Latência de inferência',
    buckets=[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
)
CACHE_HITS = Counter(
    'cathedral_v14_cache_hits_total',
    'Acertos do cache semântico'
)
REQUEST_IN_FLIGHT = Counter(
    'cathedral_v14_requests_in_flight',
    'Requisições em processamento'
)

engine = None

async def verify_token(authorization: str = Header(None), x_request_id: str = Header(None)):
    expected = "Bearer %s" % os.getenv('SIDECAR_TOKEN', 'cathedral-super-secret-token')
    if authorization != expected:
        log.warning("Tentativa de acesso não autorizada. Req ID: %s", x_request_id)
        raise HTTPException(status_code=401, detail="Unauthorized")

@asynccontextmanager
async def lifespan(app: FastAPI):
    global engine
    cfg = {
        "model_id": os.getenv("GGUF_MODEL", "unsloth/Qwen3-0.6B-GGUF"),
        "quant": os.getenv("GGUF_QUANT", "Q4_K_M"),
        "n_ctx": int(os.getenv("GGUF_CTX", "4096")),
        "n_gpu_layers": int(os.getenv("GGUF_GPU_LAYERS", "-1")),
        "cache_dir": os.getenv("GGUF_CACHE_DIR", "~/.cathedral/gguf"),
    }
    engine = GgufInferenceEngine(**cfg)
    await engine.start()
    log.info("Sidecar v14.0.0 iniciado. Modelo: %s", cfg["model_id"])
    yield
    await engine.stop()
    log.info("Sidecar v14.0.0 desligado.")

app = FastAPI(
    title="Cathedral ARKHE v14 Sidecar",
    version="14.0.0",
    lifespan=lifespan,
)

@app.post("/v1/generate")
async def generate(
    request: Request,
    authorization: str = Header(None),
    x_request_id: str = Header(None)
):
    await verify_token(authorization, x_request_id)
    req_id = x_request_id or str(uuid.uuid4())

    body = await request.json()
    prompt = body.get("prompt", "")
    max_tokens = body.get("max_tokens", 256)
    use_history = body.get("use_history", True)

    REQUEST_IN_FLIGHT.inc()

    # CORRIGIDO: O 'with' garante que o histograma registre o tempo mesmo se ocorrer Exception
    try:
        with REQUEST_LATENCY.time():
            result = await engine.generate(prompt, use_history=use_history)

        REQUEST_COUNT.labels(status="success").inc()
        if result.get("cache_hit"):
            CACHE_HITS.inc()

        log.info("Req %s processada (cache=%s, tokens=%d)",
                 req_id, result.get("cache_hit", False), result.get("tokens", 0))
        return result

    except Exception as e:
        REQUEST_COUNT.labels(status="error").inc()
        log.error("Req %s falhou: %s", req_id, e)
        raise HTTPException(status_code=500, detail=str(e))
    finally:
        REQUEST_IN_FLIGHT.dec()

@app.get("/metrics")
async def metrics():
    return Response(
        content=generate_latest(),
        media_type=CONTENT_TYPE_LATEST
    )

@app.get("/health")
async def health():
    return {
        "status": "healthy" if engine and engine.is_available() else "degraded",
        "version": "14.0.0",
        "engine_loaded": engine.is_available() if engine else False,
        "stats": engine.get_stats() if engine else {}
    }

@app.get("/v1/stats")
async def stats(authorization: str = Header(None), x_request_id: str = Header(None)):
    await verify_token(authorization, x_request_id)
    return engine.get_stats() if engine else {}

def main():
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(name)s] %(levelname)s: %(message)s"
    )
    uvicorn.run(
        app,
        host=os.getenv("SIDECAR_HOST", "0.0.0.0"),
        port=int(os.getenv("SIDECAR_PORT", "8000")),
        log_level="warning",
    )

if __name__ == "__main__":
    main()
