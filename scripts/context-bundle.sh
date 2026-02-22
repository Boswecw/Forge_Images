#!/usr/bin/env bash
# BDS Documentation Protocol v1.0 — context-bundle.sh
# Generates a focused context bundle for AI sessions.
#
# Usage:
#   ./scripts/context-bundle.sh              # Full bundle (all parts)
#   ./scripts/context-bundle.sh architecture # Trust boundary + data flow
#   ./scripts/context-bundle.sh backend      # Rust internals + templates + validation
#   ./scripts/context-bundle.sh bridge       # Python bridge + skill + API layer
#   ./scripts/context-bundle.sh testing      # Test suite reference
#   ./scripts/context-bundle.sh config       # Environment variables + CLI flags
#   ./scripts/context-bundle.sh handover     # Constraints + deployment + future work

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."
DOC_DIR="$ROOT/doc/system"
OUT="$ROOT/context-bundle.md"
PRESET="${1:-full}"

echo "ForgeImages context-bundle.sh — preset: $PRESET"

write_header() {
  echo "# ForgeImages — Context Bundle ($PRESET)" > "$OUT"
  echo "_Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) | Preset: ${PRESET}_" >> "$OUT"
  echo "" >> "$OUT"
  echo "## Critical Rules (always in scope)" >> "$OUT"
  echo "" >> "$OUT"
  echo "1. Agents suggest, ForgeImages enforces — no validation bypass allowed" >> "$OUT"
  echo "2. compile_asset() ALWAYS calls validate_asset() — no code path skips this" >> "$OUT"
  echo "3. HTTP 422 = validation failure — agents cannot proceed past 422" >> "$OUT"
  echo "4. CLI exit code 2 = validation failure — bridge interprets as 422" >> "$OUT"
  echo "5. Audit log is append-only — never modify or truncate" >> "$OUT"
  echo "6. No file paths cross the trust boundary — all data is base64" >> "$OUT"
  echo "" >> "$OUT"
}

append_file() {
  local file="$1"
  if [[ -f "$file" ]]; then
    echo "" >> "$OUT"
    echo "---" >> "$OUT"
    echo "" >> "$OUT"
    cat "$file" >> "$OUT"
  else
    echo "Warning: $file not found, skipping"
  fi
}

append_source() {
  local file="$1"
  local label="${2:-$file}"
  local lang="${3:-rust}"
  if [[ -f "$ROOT/$file" ]]; then
    echo "" >> "$OUT"
    echo "---" >> "$OUT"
    echo "<!-- source: $label -->" >> "$OUT"
    echo "\`\`\`${lang}" >> "$OUT"
    cat "$ROOT/$file" >> "$OUT"
    echo "\`\`\`" >> "$OUT"
  fi
}

write_header

case "$PRESET" in
  full)
    for f in "$DOC_DIR"/0*.md; do
      append_file "$f"
    done
    ;;

  architecture)
    # Trust boundary + data flow + overview
    append_file "$DOC_DIR/01-overview-philosophy.md"
    append_file "$DOC_DIR/02-architecture.md"
    append_file "$DOC_DIR/08-ecosystem-integration.md"
    ;;

  backend)
    # Rust internals + templates + validation + hashing + pipeline
    append_file "$DOC_DIR/03-tech-stack.md"
    append_file "$DOC_DIR/07-backend-internals.md"
    append_source "forgeimages-core/src/lib.rs" "lib.rs" "rust"
    append_source "forgeimages-core/src/pipeline.rs" "pipeline.rs" "rust"
    append_source "forgeimages-core/src/validation.rs" "validation.rs" "rust"
    ;;

  bridge)
    # Python bridge + skill + API layer
    append_file "$DOC_DIR/06-api-layer.md"
    append_file "$DOC_DIR/09-error-handling.md"
    append_source "forgeagents-forgeimages/bridge/forgeimages_bridge.py" "bridge.py" "python"
    append_source "forgeagents-forgeimages/skill/forgeimages_skill.py" "skill.py" "python"
    ;;

  testing)
    # Test suite reference
    append_file "$DOC_DIR/10-testing.md"
    append_source "forgeimages-core/tests/invariants.rs" "invariants.rs" "rust"
    append_source "forgeagents-forgeimages/tests/test_agent_boundary.py" "test_agent_boundary.py" "python"
    ;;

  config)
    # Environment variables + CLI flags + template config
    append_file "$DOC_DIR/05-config-env.md"
    append_source "forgeimages-core/templates/pwa-icon.json" "pwa-icon.json" "json"
    ;;

  handover)
    # Constraints + deployment + future work
    append_file "$DOC_DIR/11-handover.md"
    append_file "$DOC_DIR/01-overview-philosophy.md"
    ;;

  *)
    echo "Unknown preset: $PRESET"
    echo "Available presets: full, architecture, backend, bridge, testing, config, handover"
    exit 1
    ;;
esac

LINE_COUNT=$(wc -l < "$OUT")
echo ""
echo "Context bundle written: $OUT"
echo "   $LINE_COUNT lines | preset: $PRESET"
echo ""
echo "To use with Claude: copy $OUT content into your session."
