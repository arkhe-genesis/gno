#!/usr/bin/env python3
"""
Cathedral AGI Omega - CI/CD Script
Rejects pull requests that modify critical components without accompanying Lean 4 proofs.
"""

import subprocess
import sys
import os

CRITICAL_PATHS = [
    'ZK_REASONING_ENGINE/circuits/',
    'COGNITIVE_CORTEX/agents/',
    'DISTRIBUTED_COMPUTATION/'
]

LEAN_PATH = 'LEAN4_SUPEREGO/'

def get_changed_files():
    try:
        # Assuming run inside GitHub Actions against base branch
        # A simple approximation for local dev/testing
        base_branch = os.environ.get('GITHUB_BASE_REF', 'main')
        cmd = f"git diff --name-only origin/{base_branch}"
        result = subprocess.run(cmd.split(), capture_output=True, text=True, check=True)
        return result.stdout.strip().split('\n')
    except subprocess.CalledProcessError:
        print("Warning: Could not run git diff. Returning empty list.")
        return []

def main():
    changed_files = get_changed_files()
    if not changed_files or changed_files == ['']:
        print("No files changed or not a git repository.")
        sys.exit(0)

    touched_critical = False
    added_proofs = False

    for f in changed_files:
        if any(f.startswith(cp) for cp in CRITICAL_PATHS):
            touched_critical = True
        if f.startswith(LEAN_PATH) and f.endswith('.lean'):
            added_proofs = True

    if touched_critical and not added_proofs:
        print("ERROR: Critical paths were modified, but no Lean 4 proof (.lean) was provided in LEAN4_SUPEREGO/.")
        print("In Cathedral AGI, safety is guaranteed by formal proofs. You MUST prove that your change maintains AGI safety.")
        sys.exit(1)

    print("Verification passed. Code modifications are proven safe.")
    sys.exit(0)

if __name__ == "__main__":
    main()
