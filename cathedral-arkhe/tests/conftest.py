# tests/conftest.py
"""Fixtures compartilhados para todos os testes."""

import pytest
import numpy as np
import tempfile
from pathlib import Path


@pytest.fixture
def dim_4096():
    """Dimensão padrão de embedding."""
    return 4096


@pytest.fixture
def random_embedding(dim_4096):
    """Embedding aleatório normalizado."""
    rng = np.random.RandomState(42)
    return rng.randn(dim_4096).astype(np.float32) * 0.1


@pytest.fixture
def random_logits(vocab_size=32000, seq_len=10):
    """Sequência de logits aleatórios."""
    rng = np.random.RandomState(42)
    return [rng.randn(vocab_size).astype(np.float32) * 0.5
            for _ in range(seq_len)]


@pytest.fixture
def temp_gguf_file():
    """Arquivo GGUF simulado temporário para testes."""
    import struct
    from cathedral.constants import GGUF_MAGIC, GGUF_VERSION

    tmp = tempfile.NamedTemporaryFile(suffix=".gguf", delete=False)
    with open(tmp.name, "wb") as f:
        # Header
        f.write(struct.pack("<I", GGUF_MAGIC))
        f.write(struct.pack("<I", GGUF_VERSION))
        f.write(struct.pack("<Q", 3))   # tensor_count
        f.write(struct.pack("<Q", 2))   # metadata_kv_count
        # Metadata
        key = "general.architecture"
        f.write(struct.pack("<I", len(key)))
        f.write(key.encode("utf-8"))
        f.write(struct.pack("<I", 8))   # string type
        f.write(struct.pack("<Q", len("llama")))
        f.write(b"llama")
        # Fim
    yield tmp.name
    tmp.close()
    Path(tmp.name).unlink(missing_ok=True)
