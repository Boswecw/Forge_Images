# ForgeAgents + ForgeImages Integration

## Documentation Standard
This README follows the Forge ecosystem documentation standard: Overview, Status, Usage, Development, Operations, Governance, References. Service-specific details are below.

## Quick Start

**For VS Code Claude:** Open `VSCODE_CLAUDE_PROMPT.md` and copy everything into VS Code Claude.

## What This Is

Integration layer allowing ForgeAgents (Python) to use ForgeImages (Rust) through a FastAPI bridge, enforcing:

> **Agents suggest, ForgeImages enforces.**

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  ForgeAgents    │────▶│  Bridge Service  │────▶│  ForgeImages    │
│  (Python)       │     │  (FastAPI)       │     │  Core (Rust)    │
└─────────────────┘     └──────────────────┘     └─────────────────┘
        │                        │                        │
        │                        │                        │
   Skill API              Audit Log              Validation
   (no file I/O)          HTTP 422               Enforced
                          on failure
```

## Files in This Package

| File | Purpose |
|------|---------|
| `VSCODE_CLAUDE_PROMPT.md` | **Start here** - Complete prompt for VS Code Claude |
| `CONTEXT.md` | Architecture context and invariants |
| `step1-rust-core.md` | ForgeImages Rust crate implementation |
| `step2-rust-cli.md` | CLI binary for Python bridge |
| `step3-python-bridge.md` | FastAPI bridge service |
| `step4-python-skill.md` | ForgeAgents skill wrapper |
| `step5-tests.md` | Agent boundary enforcement tests |

## Agent Boundaries (Enforced by Design)

| Agents CAN | Agents CANNOT |
|------------|---------------|
| Generate candidates | Skip validation |
| Select candidates | Override templates |
| Request compilation | Write files directly |
| List templates | Change failure mode |
| Read validation errors | Bypass the bridge |

## Implementation Order

1. Copy `VSCODE_CLAUDE_PROMPT.md` to VS Code Claude
2. Create Rust crate first (`forgeimages-core/`)
3. Create Python package (`forgeagents-forgeimages/`)
4. Run tests to verify enforcement
5. Start bridge service

## Verification Commands

```bash
# Rust tests
cd forgeimages-core
cargo test
cargo test --test invariants

# Python tests  
cd forgeagents-forgeimages
python -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"
pytest -v

# Start bridge
uvicorn bridge.forgeimages_bridge:app --reload --port 8789
```

## Contract Guarantee

If all tests pass:
- Agents cannot bypass ForgeImages validation
- HTTP 422 on invalid input is the enforcement mechanism
- All agent actions are audit-logged
- No file writes occur outside Rust compilation
