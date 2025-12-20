# ForgeImages

**Template-Driven Image Asset Pipeline for the Forge Ecosystem**

ForgeImages is a Rust-based image compilation system that enforces strict validation rules through templates. It provides deterministic, auditable image asset generation with a clear separation between agent suggestions and system enforcement.

**Status:** Planning / Pre-Implementation
**Language:** Rust (core), Python (agent bridge)

---

## Core Principle

> **Agents suggest, ForgeImages enforces.**

AI agents can generate and select image candidates, but all validation and compilation must pass through ForgeImages' template-defined rules. No bypasses allowed.

---

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  ForgeAgents    │────▶│  Bridge Service  │────▶│  ForgeImages    │
│  (Python)       │     │  (FastAPI)       │     │  Core (Rust)    │
└─────────────────┘     └──────────────────┘     └─────────────────┘
        │                        │                        │
   MCP Tool               Audit Log              Validation
   Skill Call             HTTP 422               Enforced
```

### Integration with Forge Ecosystem

| Integration | Description |
|-------------|-------------|
| **VibeForge** | Primary consumer - prompt-to-image workflows |
| **ForgeAgents** | Agent orchestration via MCP tools |
| **AuthorForge** | Future - text composition, print PDF, CMYK |

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Template Contracts** | JSON-defined validation rules per asset type |
| **PrintAuthority Enum** | Clear permission levels, no conditional sprawl |
| **SHA-256 Manifests** | Cryptographic hashing for legal defensibility |
| **Deterministic Builds** | Reproducible output for CI/CD pipelines |
| **Rule/Policy Separation** | Validation rules separate from failure handling |
| **Quantization Buckets** | Template-configurable tolerances |

---

## Agent Boundaries (Enforced)

| Agents CAN | Agents CANNOT |
|------------|---------------|
| Generate candidates | Skip validation |
| Select candidates | Override templates |
| Request compilation | Write files directly |
| List templates | Change failure mode |

---

## Project Structure

```
ForgeImages/
├── README.md                    # This file
├── forgeimages-implementation/  # Step-by-step Rust core implementation
│   ├── step1-setup.md          # Cargo.toml dependencies
│   ├── step2-lib.md            # Module structure
│   ├── step3-templates.md      # Template contracts
│   ├── step4-validation.md     # Rule/policy separation
│   ├── step5-hashing.md        # SHA-256 manifests
│   ├── step6-print.md          # PrintAuthority enum
│   ├── step7-pipeline.md       # CompilationPipeline
│   ├── step8-tauri.md          # Tauri command exposure
│   ├── step9-template-example.md # PWA icon template
│   └── step10-invariant-tests.md # Contract tests
└── forgeimages-agents/          # ForgeAgents integration
    ├── VSCODE_CLAUDE_PROMPT.md  # Complete implementation prompt
    ├── step1-mcp-tool.md        # MCP tool schema
    ├── step2-skill.md           # ForgeAgents skill wrapper
    ├── step3-audit.md           # Audit logging
    ├── step4-bridge.md          # FastAPI bridge service
    └── step5-tests.md           # Agent boundary tests
```

---

## Implementation Status

- [ ] Core Rust library (`forgeimages-core`)
- [ ] Template system
- [ ] Validation pipeline
- [ ] SHA-256 manifest generation
- [ ] Tauri integration for VibeForge
- [ ] ForgeAgents bridge service
- [ ] MCP tool definitions
- [ ] Invariant tests

---

## Quick Start (When Implemented)

```bash
# Build the core library
cd forgeimages-core
cargo build

# Run validation tests
cargo test
cargo test --test invariants

# Start the bridge service (for agent integration)
python -m bridge.forgeimages_bridge
```

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| Core Engine | Rust |
| Version Handling | semver crate |
| Hashing | sha2 crate (SHA-256) |
| Desktop Integration | Tauri 2.0 |
| Agent Bridge | FastAPI (Python) |
| Serialization | serde, serde_json |

---

## Exposed Tauri Commands

Only two commands exposed to the frontend (minimal attack surface):

| Command | Purpose |
|---------|---------|
| `compile_asset` | Run full pipeline with template |
| `validate_asset` | Validate without compilation |

---

**Maintained by:** Boswell Digital Solutions LLC
**Part of:** Forge Ecosystem v5.3
