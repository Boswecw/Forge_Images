# ForgeImages — System Reference

_BDS Documentation Protocol v1.0 — Generated: 2026-02-23T16:36:14Z_

# ForgeImages System Documentation

> BDS Documentation Protocol v1.0 — modular reference for AI-assisted development

| Part | File | Contents |
|------|------|----------|
| §1 | [01-overview-philosophy.md](01-overview-philosophy.md) | Service purpose, Six Laws, ecosystem role |
| §2 | [02-architecture.md](02-architecture.md) | Trust boundary, 3-tier model, data flow |
| §3 | [03-tech-stack.md](03-tech-stack.md) | Exact dependencies and versions (Rust + Python) |
| §4 | [04-project-structure.md](04-project-structure.md) | Directory tree, key files, module responsibilities |
| §5 | [05-config-env.md](05-config-env.md) | Environment variables, CLI flags, settings |
| §6 | [06-api-layer.md](06-api-layer.md) | Bridge HTTP endpoints, CLI subcommands, exit codes |
| §7 | [07-backend-internals.md](07-backend-internals.md) | Template system, validation pipeline, hashing, print authority, compilation |
| §8 | [08-ecosystem-integration.md](08-ecosystem-integration.md) | ForgeAgents skill, VibeForge, AuthorForge integration contracts |
| §9 | [09-error-handling.md](09-error-handling.md) | The 422 contract, exit code semantics, validation violations |
| §10 | [10-testing.md](10-testing.md) | Invariant tests, boundary tests, coverage |
| §11 | [11-handover.md](11-handover.md) | Critical constraints, deployment, future work |

## Quick Assembly

```bash
./BUILD.sh         # Assembles all parts into context-bundle.md
```

*Last updated: 2026-02-19*

---

# §1 — Overview & Philosophy

## Service Identity

**ForgeImages** is the template-driven image asset pipeline for the Forge Ecosystem. It provides deterministic, auditable image asset generation with strict validation enforcement and a clear separation between agent suggestions and system enforcement.

- **Version:** v1.0.0
- **Status:** Implemented (core engine + bridge + skill)
- **Languages:** Rust 2024 (core engine + CLI), Python 3.10+ (agent bridge + skill)
- **LOC:** ~2,400 (Rust ~1,100, Python ~800, Tests ~500)

## The Six Laws (Non-Negotiable)

These are the canonical invariants. Every code path, every test, every design decision traces back to one of these laws.

1. **SVG Is Truth** — Vector masters are the canonical source; all raster exports derive from them.
2. **Templates Are Contracts** — Old templates work forever. New engines never silently change behavior. Version mismatch = clear error.
3. **Validation Is Protective** — Validation exists to catch errors before they propagate, not to punish users.
4. **Deterministic Output** — Same inputs always produce the same outputs. Manifest hashes are stable and reproducible.
5. **Manifests Enable Reproduction** — SHA-256 hashes link inputs to outputs for legal defensibility and auditability.
6. **Agents Suggest, Engine Enforces** — AI agents can generate and select candidates, but all validation and compilation must pass through ForgeImages' template-defined rules. No bypasses allowed.

## Core Principle

> **Agents suggest, ForgeImages enforces.**

This is not a guideline. It is an architectural invariant enforced at three levels:

1. **Rust type system** — `compile_asset()` always calls `validate_asset()` internally
2. **HTTP status codes** — Bridge returns 422 on validation failure; agents cannot proceed
3. **CLI exit codes** — Exit code 2 = validation failure; bridge interprets this as 422

## Ecosystem Role

```
┌──────────────────────────────────────────────────────────────┐
│                      Forge Ecosystem                          │
│                                                              │
│   VibeForge     ForgeAgents     AuthorForge                 │
│   (consumer)    (orchestration)  (future)                    │
│       │              │              │                        │
│       └──────────────┴──────────────┘                        │
│                      │                                       │
│          ┌───────────▼────────────┐                          │
│          │   ForgeImages Bridge   │  ← HTTP gateway          │
│          │   (Python / FastAPI)   │                          │
│          └───────────┬────────────┘                          │
│                      │ subprocess                            │
│          ┌───────────▼────────────┐                          │
│          │   ForgeImages Core     │  ← Validation + compile  │
│          │   (Rust 2024)          │                          │
│          └────────────────────────┘                          │
└──────────────────────────────────────────────────────────────┘
```

## What ForgeImages Is

- **A validation engine** — Enforces template-defined rules on image assets
- **A compilation pipeline** — Produces deterministic, hashed export bundles
- **An audit trail** — Every request is logged with job hashes linking inputs to outputs
- **A trust boundary** — Agents interact through the skill/bridge layer; they never touch the engine directly

## What ForgeImages Is Not

- **Not an image editor.** It does not provide editing tools; it validates and compiles assets.
- **Not a storage service.** Compiled assets are returned as base64; persistence is the caller's responsibility.
- **Not an AI model.** It does not generate images; agents upstream (VibeForge) handle generation.
- **Not a CDN.** It produces export bundles; distribution is handled elsewhere.

