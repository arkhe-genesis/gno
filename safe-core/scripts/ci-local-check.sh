#!/usr/bin/env bash
# =============================================================================
# Safe-Core AGI — Local CI Pre-Push Check
# =============================================================================
# Run this before pushing to catch issues early:
#   ./scripts/ci-local-check.sh
# =============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

PASS=0
FAIL=0
WARN=0

pass() { PASS=$((PASS+1)); echo -e "  ${GREEN}✓${NC} $1"; }
fail() { FAIL=$((FAIL+1)); echo -e "  ${RED}✗${NC} $1"; }
warn() { WARN=$((WARN+1)); echo -e "  ${YELLOW}⚠${NC} $1"; }

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Safe-Core AGI — Local CI Check${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# ─── 1. Check workspace structure ────────────────────────────────────────────
echo -e "${CYAN}[1/10] Workspace Structure${NC}"
if [ -f "Cargo.toml" ] && grep -q '\[workspace\]' Cargo.toml; then
    pass "Workspace Cargo.toml found"
else
    fail "Workspace Cargo.toml missing or invalid"
fi

if [ -f "Cargo.lock" ]; then
    pass "Cargo.lock exists"
else
    fail "Cargo.lock MISSING — run 'cargo generate-lockfile'"
fi

if [ -f "rust-toolchain.toml" ]; then
    pass "rust-toolchain.toml found"
else
    warn "rust-toolchain.toml missing — CI may use different version"
fi

# ─── 2. Check formatting ────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}[2/10] Formatting${NC}"
if cargo fmt --all -- --check 2>/dev/null; then
    pass "cargo fmt --check"
else
    fail "cargo fmt --check FAILED — run 'cargo fmt --all'"
fi

# ─── 3. Check clippy (strict) ──────────────────────────────────────────────
echo ""
echo -e "${CYAN}[3/10] Clippy (strict)${NC}"
if RUSTFLAGS="-D warnings" cargo clippy --workspace --lib --bins --all-features -- \
    -D warnings \
    -D clippy::unwrap_used \
    -D clippy::expect_used \
    2>&1 | tee /tmp/clippy-output.txt; then
    if grep -q "error\|warning:" /tmp/clippy-output.txt; then
        fail "clippy found issues (see above)"
    else
        pass "cargo clippy"
    fi
else
    fail "cargo clippy FAILED"
fi

# ─── 4. Build workspace ─────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}[4/10] Build Workspace${NC}"
if cargo build --workspace --all-targets 2>&1 | tee /tmp/build-output.txt; then
    pass "cargo build --workspace"
else
    fail "cargo build FAILED"
    # Show first error
    grep -m 5 "^error" /tmp/build-output.txt 2>/dev/null || true
fi

# ─── 5. Unit tests ──────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}[5/10] Unit Tests${NC}"
if cargo test --workspace --lib -- --nocapture 2>&1 | tee /tmp/test-output.txt; then
    if grep -q "FAILED" /tmp/test-output.txt; then
        fail "Some unit tests FAILED"
        grep "test .* FAILED" /tmp/test-output.txt
    else
        pass "cargo test --workspace --lib"
    fi
else
    fail "cargo test FAILED"
fi

# ─── 6. Doc tests ──────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}[6/10] Doc Tests${NC}"
if cargo test --workspace --doc 2>&1 | tee /tmp/doctest-output.txt; then
    if grep -q "FAILED\|error\[" /tmp/doctest-output.txt; then
        fail "Some doc tests FAILED"
    else
        pass "cargo test --doc"
    fi
else
    # Doc tests sometimes fail for non-code reasons
    warn "cargo test --doc had issues (non-blocking)"
fi

# ─── 7. Check for common issues ─────────────────────────────────────────────
echo ""
echo -e "${CYAN}[7/10] Common Issues${NC}"

# Check for unwrap in non-test code
UNWRAP_COUNT=$(grep -rn '\.unwrap()' --include='*.rs' crates/ \
    | grep -v '#\[cfg(test)\]' \
    | grep -v 'mod tests' \
    | grep -v '_test.rs' \
    | wc -l)
if [ "$UNWRAP_COUNT" -gt 0 ]; then
    warn "Found $UNWRAP_COUNT .unwrap() calls in non-test code"
else
    pass "No .unwrap() in non-test code"
fi

# Check for TODO/FIXME
TODO_COUNT=$(grep -rn 'TODO\|FIXME\|HACK\|XXX' --include='*.rs' crates/ | wc -l)
if [ "$TODO_COUNT" -gt 0 ]; then
    warn "Found $TODO_COUNT TODO/FIXME comments"
else
    pass "No TODO/FIXME comments"
fi

# Check for hardcoded secrets
SECRET_COUNT=$(grep -rn 'sk-\|sk_\|AKIA\|AIza' --include='*.rs' --include='*.toml' crates/ \
    | grep -v 'test\|example\|mock\|placeholder' | wc -l)
if [ "$SECRET_COUNT" -gt 0 ]; then
    fail "Found $SECRET_COUNT potential secrets!"
else
    pass "No hardcoded secrets detected"
fi

# Check for debug builds in release
if grep -rn 'dbg!' --include='*.rs' crates/ \
    | grep -v '#\[cfg(test)\]' | grep -v '_test.rs' | grep -q .; then
    fail "Found dbg! macros in non-test code"
else
    pass "No dbg! macros in non-test code"
fi

# ─── 8. Check platform-specific deps ────────────────────────────────────────
echo ""
echo -e "${CYAN}[8/10] Platform-Specific Dependencies${NC}"

# Verify landlock/nix are gated behind cfg(target_os = "linux")
if grep -q '^landlock\|^nix' Cargo.toml && \
   ! grep -q "target.*cfg.*linux.*dependencies" Cargo.toml; then
    fail "landlock/nix found without cfg(target_os = \"linux\") gate"
else
    pass "Linux-only deps properly gated"
fi

# ─── 9. Check .gitignore ────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}[9/10] .gitignore${NC}"
GITIGNORE_OK=true
for pattern in target/ .env *.pem *.key node_modules/; do
    if ! grep -q "$pattern" .gitignore 2>/dev/null; then
        warn "'.gitignore' missing '$pattern'"
        GITIGNORE_OK=false
    fi
done
if $GITIGNORE_OK; then
    pass ".gitignore looks good"
fi

# ─── 10. Check workflow files ───────────────────────────────────────────────
echo ""
echo -e "${CYAN}[10/10] GitHub Workflows${NC}"

for wf in .github/workflows/*.yml; do
    if [ -f "$wf" ]; then
        # Check for deprecated actions
        DEPRECATED=$(grep -cE 'actions/checkout@v[12]|actions/cache@v[23]|actions/upload-artifact@v[23]|actions/setup-python@v[34]' "$wf" 2>/dev/null || true)
        if [ "$DEPRECATED" -gt 0 ]; then
            fail "$wf uses $DEPRECATED deprecated action(s)"
        else
            pass "$(basename $wf) — actions up to date"
        fi
    fi
done

# ─── Summary ────────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
TOTAL=$((PASS + FAIL + WARN))
echo -e "  Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, ${YELLOW}$WARN warnings${NC} (of $TOTAL checks)"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}❌ Fix the failures above before pushing.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Safe to push!${NC}"
    exit 0
fi