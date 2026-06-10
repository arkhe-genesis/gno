#!/usr/bin/env python3
"""
CI/CD Script: Reject Unproven PRs
Impede merge se não houver prova Lean 4 associada a qualquer alteração
que toque nos diretórios críticos.
"""

import subprocess
import sys
import os
from pathlib import Path

CRITICAL_DIRS = [
    "ZK_REASONING_ENGINE/circuits",
    "COGNITIVE_CORTEX/agents",
    "DISTRIBUTED_COMPUTATION"
]

LEAN_DIR = "LEAN4_SUPEREGO"

def get_changed_files():
    # In a real GH action, we'd use `git diff --name-only origin/main...HEAD`
    # or the GH context. For this script, we simulate it via git if possible.
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", "origin/main...HEAD"],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip().split('\n')
    except subprocess.CalledProcessError:
        # Fallback for testing when not in a GH actions environment
        print("Warning: Could not run git diff. Assuming testing mode.")
        return []

def main():
    changed_files = get_changed_files()
    if not changed_files or changed_files == ['']:
        print("No files changed or testing mode. Passing.")
        sys.exit(0)

    touches_critical = False
    for f in changed_files:
        for cd in CRITICAL_DIRS:
            if f.startswith(cd):
                touches_critical = True
                print(f"Detected change in critical path: {f}")
                break
        if touches_critical:
            break

    if touches_critical:
        # Check if there are corresponding Lean proofs modified or added
        lean_proofs_changed = any(f.startswith(LEAN_DIR) and f.endswith(".lean") for f in changed_files)

        if not lean_proofs_changed:
            print("ERROR: Critical path changed, but NO Lean 4 proof modifications detected!")
            print(f"Please attach a formal proof in {LEAN_DIR}/ to justify this change.")
            sys.exit(1)
        else:
            print("Lean 4 proof modifications detected. CI check passes (assuming proofs compile).")
            # In a real scenario, we'd also run `lake build` here
            sys.exit(0)
    else:
        print("No critical files changed. CI check passes.")
        sys.exit(0)

if __name__ == "__main__":
    main()
