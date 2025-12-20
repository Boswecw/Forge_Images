"""
ForgeImages Bridge - HTTP interface between ForgeAgents and ForgeImages.

This module provides the FastAPI bridge service that enforces
the "Agents suggest, ForgeImages enforces" constraint.
"""

from .models import (
    AssetInput,
    CompileRequest,
    CompileResponse,
    ValidationResult,
    ValidationViolation,
    CompiledAsset,
    ExportedFile,
    TemplateInfo,
)
from .forgeimages_bridge import app
from .audit import AuditLogger, AuditEntry
from .settings import settings, Settings

__all__ = [
    # Models
    "AssetInput",
    "CompileRequest",
    "CompileResponse",
    "ValidationResult",
    "ValidationViolation",
    "CompiledAsset",
    "ExportedFile",
    "TemplateInfo",
    # App
    "app",
    # Audit
    "AuditLogger",
    "AuditEntry",
    # Settings
    "settings",
    "Settings",
]
