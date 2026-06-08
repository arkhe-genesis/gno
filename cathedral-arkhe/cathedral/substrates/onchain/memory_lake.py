"""
MemoryLake — Substrato 1100 v1.0.0
Espelho local de canonizações on-chain com Merkle tree.
"""

from __future__ import annotations
import hashlib
import threading
import time
from typing import Dict, List, Optional, Set

from cathedral.substrates.onchain.types import (
    MemoryLakeEntry, CanonizationType, SignatureStatus
)


class MemoryLake:
    """
    Lake em memória para transições de estado canônicas.
    Atua como espelho local das canonizações on-chain.
    """

    def __init__(self, max_entries: int = 100_000):
        self.entries: Dict[str, MemoryLakeEntry] = {}
        self.ordered_hashes: List[str] = []
        self.merkle_tree: Optional[List[List[str]]] = None
        self.max_entries = max_entries
        self._lock = threading.RLock()
        self._type_index: Dict[CanonizationType, Set[str]] = {
            t: set() for t in CanonizationType
        }
        self._signer_index: Dict[str, Set[str]] = {}

    def ingest(self, entry: MemoryLakeEntry) -> bool:
        """Ingestão de nova entrada no lake."""
        with self._lock:
            # FIX BUG-3: só computa hash se entry_hash estiver vazio
            if not entry.entry_hash:
                entry.entry_hash = entry.compute_hash()

            if entry.entry_hash in self.entries:
                return False

            if len(self.entries) >= self.max_entries:
                self._evict_oldest()

            self.entries[entry.entry_hash] = entry
            self.ordered_hashes.append(entry.entry_hash)
            self._type_index[entry.entry_type].add(entry.entry_hash)

            if entry.signer:
                self._signer_index.setdefault(entry.signer, set()).add(
                    entry.entry_hash
                )

            self._invalidate_merkle()
            return True

    def _evict_oldest(self):
        if self.ordered_hashes:
            oldest = self.ordered_hashes.pop(0)
            entry = self.entries.pop(oldest, None)
            if entry:
                self._type_index[entry.entry_type].discard(oldest)
                if entry.signer:
                    self._signer_index[entry.signer].discard(oldest)

    def _invalidate_merkle(self):
        self.merkle_tree = None

    def build_merkle_tree(self) -> List[List[str]]:
        with self._lock:
            if self.merkle_tree is not None:
                return self.merkle_tree

            if not self.ordered_hashes:
                return [[hashlib.sha256(b"empty").hexdigest()]]

            leaves = [
                self.entries[h].entry_hash[2:] if h.startswith("0x") else h
                for h in self.ordered_hashes
            ]

            while len(leaves) & (len(leaves) - 1) != 0:
                leaves.append(leaves[-1])

            tree = [leaves]
            current_level = leaves

            while len(current_level) > 1:
                next_level = []
                for i in range(0, len(current_level), 2):
                    combined = current_level[i] + current_level[i + 1]
                    next_level.append(
                        hashlib.sha256(combined.encode()).hexdigest()
                    )
                tree.append(next_level)
                current_level = next_level

            self.merkle_tree = tree
            return tree

    def get_merkle_root(self) -> str:
        tree = self.build_merkle_tree()
        return "0x" + tree[-1][0]

    def get_proof(self, entry_hash: str) -> Optional[List[str]]:
        tree = self.build_merkle_tree()
        try:
            idx = self.ordered_hashes.index(entry_hash)
        except ValueError:
            return None

        proof = []
        for level in tree[:-1]:
            sibling_idx = idx ^ 1
            if sibling_idx < len(level):
                proof.append(level[sibling_idx])
            idx >>= 1
        return proof

    def get_by_type(self, entry_type: CanonizationType) -> List[MemoryLakeEntry]:
        with self._lock:
            return [
                self.entries[h] for h in self._type_index[entry_type]
                if h in self.entries
            ]

    def get_by_signer(self, signer: str) -> List[MemoryLakeEntry]:
        with self._lock:
            return [
                self.entries[h] for h in self._signer_index.get(signer, set())
                if h in self.entries
            ]

    def get_recent(self, n: int = 10) -> List[MemoryLakeEntry]:
        with self._lock:
            recent_hashes = self.ordered_hashes[-n:]
            return [self.entries[h] for h in reversed(recent_hashes)]

    def snapshot(self) -> Dict:
        return {
            "merkle_root": self.get_merkle_root(),
            "total_entries": len(self.entries),
            "type_counts": {
                t.name: len(s) for t, s in self._type_index.items()
            },
            "timestamp": time.time(),
        }

    def get_telemetry(self) -> Dict:
        return {
            "module": "MemoryLake",
            "version": "1.0.0",
            "substrate": "1100",
            "seal": "MEMORY-LAKE-1100-v1.0.0-2026-06-08",
            "total_entries": len(self.entries),
            "merkle_root": self.get_merkle_root()[:16] + "...",
            "type_distribution": {
                t.name: len(s) for t, s in self._type_index.items() if s
            },
        }