## Codebase Metrics

| Metric | Count |
|--------|-------|
| Rust source files | 7 (lib + 5 modules + CLI) |
| Python source files | 6 (bridge: 4, skill: 1, init: 1) |
| Test files | 2 (Rust invariants + Python boundaries) |
| Template files | 1 (pwa-icon.json) |
| Validation rules | 3 (AspectRatio, Resolution, ColorCount) |
| CLI subcommands | 3 (templates, validate, compile) |
| Bridge endpoints | 5 (health, templates, template/:id, validate/:id, compile/:id) |
| Test methods | 26+ (Rust: 6 invariant, Python: 20+ boundary) |

---

# §2 — Architecture

## Three-Tier Trust Boundary

ForgeImages enforces a strict three-tier architecture where each layer has defined responsibilities and no layer can bypass the one below it.

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  ForgeAgents    │────▶│  Bridge Service  │────▶│  ForgeImages    │
│  (Python)       │     │  (FastAPI)       │     │  Core (Rust)    │
└─────────────────┘     └──────────────────┘     └─────────────────┘
        │                        │                        │
   Skill Call             Audit Log              Validation
   (httpx)               HTTP 422               Enforced
```

### Layer 1: Agent Skill (Python — `skill/`)

The agent-facing interface. Provides async methods for template listing, validation, and compilation. Communicates exclusively via HTTP to the bridge service.

**Agents CAN:**
- Generate image candidates
- Select candidates
- Request compilation
- List available templates

**Agents CANNOT:**
- Skip validation
- Override templates
- Write files directly
- Change failure mode

### Layer 2: Bridge Service (Python — `bridge/`)

The HTTP gateway between agents and the Rust engine. Handles request validation (Pydantic v2), audit logging (append-only JSONL), and subprocess management (CLI invocation).

**Bridge responsibilities:**
- Pydantic input validation (dimensions 1-10000, template ID format)
- Audit logging with job hash linkage
- CLI subprocess execution with 30-second timeout
- Structured error responses (422 with violations + remediation)
- Request size limiting

### Layer 3: Core Engine (Rust — `forgeimages-core/`)

The validation and compilation engine. Template contracts, validation rules, SHA-256 manifest generation, and export rendering. This is the enforcement layer.

**Engine guarantees:**
- `compile_asset()` ALWAYS calls `validate_asset()` — no code path bypasses this
- Template version compatibility is checked via semver
- All outputs include deterministic manifest hashes
- Canonical JSON ensures hash stability across platforms

## Data Flow: Compile Request

```
Agent                   Bridge                    Engine
  │                       │                         │
  │──POST /compile/X────▶│                         │
  │                       │──call_cli(compile)────▶│
  │                       │                         │──validate_asset()
  │                       │                         │──check_engine_version()
  │                       │                         │──generate_exports()
  │                       │                         │──compute_job_hash()
  │                       │◀──JSON + exit code──────│
  │                       │──audit_log.log()        │
  │◀──200 CompiledAsset──│                         │
  │   or 422 violations   │                         │
```

## Data Flow: Validation Failure

```
Agent                   Bridge                    Engine
  │                       │                         │
  │──POST /validate/X───▶│                         │
  │                       │──call_cli(validate)───▶│
  │                       │                         │──run 3 rules
  │                       │                         │──apply failure_mode
  │                       │◀──exit code 2 + JSON───│
  │                       │──audit_log.log()        │
  │◀──422 + violations───│                         │
  │                       │                         │
  │  (agent CANNOT        │                         │
  │   proceed past 422)   │                         │
