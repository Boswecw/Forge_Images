# ForgeImages Integration Context

## Architecture Overview

This integration connects two systems with a critical trust boundary:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TRUST BOUNDARY                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐          ┌──────────────┐          ┌────────────┐ │
│  │ ForgeAgents  │  ───▶    │   Bridge     │  ───▶    │ ForgeImages│ │
│  │   (Python)   │          │  (FastAPI)   │          │   (Rust)   │ │
│  │              │          │              │          │            │ │
│  │  • Creative  │          │  • HTTP 422  │          │  • Validate│ │
│  │  • Suggestive│          │  • Audit Log │          │  • Compile │ │
│  │  • No Files  │          │  • Size Limit│          │  • ENFORCE │ │
│  └──────────────┘          └──────────────┘          └────────────┘ │
│                                                                      │
│        CAN suggest            MUST reject              IS truth     │
│        CANNOT bypass          on invalid               NO bypass    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## The Core Constraint

**"Agents suggest, ForgeImages enforces."**

This is not a guideline. It is an architectural invariant enforced by code.

## What Each Layer Does

### ForgeAgents (Python) - The Creative Layer
- Generates image candidates using AI
- Selects candidates for compilation
- Requests templates and validation
- **CANNOT** skip validation
- **CANNOT** write files directly
- **CANNOT** override template constraints

### Bridge (FastAPI) - The Gateway
- HTTP interface between Python and Rust
- Returns **HTTP 422** on validation failure
- Writes append-only audit logs
- Enforces request size limits
- **CANNOT** bypass Rust validation
- **CANNOT** write compiled files

### ForgeImages (Rust) - The Enforcement Point
- Single source of truth for validation
- Deterministic compilation
- Template contract enforcement
- **ALWAYS** validates before compiling
- **NEVER** skips rules
- **NEVER** produces partial outputs

## The 422 Contract

When validation fails, the bridge returns HTTP 422 Unprocessable Entity:

```json
{
  "detail": {
    "error": "Validation failed",
    "message": "aspect_ratio: Aspect ratio mismatch",
    "job_hash": "abc123..."
  }
}
```

This is the enforcement mechanism. Agents cannot proceed past a 422.

## Key Invariants

### 1. Validation Cannot Be Bypassed
```
compile_asset() ALWAYS calls validate_asset() internally
There is no code path that skips validation
```

### 2. Templates Are Contracts
```
Old templates work forever
New engines don't silently change behavior
Version mismatch = clear error, not degradation
```

### 3. Deterministic Output
```
Same inputs = same outputs
Manifest hash is stable and reproducible
Cross-platform guarantee
```

### 4. No Direct File Access
```
Agents receive base64 data, not file paths
Bridge never writes files
Only Rust compilation produces file data
```

### 5. Audit Trail
```
Every request is logged
Job hash links requests to outputs
Append-only JSONL format
```

## Security Model

| Attack Vector | Mitigation |
|---------------|------------|
| Skip validation | Rust enforces internally |
| Override template | Pydantic rejects extra fields |
| Path injection | Template IDs are validated |
| Large payloads | Size limits enforced |
| Bypass bridge | No other entry point exists |

## Test Proof

The boundary tests in `test_agent_boundary.py` prove:

1. Invalid payload → 422 (not 200)
2. Override fields → ignored, still 422
3. Malicious IDs → rejected
4. No file-write endpoints exist
5. Skill has no file-writing methods

If any test fails, the integration is broken.

## Why This Architecture

### For Compliance
- Audit trail proves what happened
- Manifest hash proves output integrity
- Job hash links input to output

### For Quality
- Invalid assets never compile
- Professional rules enforced automatically
- No "close enough" outputs

### For Trust
- Agents can be creative freely
- System catches mistakes at the boundary
- Users get correct results every time
