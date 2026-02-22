#!/usr/bin/env bash
# BDS Documentation Protocol v1.0 — BUILD.sh
# Assembles modular doc/system/ parts into context-bundle.md
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$SCRIPT_DIR/../SYSTEM.md"

echo "# ForgeImages — System Reference" > "$OUT"
echo "" >> "$OUT"
echo "_BDS Documentation Protocol v1.0 — Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)_" >> "$OUT"
echo "" >> "$OUT"

# Include the index
cat "${SCRIPT_DIR}/_index.md" >> "$OUT"

# Include all numbered parts
for f in "$SCRIPT_DIR"/0*.md; do
  echo "" >> "$OUT"
  echo "---" >> "$OUT"
  echo "" >> "$OUT"
  cat "$f" >> "$OUT"
done

echo ""
echo "SYSTEM.md written to: $OUT"
echo "   $(wc -l < "$OUT") lines"
