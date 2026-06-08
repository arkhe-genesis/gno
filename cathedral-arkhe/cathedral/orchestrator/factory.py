# cathedral/orchestrator/factory.py
"""Fábrica de orquestradores por versão."""

from typing import Optional, Any
from pathlib import Path

from cathedral.orchestrator.v5 import CathedralOrchestratorV5 as V5
from cathedral.orchestrator.v5_1 import CathedralOrchestratorV5_1 as V5_1


VERSION_MAP = {
    "5.0.0": V5,
    "5.1.0": V5_1,
}

LATEST = "5.1.0"


def create_orchestrator(version: Optional[str] = None,
                        model_path: Optional[str] = None,
                        **kwargs) -> Any:
    """
    Cria orquestrador da versão especificada.

    Args:
        version: string de versão (ex: "5.1.0") ou None para latest
        model_path: caminho para modelo GGUF
        **kwargs: argumentos passados ao construtor do orquestrador

    Returns:
        Instância do orquestrador
    """
    v = version or LATEST
    if v not in VERSION_MAP:
        available = ", ".join(sorted(VERSION_MAP.keys(), reverse=True))
        raise ValueError(
            f"Versão '{v}' não disponível. Disponíveis: {available}")

    cls = VERSION_MAP[v]

    if model_path is not None:
        kwargs["model_path"] = model_path

    return cls(**kwargs)
