#!/usr/bin/env python3
"""
Cathedral ARKHE v14.0.0 — GGUF Inference Engine
Integração real com llama-cpp-python, async via run_in_executor.
"""
import asyncio
import hashlib
import logging
import math
import os
import time
from collections import deque
from pathlib import Path
from typing import Dict, List, Optional, Tuple

try:
    from llama_cpp import Llama
    HAS_LLAMA_CPP = True
except ImportError:
    HAS_LLAMA_CPP = False

try:
    from huggingface_hub import hf_hub_download
    HAS_HF_HUB = True
except ImportError:
    HAS_HF_HUB = False

try:
    import torch
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False

log = logging.getLogger("cathedral.v14.engine")

class GgufInferenceEngine:
    """
    Motor de inferência GGUF com fila serial, cache semântico e sliding window.
    """
    def __init__(
        self,
        model_id: str = "unsloth/Qwen3-0.6B-GGUF",
        quant: str = "Q4_K_M",
        cache_dir: str = "~/.cathedral/gguf",
        n_ctx: int = 4096,
        n_threads: int = 8,
        n_gpu_layers: int = -1,
        batch_size: int = 512,
        temperature: float = 0.7,
        top_p: float = 0.9,
        max_tokens: int = 256,
        system_prompt: str = "Voce e o Oraculo da Cathedral ARKHE. Responda de forma precisa.",
    ):
        self.model_id = model_id
        self.quant = quant.lower()
        self.cache_dir = Path(cache_dir).expanduser()
        self.n_ctx = n_ctx
        self.n_threads = n_threads
        self.n_gpu_layers_cfg = n_gpu_layers
        self.batch_size = batch_size
        self.temperature = temperature
        self.top_p = top_p
        self.max_tokens = max_tokens
        self.system_prompt = system_prompt

        self._llm = None
        self._loaded = False
        self._embed_dim = 0
        self._history = deque(maxlen=50)
        self._queue = asyncio.Queue()
        self._worker_task = None
        self._semantic_cache = deque(maxlen=30)
        self._cache_threshold = 0.98

        self._load_time_ms = 0.0
        self._tokens_generated = 0
        self._gen_time_ms = 0.0
        self._cache_hits = 0
        self._cache_misses = 0
        self._requests_total = 0

        self._try_load()

    def _detect_gpu_layers(self) -> int:
        if self.n_gpu_layers_cfg != -1:
            return self.n_gpu_layers_cfg
        if HAS_TORCH and torch.cuda.is_available():
            return 99
        return 0

    def _resolve_and_download_model(self) -> Optional[Path]:
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        for f in self.cache_dir.rglob("*%s*.gguf" % self.quant):
            return f
        if HAS_HF_HUB:
            try:
                log.info("Baixando %s (%s) via HuggingFace...", self.model_id, self.quant)
                path = hf_hub_download(repo_id=self.model_id, filename="*%s*.gguf" % self.quant,
                                       cache_dir=str(self.cache_dir))
                return Path(path)
            except Exception as e:
                log.error("Falha no download HF: %s", e)
        return None

    def _try_load(self):
        if not HAS_LLAMA_CPP:
            log.warning("llama-cpp-python nao instalado. Operando em modo Stub.")
            return
        path = self._resolve_and_download_model()
        if not path or not path.exists():
            log.warning("Modelo GGUF nao encontrado.")
            return
        try:
            start = time.time()
            self._llm = Llama(
                model_path=str(path),
                n_ctx=self.n_ctx,
                n_threads=self.n_threads,
                n_gpu_layers=self._detect_gpu_layers(),
                verbose=False,
            )
            self._load_time_ms = (time.time() - start) * 1000
            self._loaded = True
            try:
                self._embed_dim = len(self._llm.embed("test"))
            except:
                self._embed_dim = 0
            log.info("GGUF Carregado (GPU:%d, Dim:%d, Load:%.0fms)",
                     self._detect_gpu_layers(), self._embed_dim, self._load_time_ms)
        except Exception as e:
            log.error("Falha ao carregar GGUF: %s", e)

    async def start(self):
        if self._loaded and self._worker_task is None:
            self._worker_task = asyncio.create_task(self._worker_loop())
            log.info("Worker de inferencia iniciado")

    async def stop(self):
        if self._worker_task:
            self._worker_task.cancel()
            try:
                await self._worker_task
            except asyncio.CancelledError:
                pass
            log.info("Worker de inferencia parado")

    async def _worker_loop(self):
        loop = asyncio.get_running_loop()
        while True:
            req = await self._queue.get()
            try:
                result = await loop.run_in_executor(None, self._sync_generate, req["prompt"], req["kwargs"])
                if not req["future"].done():
                    req["future"].set_result(result)
            except Exception as e:
                if not req["future"].done():
                    req["future"].set_exception(e)
            finally:
                self._queue.task_done()

    def _sync_generate(self, prompt: str, kwargs: Dict) -> Dict:
        output = self._llm(prompt, **kwargs)
        text = output["choices"][0]["text"].strip()
        tokens = output.get("usage", {}).get("completion_tokens", 0)
        self._tokens_generated += tokens
        return {"text": text, "tokens": tokens}

    async def embed(self, text: str) -> List[float]:
        if not self.is_available() or self._embed_dim == 0:
            return [0.0] * 64
        loop = asyncio.get_running_loop()
        raw_emb = await loop.run_in_executor(None, self._llm.embed, text)
        norm = math.sqrt(sum(x * x for x in raw_emb))
        return [x / norm for x in raw_emb] if norm > 0 else raw_emb

    async def _check_semantic_cache(self, prompt: str) -> Optional[str]:
        if not self._semantic_cache or self._embed_dim == 0:
            return None
        prompt_emb = await self.embed(prompt)
        for cached_emb, cached_resp in self._semantic_cache:
            if sum(a * b for a, b in zip(prompt_emb, cached_emb)) >= self._cache_threshold:
                return cached_resp
        return None

    def _build_prompt_sliding_window(self, user_prompt: str) -> str:
        self._history.append({"role": "user", "content": user_prompt})
        max_ctx_chars = (self.n_ctx - self.max_tokens - 50) * 4
        parts = ["[System]: %s\n" % self.system_prompt]
        current_chars = len(parts[0])
        for msg in reversed(self._history):
            msg_str = "[%s]: %s\n" % (msg["role"].capitalize(), msg["content"])
            if current_chars + len(msg_str) > max_ctx_chars:
                break
            parts.insert(1, msg_str)
            current_chars += len(msg_str)
        parts.append("[Assistant]:")
        return "".join(parts)

    def is_available(self) -> bool:
        return self._loaded and self._llm is not None

    async def generate(self, prompt: str, use_history: bool = True) -> Dict:
        self._requests_total += 1
        if not self.is_available():
            return {"text": "[GGUF Stub] %s" % prompt[:50], "tokens": 0, "cache_hit": False}

        cached = await self._check_semantic_cache(prompt)
        if cached:
            self._cache_hits += 1
            return {"text": cached, "tokens": 0, "cache_hit": True}
        self._cache_misses += 1

        final_prompt = self._build_prompt_sliding_window(prompt) if use_history else prompt
        future = asyncio.get_running_loop().create_future()
        await self._queue.put({
            "future": future,
            "prompt": final_prompt,
            "kwargs": {
                "max_tokens": self.max_tokens,
                "temperature": self.temperature,
                "stop": ["\n\n[", "["],
            },
        })

        # CORRIGIDO: Uso de time.monotonic() que é à prova de mudanças de relógio do SO
        start = time.monotonic()
        result = await future
        duration_ms = (time.monotonic() - start) * 1000
        self._gen_time_ms += duration_ms

        if not result.get("cache_hit"):
            self._semantic_cache.append((await self.embed(prompt), result["text"]))
        return result

    def get_stats(self) -> Dict:
        tps = (self._tokens_generated / (self._gen_time_ms / 1000)) if self._gen_time_ms > 0 else 0.0
        total_c = self._cache_hits + self._cache_misses
        return {
            "loaded": self._loaded,
            "model_id": self.model_id,
            "embed_dim": self._embed_dim,
            "load_time_ms": round(self._load_time_ms, 1),
            "total_tokens": self._tokens_generated,
            "tokens_per_second": round(tps, 2),
            "cache_hits": self._cache_hits,
            "cache_misses": self._cache_misses,
            "cache_hit_rate": round(self._cache_hits / total_c, 3) if total_c > 0 else 0.0,
            "queue_size": self._queue.qsize(),
            "requests_total": self._requests_total,
        }
