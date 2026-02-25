#!/usr/bin/env bash
# BDS Documentation Protocol v2.0 — BUILD.sh
# Assembles numbered section files into fiSYSTEM.md
# Usage: bash doc/system/BUILD.sh

set -euo pipefail

PREFIX="fi"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT="${SCRIPT_DIR}/../${PREFIX}SYSTEM.md"

# Write _index.md header
cat "${SCRIPT_DIR}/_index.md" > "${OUTPUT}"
printf '\n---\n' >> "${OUTPUT}"

# Concatenate all numbered sections in order
for part in "${SCRIPT_DIR}"/[0-9][0-9]-*.md; do
  [ -f "$part" ] || continue
  printf '\n' >> "${OUTPUT}"
  cat "${part}" >> "${OUTPUT}"
  printf '\n---\n' >> "${OUTPUT}"
done

echo "${PREFIX}SYSTEM.md rebuilt ($(wc -l < "${OUTPUT}") lines)"
