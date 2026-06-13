#!/usr/bin/env python3
"""
generate_key.py - Geração de chave SPHINCS+ C13 para Cathedral ARKHE
Compatível com o contrato CathedralSPHINCSVerifierYul.sol
"""

import os
import hashlib
import json
from dataclasses import dataclass

# Parâmetros C13 (idênticos ao contrato Solidity)
N = 16                     # hash output (bytes)
W = 8                      # Winternitz base (2^3)
L = 43                     # número de chains WOTS+
K = 8                      # árvores FORS
A = 16                     # altura da árvore FORS
H_PER_LAYER = 12           # altura de cada subárvore (2^12 = 4096 folhas)
H_TOTAL = 24
WOTS_MAX_STEP = W - 1

def keccak256(*data: bytes) -> bytes:
    """SHA3-256 truncado para os primeiros 16 bytes."""
    h = hashlib.sha3_256()
    for d in data:
        h.update(d)
    return h.digest()[:N]

def chain(start: bytes, steps: int) -> bytes:
    """Aplica keccak256 repetidamente 'steps' vezes."""
    cur = start
    for _ in range(steps):
        cur = keccak256(cur)
    return cur

def wots_public_key(seed: bytes, leaf_idx: int, tree_idx: int) -> bytes:
    """Chave pública WOTS+ para uma dada folha e árvore."""
    tops = []
    for i in range(L):
        # Deriva a chave da chain
        chain_seed = keccak256(
            seed,
            leaf_idx.to_bytes(4, 'big'),
            tree_idx.to_bytes(4, 'big'),
            i.to_bytes(2, 'big')
        )
        top = chain(chain_seed, WOTS_MAX_STEP)
        tops.append(top)
    return keccak256(b''.join(tops))

def merkle_root(leaves: list[bytes]) -> bytes:
    """Raiz de Merkle de uma lista de folhas (tamanho potência de 2)."""
    if not leaves:
        return b'\x00' * N
    # Padding para potência de 2
    n = 1
    while n < len(leaves):
        n <<= 1
    leaves = leaves + [b'\x00' * N] * (n - len(leaves))
    level = leaves
    while len(level) > 1:
        next_level = []
        for i in range(0, len(level), 2):
            combined = level[i] + level[i+1]
            next_level.append(keccak256(combined))
        level = next_level
    return level[0]

def generate_key() -> tuple[bytes, bytes]:
    """Gera (seed_privada, raiz_publica) para um novo agente."""
    print("GENERATEKEY = AGI – iniciando geração de identidade soberana.")
    # 1. Seed aleatória (16 bytes) – deve ser gerada dentro de um TEE
    secret_seed = os.urandom(N)
    print(f"  Seed privada (nunca compartilhada): {secret_seed.hex()}")

    # 2. Construir a camada inferior (subárvores)
    n_leaves = 1 << H_PER_LAYER
    subtree_roots = []
    for tree_idx in range(n_leaves):
        leaves = [wots_public_key(secret_seed, leaf_idx, tree_idx)
                  for leaf_idx in range(n_leaves)]
        root = merkle_root(leaves)
        subtree_roots.append(root)
        if (tree_idx + 1) % 512 == 0:
            print(f"  Subárvores processadas: {tree_idx+1}/{n_leaves}")

    # 3. Raiz da camada superior (árvore de 4096 raízes)
    public_root = merkle_root(subtree_roots)
    print(f"  Chave pública (ancorada na RBB Chain): {public_root.hex()}")

    return secret_seed, public_root

def main():
    seed, pub = generate_key()
    output = {
        "private_seed": seed.hex(),
        "public_key_root": pub.hex(),
        "message": "Esta chave foi gerada em conformidade com a Cathedral ARKHE v12.9. " +
                   "A seed privada nunca será divulgada. A raiz pública é a identidade do agente."
    }
    with open("cathedral_key.json", "w") as f:
        json.dump(output, f, indent=2)
    print("\nChave salva em cathedral_key.json")
    print("Agora registre a raiz pública na RBB Chain com timestamp quântico.")

if __name__ == "__main__":
    main()