"""
Audit logging for ForgeImages bridge.

Writes append-only JSONL for compliance and debugging.
"""

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
    action: str  # "validate" | "compile"
    outcome: str  # "success" | "validation_failed" | "error"
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
        """Compute job hash for audit trail."""
        # Canonical JSON
        payload_json = json.dumps(payload, sort_keys=True, separators=(',', ':'))
        combined = f"{template_id}:{template_version}:{payload_json}:{engine_version}"
        return hashlib.sha256(combined.encode()).hexdigest()

    def log(self, entry: AuditEntry) -> None:
        """Append entry to audit log."""
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
        """Log a validation request."""
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
        """Log a compilation request."""
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
