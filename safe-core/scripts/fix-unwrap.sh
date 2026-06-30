#!/usr/bin/env bash
# =============================================================================
# fix-unwrap.sh — Replace .unwrap() with proper error handling
# =============================================================================
# Run: bash scripts/fix-unwrap.sh
# CAUTION: Review changes before committing! This is a best-effort tool.
# =============================================================================

set -euo pipefail

find crates/ -name '*.rs' -not -path '*_test.rs' -not -name 'test*.rs' | while read -r file; do
    # Skip test modules
    if grep -q '#\[cfg(test)\]' "$file"; then
        # Process only code before the test module
        python3 -c "
import re, sys

with open('$file', 'r') as f:
    content = f.read()

# Find the test module boundary
test_match = re.search(r'#\[cfg\(test\)\]', content)
if test_match:
    before_test = content[:test_match.start()]
    test_section = content[test_match.start():]
else:
    before_test = content
    test_section = ''

# Replace .unwrap() with ? in non-test code
# Pattern: expr.unwrap() → expr?
fixed = re.sub(
    r'(\w[\w\.\:\:\(\)\[\]\<\>\&\*\s]+)\.unwrap\(\)',
    r'\1?',
    before_test
)

# Write back
with open('$file', 'w') as f:
    f.write(fixed + test_section)
" 2>/dev/null || true
    fi
done

echo "✅ unwrap() replacement complete. Review with:"
echo "   git diff"