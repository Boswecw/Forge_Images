# Step 3: Python FastAPI Bridge

The bridge is the HTTP gateway between ForgeAgents and ForgeImages.

## Directory Structure

```
forgeagents-forgeimages/
├── pyproject.toml
├── bridge/
│   ├── __init__.py
│   ├── forgeimages_bridge.py
│   ├── models.py
│   ├── audit.py
│   └── settings.py
```

## pyproject.toml

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "forgeagents-forgeimages"
version = "1.0.0"
description = "ForgeAgents integration for ForgeImages"
requires-python = ">=3.10"
license = "Proprietary"
authors = [{ name = "Boswell Digital Solutions LLC" }]

dependencies = [
    "fastapi>=0.104.0",
    "uvicorn[standard]>=0.24.0",
    "pydantic>=2.5.0",
    "pydantic-settings>=2.0.0",
    "httpx>=0.25.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.4.0",
    "pytest-asyncio>=0.21.0",
]

[tool.hatch.build.targets.wheel]
packages = ["bridge", "skill"]

[tool.pytest.ini_options]
asyncio_mode = "auto"
testpaths = ["tests"]
```

## bridge/models.py

```python
"""Pydantic models for the ForgeImages bridge."""

from datetime import datetime
from typing import Optional
from pydantic import BaseModel, Field, field_validator
import re


class AssetInput(BaseModel):
    """Input for asset validation/compilation."""
    width: int = Field(..., ge=1, le=10000)
    height: int = Field(..., ge=1, le=10000)
    color_count: Optional[int] = Field(None, ge=1, le=256)
    format: Optional[str] = None


class CompileRequest(BaseModel):
    """Request to compile an asset."""
    template_id: str = Field(..., min_length=1, max_length=64)
    asset_input: AssetInput
    source_data: Optional[str] = None  # Base64
    seed: Optional[int] = Field(None, ge=0)
    prompt: Optional[str] = Field(None, max_length=2000)

    @field_validator('template_id')
    @classmethod
    def validate_template_id(cls, v: str) -> str:
        if not re.match(r'^[a-z0-9][a-z0-9-]*$', v):
            raise ValueError('Invalid template_id format')
        return v


class ValidationViolation(BaseModel):
    """A single validation violation."""
    rule: str
    severity: str
    message: str
    expected: Optional[str] = None
    actual: Optional[str] = None
    remediation: list[str] = []


class ValidationResult(BaseModel):
    """Result of validation."""
    valid: bool
    violations: list[ValidationViolation] = []
    template_id: str
    template_version: str


class ExportedFile(BaseModel):
    """An exported file from compilation."""
    id: str
    filename: str
    format: str
    size: list[int]
    data_base64: str
    hash: str


class CompiledAsset(BaseModel):
    """A compiled asset with all exports."""
    id: str
    template_id: str
    template_version: str
    engine_version: str
    created_at: datetime
    manifest_hash: str
    job_hash: str
    validation: ValidationResult
    exports: list[ExportedFile]


class CompileResponse(BaseModel):
    """Response from compile endpoint."""
    success: bool
    asset: Optional[CompiledAsset] = None
    error: Optional[str] = None


class TemplateInfo(BaseModel):
    """Template summary info."""
    id: str
    name: str
    version: str
    asset_class: str
    deprecated: bool = False
```

## bridge/settings.py

```python
"""Bridge configuration settings."""

from pathlib import Path
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Bridge configuration."""
    
    cli_path: Path = Path("../forgeimages-core/target/release/forgeimages-cli")
    templates_dir: Path = Path("../forgeimages-core/templates")
    audit_log_path: Path = Path("./audit.jsonl")
    max_request_size_mb: int = 10
    max_payload_size_kb: int = 512
    engine_version: str = "1.0.0"
    
    class Config:
        env_prefix = "FORGEIMAGES_"


settings = Settings()
```

## bridge/audit.py

```python
"""Audit logging for ForgeImages bridge."""

import json
import hashlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional, Any
from pydantic import BaseModel


class AuditEntry(BaseModel):
    """Single audit log entry."""
    timestamp: str
    request_id: str
    user_id: Optional[str] = None
    template_id: str
    job_hash: str
    action: str
    outcome: str
    violations_count: int = 0
    error_message: Optional[str] = None


class AuditLogger:
    """Append-only JSONL audit logger."""
    
    def __init__(self, log_path: Path):
        self.log_path = log_path
        self.log_path.parent.mkdir(parents=True, exist_ok=True)
    
    def compute_job_hash(
        self,
        template_id: str,
        template_version: str,
        payload: Any,
        engine_version: str,
    ) -> str:
        payload_json = json.dumps(payload, sort_keys=True, separators=(',', ':'))
        combined = f"{template_id}:{template_version}:{payload_json}:{engine_version}"
        return hashlib.sha256(combined.encode()).hexdigest()
    
    def log(self, entry: AuditEntry) -> None:
        with open(self.log_path, 'a') as f:
            f.write(entry.model_dump_json() + '\n')
    
    def log_validate(
        self,
        request_id: str,
        template_id: str,
        job_hash: str,
        valid: bool,
        violations_count: int,
        user_id: Optional[str] = None,
    ) -> None:
        entry = AuditEntry(
            timestamp=datetime.now(timezone.utc).isoformat(),
            request_id=request_id,
            user_id=user_id,
            template_id=template_id,
            job_hash=job_hash,
            action="validate",
            outcome="success" if valid else "validation_failed",
            violations_count=violations_count,
        )
        self.log(entry)
    
    def log_compile(
        self,
        request_id: str,
        template_id: str,
        job_hash: str,
        success: bool,
        violations_count: int = 0,
        error_message: Optional[str] = None,
        user_id: Optional[str] = None,
    ) -> None:
        if success:
            outcome = "success"
        elif error_message and "Validation" in error_message:
            outcome = "validation_failed"
        else:
            outcome = "error"
        
        entry = AuditEntry(
            timestamp=datetime.now(timezone.utc).isoformat(),
            request_id=request_id,
            user_id=user_id,
            template_id=template_id,
            job_hash=job_hash,
            action="compile",
            outcome=outcome,
            violations_count=violations_count,
            error_message=error_message,
        )
        self.log(entry)
```

## bridge/forgeimages_bridge.py

```python
"""
ForgeImages FastAPI Bridge Service.

