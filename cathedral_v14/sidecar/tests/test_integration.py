import pytest
import asyncio
from unittest.mock import patch, MagicMock

import aiohttp
from aiohttp import web
from aiohttp.test_utils import TestServer, TestClient

from ..client import GgufSidecarClient

@pytest.mark.asyncio
async def test_circuit_breaker_integration(aiohttp_client):
    # Simula um servidor lento que deve abrir o circuit breaker
    async def slow_handler(request):
        await asyncio.sleep(0.5)
        return web.json_response({"text": "ok", "tokens": 10, "cache_hit": False})

    app = web.Application()
    app.router.add_post('/v1/generate', slow_handler)
    client = await aiohttp_client(app)

    # Configura o client para ser muito sensível (slow = 0.1s)
    sidecar_client = GgufSidecarClient({
        "sidecar_url": f"http://{client.host}:{client.port}",
        "sidecar_slow_threshold_s": 0.1,
        "circuit_max_failures": 2
    })
    # Injerta o session de teste
    sidecar_client._session = client.session

    res = await sidecar_client.generate("hello", max_retries=0)
    assert res["text"] == "ok"
    # Uma requisição lenta falha, circuit avança.

    res2 = await sidecar_client.generate("hello2", max_retries=0)
    assert res2["text"] == "ok"

    # Terceira deve falhar pelo circuit breaker aberto
    res3 = await sidecar_client.generate("hello3", max_retries=0)
    assert "FALLBACK" in res3["text"]
    assert "Circuit breaker OPEN" in res3["text"]
