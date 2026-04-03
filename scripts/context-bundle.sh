#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SYSTEM_DIR="$REPO_ROOT/doc/system"
OUTPUT="$REPO_ROOT/context-bundle.md"

WITH_ROADMAP=false
WITH_SPECS=false
DRY_RUN=false
LIST_ONLY=false
INCLUDE_ALL=false
PRESET=""
declare -a SECTIONS=()

available_sections() {
  for part in "$SYSTEM_DIR"/[0-9][0-9]-*.md; do
    basename "$part" | cut -d- -f1
  done
}

available_parts() {
  find "$SYSTEM_DIR" -maxdepth 1 -type f -name '[0-9][0-9]-*.md' | sort
}

add_if_exists() {
  local section="$1"
  local match=("$SYSTEM_DIR"/"$section"-*.md)
  [[ -e "${match[0]}" ]] && SECTIONS+=("$section")
}

append_unique() {
  local value="$1"
  local existing
  for existing in "${SECTIONS[@]}"; do
    [[ "$existing" == "$value" ]] && return 0
  done
  SECTIONS+=("$value")
}

show_list() {
  echo "Available sections:"
  for part in "$SYSTEM_DIR"/[0-9][0-9]-*.md; do
    echo "  $(basename "$part")"
  done
  echo
  echo "Presets:"
  echo "  core"
  echo "  foundation"
  echo "  governance"
  echo "  docs"
  echo "  architecture"
  echo "  config"
  echo "  testing"
  echo "  handover"
  echo "  frontend"
  echo "  backend"
  echo "  api"
  echo "  schema"
  echo "  integration"
}

select_by_keywords() {
  local keywords=("$@")
  local part
  local section
  local slug
  SECTIONS=()
  while IFS= read -r part; do
    section="$(basename "$part" | cut -d- -f1)"
    slug="$(basename "$part" | tr '[:upper:]' '[:lower:]')"
    for keyword in "${keywords[@]}"; do
      if [[ "$slug" == *"$keyword"* ]]; then
        append_unique "$section"
        break
      fi
    done
  done < <(available_parts)
}

resolve_preset() {
  SECTIONS=()
  case "$1" in
    core|foundation)
      while IFS= read -r section; do
        case "$section" in
          01|02|03|04|05) append_unique "$section" ;;
        esac
      done < <(available_sections)
      while IFS= read -r section; do
        append_unique "$section"
      done < <(available_sections | tail -n 3)
      ;;
    governance)
      select_by_keywords governance boundary doctrine policy security error testing handover
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset testing
      else
        while IFS= read -r section; do
          case "$section" in
            01|02) append_unique "$section" ;;
          esac
        done < <(available_sections)
      fi
      ;;
    docs|documentation)
      while IFS= read -r section; do
        case "$section" in
          01|02|04|05) append_unique "$section" ;;
        esac
      done < <(available_sections)
      while IFS= read -r section; do
        append_unique "$section"
      done < <(available_sections | tail -n 2)
      ;;
    architecture|config)
      select_by_keywords architecture overview structure config environment tech stack
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset docs
      fi
      ;;
    testing)
      select_by_keywords testing error handover qa audit
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        while IFS= read -r section; do
          append_unique "$section"
        done < <(available_sections | tail -n 3)
      fi
      ;;
    handover)
      select_by_keywords handover migration error testing
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset testing
      fi
      ;;
    frontend)
      select_by_keywords frontend design tauri browser component ui
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset core
      fi
      ;;
    backend)
      select_by_keywords backend runtime service command api worker internals
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset core
      fi
      ;;
    api)
      select_by_keywords api route proxy middleware command
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset backend
      fi
      ;;
    schema)
      select_by_keywords schema database migration model persistence sql
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset core
      fi
      ;;
    integration)
      select_by_keywords integration ecosystem ai provider adapter forge
      if [[ ${#SECTIONS[@]} -eq 0 ]]; then
        resolve_preset core
      fi
      ;;
    *)
      echo "Unknown preset: $1" >&2
      exit 1
      ;;
  esac
}

resolve_section_file() {
  local section="$1"
  local matches=("$SYSTEM_DIR"/"$section"-*.md)
  if [[ ! -e "${matches[0]}" ]]; then
    echo "Missing section file for $section" >&2
    exit 1
  fi
  printf '%s\n' "${matches[0]}"
}

collect_roadmap_files() {
  find "$REPO_ROOT/docs" -maxdepth 1 -type f -iname '*roadmap*.md' | sort
}

collect_spec_files() {
  find "$REPO_ROOT/docs" -type f \( -iname '*spec*.md' -o -iname '*architecture*.md' \) | sort
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --list)
      LIST_ONLY=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --preset)
      PRESET="${2:-}"
      shift 2
      ;;
    --all)
      INCLUDE_ALL=true
      shift
      ;;
    --with-roadmap)
      WITH_ROADMAP=true
      shift
      ;;
    --with-specs)
      WITH_SPECS=true
      shift
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        printf -v PADDED '%02d' "$1"
        SECTIONS+=("$PADDED")
        shift
      else
        echo "Unknown argument: $1" >&2
        exit 1
      fi
      ;;
  esac
done

if [[ "$LIST_ONLY" == true ]]; then
  show_list
  exit 0
fi

if [[ -n "$PRESET" ]]; then
  resolve_preset "$PRESET"
fi

if [[ "$INCLUDE_ALL" == true ]]; then
  SECTIONS=()
  while IFS= read -r section; do
    SECTIONS+=("$section")
  done < <(available_sections)
fi

if [[ ${#SECTIONS[@]} -eq 0 ]]; then
  resolve_preset core
fi

if [[ "$DRY_RUN" == true ]]; then
  echo "Would assemble:"
  for section in "${SECTIONS[@]}"; do
    file="$(resolve_section_file "$section")"
    echo "  $section -> $(basename "$file") ($(wc -l < "$file") lines)"
  done
  if [[ "$WITH_ROADMAP" == true ]]; then
    while IFS= read -r roadmap; do
      [[ -n "$roadmap" ]] || continue
      echo "  roadmap -> $(basename "$roadmap") ($(wc -l < "$roadmap") lines)"
    done < <(collect_roadmap_files)
  fi
  if [[ "$WITH_SPECS" == true ]]; then
    while IFS= read -r spec; do
      [[ -n "$spec" ]] || continue
      echo "  spec -> $(basename "$spec") ($(wc -l < "$spec") lines)"
    done < <(collect_spec_files)
  fi
  exit 0
fi

cat "$SYSTEM_DIR/_index.md" > "$OUTPUT"

for section in "${SECTIONS[@]}"; do
  file="$(resolve_section_file "$section")"
  echo "" >> "$OUTPUT"
  echo "---" >> "$OUTPUT"
  echo "" >> "$OUTPUT"
  cat "$file" >> "$OUTPUT"
done

if [[ "$WITH_ROADMAP" == true ]]; then
  while IFS= read -r roadmap; do
    [[ -n "$roadmap" ]] || continue
    echo "" >> "$OUTPUT"
    echo "---" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    cat "$roadmap" >> "$OUTPUT"
  done < <(collect_roadmap_files)
fi

if [[ "$WITH_SPECS" == true ]]; then
  while IFS= read -r spec; do
    [[ -n "$spec" ]] || continue
    echo "" >> "$OUTPUT"
    echo "---" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    cat "$spec" >> "$OUTPUT"
  done < <(collect_spec_files)
fi

echo "context-bundle.md assembled: $(wc -l < "$OUTPUT") lines"
