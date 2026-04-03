#!/bin/bash
set -euo pipefail

PARTS_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$PARTS_DIR/../.." && pwd)"
ROOT_OUTPUT="$REPO_ROOT/SYSTEM.md"
DOC_OUTPUT="$REPO_ROOT/doc/SYSTEM.md"
LEGACY_OUTPUT_REL="doc/fiSYSTEM.md"
TMP_OUTPUT="$(mktemp)"

echo "Assembling SYSTEM.md..."

cat "$PARTS_DIR/_index.md" > "$TMP_OUTPUT"

for part in "$PARTS_DIR"/[0-9][0-9]-*.md; do
  echo "" >> "$TMP_OUTPUT"
  echo "---" >> "$TMP_OUTPUT"
  echo "" >> "$TMP_OUTPUT"
  cat "$part" >> "$TMP_OUTPUT"
done

cp "$TMP_OUTPUT" "$ROOT_OUTPUT"
cp "$TMP_OUTPUT" "$DOC_OUTPUT"

if [[ -n "$LEGACY_OUTPUT_REL" ]]; then
  LEGACY_OUTPUT="$REPO_ROOT/$LEGACY_OUTPUT_REL"
  mkdir -p "$(dirname "$LEGACY_OUTPUT")"
  cp "$TMP_OUTPUT" "$LEGACY_OUTPUT"
fi

chmod 664 "$ROOT_OUTPUT" "$DOC_OUTPUT"
if [[ -n "$LEGACY_OUTPUT_REL" ]]; then
  chmod 664 "$REPO_ROOT/$LEGACY_OUTPUT_REL"
fi

LINE_COUNT=$(wc -l < "$ROOT_OUTPUT")
rm -f "$TMP_OUTPUT"
echo "SYSTEM.md assembled: $LINE_COUNT lines"
