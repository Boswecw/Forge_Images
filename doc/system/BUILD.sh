#!/usr/bin/env bash
# Forge Documentation Protocol v1 — deterministic doc/system assembler
# Usage: bash doc/system/BUILD.sh
# Output: doc/fiSYSTEM.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT="${SCRIPT_DIR}/../fiSYSTEM.md"

mapfile -t PARTS < <(
  find "${SCRIPT_DIR}" -maxdepth 1 -type f -name '[0-9][0-9]-*.md' -printf '%f\n' | sort
)

cat "${SCRIPT_DIR}/_index.md" > "${OUTPUT}"

if ((${#PARTS[@]} > 0)); then
  printf '\n---\n' >> "${OUTPUT}"
fi

for i in "${!PARTS[@]}"; do
  part="${PARTS[$i]}"
  printf '\n' >> "${OUTPUT}"
  cat "${SCRIPT_DIR}/${part}" >> "${OUTPUT}"

  if (( i < ${#PARTS[@]} - 1 )); then
    printf '\n---\n' >> "${OUTPUT}"
  fi
done

printf '%s rebuilt (%s lines)\n' "$(basename "${OUTPUT}")" "$(wc -l < "${OUTPUT}")"