```

## PrintAuthority Model

The PrintAuthority enum determines the source of physical output specifications:

```
PrintAuthority::System    → Default (300 DPI, RGB, 0.125" bleed)
PrintAuthority::Template  → Template-defined specs
PrintAuthority::User      → User-provided (validated: 72-1200 DPI, 0-1" bleed)
```

This eliminates conditional sprawl — the enum determines behavior, not nested if/else chains.

## Template Contract Model

Templates are JSON files that define:
- Asset class (Icon, Cover, Banner, Logo)
- Aspect ratio with quantization tolerance
- Minimum resolution constraints
- Color count limits
- Export specifications (format, size, required flag)
- Failure mode (Block, Warn, Log)
- Engine version compatibility

Templates are versioned via semver. The engine checks `engineMinVersion` against `ENGINE_VERSION` before processing.

## Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| Rust core, Python bridge | Rust for determinism + safety; Python for agent ecosystem integration |
| CLI subprocess (not FFI) | Clean process boundary; exit codes are the enforcement mechanism |
| HTTP 422 for validation | Standard HTTP semantics; agents must handle 422 to proceed |
| Append-only audit log | Legal defensibility; no mutation of historical records |
| Base64 export data | No file paths cross the trust boundary; data is self-contained |
| Canonical JSON hashing | Platform-independent determinism for manifest verification |

---

# §3 — Tech Stack

## Rust Core Engine

| Dependency | Version | Purpose |
|------------|---------|---------|
| serde | 1.0 (features: derive) | Serialization/deserialization |
| serde_json | 1.0 | JSON parsing and canonical output |
| sha2 | 0.10 | SHA-256 manifest hashing |
| semver | 1.0 (features: serde) | Template version compatibility |
| thiserror | 1.0 | Typed error handling |
| base64 | 0.21 | Export data encoding |
| chrono | 0.4 (features: serde) | Timestamps |
| uuid | 1.0 (features: v4, serde) | Asset and export IDs |
| clap | 4.0 (features: derive) | CLI argument parsing |
| tempfile | 3.0 (dev only) | Test fixtures |

**Rust edition:** 2024
**MSRV:** Follows Rust 2024 edition requirements

## Python Bridge + Skill

| Dependency | Version | Purpose |
|------------|---------|---------|
| fastapi | >=0.104.0 | HTTP bridge framework |
| uvicorn[standard] | >=0.24.0 | ASGI server |
| pydantic | >=2.5.0 | Request/response validation |
| pydantic-settings | >=2.1.0 | Environment-based configuration |
| httpx | >=0.25.0 | Async HTTP client (skill → bridge) |
| pytest | >=7.4.0 (dev) | Test framework |
| pytest-asyncio | >=0.21.0 (dev) | Async test support |

**Python version:** >=3.10
**Build system:** hatchling

## Build Tools

| Tool | Purpose |
|------|---------|
| cargo | Rust build + test |
| pip / hatch | Python package management |
| uvicorn | Bridge server |
| pytest | Python tests |

## Feature Flags

| Flag | Purpose |
|------|---------|
| `test-hooks` | Enables test hook points in compilation pipeline |

---

# §4 — Project Structure

## Directory Tree

```
ForgeImages/
├── README.md                          # Project overview and quick start
├── CLAUDE.md                          # AI assistant coding standards
├── .gitignore                         # Build artifacts, venvs, audit logs
│
├── doc/                               # BDS Documentation Protocol
│   ├── system/                        # Modular system documentation parts
│   │   ├── _index.md                  # Master table of contents
│   │   ├── 01-overview-philosophy.md  # §1: Six Laws, ecosystem role
│   │   ├── 02-architecture.md         # §2: Trust boundary, data flow
│   │   ├── 03-tech-stack.md           # §3: Dependencies and versions
│   │   ├── 04-project-structure.md    # §4: This file
│   │   ├── 05-config-env.md           # §5: Environment and CLI flags
│   │   ├── 06-api-layer.md            # §6: Bridge endpoints, CLI commands
│   │   ├── 07-backend-internals.md    # §7: Template, validation, hashing, pipeline
│   │   ├── 08-ecosystem-integration.md # §8: ForgeAgents, VibeForge, AuthorForge
│   │   ├── 09-error-handling.md       # §9: 422 contract, exit codes, violations
│   │   ├── 10-testing.md             # §10: Invariant + boundary tests
│   │   ├── 11-handover.md            # §11: Constraints, deployment, future
│   │   └── BUILD.sh                   # Assembles parts into context-bundle.md
│   └── SYSTEM.md                      # Generated: full assembled reference
│
├── scripts/
│   └── context-bundle.sh             # AI session context generator (presets)
│
├── files/                             # Design documentation (pre-implementation)
│   ├── README.md                      # ForgeAgents integration guide
│   ├── CONTEXT.md                     # Trust boundary architecture
│   ├── step1-rust-core.md             # Rust core design spec
│   ├── step2-rust-cli.md              # CLI design spec
│   ├── step3-python-bridge.md         # Bridge service design spec
│   ├── step4-python-skill.md          # Skill wrapper design spec
│   ├── step5-tests.md                 # Test strategy spec
│   └── VSCODE_CLAUDE_PROMPT.md        # Implementation prompt
│
├── forgeimages-core/                  # Rust core engine
│   ├── Cargo.toml                     # Package: forgeimages-core v1.0.0, edition 2024
│   ├── src/
│   │   ├── lib.rs                     # Library root — Six Laws, public API exports
│   │   ├── templates.rs               # Template contracts (TemplateRegistry, AssetClass, ExportSpec)
│   │   ├── validation.rs              # Rule/policy separation (3 rules, Validator, FailureMode)
│   │   ├── hashing.rs                 # SHA-256 manifests (canonical JSON, job_hash, manifest_hash)
│   │   ├── print.rs                   # PrintAuthority enum (System/Template/User, DPI/CMYK)
│   │   ├── pipeline.rs               # CompilationPipeline (compile ALWAYS validates, exports)
│   │   └── bin/
│   │       └── forgeimages_cli.rs     # CLI binary (templates, validate, compile subcommands)
│   ├── templates/
│   │   └── pwa-icon.json              # PWA icon template (1:1, 512px min, 6 PNG + 1 SVG)
│   └── tests/
│       └── invariants.rs              # Contract invariant tests (6 tests)
│
└── forgeagents-forgeimages/           # Python agent integration
    ├── pyproject.toml                 # Package: forgeagents-forgeimages v1.0.0, Python >=3.10
    ├── bridge/                        # FastAPI bridge service
    │   ├── __init__.py                # Package init
    │   ├── forgeimages_bridge.py      # HTTP endpoints (health, templates, validate, compile)
    │   ├── models.py                  # Pydantic v2 models (AssetInput, CompileRequest, etc.)
    │   ├── settings.py                # Configuration (FORGEIMAGES_ env prefix)
    │   └── audit.py                   # Append-only JSONL audit logging
    ├── skill/                         # ForgeAgents skill wrapper
    │   ├── __init__.py                # Package init
    │   └── forgeimages_skill.py       # ForgeImagesSkill class (async httpx client)
    └── tests/
        └── test_agent_boundary.py     # Agent boundary enforcement tests (20+ tests)
```

## Key Module Responsibilities

### Rust Modules

| Module | LOC | Responsibility |
|--------|-----|----------------|
| `lib.rs` | 25 | Six Laws docstring, public API re-exports, version constants |
| `templates.rs` | 176 | Template, AssetClass, ValidationConfig, ExportSpec, TemplateRegistry |
| `validation.rs` | 224 | 3 rules (AspectRatio, Resolution, ColorCount), Validator, FailureMode |
| `hashing.rs` | 102 | SHA-256, canonical JSON (sorted keys), job_hash, manifest_hash |
| `print.rs` | 81 | PrintAuthority enum, PrintSpec, ColorSpace, DPI/bleed validation |
| `pipeline.rs` | 272 | CompilationPipeline, CompileRequest, CompiledAsset, ExportedFile |
| `forgeimages_cli.rs` | 149 | Clap CLI: templates, validate (exit 2), compile (exit 2) |

### Python Modules

| Module | LOC | Responsibility |
|--------|-----|----------------|
| `models.py` | 93 | Pydantic v2 models mirroring Rust types |
| `settings.py` | 33 | Environment-based config (FORGEIMAGES_ prefix) |
| `audit.py` | 105 | Append-only JSONL audit log with job hash linkage |
| `forgeimages_bridge.py` | 272 | FastAPI app with 5 endpoints, CLI subprocess calls |
| `forgeimages_skill.py` | 378 | ForgeImagesSkill class, async httpx, error handling |

---

# §5 — Configuration & Environment

## Bridge Environment Variables

All bridge configuration uses the `FORGEIMAGES_` prefix via pydantic-settings.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `FORGEIMAGES_CLI_PATH` | Path | `../forgeimages-core/target/release/forgeimages-cli` | Path to compiled Rust CLI binary |
| `FORGEIMAGES_TEMPLATES_DIR` | Path | `../forgeimages-core/templates` | Directory containing template JSON files |
| `FORGEIMAGES_AUDIT_LOG_PATH` | Path | `./audit.jsonl` | Audit log output file |
| `FORGEIMAGES_MAX_REQUEST_SIZE_MB` | int | `10` | Maximum HTTP request body size |
| `FORGEIMAGES_MAX_PAYLOAD_SIZE_KB` | int | `512` | Maximum JSON payload size |
| `FORGEIMAGES_ENGINE_VERSION` | str | `"1.0.0"` | Engine version for audit logging |

## CLI Flags

### Global Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--templates-dir` | `templates` | Directory containing template JSON files |

### Validate Subcommand

| Flag | Required | Description |
|------|----------|-------------|
| `--template` | Yes | Template ID to validate against |
| `--payload` | Yes | JSON string of AssetInput |

### Compile Subcommand

| Flag | Required | Description |
|------|----------|-------------|
| `--template` | Yes | Template ID to compile against |
| `--payload` | Yes | JSON string of CompileRequest |

## Template Configuration (pwa-icon.json)

Templates are JSON files in the templates directory. Each template defines:

```json
{
  "id": "pwa-icon",
  "name": "PWA Icon Pack",
  "templateVersion": "1.0.0",
  "engineMinVersion": "1.0.0",
  "assetClass": "icon",
  "aspectRatio": [1, 1],
  "canonicalSize": [1024, 1024],
  "vectorMaster": true,
  "validation": {
    "required": true,
    "failureMode": "block",
    "rules": {
      "aspectRatio": { "enabled": true, "tolerance": 0.01 },
      "resolution": { "enabled": true, "minWidth": 512, "minHeight": 512 },
      "colorCount": { "enabled": true, "maxColors": 16 }
    }
  },
  "exports": [
    { "id": "master", "size": [1024, 1024], "format": "svg", "required": true },
    { "id": "favicon-16", "size": [16, 16], "format": "png", "required": true },
    { "id": "favicon-32", "size": [32, 32], "format": "png", "required": true },
    { "id": "apple-touch", "size": [180, 180], "format": "png", "required": true },
    { "id": "pwa-192", "size": [192, 192], "format": "png", "required": true },
    { "id": "pwa-512", "size": [512, 512], "format": "png", "required": true }
  ]
}
```

## Rust Constants

| Constant | Value | Location |
|----------|-------|----------|
| `ENGINE_VERSION` | `env!("CARGO_PKG_VERSION")` → `"1.0.0"` | `lib.rs` |
| `MIN_TEMPLATE_VERSION` | `"1.0.0"` | `lib.rs` |

## Default Validation Tolerances

| Rule | Default | Description |
|------|---------|-------------|
| Aspect ratio tolerance | 0.01 (1%) | Quantization bucket for ratio comparison |
| Min resolution | 1024x1024 | Default minimum (overridden by template) |
| Max color count | 16 | Default maximum (overridden by template) |

## Print Defaults

| Setting | Default | Valid Range |
|---------|---------|-------------|
| DPI | 300 | 72-1200 |
| Color space | RGB | RGB, CMYK, Grayscale |
| Bleed | 0.125 inches | 0-1 inch |

---

# §6 — API Layer

## Bridge HTTP Endpoints

The bridge service runs on `http://127.0.0.1:8100` (configurable via uvicorn).

### Endpoint Reference

| Method | Endpoint | Purpose | Success | Failure |
|--------|----------|---------|---------|---------|
| GET | `/health` | Health check | 200 | — |
| GET | `/templates` | List all templates | 200 `list[TemplateInfo]` | — |
| GET | `/template/{id}` | Get template details | 200 `dict` | 404 |
| POST | `/validate/{id}` | Validate asset against template | 200 `ValidationResult` | 422 |
| POST | `/compile/{id}` | Compile asset (validates first) | 200 `CompileResponse` | 422 |

### Request/Response Models

**POST /validate/{template_id}**
```json
// Request body (AssetInput)
{
  "width": 512,        // 1-10000
  "height": 512,       // 1-10000
  "color_count": 8,    // optional, 1-256
  "format": "png"      // optional
}

// Success response (ValidationResult)
{
  "valid": true,
  "violations": [],
  "template_id": "pwa-icon",
  "template_version": "1.0.0"
}

// Failure response (HTTP 422)
{
  "valid": false,
  "violations": [
    {
      "rule": "aspect_ratio",
      "severity": "error",
      "message": "Aspect ratio 2.00 exceeds tolerance ...",
      "expected": "1.00",
      "actual": "2.00",
      "remediation": ["Resize to 1:1 aspect ratio"]
    }
  ],
  "template_id": "pwa-icon",
  "template_version": "1.0.0"
}
```

**POST /compile/{template_id}**
```json
// Request body (CompileRequest)
{
  "template_id": "pwa-icon",
  "asset_input": { "width": 512, "height": 512, "color_count": 8 },
  "source_data": "base64...",   // optional
  "seed": 42,                   // optional
  "prompt": "..."               // optional, max 2000 chars
}

// Success response (CompileResponse)
{
  "success": true,
  "asset": {
    "id": "uuid",
    "template_id": "pwa-icon",
    "template_version": "1.0.0",
    "engine_version": "1.0.0",
    "created_at": "2026-01-15T12:00:00Z",
    "manifest_hash": "sha256:...",
    "job_hash": "sha256:...",
    "validation": { "valid": true, "violations": [] },
    "exports": [
      {
        "id": "master",
        "filename": "master.svg",
        "format": "svg",
        "size": [1024, 1024],
        "data_base64": "...",
        "hash": "sha256:..."
      }
    ]
  }
}
```

### Error Semantics

| HTTP Status | Meaning | When |
|-------------|---------|------|
| 200 | Success | Valid asset or successful compilation |
| 404 | Not found | Template ID does not exist |
| 422 | Validation failure | Asset violates template rules (blocking mode) |
| 503 | Service unavailable | Rust CLI binary not found |
| 504 | Timeout | CLI execution exceeded 30 seconds |

### Middleware

- **Request size limiter** — Enforces `max_request_size_mb` (default 10 MB)

### Headers

| Header | Direction | Purpose |
|--------|-----------|---------|
| `X-User-ID` | Request | Optional user identification for audit trail |
| `Content-Type` | Both | `application/json` |

## CLI Subcommands

The Rust CLI is invoked by the bridge as a subprocess. It is also usable directly.

### `templates` — List Templates

```bash
forgeimages-cli templates --templates-dir ./templates
```

Output: JSON array of template summaries.

### `validate` — Validate Asset

```bash
forgeimages-cli validate \
  --template pwa-icon \
  --payload '{"width":512,"height":512,"color_count":8}' \
  --templates-dir ./templates
```

Output: JSON ValidationResult. Exit code 0 = valid, exit code 2 = invalid.

### `compile` — Compile Asset

```bash
forgeimages-cli compile \
  --template pwa-icon \
  --payload '{"template_id":"pwa-icon","asset_input":{"width":512,"height":512,"color_count":8}}' \
  --templates-dir ./templates
```

Output: JSON CompiledAsset. Exit code 0 = success, exit code 2 = validation/compilation failure.

### CLI Exit Code Semantics

| Code | Meaning | Bridge Interpretation |
|------|---------|----------------------|
| 0 | Success | HTTP 200 |
| 1 | System error | HTTP 500 |
| 2 | Validation failure | HTTP 422 |

---

# §7 — Backend Internals

## Template System (`templates.rs`)

### Core Types

**Template** — The contract definition:
- `id: TemplateId` — Unique identifier (e.g., "pwa-icon")
- `name: String` — Human-readable name
- `template_version: Version` — Semver version
- `engine_min_version: Version` — Minimum compatible engine
- `asset_class: AssetClass` — Icon, Cover, Banner, or Logo
- `aspect_ratio: (u32, u32)` — Expected ratio
- `canonical_size: (u32, u32)` — Ideal dimensions
- `vector_master: bool` — Whether SVG master is required
- `validation: ValidationConfig` — Rules and failure mode
- `exports: Vec<ExportSpec>` — Output specifications

**AssetClass enum** — `Icon | Cover | Banner | Logo`

**FailureMode enum** — Determines what happens when validation finds errors:
- `Block` (default) — Errors prevent compilation
- `Warn` — Errors recorded but compilation proceeds
- `Log` — Errors recorded silently

**TemplateRegistry** — HashMap-backed registry:
- `load_from_dir(path)` — Loads all `.json` files from directory
- `get(id)` — Retrieve template by ID
- `list()` — List all registered templates
- `register(template)` — Add a template to the registry

### Template Versioning

Templates use semver. The engine checks compatibility before processing:

```rust
fn check_engine_version(template: &Template) -> Result<(), PipelineError> {
    let engine = Version::parse(ENGINE_VERSION)?;
    if engine < template.engine_min_version {
        return Err(PipelineError::EngineVersionMismatch { ... });
    }
    Ok(())
}
```

## Validation Pipeline (`validation.rs`)

### Rule/Policy Separation

Validation rules produce violations. Failure mode applies policy. These are intentionally separate concerns:

1. **Rules** generate `ValidationViolation` structs with severity, message, expected/actual values, and remediation hints
2. **Validator** collects all violations from all rules, then applies the failure mode policy to determine `valid: bool`

### Three Validation Rules

**AspectRatioRule** — Compares input width:height ratio against template target within configurable tolerance.
```
expected_ratio = template.aspect_ratio.0 / template.aspect_ratio.1
actual_ratio = input.width / input.height
valid = |expected - actual| <= tolerance
```

**ResolutionRule** — Enforces minimum width and height from template config.
```
valid = input.width >= min_width && input.height >= min_height
```

**ColorCountRule** — Enforces maximum color palette. Severity is `Warning` (not Error), so it never blocks in Block mode.
```
valid = input.color_count <= max_colors
```

### Failure Mode Application

| Mode | Error violations | Warning violations | Result |
|------|------------------|--------------------|--------|
| Block | `valid = false` | Recorded, not blocking | Blocks compilation |
| Warn | Recorded | Recorded | Never blocks |
| Log | Recorded | Recorded | Never blocks |

## SHA-256 Hashing (`hashing.rs`)

### Canonical JSON

All JSON is canonicalized before hashing: keys sorted alphabetically, no whitespace, nested objects recursively sorted. This ensures identical hashes across platforms and serialization orders.

```rust
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let sorted = sort_value(&v);  // Recursive key sorting
    serde_json::to_string(&sorted)
}
```

### Hash Functions

**`compute_manifest_hash(value)`** — SHA-256 of canonical JSON representation. Used for content-addressable identification.

**`compute_job_hash(template_id, template_version, payload, engine_version)`** — SHA-256 of concatenated string:
```
"{template_id}:{template_version}:{canonical_payload}:{engine_version}"
```
Links a specific compilation request to its output. Used in audit trail to verify that the same inputs produce the same outputs.

## Print Authority (`print.rs`)

### PrintAuthority Enum

| Variant | DPI | Color Space | Bleed | Use Case |
|---------|-----|-------------|-------|----------|
| System | 300 | RGB | 0.125" | Default fallback |
| Template | Template-defined | Template-defined | Template-defined | Template specifies print requirements |
| User | User-provided | User-provided | User-provided | User override with validation |

### User Validation Constraints

| Property | Range | Error |
|----------|-------|-------|
| DPI | 72-1200 | Out of range |
| Bleed | 0-1 inch | Out of range |

### ColorSpace Enum

`Rgb | Cmyk | Grayscale` — Serialized as UPPERCASE strings.

## Compilation Pipeline (`pipeline.rs`)

### The Critical Invariant

```rust
pub fn compile_asset(&self, request: CompileRequest) -> Result<CompiledAsset, PipelineError> {
    // ALWAYS validate first — this is the Six Laws in code
    let validation = self.validate_asset(&request.template_id, &request.asset_input)?;
    if !validation.valid {
        return Err(PipelineError::ValidationFailed(validation));
    }
    // ... proceed with compilation
}
```

There is no code path that reaches export generation without first passing through validation. This is enforced by the Rust type system and tested in `invariants.rs`.

### Pipeline Steps

1. **Resolve template** — Look up template by ID from registry
2. **Check engine version** — Semver compatibility check
3. **Validate asset** — Run 3 rules, apply failure mode
4. **Generate exports** — Create one ExportedFile per ExportSpec
5. **Compute hashes** — manifest_hash + job_hash
6. **Return CompiledAsset** — Immutable result with all metadata

### ExportedFile

Each export contains:
- `id` — Matches the ExportSpec ID (e.g., "favicon-32")
- `filename` — Generated filename (e.g., "favicon-32.png")
- `format` — ExportFormat enum value
- `size` — Pixel dimensions
- `data_base64` — Base64-encoded file content
- `hash` — SHA-256 of the raw file data

## Audit Logging (`audit.py`)

Append-only JSONL format. Each entry contains:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | ISO 8601 | When the request was processed |
| `request_id` | UUID | Unique request identifier |
| `user_id` | string? | From X-User-ID header |
| `template_id` | string | Which template was used |
| `job_hash` | string | SHA-256 linking input to output |
| `action` | string | "validate" or "compile" |
| `outcome` | string | "success", "validation_failed", or "error" |
| `violations_count` | int | Number of violations found |
| `error_message` | string? | Error details if outcome is "error" |

The audit logger computes `job_hash` using the same algorithm as the Rust engine, ensuring cross-layer linkage.

---

# §8 — Ecosystem Integration

## Integration Map

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  VibeForge   │     │ ForgeAgents  │     │ AuthorForge  │
│ (consumer)   │     │ (orch.)      │     │ (future)     │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │    ┌───────────────┘                    │
       │    │                                    │
       ▼    ▼                                    ▼
┌──────────────────────────────────────────────────────────┐
│              ForgeImages Skill (httpx)                     │
│         forgeagents-forgeimages/skill/                     │
└────────────────────────┬─────────────────────────────────┘
                         │ HTTP
┌────────────────────────▼─────────────────────────────────┐
│              ForgeImages Bridge (FastAPI)                  │
│         forgeagents-forgeimages/bridge/                    │
└────────────────────────┬─────────────────────────────────┘
                         │ subprocess
┌────────────────────────▼─────────────────────────────────┐
│              ForgeImages Core (Rust)                       │
│         forgeimages-core/                                  │
└──────────────────────────────────────────────────────────┘
```

## ForgeAgents Integration (Active)

### Skill Wrapper

The `ForgeImagesSkill` class in `skill/forgeimages_skill.py` provides the agent-facing interface:

```python
skill = ForgeImagesSkill(bridge_url="http://localhost:8100", user_id="agent-1")

# List templates
templates = await skill.list_templates()

# Validate
result = await skill.validate("pwa-icon", width=512, height=512, color_count=8)
if not result.valid:
    hints = result.violations  # Remediation hints included

# Compile
asset = await skill.compile("pwa-icon", width=512, height=512)
export = asset.get_export("pwa-512")
raw_data = asset.get_export_data("pwa-512")  # Decoded bytes

# Convenience: validate then compile
validation, asset = await skill.validate_and_compile("pwa-icon", width=512, height=512)
```

### Agent Boundary Enforcement

| Allowed | Blocked |
|---------|---------|
| `skill.list_templates()` | Direct file writes |
| `skill.validate(...)` | Template modification |
| `skill.compile(...)` | Validation bypass |
| `skill.get_template(...)` | Failure mode override |

### Error Handling

- `ForgeImagesError` with `status_code` and `violations`
- `is_validation_error()` returns `True` for HTTP 422
- `get_remediation_hints()` extracts remediation from violation list
- `validate_and_compile()` returns `(ValidationResult, None)` on validation failure — does NOT raise

### HTTP Headers

The skill includes `X-User-ID` in all requests (if set at construction), enabling audit trail attribution.

## VibeForge Integration (Planned)

VibeForge is the primary consumer of ForgeImages. The planned integration:

- **Prompt-to-image workflows** — VibeForge generates image candidates via AI, ForgeImages validates and compiles them
- **Tauri IPC** — Future Tauri commands will wrap ForgeImages operations for desktop use
- **Template browsing** — VibeForge UI will display available templates and their constraints

**Status:** Not yet implemented. Requires Tauri command integration.

## AuthorForge Integration (Future)

AuthorForge will use ForgeImages for:

- **Book covers** — Cover template with specific aspect ratios and export formats
- **Print PDF** — CMYK color space via PrintAuthority
- **Chapter illustrations** — Banner/icon templates for digital and print
- **Print bleed** — Physical bleed specifications from PrintSpec

**Status:** Not yet implemented. Requires CMYK export pipeline and print templates.

## DataForge Integration

ForgeImages does not write directly to DataForge. The expected data flow:

1. Agent requests compilation via ForgeImages
2. Agent receives compiled asset (base64 + hashes)
3. Agent persists metadata and results to DataForge

This maintains the DataForge source-of-truth contract while keeping ForgeImages stateless.

## Integration Contracts

### Skill → Bridge Contract

| Property | Value |
|----------|-------|
| Protocol | HTTP/1.1 |
| Content-Type | application/json |
| Auth | None (X-User-ID for audit only) |
| Timeout | 30 seconds (configurable) |
| Retry | Not built-in; caller's responsibility |

### Bridge → CLI Contract

| Property | Value |
|----------|-------|
| Interface | Subprocess (stdin/stdout) |
| Input | JSON via `--payload` flag |
| Output | JSON on stdout |
| Exit 0 | Success |
| Exit 2 | Validation/compilation failure |
| Exit 1 | System error |
| Timeout | 30 seconds |

---

# §9 — Error Handling

## The 422 Contract

HTTP 422 (Unprocessable Entity) is the enforcement mechanism. When validation fails with blocking errors, the bridge returns 422 with structured violation data. Agents must handle 422 to proceed — there is no workaround.

```
Agent → POST /compile/pwa-icon → Bridge → CLI (validate) → exit 2
                                 Bridge ← JSON violations
Agent ← 422 + violations ← Bridge
```

The 422 response body is always a `ValidationResult` with `valid: false` and a non-empty `violations` array.

## Rust Error Types (`pipeline.rs`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Validation failed")]
    ValidationFailed(ValidationResult),

    #[error("Engine version {engine} < required {required}")]
    EngineVersionMismatch { engine: String, required: String },

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

## Validation Violations

Each violation contains full diagnostic information:

| Field | Type | Purpose |
|-------|------|---------|
| `rule` | string | Which rule produced this (e.g., "aspect_ratio") |
| `severity` | enum | Error, Warning, or Info |
| `message` | string | Human-readable description |
| `expected` | string | What the template requires |
| `actual` | string | What the input provided |
| `remediation` | string[] | Actionable fix suggestions |

### Severity Semantics

| Severity | Blocks in Block mode | Blocks in Warn mode | Blocks in Log mode |
|----------|---------------------|---------------------|-------------------|
| Error | Yes | No | No |
| Warning | No | No | No |
| Info | No | No | No |

## CLI Exit Codes

| Code | Meaning | Stdout | Bridge Action |
|------|---------|--------|---------------|
| 0 | Success | JSON result | Return 200 |
| 1 | System error | Error message | Return 500 |
| 2 | Validation failure | JSON with violations | Return 422 |

The bridge interprets exit code 2 specifically as a validation failure and parses the stdout JSON for violation details.

## Bridge Error Responses

| HTTP Status | Condition | Response Body |
|-------------|-----------|---------------|
| 404 | Template not found | `{"detail": "Template not found: {id}"}` |
| 422 | Validation failure | `ValidationResult` with violations |
| 503 | CLI binary not found | `{"detail": "ForgeImages CLI not available"}` |
| 504 | CLI timeout | `{"detail": "CLI execution timed out"}` |

## Python Exceptions

### Skill-Level Errors

```python
class ForgeImagesError(Exception):
    """Raised when bridge returns non-2xx status."""
    status_code: int
    violations: list[dict]

    def is_validation_error(self) -> bool:
        """True if HTTP 422 (validation failure)."""
        return self.status_code == 422

    def get_remediation_hints(self) -> list[str]:
        """Extract remediation from all violations."""
```

### Error Flow

```
compile() called
  → httpx.post("/compile/{id}")
    → HTTP 422 returned
      → Parse violations from response
      → Raise ForgeImagesError(status_code=422, violations=[...])

validate_and_compile() called
  → validate() called first
    → HTTP 422 returned
      → Return (ValidationResult(valid=False, ...), None)
      → Does NOT raise — caller checks result.valid
```

The `validate_and_compile()` convenience method intentionally does not raise on validation failure. It returns `(result, None)` so agents can inspect violations and decide how to respond.

## Pydantic Validation (Bridge Input)

The bridge validates all inputs before calling the CLI:

| Constraint | Field | Rule |
|------------|-------|------|
| Dimension range | width, height | 1-10000 |
| Color count range | color_count | 1-256 (optional) |
| Template ID format | template_id | `^[a-zA-Z0-9_-]+$` (no path traversal) |
| Prompt length | prompt | Max 2000 characters |
| Seed value | seed | Non-negative integer |

Invalid input is rejected at the Pydantic layer before the CLI is ever invoked.
