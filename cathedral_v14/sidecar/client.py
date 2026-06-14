#!/usr/bin/env python3
"""
Cathedral ARKHE v14.0.0 — Sidecar Client (Production-Ready)
Circuit breaker sensível a timeout, correlation ID, retry com backoff, fallback seguro.
"""
import asyncio
import logging
import os
import time
import uuid
from typing import Dict, Optional

try:
    import aiohttp
    HAS_AIOHTTP = True
except ImportError:
    HAS_AIOHTTP = False

log = logging.getLogger("cathedral.v14.client")

class CircuitBreaker:
    """Circuit breaker com estados CLOSED, OPEN, HALF_OPEN."""
    def __init__(self, max_failures: int = 5, recovery_timeout: float = 10.0,
                 slow_threshold: float = 5.0):
        self.max_failures = max_failures
        self.recovery_timeout = recovery_timeout
        self.slow_threshold = slow_threshold
        self._failures = 0
        self._state = "CLOSED"
        self._last_failure_time = 0.0

    def is_available(self) -> bool:
        return self._state != "OPEN"

    def check(self) -> bool:
        if self._state == "CLOSED":
            return True
        if self._state == "OPEN":
            if (time.time() - self._last_failure_time) > self.recovery_timeout:
                self._state = "HALF_OPEN"
                log.info("Circuit breaker: HALF_OPEN")
                return True
            return False
        return True  # HALF_OPEN

    def record_success(self, duration_s: float):
        if self._state == "HALF_OPEN":
            log.info("Circuit breaker: HALF_OPEN -> CLOSED")
            self._state = "CLOSED"
            self._failures = 0
        if duration_s > self.slow_threshold:
            log.warning("Requisição lenta (%.2fs > %.2fs)", duration_s, self.slow_threshold)
            self.record_failure()
        else:
            self._failures = 0

    def record_failure(self):
        self._failures += 1
        self._last_failure_time = time.time()
        if self._failures >= self.max_failures and self._state != "OPEN":
            self._state = "OPEN"
            log.critical("Circuit breaker: OPEN")

class GgufSidecarClient:
    def __init__(self, config: Optional[Dict] = None):
        cfg = config or {}
        self.endpoint = os.getenv("SIDECAR_URL", cfg.get("sidecar_url", "http://inference-v14:8000")).rstrip("/")
        self.token = os.getenv("SIDECAR_TOKEN", cfg.get("sidecar_token", "cathedral-super-secret-token"))
        self.timeout_total = cfg.get("sidecar_timeout_s", 15.0)
        self.circuit = CircuitBreaker(
            max_failures=cfg.get("circuit_max_failures", 5),
            recovery_timeout=cfg.get("circuit_recovery_s", 10.0),
            slow_threshold=cfg.get("sidecar_slow_threshold_s", 5.0),
        )
        self._session: Optional[aiohttp.ClientSession] = None

    async def _get_session(self) -> aiohttp.ClientSession:
        if self._session is None or self._session.closed:
            timeout = aiohttp.ClientTimeout(total=self.timeout_total)
            self._session = aiohttp.ClientSession(timeout=timeout)
        return self._session

    async def close(self):
        if self._session and not self._session.closed:
            await self._session.close()
            self._session = None

    async def generate(self, prompt: str, max_tokens: int = 256, max_retries: int = 2) -> Dict:
        if not self.circuit.check():
            return self._fallback_response("Circuit breaker OPEN")
        if not HAS_AIOHTTP:
            return self._fallback_response("aiohttp não instalado")

        request_id = str(uuid.uuid4())
        headers = {
            "Authorization": "Bearer %s" % self.token,
            "Content-Type": "application/json",
            "X-Request-ID": request_id,
        }
        payload = {"prompt": prompt, "max_tokens": max_tokens}
        log.info("Enviando req %s", request_id, extra={"request_id": request_id})

        session = await self._get_session()
        for attempt in range(max_retries + 1):
            try:
                start = time.monotonic()
                async with session.post(
                    "%s/v1/generate" % self.endpoint,
                    json=payload,
                    headers=headers,
                ) as resp:
                    duration = time.monotonic() - start
                    if resp.status == 200:
                        self.circuit.record_success(duration)
                        data = await resp.json()
                        log.info("Req %s sucedida em %.3fs", request_id, duration,
                                 extra={"request_id": request_id})
                        return data
                    elif resp.status == 401:
                        log.error("Auth failed req %s", request_id)
                        return self._fallback_response("Auth Failed")
                    else:
                        text = await resp.text()
                        raise Exception("HTTP %d: %s" % (resp.status, text[:200]))
            except asyncio.TimeoutError:
                log.warning("Timeout tentativa %d/%d req %s", attempt + 1, max_retries + 1, request_id)
            except Exception as e:
                log.warning("Erro tentativa %d/%d req %s: %s", attempt + 1, max_retries + 1, request_id, e)
            if attempt < max_retries:
                await asyncio.sleep(0.1 * (2 ** attempt))

        self.circuit.record_failure()
        return self._fallback_response("Max retries exceeded")

    async def health(self) -> Dict:
        if not HAS_AIOHTTP:
            return {"status": "no_aiohttp"}
        try:
            session = await self._get_session()
            async with session.get("%s/health" % self.endpoint, timeout=aiohttp.ClientTimeout(total=5)) as resp:
                if resp.status == 200:
                    return await resp.json()
                return {"status": "http_%d" % resp.status}
        except Exception as e:
            return {"status": "unreachable", "error": str(e)}

    async def stats(self) -> Dict:
        if not HAS_AIOHTTP:
            return {"status": "no_aiohttp"}
        try:
            session = await self._get_session()
            headers = {"Authorization": "Bearer %s" % self.token}
            async with session.get("%s/v1/stats" % self.endpoint, headers=headers,
                                   timeout=aiohttp.ClientTimeout(total=5)) as resp:
                if resp.status == 200:
                    return await resp.json()
                return {"status": "http_%d" % resp.status}
        except Exception as e:
            return {"status": "unreachable", "error": str(e)}

    def _fallback_response(self, reason: str) -> Dict:
        log.warning("Fallback: %s", reason)
        return {
            "text": "[FALLBACK V12] Inferência indisponível (%s)." % reason,
            "tokens": 0,
            "cache_hit": False,
        }
