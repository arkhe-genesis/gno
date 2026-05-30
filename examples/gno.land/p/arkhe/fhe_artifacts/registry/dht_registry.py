#!/usr/bin/env python3
"""
ARKHE Global Mesh — Registry (DHT)
Substrato 972 — ARKHE-GLOBAL-MESH

Registro distribuido de nos e substratos usando DHT Kademlia.
Cada no e responsavel por uma fatia do hash space.
"""

import hashlib
import json
from typing import Dict, List, Optional
from dataclasses import dataclass

class KademliaDHT:
    """DHT para registro global de nos."""

    def __init__(self, k: int = 20, alpha: int = 3):
        self.k = k  # Tamanho da bucket
        self.alpha = alpha  # Paralelismo de lookup
        self.buckets: Dict[int, List[str]] = {}  # Prefixo -> [node_ids]
        self.storage: Dict[str, any] = {}  # Chave -> Valor

    def _distance(self, a: str, b: str) -> int:
        """Distancia XOR entre dois hashes."""
        return int(a, 16) ^ int(b, 16)

    def store(self, key: str, value: any) -> bool:
        """Armazena valor na DHT."""
        self.storage[key] = value
        return True

    def find_node(self, target_id: str) -> List[str]:
        """Encontra k nos mais proximos do target."""
        distances = []
        for node_id in self.storage:
            dist = self._distance(node_id, target_id)
            distances.append((dist, node_id))

        distances.sort()
        return [node_id for _, node_id in distances[:self.k]]

    def find_value(self, key: str) -> Optional[any]:
        """Busca valor na DHT."""
        return self.storage.get(key)