This bridge is the ONLY interface between ForgeAgents and ForgeImages.
It enforces HTTP 422 on validation failure.
"""

import json
import subprocess
import uuid
from typing import Optional

from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse

from .models import (
    AssetInput,
    CompileRequest,
    ValidationResult,
    TemplateInfo,
)
from .audit import AuditLogger
from .settings import settings


app = FastAPI(
    title="ForgeImages Bridge",
    description="Bridge service for ForgeAgents to access ForgeImages",
    version="1.0.0",
)

audit = AuditLogger(settings.audit_log_path)


def call_cli(command: list[str]) -> tuple[int, str]:
    """Call the Rust CLI and return (exit_code, stdout)."""
    full_command = [
        str(settings.cli_path),
        "--templates-dir", str(settings.templates_dir),
        *command
    ]
    
    try:
        result = subprocess.run(
            full_command,
            capture_output=True,
            text=True,
            timeout=30,
        )
        return result.returncode, result.stdout
    except FileNotFoundError:
        raise HTTPException(
            status_code=503,
            detail="ForgeImages CLI not found. Build with: cargo build --release"
        )
    except subprocess.TimeoutExpired:
        raise HTTPException(status_code=504, detail="CLI timeout")


@app.middleware("http")
async def limit_request_size(request: Request, call_next):
    """Limit request body size."""
    content_length = request.headers.get("content-length")
    if content_length:
        if int(content_length) > settings.max_request_size_mb * 1024 * 1024:
            return JSONResponse(status_code=413, content={"detail": "Request too large"})
    return await call_next(request)


@app.get("/health")
async def health():
    return {"status": "ok", "service": "forgeimages-bridge"}


@app.get("/templates", response_model=list[TemplateInfo])
async def list_templates():
    exit_code, output = call_cli(["templates"])
    if exit_code != 0:
        raise HTTPException(status_code=500, detail="Failed to list templates")
    return json.loads(output)


@app.post("/validate", response_model=ValidationResult)
async def validate_asset(
    template_id: str,
    asset_input: AssetInput,
    user_id: Optional[str] = None,
):
    request_id = str(uuid.uuid4())
    payload = asset_input.model_dump(exclude_none=True)
    payload_json = json.dumps(payload)
    
    job_hash = audit.compute_job_hash(
        template_id=template_id,
        template_version="unknown",
        payload=payload,
        engine_version=settings.engine_version,
    )
    
    exit_code, output = call_cli([
        "validate",
        "--template", template_id,
        "--payload", payload_json,
    ])
    
    result = json.loads(output)
    validation = ValidationResult(**result)
    
    audit.log_validate(
        request_id=request_id,
        template_id=template_id,
        job_hash=job_hash,
        valid=validation.valid,
        violations_count=len(validation.violations),
        user_id=user_id,
    )
    
    return validation


@app.post("/compile")
async def compile_asset(
    request: CompileRequest,
    user_id: Optional[str] = None,
):
    """
    Compile an asset. Returns 422 on validation failure.
    
    CRITICAL: This endpoint NEVER bypasses validation.
    """
    request_id = str(uuid.uuid4())
    payload = request.model_dump(exclude_none=True)
    payload_json = json.dumps(payload)
    
    if len(payload_json) > settings.max_payload_size_kb * 1024:
        raise HTTPException(status_code=413, detail="Payload too large")
    
    job_hash = audit.compute_job_hash(
        template_id=request.template_id,
        template_version="unknown",
        payload=payload,
        engine_version=settings.engine_version,
    )
    
    exit_code, output = call_cli([
        "compile",
        "--template", request.template_id,
        "--payload", payload_json,
    ])
    
    try:
        result = json.loads(output)
    except json.JSONDecodeError:
        audit.log_compile(
            request_id=request_id,
            template_id=request.template_id,
            job_hash=job_hash,
            success=False,
            error_message="Invalid CLI response",
            user_id=user_id,
        )
        raise HTTPException(status_code=500, detail="Invalid CLI response")
    
    if result.get("success"):
        audit.log_compile(
            request_id=request_id,
            template_id=request.template_id,
            job_hash=job_hash,
            success=True,
            user_id=user_id,
        )
        return result
    
    error = result.get("error", "Unknown error")
    
    audit.log_compile(
        request_id=request_id,
        template_id=request.template_id,
        job_hash=job_hash,
        success=False,
        error_message=error,
        user_id=user_id,
    )
    
    # 422 for validation failure
    if "Validation failed" in error or exit_code == 2:
        raise HTTPException(
            status_code=422,
            detail={
                "error": "Validation failed",
                "message": error,
                "job_hash": job_hash,
            }
        )
    
    raise HTTPException(status_code=500, detail=error)
```

## bridge/__init__.py

```python
"""ForgeImages Bridge Service."""
```

## Run Command

```bash
cd forgeagents-forgeimages
pip install -e .
uvicorn bridge.forgeimages_bridge:app --reload --port 8789
```
